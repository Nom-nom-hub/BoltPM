
# Architecture Overview

BoltPM is a monorepo project composed of several Rust crates that work together to provide a fast and modern package management experience.

## Core Components

*   **`boltpm` (CLI):** The command-line interface for BoltPM. It's responsible for parsing user commands, managing the installation lifecycle, and interacting with the registry.
*   **`boltpm-gui` (GUI):** A Tauri-based desktop application that provides a graphical interface for BoltPM.
*   **`boltpm-registry` (Registry):** A self-hosted package registry that stores and serves your private packages.
*   **`boltpm-plugin-api` (Plugin API):** A shared crate that defines the API for BoltPM plugins.

## Project Structure

The project is organized into the following directories:

*   `cli/`: The source code for the BoltPM CLI.
*   `gui/`: The source code for the BoltPM GUI.
*   `registry/`: The source code for the BoltPM registry.
*   `plugin_api/`: The source code for the plugin API.
*   `plugins/`: Example plugins.
*   `docs/`: Project documentation.

## Technology Stack

*   **Rust:** The primary programming language for all components.
*   **Tauri:** Used to build the cross-platform desktop GUI.
*   **Axum:** A web application framework used to build the registry.
*   **Clap:** A command-line argument parser for the CLI.
*   **Serde:** A framework for serializing and deserializing Rust data structures.
