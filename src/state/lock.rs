use fs2::FileExt;
use std::fs::File;
use std::path::Path;

use crate::error::{Result, StateError};

const STATE_DIR: &str = ".apptainer-compose";
const LOCK_FILE: &str = "state.lock";

/// A file-based lock for concurrent access protection
pub struct StateLock {
    _file: File,
}

impl StateLock {
    /// Acquire an exclusive lock on the state directory
    pub fn acquire(project_dir: &Path) -> Result<Self> {
        let lock_dir = project_dir.join(STATE_DIR);
        std::fs::create_dir_all(&lock_dir)?;

        let lock_path = lock_dir.join(LOCK_FILE);
        let file = File::create(&lock_path)
            .map_err(|e| StateError::LockFailed(format!("failed to create lock file: {e}")))?;

        file.try_lock_exclusive()
            .map_err(|e| StateError::LockFailed(format!("failed to acquire lock: {e}")))?;

        Ok(StateLock { _file: file })
    }
}

impl Drop for StateLock {
    fn drop(&mut self) {
        let _ = self._file.unlock();
    }
}
