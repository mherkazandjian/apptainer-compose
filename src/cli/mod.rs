pub mod commands;

use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::error::Result;

#[derive(Parser)]
#[command(
    name = "apptainer-compose",
    version,
    about = "Define and run multi-container applications with Apptainer",
    long_about = "A drop-in replacement for docker-compose using Apptainer as the container runtime"
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalOpts,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Args, Clone)]
pub struct GlobalOpts {
    /// Compose configuration files
    #[arg(short = 'f', long = "file", global = true)]
    pub file: Vec<PathBuf>,

    /// Project name
    #[arg(short = 'p', long = "project-name", global = true, env = "COMPOSE_PROJECT_NAME")]
    pub project_name: Option<String>,

    /// Specify an alternate working directory
    #[arg(long = "project-directory", global = true)]
    pub project_directory: Option<PathBuf>,

    /// Specify an alternate environment file
    #[arg(long = "env-file", global = true)]
    pub env_file: Vec<PathBuf>,

    /// Specify a profile to enable
    #[arg(long = "profile", global = true)]
    pub profile: Vec<String>,

    /// Control when to print ANSI control characters
    #[arg(long = "ansi", global = true, default_value = "auto")]
    pub ansi: String,

    /// Execute command in dry run mode
    #[arg(long = "dry-run", global = true)]
    pub dry_run: bool,

    /// Set type of progress output
    #[arg(long = "progress", global = true, default_value = "auto")]
    pub progress: String,

    /// Control max parallelism, -1 for unlimited
    #[arg(long = "parallel", global = true)]
    pub parallel: Option<i32>,

    /// Run compose in backward compatibility mode
    #[arg(long = "compatibility", global = true)]
    pub compatibility: bool,

    /// Include all resources, even those not used by services
    #[arg(long = "all-resources", global = true)]
    pub all_resources: bool,

    /// Print additional information
    #[arg(short = 'v', long = "verbose", global = true)]
    pub verbose: bool,

    /// Suppress normal output
    #[arg(short = 'q', long = "quiet", global = true)]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create and start containers
    Up(commands::up::UpArgs),
    /// Stop and remove containers, networks
    Down(commands::down::DownArgs),
    /// List containers
    Ps(commands::ps::PsArgs),
    /// View output from containers
    Logs(commands::logs::LogsArgs),
    /// Execute a command in a running container
    Exec(commands::exec::ExecArgs),
    /// Run a one-off command on a service
    Run(commands::run::RunArgs),
    /// Build or rebuild services
    Build(commands::build::BuildArgs),
    /// Pull service images
    Pull(commands::pull::PullArgs),
    /// Start services
    Start(commands::start::StartArgs),
    /// Stop services
    Stop(commands::stop::StopArgs),
    /// Restart service containers
    Restart(commands::restart::RestartArgs),
    /// Parse, resolve and render compose file in canonical format
    Config(commands::config::ConfigArgs),
    /// Creates containers for a service
    Create(commands::create::CreateArgs),
    /// Force stop service containers
    Kill(commands::kill::KillArgs),
    /// Removes stopped service containers
    Rm(commands::rm::RmArgs),
    /// Pause services
    Pause(commands::pause::PauseArgs),
    /// Unpause services
    Unpause(commands::unpause::UnpauseArgs),
    /// Display the running processes
    Top(commands::top::TopArgs),
    /// List images used by the created containers
    Images(commands::images::ImagesArgs),
    /// Scale services
    Scale(commands::scale::ScaleArgs),
    /// Print the public port for a port binding
    Port(commands::port::PortArgs),
    /// Show the version information
    Version(commands::version::VersionArgs),
    /// Copy files/folders between a service container and the local filesystem
    Cp(commands::cp::CpArgs),
    /// Display container resource usage statistics
    Stats(commands::stats::StatsArgs),
    /// List running compose projects
    Ls(commands::ls::LsArgs),
    /// Receive real time events from containers
    Events(commands::events::EventsArgs),
    /// Block until containers of all (or specified) services stop
    Wait(commands::wait::WaitArgs),
    /// Attach local standard input, output, and error streams to a service's running container
    Attach(commands::attach::AttachArgs),
    /// Push service images
    Push(commands::push::PushArgs),
    /// Watch build context for service and rebuild/refresh containers when files are updated
    Watch(commands::watch::WatchArgs),
    /// List volumes
    Volumes(commands::volumes::VolumesArgs),
    /// Export a service container's filesystem as a tar archive
    Export(commands::export::ExportArgs),
    /// Create a new image from a service container's changes
    Commit(commands::commit::CommitArgs),
}

pub async fn dispatch(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Up(args) => commands::up::run(cli.global, args).await,
        Commands::Down(args) => commands::down::run(cli.global, args).await,
        Commands::Ps(args) => commands::ps::run(cli.global, args).await,
        Commands::Logs(args) => commands::logs::run(cli.global, args).await,
        Commands::Exec(args) => commands::exec::run(cli.global, args).await,
        Commands::Run(args) => commands::run::run(cli.global, args).await,
        Commands::Build(args) => commands::build::run(cli.global, args).await,
        Commands::Pull(args) => commands::pull::run(cli.global, args).await,
        Commands::Start(args) => commands::start::run(cli.global, args).await,
        Commands::Stop(args) => commands::stop::run(cli.global, args).await,
        Commands::Restart(args) => commands::restart::run(cli.global, args).await,
        Commands::Config(args) => commands::config::run(cli.global, args).await,
        Commands::Create(args) => commands::create::run(cli.global, args).await,
        Commands::Kill(args) => commands::kill::run(cli.global, args).await,
        Commands::Rm(args) => commands::rm::run(cli.global, args).await,
        Commands::Pause(args) => commands::pause::run(cli.global, args).await,
        Commands::Unpause(args) => commands::unpause::run(cli.global, args).await,
        Commands::Top(args) => commands::top::run(cli.global, args).await,
        Commands::Images(args) => commands::images::run(cli.global, args).await,
        Commands::Scale(args) => commands::scale::run(cli.global, args).await,
        Commands::Port(args) => commands::port::run(cli.global, args).await,
        Commands::Version(args) => commands::version::run(cli.global, args).await,
        Commands::Cp(args) => commands::cp::run(cli.global, args).await,
        Commands::Stats(args) => commands::stats::run(cli.global, args).await,
        Commands::Ls(args) => commands::ls::run(cli.global, args).await,
        Commands::Events(args) => commands::events::run(cli.global, args).await,
        Commands::Wait(args) => commands::wait::run(cli.global, args).await,
        Commands::Attach(args) => commands::attach::run(cli.global, args).await,
        Commands::Push(args) => commands::push::run(cli.global, args).await,
        Commands::Watch(args) => commands::watch::run(cli.global, args).await,
        Commands::Volumes(args) => commands::volumes::run(cli.global, args).await,
        Commands::Export(args) => commands::export::run(cli.global, args).await,
        Commands::Commit(args) => commands::commit::run(cli.global, args).await,
    }
}
