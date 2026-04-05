use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;

use crate::error::{ApptainerError, AppError, Result};

#[derive(Debug, Clone)]
pub struct Apptainer {
    pub binary: PathBuf,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct LiveInstance {
    #[serde(rename = "instance")]
    pub name: String,
    pub pid: u32,
    #[serde(rename = "img")]
    pub image: String,
    pub ip: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct InstanceListOutput {
    instances: Vec<LiveInstance>,
}

impl Apptainer {
    /// Detect apptainer binary on PATH
    pub fn detect() -> Result<Self> {
        let binary = which::which("apptainer")
            .or_else(|_| which::which("singularity"))
            .map_err(|_| ApptainerError::NotFound)?;

        Ok(Apptainer { binary })
    }

    /// Get apptainer version
    pub async fn version(&self) -> Result<String> {
        let output = Command::new(&self.binary)
            .arg("version")
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// List running instances (returns JSON-parsed list)
    pub async fn instance_list(&self) -> Result<Vec<LiveInstance>> {
        let output = Command::new(&self.binary)
            .args(["instance", "list", "--json"])
            .output()
            .await?;

        if !output.status.success() {
            // No instances running is not an error
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("no instance found") || output.stdout.is_empty() {
                return Ok(vec![]);
            }
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            return Ok(vec![]);
        }

        let parsed: InstanceListOutput = serde_json::from_str(&stdout).map_err(|e| {
            AppError::Other(format!("failed to parse instance list: {e}\nOutput: {stdout}"))
        })?;

        Ok(parsed.instances)
    }

    /// Run an instance with the given arguments.
    /// Uses `instance run` (not `instance start`) so that the container's
    /// runscript (Docker CMD/ENTRYPOINT) executes as the main process.
    /// `instance start` only runs the %startscript, which Docker images don't have.
    pub async fn instance_run(&self, args: &[String]) -> Result<()> {
        let output = Command::new(&self.binary)
            .arg("instance")
            .arg("run")
            .args(args)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ApptainerError::CommandFailed {
                command: format!("instance run {}", args.join(" ")),
                stderr: stderr.to_string(),
            }
            .into());
        }

        Ok(())
    }

    /// Stop an instance
    pub async fn instance_stop(
        &self,
        name: &str,
        signal: Option<&str>,
        timeout: Option<i32>,
    ) -> Result<()> {
        let mut cmd = Command::new(&self.binary);
        cmd.args(["instance", "stop"]);

        if let Some(sig) = signal {
            cmd.args(["--signal", sig]);
        }
        if let Some(t) = timeout {
            cmd.args(["--timeout", &t.to_string()]);
        }

        cmd.arg(name);

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ApptainerError::StopFailed {
                name: name.to_string(),
                reason: stderr.to_string(),
            }
            .into());
        }

        Ok(())
    }

    /// Execute a command in a running instance (inherits stdio for interactive use)
    pub async fn exec_instance(
        &self,
        instance_name: &str,
        command: &[&str],
        extra_args: &[String],
    ) -> Result<i32> {
        let mut cmd = Command::new(&self.binary);
        cmd.arg("exec");
        cmd.args(extra_args);
        cmd.arg(format!("instance://{instance_name}"));
        cmd.args(command);

        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        let status = cmd.status().await?;
        Ok(status.code().unwrap_or(1))
    }

    /// Execute a command in a running instance and capture output
    pub async fn exec_instance_capture(
        &self,
        instance_name: &str,
        command: &[&str],
        extra_args: &[String],
    ) -> Result<String> {
        let mut cmd = Command::new(&self.binary);
        cmd.arg("exec");
        cmd.args(extra_args);
        cmd.arg(format!("instance://{instance_name}"));
        cmd.args(command);

        let output = cmd.output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ApptainerError::CommandFailed {
                command: format!("exec instance://{instance_name} {}", command.join(" ")),
                stderr: stderr.to_string(),
            }
            .into());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Run a one-off container (not an instance - runs in foreground)
    pub async fn run_oneoff(
        &self,
        start_args: &crate::planner::reconciler::StartArgs,
        command: &[String],
    ) -> Result<i32> {
        let mut cmd = Command::new(&self.binary);
        cmd.arg("run");

        // Add all the same flags as instance start
        for arg in &start_args.apptainer_args {
            cmd.arg(arg);
        }

        cmd.arg(&start_args.image_path);

        if !command.is_empty() {
            cmd.args(command);
        }

        cmd.stdin(Stdio::inherit());
        cmd.stdout(Stdio::inherit());
        cmd.stderr(Stdio::inherit());

        let status = cmd.status().await?;
        Ok(status.code().unwrap_or(1))
    }

    /// Pull an image from a URI to a SIF file
    pub async fn pull(&self, uri: &str, dest: &str) -> Result<()> {
        let output = Command::new(&self.binary)
            .args(["pull", "--force", dest, uri])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ApptainerError::PullFailed {
                uri: uri.to_string(),
                reason: stderr.to_string(),
            }
            .into());
        }

        Ok(())
    }

    /// Build an image from a def file
    pub async fn build(&self, dest: &str, def_file: &str) -> Result<()> {
        let output = Command::new(&self.binary)
            .args(["build", "--force", dest, def_file])
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ApptainerError::CommandFailed {
                command: format!("build {dest} {def_file}"),
                stderr: stderr.to_string(),
            }
            .into());
        }

        Ok(())
    }

    /// Send a signal to an instance by looking up its PID
    pub async fn signal_instance(&self, instance_name: &str, signal: &str) -> Result<()> {
        let instances = self.instance_list().await?;
        let inst = instances
            .iter()
            .find(|i| i.name == instance_name)
            .ok_or_else(|| ApptainerError::InstanceNotFound {
                name: instance_name.to_string(),
            })?;

        let sig = match signal {
            "SIGSTOP" => nix::sys::signal::Signal::SIGSTOP,
            "SIGCONT" => nix::sys::signal::Signal::SIGCONT,
            "SIGTERM" => nix::sys::signal::Signal::SIGTERM,
            "SIGKILL" => nix::sys::signal::Signal::SIGKILL,
            "SIGHUP" => nix::sys::signal::Signal::SIGHUP,
            "SIGUSR1" => nix::sys::signal::Signal::SIGUSR1,
            "SIGUSR2" => nix::sys::signal::Signal::SIGUSR2,
            _ => nix::sys::signal::Signal::SIGTERM,
        };

        nix::sys::signal::kill(nix::unistd::Pid::from_raw(inst.pid as i32), sig)
            .map_err(|e| AppError::Other(format!("failed to send {signal} to PID {}: {e}", inst.pid)))?;

        Ok(())
    }

    /// Get stats for a named instance
    pub async fn instance_stats(&self, instance_name: &str) -> Result<String> {
        let output = Command::new(&self.binary)
            .args(["instance", "stats", instance_name])
            .output()
            .await?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
