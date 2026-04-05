use crate::cli::GlobalOpts;
use std::io::IsTerminal;
use tracing_subscriber::EnvFilter;

pub fn init(opts: &GlobalOpts) {
    let filter = if opts.verbose {
        EnvFilter::new("debug")
    } else if opts.quiet {
        EnvFilter::new("error")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    let use_ansi = match opts.ansi.as_str() {
        "never" => false,
        "always" => true,
        _ => std::io::stderr().is_terminal(),
    };

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(use_ansi)
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();
}
