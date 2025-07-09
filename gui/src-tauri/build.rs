use std::process::Command;

fn main() {
    // Clean up AppleDouble files (._*) in the build output
    let _ = Command::new("find")
        .args(&[".", "-name", "._*", "-type", "f", "-delete"])
        .status();

    tauri_build::build();
}
