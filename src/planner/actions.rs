use std::path::PathBuf;

/// Represents an action to be taken on the infrastructure
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Action {
    PullImage {
        service: String,
        uri: String,
        dest: PathBuf,
    },
    BuildImage {
        service: String,
        context: PathBuf,
        dest: PathBuf,
    },
    CreateVolume {
        name: String,
        path: PathBuf,
    },
    StartInstance {
        service: String,
        instance_name: String,
    },
    StopInstance {
        instance_name: String,
        signal: Option<String>,
        timeout: Option<i32>,
    },
    RemoveInstance {
        instance_name: String,
    },
    ExecInInstance {
        instance_name: String,
        command: Vec<String>,
    },
}
