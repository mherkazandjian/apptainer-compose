use std::collections::{HashMap, HashSet, VecDeque};

use indexmap::IndexMap;

use crate::compose::types::Service;
use crate::error::{ComposeFileError, Result};

/// Resolve the start order of services using topological sort (Kahn's algorithm).
/// Returns services in dependency order (dependencies first).
pub fn resolve_order(
    services: &IndexMap<String, Service>,
    targets: &[String],
) -> Result<Vec<String>> {
    // Build adjacency list and in-degree map
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();

    // Collect all services that need to be started (targets + their transitive deps)
    let needed = collect_needed_services(services, targets);

    for name in &needed {
        in_degree.entry(name.clone()).or_insert(0);
        adj.entry(name.clone()).or_default();
    }

    for name in &needed {
        if let Some(service) = services.get(name) {
            if let Some(ref deps) = service.depends_on {
                for dep in deps.service_names() {
                    if needed.contains(&dep) {
                        adj.entry(dep.clone()).or_default().push(name.clone());
                        *in_degree.entry(name.clone()).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    // Kahn's algorithm
    let mut queue: VecDeque<String> = VecDeque::new();
    for (name, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(name.clone());
        }
    }

    let mut result = Vec::new();
    while let Some(node) = queue.pop_front() {
        result.push(node.clone());
        if let Some(neighbors) = adj.get(&node) {
            for neighbor in neighbors {
                let degree = in_degree.get_mut(neighbor).unwrap();
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(neighbor.clone());
                }
            }
        }
    }

    // Check for cycles
    if result.len() != needed.len() {
        let remaining: Vec<String> = needed
            .into_iter()
            .filter(|n| !result.contains(n))
            .collect();
        return Err(ComposeFileError::CircularDependency { cycle: remaining }.into());
    }

    Ok(result)
}

/// Collect all services needed to start the given targets, including transitive dependencies
fn collect_needed_services(
    services: &IndexMap<String, Service>,
    targets: &[String],
) -> Vec<String> {
    let mut needed: HashSet<String> = HashSet::new();
    let mut queue: VecDeque<String> = targets.iter().cloned().collect();

    while let Some(name) = queue.pop_front() {
        if needed.contains(&name) {
            continue;
        }
        needed.insert(name.clone());

        if let Some(service) = services.get(&name) {
            if let Some(ref deps) = service.depends_on {
                for dep in deps.service_names() {
                    if !needed.contains(&dep) {
                        queue.push_back(dep);
                    }
                }
            }
        }
    }

    // Return in a stable order (preserving compose file order)
    services
        .keys()
        .filter(|k| needed.contains(*k))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compose::types::{DependsOn, Service};

    fn make_service(deps: Option<Vec<&str>>) -> Service {
        let depends_on = deps.map(|d| DependsOn::List(d.into_iter().map(String::from).collect()));
        Service {
            image: Some("test:latest".to_string()),
            build: None,
            command: None,
            entrypoint: None,
            environment: None,
            env_file: None,
            ports: None,
            volumes: None,
            depends_on,
            restart: None,
            networks: None,
            hostname: None,
            dns: None,
            cap_add: None,
            cap_drop: None,
            deploy: None,
            healthcheck: None,
            working_dir: None,
            user: None,
            privileged: None,
            stdin_open: None,
            tty: None,
            labels: None,
            logging: None,
            tmpfs: None,
            shm_size: None,
            sysctls: None,
            ulimits: None,
            runtime: None,
            devices: None,
            extra_hosts: None,
            pid: None,
            ipc: None,
            stop_signal: None,
            stop_grace_period: None,
            network_mode: None,
            container_name: None,
            profiles: None,
            init: None,
            platform: None,
            pull_policy: None,
            expose: None,
            x_apptainer: None,
            extensions: Default::default(),
        }
    }

    #[test]
    fn test_simple_dependency_order() {
        let mut services = IndexMap::new();
        services.insert("web".to_string(), make_service(Some(vec!["db"])));
        services.insert("db".to_string(), make_service(None));

        let order = resolve_order(&services, &["web".to_string()]).unwrap();
        assert_eq!(order, vec!["db", "web"]);
    }

    #[test]
    fn test_no_deps() {
        let mut services = IndexMap::new();
        services.insert("web".to_string(), make_service(None));
        services.insert("worker".to_string(), make_service(None));

        let order =
            resolve_order(&services, &["web".to_string(), "worker".to_string()]).unwrap();
        assert_eq!(order.len(), 2);
    }

    #[test]
    fn test_circular_dependency_detected() {
        let mut services = IndexMap::new();
        services.insert("a".to_string(), make_service(Some(vec!["b"])));
        services.insert("b".to_string(), make_service(Some(vec!["a"])));

        let result = resolve_order(&services, &["a".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_transitive_deps() {
        let mut services = IndexMap::new();
        services.insert("app".to_string(), make_service(Some(vec!["api"])));
        services.insert("api".to_string(), make_service(Some(vec!["db"])));
        services.insert("db".to_string(), make_service(None));

        let order = resolve_order(&services, &["app".to_string()]).unwrap();
        assert_eq!(order, vec!["db", "api", "app"]);
    }
}
