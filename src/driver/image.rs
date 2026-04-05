use std::path::{Path, PathBuf};
use std::sync::Arc;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use indexmap::IndexMap;

use crate::compose::types::Service;
use crate::driver::apptainer::Apptainer;
use crate::error::{AppError, Result};

const IMAGES_DIR: &str = ".apptainer-compose/images";

/// Get the images directory for the project
fn images_dir(project_dir: &Path) -> PathBuf {
    project_dir.join(IMAGES_DIR)
}

/// Sanitize an image name for use as a filename
fn image_filename(image: &str) -> String {
    image
        .replace('/', "_")
        .replace(':', "_")
        .replace('.', "_")
        + ".sif"
}

/// Resolve the SIF path for a service without pulling.
/// Returns the path string if image already exists or after pulling.
pub fn resolve_sif_path(project_dir: &Path, service: &Service) -> Option<(String, String)> {
    let image = service.image.as_ref()?;
    let dir = images_dir(project_dir);
    let filename = image_filename(image);
    let sif_path = dir.join(&filename);
    Some((image.clone(), sif_path.to_string_lossy().to_string()))
}

/// Ensure an image is available (pull if necessary), return path to SIF file
pub async fn ensure_image(
    apptainer: &Apptainer,
    project_dir: &Path,
    service_name: &str,
    service: &Service,
) -> Result<String> {
    if let Some(ref image) = service.image {
        let sif_path = pull_image(apptainer, project_dir, service_name, image).await?;
        return Ok(sif_path);
    }

    if service.build.is_some() {
        return build_image(apptainer, project_dir, service_name, service).await;
    }

    Err(AppError::Other(format!(
        "service '{service_name}' has neither 'image' nor 'build' specified"
    )))
}

/// Pull all images for the given services in parallel with a fancy progress display.
/// Returns a map of service_name -> sif_path.
pub async fn pull_images_parallel(
    apptainer: &Apptainer,
    project_dir: &Path,
    services: &IndexMap<String, Service>,
    target_services: &[String],
) -> Result<std::collections::HashMap<String, String>> {
    let dir = images_dir(project_dir);
    std::fs::create_dir_all(&dir)?;

    let mut results: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let mut pull_tasks = Vec::new();

    let mp = MultiProgress::new();
    let style_pulling = ProgressStyle::with_template(
        "{prefix:.bold.cyan} {spinner:.green} {wide_msg}"
    )
    .unwrap()
    .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏");

    let style_done = ProgressStyle::with_template(
        "{prefix:.bold.cyan} {wide_msg}"
    )
    .unwrap();

    for service_name in target_services {
        let service = match services.get(service_name) {
            Some(s) => s,
            None => continue,
        };

        let image = match &service.image {
            Some(img) => img.clone(),
            None => continue, // build-only services handled separately
        };

        let filename = image_filename(&image);
        let sif_path = dir.join(&filename);
        let sif_str = sif_path.to_string_lossy().to_string();

        // Already pulled — mark done immediately
        if sif_path.exists() {
            let pb = mp.add(ProgressBar::new_spinner());
            pb.set_style(style_done.clone());
            pb.set_prefix(format!("{service_name:>15}"));
            pb.finish_with_message(format!("Pulled {image}"));
            results.insert(service_name.clone(), sif_str);
            continue;
        }

        let uri = if image.contains("://") {
            image.clone()
        } else {
            format!("docker://{image}")
        };

        let pb = mp.add(ProgressBar::new_spinner());
        pb.set_style(style_pulling.clone());
        pb.set_prefix(format!("{service_name:>15}"));
        pb.set_message(format!("Pulling {image}..."));
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        let apptainer = apptainer.clone();
        let svc = service_name.clone();
        let img = image.clone();
        let done_style = style_done.clone();

        let handle = tokio::spawn(async move {
            let pb_clone = pb.clone();
            let result = apptainer
                .pull_with_progress(&uri, &sif_str, move |line| {
                    pb_clone.set_message(line.to_string());
                })
                .await;

            match &result {
                Ok(()) => {
                    pb.set_style(done_style);
                    pb.finish_with_message(format!("Pulled {img}"));
                }
                Err(e) => {
                    pb.finish_with_message(format!("Error pulling {img}: {e}"));
                }
            }
            result.map(|()| (svc, sif_str))
        });

        pull_tasks.push(handle);
    }

    // Wait for all pulls to complete
    for handle in pull_tasks {
        let (svc, sif_str) = handle.await.map_err(|e| AppError::Other(e.to_string()))??;
        results.insert(svc, sif_str);
    }

    Ok(results)
}

/// Pull an image and return the path to the SIF file
pub async fn pull_image(
    apptainer: &Apptainer,
    project_dir: &Path,
    _service_name: &str,
    image: &str,
) -> Result<String> {
    let dir = images_dir(project_dir);
    std::fs::create_dir_all(&dir)?;

    let filename = image_filename(image);
    let sif_path = dir.join(&filename);
    let sif_str = sif_path.to_string_lossy().to_string();

    // Skip if already pulled
    if sif_path.exists() {
        tracing::debug!("Image already exists: {sif_str}");
        return Ok(sif_str);
    }

    // Convert docker image reference to docker:// URI
    let uri = if image.contains("://") {
        image.to_string()
    } else {
        format!("docker://{image}")
    };

    tracing::info!("Pulling {uri} -> {sif_str}");
    apptainer.pull(&uri, &sif_str).await?;

    Ok(sif_str)
}

/// Build an image from a Dockerfile or def file
pub async fn build_image(
    apptainer: &Apptainer,
    project_dir: &Path,
    service_name: &str,
    service: &Service,
) -> Result<String> {
    let dir = images_dir(project_dir);
    std::fs::create_dir_all(&dir)?;

    let sif_path = dir.join(format!("{service_name}.sif"));
    let sif_str = sif_path.to_string_lossy().to_string();

    let build_config = service.build.as_ref().unwrap();

    // Check for Apptainer-specific def file
    if let Some(ref ext) = service.x_apptainer {
        if let Some(ref def_file) = ext.def_file {
            tracing::info!("Building from def file: {def_file}");
            apptainer.build(&sif_str, def_file).await?;
            return Ok(sif_str);
        }
    }

    let context = match build_config {
        crate::compose::types::BuildConfig::Simple(path) => path.clone(),
        crate::compose::types::BuildConfig::Detailed(d) => {
            d.context.clone().unwrap_or_else(|| ".".to_string())
        }
    };

    let context_path = if Path::new(&context).is_absolute() {
        PathBuf::from(&context)
    } else {
        project_dir.join(&context)
    };

    // Check if there's a Dockerfile
    let dockerfile = match build_config {
        crate::compose::types::BuildConfig::Detailed(d) => {
            d.dockerfile.clone().unwrap_or_else(|| "Dockerfile".to_string())
        }
        _ => "Dockerfile".to_string(),
    };

    let dockerfile_path = context_path.join(&dockerfile);

    if dockerfile_path.exists() {
        // Try to convert simple Dockerfile to def file
        let def_content = dockerfile_to_def(&dockerfile_path)?;
        let def_path = dir.join(format!("{service_name}.def"));
        std::fs::write(&def_path, def_content)?;
        tracing::info!("Converted Dockerfile to def file: {}", def_path.display());
        apptainer
            .build(&sif_str, &def_path.to_string_lossy())
            .await?;
        return Ok(sif_str);
    }

    Err(AppError::Other(format!(
        "Cannot build service '{service_name}': no Dockerfile or def file found at {}",
        context_path.display()
    )))
}

/// Simple Dockerfile to Apptainer def file conversion
/// Handles basic FROM, RUN, COPY, ENV, WORKDIR, CMD, ENTRYPOINT
fn dockerfile_to_def(dockerfile_path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(dockerfile_path)?;
    let mut def = String::new();
    let mut post_commands = Vec::new();
    let mut env_vars = Vec::new();
    let mut labels = Vec::new();
    let mut runscript = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some(rest) = line.strip_prefix("FROM ") {
            let image = rest.split_whitespace().next().unwrap_or(rest);
            // Skip "AS builder" stage names
            if image.contains("AS ") || image.contains(" as ") {
                tracing::warn!("Multi-stage builds are not supported; using final FROM");
                continue;
            }
            def.push_str("Bootstrap: docker\n");
            def.push_str(&format!("From: {image}\n"));
        } else if let Some(rest) = line.strip_prefix("RUN ") {
            post_commands.push(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("ENV ") {
            // Handle ENV KEY=VALUE or ENV KEY VALUE
            if let Some((key, value)) = rest.split_once('=') {
                env_vars.push(format!(
                    "export {}={}",
                    key.trim(),
                    value.trim().trim_matches('"')
                ));
            } else if let Some((key, value)) = rest.split_once(' ') {
                env_vars.push(format!(
                    "export {}={}",
                    key.trim(),
                    value.trim().trim_matches('"')
                ));
            }
        } else if let Some(rest) = line.strip_prefix("WORKDIR ") {
            post_commands.push(format!("mkdir -p {rest} && cd {rest}"));
        } else if let Some(rest) = line.strip_prefix("COPY ") {
            // COPY in Dockerfile copies from build context
            // In def file, this maps to %files section
            // For simplicity, we skip COPY --from (multi-stage)
            if rest.starts_with("--from") {
                tracing::warn!("COPY --from is not supported in Dockerfile conversion");
                continue;
            }
            let parts: Vec<&str> = rest.splitn(2, ' ').collect();
            if parts.len() == 2 {
                tracing::warn!(
                    "COPY directive partially supported: {} -> {}",
                    parts[0],
                    parts[1]
                );
            }
        } else if let Some(rest) = line.strip_prefix("CMD ") {
            let cmd = rest.trim_start_matches('[').trim_end_matches(']');
            let cmd = cmd.replace('"', "").replace(',', " ");
            runscript = cmd.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("ENTRYPOINT ") {
            let cmd = rest.trim_start_matches('[').trim_end_matches(']');
            let cmd = cmd.replace('"', "").replace(',', " ");
            runscript = cmd.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("LABEL ") {
            labels.push(rest.to_string());
        } else if let Some(rest) = line.strip_prefix("EXPOSE ") {
            // Informational only in Apptainer
            labels.push(format!("expose {rest}"));
        } else if line.starts_with("ARG ") || line.starts_with("SHELL ") || line.starts_with("USER ") {
            tracing::warn!("Dockerfile directive not supported in conversion: {line}");
        }
    }

    if def.is_empty() {
        return Err(AppError::Other(
            "Dockerfile has no FROM directive".to_string(),
        ));
    }

    def.push('\n');

    if !env_vars.is_empty() {
        def.push_str("%environment\n");
        for env in &env_vars {
            def.push_str(&format!("    {env}\n"));
        }
        def.push('\n');
    }

    if !post_commands.is_empty() {
        def.push_str("%post\n");
        for cmd in &post_commands {
            def.push_str(&format!("    {cmd}\n"));
        }
        def.push('\n');
    }

    if !runscript.is_empty() {
        def.push_str("%runscript\n");
        def.push_str(&format!("    {runscript}\n"));
        def.push('\n');
    }

    if !labels.is_empty() {
        def.push_str("%labels\n");
        for label in &labels {
            def.push_str(&format!("    {label}\n"));
        }
        def.push('\n');
    }

    Ok(def)
}

/// Remove all pulled/built images
pub fn remove_images(project_dir: &Path) -> Result<()> {
    let dir = images_dir(project_dir);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}
