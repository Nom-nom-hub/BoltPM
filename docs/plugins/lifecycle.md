
# Plugin Lifecycle Guide

BoltPM plugins are executed at specific points in the installation lifecycle. These points are called "hooks."

## Available Hooks

*   `preinstall`: This hook is run before a package is installed. It's a good place to perform any setup tasks that are required before the package is installed.
*   `postinstall`: This hook is run after a package is installed. It's a good place to perform any cleanup tasks or to run any scripts that are required by the package.
*   `onError`: This hook is run if an error occurs during the installation process. It's a good place to log errors or to perform any other error handling tasks.

## Hook Execution Order

The hooks are executed in the following order:

1.  `preinstall`
2.  Package installation
3.  `postinstall`

If an error occurs at any point during this process, the `onError` hook is executed.

## The `PluginContext`

The `PluginContext` struct is passed to your plugin's `run` function and contains all the information about the current state of the installation, including the current hook.

You can use the `hook` field to determine which hook is currently being executed and to perform different actions based on the hook.

```rust
use boltpm_plugin_api::PluginContext;

#[no_mangle]
pub fn run(context: PluginContext) -> Result<(), String> {
    match context.hook.as_str() {
        "preinstall" => {
            println!("Running pre-install tasks...");
        }
        "postinstall" => {
            println!("Running post-install tasks...");
        }
        "onError" => {
            println!("An error occurred!");
        }
        _ => {}
    }
    Ok(())
}
```
