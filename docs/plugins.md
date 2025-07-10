# BoltPM Plugin System

## ABI Compatibility

When installing a plugin (WASM), BoltPM checks that the plugin exports the required ABI version symbol: `_boltpm_plugin_v1`. If this symbol is missing, installation will fail with an error. This ensures only compatible plugins are installed.

## Local Plugin Management

### List Installed Plugins

```
boltpm plugin list
```
Lists all plugins currently installed in the local `.boltpm/plugins` directory.

### Uninstall a Plugin

```
boltpm plugin uninstall <plugin_name>
```
Removes the specified plugin from the local plugins directory.

## Remote Plugin Install

When running:

```
boltpm plugin install <plugin_name>
```
BoltPM will:
- Download the plugin from the remote registry
- Verify the plugin's hash
- Check for the required ABI version symbol (`_boltpm_plugin_v1` for WASM)
- Install the plugin if all checks pass

If any check fails, installation is aborted with a clear error message.

---

For more details, see the main documentation or run `boltpm plugin --help`. 