use plugin_api::PluginContext;
use std::fs;
use std::path::PathBuf;

#[no_mangle]
pub extern "C" fn run(ctx: PluginContext) -> i32 {
    let output_file = PathBuf::from(&ctx.output_path).join("PLUGIN_TEST");
    fs::create_dir_all(&ctx.output_path).unwrap();
    fs::write(&output_file, "Plugin executed!").unwrap();
    0
} 