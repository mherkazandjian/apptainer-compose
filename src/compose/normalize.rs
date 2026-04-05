use crate::compose::types::{VolumeMount, VolumeMountDetails};

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

/// Normalize a VolumeMount to VolumeMountDetails
pub fn volume_to_details(vol: &VolumeMount) -> VolumeMountDetails {
    match vol {
        VolumeMount::Short(s) => normalize_volume_mount(s),
        VolumeMount::Long(d) => d.clone(),
    }
}
