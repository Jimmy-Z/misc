use std::{collections::HashMap, fs::read_to_string, process::Command, str::FromStr};

use time::{
	OffsetDateTime, format_description::StaticFormatDescription, macros::format_description,
};
use toml::Table;

const FMT: StaticFormatDescription = format_description!(
	"[year]-[month]-[day] [hour]:[minute]:[second] UTC[offset_hour sign:mandatory]:[offset_minute]"
);

fn run(cmd: &str, args: &[&str]) -> Option<String> {
	let output = Command::new(cmd)
		.args(args)
		// .current_dir(env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()))
		.output()
		.ok()?;

	if !output.status.success() {
		return None;
	}

	String::from_utf8(output.stdout).ok()
}

fn git_stuff(out: &mut String) {
	match run("git", &["branch", "--show-current"]) {
		Some(b) => out.push_str(b.trim_ascii()),
		None => {
			out.push_str("unknown");
			return;
		}
	}

	if let Some(rev) = run("git", &["rev-parse", "--short", "HEAD"]) {
		out.push('-');
		out.push_str(rev.trim_ascii());
	}

	if let Some(status) = run("git", &["status", "--short", "--porcelain"])
		&& !status.is_empty()
	{
		out.push_str(" dirty!");
	}
}

fn rev(deps: &[&str]) -> String {
	let mut rev = String::with_capacity(0x100);

	git_stuff(&mut rev);

	// to do: std::net::hostname is still nightly only
	let now = OffsetDateTime::now_local().unwrap();
	rev.push(' ');
	rev.push_str(&now.format(&FMT).unwrap());

	if !deps.is_empty() {
		// to do: detect workspace
		let lock = read_to_string("Cargo.lock")
			.unwrap_or_else(|_| read_to_string("../Cargo.lock").unwrap());
		let lock = Table::from_str(&lock).unwrap();
		let mut vers = HashMap::with_capacity(deps.len());
		for pkg in lock["package"].as_array().unwrap() {
			let pkg = pkg.as_table().unwrap();
			let name = pkg["name"].as_str().unwrap();
			if deps.contains(&name) {
				vers.insert(name, pkg["version"].as_str().unwrap());
			}
		}
		for dep in deps {
			rev.push_str(&format!(", {dep} {}", vers.get(dep).unwrap()));
		}
	}

	rev
}

// at compile time, set env rev
pub fn comp_time_env_rev(deps: &[&str]) {
	println!("cargo::rustc-env=REV={}", rev(deps));
}

#[test]
fn test_rev() {
	eprintln!("{}", rev(&["toml", "time"]));
}
