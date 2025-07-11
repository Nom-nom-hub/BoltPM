use plugin_api::PluginContext;
use std::fs;
use std::fs::write;
use std::path::PathBuf;
use std::slice;

/// # Safety
/// This function dereferences raw pointers. The caller must ensure the pointers are valid and the memory is properly aligned and sized.
#[no_mangle]
pub unsafe extern "C" fn run(ctx_ptr: *const u8, ctx_len: usize) -> i32 {
    let _ = write("/tmp/plugin_entry.txt", b"entered run()\n");
    // SAFETY: ctx_ptr must be valid for ctx_len bytes
    let ctx_slice = slice::from_raw_parts(ctx_ptr, ctx_len);
    let ctx: PluginContext = match serde_json::from_slice(ctx_slice) {
        Ok(c) => c,
        Err(e) => {
            let _ = fs::write(
                "/tmp/boltpm_plugin_deser_error.txt",
                format!("Deserialization error: {e}"),
            );
            return 1;
        }
    };
    // Plugin logic: write output file to ctx.install_path/.boltpm/plugins_output/PLUGIN_TEST
    let output_dir = PathBuf::from(&ctx.install_path).join(".boltpm/plugins_output");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        let _ = fs::write(
            "/tmp/boltpm_plugin_error.txt",
            format!("Failed to create output dir: {e}"),
        );
        return 1;
    }
    let output_file = output_dir.join("PLUGIN_TEST");
    if let Err(e) = fs::write(&output_file, "plugin ran") {
        let _ = fs::write(
            "/tmp/boltpm_plugin_error.txt",
            format!("Failed to write output file: {e}"),
        );
        return 1;
    }
    0
}
