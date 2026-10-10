# ---------------------------------------------------------------------------
# pkg-flatpak-rust.mk — every crate in Cargo.lock as a Flatpak source, with its
# checksum, so the offline sandbox can build (module: pkg-flatpak, rust profile).
# ---------------------------------------------------------------------------

# A pinned revision of flatpak-builder-tools, checked by hash before it runs.
FLATPAK_CARGO_GENERATOR ?= https://raw.githubusercontent.com/flatpak/flatpak-builder-tools/41c20aa10819cdb2a4f3ca171758a96d1955c018/cargo/flatpak-cargo-generator.py
FLATPAK_CARGO_GENERATOR_SHA256 ?= 0a2db6be87d75910facef28ab46d4d6460802e8419ab850d0caa6a364d26b380
FLATPAK_VENV ?= .cache/flatpak-venv

flatpak-sources: packaging/flatpak/cargo-sources.json

packaging/flatpak/cargo-sources.json: Cargo.lock
	@echo "==> [$(PROJECT_SHORT)] Listing the crates for the offline Flatpak build..."
	@mkdir -p .cache
	@curl -sSfL -o .cache/flatpak-cargo-generator.py "$(FLATPAK_CARGO_GENERATOR)"
	@echo "$(FLATPAK_CARGO_GENERATOR_SHA256)  .cache/flatpak-cargo-generator.py" | sha256sum --check --quiet
	@test -x $(FLATPAK_VENV)/bin/python || { python3 -m venv $(FLATPAK_VENV) \
		&& $(FLATPAK_VENV)/bin/python -m pip install --quiet --disable-pip-version-check "aiohttp==3.14.3" "tomlkit==0.15.1"; }
	@$(FLATPAK_VENV)/bin/python .cache/flatpak-cargo-generator.py Cargo.lock -o $@
