# =========================
# Toolchain & basic config
# =========================

CARGO          := cargo
RUSTUP_TOOLCHAIN := +nightly

# =========================
# llvm-cov ignore settings
# =========================

LLVM_COV_IGNORE_REGEX := collector\.rs|generate_build\.rs|intermediate_print\.rs|merge_utils\.rs|tool\.rs|ast\.rs|main\.rs|printmessage\.rs|test_mod\.rs|transform\.rs|transform_annex\.rs

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
