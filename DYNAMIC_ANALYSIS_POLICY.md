# Dynamic analysis

The current repository uses `proptest` to generate inputs for identifier
validation. Run the tests locally with `cargo test`. The
[`fuzzing.yml`](.github/workflows/fuzzing.yml) workflow runs the project's
property-test target on pull requests and on a weekly schedule. It is
property-based testing, not coverage-guided fuzzing of an input parser.

The CLI does not yet read map files or other untrusted application input, and
there is no deserializer or recovery-procedure parser to fuzz. When those input
boundaries are added, the relevant generators and fuzz targets should be added
alongside them.

If a generated case reveals a security issue, report it through
[SECURITY.md](SECURITY.md), keep the failing input as a regression case without
including real secrets, and verify the fix with the test suite.
