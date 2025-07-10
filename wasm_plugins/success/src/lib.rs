// Minimal BoltPM WASM plugin (WASI + host logging)

#[no_mangle]
pub extern "C" fn _boltpm_plugin_v1() {}

#[no_mangle]
pub extern "C" fn _run() {
    // WASI stdout
    println!("[PLUGIN] Hello from WASI stdout!");

    // Host logging function
    let msg = "[PLUGIN] Hello from host logging!";
    unsafe {
        boltpm_log(msg.as_ptr(), msg.len());
    }
}

extern "C" {
    fn boltpm_log(ptr: *const u8, len: usize);
} 