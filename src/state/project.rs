use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::driver::apptainer::LiveInstance;
use crate::error::Result;

const STATE_DIR: &str = ".apptainer-compose";
const STATE_FILE: &str = "state.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectState {
    pub version: u32,
    pub project_name: String,
    pub compose_files: Vec<String>,
    pub services: IndexMap<String, ServiceState>,
    pub volumes: IndexMap<String, VolumeState>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceState {
    pub instances: Vec<InstanceState>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanceState {
    pub instance_name: String,
    pub image_path: String,
    pub image_source: String,
    pub status: String,
    pub pid: Option<u32>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VolumeState {
    pub host_path: String,
    pub created_at: String,
}

impl ProjectState {
    /// Load state from disk, or return a default state
    pub fn load_or_default(project_dir: &Path, project_name: &str) -> Self {
        let state_path = project_dir.join(STATE_DIR).join(STATE_FILE);

        if let Ok(content) = std::fs::read_to_string(&state_path) {
            if let Ok(state) = serde_json::from_str::<ProjectState>(&content) {
                return state;
            }
        }

        ProjectState {
            version: 1,
            project_name: project_name.to_string(),
            compose_files: vec![],
            services: IndexMap::new(),
            volumes: IndexMap::new(),
        }
    }

    /// Save state to disk
    pub fn save(&self, project_dir: &Path) -> Result<()> {
        let state_dir = project_dir.join(STATE_DIR);
        std::fs::create_dir_all(&state_dir)?;

        let state_path = state_dir.join(STATE_FILE);
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| crate::error::StateError::WriteFailed(e.to_string()))?;

        std::fs::write(&state_path, content)
            .map_err(|e| crate::error::StateError::WriteFailed(e.to_string()))?;

        Ok(())
    }

    /// Reconcile state with live Apptainer instances
    pub fn reconcile_with_live(&mut self, live_instances: &[LiveInstance]) {
        for (_name, svc_state) in &mut self.services {
            for inst in &mut svc_state.instances {
                let is_live = live_instances.iter().any(|li| li.name == inst.instance_name);
                if is_live {
                    inst.status = "running".to_string();
                    // Update PID from live data
                    if let Some(li) = live_instances.iter().find(|li| li.name == inst.instance_name)
                    {
                        inst.pid = Some(li.pid);
                    }
                } else if inst.status == "running" {
                    inst.status = "exited".to_string();
                    inst.pid = None;
                }
            }
        }
    }

    /// Check if an instance is running
    pub fn is_instance_running(&self, instance_name: &str) -> bool {
        self.services.values().any(|svc| {
            svc.instances
                .iter()
                .any(|i| i.instance_name == instance_name && i.status == "running")
        })
    }

    /// Mark an instance as running
    pub fn set_instance_running(
        &mut self,
        service_name: &str,
        instance_name: &str,
        image_path: &str,
        image_source: &str,
    ) {
        let svc_state = self
            .services
            .entry(service_name.to_string())
            .or_insert_with(|| ServiceState {
                instances: Vec::new(),
            });

        // Update existing or add new
        if let Some(inst) = svc_state
            .instances
            .iter_mut()
            .find(|i| i.instance_name == instance_name)
        {
            inst.status = "running".to_string();
            inst.image_path = image_path.to_string();
            inst.image_source = image_source.to_string();
        } else {
            svc_state.instances.push(InstanceState {
                instance_name: instance_name.to_string(),
                image_path: image_path.to_string(),
                image_source: image_source.to_string(),
                status: "running".to_string(),
                pid: None,
                created_at: chrono::Utc::now().to_rfc3339(),
            });
        }
    }

    /// Mark an instance as stopped
    pub fn set_instance_stopped(&mut self, service_name: &str) {
        if let Some(svc_state) = self.services.get_mut(service_name) {
            for inst in &mut svc_state.instances {
                inst.status = "exited".to_string();
                inst.pid = None;
            }
        }
    }

    /// Remove a service from state
    pub fn remove_service(&mut self, service_name: &str) {
        self.services.swap_remove(service_name);
    }
}
