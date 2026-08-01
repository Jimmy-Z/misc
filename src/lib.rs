pub mod build;
pub mod pretty;

pub use build::comp_time_env_rev;
pub use pretty::Pretty;

#[cfg(debug_assertions)]
const LOG_LEVEL: &str = "debug";
#[cfg(not(debug_assertions))]
const LOG_LEVEL: &str = "info";

pub fn init_env_logger() {
	env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(LOG_LEVEL)).init();
}
