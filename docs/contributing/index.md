
# Contributing Guide

We welcome contributions from the community! This guide will help you get started with contributing to BoltPM.

## Getting Started

1.  **Fork the repository** on GitHub.
2.  **Clone your fork** to your local machine.
3.  **Create a new branch** for your changes.
4.  **Make your changes** and commit them to your branch.
5.  **Push your branch** to your fork on GitHub.
6.  **Create a pull request** to the main BoltPM repository.

## Development

To build the entire project, run the following command from the root of the repository:

```bash
cargo build --workspace
```

To run the tests, run:

```bash
cargo test --workspace
```

## Code Style

We use `rustfmt` to format our code. Please make sure to run `rustfmt` before submitting a pull request.

We also use `clippy` to lint our code. Please make sure to run `clippy` and address any warnings before submitting a pull request.

## Submitting a Pull Request

When you're ready to submit a pull request, please make sure to:

*   **Provide a clear and descriptive title** for your pull request.
*   **Describe the changes** you've made in the pull request description.
*   **Reference any related issues** in the pull request description.
*   **Make sure all tests pass** before submitting.
