use std::path::Path;
use std::process::Command;

use crate::compose::normalize;
use crate::compose::types::{ComposeFile, VolumeConfig, VolumeMount};
use crate::error::{AppError, Result};

const VOLUMES_DIR: &str = ".apptainer-compose/volumes";
const DEFAULT_EXT3_SIZE: &str = "256M";

/// Check if a named volume has ext3 backend configured
fn is_ext3_volume(
    volume_name: &str,
    defined_volumes: &Option<indexmap::IndexMap<String, Option<VolumeConfig>>>,
) -> bool {
    if let Some(ref vols) = defined_volumes {
        if let Some(Some(config)) = vols.get(volume_name) {
            if let Some(ref ext) = config.x_apptainer {
                return ext.backend.as_deref() == Some("ext3");
            }
        }
    }
    false
}

/// Get the configured size for an ext3 volume
fn ext3_volume_size(
    volume_name: &str,
    defined_volumes: &Option<indexmap::IndexMap<String, Option<VolumeConfig>>>,
) -> String {
    if let Some(ref vols) = defined_volumes {
        if let Some(Some(config)) = vols.get(volume_name) {
            if let Some(ref ext) = config.x_apptainer {
                if let Some(ref size) = ext.size {
                    return size.clone();
                }
            }
        }
    }
    DEFAULT_EXT3_SIZE.to_string()
}

/// Parse a size string like "256M", "1G", "512" into megabytes
fn parse_size_to_mb(size: &str) -> u64 {
    let s = size.trim();
    if let Some(n) = s.strip_suffix('G').or_else(|| s.strip_suffix('g')) {
        n.trim().parse::<u64>().unwrap_or(256) * 1024
    } else if let Some(n) = s.strip_suffix('M').or_else(|| s.strip_suffix('m')) {
        n.trim().parse::<u64>().unwrap_or(256)
    } else {
        // Assume megabytes if no suffix
        s.parse::<u64>().unwrap_or(256)
    }
}

/// Create an ext3 volume image using `apptainer overlay create`
fn create_ext3_volume(apptainer_binary: &Path, image_path: &Path, size: &str) -> Result<()> {
    let size_mb = parse_size_to_mb(size);

    tracing::info!(
        "Creating ext3 volume: {} ({}MB)",
        image_path.display(),
        size_mb
    );

    // Ensure parent directory exists
    if let Some(parent) = image_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let output = Command::new(apptainer_binary)
        .args([
            "overlay",
            "create",
            "--size",
            &size_mb.to_string(),
            &image_path.to_string_lossy(),
        ])
        .output()
        .map_err(|e| AppError::Other(format!("failed to run apptainer overlay create: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::Other(format!(
            "failed to create ext3 volume {}: {}",
            image_path.display(),
            stderr
        )));
    }

    Ok(())
}

/// Ensure all volumes for a service exist.
/// Named volumes become managed directories or ext3 images under .apptainer-compose/volumes/.
/// Bind mounts are left as-is (host paths).
pub fn ensure_volumes(
    project_dir: &Path,
    compose: &ComposeFile,
    service_name: &str,
    apptainer_binary: &Path,
) -> Result<()> {
    let service = &compose.services[service_name];

    if let Some(ref volumes) = service.volumes {
        for vol in volumes {
            let details = normalize::volume_to_details(vol);

            match details.mount_type.as_deref() {
                Some("volume") => {
                    // Named volume
                    if let Some(ref source) = details.source {
                        if is_ext3_volume(source, &compose.volumes) {
                            // ext3 image volume
                            let img_path = project_dir
                                .join(VOLUMES_DIR)
                                .join(format!("{}.ext3", source));
                            if !img_path.exists() {
                                let size =
                                    ext3_volume_size(source, &compose.volumes);
                                create_ext3_volume(apptainer_binary, &img_path, &size)?;
                            }
                        } else {
                            // Plain directory volume (default)
                            let vol_dir = project_dir.join(VOLUMES_DIR).join(source);
                            if !vol_dir.exists() {
                                std::fs::create_dir_all(&vol_dir)?;
                                tracing::debug!(
                                    "Created volume directory: {}",
                                    vol_dir.display()
                                );
                            }
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
    defined_volumes: &Option<indexmap::IndexMap<String, Option<VolumeConfig>>>,
) -> Option<String> {
    let details = normalize::volume_to_details(vol);

    let source = match details.source {
        Some(ref s) => s.clone(),
        None => return None,
    };

    let target = &details.target;

    match details.mount_type.as_deref() {
        Some("volume") => {
            if is_ext3_volume(&source, defined_volumes) {
                // ext3 image volume -> --bind img.ext3:/target:image-src=/
                let img_path = project_dir
                    .join(VOLUMES_DIR)
                    .join(format!("{}.ext3", &source));
                let host = img_path.to_string_lossy().to_string();
                let opts = if details.read_only == Some(true) {
                    ":image-src=/:ro"
                } else {
                    ":image-src=/"
                };
                Some(format!("{host}:{target}{opts}"))
            } else {
                // Plain directory volume
                let vol_dir = project_dir.join(VOLUMES_DIR).join(&source);
                let host = vol_dir.to_string_lossy().to_string();
                let opts = if details.read_only == Some(true) {
                    ":ro"
                } else {
                    ""
                };
                Some(format!("{host}:{target}{opts}"))
            }
        }
        Some("bind") => {
            let host = if Path::new(&source).is_absolute() {
                source
            } else {
                project_dir.join(&source).to_string_lossy().to_string()
            };
            let opts = if details.read_only == Some(true) {
                ":ro"
            } else {
                ""
            };
            Some(format!("{host}:{target}{opts}"))
        }
        _ => None,
    }
}

/// Remove all managed volume directories and ext3 images
pub fn remove_volumes(project_dir: &Path) -> Result<()> {
    let dir = project_dir.join(VOLUMES_DIR);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}
