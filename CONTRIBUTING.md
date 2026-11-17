# Contributing to OpSecMap

OpSecMap is in early development. The Rust library models account access and
recovery dependencies; the CLI reads map commands from standard input. Issues
that clarify the model, identify a reproducible bug, or improve documentation
are welcome. Please use [SECURITY.md](SECURITY.md) for vulnerability reports,
and describe examples with fictional resources, never your real accounts.

## Get started

You need Cargo and a Rust toolchain. `Cargo.toml` declares Rust 1.88 as the
minimum version.

```sh
git clone https://github.com/onyks-os/OpSecMap.git
cd OpSecMap
cargo test
cargo fmt --all -- --check
```

The repository's CI also runs Clippy, the test suite on stable, beta and the
minimum Rust version, property tests, dependency checks, and a release build.
Run `cargo clippy --all-targets --all-features -- -D warnings` when working on
Rust code, and `make fuzz` when you change a parser or validator. The test suite
needs no external service; CLI tests run the binary on fixture files in
`tests/fixtures/`, which must contain fictional data only.

## Make a change

1. Open an issue for a larger design change so its assumptions can be discussed
   before implementation. Small fixes can go straight to a pull request.
2. Keep the library model independent of CLI formatting. Add tests for behavior
   that changes, especially AND/OR routes, exclusions, cycles, and explanations.
3. Update the relevant public documentation and the
   [changelog](CHANGELOG.md). The [documentation policy](docs/documentation-policy.md)
   lists the repository's review expectations.
4. Run the checks above and describe any remaining failure in the pull request.

The public Rust API is still evolving. Keep IDs stable across label changes and
make validation failures leave the map unchanged.

## Commits and pull requests

The repository uses the [Developer Certificate of Origin](https://developercertificate.org/).
Sign off each commit with your real name and email:

```sh
git commit -s -m "docs: clarify current evaluator behavior"
```

The DCO workflow checks pull requests. Conventional commit prefixes such as
`feat:`, `fix:`, `test:`, and `docs:` make the history easier to scan. Include a
short explanation of what changed, why, and how you verified it.

By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).
