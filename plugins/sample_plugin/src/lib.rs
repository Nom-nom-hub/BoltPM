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
    // Plugin logic: write output file to ctx.install_path/.boltpm/plugins_output/PLUGIN_TEST
    let output_dir = PathBuf::from(&ctx.install_path).join(".boltpm/plugins_output");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        let _ = fs::write("/tmp/boltpm_plugin_error.txt", format!("Failed to create output dir: {}", e));
        return 1;
    }
    let output_file = output_dir.join("PLUGIN_TEST");
    if let Err(e) = fs::write(&output_file, "plugin ran") {
        let _ = fs::write("/tmp/boltpm_plugin_error.txt", format!("Failed to write output file: {}", e));
        return 1;
    }
    0
} 