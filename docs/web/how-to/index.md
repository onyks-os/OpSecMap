# How-To Guides

## Evaluate a map from a file

1. Write one command per line in a text file, using fictional data. The
   [CLI reference](../reference/cli.md) lists the commands.
2. Run `opsecmap < commands.txt`, or `cargo run < commands.txt` from the repository.
3. Read the report of each `evaluate` line. Errors appear on standard error with
   their line number; the exit code is `1` if any command failed.

`tests/fixtures/phone-lost.txt` in the repository is a complete example.

## Keep a map you built interactively

Run `list` before `quit`. Its output is the map written as commands; save it to a
file and run that file later to rebuild the same map. The file is plain text, so
keep it fictional until encrypted storage exists. The
[roadmap](https://github.com/onyks-os/OpSecMap/blob/main/ROADMAP.md) tracks
persistence and encryption.
