# Tutorial: Quickstart

This tutorial runs the demo built into the CLI, then describes a small map with
commands. Use fictional data: the CLI does not save or encrypt anything.

**You will need:** Rust 1.88 or newer with Cargo.

---

## 1. Get the source

```bash
git clone https://github.com/onyks-os/OpSecMap.git
cd OpSecMap
```

Check that Cargo can build the project:

```bash
cargo run -- --version
```

You should see `opsecmap 0.1.0`.

## 2. Run the demo

```bash
cargo run -- --demo
```

The program creates four sample resources and two alternative routes to an email
account. Login requires both a password source and a phone; recovery requires an
offline sheet. The routes to the email account are declared completely
documented. It saves the map to a temporary file, opens that file again and
deletes it, so the scenarios run on a map read back from disk. It then evaluates
two scenarios: a lost phone with the recovery sheet available, and only the
password manager available.

## 3. Read the result

The example prints:

```text
Scenario: Phone lost
Initial: recovery-sheet; excluded: phone
email (Test email): accessible via email-recovery
password-store (Test password manager): undetermined: documentation incomplete or not declared
phone (Test phone): excluded by the scenario
recovery-sheet (Test recovery sheet): available at start
Blocked routes: none

Scenario: Only password manager
Initial: password-store; excluded: none
email (Test email): not accessible with the documented procedures
password-store (Test password manager): available at start
phone (Test phone): undetermined: documentation incomplete or not declared
recovery-sheet (Test recovery sheet): undetermined: documentation incomplete or not declared
email-login -> email: missing phone
email-recovery -> email: missing recovery-sheet
```

With the phone lost, login is blocked but recovery through the sheet still
reaches the email account. With only the password manager, both routes are
blocked and each line names what is missing. The email account is reported as a
dead end only because its coverage is declared complete; resources with unknown
coverage are reported as undetermined, not as impossible. The tests exercise other
cases, including chains, alternatives, exclusions and cycles.

To run those tests yourself:

```bash
cargo test
```

## 4. Describe a map yourself

Start the CLI without arguments and type commands. `#` starts a comment.

```text
$ cargo run
OpSecMap v0.1.0. Type "help" for the list of commands, "quit" to exit.
> resource email Test email
> resource phone Test phone
> resource backup-codes Test backup codes
> route email-login email phone Email login
> route email-codes email backup-codes Backup codes
> evaluate backup-codes phone
Initial: backup-codes; excluded: phone
email (Test email): accessible via email-codes
phone (Test phone): excluded by the scenario
backup-codes (Test backup codes): available at start
Blocked routes: none
> quit
```

To repeat the same session later, put the commands in a file and run
`cargo run < commands.txt`. The `list` command prints the current map as
commands, ready to save.

To keep the map itself, type `save my-map.json` before `quit`; in a later session,
`open my-map.json` brings it back. The file is plain JSON and is not encrypted,
so keep using fictional data.

---

## Next Steps

- [Architecture](../explanation/architecture.md) — understand the model and its limits.
- [CLI reference](../reference/cli.md) — see the current command behavior.
