# ---------------------------------------------------------------------------
# pkg-rpm.mk — the .rpm, built by cargo-generate-rpm from
# [package.metadata.generate-rpm] in Cargo.toml (module: pkg-rpm).
# Published by .github/workflows/package-rpm.yml.
# ---------------------------------------------------------------------------

.PHONY: package-rpm

CARGO_GENERATE_RPM_VERSION ?= 0.21.0

##@ Distribution packages

# Packages the binary lang-build just produced; no rpmbuild is involved.
# SOURCE_DATE_EPOCH, exported by make/release.mk, clamps every timestamp.
package-rpm: lang-build ## Build the .rpm into dist/ (needs cargo-generate-rpm)
	@command -v cargo-generate-rpm >/dev/null 2>&1 || { \
		echo "!!! cargo-generate-rpm not found. Install: cargo install cargo-generate-rpm@$(CARGO_GENERATE_RPM_VERSION) --locked"; exit 1; }
	@echo "==> [$(PROJECT_SHORT)] Building the .rpm..."
	@$(CARGO) generate-rpm --output $(DIST_DIR)/
