use std::fs;
use std::path::{Path, PathBuf};
use std::io::{self, Write};
use serde_json;
use anyhow;
use libloading::{Library, Symbol};

#[cfg(feature = "wasm_plugins")]
use wasmtime::*;

use plugin_api::PluginContext;

#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Plugin load error: {0}")]
    Load(#[from] libloading::Error),
    #[error("Plugin returned error code: {0}")]
    PluginFailed(i32),
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[allow(dead_code)]
pub enum PluginType {
    Native(PathBuf),
    Wasm(PathBuf),
}

#[allow(dead_code)]
pub fn discover_plugins(plugin_dir: &Path) -> Vec<PluginType> {
    let mut plugins = vec![];
    let mut seen = std::collections::HashSet::new();
    if let Ok(entries) = fs::read_dir(plugin_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if cfg!(feature = "wasm_plugins") && path.extension().map(|e| e == "wasm").unwrap_or(false) {
                if seen.insert(stem.to_string()) {
                    plugins.push(PluginType::Wasm(path));
                }
            } else if path.extension().map(|e| e == "dylib" || e == "so" || e == "dll").unwrap_or(false) {
                if !seen.contains(stem) {
                    plugins.push(PluginType::Native(path));
                }
            }
        }
    }
    plugins
}

#[allow(dead_code)]
pub fn run_plugin(plugin: PluginType, context: &serde_json::Value) -> anyhow::Result<i32> {
    match plugin {
        PluginType::Native(path) => {
            run_native_plugin(&path)
        },
        PluginType::Wasm(path) => {
            #[cfg(feature = "wasm_plugins")]
            { run_wasm_plugin(&path, context) }
            #[cfg(not(feature = "wasm_plugins"))]
            { Err(anyhow::anyhow!("WASM plugin support not enabled")) }
        },
    }
}

#[allow(dead_code)]
pub fn run_plugins(hook: &str, ctx: &PluginContext) -> Result<(), PluginError> {
    let cwd = std::env::current_dir().expect("Failed to get current working directory");
    println!("[PLUGIN LOADER DEBUG] Current working directory: {}", cwd.display());
    let plugins_dir = cwd.join(".boltpm/plugins");
    println!("[PLUGIN LOADER DEBUG] Resolved plugin directory: {}", plugins_dir.display());
    if !plugins_dir.exists() {
        println!("[PLUGIN DEBUG] Plugins directory does not exist: {}", plugins_dir.display());
        return Ok(()); // No plugins to run
    }
    let entries = fs::read_dir(&plugins_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        println!("[PLUGIN LOADER DEBUG] Found file: {}", path.display());
        if path.extension().and_then(|s| s.to_str()).map(|s| {
            s == "so" || s == "dll" || s == "dylib"
        }).unwrap_or(false) {
            let abs_path = path.canonicalize().unwrap_or(path.clone());
            println!("[PLUGIN LOADER DEBUG] Attempting to load plugin dylib: {}", abs_path.display());
            // Load the plugin
            unsafe {
                match Library::new(&path) {
                    Ok(lib) => {
                        let func: Result<Symbol<unsafe extern fn(*const u8, usize) -> i32>, _> = lib.get(b"run");
                        match func {
                            Ok(func) => {
                                let ctx_json = serde_json::to_vec(ctx)?;
                                std::fs::write("/tmp/boltpm_plugin_ctx.json", &ctx_json).expect("Failed to write plugin context JSON");
                                eprintln!("[PluginLoader] About to call run()");
                                let result = std::panic::catch_unwind(|| {
                                    let code = func(ctx_json.as_ptr(), ctx_json.len());
                                    eprintln!("[PluginLoader] run() returned: {}", code);
                                    code
                                });
                                match result {
                                    Ok(code) => {
                                        if code != 0 {
                                            eprintln!("[PLUGIN ERROR] Plugin {} failed with code {}", abs_path.display(), code);
                                            return Err(PluginError::PluginFailed(code));
                                        }
                                    },
                                    Err(_) => {
                                        eprintln!("[PluginLoader] run() panicked!");
                                        return Err(PluginError::PluginFailed(-1));
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("[PLUGIN ERROR] Failed to get 'run' symbol from {}: {}", abs_path.display(), e);
                                return Err(PluginError::Load(e));
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("[PLUGIN ERROR] Failed to load plugin {}: {}", abs_path.display(), e);
                        return Err(PluginError::Load(e));
                    }
                }
            }
        } else {
            println!("[PLUGIN LOADER DEBUG] Skipping file with unsupported extension: {}", path.display());
        }
    }
    Ok(())
}

// Existing native plugin loader logic
#[allow(dead_code)]
fn run_native_plugin(path: &Path) -> anyhow::Result<i32> {
    let cwd = std::env::current_dir().expect("Failed to get current working directory");
    let plugins_dir = cwd.join(".boltpm/plugins");
    println!("[PLUGIN LOADER DEBUG] Searching for plugins in: {}", plugins_dir.display());
    if !plugins_dir.exists() {
        println!("[PLUGIN DEBUG] Plugins directory does not exist: {}", plugins_dir.display());
        return Ok(0); // No plugins to run
    }
    let entries = fs::read_dir(&plugins_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        println!("[PLUGIN LOADER DEBUG] Found file: {}", path.display());
        if path.extension().and_then(|s| s.to_str()).map(|s| {
            s == "so" || s == "dll" || s == "dylib"
        }).unwrap_or(false) {
            let abs_path = path.canonicalize().unwrap_or(path.clone());
            println!("[PLUGIN LOADER DEBUG] Attempting to load plugin dylib: {}", abs_path.display());
            // Load the plugin
            unsafe {
                match Library::new(&path) {
                    Ok(lib) => {
                        let func: Result<Symbol<unsafe extern fn(*const u8, usize) -> i32>, _> = lib.get(b"run");
                        match func {
                            Ok(func) => {
                                let ctx_json = serde_json::to_vec(&serde_json::Value::Null)?; // Placeholder for context
                                let code = func(ctx_json.as_ptr(), ctx_json.len());
                                println!("[PLUGIN DEBUG] Plugin {} returned code {}", abs_path.display(), code);
                                if code != 0 {
                                    eprintln!("[PLUGIN ERROR] Plugin {} failed with code {}", abs_path.display(), code);
                                    return Err(PluginError::PluginFailed(code).into());
                                }
                                return Ok(code);
                            },
                            Err(e) => {
                                eprintln!("[PLUGIN ERROR] Failed to get 'run' symbol from {}: {}", abs_path.display(), e);
                                return Err(PluginError::Load(e).into());
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("[PLUGIN ERROR] Failed to load plugin {}: {}", abs_path.display(), e);
                        return Err(PluginError::Load(e).into());
                    }
                }
            }
        } else {
            println!("[PLUGIN LOADER DEBUG] Skipping file with unsupported extension: {}", path.display());
        }
    }
    Ok(0) // No native plugin found
}

#[cfg(feature = "wasm_plugins")]
#[allow(dead_code)]
fn run_wasm_plugin(path: &Path, context: &serde_json::Value) -> anyhow::Result<i32> {
    use wasmtime::{Caller, Extern, Func, Linker, Memory};
    use std::fs;
    use std::path::PathBuf;
    let engine = Engine::default();
    let module = Module::from_file(&engine, path)?;
    let mut store = Store::new(&engine, ());
    let mut linker = Linker::new(&engine);
    // Provide host_write_file import
    linker.func_wrap("env", "host_write_file", |mut caller: Caller<'_, ()>, ptr: i32, len: i32| {
        let memory = caller.get_export("memory").and_then(|e| e.into_memory()).ok_or(anyhow::anyhow!("No memory export"))?;
        let mut buf = vec![0u8; len as usize];
        memory.read(&caller, ptr as usize, &mut buf)?;
        let out_path = PathBuf::from(".boltpm/plugins_output/wasm_test_hook");
        fs::create_dir_all(out_path.parent().unwrap()).ok();
        fs::write(&out_path, &buf)?;
        Ok(())
    })?;
    let instance = linker.instantiate(&mut store, &module)?;
    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or_else(|| anyhow::anyhow!("WASM plugin missing `memory` export"))?;
    let run_func = instance
        .get_func(&mut store, "run")
        .ok_or_else(|| anyhow::anyhow!("WASM plugin missing `run` export"))?
        .typed::<(i32, i32), i32>(&store)?;
    let ctx_json = serde_json::to_vec(context)?;
    let ctx_len = ctx_json.len() as i32;
    let ctx_ptr = 1024; // fixed offset for now
    memory.write(&mut store, ctx_ptr as usize, &ctx_json)?;
    let result = run_func.call(&mut store, (ctx_ptr, ctx_len))?;
    Ok(result)
} 