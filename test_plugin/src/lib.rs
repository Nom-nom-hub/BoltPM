use std::fs;
use std::path::PathBuf;

#[no_mangle]
pub unsafe extern "C" fn run(ctx_json: *const u8, ctx_len: usize, hook: &str, pkg: &str) -> i32 {
    // Write canary file to confirm FFI entry and context
    let canary_content = format!("run() entered at {}\nctx_len: {}\n", 
        chrono::Utc::now().to_rfc3339(), ctx_len);
    let _ = std::fs::write("/tmp/plugin_canary.txt", canary_content);
    
    // Safety: ctx_json is a pointer to a valid JSON string
    let ctx_slice = std::slice::from_raw_parts(ctx_json, ctx_len);
    let ctx_str = match std::str::from_utf8(ctx_slice) {
        Ok(s) => s,
        Err(_) => return 1,
    };
    let ctx: serde_json::Value = match serde_json::from_str(ctx_str) {
        Ok(v) => v,
        Err(_) => return 1,
    };
    println!("[TestPlugin] Hook: {hook} | Package: {pkg}");
    let output_dir = PathBuf::from(".boltpm/plugins_output");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        let _ = fs::write("/tmp/boltpm_plugin_error.txt", format!("Failed to create output dir: {e}"));
        return 1;
    }
    let output_file = output_dir.join("PLUGIN_TEST");
    if let Err(e) = fs::write(&output_file, format!("plugin ran for {hook}")) {
        let _ = fs::write("/tmp/boltpm_plugin_error.txt", format!("Failed to write output file: {e}"));
        return 1;
    }
    0
} 