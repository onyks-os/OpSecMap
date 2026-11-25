# Reference: Dependencies

The crate's direct dependencies are declared in
[`Cargo.toml`](https://github.com/onyks-os/OpSecMap/blob/main/Cargo.toml).

The Rust package's runtime crates are `serde` and `serde_json`, which read and
write the map file format, and `argon2`, `chacha20poly1305`, `getrandom`,
`rpassword` and `zeroize`, for encrypted map files. It uses `proptest` as a development
dependency for property tests. Building from source requires
Rust 1.88 or newer and Cargo; see the [quickstart](../tutorials/quickstart.md).
