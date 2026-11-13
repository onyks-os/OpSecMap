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
offline sheet. It then evaluates a scenario where the phone starts available but
the email account is explicitly excluded.

## 3. Read the result

The example output ends with:

```text
Scenario access:
phone: initially accessible
Email accessible: false
```

The email account remains inaccessible because the scenario excludes it. The
program does not save any map or scenario to disk. The tests exercise other
cases, including chains, alternatives, exclusions and cycles.

To run those tests yourself:

```bash
cargo test
```

---

## Next Steps

- [Architecture](../explanation/architecture.md) — understand the model and its limits.
- [CLI reference](../reference/cli.md) — see the current command behavior.
