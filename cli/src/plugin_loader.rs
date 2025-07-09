use std::fs;
use libloading::{Library, Symbol};
use plugin_api::PluginContext;

pub fn run_plugins(hook: &str, ctx: PluginContext) {
    let _hook = hook;
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
                println!("[PLUGIN LOADER DEBUG] Skipping non-regular or metadata file: {}", path.display());
                continue;
            }
            if path.extension().map(|e| e == "so" || e == "dylib" || e == "dll").unwrap_or(false) {
                println!("[PLUGIN LOADER DEBUG] Attempting to load plugin: {}", path.display());
                unsafe {
                    let lib = Library::new(&path).unwrap();
                    let func: Symbol<unsafe extern fn(PluginContext) -> i32> = lib.get(b"run").unwrap();
                    let _ = func(ctx.clone());
                }
            } else {
                println!("[PLUGIN LOADER DEBUG] Skipping file with unsupported extension: {}", path.display());
            }
        }
    } else {
        println!("No plugins found in {:?}", plugin_dir);
    }
} 