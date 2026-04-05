use clap::Args;

use crate::cli::GlobalOpts;
use crate::driver::apptainer::Apptainer;
use crate::error::Result;

#[derive(Args)]
pub struct VersionArgs {
    /// Format the output
    #[arg(long)]
    pub format: Option<String>,

    /// Shows only Compose's version number
    #[arg(long)]
    pub short: bool,
}

pub async fn run(_global: GlobalOpts, args: VersionArgs) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");

    if args.short {
        println!("{version}");
        return Ok(());
    }

    let apptainer_version = match Apptainer::detect() {
        Ok(a) => a.version().await.unwrap_or_else(|_| "unknown".to_string()),
        Err(_) => "not found".to_string(),
    };

    if args.format.as_deref() == Some("json") {
        println!(
            r#"{{"apptainerCompose": "{version}", "apptainer": "{apptainer_version}"}}"#
        );
    } else {
        println!("apptainer-compose version {version}");
        println!("apptainer version {apptainer_version}");
    }

    Ok(())
}
