# Reference: Command Line Interface

The CLI currently runs one built-in demonstration. It accepts no map or scenario
files. This page documents the behavior implemented in `src/main.rs`.

---

## Invocation

```sh
cargo run --
cargo run -- --version
```

| Argument | Behavior |
| :------- | :------- |
| `--version`, `-V` | Print `opsecmap 0.1.0` and exit. |
| No argument | Build and evaluate the fixed example, then print its summary. |

Other arguments are currently ignored; there is no `--help` handler or input-file
option yet. The demonstration builds four fictional resources and two routes to
`email`, then evaluates two scenarios: `recovery-sheet` available with `phone`
excluded, and only `password-store` available. For each scenario it prints every
resource with its status (available at start, accessible via a route, excluded,
not accessible with the documented procedures, or undetermined) and every blocked
route with its missing prerequisites.

---

## Exit Codes

| Code | Meaning |
| :--- | :------ |
| `0`  | Success. |
| `1`  | The built-in example could not be constructed or evaluated. |
