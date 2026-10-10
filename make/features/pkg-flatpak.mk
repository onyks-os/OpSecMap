# ---------------------------------------------------------------------------
# pkg-flatpak.mk — a single-file Flatpak bundle, built by flatpak-builder from
# packaging/flatpak/io.github.onyks_it.OpSecMap.yml (module: pkg-flatpak).
# Published by .github/workflows/package-flatpak.yml.
# ---------------------------------------------------------------------------

.PHONY: package-flatpak flatpak-sources

FLATPAK_APP_ID   := io.github.onyks_it.OpSecMap
FLATPAK_MANIFEST := packaging/flatpak/$(FLATPAK_APP_ID).yml
FLATPAK_REMOTE   ?= https://dl.flathub.org/repo/flathub.flatpakrepo

##@ Distribution packages

# The sandboxed build has no network, so everything it needs is declared as a
# source in the manifest first (flatpak-sources; a language fragment may add to
# it). The runtime and SDK come from Flathub. Once built, the program is run
# inside the build's own sandbox, which is where it will run for a user.
package-flatpak: flatpak-sources ## Build dist/<name>-<version>.flatpak (needs flatpak-builder)
	@command -v flatpak-builder >/dev/null 2>&1 || { echo "!!! flatpak-builder not found."; exit 1; }
	@echo "==> [$(PROJECT_SHORT)] Building the Flatpak..."
	@flatpak-builder --user --install-deps-from=flathub --force-clean --disable-rofiles-fuse \
		--state-dir=.flatpak-builder --repo=.flatpak-repo build-flatpak $(FLATPAK_MANIFEST)
	@mkdir -p $(DIST_DIR)
	@flatpak build-bundle --runtime-repo=$(FLATPAK_REMOTE) .flatpak-repo \
		$(DIST_DIR)/$(PROJECT_DIST)-$(VERSION).flatpak $(FLATPAK_APP_ID)
	@echo "==> [$(PROJECT_SHORT)] Running it in the sandbox: $(PACKAGE_SMOKE_CMD)"
	@flatpak-builder --run build-flatpak $(FLATPAK_MANIFEST) $(PACKAGE_SMOKE_CMD)

flatpak-sources:
