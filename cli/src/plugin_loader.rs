use std::fs;
use std::path::PathBuf;
use libloading::{Library, Symbol};
use plugin_api::PluginContext;

pub fn run_plugins(hook: &str, ctx: PluginContext) {
    // Use the current working directory for plugin search
    let cwd = std::env::current_dir().expect("Failed to get current working directory");
    let plugin_dir = cwd.join(".boltpm/plugins");
    println!("[PLUGIN LOADER DEBUG] Searching for plugins in: {}", plugin_dir.display());
    if let Ok(entries) = fs::read_dir(&plugin_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            println!("[PLUGIN LOADER DEBUG] Found file: {}", path.display());
            // Skip macOS metadata files and non-regular files
            if !path.is_file() || path.file_name().map(|n| n.to_string_lossy().starts_with("._")).unwrap_or(false) {
                continue;
            }
            if let Some(extension) = path.extension() {
                if extension == "dylib" || extension == "so" || extension == "dll" {
                    println!("[PLUGIN LOADER DEBUG] Attempting to load plugin dylib: {}", path.display());
                    unsafe {
                        let lib = Library::new(&path).unwrap();
                        let func: Symbol<unsafe extern "C" fn(PluginContext) -> i32> = lib.get(b"run").unwrap();
                        let result = func(ctx.clone());
                        if result != 0 {
                            eprintln!("[PLUGIN ERROR] Plugin {} failed with code {}", path.display(), result);
                        }
                    }
                }
            }
        }
    }
} 