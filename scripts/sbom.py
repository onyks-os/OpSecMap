#!/usr/bin/env python3
# Copyright (c) 2026 onyks
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Write a CycloneDX SBOM next to each release artifact, read from the artifact itself.

    scripts/sbom.py [--version V] [--cargo] dist/<artifact>...

For every artifact given, writes <artifact>.cdx.json declaring:

- the artifact itself as the described component: name, version, SHA-256;
- what it *requires*, as the artifact declares it — Requires-Dist for a wheel or
  sdist, dependencies for an npm tarball or a .crate, Depends for a .deb,
  Requires for an .rpm, depend for an Arch package — with the version constraint
  and who is expected to provide it (pip, npm, cargo, or the distribution);
- with --cargo, what it *contains*: every crate compiled into a binary artifact
  (the bare binary, .deb, .rpm, Arch package, Flatpak), resolved from Cargo.lock
  for the host platform, normal dependencies only, with its checksum and licence.

One SBOM describing the checkout, or the CI runner's environment, describes no
artifact at all: a distribution package depends on the distribution's libraries,
not on the versions pip happened to install on the build machine. Hence one SBOM
per artifact, from the artifact.

Deterministic: the serial number derives from the artifact's digest and the
timestamp from SOURCE_DATE_EPOCH, so the same artifact always gives the same SBOM
and a reproducible build stays reproducible. Standard library only, plus the
distribution's own tools for the formats that need them (dpkg-deb, rpm, tar with
zstd, cargo).
"""

from __future__ import annotations

import argparse
import email.parser
import hashlib
import json
import os
import re
import subprocess
import sys
import tarfile
import time
import uuid
import zipfile
from pathlib import Path
from typing import Any

NAME = "opsecmap"
LICENSE = "AGPL-3.0-or-later"
SPEC_VERSION = "1.6"
#: Namespace of this project's custom properties (CycloneDX reserves `cdx:`).
PROP = f"{NAME}:"
#: uuid5 namespace for serial numbers: stable, so equal artifacts get equal SBOMs.
SERIAL_NAMESPACE = uuid.UUID("7d0f5d3a-3b0e-4f5e-9a51-3c2e8d6b1f40")

#: Artifacts that carry compiled code, so --cargo lists the crates inside them.
BINARY_KINDS = {"file", "deb", "rpm", "arch", "flatpak"}

#: Declared requirements that are not packages: the rpm runtime, file paths.
NOT_A_COMPONENT = re.compile(r"^(rpmlib\(|/|config\()")

Component = dict[str, Any]


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def run(*argv: str) -> str:
    return subprocess.run(argv, check=True, capture_output=True, text=True).stdout


def requirement(name: str, constraint: str | None, provided_by: str) -> Component:
    component: Component = {
        "type": "library",
        "bom-ref": f"requires:{provided_by}:{name}",
        "name": name,
        "scope": "required",
        "properties": [{"name": PROP + "provided-by", "value": provided_by}],
    }
    if constraint:
        component["properties"].append({"name": PROP + "version-constraint", "value": constraint})
    return component


def unique(components: list[Component]) -> list[Component]:
    """One component per bom-ref: a name required twice is still one component."""
    seen: dict[str, Component] = {}
    for c in components:
        seen.setdefault(c["bom-ref"], c)
    return list(seen.values())


# ---------------------------------------------------------------------------
# Readers: artifact -> (kind, version, purl, requirements)
# ---------------------------------------------------------------------------


def pypi_name(name: str) -> str:
    """The normalised project name purl uses for PyPI (PEP 503)."""
    return re.sub(r"[-_.]+", "-", name).lower()


def python_requirements(metadata: str) -> list[Component]:
    out = []
    for req in email.parser.Parser().parsestr(metadata).get_all("Requires-Dist") or []:
        if "extra ==" in req:
            continue  # an optional extra is not what the artifact requires
        m = re.match(r"\s*([A-Za-z0-9._-]+)\s*(.*)", req.split(";")[0])
        if m:
            out.append(requirement(pypi_name(m.group(1)), m.group(2).strip() or None, "pip"))
    return out


def read_wheel(path: Path) -> tuple[str, str | None, str | None, list[Component]]:
    with zipfile.ZipFile(path) as wheel:
        member = next(n for n in wheel.namelist() if n.endswith(".dist-info/METADATA"))
        text = wheel.read(member).decode("utf-8")
    meta = email.parser.Parser().parsestr(text)
    return "wheel", meta["Version"], f"pkg:pypi/{pypi_name(meta['Name'])}@{meta['Version']}", python_requirements(text)


def read_tar_member(path: Path, predicate) -> bytes | None:
    with tarfile.open(path) as archive:
        for member in archive.getmembers():
            if predicate(member.name):
                handle = archive.extractfile(member)
                return handle.read() if handle else None
    return None


def read_tar_gz(path: Path) -> tuple[str, str | None, str | None, list[Component]]:
    """An sdist when it carries PKG-INFO at its top level; otherwise an opaque archive."""
    raw = read_tar_member(path, lambda n: n.count("/") == 1 and n.endswith("/PKG-INFO"))
    if raw is None:
        return "archive", None, None, []
    text = raw.decode("utf-8")
    meta = email.parser.Parser().parsestr(text)
    purl = f"pkg:pypi/{pypi_name(meta['Name'])}@{meta['Version']}?type=sdist"
    return "sdist", meta["Version"], purl, python_requirements(text)


def npm_purl(name: str, version: str) -> str:
    if name.startswith("@"):
        scope, _, bare = name.partition("/")
        return f"pkg:npm/%40{scope[1:]}/{bare}@{version}"
    return f"pkg:npm/{name}@{version}"


def read_npm(path: Path) -> tuple[str, str | None, str | None, list[Component]]:
    raw = read_tar_member(path, lambda n: n == "package/package.json")
    if raw is None:
        return "archive", None, None, []
    manifest = json.loads(raw)
    requires = [requirement(n, c, "npm") for n, c in sorted(manifest.get("dependencies", {}).items())]
    for n, c in sorted(manifest.get("optionalDependencies", {}).items()):
        component = requirement(n, c, "npm")
        component["scope"] = "optional"
        requires.append(component)
    version = manifest.get("version")
    return "npm", version, npm_purl(manifest["name"], version), requires


def read_crate(path: Path) -> tuple[str, str | None, str | None, list[Component]]:
    try:
        import tomllib
    except ImportError:  # Python < 3.11
        raise SystemExit("reading a .crate needs Python 3.11 or later (tomllib)") from None
    raw = read_tar_member(path, lambda n: n.count("/") == 1 and n.endswith("/Cargo.toml"))
    if raw is None:
        raise SystemExit(f"{path.name}: no Cargo.toml at the top of the .crate")
    manifest = tomllib.loads(raw.decode("utf-8"))
    package = manifest["package"]
    requires = []
    # Normal dependencies only: dev- and build-dependencies never reach a user.
    for key, spec in sorted(manifest.get("dependencies", {}).items()):
        if isinstance(spec, str):
            requires.append(requirement(key, spec, "cargo"))
        else:
            component = requirement(spec.get("package", key), spec.get("version"), "cargo")
            if spec.get("optional"):
                component["scope"] = "optional"
            requires.append(component)
    return "crate", package["version"], f"pkg:cargo/{package['name']}@{package['version']}", requires


def read_deb(path: Path) -> tuple[str, str | None, str | None, list[Component]]:
    def field(name: str) -> str:
        return run("dpkg-deb", "-f", str(path), name).strip()

    requires = []
    for item in field("Depends").split(","):
        # "a | b" is one requirement with alternatives; each is recorded, since a
        # scanner matching either name should find it.
        for alternative in item.split("|"):
            m = re.fullmatch(r"\s*([a-z0-9.+-]+)(?::[a-z0-9]+)?\s*(?:\((.*)\))?\s*", alternative)
            if m:
                requires.append(requirement(m.group(1), m.group(2), "distribution"))
    return "deb", field("Version"), None, requires


def read_rpm(path: Path) -> tuple[str, str | None, str | None, list[Component]]:
    version = run("rpm", "-qp", "--queryformat", "%{VERSION}-%{RELEASE}", str(path)).strip()
    requires = []
    for line in run("rpm", "-qp", "--requires", str(path)).splitlines():
        name, _, constraint = line.strip().partition(" ")
        if name and not NOT_A_COMPONENT.match(name):
            requires.append(requirement(name, constraint or None, "distribution"))
    return "rpm", version, None, requires


def read_arch(path: Path) -> tuple[str, str | None, str | None, list[Component]]:
    info = subprocess.run(
        ["tar", "--zstd", "-xOf", str(path), ".PKGINFO"], check=True, capture_output=True
    ).stdout.decode("utf-8")
    fields: dict[str, list[str]] = {}
    for line in info.splitlines():
        key, sep, value = line.partition(" = ")
        if sep:
            fields.setdefault(key.strip(), []).append(value.strip())
    requires = []
    for dep in fields.get("depend", []):
        m = re.fullmatch(r"([A-Za-z0-9@._+-]+?)((?:[<>=]=?|=).+)?", dep)
        if m:
            requires.append(requirement(m.group(1), m.group(2), "distribution"))
    return "arch", fields.get("pkgver", [None])[0], None, requires


READERS = [
    (".whl", read_wheel),
    (".tar.gz", read_tar_gz),
    (".tgz", read_npm),
    (".crate", read_crate),
    (".deb", read_deb),
    (".rpm", read_rpm),
    (".pkg.tar.zst", read_arch),
    (".flatpak", lambda p: ("flatpak", None, None, [])),
]


def describe(path: Path) -> tuple[str, str | None, str | None, list[Component]]:
    for suffix, reader in READERS:
        if path.name.endswith(suffix):
            return reader(path)
    return "file", None, None, []


# ---------------------------------------------------------------------------
# --cargo: the crates compiled into a binary
# ---------------------------------------------------------------------------


def cargo_components() -> list[Component]:
    """Normal-dependency closure of the root package, for the host platform."""
    host = next(
        line.split(": ", 1)[1] for line in run("rustc", "-vV").splitlines() if line.startswith("host: ")
    )
    meta = json.loads(run("cargo", "metadata", "--format-version", "1", "--locked", "--filter-platform", host))
    packages = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}
    root = meta["resolve"]["root"]
    if root is None:
        raise SystemExit("--cargo needs a single-package Cargo.toml (no workspace root package found)")

    checksums: dict[tuple[str, str], str] = {}
    lock = Path("Cargo.lock")
    if lock.is_file():
        import tomllib

        for entry in tomllib.loads(lock.read_text(encoding="utf-8")).get("package", []):
            if "checksum" in entry:
                checksums[(entry["name"], entry["version"])] = entry["checksum"]

    seen: set[str] = set()
    stack = [root]
    while stack:
        for dep in nodes[stack.pop()]["deps"]:
            if any(kind["kind"] is None for kind in dep["dep_kinds"]) and dep["pkg"] not in seen:
                seen.add(dep["pkg"])
                stack.append(dep["pkg"])

    out = []
    for pkg_id in sorted(seen, key=lambda i: (packages[i]["name"], packages[i]["version"])):
        p = packages[pkg_id]
        component: Component = {
            "type": "library",
            "bom-ref": f"contains:cargo:{p['name']}@{p['version']}",
            "name": p["name"],
            "version": p["version"],
            "scope": "required",
            "properties": [{"name": PROP + "provided-by", "value": "compiled-in"}],
        }
        source = p.get("source") or ""
        if source.startswith("registry+https://github.com/rust-lang/crates.io-index") or source.startswith(
            "sparse+https://index.crates.io/"
        ):
            component["purl"] = f"pkg:cargo/{p['name']}@{p['version']}"
        elif source:
            component["properties"].append({"name": PROP + "source", "value": source})
        if p.get("license"):
            component["licenses"] = [{"expression": p["license"]}]
        if (p["name"], p["version"]) in checksums:
            component["hashes"] = [{"alg": "SHA-256", "content": checksums[(p["name"], p["version"])]}]
        out.append(component)
    return out


# ---------------------------------------------------------------------------


def sbom_for(path: Path, version: str | None, contained: list[Component]) -> dict[str, Any]:
    kind, declared_version, purl, requires = describe(path)
    digest = sha256(path)
    epoch = int(os.environ.get("SOURCE_DATE_EPOCH") or time.time())
    subject: Component = {
        "type": "library" if kind in {"wheel", "sdist", "npm", "crate"} else "application",
        "bom-ref": "artifact",
        "name": NAME,
        "version": declared_version or version or "unknown",
        "hashes": [{"alg": "SHA-256", "content": digest}],
        "licenses": [{"expression": LICENSE}],
        "properties": [
            {"name": PROP + "artifact", "value": path.name},
            {"name": PROP + "artifact-kind", "value": kind},
        ],
    }
    if purl:
        subject["purl"] = purl
    components = unique((contained if kind in BINARY_KINDS else []) + requires)
    return {
        "bomFormat": "CycloneDX",
        "specVersion": SPEC_VERSION,
        "serialNumber": f"urn:uuid:{uuid.uuid5(SERIAL_NAMESPACE, digest)}",
        "version": 1,
        "metadata": {
            "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(epoch)),
            "tools": {"components": [{"type": "application", "name": "scripts/sbom.py"}]},
            "component": subject,
        },
        "components": components,
        "dependencies": [
            {"ref": "artifact", "dependsOn": [c["bom-ref"] for c in components]},
            *({"ref": c["bom-ref"]} for c in components),
        ],
    }


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--version", help="version for artifacts that do not declare one (a bare binary)")
    parser.add_argument("--cargo", action="store_true", help="list the crates compiled into binary artifacts")
    parser.add_argument("artifacts", nargs="+", type=Path)
    args = parser.parse_args(argv)

    contained = cargo_components() if args.cargo else []
    for path in args.artifacts:
        if not path.is_file():
            raise SystemExit(f"not a file: {path}")
        out = path.with_name(path.name + ".cdx.json")
        document = sbom_for(path, args.version, contained)
        out.write_text(json.dumps(document, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        print(f"wrote {out} ({len(document['components'])} components)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
