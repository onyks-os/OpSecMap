# OpSecMap

OpSecMap models the resources needed to access or recover an account. Its Rust
model represents accounts, devices and recovery material as routes and evaluates
what remains accessible in a loss scenario.

[Quickstart Guide](tutorials/quickstart.md){ .md-button .md-button--primary }
[Architecture](explanation/architecture.md){ .md-button }
[GitHub Repository](https://github.com/onyks-os/OpSecMap){ .md-button }

---

!!! warning "Early development"
    The CLI currently runs only a fixed example. It cannot load your own map or
    scenario, and it does not provide persistence, encryption, or a graphical view.
    Keep passwords and recovery codes out of source files and examples.

---

## Implemented in the Rust model

<div class="grid cards" markdown>

- **Resources and routes**

    ---

    A route requires all of its prerequisites. Several routes to one resource
    represent alternatives.

- **Scenario evaluation**

    ---

    Starting from the resources declared available, the evaluator follows usable
    routes until no new resources are found. It records missing prerequisites for
    routes to unresolved targets. Excluded resources remain unavailable.

</div>

---

## Documentation Map

This site follows the [Diátaxis](https://diataxis.fr/) framework:

| Section | Answers | Read it when |
| :------ | :------ | :----------- |
| [Tutorials](tutorials/quickstart.md) | "Teach me the basics." | You are new. |
| [How-To Guides](how-to/index.md) | "How do I solve X?" | You have a specific goal. |
| [Explanation](explanation/architecture.md) | "Why is it built this way?" | You want to understand. |
| [Reference](reference/cli.md) | "What are the exact options?" | You need precise detail. |

---

## Project Status

Manifest version: **v0.1.0**; no GitHub release is published yet. The [roadmap](https://github.com/onyks-os/OpSecMap/blob/main/ROADMAP.md)
tracks what remains before personal maps can be evaluated through the CLI.

Security policy and vulnerability reporting:
[SECURITY.md](https://github.com/onyks-os/OpSecMap/blob/main/SECURITY.md).
