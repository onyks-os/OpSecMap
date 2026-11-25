# How-To Guides

## Evaluate a map from a file

1. Write one command per line in a text file, using fictional data. The
   [CLI reference](../reference/cli.md) lists the commands.
2. Run `opsecmap < commands.txt`, or `cargo run < commands.txt` from the repository.
3. Read the report of each `evaluate` line. Errors appear on standard error with
   their line number; the exit code is `1` if any command failed.

`tests/fixtures/phone-lost.txt` in the repository is a complete example.

## Keep a map you built interactively

Type `save <path>` before `quit`: the map is written encrypted with a password
you choose, and `open <path>` brings it back in a later session. There is no
password recovery, so keep a copy of the file in a second place and the password
separately.

`list` prints the map written as commands instead; a file of those commands, or
one written by `export`, is plain text and readable by anyone with a copy.

## Protect open maps

An open map is held in memory, which the system can write to disk. See
[Harden the computer that opens your maps](harden-your-computer.md).
