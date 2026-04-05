use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::error::Result;

#[derive(Args)]
pub struct ConfigArgs {
    /// Services to show
    pub services: Vec<String>,

    /// Format the output
    #[arg(long, default_value = "yaml")]
    pub format: String,

    /// Only validate the configuration, don't print anything
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Print the service names, one per line
    #[arg(long)]
    pub services_flag: bool,

    /// Print the volume names, one per line
    #[arg(long)]
    pub volumes: bool,

    /// Print the network names, one per line
    #[arg(long)]
    pub networks: bool,

    /// Print the profile names, one per line
    #[arg(long)]
    pub profiles: bool,

    /// Print the image names, one per line
    #[arg(long)]
    pub images: bool,

    /// Print the service config hash, one per line
    #[arg(long)]
    pub hash: Option<String>,

    /// Save to file
    #[arg(short = 'o', long)]
    pub output: Option<String>,

    /// Don't resolve service env files
    #[arg(long)]
    pub no_env_resolution: bool,

    /// Don't interpolate environment variables
    #[arg(long)]
    pub no_interpolate: bool,

    /// Don't normalize compose model
    #[arg(long)]
    pub no_normalize: bool,

    /// Don't resolve file paths
    #[arg(long)]
    pub no_path_resolution: bool,

    /// Don't check model consistency
    #[arg(long)]
    pub no_consistency: bool,

    /// Pin image tags to digests
    #[arg(long)]
    pub resolve_image_digests: bool,

    /// Produces an override file with image digests
    #[arg(long)]
    pub lock_image_digests: bool,

    /// Print environment used for interpolation
    #[arg(long)]
    pub environment: bool,

    /// Print model variables and default values
    #[arg(long)]
    pub variables: bool,
}

pub async fn run(global: GlobalOpts, args: ConfigArgs) -> Result<()> {
    let compose = load_compose(&global)?;

    if args.quiet {
        // Just validate - if we get here, parsing succeeded
        return Ok(());
    }

    if args.services_flag {
        for name in compose.services.keys() {
            println!("{name}");
        }
        return Ok(());
    }

    if args.volumes {
        if let Some(ref vols) = compose.volumes {
            for name in vols.keys() {
                println!("{name}");
            }
        }
        return Ok(());
    }

    if args.networks {
        if let Some(ref nets) = compose.networks {
            for name in nets.keys() {
                println!("{name}");
            }
        }
        return Ok(());
    }

    if args.images {
        for (_, service) in &compose.services {
            if let Some(ref img) = service.image {
                println!("{img}");
            }
        }
        return Ok(());
    }

    let output = match args.format.as_str() {
        "json" => serde_json::to_string_pretty(&compose)
            .map_err(|e| crate::error::AppError::Other(e.to_string()))?,
        _ => serde_yaml::to_string(&compose)
            .map_err(|e| crate::error::AppError::Other(e.to_string()))?,
    };

    if let Some(ref path) = args.output {
        std::fs::write(path, &output)?;
    } else {
        print!("{output}");
    }

    Ok(())
}
