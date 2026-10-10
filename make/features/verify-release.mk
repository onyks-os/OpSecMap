# ---------------------------------------------------------------------------
# verify-release.mk — check a published release as a user would (module: verify-release).
# ---------------------------------------------------------------------------

.PHONY: verify-release

##@ Release verification

verify-release: ## Verify a published release — make verify-release TAG=v1.2.3
	@test -n "$(TAG)" || { echo "Usage: make verify-release TAG=v$(VERSION)"; exit 2; }
	@./scripts/verify-release.sh "$(TAG)"
