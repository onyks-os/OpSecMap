# ---------------------------------------------------------------------------
# reproducible.mk — prove the release build is deterministic (module: reproducible).
# ---------------------------------------------------------------------------

.PHONY: check-reproducible

# Run in each copy before its build; a language fragment sets it when the build
# needs dependencies installed first.
REPRO_PREPARE ?=
# Added to the make command line of each build, e.g. an interpreter to reuse.
REPRO_MAKE_ARGS ?=

##@ Reproducible builds

check-reproducible: ## Build twice from two copies of the tree; fail unless identical
	@REPRO_PREPARE="$(REPRO_PREPARE)" REPRO_MAKE_ARGS="$(REPRO_MAKE_ARGS)" ./scripts/check-reproducible.sh $(OUT)
