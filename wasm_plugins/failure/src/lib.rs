extern crate alloc;
use core::slice;
use core::str;

#[link(wasm_import_module = "env")]
extern "C" {
    fn host_write_file(ptr: i32, len: i32);
}

#[no_mangle]
pub extern "C" fn run(_ptr: i32, _len: i32) -> i32 {
    let msg = b"wasm plugin ran";
    unsafe { host_write_file(msg.as_ptr() as i32, msg.len() as i32); }
    1
} 