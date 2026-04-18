CARGO       := cargo
INSTALL_DIR := $(HOME)/.local/bin
UNAME       := $(shell uname)

.PHONY: all build release test install perf octave-compare clean help

all: help

build:
	$(CARGO) build --workspace --features viewer

release:
	$(CARGO) build --release --features viewer
	$(CARGO) build --release -p rustlab-notebook

test:
	$(CARGO) test --workspace --features viewer

install: release
	mkdir -p $(INSTALL_DIR)
	cp target/release/rustlab $(INSTALL_DIR)/rustlab
	cp target/release/rustlab-viewer $(INSTALL_DIR)/rustlab-viewer
	cp target/release/rustlab-notebook $(INSTALL_DIR)/rustlab-notebook
ifeq ($(UNAME), Darwin)
	codesign --sign - --force $(INSTALL_DIR)/rustlab
	codesign --sign - --force $(INSTALL_DIR)/rustlab-viewer
	codesign --sign - --force $(INSTALL_DIR)/rustlab-notebook
endif
	@echo "Installed to $(INSTALL_DIR) (override with INSTALL_DIR=...)"

perf:
	@bash perf/run_perf.sh

octave-compare:
	@bash tests/octave/run_compare.sh

clean:
	$(CARGO) clean

help:
	@echo ""
	@echo "Usage: make <target>"
	@echo ""
	@echo "  build     Debug build (all crates)"
	@echo "  release   Release build (all crates)"
	@echo "  test      Run all tests"
	@echo "  install   Release build + install to $(INSTALL_DIR)"
	@echo "  perf      Release build, run benchmarks, write perf/report.md"
	@echo "  octave-compare  Regenerate CSVs and compare rustlab vs Octave (requires octave)"
	@echo "  clean     Remove build artifacts"
	@echo ""
	@echo "Workflow:  make build → make test → make install"
	@echo ""
