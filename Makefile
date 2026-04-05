# =============================================================================
# Makefile for apptainer-compose
#
# All build, test, and lint commands run inside Docker containers.
# Nothing is assumed to be installed on the host except Docker.
#
# Usage:
#   make build              - Debug build (musl static binary)
#   make release            - Release build (musl static binary)
#   make clean              - Remove build artifacts
#   make check              - Run cargo check
#   make clippy             - Run cargo clippy
#   make fmt                - Check formatting (cargo fmt --check)
#   make fmt-fix            - Apply formatting (cargo fmt)
#   make test               - Run unit tests (add integration=1 for integration tests too)
#   make test-unit          - Run unit tests only
#   make test-integration   - Build debug binary, then run integration tests in apptainer
#   make verify             - Verify release binary is statically linked
#   make all                - fmt, check, clippy, test, build
#   make help               - Show this help
# =============================================================================

# Docker images
RUST_IMAGE      := rust:latest
APPTAINER_IMAGE := kaczmarj/apptainer
ALPINE_IMAGE    := alpine:latest

# Musl target triple
TARGET := x86_64-unknown-linux-musl

# Common Docker flags
DOCKER_RUN := docker run --rm -v $$(pwd):/workspace -w /workspace

# Rust container: add musl target before running any cargo command
RUST_RUN := $(DOCKER_RUN) $(RUST_IMAGE) sh -c "rustup target add $(TARGET) && $(1)"

# Binary paths
DEBUG_BIN   := target/$(TARGET)/debug/apptainer-compose
RELEASE_BIN := target/$(TARGET)/release/apptainer-compose
DEBUG_BIN_ALT   := target/$(TARGET)/debug/apptainer
RELEASE_BIN_ALT := target/$(TARGET)/release/apptainer

# Check if integration tests are requested
integration ?= 0

.PHONY: all build release clean check clippy fmt fmt-fix test test-unit test-integration verify help

# Default target
all: fmt check clippy test build

build:
	@echo "==> Building debug binary ($(TARGET)) in Docker..."
	@$(DOCKER_RUN) $(RUST_IMAGE) sh -c "rustup target add $(TARGET) && cargo build --target $(TARGET)"

release:
	@echo "==> Building release binary ($(TARGET)) in Docker..."
	@$(DOCKER_RUN) $(RUST_IMAGE) sh -c "rustup target add $(TARGET) && cargo build --release --target $(TARGET)"

clean:
	@echo "==> Cleaning build artifacts in Docker..."
	@$(DOCKER_RUN) $(RUST_IMAGE) cargo clean

check:
	@echo "==> Running cargo check in Docker..."
	@$(DOCKER_RUN) $(RUST_IMAGE) sh -c "rustup target add $(TARGET) && cargo check --target $(TARGET)"

clippy:
	@echo "==> Running cargo clippy in Docker..."
	@$(DOCKER_RUN) $(RUST_IMAGE) sh -c "rustup target add $(TARGET) && cargo clippy --target $(TARGET)"

fmt:
	@echo "==> Checking formatting in Docker..."
	@$(DOCKER_RUN) $(RUST_IMAGE) cargo fmt --check

fmt-fix:
	@echo "==> Applying formatting in Docker..."
	@$(DOCKER_RUN) $(RUST_IMAGE) cargo fmt

test-unit:
	@echo "==> Running unit tests in Docker..."
	@$(DOCKER_RUN) $(RUST_IMAGE) sh -c "rustup target add $(TARGET) && cargo test --target $(TARGET)"

test-integration:
	@echo "==> Building debug binary for integration tests..."
	@$(DOCKER_RUN) $(RUST_IMAGE) sh -c "rustup target add $(TARGET) && cargo build --target $(TARGET)"
	@echo "==> Running integration tests in apptainer container (privileged)..."
	@$(DOCKER_RUN) --privileged --entrypoint /bin/sh $(APPTAINER_IMAGE) -c "/workspace/tests/integration/run_tests.sh"

test: test-unit
ifeq ($(integration),1)
	@$(MAKE) test-integration
endif

verify:
	@echo "==> Verifying release binary is statically linked..."
	@$(DOCKER_RUN) $(ALPINE_IMAGE) sh -c "ldd /workspace/$(RELEASE_BIN) || true"

help:
	@echo "apptainer-compose Makefile"
	@echo ""
	@echo "All commands run inside Docker containers. Only Docker is required on the host."
	@echo ""
	@echo "Targets:"
	@echo "  build              Debug build (musl static binary)"
	@echo "  release            Release build (musl static binary)"
	@echo "  clean              Remove build artifacts"
	@echo "  check              Run cargo check"
	@echo "  clippy             Run cargo clippy"
	@echo "  fmt                Check formatting (cargo fmt --check)"
	@echo "  fmt-fix            Apply formatting (cargo fmt)"
	@echo "  test               Run unit tests (set integration=1 for integration tests too)"
	@echo "  test-unit          Run unit tests only"
	@echo "  test-integration   Build debug binary, then run integration tests in apptainer"
	@echo "  verify             Verify release binary is statically linked"
	@echo "  all                Run fmt, check, clippy, test, and build"
	@echo "  help               Show this help"
	@echo ""
	@echo "Examples:"
	@echo "  make build                  # Debug build"
	@echo "  make release verify         # Release build and verify it is static"
	@echo "  make test integration=1     # Unit + integration tests"
