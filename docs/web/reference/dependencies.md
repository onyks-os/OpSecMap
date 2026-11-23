# Reference: Dependencies

The crate's direct dependencies are declared in
[`Cargo.toml`](https://github.com/onyks-os/OpSecMap/blob/main/Cargo.toml).

The Rust package has two runtime crate dependencies, `serde` and `serde_json`,
which read and write the map file format. It uses `proptest` as a development
dependency for property tests. Building from source requires
Rust 1.88 or newer and Cargo; see the [quickstart](../tutorials/quickstart.md).
