use libloading::{Library, Symbol};
use log::{debug, error, info};
use plugin_api::PluginContext;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum PluginError {
    Io(std::io::Error),
    Loading(libloading::Error),
    Serialization(serde_json::Error),
    Execution(i32),
    Panic,
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::Io(e) => write!(f, "IO error: {e}"),
            PluginError::Loading(e) => write!(f, "Plugin loading error: {e}"),
            PluginError::Serialization(e) => write!(f, "Serialization error: {e}"),
            PluginError::Execution(code) => write!(f, "Plugin execution failed with code: {code}"),
            PluginError::Panic => write!(f, "Plugin panicked during execution"),
        }
    }
}

impl From<std::io::Error> for PluginError {
    fn from(err: std::io::Error) -> Self {
        PluginError::Io(err)
    }
}

impl From<libloading::Error> for PluginError {
    fn from(err: libloading::Error) -> Self {
        PluginError::Loading(err)
    }
}

impl From<serde_json::Error> for PluginError {
    fn from(err: serde_json::Error) -> Self {
        PluginError::Serialization(err)
    }
}

impl std::error::Error for PluginError {}

pub fn run_plugins(_hook: &str, ctx: &PluginContext) -> Result<(), PluginError> {
    let cwd = std::env::current_dir().map_err(|e| {
        PluginError::Io(std::io::Error::other(format!(
            "Failed to get current working directory: {e}"
        )))
    })?;
    let plugins_dir = cwd.join(".boltpm/plugins");
    debug!("Searching for plugins in: {}", plugins_dir.display());

    if !plugins_dir.exists() {
        debug!(
            "Plugins directory does not exist: {}",
            plugins_dir.display()
        );
        return Ok(()); // No plugins to run
    }

    let entries = fs::read_dir(&plugins_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        debug!("Found file: {}", path.display());

        if is_plugin_file(&path) {
            debug!("Attempting to load plugin: {}", path.display());
            load_and_run_plugin(&path, ctx)?;
        } else {
            debug!(
                "Skipping file with unsupported extension: {}",
                path.display()
            );
        }
    }
    Ok(())
}

/// Check if a file is a valid plugin based on its extension
fn is_plugin_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let ext = e.to_ascii_lowercase();
            ext == "so" || ext == "dylib" || ext == "dll"
        })
        .unwrap_or(false)
}

/// Load and execute a plugin with proper error handling
fn load_and_run_plugin(path: &Path, ctx: &PluginContext) -> Result<(), PluginError> {
    // Serialize context to JSON bytes for FFI-safe passing
    let ctx_json = serde_json::to_string(ctx)?;
    let ctx_bytes = ctx_json.as_bytes();

    unsafe {
        let lib = Library::new(path)?;
        let func: Result<Symbol<unsafe extern "C" fn(*const u8, usize) -> i32>, _> =
            lib.get(b"run");

        match func {
            Ok(func) => {
                use std::panic;

                // Note: catch_unwind only catches Rust panics, not segmentation faults
                // or aborts from FFI code. For untrusted plugins, consider isolating
                // them in a subprocess for better security.
                let call_result = panic::catch_unwind(|| func(ctx_bytes.as_ptr(), ctx_bytes.len()));

                match call_result {
                    Ok(result) => {
                        if result != 0 {
                            error!("Plugin {} failed with code {}", path.display(), result);
                            return Err(PluginError::Execution(result));
                        }
                        info!("Plugin {} executed successfully", path.display());
                    }
                    Err(_) => {
                        error!("Plugin {} panicked during execution", path.display());
                        return Err(PluginError::Panic);
                    }
                }
            }
            Err(e) => {
                error!(
                    "Failed to load 'run' function from {}: {}",
                    path.display(),
                    e
                );
                return Err(PluginError::Loading(e));
            }
        }
    }
    Ok(())
}
