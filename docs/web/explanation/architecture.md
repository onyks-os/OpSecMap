# Explanation: System Architecture

This page gives a short explanation of the current model. The
[architecture document](https://github.com/onyks-os/OpSecMap/blob/main/docs/architecture.md)
describes the intended system in more detail.

---

## Resources, routes and scenarios

A resource represents something a person may need access to, such as an account,
device or recovery sheet. A route describes one way to reach a target resource.
Every prerequisite in that route must already be accessible. Multiple routes to
the same target are alternatives.

A scenario identifies resources available at the start and resources excluded for
the entire evaluation. An excluded resource cannot be recovered through a route.

## Evaluation

```mermaid
flowchart LR
    P[Password source] --> L[Login route]
    F[Phone] --> L
    L --> E[Email account]
    S[Recovery sheet] --> R[Recovery route]
    R --> E
```

The login route needs both the password source and phone. The recovery route can
establish access through the sheet instead. The evaluator starts from available
resources, applies usable routes, then repeats until no new resources can be
reached. It records an initial or route-based reason for each accessible resource
and missing prerequisites for routes to unresolved targets.
Cycles do not create access on their own; a route into a cycle must first become
usable from available resources.

## Current limits

The evaluator works on an in-memory Rust map. The CLI builds that map from
commands, saves it to a password-encrypted file with `save` (Argon2id and
XChaCha20-Poly1305) and exports it as plain JSON with `export`. Encryption
protects a copy of the file, not a map open on a compromised computer. The
model does not verify that an account provider will honor the documented recovery
procedure. Documentation coverage is a statement you make about your own map;
`Complete` does not prove that a provider offers no other recovery path. A
graphical view is planned work.

The model describes availability after a loss. It does not analyze credential
compromise, data integrity or whether backups contain recoverable data.
