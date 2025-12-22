
# Vinwolf

Vinwolf is an independent implementation of the JAM (Join-Accumulate Machine) protocol, designed by Gavin Wood, developed as a research and conformance project with the goal of validating, executing, and experimenting with the official protocol specification as defined in the [Gray Paper](https://github.com/gavofyork/graypaper)

The primary focus of the project is correctness, traceability, and fidelity to the specification, prioritizing clarity and verifiability over premature optimizations.

### Objectives

- Implement the JAM protocol in accordance with the reference specification defined in the Gray Paper.
- Verify behavior using test vectors and conformance test suites.
- Serve as a base for experimentation and as a technical reference.
- Facilitate auditing, review, and external validation of the development.

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