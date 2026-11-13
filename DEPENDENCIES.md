# Dependencies

[`Cargo.toml`](Cargo.toml) declares the direct Rust dependencies;
[`Cargo.lock`](Cargo.lock) records the versions resolved for this checkout.

| Kind | Dependency | Declared constraint | Purpose |
| :--- | :--- | :--- | :--- |
| Runtime crate | None | — | The current library and binary use the Rust standard library. |
| Development crate | `proptest` | `1` | Property tests for validated identifiers. |
| Build toolchain | Rust and Cargo | Rust 1.75 declared in `Cargo.toml` | Build and test the crate. |

The CLI example does not require an application service or database. Cargo may
contact its configured registry to resolve or download build dependencies; use
`cargo test --offline` when the dependencies are already cached.

Before adding a dependency, document why it is needed, review its maintenance
and license, and update this inventory. Automated dependency and vulnerability
checks are configured in the repository workflows, but a passing workflow is
not a substitute for reviewing the dependency's use in this project.
