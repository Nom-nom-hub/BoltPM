use plugin_api::{PluginContext, PluginResult};

#[no_mangle]
pub extern "C" fn run(_ctx: PluginContext) -> i32 {
    // Example plugin logic
    println!("Sample plugin executed!");
    0 // Success
} 