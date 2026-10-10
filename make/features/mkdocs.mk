# ---------------------------------------------------------------------------
# mkdocs.mk — the MkDocs documentation site (module: mkdocs).
# ---------------------------------------------------------------------------

.PHONY: docs docs-build docs-serve docs-sync

MKDOCS ?= $(shell command -v mkdocs 2>/dev/null || echo mkdocs)

##@ Documentation site

docs: docs-build ## Alias for docs-build

docs-build: docs-sync ## Build the MkDocs documentation site
	@echo "==> [$(PROJECT_SHORT)] Building MkDocs site..."
	@if command -v mkdocs >/dev/null 2>&1; then \
		$(MKDOCS) build --strict; \
	else \
		echo "==> mkdocs not found. Install: pip install -r docs/requirements.txt"; \
		exit 1; \
	fi

docs-serve: ## Serve the documentation with live reload on :8000
	@echo "==> [$(PROJECT_SHORT)] Starting MkDocs dev server (http://127.0.0.1:8000)..."
	@if command -v mkdocs >/dev/null 2>&1; then \
		$(MKDOCS) serve; \
	else \
		echo "==> mkdocs not found. Install: pip install -r docs/requirements.txt"; \
		exit 1; \
	fi

docs-sync: ## Mirror the root CHANGELOG into the documentation site
	@echo "==> [$(PROJECT_SHORT)] Syncing CHANGELOG into the docs site..."
	@# `0,/re/` rather than `1,/re/`: with `1,`, sed starts looking for the end
	@# pattern on line *two*, so a CHANGELOG whose first line is already
	@# "# Changelog" never matches and the entire file is deleted instead of
	@# having its heading stripped.
	@{ \
		echo "# Release Notes & Changelog"; \
		echo ""; \
		sed -e '0,/^# Changelog$$/d' CHANGELOG.md; \
	} | cat -s > docs/web/release-notes/changelog.md

# The local CI rehearsal builds the site too, as the Documentation workflow does.
ci-local: docs-build
