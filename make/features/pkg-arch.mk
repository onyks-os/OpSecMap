# ---------------------------------------------------------------------------
# pkg-arch.mk — the Arch Linux package, built by makepkg from
# packaging/arch/PKGBUILD (module: pkg-arch).
# Published by .github/workflows/package-arch.yml.
# ---------------------------------------------------------------------------

.PHONY: package-arch check-pkgbuild

##@ Distribution packages

# makepkg builds from an archive of the committed tree (git archive HEAD), never
# from a download of the GitHub tarball, so a package built before the tag is
# pushed is built exactly as the released one will be. Runs on Arch, as a user
# that is not root (makepkg refuses root).
package-arch: check-pkgbuild ## Build the Arch package from the committed tree into dist/ (on Arch)
	@command -v makepkg >/dev/null 2>&1 || { echo "!!! makepkg not found: this target runs on Arch Linux."; exit 1; }
	@echo "==> [$(PROJECT_SHORT)] Building the Arch package..."
	@rm -f packaging/arch/*.pkg.tar.zst packaging/arch/*.tar.gz
	@git archive --prefix="$(PROJECT_DIST)-$(VERSION)/" -o "packaging/arch/$(PROJECT_DIST)-$(VERSION).tar.gz" HEAD
	@cd packaging/arch && makepkg --force --cleanbuild --noconfirm $(MAKEPKG_ARGS)
	@mkdir -p $(DIST_DIR)
	@# makepkg may also split out a -debug package with the symbols; users install the main one.
	@find packaging/arch -maxdepth 1 -name '*.pkg.tar.zst' ! -name '*-debug-*' -exec cp {} $(DIST_DIR)/ \;

# The PKGBUILD carries its own copy of the version, so it is checked against the
# manifest before a build and before a release is tagged.
check-pkgbuild:
	@grep -qx "pkgver=$(VERSION)" packaging/arch/PKGBUILD \
		|| { echo "!!! packaging/arch/PKGBUILD: pkgver is not $(VERSION). Update it with the version."; exit 1; }

release-check: check-pkgbuild
