<p align="center">
  <img src="assets/opsecmap-mark.svg" alt="OpSecMap logo: two access routes leading to one resource" width="112" height="112">
</p>

<h1 align="center">OpSecMap</h1>

<p align="center">Map the resources you need to access or recover an account, and see what remains reachable after a loss.</p>

<p align="center">
  <a href="https://github.com/onyks-os/OpSecMap/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/onyks-os/OpSecMap/ci.yml?branch=main&style=for-the-badge&logo=github" alt="CI status"></a>
  <img src="https://img.shields.io/badge/Rust-2021%20edition-orange?style=for-the-badge&logo=rust" alt="Rust 2021 edition">
  <img src="https://img.shields.io/badge/status-early%20development-6b7280?style=for-the-badge" alt="Early development">
  <a href="docs/web/index.md"><img src="https://img.shields.io/badge/docs-source-526CFE?style=for-the-badge&logo=materialformkdocs&logoColor=white" alt="Documentation source"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-AGPL--3.0--or--later-green?style=for-the-badge" alt="AGPL-3.0-or-later license"></a>
</p>

<p align="center">
  <a href="#project-status">Status</a> ·
  <a href="#what-works-today">Current capabilities</a> ·
  <a href="#run-the-example">Run the example</a> ·
  <a href="#how-it-works">How it works</a> ·
  <a href="#development-and-verification">Development</a> ·
  <a href="#contributing">Contribute</a>
</p>

---

If your phone disappears, can you still sign in to your email account? OpSecMap represents each access or recovery procedure as a **route**. A route can require several resources together, while another route offers an alternative. A scenario states what you still have and what is excluded; the Rust evaluator works out which resources the documented routes can reach.

## Project status

OpSecMap is at an early, **source-only** stage. The crate's version is `0.1.0`, and the command-line program runs one fixed demonstration. It cannot load a personal map or scenario. There is no persistence, encryption, graphical view, or downloadable release yet. The [roadmap](ROADMAP.md) tracks those steps.

The result describes the routes in the map; it does not check whether a provider will accept a real recovery procedure. Do not place passwords, recovery codes, or other secrets in source files or examples.

## What works today

| In the Rust library | What it does |
| :--- | :--- |
| Resources and routes | Validates IDs, labels, route prerequisites, and references to resources in the map. A resource can be renamed without changing its ID. |
| Scenarios | Declares resources initially available and resources excluded throughout evaluation. |
| Access evaluation | Follows AND prerequisites and alternative routes through chains, even when the map contains cycles. |
| Explanations | Records whether access was initial or derived through a chosen route; for unresolved targets, lists each blocked route and its missing prerequisites. |

The CLI prints a sample map and the result of one hard-coded scenario. It does not accept a map file, scenario file, or interactive input.

## Run the example

You need Cargo and a Rust toolchain. `Cargo.toml` declares Rust 1.75 as the minimum version.

```sh
git clone https://github.com/onyks-os/OpSecMap.git
cd OpSecMap
cargo run --
```

The example creates four resources and two routes to `email`. It then starts with `phone` accessible and excludes `email`. Its output ends with:

```text
Scenario access:
phone: initially accessible
Email accessible: false
```

The exclusion keeps `email` inaccessible even though routes to it exist. `cargo run -- --version` prints the crate version. The [CLI reference](docs/web/reference/cli.md) describes the other current behavior.

## How it works

```text
password manager + phone ── login ────┐
                                      ├── email account
offline recovery sheet ── recovery ───┘
```

The login route needs **both** the password manager and phone. The recovery route is an alternative. The evaluator starts with the resources marked available, applies every usable route, and repeats until no more resources become accessible. A cycle alone cannot create access.

For each accessible resource, the result distinguishes an initial assumption from access derived through a route. For an inaccessible, non-excluded target, it records which prerequisites are missing from each documented route. That information helps identify what to prepare or check before an incident; it is not proof that recovery is impossible in the real world. The [architecture document](docs/architecture.md) explains the model and its limits.

## Development and verification

```sh
cargo test
cargo fmt --all -- --check
```

The tests cover validation, alternative routes, chains, cycles, exclusions, evidence, and blocked routes. The evaluator runs in memory and the example does not save application data. Release packaging and verification are future work.

## Contributing

Bug reports, questions about the model, and small contributions are welcome; see [CONTRIBUTING.md](CONTRIBUTING.md). For security issues, follow the [security policy](SECURITY.md).

### Use of AI

The maintainer is learning Rust and writes the core Rust engine. AI tools are used to help write tests and documentation, discuss the initial architecture, and make small supporting code changes that the maintainer reviews. The maintainer decides what belongs in the project and checks the resulting work.

## License

OpSecMap is licensed under the [GNU Affero General Public License v3.0 or later](LICENSE) (`AGPL-3.0-or-later`).
