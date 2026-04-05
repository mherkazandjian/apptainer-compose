use std::path::Path;

use crate::compose::types::{ComposeFile, Service};
use crate::driver::volume;
use crate::error::Result;

/// Arguments needed to start an Apptainer instance
#[derive(Debug, Clone)]
pub struct StartArgs {
    pub instance_name: String,
    pub image_path: String,
    pub apptainer_args: Vec<String>,
    pub command_args: Vec<String>,
}

/// Build the Apptainer instance start arguments from a compose service definition
pub fn build_start_args(
    project_dir: &Path,
    project_name: &str,
    service_name: &str,
    service: &Service,
    compose: &ComposeFile,
    image_path: &str,
    hosts_file: &Path,
) -> Result<StartArgs> {
    let instance_name = format!("{}_{}_1", project_name, service_name);
    let mut args: Vec<String> = Vec::new();

    // --compat by default (unless explicitly disabled via x-apptainer)
    let use_compat = service
        .x_apptainer
        .as_ref()
        .and_then(|x| x.compat)
        .unwrap_or(true);

    if use_compat {
        args.push("--compat".to_string());
    }

    // Bind mounts for volumes
    if let Some(ref volumes) = service.volumes {
        for vol in volumes {
            if let Some(bind_arg) =
                volume::volume_to_bind_arg(project_dir, vol, &compose.volumes)
            {
                args.push("--bind".to_string());
                args.push(bind_arg);
            }
        }
    }

    // Bind mount the hosts file for service discovery
    args.push("--bind".to_string());
    args.push(format!(
        "{}:/etc/hosts",
        hosts_file.to_string_lossy()
    ));

    // Environment variables
    if let Some(ref env) = service.environment {
        for (key, value) in env.to_map() {
            args.push("--env".to_string());
            args.push(format!("{key}={value}"));
        }
    }

    // Env files
    if let Some(ref env_file) = service.env_file {
        for ef in env_file.to_vec() {
            let ef_path = if Path::new(&ef).is_absolute() {
                ef
            } else {
                project_dir.join(&ef).to_string_lossy().to_string()
            };
            args.push("--env-file".to_string());
            args.push(ef_path);
        }
    }

    // Hostname
    if let Some(ref hostname) = service.hostname {
        args.push("--hostname".to_string());
        args.push(hostname.clone());
    } else {
        args.push("--hostname".to_string());
        args.push(service_name.to_string());
    }

    // DNS
    if let Some(ref dns) = service.dns {
        let dns_servers = dns.to_vec().join(",");
        args.push("--dns".to_string());
        args.push(dns_servers);
    }

    // Capabilities
    if let Some(ref caps) = service.cap_add {
        if !caps.is_empty() {
            args.push("--add-caps".to_string());
            args.push(caps.join(","));
        }
    }
    if let Some(ref caps) = service.cap_drop {
        if !caps.is_empty() {
            args.push("--drop-caps".to_string());
            args.push(caps.join(","));
        }
    }

    // Resource limits from deploy.resources
    if let Some(ref deploy) = service.deploy {
        if let Some(ref resources) = deploy.resources {
            if let Some(ref limits) = resources.limits {
                if let Some(ref cpus) = limits.cpus {
                    args.push("--cpus".to_string());
                    args.push(cpus.to_string());
                }
                if let Some(ref memory) = limits.memory {
                    args.push("--memory".to_string());
                    args.push(memory.clone());
                }
                if let Some(pids) = limits.pids {
                    args.push("--pids-limit".to_string());
                    args.push(pids.to_string());
                }
            }

            // GPU support from device reservations
            if let Some(ref reservations) = resources.reservations {
                if let Some(ref devices) = reservations.devices {
                    for device in devices {
                        if let Some(ref caps) = device.capabilities {
                            if caps.iter().any(|c| c == "gpu") {
                                // Default to NVIDIA
                                args.push("--nv".to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Privileged mode (approximated with --fakeroot)
    if service.privileged == Some(true) {
        tracing::warn!(
            "Service '{service_name}': privileged mode approximated with --fakeroot"
        );
        args.push("--fakeroot".to_string());
    }

    // Runtime (nvidia)
    if service.runtime.as_deref() == Some("nvidia") {
        args.push("--nv".to_string());
    }

    // Working directory
    if let Some(ref workdir) = service.working_dir {
        args.push("--cwd".to_string());
        args.push(workdir.clone());
    }

    // Extra hosts
    if let Some(ref extra_hosts) = service.extra_hosts {
        // These would need to be appended to the hosts file
        // For now, warn
        if !extra_hosts.is_empty() {
            tracing::warn!(
                "Service '{service_name}': extra_hosts are partially supported; consider adding them to the hosts file"
            );
        }
    }

    // Apptainer-specific extensions
    if let Some(ref ext) = service.x_apptainer {
        if ext.fakeroot == Some(true) {
            args.push("--fakeroot".to_string());
        }
        if ext.nv == Some(true) {
            args.push("--nv".to_string());
        }
        if ext.rocm == Some(true) {
            args.push("--rocm".to_string());
        }
        if ext.writable_tmpfs == Some(true) && !use_compat {
            args.push("--writable-tmpfs".to_string());
        }
        if ext.cleanenv == Some(true) && !use_compat {
            args.push("--cleanenv".to_string());
        }
        if ext.containall == Some(true) && !use_compat {
            args.push("--containall".to_string());
        }
        if let Some(ref binds) = ext.bind_extra {
            for bind in binds {
                args.push("--bind".to_string());
                args.push(bind.clone());
            }
        }
        if let Some(ref overlays) = ext.overlay {
            for overlay in overlays {
                args.push("--overlay".to_string());
                args.push(overlay.clone());
            }
        }
        if let Some(ref security) = ext.security {
            for sec in security {
                args.push("--security".to_string());
                args.push(sec.clone());
            }
        }
    }

    // tmpfs mounts -> --scratch
    if let Some(ref tmpfs) = service.tmpfs {
        for path in tmpfs.to_vec() {
            args.push("--scratch".to_string());
            args.push(path);
        }
    }

    // Command args
    let command_args = service
        .command
        .as_ref()
        .map(|c| c.to_vec())
        .unwrap_or_default();

    Ok(StartArgs {
        instance_name,
        image_path: image_path.to_string(),
        apptainer_args: args,
        command_args,
    })
}
