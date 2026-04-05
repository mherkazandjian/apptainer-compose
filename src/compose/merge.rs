use crate::compose::types::ComposeFile;

/// Merge multiple compose files. Later files override earlier ones.
/// This follows docker-compose merge semantics.
pub fn merge_compose_files(files: Vec<ComposeFile>) -> ComposeFile {
    let mut base = files.into_iter().next().expect("at least one compose file");

    // For now, simple last-wins merge for services
    // A full implementation would merge individual service fields
    for file in std::iter::once(base.clone()).skip(1) {
        for (name, service) in file.services {
            base.services.insert(name, service);
        }

        if let Some(volumes) = file.volumes {
            let base_volumes = base.volumes.get_or_insert_with(Default::default);
            for (name, config) in volumes {
                base_volumes.insert(name, config);
            }
        }

        if let Some(networks) = file.networks {
            let base_networks = base.networks.get_or_insert_with(Default::default);
            for (name, config) in networks {
                base_networks.insert(name, config);
            }
        }
    }

    base
}
