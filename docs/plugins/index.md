
# BoltPM Plugin System

The BoltPM plugin system allows you to extend the functionality of BoltPM by creating your own custom plugins.

## Key Features

*   **Dynamic Hooks:** Plugins can hook into the `preinstall`, `postinstall`, and `onError` stages of the installation lifecycle.
*   **WASM and Native Plugins:** Write plugins in Rust and compile them to either WebAssembly or native libraries.
*   **Simple API:** The plugin API is designed to be simple and easy to use.

## Getting Started

To create a new plugin, you'll need to implement the `run` function and build it as a `cdylib`. Then, place the compiled library in the `.boltpm/plugins/` directory.

For more detailed information on creating and using plugins, see the [Plugin Authoring Guide](authoring.md) and the [Plugin Lifecycle Guide](lifecycle.md).
