use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ComposeFile {
    pub version: Option<String>,

    #[serde(default)]
    pub services: IndexMap<String, Service>,

    #[serde(default)]
    pub volumes: Option<IndexMap<String, Option<VolumeConfig>>>,

    #[serde(default)]
    pub networks: Option<IndexMap<String, Option<NetworkConfig>>>,

    #[serde(default)]
    pub configs: Option<IndexMap<String, serde_yaml::Value>>,

    #[serde(default)]
    pub secrets: Option<IndexMap<String, serde_yaml::Value>>,

    #[serde(flatten)]
    pub extensions: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Service {
    pub image: Option<String>,

    pub build: Option<BuildConfig>,

    #[serde(default)]
    pub command: Option<Command>,

    #[serde(default)]
    pub entrypoint: Option<Command>,

    #[serde(default)]
    pub environment: Option<Environment>,

    #[serde(default)]
    pub env_file: Option<StringOrList>,

    #[serde(default)]
    pub ports: Option<Vec<PortMapping>>,

    #[serde(default)]
    pub volumes: Option<Vec<VolumeMount>>,

    #[serde(default)]
    pub depends_on: Option<DependsOn>,

    pub restart: Option<String>,

    #[serde(default)]
    pub networks: Option<ServiceNetworks>,

    pub hostname: Option<String>,

    #[serde(default)]
    pub dns: Option<StringOrList>,

    #[serde(default)]
    pub cap_add: Option<Vec<String>>,

    #[serde(default)]
    pub cap_drop: Option<Vec<String>>,

    pub deploy: Option<DeployConfig>,

    pub healthcheck: Option<HealthCheck>,

    pub working_dir: Option<String>,

    pub user: Option<String>,

    #[serde(default)]
    pub privileged: Option<bool>,

    #[serde(default)]
    pub stdin_open: Option<bool>,

    #[serde(default)]
    pub tty: Option<bool>,

    #[serde(default)]
    pub labels: Option<Labels>,

    pub logging: Option<LoggingConfig>,

    #[serde(default)]
    pub tmpfs: Option<StringOrList>,

    pub shm_size: Option<String>,

    #[serde(default)]
    pub sysctls: Option<HashMap<String, StringOrNumber>>,

    #[serde(default)]
    pub ulimits: Option<HashMap<String, UlimitConfig>>,

    pub runtime: Option<String>,

    #[serde(default)]
    pub devices: Option<Vec<String>>,

    #[serde(default)]
    pub extra_hosts: Option<Vec<String>>,

    pub pid: Option<String>,

    pub ipc: Option<String>,

    pub stop_signal: Option<String>,

    pub stop_grace_period: Option<String>,

    pub network_mode: Option<String>,

    pub container_name: Option<String>,

    #[serde(default)]
    pub profiles: Option<Vec<String>>,

    pub init: Option<bool>,

    pub platform: Option<String>,

    pub pull_policy: Option<String>,

    #[serde(default)]
    pub expose: Option<Vec<StringOrNumber>>,

    /// Apptainer-specific extensions
    #[serde(rename = "x-apptainer", default)]
    pub x_apptainer: Option<ApptainerExtensions>,

    #[serde(flatten)]
    pub extensions: HashMap<String, serde_yaml::Value>,
}

// --- Build ---

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum BuildConfig {
    Simple(String),
    Detailed(BuildDetails),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BuildDetails {
    pub context: Option<String>,
    pub dockerfile: Option<String>,
    pub args: Option<BuildArgs>,
    pub target: Option<String>,
    pub cache_from: Option<Vec<String>>,
    pub labels: Option<Labels>,
    pub shm_size: Option<String>,
    pub network: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum BuildArgs {
    Map(HashMap<String, Option<String>>),
    List(Vec<String>),
}

// --- Command ---

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Command {
    Simple(String),
    List(Vec<String>),
}

impl Command {
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            Command::Simple(s) => shell_words::split(s).unwrap_or_else(|_| vec![s.clone()]),
            Command::List(v) => v.clone(),
        }
    }
}

// --- Environment ---

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Environment {
    Map(IndexMap<String, Option<StringOrNumber>>),
    List(Vec<String>),
}

impl Environment {
    pub fn to_map(&self) -> IndexMap<String, String> {
        match self {
            Environment::Map(m) => m
                .iter()
                .map(|(k, v)| {
                    let val = v
                        .as_ref()
                        .map(|v| v.to_string())
                        .unwrap_or_default();
                    (k.clone(), val)
                })
                .collect(),
            Environment::List(l) => l
                .iter()
                .map(|s| {
                    if let Some((k, v)) = s.split_once('=') {
                        (k.to_string(), v.to_string())
                    } else {
                        (s.clone(), std::env::var(s).unwrap_or_default())
                    }
                })
                .collect(),
        }
    }
}

// --- StringOrNumber ---

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum StringOrNumber {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl std::fmt::Display for StringOrNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringOrNumber::String(s) => write!(f, "{s}"),
            StringOrNumber::Int(n) => write!(f, "{n}"),
            StringOrNumber::Float(n) => write!(f, "{n}"),
            StringOrNumber::Bool(b) => write!(f, "{b}"),
        }
    }
}

// --- StringOrList ---

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum StringOrList {
    String(String),
    List(Vec<String>),
}

impl StringOrList {
    pub fn to_vec(&self) -> Vec<String> {
        match self {
            StringOrList::String(s) => vec![s.clone()],
            StringOrList::List(v) => v.clone(),
        }
    }
}

// --- Ports ---

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum PortMapping {
    Short(String),
    Long(PortDetails),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PortDetails {
    pub target: u16,
    pub published: Option<u16>,
    pub protocol: Option<String>,
    pub host_ip: Option<String>,
    pub mode: Option<String>,
}

// --- Volumes ---

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum VolumeMount {
    Short(String),
    Long(VolumeMountDetails),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VolumeMountDetails {
    #[serde(rename = "type")]
    pub mount_type: Option<String>,
    pub source: Option<String>,
    pub target: String,
    pub read_only: Option<bool>,
    pub bind: Option<BindOptions>,
    pub volume: Option<VolumeOptions>,
    pub tmpfs: Option<TmpfsOptions>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BindOptions {
    pub propagation: Option<String>,
    pub create_host_path: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VolumeOptions {
    pub nocopy: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TmpfsOptions {
    pub size: Option<u64>,
    pub mode: Option<u32>,
}

// --- Volume config ---

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VolumeConfig {
    pub driver: Option<String>,
    pub driver_opts: Option<HashMap<String, String>>,
    pub external: Option<ExternalConfig>,
    pub labels: Option<Labels>,
    pub name: Option<String>,
    #[serde(rename = "x-apptainer")]
    pub x_apptainer: Option<VolumeApptainerExtensions>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VolumeApptainerExtensions {
    /// Volume backend: "ext3" for ext3 image files, or omit for plain directory (default)
    pub backend: Option<String>,
    /// Size of the ext3 image (e.g., "256M", "1G"). Default: 256M
    pub size: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum ExternalConfig {
    Bool(bool),
    Named { name: String },
}

// --- Networks ---

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NetworkConfig {
    pub driver: Option<String>,
    pub driver_opts: Option<HashMap<String, String>>,
    pub external: Option<ExternalConfig>,
    pub internal: Option<bool>,
    pub attachable: Option<bool>,
    pub labels: Option<Labels>,
    pub ipam: Option<IpamConfig>,
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IpamConfig {
    pub driver: Option<String>,
    pub config: Option<Vec<IpamPool>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IpamPool {
    pub subnet: Option<String>,
    pub gateway: Option<String>,
    pub ip_range: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum ServiceNetworks {
    List(Vec<String>),
    Map(IndexMap<String, Option<ServiceNetworkConfig>>),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServiceNetworkConfig {
    pub aliases: Option<Vec<String>>,
    pub ipv4_address: Option<String>,
    pub ipv6_address: Option<String>,
    pub priority: Option<i32>,
}

// --- DependsOn ---

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum DependsOn {
    List(Vec<String>),
    Map(IndexMap<String, DependsOnCondition>),
}

impl DependsOn {
    pub fn service_names(&self) -> Vec<String> {
        match self {
            DependsOn::List(v) => v.clone(),
            DependsOn::Map(m) => m.keys().cloned().collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DependsOnCondition {
    pub condition: Option<String>,
    pub restart: Option<bool>,
    pub required: Option<bool>,
}

// --- Deploy ---

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeployConfig {
    pub replicas: Option<u32>,
    pub resources: Option<Resources>,
    pub restart_policy: Option<RestartPolicy>,
    pub labels: Option<Labels>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Resources {
    pub limits: Option<ResourceLimits>,
    pub reservations: Option<ResourceReservations>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourceLimits {
    pub cpus: Option<StringOrNumber>,
    pub memory: Option<String>,
    pub pids: Option<i64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ResourceReservations {
    pub cpus: Option<StringOrNumber>,
    pub memory: Option<String>,
    pub devices: Option<Vec<DeviceReservation>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeviceReservation {
    pub capabilities: Option<Vec<String>>,
    pub driver: Option<String>,
    pub count: Option<StringOrNumber>,
    pub device_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct RestartPolicy {
    pub condition: Option<String>,
    pub delay: Option<String>,
    pub max_attempts: Option<u32>,
    pub window: Option<String>,
}

// --- HealthCheck ---

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HealthCheck {
    pub test: Option<HealthCheckTest>,
    pub interval: Option<String>,
    pub timeout: Option<String>,
    pub retries: Option<u32>,
    pub start_period: Option<String>,
    pub start_interval: Option<String>,
    pub disable: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum HealthCheckTest {
    Simple(String),
    List(Vec<String>),
}

// --- Labels ---

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum Labels {
    Map(HashMap<String, String>),
    List(Vec<String>),
}

impl Labels {
    pub fn to_map(&self) -> HashMap<String, String> {
        match self {
            Labels::Map(m) => m.clone(),
            Labels::List(l) => l
                .iter()
                .filter_map(|s| {
                    s.split_once('=')
                        .map(|(k, v)| (k.to_string(), v.to_string()))
                })
                .collect(),
        }
    }
}

// --- Logging ---

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub driver: Option<String>,
    pub options: Option<HashMap<String, String>>,
}

// --- Ulimits ---

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum UlimitConfig {
    Single(i64),
    Range { soft: i64, hard: i64 },
}

// --- Apptainer Extensions ---

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ApptainerExtensions {
    #[serde(default)]
    pub compat: Option<bool>,
    #[serde(default)]
    pub fakeroot: Option<bool>,
    #[serde(default)]
    pub nv: Option<bool>,
    #[serde(default)]
    pub rocm: Option<bool>,
    #[serde(default)]
    pub writable_tmpfs: Option<bool>,
    #[serde(default)]
    pub cleanenv: Option<bool>,
    #[serde(default)]
    pub containall: Option<bool>,
    #[serde(default)]
    pub sandbox: Option<bool>,
    #[serde(default)]
    pub bind_extra: Option<Vec<String>>,
    #[serde(default)]
    pub overlay: Option<Vec<String>>,
    #[serde(default)]
    pub security: Option<Vec<String>>,
    #[serde(default)]
    pub def_file: Option<String>,
}
