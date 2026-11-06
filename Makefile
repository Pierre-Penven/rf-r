.PHONY: check fmt clippy test doc coverage audit docs-check docs bench

check: fmt clippy test doc coverage audit docs-check

fmt:
	cargo fmt --all --check

clippy:
	cargo clippy --locked --all-targets --all-features -- -D warnings

test:
	cargo test --locked --all-features

doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps

coverage:
	cargo llvm-cov --locked --all-features --fail-under-lines 100

audit:
	cargo audit

docs-check:
	@bash scripts/docs/check_docs.sh
	@python3 scripts/docs/check_links.py .

docs: docs-check
	@echo "Documentation checks passed"

bench:
	cargo bench
