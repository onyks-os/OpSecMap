<!--
Copyright (c) 2026 onyks-os
SPDX-License-Identifier: AGPL-3.0-or-later
-->

# ADR 0003: A Tauri preview first, then an egui desktop app

## Status

Accepted during 0.1.0 development

## Context

OpSecMap has only a command-line interface. A list of commands is correct but
hard to explore when many accounts share one phone or one recovery address, and
the project needs testers, who expect a graphical interface.

[architecture.md](../architecture.md) section 9 leaves the graphical interface
open, with three requirements: it uses the library's model and evaluator instead
of recomputing access, it shows which prerequisites are needed *together* and
which routes are *alternatives*, and it has a textual view usable from the
keyboard and without telling colors apart.

The desktop toolkits considered pull in opposite directions:

* A **web view** (Tauri with Svelte) gives a modern interface quickly, but the
  map and the password then live in a JavaScript engine in another process,
  where they cannot be cleared from memory, and the npm ecosystem adds hundreds
  of packages to review.
* A **Rust toolkit** (egui) keeps every byte in Rust, where the password field
  can be a buffer cleared on drop, but it looks like a developer tool and its
  API still changes with every release (0.37 at the time of writing).

Until the first release the documentation already recommends fictional maps,
because the encryption has not been independently reviewed. Testers will not
enter real maps in that period.

## Decision

We decided to build two desktop frontends over the same library, in order:

1. **A preview desktop app with Tauri 2 and Svelte**, for testers. It is labeled
   a preview in its window and its documentation, and is meant for fictional
   maps. It must follow these rules:
   * all logic stays in the Rust library: evaluation, graph layout, encryption,
     file reading and writing. The web view only displays what Rust sends and
     sends back what the user does;
   * the web view loads no remote content and runs under a strict Content
     Security Policy; text from a map is never inserted as HTML, because a map
     received from someone else is untrusted input;
   * the app grants the web view only the commands it defines: no shell, file
     system or HTTP plugin. File dialogs and file access happen in Rust, so the
     web view never handles a path;
   * developer tools are disabled in release builds, and the stable Tauri 2
     series is used, not a pre-release.
2. **A desktop app with egui**, for real maps, once the preview has users. The
   password field and the decrypted map stay in buffers that are cleared on
   drop, and the zeroization limits in
   [encrypted-format.md](../encrypted-format.md#limits) are reduced there first.

The command-line interface stays, as the reference frontend and for scripts.
The graph layout is computed by the library, so both apps draw the same graph
and it can be tested without a window. Each app is a separate crate, so the
library and the command-line tool do not depend on any GUI toolkit.

### Alternatives Considered

| Alternative | Why it was rejected |
| :---------- | :------------------ |
| egui only | The safest choice, but slower to make attractive to the first testers. |
| Tauri only | Fast, but it leaves no frontend where the password and the map can be cleared from memory. |
| GTK 4 with libadwaita | Native on GNOME and good with Flatpak, but a large C library behind bindings, weaker on Windows and macOS, and harder to learn alongside Rust. |
| iced or Slint | Viable Rust toolkits; egui has the larger user base and the simplest model for drawing a custom graph. |
| Electron | A whole browser per app, and the application logic would move to JavaScript. |
| A web app in the browser (WASM) | Maps would be handled by a browser profile shared with every website the user visits. |

## Consequences

* **Pros**:
  * Testers get a modern interface early, without real data at stake.
  * Both apps reuse the library, including the layout, so neither reimplements
    recovery rules.
  * The egui app can give stronger memory guarantees than the command line.
* **Cons**:
  * Two graphical frontends to maintain until one is retired.
  * The preview adds an npm toolchain, a web view and many crates; the
    dependency, licensing and CI policies must cover them.
  * Testers will get used to the preview's look, which the egui app will not
    match exactly.
* **Neutral / follow-up work**:
  * Reading and writing map files, now in the command-line tool, moves into the
    library so that all frontends share the size limit and the atomic write.
  * Whether the preview is kept after the egui app exists is a later decision.

## References

* [architecture.md](../architecture.md), section 9
* [encrypted-format.md](../encrypted-format.md#limits)
* [Tauri security](https://v2.tauri.app/security/)
* [egui](https://github.com/emilk/egui)
