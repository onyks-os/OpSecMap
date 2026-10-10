# Atlas Local — Technical Design Document

**Type:** Single Page Application (SPA), statically deployable, no backend.
**License:** AGPLv3.
**Core Purpose:** Personal Operational Security (OpSec) visualization and dependency modeling — mapping how a person's digital life (accounts, devices, backups, recovery paths) depends on itself, so single points of failure become visible before an incident, not during one.
**Philosophy:** Local-first, zero-trust, privacy-by-design. No servers, no cloud databases, no analytics, no telemetry. All state lives in a file on the user's own device, optionally encrypted.

---

## 1. Target Audience & User Archetypes

The application serves three profiles, mapped to two UI modes:

1. **The Organizer (Basic User)** → **Analyst Mode** (Beginner). Priorities: mental clarity, low friction, optimistic scoring that doesn't punish an incomplete graph.
2. **The Privacy-Conscious User (Intermediate)** → **Operator Mode** (Advanced), using the same interface as archetype 3 but ignoring hostile-environment-specific tooling. Priorities: digital sovereignty, backup management, resilience tracking.
3. **The High-Risk User (Journalist / Activist / Advanced)** → **Operator Mode + Active Defense**. Priorities: physical OpSec, non-repudiation, formal/strict scoring, survival planning.

Mode is an engine-level setting, not just a UI theme — it changes which edges are treated as mandatory (see §5.2).

---

## 2. Architecture Overview

All logic — validation, risk propagation, scoring, simulation, cryptography, schema migration, CSV ingestion — lives in a single framework-agnostic Rust core with no I/O side effects. The frontend is a pure view layer with no business logic of its own.

```
┌───────────────────────────────┐        ┌────────────────────────────────┐
│           atlas-core            │        │            View Layer            │
│    (Rust, pure, no I/O)          │◄──────►│   (Svelte 5 + Svelte Flow)        │
│                                  │  calls │                                    │
│  - Unified Rule Engine            │        │  - Graph canvas, node/edge UI      │
│  - AtlasState + schema migration  │        │  - Modals, command palette         │
│  - Cryptography (Argon2id, AES)   │        │  - Accessibility, theming, layout  │
│  - CSV ingestion mapping          │        │                                    │
└───────────────┬──────────────────┘        └────────────────────────────────┘
                │
     compiles unchanged to two targets:
                │
       ┌────────┴─────────┐
       ▼                   ▼
   wasm32 target       native target
  (browser, in scope)   (optional desktop packaging, out of scope for V1)
```

This is a **Ports-and-Adapters (Hexagonal) architecture**: the core exposes its logic behind small trait boundaries (a randomness source, a clock) with no direct filesystem or browser-API calls baked in. A thin WASM adapter exposes it to the browser today; an equally thin native adapter could expose the same crate to a desktop shell later, with zero changes to the logic itself. The view layer never touches cryptography, validation, or scoring directly — it only calls exported functions and renders results.

---

## 3. Technology Stack

**Core logic (`atlas-core`):**
* Rust, `#![forbid(unsafe_code)]`.
* **Rule engine:** [`crepe`](https://crates.io/crates/crepe) — Datalog-style logic programming as a Rust procedural macro, with semi-naive evaluation and stratified negation. Implements the Unified Rule Engine (§5).
* **Cryptography:** `argon2` (Argon2id) for key derivation, `aes-gcm` for authenticated encryption.
* **Serialization:** `serde` / `serde_json`.
* **Type export:** [`specta`](https://specta.dev/) generates TypeScript types directly from Rust structs and enums — every type is declared once, in Rust, never duplicated by hand on the frontend.
* **Property-based testing:** `proptest`, for the rule engine's determinism, termination, and partition properties.

**WASM adapter:**
* `wasm-bindgen` / `wasm-pack` compile `atlas-core` to `wasm32-unknown-unknown` with generated JS bindings.
* `getrandom`, with the **`wasm_js`** feature explicitly enabled, so randomness is sourced from `Crypto.getRandomValues` via `wasm-bindgen`. This is not optional — without it, any code path touching randomness (including key generation inside `argon2`/`aes-gcm`) fails to compile or panics at runtime on this target. It is the single most common mistake when compiling Rust cryptography to WASM.
* `wasm-bindgen-test` for boundary-layer tests.

**View layer:**
* **Framework:** Svelte 5, using runes (`$state`, `$derived`) for reactivity — no external state-management library is needed.
* **Graph visualization:** Svelte Flow (`@xyflow/svelte`) — a mature, actively maintained node/edge canvas library with drag handles, edge routing, minimap, and connection-validation UX out of the box.
* **Auto-layout:** `elkjs` — a single choice, covering both simple and complex hierarchical layouts, so no second layout library is carried for marginal gain.
* **Undo/redo:** a small hand-rolled patch stack (a bounded ring buffer of `AtlasState` diffs) — Svelte's reactivity model makes an external undo/redo middleware unnecessary.
* **UI components:** `shadcn-svelte`, built on Bits UI — accessible, unstyled primitives with Tailwind styling on top.
* **Command palette:** `cmdk-sv`. **Toasts:** `svelte-sonner`.
* **Input validation:** Zod, used only to validate raw external input (CSV rows, imported files) before it reaches the WASM boundary — not for the node/edge schema itself, which is generated from Rust.
* **Testing:** `vitest` for components, `Playwright` for end-to-end and cross-browser testing (Chromium, Firefox, WebKit).

**Accessibility:** WCAG 2.1 AA is a hard requirement, addressed in full in §8. This is a UI-layer concern and does not depend on the choice of core language.

---

## 4. Data Architecture

### 4.1 Storage Format

Two supported formats:
1. `.json` — plaintext, discouraged for anything sensitive but supported for portability/debugging.
2. `.atlas` — encrypted, the recommended default.

```rust
#[derive(Serialize, Deserialize, Type)]
pub struct AtlasEncryptedFile {
    pub meta: CryptoMeta,
    pub data: String, // base64 ciphertext of AtlasState
}

#[derive(Serialize, Deserialize, Type)]
pub struct CryptoMeta {
    pub algo: String,       // "AES-256-GCM"
    pub kdf: String,        // "argon2id"
    pub salt: String,       // base64
    pub nonce: String,      // base64, 96-bit for GCM
    pub kdf_params: Argon2Params,
    pub hint: Option<String>,
}

#[derive(Serialize, Deserialize, Type)]
pub struct Argon2Params {
    pub memory_kib: u32,  // OWASP baseline: 19456 (19 MiB) minimum; tuned higher, since this runs entirely on the user's own device
    pub iterations: u32,  // 2 minimum
    pub parallelism: u32, // 1 minimum
}

#[derive(Serialize, Deserialize, Type)]
pub struct AtlasState {
    pub meta: StateMeta,
    pub nodes: Vec<AtlasNode>,
    pub edges: Vec<EdgeData>,
    pub preferences: Option<Preferences>,
}

#[derive(Serialize, Deserialize, Type)]
pub struct StateMeta {
    pub schema_version: u32,
    pub app_version: String,
    pub updated_at: String, // ISO 8601
}
```

### 4.2 Node Model — Faceted, Not Single-Typed

A node's identity is not a single type tag; it is a set of independent facets, because real assets are genuinely polymorphic — a laptop is simultaneously `COMPUTE` and, if it holds local backups, also `DATA_STORE`. Forcing a single-type enum would either lose information the rule engine needs or force artificial categorization choices.

```rust
#[derive(Serialize, Deserialize, Type, Clone, Copy, PartialEq, Eq)]
pub enum NodeNature { Physical, Digital }

#[derive(Serialize, Deserialize, Type, Clone, Copy, PartialEq, Eq)]
pub enum NodeCapability {
    DataStore,      // HDD, cloud storage, NAS
    Compute,        // PC, server, phone
    Authentication, // hardware key, authenticator app
    CredentialMgr,  // password manager
    Identity,       // root email, primary identity anchor
    Gateway,        // router, VPN
}

#[derive(Serialize, Deserialize, Type, Clone, Copy, PartialEq, Eq)]
pub enum NodeTag {
    BackupTarget,
    EntryPoint,
    Critical,
    Offsite,
    RecoveryPath,
}

#[derive(Serialize, Deserialize, Type)]
pub struct AtlasNode {
    pub id: String,
    pub label: String,
    pub nature: NodeNature,
    pub capabilities: Vec<NodeCapability>,
    pub tags: Vec<NodeTag>,
    pub notes: Option<String>,
    pub is_sensitive: bool,
    pub last_backup: Option<String>,
    pub frequency_days: Option<u32>,
}
```

This is a faceted / role-based classification, structurally the same pattern as an Entity-Component-System: a node is an id, capabilities and tags are components attached in any combination, and the rule engine's rules are systems querying combinations of components. Capabilities and tags are stored as relations (`HasCapability(node, capability)`, `HasTag(node, tag)`), not as a single field, precisely so the rule engine can query arbitrary combinations.

**On combinatorial sprawl:** with independent facets the space of possible combinations is $2^{|Capability|} \times 2^{|Tag|}$, and not all combinations are meaningful (a node with zero capabilities and zero tags is a modeling smell). This is addressed as a separate, additive constraint layer — shape constraints (minimum/maximum cardinality, forbidden co-occurrences) discovered empirically through use and added on top, rather than rigidifying the base schema up front to rule out cases that haven't been observed yet.

### 4.3 Edge Model

```rust
#[derive(Serialize, Deserialize, Type, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeType {
    DependsOn,             // generic structural dependency
    ProvidesCredentialsTo, // password manager -> service
    Provides2faTo,         // auth app -> service
    ProvidesStorageFor,    // hard drive -> data/laptop
    OpensAccessTo,         // VPN/ISP -> internet
}

#[derive(Serialize, Deserialize, Type)]
pub struct EdgeData {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: EdgeType,
}
```

### 4.4 Schema Migration

A `migrate(state)` pipeline runs on file load: if `state.meta.schema_version` is below the current version, sequential pure transformers are applied before the state is accepted anywhere else in the app. Implemented as ordinary Rust, tested directly with `cargo test` against fixture states at each historical schema version.

---

## 5. The Unified Rule Engine

### 5.1 Why a single engine

Validation ("can this edge exist?"), resilience scoring ("how safe is this graph overall?"), disaster simulation ("what happens if this node is compromised?"), and explainability ("why is this node at risk?") are four different queries against the same fact base and the same rules — not four independent subsystems. Treating them as one rule set queried four ways, instead of four hand-written traversals, removes an entire class of bug: silent divergence between "engines" that are supposed to agree but were never actually the same code.

### 5.2 Formal Model

**Node status domain.** A node's classification is not simply a 3-value lattice — it is a wrapper around one, because "no evidence was found to classify this node" is a fundamentally different kind of fact from "this node was actively verified safe." Collapsing the two would mean an unclassified node silently reads as safe, which is the wrong default for a security tool.

```rust
#[derive(Serialize, Deserialize, Type, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel { Safe, AtRisk, Compromised }

#[derive(Serialize, Deserialize, Type, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    Classified(RiskLevel),
    Unclassified,
}
```

`NodeStatus` deliberately does not implement a total ordering against `RiskLevel` values — `Unclassified` is not "worse than Compromised" or "better than Safe," it is orthogonal: an epistemic gap, not a risk position. Any code that needs to sort or compare statuses must handle `Unclassified` explicitly rather than by accident.

**Edge requirement relation.** A finite relation, not a grammar or a scoring heuristic:

$$
R(\text{edgeType}, \text{mode}) \in \{\text{OPTIONAL}, \text{MITIGATING}, \text{REQUIRED}\}
$$

Mode-dependent: Advanced mode pushes most edges to `REQUIRED`; Beginner/Intermediate mode is more permissive (OR-inclusive on mitigators — a node is considered backed up if *at least one* connected backup provider is valid). Implemented as an exhaustive Rust `match`, so the compiler — not a test suite — rejects the build if any `(EdgeType, Mode)` pair is left unhandled.

**Mitigator tags.** `BackupTarget` and `RecoveryPath` reduce a node's risk level toward `Safe` but never *cause* `Compromised` or `AtRisk` on their own — a valid backup is evidence of safety, its absence is not evidence of compromise.

**Rules**, expressed as Datalog and implemented via `crepe`:

```rust
crepe::crepe! {
    @input
    struct Edge(NodeId, NodeId, EdgeType);      // (source, target, type)
    @input
    struct HasTag(NodeId, NodeTag);
    @input
    struct MarkedCompromised(NodeId);           // set by the simulation query, empty otherwise
    @input
    struct Requirement(EdgeType, Mode, ReqLevel); // the R(edgeType, mode) relation
    @input
    struct ActiveMode(Mode);                     // singleton fact: current evaluation mode

    struct EntryPoint(NodeId);
    EntryPoint(n) <- HasTag(n, NodeTag::EntryPoint);

    struct Mitigator(NodeId);
    Mitigator(n) <- HasTag(n, NodeTag::BackupTarget);
    Mitigator(n) <- HasTag(n, NodeTag::RecoveryPath);

    @output
    struct Compromised(NodeId);
    Compromised(n) <- MarkedCompromised(n);
    Compromised(n) <- Edge(u, n, t), Requirement(t, mode, ReqLevel::Required),
                      Compromised(u), ActiveMode(mode);

    struct SafeViaMitigation(NodeId);
    SafeViaMitigation(n) <- Edge(m, n, _), Mitigator(m), Safe(m);

    @output
    struct AtRisk(NodeId);
    AtRisk(n) <- Edge(u, n, t), Requirement(t, mode, ReqLevel::Mitigating),
                 Compromised(u), ActiveMode(mode), !SafeViaMitigation(n);
    AtRisk(n) <- Edge(u, n, t), Requirement(t, mode, r), r != ReqLevel::Optional,
                 AtRisk(u), ActiveMode(mode);

    @output
    struct Safe(NodeId);
    Safe(n) <- EntryPoint(n), !Compromised(n), !AtRisk(n);
    Safe(n) <- Edge(u, n, _), Safe(u), !Compromised(n), !AtRisk(n);
}
```

*(This is the conceptual skeleton; the exact stratification of negation across `Compromised`/`AtRisk`/`Safe`/`SafeViaMitigation` needs finishing during implementation. The important property is that it is executable, testable, and terminates deterministically on any finite graph via `crepe`'s semi-naive evaluation.)*

**Resolving `Unclassified`.** The three output relations above are not guaranteed to cover every node — a node disconnected from any `EntryPoint`, for instance, derives none of them. Status is resolved in plain Rust after the fixpoint runs, as a total function with an explicit fallback and a fail-safe ordering (worst known state wins if outputs ever overlap, which stratified negation should prevent but is worth defending against defensively):

```rust
fn resolve_status(
    node: NodeId,
    compromised: &HashSet<NodeId>,
    at_risk: &HashSet<NodeId>,
    safe: &HashSet<NodeId>,
) -> NodeStatus {
    if compromised.contains(&node) { NodeStatus::Classified(RiskLevel::Compromised) }
    else if at_risk.contains(&node) { NodeStatus::Classified(RiskLevel::AtRisk) }
    else if safe.contains(&node)    { NodeStatus::Classified(RiskLevel::Safe) }
    else                             { NodeStatus::Unclassified }
}
```

`Unclassified` nodes must be visually distinct in the UI — not hidden, not rendered as green — per the accessibility requirements in §8 (a dedicated icon, not color alone).

**Structural invariants**, computed as ordinary graph algorithms rather than Datalog facts, since they are properties of the graph's shape, not of the fact base:
* **Cycle detection:** three-color DFS, run before any new edge is accepted. A cycle would break the topological ordering the propagation above implicitly assumes.
* **Reachability from entry points:** standard BFS/DFS, used both for the resilience score aggregation and for surfacing orphaned nodes in the UI.

### 5.3 Four Queries, One Rule Set

* **Validation** — before committing a new `Edge` fact: run cycle detection, then check `Requirement` compatibility for the given node-type/edge-type pair. Synchronous, cheap, blocks invalid graph states at the point of interaction.
* **Resilience scoring** — an aggregate query over the classified outputs:
$$
R_s = 100 \times \frac{\sum_{v \in V} w_v \cdot \mathbf{1}[\text{status}(v) = \text{Safe}]}{\sum_{v \in V} w_v}, \qquad w_v = 1 + \text{betweennessCentrality}(v)
$$
`Unclassified` nodes count in the denominator but never in the numerator — they cannot silently inflate the score.
* **Disaster simulation** — assert a temporary `MarkedCompromised(x)` fact, re-run the fixpoint, diff the resulting classification against the baseline. No separate simulation subsystem is needed.
* **Explainability** — `crepe`'s derivation is itself a proof. The reason trace for a given output fact is recovered from which rule instances fired to produce it, not maintained as a separate hand-written data structure that could drift from the rules it's supposed to explain.

### 5.4 Limits of the Formal Model — and How the Project Compensates

No formal verification of this rule engine — however thorough — can guarantee that the *schema* (the fixed vocabulary of `NodeCapability`, `NodeTag`, `EdgeType`) captures every real-world attack pattern. Verification proves internal consistency (the engine behaves according to its own rules); it cannot prove external fidelity (the rules describe reality completely). This is not a gap specific to this project — it is a structural limit of any formal model of risk, and treating it as solvable with "more rigor" would be a category error.

The project's answer is not to chase unreachable completeness, but to make the model's incomplete coverage explicit and continuously falsifiable:

1. **`Unclassified` as a first-class, visible state** (§5.2) — absence of evidence is never allowed to silently read as safety.
2. **An Incident Replay Corpus** (§10.2) — a growing set of test fixtures built from real, documented security incidents, used to continuously probe the schema for missing vocabulary rather than to "prove" coverage. Every schema gap the corpus surfaces is logged as an actionable backlog item, not treated as a hidden flaw to be quietly patched over.

---

## 6. Security Engine

* **Key derivation: Argon2id.** OWASP's Password Storage Cheat Sheet lists Argon2id as the current first-choice recommendation; PBKDF2 is reserved for cases requiring FIPS certification or broad legacy platform support, neither of which applies here. Minimum parameters follow OWASP's baseline (19 MiB memory, 2 iterations, parallelism 1), tuned upward since this runs entirely on the user's own device with no server-side cost concern.
* **Encryption:** AES-256-GCM.
* **Randomness on the WASM target:** `getrandom` with the `wasm_js` feature enabled — flagged explicitly in the tech stack (§3) because it is the most common way to accidentally ship a WASM build whose cryptography silently fails.
* **Zero-knowledge:** the password is never persisted; it exists only in memory for the duration of a single encrypt/decrypt call.

---

## 7. Visual Experience & Accessibility

WCAG 2.1 AA is a hard requirement, not an aspiration:

* **Screen readers:** every node carries an `aria-label` describing its label and status (e.g., "Node: Laptop, Status: Compromised"). The canvas supports full keyboard navigation (arrow keys to pan, Tab to select).
* **Contrast & color blindness:** status is never conveyed by color alone — every status (including `Unclassified`) pairs a color with a distinct icon or texture. A high-contrast theme forces black/white borders for maximum legibility.
* **Focus management:** logical tab order, especially inside modals and side panels.
* **Edge semantics:**

| Edge Type | Semantics | Visual Style | Animation |
|---|---|---|---|
| DependsOn | Structural dependency | Solid grey line | None |
| ProvidesStorageFor | Backup / sync | Dashed blue line | Flowing particles, source → target |
| Provides2faTo / ProvidesCredentialsTo | Credential / 2FA | Solid green line | Fast dash animation |
| OpensAccessTo | Gateway containment | Dotted grey line | None |

* **Interaction:** hovering a node dims the rest of the graph and highlights only connected paths (desktop). On touch, the equivalent is a long-press or a dedicated inspect-mode toggle, with touch targets exceeding 44px.

---

## 8. V1 Functional Scope

**In scope:**
* Onboarding: starter-pack templates and a first-load interactive tutorial.
* Full encrypted save/open flow (Argon2id + AES-GCM), including schema migration.
* CSV ingestion (generic, Bitwarden, Proton Pass exports), parsed client-side and mapped to `AtlasNode`/`EdgeData` inside the core.
* Command palette, keyboard shortcuts, undo/redo.
* Beginner-mode resilience scoring (permissive, optimistic).
* Advanced-mode strict scoring — included, since the rule engine produces it almost for free once the base rules exist; its UI polish is secondary, not a release blocker.

**Explicitly out of scope for V1** (see §9): disaster-simulation UI, third-party breach intelligence, air-gapped sync, steganographic export.

---

## 9. Deferred Features

None of the following block a first release:
* Disaster-simulation UI with probability scoring (the simulation *query* exists in the rule engine from day one — only the interactive UI around it is deferred).
* Breach-intelligence integration (checking credentials against known-compromised lists, via k-anonymity so no plaintext data leaves the device).
* Air-gapped state transfer (e.g., via animated QR codes).
* Steganographic export.
* Printable emergency/recovery sheet (PDF export).
* Graph export to a portable diagram format.

---

## 10. Testing Strategy

### 10.1 Standard Test Layers

* **Core (Rust):** `cargo test` for cryptographic round-trips (encrypt → decrypt returns the exact original state), schema migration transformers, and CSV mapping.
* **Rule engine properties:** `proptest`, generating random finite graphs (bounded size, no dangling references) and asserting:
  * Termination on every generated graph.
  * Every node resolves to exactly one `NodeStatus` (never two, never an implicit default).
  * Monotonicity: adding a `MarkedCompromised` fact never improves any node's resolved status.
* **WASM boundary:** `wasm-bindgen-test`, specifically targeting the adapter layer where randomness sourcing and serialization mismatches would surface.
* **View layer:** `vitest` for component logic; `Playwright` for end-to-end flows (create → connect → save encrypted → reload → open decrypted → verify identical graph), run across Chromium, Firefox, and WebKit, plus `axe-core` accessibility scans.

### 10.2 Incident Replay Corpus

A growing set of test fixtures, each built from a real, documented security incident, encoded as a small graph with an expected classification outcome. This corpus exists to continuously *falsify* the schema's coverage, not to prove completeness — every incident that the current vocabulary cannot represent is a concrete, actionable signal to extend `NodeCapability`/`EdgeType`, logged rather than silently patched around.

```md
## Incident replay fixture template
- Scenario: <name, with source>
- Graph: <nodes and edges, using current schema>
- Expected classification: <per-node expected NodeStatus>
- Schema gaps found: <anything the current vocabulary cannot represent, if any>
- Action: <extend schema / accepted as out-of-scope, with reasoning>
```

Seed set to build against before V1:

1. **SIM-swap cascade** — attacker social-engineers the phone carrier into transferring a SIM, intercepts SMS-based 2FA, resets the email password, then reaches banking/wallet accounts. Graph: `Phone --Provides2faTo--> Email --ProvidesCredentialsTo--> Bank`. Known gap: no `NodeCapability` currently represents "third-party account with a social-engineerable support channel" (the phone carrier itself); this needs a new capability before the scenario is fully representable.
2. **Linked-account cascade** (the 2012 Mat Honan/Apple ID incident pattern) — a single support-channel social-engineering attack cascades across linked cloud identity, email, and remote-wipe-capable devices.
3. **Password-manager master-password compromise** — a single credential vault as an unmitigated single point of failure for every downstream `ProvidesCredentialsTo` edge.
4. **Lost or stolen device without remote wipe** — physical compromise of a `Physical`/`Compute` node with no `OFFSITE` mitigation and no remote-wipe capability represented.
5. **Recovery-email takeover** — compromise propagating backward along a `RecoveryPath` edge, the mitigator relation being exploited as an attack path rather than a defense.

Each fixture becomes a permanent regression test once its expected classification is agreed, and the corpus grows every time a new real-world pattern surfaces.

---

## 11. Licensing & Community Audit

* **License:** AGPLv3 — modifications must remain open-source, preventing closed commercial forks without contribution back.
* **CI/CD:** `cargo test` + `cargo clippy` for the core, `vitest`/`Playwright` for the view layer, dependency vulnerability scanning on every build.
* **Audit invitation:** the codebase is explicitly open to security review; cryptographic implementation details are documented, not obscured.

---

## 12. Optional Native Packaging

Because `atlas-core` has no I/O side effects, it compiles unchanged to a native target and can be embedded in a native desktop shell whenever a packaged build is wanted — gaining full OS filesystem access, native CSPRNG (no WASM randomness workaround needed), and eventually native USB/HID access for hardware security keys, none of which are reachable from a browser sandbox. Type-safe bindings from Rust to the frontend would be generated the same way in that scenario (via `specta`), so the type-safety guarantee holds across both deployment targets. This is not a V1 requirement — it is a packaging option the architecture leaves open at no cost, not a milestone to plan toward.
