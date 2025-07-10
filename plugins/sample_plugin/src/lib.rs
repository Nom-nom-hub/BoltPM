use plugin_api::PluginContext;
use std::fs;
use std::path::PathBuf;
use std::slice;
use serde_json;
use std::fs::write;

#[no_mangle]
pub extern "C" fn run(ctx_ptr: *const u8, ctx_len: usize) -> i32 {
    let _ = write("/tmp/plugin_entry.txt", b"entered run()\n");
    // SAFETY: ctx_ptr must be valid for ctx_len bytes
    let ctx_slice = unsafe { slice::from_raw_parts(ctx_ptr, ctx_len) };
    let ctx: PluginContext = match serde_json::from_slice(ctx_slice) {
        Ok(c) => c,
        Err(e) => {
            let _ = fs::write("/tmp/boltpm_plugin_deser_error.txt", format!("Deserialization error: {}", e));
            return 1;
        }
    };
    println!("[SamplePlugin] Executing plugin...");
    println!("[SamplePlugin] ctx.output_path: {:?}", ctx.output_path);
    let output_dir = PathBuf::from(&ctx.output_path);
    let output_file = output_dir.join("PLUGIN_TEST");
    if let Err(_e) = fs::create_dir_all(&output_dir) {
        eprintln!("[SamplePlugin] Failed to create output directory");
        return 1;
    }
    if let Err(_e) = fs::write(&output_file, "Plugin executed successfully!") {
        eprintln!("[SamplePlugin] Failed to write output file");
        return 1;
    }
    // Write install_path debug info to a file
    let debug_file = output_dir.join("INSTALL_PATH_DEBUG");
    if let Err(e) = fs::write(&debug_file, &ctx.install_path) {
    }
    0
} 