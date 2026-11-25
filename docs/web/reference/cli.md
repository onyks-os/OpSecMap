# Reference: Command Line Interface

The CLI reads commands from standard input, one per line, and keeps the map in
memory. It works interactively or with a file of commands. `save` writes the map
to a password-encrypted file, `export` writes it as plain JSON, and `open` reads
either back. **Command files and exported files are plain text:** keep passwords
and recovery codes out of them. The encryption has not been independently
reviewed; until the first release, prefer fictional maps.

---

## Invocation

```sh
opsecmap                     # read commands from standard input
opsecmap < commands.txt      # run a file of commands
opsecmap --demo              # export, reopen and evaluate a fictional map
opsecmap --help
opsecmap --version
```

From the repository, use `cargo run --` in place of `opsecmap`, for example
`cargo run -- --demo` or `cargo run < tests/fixtures/phone-lost.txt`.

| Argument | Behavior |
| :------- | :------- |
| None | Read commands from standard input until `quit` or the end of input. In a terminal, print a short greeting and a `>` prompt. |
| `--demo` | Build the fictional example map, export it to a temporary file, open that file in a new map, delete the file and evaluate two scenarios on the reopened map. It never asks for a password. |
| `--help`, `-h` | Print usage and exit. |
| `--version`, `-V` | Print `opsecmap 0.1.0` and exit. |
| Anything else | Print usage on standard error and exit with code `2`. |

---

## Commands

Words are separated by spaces. ID lists are separated by commas without spaces,
and `-` stands for an empty list. Blank lines and lines starting with `#` are
ignored. A label is the rest of the line.

| Command | Example | Effect |
| :--- | :--- | :--- |
| `resource <id> <label...>` | `resource phone Test phone` | Add a resource. |
| `route <id> <target> <prereq,...> <label...>` | `route email-login email password-store,phone Email login` | Add a route. All prerequisites are required together; several routes to one target are alternatives. |
| `coverage <id> <unknown\|incomplete\|complete>` | `coverage email complete` | State how completely the routes to a resource are documented. New resources start as `unknown`. |
| `evaluate <initial,...\|-> <excluded,...\|->` | `evaluate recovery-sheet phone` | Evaluate a scenario and print the report. |
| `remove resource <id>` | `remove resource phone` | Remove a resource. Refused while any route uses it as target or prerequisite. |
| `remove route <id>` | `remove route email-login` | Remove a route. |
| `list` | `list` | Print the map as commands. Running that output again rebuilds the same map. |
| `save <path>` | `save maps/test.opsecmap` | Ask for a new password twice, encrypt the map, write it to the file, replacing any file already there, and print `saved 4 resources and 2 routes to maps/test.opsecmap (encrypted)`. |
| `export <path>` | `export maps/test.json` | Write the map as plain JSON, replacing any file already there, and print `exported 4 resources and 2 routes to maps/test.json (plain text, NOT ENCRYPTED)`. |
| `open <path>` | `open maps/test.opsecmap` | Replace the map with the one in the file, asking for the password if it is encrypted, and print `opened 4 resources and 2 routes from maps/test.opsecmap (encrypted)`, or `(plain text)` for an exported file. |
| `help` | `help` | Print the command reference. |
| `quit` | `quit` | Stop reading commands. |

IDs cannot contain a comma or be `-`, because the list syntax reserves them.

---

## Map files

The path given to `save`, `export` and `open` is the rest of the line exactly as typed,
apart from spaces at its start and end: `save my map.json` writes `my map.json`,
and two spaces in a row stay two spaces. A relative path starts from the
directory the program runs in. The formats are specified in
[encrypted-format.md](https://github.com/onyks-os/OpSecMap/blob/main/docs/encrypted-format.md)
and
[map-format.md](https://github.com/onyks-os/OpSecMap/blob/main/docs/map-format.md).

`save` and `export` write the whole file to `<path>.tmp` in the same directory,
flush it to disk and rename it over `<path>`. If any step fails, the temporary
file is removed and an existing file at `<path>` is left as it was. The
temporary file of `save` is already encrypted.

### Passwords

The password is read from the terminal with echo turned off, not from standard
input: it works with `opsecmap < commands.txt`, and it never appears on screen,
in the map or in an error message.

- `save` first checks that `<path>.tmp` can be created, so a missing folder or a
  denied permission is reported before you type anything. It then asks
  `New password:` and `Repeat password:`, and refuses an empty password or two
  different ones.
- `open` asks `Password:` only for an encrypted file, after the size and header
  checks. A wrong password and a modified file give the same error,
  `wrong password, or the map file is damaged or was modified`.
- When the environment variable `OPSECMAP_PASSWORD` is set, its value is used
  instead of the terminal, for both questions. It is meant for tests and
  fictional maps: other processes of the same user can read it.

There is no password recovery. A lost password makes the map unreadable; see
the [backup advice](https://github.com/onyks-os/OpSecMap/blob/main/docs/encrypted-format.md#backups).

`open` replaces the map only after the whole file has been read and validated.
It refuses, and leaves the current map unchanged, when the file:

- cannot be read, for example because it does not exist;
- is larger than 1 MiB (1,048,576 bytes);
- is encrypted and the password is wrong, or the file was modified, is
  truncated or has an unsupported encrypted format version;
- is not valid UTF-8 or not valid JSON;
- is not an OpSecMap map file, or has a version this build does not read;
- breaks a rule of the map, such as a route that names an absent resource.

---

## Evaluation report

`evaluate` prints the scenario's assumptions, one line per resource in map order,
then the blocked routes:

```text
Initial: password-store; excluded: none
email (Test email): not accessible with the documented procedures
password-store (Test password manager): available at start
phone (Test phone): undetermined: documentation incomplete or not declared
recovery-sheet (Test recovery sheet): undetermined: documentation incomplete or not declared
email-login -> email: missing phone
email-recovery -> email: missing recovery-sheet
```

| Status | Meaning |
| :--- | :--- |
| `excluded by the scenario` | The scenario rules this resource out. |
| `available at start` | The scenario declares it available. |
| `accessible via <route>` | A documented route works; it is the route chosen as explanation, not necessarily the only one. |
| `not accessible with the documented procedures` | No documented route works and coverage is `complete`. This holds for this map only. |
| `undetermined: documentation incomplete or not declared` | No documented route works, but coverage is `unknown` or `incomplete`. |

Blocked routes are listed only for targets that remain unresolved, with the
prerequisites still missing. `Blocked routes: none` means there are none.

---

## Errors

A command that fails is reported on standard error with its line number, for
example `error (line 3): resource "phone" already exists`, and leaves the map
unchanged. Reading continues with the next line.

---

## Exit Codes

| Code | Meaning |
| :--- | :------ |
| `0`  | Every command succeeded, or a flag completed. |
| `1`  | At least one command failed, input could not be read, or the demo failed. |
| `2`  | Unknown argument. |
