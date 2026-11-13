# OpSecMap (OpSecMap) — developer entrypoint.
#
# Every target is documented inline; run `make help` for the full list.
# Logic lives in the modular fragments under make/ so that this file stays readable:
#
#   make/common.mk   — environment, help, housekeeping, TODO tracking
#   make/quality.mk  — lint, format, audit, security gates
#   make/docs.mk     — ADR scaffolding
#   make/release.mk  — versioning, build, checksums, signing, release notes
#   make/rust.mk  — language-specific implementation of the target contract
#   make/features/*.mk — one fragment per optional module chosen at scaffold time
#   make/project.mk  — optional, committed, targets that exist only in this project
#   make/local.mk    — optional, git-ignored, machine-local overrides

SHELL := /bin/bash
.DEFAULT_GOAL := help

# ---------------------------------------------------------------------------
# Project identity — the single source of truth for scripts and workflows.
# ---------------------------------------------------------------------------
PROJECT_NAME  := OpSecMap
PROJECT_SHORT := OpSecMap
PROJECT_SLUG  := OpSecMap
PROJECT_PKG   := opsecmap
PROJECT_DIST  := opsecmap
GITHUB_OWNER  := onyks-os
# Prefer the language manifest over a second copy of the version number. Two
# copies drift, and the one that drifts is the one nobody edits: a scaffolded
# project hit exactly this on its first release, with `make release-check`
# printing the previous version's tag for a tree that built the next one.
#
# Falls back to the scaffolded literal, so a profile without a manifest (generic)
# keeps working. package.json is read by its first "version" key, which is the
# package's own: npm writes it before any nested object.
_MANIFEST_VERSION := $(or \
	$(shell grep -h -m1 '^version = ' pyproject.toml Cargo.toml 2>/dev/null | head -1 | cut -d '"' -f2), \
	$(shell grep -m1 -E '^[[:space:]]*"version"[[:space:]]*:' package.json 2>/dev/null | cut -d '"' -f4))
VERSION       := $(or $(_MANIFEST_VERSION),0.1.0)

# Directories that hold first-party source, tests, and shell scripts.
SRC_DIRS     := opsecmap
TEST_DIRS    := tests
SCRIPT_DIRS  := scripts

include make/common.mk
include make/quality.mk
include make/docs.mk
include make/release.mk
include make/rust.mk
# Optional modules. Removing a module is deleting its fragment and its files;
# nothing in the fragments above names one.
-include $(sort $(wildcard make/features/*.mk))
-include make/project.mk
-include make/local.mk
