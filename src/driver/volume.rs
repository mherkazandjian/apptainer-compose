use std::path::Path;

use crate::compose::normalize;
use crate::compose::types::{ComposeFile, VolumeMount};
use crate::error::Result;

const VOLUMES_DIR: &str = ".apptainer-compose/volumes";

/// Ensure all volumes for a service exist.
/// Named volumes become managed directories under .apptainer-compose/volumes/.
/// Bind mounts are left as-is (host paths).
pub fn ensure_volumes(
    project_dir: &Path,
    compose: &ComposeFile,
    service_name: &str,
) -> Result<()> {
    let service = &compose.services[service_name];

    if let Some(ref volumes) = service.volumes {
        for vol in volumes {
            let details = normalize::volume_to_details(vol);

            match details.mount_type.as_deref() {
                Some("volume") => {
                    // Named volume - create managed directory
                    if let Some(ref source) = details.source {
                        let vol_dir = project_dir.join(VOLUMES_DIR).join(source);
                        if !vol_dir.exists() {
                            std::fs::create_dir_all(&vol_dir)?;
                            tracing::debug!("Created volume directory: {}", vol_dir.display());
                        }
                    }
                }
                Some("bind") => {
                    // Bind mount - ensure host path exists
                    if let Some(ref source) = details.source {
                        let src_path = if Path::new(source).is_absolute() {
                            std::path::PathBuf::from(source)
                        } else {
                            project_dir.join(source)
                        };
                        if !src_path.exists() {
                            std::fs::create_dir_all(&src_path)?;
                            tracing::debug!("Created bind mount source: {}", src_path.display());
                        }
                    }
                }
                Some("tmpfs") => {
                    // tmpfs handled by --scratch at runtime
                }
                _ => {}
            }
        }
    }

    Ok(())
}

/// Convert a VolumeMount to an Apptainer --bind argument
pub fn volume_to_bind_arg(
    project_dir: &Path,
    vol: &VolumeMount,
    _defined_volumes: &Option<indexmap::IndexMap<String, Option<crate::compose::types::VolumeConfig>>>,
) -> Option<String> {
    let details = normalize::volume_to_details(vol);

    let source = match details.source {
        Some(ref s) => s.clone(),
        None => return None,
    };

    let host_path = match details.mount_type.as_deref() {
        Some("volume") => {
            // Named volume -> managed directory
            let vol_dir = project_dir.join(VOLUMES_DIR).join(&source);
            vol_dir.to_string_lossy().to_string()
        }
        Some("bind") => {
            if Path::new(&source).is_absolute() {
                source
            } else {
                project_dir.join(&source).to_string_lossy().to_string()
            }
        }
        _ => return None,
    };

    let target = &details.target;

    let opts = if details.read_only == Some(true) {
        ":ro"
    } else {
        ""
    };

    Some(format!("{host_path}:{target}{opts}"))
}

/// Remove all managed volume directories
pub fn remove_volumes(project_dir: &Path) -> Result<()> {
    let dir = project_dir.join(VOLUMES_DIR);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}
