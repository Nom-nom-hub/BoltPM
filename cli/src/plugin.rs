"""use std::fs;
use std::path::{Path};
use std::fmt;
use libloading::{Library, Symbol};
use plugin_api::PluginContext;
use serde_json;
use colored::*;
use wasmtime::*;
use wasmtime_wasi::{WasiCtxBuilder, WasiCtx};""

#[derive(Debug)]
pub enum PluginError {
    IoError(std::io::Error),
    LoadingError(libloading::Error),
    SerializationError(serde_json::Error),
    WasmError(String),
    PluginExecutionError(i32),
    PluginPanicError,
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::IoError(e) => write!(f, "IO error: {}", e),
            PluginError::LoadingError(e) => write!(f, "Plugin loading error: {}", e),
            PluginError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            PluginError::WasmError(e) => write!(f, "WASM error: {}", e),
            PluginError::PluginExecutionError(code) => write!(f, "Plugin execution failed with code: {}", code),
            PluginError::PluginPanicError => write!(f, "Plugin panicked during execution"),
        }
    }
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

impl std::error::Error for PluginError {}

/// Run all plugins in the given directory for the specified hook and context.
/// Supports both native and WASM plugins. Returns a vector of (plugin_name, status) results.
pub fn run_plugins(
    hook: &str,
    ctx: &PluginContext,
    plugins_dir: &Path,
    verbose: bool,
) -> Result<Vec<(String, String)>, PluginError> {
    let mut results = Vec::new();
    if verbose {
        println!("[PLUGIN LOADER DEBUG] Searching for plugins in: {}", plugins_dir.display());
    }
    if !plugins_dir.exists() {
        if verbose {
            println!("[PLUGIN DEBUG] Plugins directory does not exist: {}", plugins_dir.display());
        }
        return Ok(results); // No plugins to run
    }
    let entries = fs::read_dir(plugins_dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let plugin_name = path.file_name().unwrap().to_string_lossy().to_string();
        if verbose {
            println!("[PLUGIN LOADER DEBUG] Found file: {}", path.display());
        }
        if is_native_plugin(&path) {
            if verbose {
                println!("[PLUGIN LOADER DEBUG] Attempting to load plugin dylib: {}", path.display());
            }
            match run_native_plugin(&path, ctx) {
                Ok(_) => {
                    println!("{}", format!("[PLUGIN SUCCESS] {}", plugin_name).green());
                    results.push((plugin_name, "success".to_string()));
                }
                Err(e) => {
                    eprintln!("{}", format!("[PLUGIN ERROR] {}: {}", plugin_name, e).red().bold());
                    results.push((plugin_name, "error".to_string()));
                }
            }
        } else if is_wasm_plugin(&path) {
            if verbose {
                println!("[PLUGIN LOADER DEBUG] Attempting to load WASM plugin: {}", path.display());
            }
            let ctx_json = serde_json::to_string(ctx)?;
            match run_wasm_plugin(&path, &ctx_json) {
                Ok(_) => {
                    println!("{}", format!("[PLUGIN SUCCESS] {}", plugin_name).green());
                    results.push((plugin_name, "success".to_string()));
                }
                Err(e) => {
                    eprintln!("{}", format!("[PLUGIN ERROR] {}: {}", plugin_name, e).red().bold());
                    results.push((plugin_name, "error".to_string()));
                }
            }
        } else if verbose {
            println!("[PLUGIN LOADER DEBUG] Skipping file with unsupported extension: {}", path.display());
        }
    }
    Ok(results)
}

fn is_native_plugin(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            let ext = e.to_ascii_lowercase();
            ext == "so" || ext == "dylib" || ext == "dll"
        })
        .unwrap_or(false)
}

fn is_wasm_plugin(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case("wasm"))
        .unwrap_or(false)
}

fn run_native_plugin(path: &Path, ctx: &PluginContext) -> Result<(), PluginError> {
    println!("[PLUGIN DEBUG] About to run native plugin: {}", path.display());
    println!("[PLUGIN DEBUG] PluginContext: output_path={}, install_path={}, package_name={}, hook={}", ctx.output_path, ctx.install_path, ctx.package_name, ctx.hook);
    let ctx_json = serde_json::to_string(ctx)?;
    let ctx_bytes = ctx_json.as_bytes();
    unsafe {
        let lib = Library::new(path)?;
        let func: Result<Symbol<unsafe extern "C" fn(*const u8, usize) -> i32>, _> = lib.get(b"run");
        match func {
            Ok(func) => {
                use std::panic;
                let result = panic::catch_unwind(|| {
                    func(ctx_bytes.as_ptr(), ctx_bytes.len())
                });
                match result {
                    Ok(code) => {
                        if code != 0 {
                            return Err(PluginError::PluginExecutionError(code));
                        }
                    }
                    Err(_) => {
                        return Err(PluginError::PluginPanicError);
                    }
                }
            }
            Err(e) => {
                return Err(PluginError::LoadingError(e));
            }
        }
    }
    Ok(())
}

fn run_wasm_plugin(wasm_path: &Path, ctx_json: &str) -> Result<(), PluginError> {
    println!("[PLUGIN DEBUG] About to run WASM plugin: {}", wasm_path.display());
    // Try to parse PluginContext for debug
    if let Ok(ctx) = serde_json::from_str::<plugin_api::PluginContext>(ctx_json) {
        println!("[PLUGIN DEBUG] PluginContext: output_path={}, install_path={}, package_name={}, hook={}", ctx.output_path, ctx.install_path, ctx.package_name, ctx.hook);
    }
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    let mut wasi = WasiCtxBuilder::new()
        .inherit_stdio()
        .env("PLUGIN_CONTEXT", ctx_json)
        .map_err(|e| PluginError::WasmError(format!("WASI env error: {}", e)))?
        .build();
    wasmtime_wasi::add_to_linker(&mut linker, |s| s)
        .map_err(|e| PluginError::WasmError(format!("WASI linker error: {}", e)))?;
    let mut store = Store::new(&engine, wasi);
    let module = Module::from_file(&engine, wasm_path)
        .map_err(|e| PluginError::WasmError(format!("Failed to load WASM module: {}", e)))?;
    let instance = linker.instantiate(&mut store, &module)
        .map_err(|e| PluginError::WasmError(format!("Failed to instantiate WASM: {}", e)))?;
    let run = instance.get_typed_func::<(), ()>(&mut store, "_run")
        .map_err(|e| PluginError::WasmError(format!("Failed to get _run: {}", e)))?;
    run.call(&mut store, ())
        .map_err(|e| PluginError::WasmError(format!("WASM _run call failed: {}", e)))?;
    Ok(())
}
 