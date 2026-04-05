use std::path::{Path, PathBuf};

use crate::cli::GlobalOpts;
use crate::compose::interpolation;
use crate::compose::merge;
use crate::compose::types::ComposeFile;
use crate::error::{AppError, ComposeFileError, Result};

const DEFAULT_FILES: &[&str] = &[
    "apptainer-compose.yaml",
    "apptainer-compose.yml",
    "compose.yaml",
    "compose.yml",
    "docker-compose.yaml",
    "docker-compose.yml",
];

/// Resolve the project directory from global options
pub fn resolve_project_dir(global: &GlobalOpts) -> Result<PathBuf> {
    if let Some(ref dir) = global.project_directory {
        Ok(dir.clone())
    } else if let Some(ref first_file) = global.file.first() {
        Ok(first_file
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from(".")))
    } else {
        Ok(std::env::current_dir()?)
    }
}

/// Resolve the project name from global options
pub fn resolve_project_name(global: &GlobalOpts, project_dir: &Path) -> String {
    if let Some(ref name) = global.project_name {
        return name.clone();
    }

    if let Ok(name) = std::env::var("COMPOSE_PROJECT_NAME") {
        return name;
    }

    // Default: directory name, sanitized
    project_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "default".to_string())
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric() && c != '_' && c != '-', "")
}

/// Find compose files based on global options
fn find_compose_files(global: &GlobalOpts) -> Result<Vec<PathBuf>> {
    if !global.file.is_empty() {
        // Validate all specified files exist
        for f in &global.file {
            if !f.exists() {
                return Err(ComposeFileError::NotFound { path: f.clone() }.into());
            }
        }
        return Ok(global.file.clone());
    }

    // Check COMPOSE_FILE env var
    if let Ok(compose_file) = std::env::var("COMPOSE_FILE") {
        let separator = std::env::var("COMPOSE_PATH_SEPARATOR").unwrap_or_else(|_| ":".to_string());
        let files: Vec<PathBuf> = compose_file.split(&separator).map(PathBuf::from).collect();
        for f in &files {
            if !f.exists() {
                return Err(ComposeFileError::NotFound { path: f.clone() }.into());
            }
        }
        return Ok(files);
    }

    // Search for default files
    let project_dir = resolve_project_dir(global)?;
    for name in DEFAULT_FILES {
        let path = project_dir.join(name);
        if path.exists() {
            return Ok(vec![path]);
        }
    }

    Err(ComposeFileError::NotFound {
        path: project_dir.join("docker-compose.yml"),
    }
    .into())
}

/// Load and parse compose file(s) with interpolation and merging
pub fn load_compose(global: &GlobalOpts) -> Result<ComposeFile> {
    let files = find_compose_files(global)?;

    // Collect environment for interpolation
    let mut env_files = global.env_file.clone();
    let project_dir = resolve_project_dir(global)?;
    let default_env = project_dir.join(".env");
    if default_env.exists() && !env_files.contains(&default_env) {
        env_files.insert(0, default_env);
    }
    let env = interpolation::collect_env(&env_files);

    let mut compose_files: Vec<ComposeFile> = Vec::new();

    for file in &files {
        let content = std::fs::read_to_string(file).map_err(|e| ComposeFileError::ParseError {
            file: file.clone(),
            message: e.to_string(),
        })?;

        // Parse as raw YAML first for interpolation
        let raw: serde_yaml::Value = serde_yaml::from_str(&content).map_err(|e| {
            ComposeFileError::ParseError {
                file: file.clone(),
                message: e.to_string(),
            }
        })?;

        // Interpolate environment variables
        let interpolated = interpolation::interpolate_yaml(&raw, &env).map_err(|e| {
            AppError::ComposeFile(ComposeFileError::Interpolation(e))
        })?;

        // Deserialize into typed struct
        let compose: ComposeFile =
            serde_yaml::from_value(interpolated).map_err(|e| ComposeFileError::ParseError {
                file: file.clone(),
                message: e.to_string(),
            })?;

        compose_files.push(compose);
    }

    // Merge multiple compose files
    let merged = if compose_files.len() == 1 {
        compose_files.into_iter().next().unwrap()
    } else {
        merge::merge_compose_files(compose_files)
    };

    // Filter by profiles if specified
    let filtered = if global.profile.is_empty() {
        merged
    } else {
        filter_by_profiles(merged, &global.profile)
    };

    // Validate
    validate_compose(&filtered)?;

    Ok(filtered)
}

/// Filter services by active profiles
fn filter_by_profiles(mut compose: ComposeFile, active_profiles: &[String]) -> ComposeFile {
    compose.services.retain(|_name, service| {
        match &service.profiles {
            None => true, // services without profiles are always active
            Some(profiles) => profiles.iter().any(|p| active_profiles.contains(p)),
        }
    });
    compose
}

/// Validate the compose file for common errors
fn validate_compose(compose: &ComposeFile) -> Result<()> {
    for (name, service) in &compose.services {
        // Check that each service has either image or build
        if service.image.is_none() && service.build.is_none() {
            return Err(ComposeFileError::NoImageOrBuild {
                service: name.clone(),
            }
            .into());
        }

        // Validate depends_on references
        if let Some(ref deps) = service.depends_on {
            for dep in deps.service_names() {
                if !compose.services.contains_key(&dep) {
                    return Err(AppError::Other(format!(
                        "service '{name}' depends on unknown service '{dep}'"
                    )));
                }
            }
        }

        // Validate network references
        if let Some(ref networks) = service.networks {
            let net_names: Vec<String> = match networks {
                crate::compose::types::ServiceNetworks::List(l) => l.clone(),
                crate::compose::types::ServiceNetworks::Map(m) => m.keys().cloned().collect(),
            };
            if let Some(ref defined_nets) = compose.networks {
                for net_name in &net_names {
                    if net_name != "default" && !defined_nets.contains_key(net_name) {
                        return Err(ComposeFileError::UnknownNetwork {
                            service: name.clone(),
                            network: net_name.clone(),
                        }
                        .into());
                    }
                }
            }
        }
    }

    Ok(())
}
