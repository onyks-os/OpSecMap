# Dynamic analysis

The project uses property-based testing with `proptest`. `make fuzz` runs every
property test:

| Input boundary | Properties | Location |
| :--- | :--- | :--- |
| Resource and route IDs | Accept exactly the non-blank values | `tests/properties.rs` |
| CLI command lines | Never panic on any line; a valid `resource` line keeps its ID and label | `src/cli/command.rs`, module `properties` |

The [`fuzzing.yml`](.github/workflows/fuzzing.yml) workflow runs `make fuzz` on
pull requests with 1,024 cases per property, and weekly with 100,000. This is
property-based testing, not coverage-guided fuzzing.

When a new input boundary is added, its properties are added in the same change:
the planned map-file reader must never panic on arbitrary bytes, and must reject
any file that the model's own validation would reject. A coverage-guided fuzz
target (`cargo fuzz`) is worth adding once the program reads files, since a file
format is more exposed than a line-based command.

If a generated case reveals a defect, keep the failing input as a regression test
without real data, fix it, and report security-relevant cases through
[SECURITY.md](SECURITY.md).
