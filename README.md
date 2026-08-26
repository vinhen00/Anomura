# project_mockingbird (anomura)

Master thesis project for improved mocking in Rust.

## Overview

Anomura is a Rust compiler plugin that enables function and method mocking at the crate level. It uses a custom `rustc` driver to intercept and substitute function implementations at compile time, allowing you to write expressive mock definitions using proc macros (`mock_fn`, `mock_method`, `mock_struct`).

## Prerequisites

- Rust nightly (`nightly-2025-08-20`) — installed automatically via `rust-toolchain.toml`
- Components: `rust-src`, `rustc-dev`, `llvm-tools-preview`

## Project Structure

```
crates/
  anomura_plugins/    # cargo-mock plugin (the main entry point)
  anomura_driver/     # Custom rustc driver for mock substitution
  mock-macro/         # Proc macros: mock_fn, mock_method, mock_struct
  context/            # Runtime mock context (expectations, call tracking)
  anomura_definitions/# Shared type definitions
test-suite/
  fns/                # Library with functions to be mocked in tests
  mocks/              # Test crate that uses the mocking macros
```

## Setup

### Install the plugin

The `cargo-mock` plugin and its driver executables must be installed to `~/.cargo/bin/`. From the workspace root:

```bash
cargo install --path crates/anomura_plugins
```

This installs three binaries:
- `cargo-mock` — the cargo subcommand
- `mock_discover_driver_exec` — the discovery pass driver
- `mock_substitute_driver_exec` — the substitution pass driver

If you've previously installed an older version, use `--force`:

```bash
cargo install --path crates/anomura_plugins --force
```

> **Note:** You must reinstall after making changes to `anomura_plugins` or `anomura_driver`.

## Running the Tests

You must run `cargo mock` from inside the test crate directory (`test-suite/mocks/`), **not** from the workspace root. The plugin uses `cargo metadata` to resolve the root package, which doesn't work in a virtual workspace.

```bash
cd test-suite/mocks
cargo mock test
```

Running from the workspace root (`cargo mock test -p mocks`) will **not** work — it panics because a virtual workspace has no root package.

### Run with logging

For debug output from the plugin passes:

```bash
cd test-suite/mocks
RUST_LOG=debug cargo mock test
```

## How It Works

When you run `cargo mock test`, the plugin:

1. **Discover pass** — compiles the test crate, finds all `mock_fn!`/`mock_method!`/`mock_struct!` invocations, and resolves which external crates contain the functions being mocked.
2. **Substitution pass** — recompiles those crates using the custom driver, replacing the real function bodies with the mocked implementations that route through the `context` crate at runtime.

## Writing Tests

Tests live in `test-suite/mocks/src/main.rs`. A basic test looks like:

```rust
#[test]
fn my_mock_test() {
    mock_fn!(
        fns,                        // crate containing the function
        fn some_function(x: u32) {
            default_return(42);     // what to return when called
            expect(true, once());   // expectation: called exactly once
        }
    );
    context::finish_building_context();

    let result = fns::some_function(10);
    assert_eq!(result, 42);
}
```

The `fns` crate (`test-suite/fns/`) provides the real function signatures that get mocked during tests.

## Troubleshooting

- **`should be some root` panic**: You're running `cargo mock` from the workspace root. `cd` into `test-suite/mocks/` first.
- **Plugin not found / old version**: Reinstall with `cargo install --path crates/anomura_plugins --force`.
- **Wrong toolchain errors**: Make sure the nightly version matches across `rust-toolchain.toml` files. Currently pinned to `nightly-2025-08-20`.
- **Link errors about `context`**: The substitution pass automatically links the `context` crate. Make sure it's built as part of the workspace.
