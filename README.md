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

OpSecMap is at an early, **source-only** stage. The crate's version is `0.1.0`, and the command-line program reads a map and scenarios as commands, interactively or from a file, and can save the map to a JSON file and open it again. There is no encryption, graphical view, or downloadable release yet. The [roadmap](ROADMAP.md) tracks those steps.

The result describes the routes in the map; it does not check whether a provider will accept a real recovery procedure. Do not place passwords, recovery codes, or other secrets in source files or examples.

## What works today

| In the Rust library | What it does |
| :--- | :--- |
| Resources and routes | Validates IDs, labels, route prerequisites, and references to resources in the map. A resource can be renamed without changing its ID. |
| Scenarios | Declares resources initially available and resources excluded throughout evaluation. |
| Access evaluation | Follows AND prerequisites and alternative routes through chains, even when the map contains cycles. |
| Explanations | Records whether access was initial or derived through a chosen route; for unresolved targets, lists each blocked route and its missing prerequisites. |
| Documentation coverage | Lets you state whether a resource's routes are documented completely, so an unresolved result can be read as a documented dead end or as undetermined. |
| Safe removal | Removes routes and resources, refusing to remove a resource that a route still uses. |

The CLI turns these into commands read from standard input, and can save the map to a file. Saved files are not encrypted, so use fictional data only.

## Run the example

You need Cargo and a Rust toolchain. `Cargo.toml` declares Rust 1.88 as the minimum version.

```sh
git clone https://github.com/onyks-os/OpSecMap.git
cd OpSecMap
cargo run -- --demo
```

The demo creates four resources and two routes to `email`, and declares the routes to `email` completely documented. It then evaluates two scenarios:

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

In the first scenario the lost phone blocks login, but the recovery sheet still reaches `email`. In the second, no documented route works; because coverage for `email` is `Complete`, that is a dead end within this map, while resources with unknown coverage stay undetermined. `cargo run -- --version` prints the crate version.

## Describe your own map

Without arguments, `opsecmap` reads commands from standard input. The same demo map, written as commands:

```text
resource email Test email
resource password-store Test password manager
resource phone Test phone
resource recovery-sheet Test recovery sheet
route email-login email password-store,phone Email login
route email-recovery email recovery-sheet Email recovery
coverage email complete
evaluate recovery-sheet phone
```

Type them after `cargo run`, or save them in a file and run `cargo run < commands.txt`; [`tests/fixtures/phone-lost.txt`](tests/fixtures/phone-lost.txt) is a complete example. `list` prints the current map as commands you can run again. The [CLI reference](docs/web/reference/cli.md) lists every command, status and exit code.

### Save and open a map

```text
save maps/test.json
open maps/test.json
```

`save` writes the current map to a JSON file, replacing any file already at that path; `open` replaces the current map with the one in the file. A file that is missing, malformed, larger than 1 MiB or of an unsupported version is refused and the map in use stays as it was. The format is specified in [docs/map-format.md](docs/map-format.md).

> **The file is plain text and is not encrypted.** Anyone with a copy can read every resource and route in it. Use fictional data only until encryption is available.

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
cargo clippy --all-targets --all-features -- -D warnings
make fuzz   # property tests, including arbitrary CLI input
```

The tests cover validation, alternative routes, chains, cycles, exclusions, evidence, blocked routes, and the CLI run end to end on fixture files. CI runs them on stable, beta and the minimum Rust version, together with a dependency audit, CodeQL and a reproducible-build check. Release packaging and verification are prepared but nothing has been published yet.

## Contributing

Bug reports, questions about the model, and small contributions are welcome; see [CONTRIBUTING.md](CONTRIBUTING.md). For security issues, follow the [security policy](SECURITY.md).

### Use of AI

The maintainer is learning Rust and writes the core Rust engine. AI tools are used to help write tests and documentation, discuss the initial architecture, and make small supporting code changes that the maintainer reviews. The maintainer decides what belongs in the project and checks the resulting work.

## License

OpSecMap is licensed under the [GNU Affero General Public License v3.0 or later](LICENSE) (`AGPL-3.0-or-later`).
