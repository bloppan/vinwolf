
# Vinwolf

Vinwolf is an independent implementation of the JAM (Join-Accumulate Machine) protocol, designed by Gavin Wood, developed as a research and conformance project with the goal of validating, executing, and experimenting with the official protocol specification as defined in the [Gray Paper](https://github.com/gavofyork/graypaper)

The primary focus of the project is correctness, traceability, and fidelity to the specification, prioritizing clarity and verifiability over premature optimizations.

### Objectives

- Implement the JAM protocol in accordance with the reference specification defined in the Gray Paper.
- Verify behavior using test vectors and conformance test suites.
- Serve as a base for experimentation and as a technical reference.
- Facilitate auditing, review, and external validation of the development.

### Getting Started

#### Prerequisites

- Rust toolchain (stable or nightly)
- Git with submodule support
- Cargo

#### Installation

Clone the repository with submodules:

```bash
git clone https://github.com/bloppan/vinwolf.git
cd vinwolf
git submodule update --init --recursive
```

#### Building the Project

```bash
# Build the project (default: tiny mode for testing)
cargo build

# Build with optimizations
cargo build --release

# Build with full production-sized parameters
cargo build --features full

# Build with RocksDB storage backend
cargo build --features DB

# Check for errors without building
cargo check
```

#### Running Tests

```bash
# Run all tests
cargo test

# Run all tests with full feature set
cargo test --features full

# Run tests for a specific module
cargo test pvm
cargo test assurances

# Run a specific test by name
cargo test <test_name>

# Run with debug output
RUST_LOG=debug cargo test
```

#### Building vinwolf-target for Conformance Testing

The `vinwolf-target` is a specialized binary for conformance and fuzz testing:

```bash
# Build the conformance target
cargo build -p vinwolf-target --release

# Run the conformance target
./target/release/vinwolf-target --fuzz /tmp/jam_target.sock

# Or with default socket
./target/release/vinwolf-target --fuzz
```

### Architecture Overview

Vinwolf is organized as a Cargo workspace with modular components:

- **blockchain/** - Block and state transition logic
  - **block/** - Block structure and header verification
  - **state/** - State management modules (accumulation, authorization, disputes, entropy, recent-history, reports, safrole, services, statistics, validators, state-controller, state-handler)
- **jam-types/** - Core JAM protocol types and data structures
- **jam-utils/** - Shared utilities (codec, bandersnatch-vrf-spec, grandpa, misc, serialization, shuffle, trie)
- **pvm/** - PVM implementation with instruction execution, host calls, and memory management
- **storage/** - Storage abstraction layer (in-memory and RocksDB backends)
- **constants/** - Protocol constants (configurable via features)
- **tools/** - Development and utility tools
- **tests/** - Test suite using external test vectors
- **vinwolf-target/** - Conformance testing binary
- **network/** - Network layer implementation
- **experiments/** - Proof-of-concepts for various technologies


### Testing and conformance

Protocol correctness is validated using external test suites and reference data, included in this repository as Git submodules under the `tests/` directory:

- **`tests/conformance_testing`**
  Location for vinwolf-target binaries built specifically for conformance testing.
- **`tests/jam-conformance`**
  Repository containing disputed JAM conformance reports used to evaluate protocol correctness.
- **`tests/jamtestvectors`**
  Reference JAM test vectors used to validate execution and state transitions.


### Project status

The project is in active development.

The implementation is being built incrementally, with continuous validation through tests and conformance tools.

### Account IDs
- DOT: 1urZ9pp1D6aL6SRwepP9zhU2kzgxJ3dtRodSLe4paJCpLrk
- KUS: GA57XtETAdgMkJU7VwZLKHbWP2bnCN5GpysKjffF9yk3xin

### License

This project is licensed under the Apache License 2.0.

Submodules retain their original licenses.