# Dependencies

[`Cargo.toml`](Cargo.toml) declares the direct Rust dependencies;
[`Cargo.lock`](Cargo.lock) records the versions resolved for this checkout.

| Kind | Dependency | Declared constraint | Purpose |
| :--- | :--- | :--- | :--- |
| Runtime crate | `argon2` | `0.6`, default features off, `alloc` | Derives the encryption key from the map password (Argon2id). |
| Runtime crate | `chacha20poly1305` | `0.11`, default features off, `alloc` | Encrypts and authenticates map files (XChaCha20-Poly1305). |
| Runtime crate | `getrandom` | `0.4` | Random salt and nonce from the operating system. |
| Runtime crate | `rpassword` | `7` | Reads the map password from the terminal without echo. |
| Runtime crate | `serde` | `1` | Serialization of the map file format. |
| Runtime crate | `serde_json` | `1` | Reading and writing the map file as JSON. |
| Runtime crate | `zeroize` | `1` | Clears passwords, keys and decrypted maps from memory. |
| Development crate | `proptest` | `1` | Property tests for validated identifiers. |
| Build toolchain | Rust and Cargo | Rust 1.88 declared in `Cargo.toml` | Build and test the crate. |

The CLI example does not require an application service or database. Saving and
opening map files uses the Rust standard library for file access. The
encryption crates implement encrypted map files; see
[ADR 0002](docs/decisions/0002-encrypt-map-files-with-argon2id-and-xchacha20-poly1305.md).
`Cargo.toml` builds `argon2` and its `blake2` dependency optimized in debug
builds too, so tests that derive a key stay fast. Cargo may
contact its configured registry to resolve or download build dependencies; use
`cargo test --offline` when the dependencies are already cached.

Before adding a dependency, document why it is needed, review its maintenance
and license, and update this inventory. Automated dependency and vulnerability
checks are configured in the repository workflows, but a passing workflow is
not a substitute for reviewing the dependency's use in this project.
