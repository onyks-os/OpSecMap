# ---------------------------------------------------------------------------
# docs.mk — ADR scaffolding. The documentation site, when the project has one,
# lives in make/features/mkdocs.mk.
# ---------------------------------------------------------------------------

.PHONY: adr

ADR_DIR := docs/decisions

##@ Documentation

adr: ## Scaffold a new ADR — make adr TITLE="Short decision title"
	@if [ -z "$(TITLE)" ]; then \
		echo "Usage: make adr TITLE=\"Short decision title\""; exit 2; \
	fi
	@next=$$(printf '%04d' $$(( $$(ls $(ADR_DIR) | grep -cE '^[0-9]{4}-') ))); \
	slug=$$(echo "$(TITLE)" | tr '[:upper:]' '[:lower:]' | sed -e 's/[^a-z0-9]\+/-/g' -e 's/^-//' -e 's/-$$//'); \
	target="$(ADR_DIR)/$$next-$$slug.md"; \
	if [ -e "$$target" ]; then echo "!!! $$target already exists."; exit 1; fi; \
	sed -e "s/^# ADR NNNN: .*/# ADR $$next: $(TITLE)/" \
	    -e "s/^Proposed | Accepted.*/Proposed/" \
	    $(ADR_DIR)/0000-adr-template.md > "$$target"; \
	echo "==> Created $$target"; \
	echo "==> Remember to register it in $(ADR_DIR)/README.md"
