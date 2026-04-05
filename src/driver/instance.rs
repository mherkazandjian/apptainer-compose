use crate::driver::apptainer::Apptainer;
use crate::error::Result;
use crate::planner::reconciler::StartArgs;

/// Launch an Apptainer instance using `instance run`.
///
/// `instance run` (not `instance start`) is required so the container's
/// runscript executes. For Docker images, the runscript is the CMD/ENTRYPOINT.
/// `instance start` only runs the %startscript, which Docker images lack,
/// so the service process would never actually start.
pub async fn start_instance(apptainer: &Apptainer, start_args: &StartArgs) -> Result<()> {
    let mut args: Vec<String> = Vec::new();

    // Add all apptainer-specific arguments
    args.extend(start_args.apptainer_args.clone());

    // Image path
    args.push(start_args.image_path.clone());

    // Instance name
    args.push(start_args.instance_name.clone());

    // Command arguments (passed to the runscript)
    args.extend(start_args.command_args.clone());

    apptainer.instance_run(&args).await
}
