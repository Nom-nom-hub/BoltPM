use std::fs;
use std::path::PathBuf;
use libloading::{Library, Symbol};
use plugin_api::PluginContext;

pub fn run_plugins(hook: &str, ctx: &PluginContext) {
    let plugin_dir = PathBuf::from(".boltpm/plugins");
    if let Ok(entries) = fs::read_dir(&plugin_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "so" || e == "dylib" || e == "dll").unwrap_or(false) {
                println!("Loading plugin: {:?}", path);
                unsafe {
                    let lib = Library::new(&path).unwrap();
                    let func: Symbol<unsafe extern fn(PluginContext) -> i32> = lib.get(b"run").unwrap();
                    let _ = func(ctx.clone());
                }
            }
        }
    } else {
        println!("No plugins found in {:?}", plugin_dir);
    }
} 