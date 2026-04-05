use crate::compose::types::{PortDetails, PortMapping, VolumeMount, VolumeMountDetails};

/// Parse a short-form port mapping like "8080:80" or "8080:80/tcp" into PortDetails
pub fn normalize_port(short: &str) -> PortDetails {
    let (port_part, protocol) = if let Some((p, proto)) = short.rsplit_once('/') {
        (p, Some(proto.to_string()))
    } else {
        (short, None)
    };

    let parts: Vec<&str> = port_part.splitn(3, ':').collect();
    match parts.len() {
        1 => {
            let target: u16 = parts[0].parse().unwrap_or(0);
            PortDetails {
                target,
                published: None,
                protocol,
                host_ip: None,
                mode: None,
            }
        }
        2 => {
            let published: u16 = parts[0].parse().unwrap_or(0);
            let target: u16 = parts[1].parse().unwrap_or(0);
            PortDetails {
                target,
                published: Some(published),
                protocol,
                host_ip: None,
                mode: None,
            }
        }
        3 => {
            let host_ip = Some(parts[0].to_string());
            let published: u16 = parts[1].parse().unwrap_or(0);
            let target: u16 = parts[2].parse().unwrap_or(0);
            PortDetails {
                target,
                published: Some(published),
                protocol,
                host_ip,
                mode: None,
            }
        }
        _ => PortDetails {
            target: 0,
            published: None,
            protocol,
            host_ip: None,
            mode: None,
        },
    }
}

/// Parse a short-form volume mount like "./data:/data:ro" into VolumeMountDetails
pub fn normalize_volume_mount(short: &str) -> VolumeMountDetails {
    let parts: Vec<&str> = short.splitn(3, ':').collect();

    match parts.len() {
        1 => VolumeMountDetails {
            mount_type: Some("volume".to_string()),
            source: None,
            target: parts[0].to_string(),
            read_only: None,
            bind: None,
            volume: None,
            tmpfs: None,
        },
        2 => {
            let source = parts[0].to_string();
            let target = parts[1].to_string();
            let mount_type = if source.starts_with('.') || source.starts_with('/') {
                "bind"
            } else {
                "volume"
            };
            VolumeMountDetails {
                mount_type: Some(mount_type.to_string()),
                source: Some(source),
                target,
                read_only: None,
                bind: None,
                volume: None,
                tmpfs: None,
            }
        }
        3 => {
            let source = parts[0].to_string();
            let target = parts[1].to_string();
            let opts = parts[2];
            let mount_type = if source.starts_with('.') || source.starts_with('/') {
                "bind"
            } else {
                "volume"
            };
            VolumeMountDetails {
                mount_type: Some(mount_type.to_string()),
                source: Some(source),
                target,
                read_only: Some(opts.contains("ro")),
                bind: None,
                volume: None,
                tmpfs: None,
            }
        }
        _ => VolumeMountDetails {
            mount_type: None,
            source: None,
            target: short.to_string(),
            read_only: None,
            bind: None,
            volume: None,
            tmpfs: None,
        },
    }
}

/// Normalize a PortMapping to PortDetails
pub fn port_to_details(port: &PortMapping) -> PortDetails {
    match port {
        PortMapping::Short(s) => normalize_port(s),
        PortMapping::Long(d) => d.clone(),
    }
}

/// Normalize a VolumeMount to VolumeMountDetails
pub fn volume_to_details(vol: &VolumeMount) -> VolumeMountDetails {
    match vol {
        VolumeMount::Short(s) => normalize_volume_mount(s),
        VolumeMount::Long(d) => d.clone(),
    }
}
