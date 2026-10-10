# ---------------------------------------------------------------------------
# packaging.mk — what every distribution package module shares
# (module: packaging-support, pulled in by pkg-deb, pkg-rpm, pkg-arch, pkg-flatpak).
# ---------------------------------------------------------------------------

.PHONY: smoke-test-package

# Run inside each clean container once the package is installed. It must prove
# the installed program starts, not merely that files were copied.
PACKAGE_SMOKE_CMD ?= $(PROJECT_PKG) --version

##@ Distribution packages

smoke-test-package: ## Install a package in a clean container and run it — PACKAGE=<file> IMAGE=<image>
	@test -n "$(PACKAGE)" -a -n "$(IMAGE)" \
		|| { echo "Usage: make smoke-test-package PACKAGE=dist/<file> IMAGE=docker.io/library/debian:13"; exit 2; }
	@./scripts/smoke-test-package.sh "$(PACKAGE)" "$(IMAGE)" $(PACKAGE_SMOKE_CMD)
