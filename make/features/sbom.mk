# ---------------------------------------------------------------------------
# sbom.mk — a CycloneDX SBOM next to every release artifact (module: sbom).
#
# `make release-artifacts` runs `sbom` before writing SHA256SUMS, so every
# <artifact>.cdx.json is checksummed and signed with the artifact it describes.
# Workflows that build an artifact elsewhere (a distribution container, a
# Flatpak runtime) describe theirs with `make sbom-files FILES=...`.
# ---------------------------------------------------------------------------

.PHONY: sbom sbom-files sbom-validate

SBOM_PYTHON ?= python3
# Extra arguments for scripts/sbom.py; a language fragment may add to them.
SBOM_ARGS   ?=
# The schema validator runs from its own virtualenv, so it needs nothing from the
# project's environment and works the same in every language profile.
SBOM_VENV   ?= .cache/sbom-venv
SBOM_VALIDATOR_REQ ?= cyclonedx-python-lib[json-validation]==11.12.0

# Everything in dist/ except what describes, checksums or signs something else.
_sbom_artifacts = $(shell find $(DIST_DIR) -maxdepth 1 -type f ! -name '*.cdx.json' \
	! -name 'SHA256SUMS*' ! -name '*.sigstore*' 2>/dev/null | LC_ALL=C sort)

##@ SBOM

sbom: build ## Write and validate a CycloneDX SBOM for every artifact in dist/
	@$(MAKE) --no-print-directory sbom-files FILES="$(_sbom_artifacts)"

sbom-files: ## SBOMs for the given artifacts only — make sbom-files FILES="dist/x.deb"
	@test -n "$(strip $(FILES))" || { echo "!!! No artifacts to describe (FILES is empty)."; exit 1; }
	@echo "==> [$(PROJECT_SHORT)] Writing SBOMs..."
	@$(SBOM_PYTHON) scripts/sbom.py --version "$(VERSION)" $(SBOM_ARGS) $(FILES)
	@$(MAKE) --no-print-directory sbom-validate FILES="$(addsuffix .cdx.json,$(FILES))"

sbom-validate: ## Validate SBOMs against the CycloneDX schema (default: dist/*.cdx.json)
	@if [ ! -x $(SBOM_VENV)/bin/python ]; then \
		echo "==> [$(PROJECT_SHORT)] Installing the CycloneDX validator into $(SBOM_VENV)..."; \
		$(SBOM_PYTHON) -m venv $(SBOM_VENV) \
		&& $(SBOM_VENV)/bin/python -m pip install --quiet --disable-pip-version-check "$(SBOM_VALIDATOR_REQ)"; \
	fi
	@$(SBOM_VENV)/bin/python scripts/validate-sbom.py $(or $(FILES),$(wildcard $(DIST_DIR)/*.cdx.json))

checksums: sbom
RELEASE_EXPECT += *.cdx.json
