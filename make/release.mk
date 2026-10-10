# ---------------------------------------------------------------------------
# release.mk — version handling, artifact build, checksums, signing, notes.
#
# The release itself is performed by .github/workflows/release.yml when a
# v* tag is pushed; these targets exist to rehearse and verify it locally.
#
# `make release-artifacts` is the single entry point the workflow calls, and
# everything it signs and publishes is whatever that target leaves in dist/.
# Optional modules (make/features/*.mk) extend it without editing this file:
#
#   checksums: my-extra          # runs before SHA256SUMS is written
#   my-extra: build              # and after the artifacts exist
#   RELEASE_EXPECT += *.my-ext   # and the release fails if it produced nothing
#
# Make merges the prerequisites of every rule that names the same target, so a
# module adds a step by naming it, and removing the module removes the step.
# ---------------------------------------------------------------------------

.PHONY: build version release-artifacts checksums sign release-check release-dry release-notes

DIST_DIR ?= dist

# Globs, relative to $(DIST_DIR), that must each match at least one file before
# anything is signed. The language fragment names its artifacts; modules append.
# A build that silently skips an artifact would otherwise publish an incomplete
# release that nothing flags.
RELEASE_EXPECT ?=

# Reproducible builds: every timestamp a build tool writes comes from the commit
# being built, not from the clock, so two builds of one commit can match byte for
# byte. An explicit value wins. Left unset outside a git checkout, because an
# empty SOURCE_DATE_EPOCH makes some build backends fail rather than ignore it.
ifeq ($(origin SOURCE_DATE_EPOCH),undefined)
SOURCE_DATE_EPOCH := $(shell git log -1 --format=%ct 2>/dev/null)
endif
ifneq ($(SOURCE_DATE_EPOCH),)
export SOURCE_DATE_EPOCH
endif

##@ Release

version: ## Print the version the manifest declares
	@echo "$(VERSION)"

# dist/ is emptied first: an artifact left over from a previous build — the last
# version's package, a renamed binary — would otherwise be signed and published
# alongside the new ones.
build: ## Build the distributable artifacts into dist/ (from scratch)
	@rm -rf $(DIST_DIR)
	@$(MAKE) --no-print-directory lang-build

release-artifacts: checksums ## Everything release.yml signs and publishes, into dist/
	@missing=0; \
	for pattern in $(RELEASE_EXPECT); do \
		if ! compgen -G "$(DIST_DIR)/$$pattern" > /dev/null; then \
			echo "!!! No release artifact matched $(DIST_DIR)/$$pattern"; missing=1; \
		fi; \
	done; \
	[ "$$missing" -eq 0 ] || exit 1
	@echo "==> [$(PROJECT_SHORT)] Release artifacts in $(DIST_DIR)/:"
	@ls -la $(DIST_DIR)/

# Written through a sorted file list, not `sha256sum *`: a glob expanded after the
# redirection has created SHA256SUMS would hash the half-written file itself, and
# an unsorted list makes the file differ between two identical builds.
checksums: build ## Generate SHA256SUMS for everything in dist/
	@echo "==> [$(PROJECT_SHORT)] Generating checksums..."
	@cd $(DIST_DIR) && find . -maxdepth 1 -type f ! -name 'SHA256SUMS*' ! -name '*.sigstore*' -printf '%P\n' \
		| LC_ALL=C sort | xargs -r -d '\n' sha256sum -- > SHA256SUMS.tmp \
		&& mv SHA256SUMS.tmp SHA256SUMS && cat SHA256SUMS

sign: ## Clearsign dist/SHA256SUMS with GPG (CI uses Sigstore instead)
	@test -f $(DIST_DIR)/SHA256SUMS || { echo "!!! No $(DIST_DIR)/SHA256SUMS. Run: make release-artifacts"; exit 1; }
	@echo "==> [$(PROJECT_SHORT)] Signing SHA256SUMS (key: $(or $(GPG_KEY_ID),default))..."
	@gpg --clearsign $(if $(GPG_KEY_ID),--local-user $(GPG_KEY_ID),) \
		--output $(DIST_DIR)/SHA256SUMS.asc $(DIST_DIR)/SHA256SUMS

# The body of a GitHub Release is the CHANGELOG section for the version, header
# included. GitHub's generated notes list merged pull requests instead, which on a
# repository whose work lands as direct commits yields a near-empty changelog and
# a "New Contributors" line crediting the maintainer as a first-time contributor.
#
# The section is copied verbatim, blank lines included; only trailing blank lines
# and the horizontal rule that separates versions are trimmed. Keep a Changelog
# puts every version's link definition at the end of the file, which is inside
# the oldest section: those are dropped, and this version's own is appended so
# the heading still links to its release.
release-notes: ## Print the CHANGELOG section for VERSION as release notes
	@awk -v ver="$(VERSION)" ' \
		index($$0, "[" ver "]: ") == 1 { ref = $$0 } \
		/^\[[^]]+\]: / { next } \
		index($$0, "## [" ver "]") == 1 { found = 1 } \
		found && index($$0, "## [") == 1 && index($$0, "## [" ver "]") != 1 { found = 0 } \
		found { lines[++n] = $$0 } \
		END { \
			while (n > 0 && (lines[n] ~ /^[[:space:]]*$$/ || lines[n] ~ /^---+$$/)) n--; \
			for (i = 1; i <= n; i++) print lines[i]; \
			if (n > 0 && ref != "") { print ""; print ref } \
		}' CHANGELOG.md

release-check: ## Verify the repository is ready to be tagged
	@echo "==> [$(PROJECT_SHORT)] Pre-release checks..."
	@test -z "$$(git status --porcelain)" \
		|| { echo "!!! Working tree is dirty."; exit 1; }
	@grep -q "## \[$(VERSION)\]" CHANGELOG.md \
		|| { echo "!!! CHANGELOG.md has no section for $(VERSION)."; exit 1; }
	@! grep -rIq "TODO(template)" README.md SECURITY.md \
		|| { echo "!!! README.md or SECURITY.md still contains template placeholders."; exit 1; }
	@echo "==> Ready to tag: git tag -s v$(VERSION) -m \"$(PROJECT_SHORT) v$(VERSION)\""

release-dry: release-check verify release-artifacts ## Full local release rehearsal
	@echo "======================================================================"
	@echo "[$(PROJECT_SHORT)] Release rehearsal complete. Artifacts in $(DIST_DIR)/:"
	@ls -la $(DIST_DIR)/
	@echo "======================================================================"
