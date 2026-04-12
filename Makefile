CARGO       := cargo
RUSTLAB     := cargo run -q --bin rustlab --
INSTALL_DIR := $(HOME)/.local/bin
UNAME       := $(shell uname)

# All examples. Those marked (*) open an interactive terminal chart.
EXAMPLES_ALL := \
	complex_basics \
	save_load \
	firpm \
	fixed_point \
	vectors \
	lowpass \
	bandpass \
	fft \
	kaiser_fir \
	random

# Non-interactive subset — safe for headless / CI runs.
EXAMPLES_CI := complex_basics save_load firpm fixed_point

.PHONY: install install-viewer examples examples-ci perf clean-examples clean help $(EXAMPLES_ALL)

## Build release binaries (rustlab + rustlab-viewer + rustlab-notebook) and install to $(INSTALL_DIR)
install:
	$(CARGO) build --release --features viewer
	$(CARGO) build --release -p rustlab-notebook
	mkdir -p $(INSTALL_DIR)
	cp target/release/rustlab $(INSTALL_DIR)/rustlab
	cp target/release/rustlab-viewer $(INSTALL_DIR)/rustlab-viewer
	cp target/release/rustlab-notebook $(INSTALL_DIR)/rustlab-notebook
ifeq ($(UNAME), Darwin)
	codesign --sign - --force $(INSTALL_DIR)/rustlab
	codesign --sign - --force $(INSTALL_DIR)/rustlab-viewer
	codesign --sign - --force $(INSTALL_DIR)/rustlab-notebook
endif
	@echo "Installed to $(INSTALL_DIR)/rustlab, $(INSTALL_DIR)/rustlab-viewer, and $(INSTALL_DIR)/rustlab-notebook"
	@echo "Make sure $(INSTALL_DIR) is on your PATH"

## Build and install rustlab-viewer (interactive egui plot viewer)
install-viewer:
	$(CARGO) build --release -p rustlab-viewer
	mkdir -p $(INSTALL_DIR)
	cp target/release/rustlab-viewer $(INSTALL_DIR)/rustlab-viewer
ifeq ($(UNAME), Darwin)
	codesign --sign - --force $(INSTALL_DIR)/rustlab-viewer
endif
	@echo "Installed to $(INSTALL_DIR)/rustlab-viewer"

## Run all examples (interactive ones require a real terminal)
examples:
	@for ex in $(EXAMPLES_ALL); do \
		printf "\n── %s ──────────────────────────────────\n" "$$ex"; \
		$(RUSTLAB) run examples/$$ex.r; \
	done

## Run non-interactive examples only (safe for headless / CI use)
examples-ci:
	@for ex in $(EXAMPLES_CI); do \
		printf "\n── %s ──────────────────────────────────\n" "$$ex"; \
		$(RUSTLAB) run examples/$$ex.r; \
	done

## Run a single example by name:  make lowpass
$(EXAMPLES_ALL):
	$(RUSTLAB) run examples/$@.r

## Build release binary, run all benchmarks, and write perf/report.md
perf:
	@bash perf/run_perf.sh

## Remove generated output files (*.svg *.npy *.csv *.npz)
clean-examples:
	@rm -f *.svg *.npy *.csv *.npz
	@echo "Example outputs removed."

## Remove example outputs and cargo build artifacts
clean: clean-examples
	$(CARGO) clean

## Show this help
help:
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "  install             Build release binary and install to $(INSTALL_DIR) (override with INSTALL_DIR=...)"
	@echo "                      (runs codesign automatically on macOS, skips it on Linux)"
	@echo "  examples            Run all examples (interactive ones need a real terminal)"
	@echo "  examples-ci         Run non-interactive examples only"
	@echo "  <name>              Run one example, e.g.  make lowpass"
	@echo "  clean-examples      Remove *.svg *.npy *.csv *.npz from the workspace root"
	@echo "  clean               clean-examples + cargo clean"
	@echo "  perf                Build release, run benchmarks, write perf/report.md"
	@echo ""
	@echo "  Available examples:"
	@for ex in $(EXAMPLES_ALL); do echo "    $$ex"; done
	@echo ""
