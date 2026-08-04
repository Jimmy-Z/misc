use std::{
	f32,
	fmt::{Debug, Display, Formatter, Result},
	time::Duration,
};

pub struct Pretty<T>(pub T);

impl Display for Pretty<usize> {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		if self.0 < 1000 {
			write!(f, "{} ", self.0)
		} else {
			pretty_f32(self.0 as f32, f)
		}
	}
}

impl Display for Pretty<f32> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		pretty_f32(self.0, f)
	}
}

// print if it's printable ascii, otherwise hexdump
impl Display for Pretty<&[u8]> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		pretty_bytes(self.0, f)
	}
}

// always hexdump, if it gets too long, break into multiple lines and add ruler
impl Debug for Pretty<&[u8]> {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		debug_bytes(self.0, f)
	}
}

impl Display for Pretty<Duration> {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		pretty_duration(self.0.as_secs_f32(), f)
	}
}

// supports only a part of SI
// f32 is capable of e+/-38, si prefixes simply can't cover them anyway
const SI_PREFIXES: &[&str] = &[
	"f", "p", "n", "μ", "m", "", "k", "M", "G", "T", "P", "E", "Z",
];
const SI_BASE: i32 = 5;
const SI_MIN: i32 = -SI_BASE * 3;
const SI_MAX: i32 = (SI_PREFIXES.len() as i32 - 1 - SI_BASE) * 3;
// PartialEq is not const so this unfortunately doesn't work
// const _SI_CHK: () = assert!(SI_PREFIXES[SI_PREFIX_BASE as usize] == "");

fn pretty_f32(v: f32, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	// to do: infinity and more?
	if v.is_nan() {
		return write!(f, "NaN ");
	}

	let (mut v, sign) = if v >= 0.0 { (v, "") } else { (-v, "-") };

	let mut i = v.log10().floor() as i32;
	// eprintln!("exp: {i}");

	if !(SI_MIN..SI_MAX + 3).contains(&i) {
		v /= 10f32.powi(i);
		return write!(f, "{sign}{v:.2}e{i} ");
	}

	i = (i + SI_BASE * 3) / 3 - SI_BASE;
	v /= 10f32.powi(i * 3);
	// always 3 significants
	let si = SI_PREFIXES[(i + SI_BASE) as usize];
	if v >= 100.0 {
		write!(f, "{sign}{:.0} {si}", v)
	} else if v >= 10.0 {
		write!(f, "{sign}{:.1} {si}", v)
	} else {
		write!(f, "{sign}{:.2} {si}", v)
	}
}

fn pretty_bytes(v: &[u8], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	if v.iter().all(printable) {
		return write!(f, "{}", unsafe { str::from_utf8_unchecked(v) });
	}
	debug_bytes_plain(v, f)
}

fn printable(b: &u8) -> bool {
	matches!(b, b' '..=b'~')
}

const DEBUG_BYTES_LEN: usize = 16;

fn debug_bytes(v: &[u8], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	if v.len() < DEBUG_BYTES_LEN {
		return debug_bytes_plain(v, f);
	}
	// ruler, maybe like every few lines?
	write!(
		f,
		"     0  1  2  3  4  5  6  7  8  9  a  b  c  d  e  f 0123456789abcdef"
	)?;
	let full_lines = v.len() / DEBUG_BYTES_LEN;
	for i in 0..full_lines {
		write!(f, "\n{i:3} ")?;
		debug_bytes_plain(&v[DEBUG_BYTES_LEN * i..DEBUG_BYTES_LEN * (i + 1)], f)?;
	}
	let reminder = v.len() % DEBUG_BYTES_LEN;
	if reminder > 0 {
		write!(f, "\n{:3} ", full_lines)?;
		debug_bytes_with_padding(&v[v.len() - reminder..], f)?;
	}
	Ok(())
}

fn debug_bytes_plain(v: &[u8], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	for &b in v {
		write!(f, "{:02x} ", b)?;
	}
	for &b in v {
		write!(f, "{}", if printable(&b) { char::from(b) } else { '·' })?;
	}
	Ok(())
}

fn debug_bytes_with_padding(v: &[u8], f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	for &b in v {
		write!(f, "{:02x} ", b)?;
	}
	for _ in 0..(DEBUG_BYTES_LEN - v.len()) {
		write!(f, "   ")?;
	}
	for &b in v {
		write!(f, "{}", if printable(&b) { char::from(b) } else { '·' })?;
	}
	Ok(())
}

const TIME_STEPS: &[(f32, &str, &str)] = &[
	(60.0, "s", "s"),
	(60.0, "minute", "minutes"),
	(24.0, "hour", "hours"),
	(7.0, "day", "days"),
	(356.25 / 12.0 / 7.0, "week", "weeks"),
	(12.0, "month", "months"),
	(10.0, "year", "years"),
	(10.0, "decade", "decades"),
	(10.0, "century", "centuries"),
	(f32::INFINITY, "millennium", "millenniums"),
];

fn pretty_duration(mut v: f32, f: &mut Formatter<'_>) -> Result {
	for unit in TIME_STEPS {
		if v < unit.0 {
			return write!(f, "{}{}", Pretty(v), unit.2);
		}
		v /= unit.0;
	}
	unreachable!()
}

#[cfg(test)]
mod tests {
	use std::{collections::HashMap, f32};

	use super::*;

	#[test]
	fn test_f32() {
		assert_eq!(SI_PREFIXES[SI_BASE as usize], "");
		eprintln!("SI_MIN = {SI_MIN}");
		eprintln!("SI_MAX = {SI_MAX}");

		let tests: &[(f32, &str)] = &[
			(f32::consts::PI, "3.14 "),
			(42.0, "42.0 "),
			(1984.0, "1.98 k"),
			(0.618, "618 m"),
			(999.0 * 10f32.powi(SI_MAX), "999 Z"),
			(999.6 * 10f32.powi(SI_MAX), "1000 Z"), // rust print rounds to nearest
			(1000.0 * 10f32.powi(SI_MAX), "1.00e24 "),
			(10f32.powi(SI_MIN), "1.00 f"),
			(0.999 * 10f32.powi(SI_MIN), "9.99e-16 "),
		];

		for &(v, s) in tests {
			let p = format!("{}", Pretty(v));
			eprintln!("{v}, expect {s}, got {p}");
			assert_eq!(s, p);
		}

		const TEST_MIN: i32 = -20;
		const TEST_MAX: i32 = 30;
		// also inspect these by eye
		for b in [1f32, 999f32] {
			let mut counter = HashMap::<String, usize>::new();
			for i in TEST_MIN..=TEST_MAX {
				let v = b * 10f32.powi(i);
				let p = format!("{}", Pretty(v));
				eprintln!("{v} -> {}", p);
				let sig: String = p
					.chars()
					.filter(|c| !matches!(c, '0'..='9' | '.' | '-' | ' '))
					.collect();
				assert!(SI_PREFIXES.contains(&sig.as_str()) || sig == "e");
				*counter.entry(sig).or_default() += 1;
			}
			for &sig in SI_PREFIXES {
				// eprintln!("sig: {sig}");
				assert!(counter[sig] == 3);
				assert!(counter["e"] == (TEST_MAX - TEST_MIN) as usize + 1 - SI_PREFIXES.len() * 3);
			}
		}
	}
	#[test]
	fn test_bytes() {
		let tests: &[(&[u8], &str)] = &[(b"hello", "hello"), (&[0x2, 5, 0, 1], "02 05 00 01 ····")];
		for &(b, expect) in tests {
			eprintln!("{} - {0:?}", Pretty(b));
			let got = format!("{}", Pretty(b));
			assert_eq!(expect, got);
		}
	}

	#[test]
	fn test_duration() {
		for p in -20..25 {
			let d = Duration::from_secs_f32(f32::consts::PI.powi(p));
			eprintln!("{:?} {}", d, Pretty(d));
		}
	}
}
