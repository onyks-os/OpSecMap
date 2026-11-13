# Source verification

OpSecMap has no published GitHub release or tagged artifact at present. There is
therefore no release checksum, signature, or packaged binary to verify. The
release workflows in the repository describe a future publishing path; their
presence does not establish that an artifact has been published or verified.

To check the current source tree locally, use a Rust toolchain and Cargo:

```sh
cargo test
cargo fmt --all -- --check
```

These commands test the implementation and formatting of the checkout you have.
They do not authenticate a downloaded release. When release artifacts are first
published, this page will document their exact assets, signing identity, and
verification commands.
