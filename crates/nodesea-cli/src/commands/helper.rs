use std::{
    env,
    io::{self, IsTerminal},
};

const NO_COLOR_ENV: &str = "NO_COLOR";

/// Returns whether ANSI color output should be emitted.
///
/// Color is enabled only for an interactive stdout. This keeps redirected
/// output and machine-readable consumers free of terminal escape sequences.
/// The `NO_COLOR` environment variable provides the standard user opt-out.
///
/// # Examples
/// ```
/// use nodesea_cli::commands::color_enabled;
/// if color_enabled() {
///    println!("Color output is enabled.");
/// } else {
///    println!("Color output is disabled.");
/// }
/// ```
///
/// # Returns
/// - `true` if color output is enabled.
/// - `false` if color output is disabled.
pub(crate) fn color_enabled() -> bool {
    io::stdout().is_terminal() && env::var_os(NO_COLOR_ENV).is_none()
}
