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
option yet. The demonstration prints four resources, two routes, and a scenario
where `phone` is initially accessible and `email` is excluded.

---

## Exit Codes

| Code | Meaning |
| :--- | :------ |
| `0`  | Success. |
| `1`  | The built-in example could not be constructed or evaluated. |
