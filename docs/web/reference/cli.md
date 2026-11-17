# Reference: Command Line Interface

The CLI reads commands from standard input, one per line, and keeps the map in
memory only. It works interactively or with a file of commands. Nothing is saved
to disk, and nothing is encrypted: use fictional data, and keep passwords and
recovery codes out of command files.

---

## Invocation

```sh
opsecmap                     # read commands from standard input
opsecmap < commands.txt      # run a file of commands
opsecmap --demo              # evaluate two scenarios on a fictional map
opsecmap --help
opsecmap --version
```

From the repository, use `cargo run --` in place of `opsecmap`, for example
`cargo run -- --demo` or `cargo run < tests/fixtures/phone-lost.txt`.

| Argument | Behavior |
| :------- | :------- |
| None | Read commands from standard input until `quit` or the end of input. In a terminal, print a short greeting and a `>` prompt. |
| `--demo` | Build the fictional example map and evaluate two scenarios. |
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
| `help` | `help` | Print the command reference. |
| `quit` | `quit` | Stop reading commands. |

IDs cannot contain a comma or be `-`, because the list syntax reserves them.

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
