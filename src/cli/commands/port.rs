use clap::Args;

use crate::cli::GlobalOpts;
use crate::compose::parser::load_compose;
use crate::error::Result;

#[derive(Args)]
pub struct PortArgs {
    /// Service name
    pub service: String,

    /// Private port
    pub private_port: u16,

    /// Index of the container if service has multiple replicas
    #[arg(long)]
    pub index: Option<u32>,

    /// tcp or udp
    #[arg(long, default_value = "tcp")]
    pub protocol: String,
}

pub async fn run(global: GlobalOpts, args: PortArgs) -> Result<()> {
    let compose = load_compose(&global)?;

    let service = compose
        .services
        .get(&args.service)
        .ok_or_else(|| crate::error::AppError::Other(format!("service '{}' not found", args.service)))?;

    // With host networking, the port is directly accessible
    if let Some(ref ports) = service.ports {
        for port in ports {
            match port {
                crate::compose::types::PortMapping::Short(s) => {
                    if s.contains(&format!("{}", args.private_port)) {
                        println!("0.0.0.0:{}", args.private_port);
                        return Ok(());
                    }
                }
                crate::compose::types::PortMapping::Long(p) => {
                    if p.target == args.private_port {
                        let host_port = p.published.unwrap_or(p.target);
                        println!("0.0.0.0:{host_port}");
                        return Ok(());
                    }
                }
            }
        }
    }

    // With Apptainer host networking, the port is directly accessible on the host
    println!("0.0.0.0:{}", args.private_port);
    Ok(())
}
