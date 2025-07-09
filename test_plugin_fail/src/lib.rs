#[no_mangle]
pub extern "C" fn run(_ctx_json: *const u8, _ctx_len: usize) -> i32 {
    // Immediately fail
    1
} 