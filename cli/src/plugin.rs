use std::fs;
use std::path::Path;
use std::io;
use libloading::{Library, Symbol};
use plugin_api::PluginContext;

#[derive(Debug)]
pub enum PluginType {
    Native(Box<Path>),
    Wasm(Box<Path>),
}

#[derive(Debug)]
pub enum PluginError {
    IoError(std::io::Error),
    LoadingError(libloading::Error),
    SerializationError(serde_json::Error),
}

impl From<std::io::Error> for PluginError {
    fn from(err: std::io::Error) -> Self {
        PluginError::IoError(err)
    }
}

impl From<libloading::Error> for PluginError {
    fn from(err: libloading::Error) -> Self {
        PluginError::LoadingError(err)
    }
}

impl From<serde_json::Error> for PluginError {
    fn from(err: serde_json::Error) -> Self {
        PluginError::SerializationError(err)
    }
}

pub fn run_plugin(plugin: PluginType, _context: &serde_json::Value) -> anyhow::Result<i32> {
    match plugin {
        PluginType::Native(path) => {
            println!("[PLUGIN DEBUG] Running native plugin: {}", path.display());
            run_native_plugin(&path)
        }
        PluginType::Wasm(_path) => {
            println!("[PLUGIN DEBUG] WASM plugins not yet implemented");
            Ok(0)
        }
    }
}

pub fn run_plugins(_hook: &str, ctx: &PluginContext) -> Result<(), PluginError> {
    let cwd = std::env::current_dir().expect("Failed to get current working directory");
    let plugins_dir = cwd.join(".boltpm/plugins");
    println!("[PLUGIN LOADER DEBUG] Searching for plugins in: {}", plugins_dir.display());
    if !plugins_dir.exists() {
        println!("[PLUGIN DEBUG] Plugins directory does not exist: {}", plugins_dir.display());
        return Ok(()); // No plugins to run
    }
    let entries = fs::read_dir(&plugins_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        println!("[PLUGIN LOADER DEBUG] Found file: {}", path.display());
        if path.extension().map(|e| e == "so" || e == "dylib" || e == "dll").unwrap_or(false) {
            println!("[PLUGIN LOADER DEBUG] Attempting to load plugin dylib: {}", path.display());
            
            // Serialize context to JSON bytes for FFI-safe passing
            let ctx_json = serde_json::to_string(ctx)?;
            let ctx_bytes = ctx_json.as_bytes();
            
            unsafe {
                let lib = Library::new(&path)?;
                let func: Result<Symbol<unsafe extern "C" fn(*const u8, usize) -> i32>, _> = lib.get(b"run");
                match func {
                    Ok(func) => {
                        let result = func(ctx_bytes.as_ptr(), ctx_bytes.len());
                        if result != 0 {
                            eprintln!("[PLUGIN ERROR] Plugin {} failed with code {}", path.display(), result);
                            return Err(PluginError::IoError(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Plugin returned error code: {}", result)
                            )));
                        }
                    }
                    Err(e) => {
                        eprintln!("[PLUGIN ERROR] Failed to load 'run' function from {}: {}", path.display(), e);
                        return Err(PluginError::LoadingError(e));
                    }
                }
            }
        } else {
            println!("[PLUGIN DEBUG] Skipping file with unsupported extension: {}", path.display());
        }
    }
    Ok(())
}

fn run_native_plugin(_path: &Path) -> anyhow::Result<i32> {
    // This function is a placeholder for native plugin execution
    // Currently, all plugins are loaded as dynamic libraries
    println!("[PLUGIN DEBUG] Native plugin execution not implemented");
    Ok(0)
} 