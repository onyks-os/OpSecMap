# Static analysis

The repository configures Rust formatting and Clippy in
[`ci.yml`](.github/workflows/ci.yml), plus CodeQL in
[`codeql.yml`](.github/workflows/codeql.yml). These workflows run on pull
requests to `main`; the configuration itself does not guarantee that every
finding has been resolved or that branch protection is enabled.

Local checks for Rust changes:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

Review a finding in its code context. Fix confirmed defects and document the
reason for any suppression. Security-relevant findings should be reported or
handled through [SECURITY.md](SECURITY.md).
