# Dynamic analysis

The project uses property-based testing with `proptest`. `make fuzz` runs every
property test:

| Input boundary | Properties | Location |
| :--- | :--- | :--- |
| Resource and route IDs | Accept exactly the non-blank values | `tests/properties.rs` |
| CLI command lines | Never panic on any line; a valid `resource` line keeps its ID and label | `src/cli/command.rs`, module `properties` |
| Plain-text map files | `from_json` never panics on any text or on map-shaped JSON | `tests/properties.rs` |
| Encrypted map files | `decrypt` never panics or succeeds on arbitrary bytes, and gives the error of the first failed check after the magic | `tests/properties.rs` |

The [`fuzzing.yml`](.github/workflows/fuzzing.yml) workflow runs `make fuzz` on
pull requests with 1,024 cases per property, and weekly with 100,000. This is
property-based testing, not coverage-guided fuzzing.

A property whose every case runs Argon2 (64 MiB) would take minutes, so the
tamper check runs as an exhaustive unit test instead: every byte of a valid
encrypted file is changed in turn and must be refused
(`crypto::tests::changing_any_byte_of_the_file_is_rejected`).

When a new input boundary is added, its properties are added in the same change.
A coverage-guided fuzz target (`cargo fuzz`) for `decrypt` and `from_json` is
still to be added: a file format is more exposed than a line-based command.

If a generated case reveals a defect, keep the failing input as a regression test
without real data, fix it, and report security-relevant cases through
[SECURITY.md](SECURITY.md).
