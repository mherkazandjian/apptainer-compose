use colored::Colorize;
use std::collections::HashMap;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::driver::apptainer::Apptainer;
use crate::error::Result;

const COLORS: &[&str] = &[
    "green", "yellow", "blue", "magenta", "cyan", "red",
    "bright_green", "bright_yellow", "bright_blue", "bright_magenta", "bright_cyan",
];

/// Stream logs from one or more instances
pub async fn stream_logs(
    apptainer: &Apptainer,
    instance_names: &[String],
    follow: bool,
    no_prefix: bool,
    _timestamps: bool,
) -> Result<()> {
    // Get instance log file paths
    let output = Command::new(&apptainer.binary)
        .args(["instance", "list", "--logs"])
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Parse log file locations from instance list --logs output
    let mut log_files: HashMap<String, (String, String)> = HashMap::new(); // name -> (stdout_log, stderr_log)

    for line in stdout.lines() {
        let line = line.trim();
        // Parse lines like: "instance_name    PID    IMAGE    stdout_log    stderr_log"
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 5 {
            let name = parts[0].to_string();
            if instance_names.contains(&name) {
                log_files.insert(
                    name,
                    (parts[3].to_string(), parts[4].to_string()),
                );
            }
        }
    }

    if log_files.is_empty() {
        // Fallback: find logs at the default Apptainer location
        // Path structure: ~/.apptainer/instances/logs/{hostname}/{username}/{name}.out/.err
        let home = dirs::home_dir().unwrap_or_default();
        let hostname = gethostname::gethostname();
        let hostname = hostname.to_string_lossy();
        let username = whoami::username();
        let logs_dir = home.join(format!(".apptainer/instances/logs/{hostname}/{username}"));

        for name in instance_names {
            let stdout_log = logs_dir.join(format!("{name}.out"));
            let stderr_log = logs_dir.join(format!("{name}.err"));

            if stdout_log.exists() {
                log_files.insert(
                    name.clone(),
                    (
                        stdout_log.to_string_lossy().to_string(),
                        stderr_log.to_string_lossy().to_string(),
                    ),
                );
            }
        }
    }

    if log_files.is_empty() {
        tracing::info!("No log files found for the specified instances");
        return Ok(());
    }

    // Spawn tail processes for each log file
    let mut handles = Vec::new();

    for (idx, (name, (stdout_log, _stderr_log))) in log_files.iter().enumerate() {
        let color = COLORS[idx % COLORS.len()];
        let name = name.clone();
        let log_path = stdout_log.clone();
        let no_prefix = no_prefix;
        let color = color.to_string();

        let handle = tokio::spawn(async move {
            let tail_args = if follow {
                vec!["tail", "-f", "-n", "+1", &log_path]
            } else {
                vec!["cat", &log_path]
            };

            let mut child = match Command::new(tail_args[0])
                .args(&tail_args[1..])
                .stdout(std::process::Stdio::piped())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    tracing::warn!("Failed to read logs for {name}: {e}");
                    return;
                }
            };

            if let Some(stdout) = child.stdout.take() {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();

                while let Ok(Some(line)) = lines.next_line().await {
                    if no_prefix {
                        println!("{line}");
                    } else {
                        let prefix = format!("{name} |");
                        let colored_prefix = match color.as_str() {
                            "green" => prefix.green().to_string(),
                            "yellow" => prefix.yellow().to_string(),
                            "blue" => prefix.blue().to_string(),
                            "magenta" => prefix.magenta().to_string(),
                            "cyan" => prefix.cyan().to_string(),
                            "red" => prefix.red().to_string(),
                            _ => prefix.green().to_string(),
                        };
                        println!("{colored_prefix} {line}");
                    }
                }
            }

            let _ = child.wait().await;
        });

        handles.push(handle);
    }

    // Wait for all log streams
    for handle in handles {
        let _ = handle.await;
    }

    Ok(())
}
