# ---------------------------------------------------------------------------
# pkg-deb.mk — the .deb, built by cargo-deb from [package.metadata.deb] in
# Cargo.toml (module: pkg-deb). Published by .github/workflows/package-deb.yml.
# ---------------------------------------------------------------------------

.PHONY: package-deb

CARGO_DEB_VERSION ?= 3.8.0

##@ Distribution packages

# --no-build packages the binary lang-build just produced, so the .deb holds
# the same bytes as the bare binary in the release. SOURCE_DATE_EPOCH, exported
# by make/release.mk, fixes every timestamp inside it.
package-deb: lang-build ## Build the .deb into dist/ (needs cargo-deb)
	@command -v cargo-deb >/dev/null 2>&1 || { \
		echo "!!! cargo-deb not found. Install: cargo install cargo-deb@$(CARGO_DEB_VERSION) --locked"; exit 1; }
	@echo "==> [$(PROJECT_SHORT)] Building the .deb..."
	@$(CARGO) deb --no-build --output $(DIST_DIR)/
