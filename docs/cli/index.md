
# BoltPM CLI

The BoltPM command-line interface (CLI) is the primary way to interact with your projects. It's designed to be fast, intuitive, and fully featured.

## Key Features

*   **Fast, Concurrent Operations:** BoltPM leverages Rust's performance and concurrency to execute commands quickly.
*   **Comprehensive Command Set:** Includes all the commands you'd expect from a modern package manager: `install`, `remove`, `update`, `run`, `init`, `link`, and more.
*   **Lockfile for Deterministic Installs:** The `bolt.lock` file ensures that every install results in the same dependency tree, every time.
*   **Plugin System:** Extend BoltPM's functionality with custom plugins that hook into the installation lifecycle.
*   **Migration Scripts:** Easily migrate your existing projects from NPM, Yarn, or PNPM.

## Getting Started

To get started with the BoltPM CLI, run the `init` command in your project's root directory:

```bash
boltpm init
```

This will create a `package.json` file if one doesn't already exist.

## Commands

For a full list of commands and their options, run:

```bash
boltpm --help
```
