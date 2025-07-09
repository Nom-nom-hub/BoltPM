use std::ffi::CStr;
use std::os::raw::c_char;
use std::fs;
use std::path::PathBuf;

#[no_mangle]
pub extern "C" fn run(ctx_json: *const u8, ctx_len: usize) -> i32 {
    // Safety: ctx_json is a pointer to a valid JSON string
    let ctx_slice = unsafe { std::slice::from_raw_parts(ctx_json, ctx_len) };
    let ctx_str = match std::str::from_utf8(ctx_slice) {
        Ok(s) => s,
        Err(_) => return 1,
    };
    let ctx: serde_json::Value = match serde_json::from_str(ctx_str) {
        Ok(v) => v,
        Err(_) => return 1,
    };
    let hook = ctx["hook"].as_str().unwrap_or("unknown");
    let pkg = ctx["package_name"].as_str().unwrap_or("");
    println!("[TestPlugin] Hook: {} | Package: {}", hook, pkg);
    let output_dir = PathBuf::from(".boltpm/plugins_output");
    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!("[TestPlugin] Failed to create output directory: {}", e);
        return 1;
    }
    let output_file = output_dir.join(hook);
    if let Err(e) = fs::write(&output_file, format!("plugin ran for {}", hook)) {
        eprintln!("[TestPlugin] Failed to write plugin output file: {}", e);
        return 1;
    }
    0
} 