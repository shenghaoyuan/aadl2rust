# =========================
# Toolchain & basic config
# =========================

CARGO          := cargo
RUSTUP_TOOLCHAIN := +nightly

# =========================
# llvm-cov ignore settings
# =========================

LLVM_COV_IGNORE_REGEX := collector\.rs|generate_build\.rs|intermediate_print\.rs|merge_utils\.rs|tool\.rs|ast\.rs|main\.rs|printmessage\.rs|test_mod\.rs|transform\.rs|transform_annex\.rs|model_statistics\.rs|test_mod2\.rs

# =========================
# Default target
# =========================

.PHONY: all
all: cov

# =========================
# Coverage targets
# =========================

.PHONY: cov
cov:
	$(CARGO) $(RUSTUP_TOOLCHAIN) llvm-cov \
		--branch \
		--open \
		--ignore-filename-regex "$(LLVM_COV_IGNORE_REGEX)"

# =========================
# Clean coverage artifacts
# =========================

.PHONY: cov-clean
cov-clean:
	$(CARGO) llvm-cov clean

.PHONY: code
code:
	@echo "===== Code Line Statistics (cloc, code-only) ====="
	@echo "[AST declaration]         $$(cloc src/ast.rs src/aadl_ast2rust_code/intermediate_ast.rs --json | jq '.SUM.code')"
	@echo "[Parsing]                 $$(cloc src/aadl.pest src/transform.rs src/transform_annex.rs --json | jq '.SUM.code')"
	@echo "[Model-to-IR translation] $$(cloc \
		src/aadl_ast2rust_code/converter.rs \
		src/aadl_ast2rust_code/converter_annex.rs \
		src/aadl_ast2rust_code/implementations \
		src/aadl_ast2rust_code/types \
		--json | jq '.SUM.code')"
	@echo "[Rust code printer]       $$(cloc src/aadl_ast2rust_code/intermediate_print.rs --json | jq '.SUM.code')"
	@echo "-----------------------------------------------"
	@echo "[Total]                   $$(( \
		$$(cloc src/ast.rs --json | jq '.SUM.code') + \
		$$(cloc src/aadl.pest src/transform.rs src/transform_annex.rs --json | jq '.SUM.code') + \
		$$(cloc \
			src/aadl_ast2rust_code/converter.rs \
			src/aadl_ast2rust_code/converter_annex.rs \
			src/aadl_ast2rust_code/implementations \
			src/aadl_ast2rust_code/types \
			--json | jq '.SUM.code') + \
		$$(cloc src/aadl_ast2rust_code/intermediate_print.rs --json | jq '.SUM.code') \
	))"