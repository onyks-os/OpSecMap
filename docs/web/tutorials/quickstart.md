# Tutorial: Quickstart

This tutorial runs the example currently built into the CLI. The application does
not yet accept a personal map or scenario.

**You will need:** Rust 1.75 or newer with Cargo.

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

## 2. Run the example

```bash
cargo run --
```

The program creates four sample resources and two alternative routes to an email
account. Login requires both a password source and a phone; recovery requires an
offline sheet. The routes to the email account are declared completely
documented. It then evaluates two scenarios: a lost phone with the recovery sheet
available, and only the password manager available.

## 3. Read the result

The example prints:

```text
Scenario: Phone lost
email (Test email): accessible via email-recovery
password-store (Test password manager): undetermined: documentation incomplete or not declared
phone (Test phone): excluded by the scenario
recovery-sheet (Test recovery sheet): available at start
Blocked routes: none

Scenario: Only password manager
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
coverage are reported as undetermined, not as impossible. The program does not
save any map or scenario to disk. The tests exercise other
cases, including chains, alternatives, exclusions and cycles.

To run those tests yourself:

```bash
cargo test
```

---

## Next Steps

- [Architecture](../explanation/architecture.md) — understand the model and its limits.
- [CLI reference](../reference/cli.md) — see the current command behavior.
