//! Command-line syntax and global options for `nodesea`.

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "nodesea", about = "NodeSea command-line client")]
/// Parsed top-level command-line arguments.
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Args)]
/// Options shared by every CLI command.
pub struct GlobalArgs {
    /// Optional path to the daemon's local IPC socket.
    #[arg(long, env = "NODESEA_SOCKET", value_name = "PATH")]
    pub socket: Option<PathBuf>,

    /// Serialization format for command output.
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    pub output: OutputFormat,
}

/// Selects the embedded catalog matching the operating system locale.
pub fn apply_system_locale() {
    let locale = sys_locale::get_locale();
    let catalog = locale.as_deref().map(catalog_for_locale).unwrap_or("en");
    rust_i18n::set_locale(catalog);
}

/// Maps a system BCP 47/POSIX locale to one of the embedded catalogs.
fn catalog_for_locale(locale: &str) -> &'static str {
    // POSIX locale names may append an encoding or modifier, for example
    // `zh_CN.UTF-8` or `zh_CN@calendar`; these are not locale components.
    let normalized = locale
        .split(['.', '@'])
        .next()
        .unwrap_or(locale)
        .replace('_', "-")
        .to_ascii_lowercase();
    let mut components = normalized.split('-');
    let language = components.next().unwrap_or_default();

    if language == "zh" {
        let is_simplified =
            components.any(|component| matches!(component, "cn" | "hans" | "sg" | "my"));
        if is_simplified {
            return "zh-CN";
        }
    }

    if language == "en" {
        return "en";
    }

    "en"
}

#[derive(Debug, Clone, Copy, ValueEnum)]
/// Output formats supported by CLI commands.
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug, Subcommand)]
/// Commands exposed by the NodeSea CLI.
pub enum Command {
    /// Show the current daemon engine status.
    Status,
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command, OutputFormat, catalog_for_locale};

    #[test]
    fn parses_status_with_default_options() {
        let cli = Cli::try_parse_from(["nodesea", "status"]).unwrap();

        assert!(matches!(cli.command, Command::Status));
        assert!(matches!(cli.global.output, OutputFormat::Text));
    }

    #[test]
    fn parses_json_options() {
        let cli = Cli::try_parse_from(["nodesea", "--output", "json", "status"]).unwrap();

        assert!(matches!(cli.global.output, OutputFormat::Json));
    }

    #[test]
    fn maps_system_locales_to_embedded_catalogs() {
        assert_eq!(catalog_for_locale("en_US.UTF-8"), "en");
        assert_eq!(catalog_for_locale("zh_CN.UTF-8"), "zh-CN");
        assert_eq!(catalog_for_locale("zh-Hans-CN"), "zh-CN");
        assert_eq!(catalog_for_locale("zh-TW"), "en");
        assert_eq!(catalog_for_locale("de-DE"), "en");
    }
}
