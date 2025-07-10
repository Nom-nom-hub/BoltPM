
# BoltPM Registry

BoltPM includes a self-hosted private registry, allowing you to manage your own private packages.

## Key Features

*   **Self-Hosted:** You have full control over your package registry.
*   **Token-Based Authentication:** Secure your registry with authentication tokens.
*   **CORS Enabled:** Allows the GUI to interact with the registry.
*   **Simple JSON Index:** The `packages.json` file provides a simple and easy-to-understand index of your packages.

## Getting Started

To run the registry, navigate to the `registry` directory and run:

```bash
cargo run
```

The registry will run at `http://localhost:4000`.
