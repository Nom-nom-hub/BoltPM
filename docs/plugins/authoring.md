
# Plugin Authoring Guide

This guide will walk you through the process of creating your own BoltPM plugins.

## Plugin Structure

A BoltPM plugin is a dynamic library (e.g., `.so`, `.dylib`, `.dll`) or a WebAssembly module (`.wasm`) that exports a single function:

```rust
pub fn run(context: PluginContext) -> Result<(), String>;
```

## The `PluginContext`

The `PluginContext` struct is passed to your plugin's `run` function and contains all the information about the current state of the installation. It includes:

*   `cwd`: The current working directory.
*   `hook`: The current lifecycle hook (e.g., `preinstall`, `postinstall`).
*   `package_name`: The name of the package being installed.
*   `package_version`: The version of the package being installed.

## Creating a Native Plugin

1.  **Create a new Rust library crate:**

    ```bash
    cargo new --lib my_plugin
    ```

2.  **Add the following to your `Cargo.toml`:**

    ```toml
    [lib]
    crate-type = ["cdylib"]
    ```

3.  **Implement the `run` function in `src/lib.rs`:**

    ```rust
    use boltpm_plugin_api::PluginContext;

    #[no_mangle]
    pub fn run(context: PluginContext) -> Result<(), String> {
        println!("Hello from my plugin!");
        Ok(())
    }
    ```

4.  **Build the plugin:**

    ```bash
    cargo build --release
    ```

5.  **Copy the compiled library** (e.g., `target/release/libmy_plugin.so`) to the `.boltpm/plugins/` directory in your project.

## Creating a WASM Plugin

The process for creating a WASM plugin is similar to creating a native plugin, but with a few key differences:

1.  **Set the crate type to `cdylib`** in your `Cargo.toml`.
2.  **Add the `wasm-bindgen` dependency.**
3.  **Use the `#[wasm_bindgen]` attribute** on your `run` function.
4.  **Build the plugin with `wasm-pack`** or a similar tool.

## Best Practices

*   **Keep your plugins small and focused.**
*   **Avoid making network requests or other long-running operations** in your plugins, as this can slow down the installation process.
*   **Use the `onError` hook** to handle errors gracefully.
