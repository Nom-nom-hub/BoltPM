/// # Safety
/// This function dereferences raw pointers. The caller must ensure the pointers are valid and the memory is properly aligned and sized.
#[no_mangle]
pub unsafe extern "C" fn run(
    ctx_json: *const u8, ctx_len: usize,
    hook_ptr: *const u8, hook_len: usize,
    pkg_ptr: *const u8, pkg_len: usize
) -> i32 {
    // Write canary file to confirm FFI entry and context
    let canary_content = format!("run() entered at {}\nctx_len: {}\n", 
        chrono::Utc::now(), ctx_len);
    let _ = std::fs::write("/tmp/plugin_canary.txt", canary_content);
    
    // Safety: ctx_json is a pointer to a valid JSON string
    let ctx_slice = std::slice::from_raw_parts(ctx_json, ctx_len);
    let ctx_str = match std::str::from_utf8(ctx_slice) {
        Ok(s) => s,
        Err(_) => return 1,
    };
    let _ctx: serde_json::Value = match serde_json::from_str(ctx_str) {
        Ok(val) => val,
        Err(_) => return 1,
    };
    let hook = std::str::from_utf8(std::slice::from_raw_parts(hook_ptr, hook_len)).unwrap_or("unknown");
    let pkg = std::str::from_utf8(std::slice::from_raw_parts(pkg_ptr, pkg_len)).unwrap_or("");
    println!("[TestPlugin] Hook: {hook} | Package: {pkg}");
    let output_dir = std::path::PathBuf::from(".boltpm/plugins_output");
    if let Err(e) = std::fs::create_dir_all(&output_dir) {
        let _ = std::fs::write("/tmp/boltpm_plugin_error.txt", format!("Failed to create output dir: {e}"));
        return 1;
    }
    let output_file = output_dir.join("PLUGIN_TEST");
    if let Err(e) = std::fs::write(&output_file, format!("plugin ran for {hook}")) {
        let _ = std::fs::write("/tmp/boltpm_plugin_error.txt", format!("Failed to write output file: {e}"));
        return 1;
    }
    0
} 