#!/bin/sh
# =============================================================================
# Integration test runner for apptainer-compose
#
# Runs inside a kaczmarj/apptainer container with the project mounted
# at /workspace. Tests validate YAML parsing, config merging, profile
# filtering, and (when Apptainer is available) pull/up/down lifecycle.
#
# Usage:
#   ./tests/integration/run_tests.sh [path-to-binary]
# =============================================================================

set -e

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------

BIN="${1:-/workspace/target/x86_64-unknown-linux-musl/debug/apptainer-compose}"
EXAMPLES="/workspace/examples"

# ---------------------------------------------------------------------------
# Color helpers
# ---------------------------------------------------------------------------

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BOLD='\033[1m'
RESET='\033[0m'

# ---------------------------------------------------------------------------
# Counters
# ---------------------------------------------------------------------------

PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0

# ---------------------------------------------------------------------------
# Helper functions
# ---------------------------------------------------------------------------

pass() {
    PASS_COUNT=$((PASS_COUNT + 1))
    echo -e "  ${GREEN}PASS${RESET} $1"
}

fail() {
    FAIL_COUNT=$((FAIL_COUNT + 1))
    echo -e "  ${RED}FAIL${RESET} $1"
    if [ -n "$2" ]; then
        echo -e "       ${RED}$2${RESET}"
    fi
}

skip() {
    SKIP_COUNT=$((SKIP_COUNT + 1))
    echo -e "  ${YELLOW}SKIP${RESET} $1"
}

run_test() {
    local name="$1"
    echo -e "${BOLD}--- $name${RESET}"
    "$name"
    echo ""
}

assert_exit_zero() {
    local description="$1"
    shift
    local output
    if output=$("$@" 2>&1); then
        echo "$output"
        return 0
    else
        local rc=$?
        echo "$output"
        fail "$description (exit code $rc)"
        return 1
    fi
}

assert_contains() {
    local output="$1"
    local pattern="$2"
    local label="$3"
    if echo "$output" | grep -qi "$pattern"; then
        return 0
    else
        fail "$label: expected output to contain '$pattern'"
        return 1
    fi
}

assert_not_contains() {
    local output="$1"
    local pattern="$2"
    local label="$3"
    if echo "$output" | grep -qi "$pattern"; then
        fail "$label: expected output NOT to contain '$pattern'"
        return 1
    else
        return 0
    fi
}

# ---------------------------------------------------------------------------
# Preflight checks
# ---------------------------------------------------------------------------

echo -e "${BOLD}=== apptainer-compose integration tests ===${RESET}"
echo ""

if [ ! -x "$BIN" ]; then
    echo -e "${RED}ERROR: binary not found or not executable: $BIN${RESET}"
    exit 1
fi

echo "Binary: $BIN"
echo ""

# ---------------------------------------------------------------------------
# Test functions
# ---------------------------------------------------------------------------

test_00_hello_world() {
    local output
    output=$("$BIN" -f "$EXAMPLES/00-hello-world/apptainer-compose.yml" config 2>&1) || {
        fail "test_00_hello_world: config command failed"
        return
    }
    local ok=true
    assert_contains "$output" "hello" "test_00_hello_world" || ok=false
    if $ok; then
        pass "test_00_hello_world"
    fi
}

test_01_web_server() {
    local output
    output=$("$BIN" -f "$EXAMPLES/01-web-server/apptainer-compose.yml" config 2>&1) || {
        fail "test_01_web_server: config command failed"
        return
    }
    local ok=true
    assert_contains "$output" "web" "test_01_web_server" || ok=false
    assert_contains "$output" "nginx" "test_01_web_server" || ok=false
    if $ok; then
        pass "test_01_web_server"
    fi
}

test_02_multi_service() {
    local output
    output=$(cd "$EXAMPLES/02-multi-service" && "$BIN" config 2>&1) || {
        fail "test_02_multi_service: config command failed"
        return
    }
    local ok=true
    assert_contains "$output" "api" "test_02_multi_service" || ok=false
    assert_contains "$output" "worker" "test_02_multi_service" || ok=false
    assert_contains "$output" "cache" "test_02_multi_service" || ok=false
    if $ok; then
        pass "test_02_multi_service"
    fi
}

test_03_environment_variables() {
    local output
    output=$(cd "$EXAMPLES/03-environment-variables" && "$BIN" config 2>&1) || {
        fail "test_03_environment_variables: config command failed"
        return
    }
    local ok=true
    assert_contains "$output" "DB_HOST" "test_03_environment_variables" || ok=false
    assert_contains "$output" "APP_ENV" "test_03_environment_variables" || ok=false
    if $ok; then
        pass "test_03_environment_variables"
    fi
}

test_04_volumes() {
    local output
    output=$(cd "$EXAMPLES/04-volumes" && "$BIN" config 2>&1) || {
        fail "test_04_volumes: config command failed"
        return
    }
    local ok=true
    assert_contains "$output" "shared-data" "test_04_volumes" || ok=false
    if $ok; then
        pass "test_04_volumes"
    fi
}

test_05_depends_on() {
    local output
    output=$(cd "$EXAMPLES/05-depends-on" && "$BIN" config 2>&1) || {
        fail "test_05_depends_on: config command failed"
        return
    }
    local ok=true
    assert_contains "$output" "db" "test_05_depends_on" || ok=false
    assert_contains "$output" "migrate" "test_05_depends_on" || ok=false
    assert_contains "$output" "api" "test_05_depends_on" || ok=false
    assert_contains "$output" "frontend" "test_05_depends_on" || ok=false
    if $ok; then
        pass "test_05_depends_on"
    fi
}

test_06_healthcheck() {
    local output
    output=$(cd "$EXAMPLES/06-healthcheck" && "$BIN" config 2>&1) || {
        fail "test_06_healthcheck: config command failed"
        return
    }
    local ok=true
    assert_contains "$output" "pg_isready" "test_06_healthcheck" || ok=false
    assert_contains "$output" "interval" "test_06_healthcheck" || ok=false
    assert_contains "$output" "service_healthy" "test_06_healthcheck" || ok=false
    if $ok; then
        pass "test_06_healthcheck"
    fi
}

test_07_gpu_nvidia() {
    local output
    output=$(cd "$EXAMPLES/07-gpu-nvidia" && "$BIN" config 2>&1) || {
        fail "test_07_gpu_nvidia: config command failed"
        return
    }
    local ok=true
    assert_contains "$output" "nvidia" "test_07_gpu_nvidia" || ok=false
    assert_contains "$output" "nv" "test_07_gpu_nvidia" || ok=false
    if $ok; then
        pass "test_07_gpu_nvidia"
    fi
}

test_08_build_from_dockerfile() {
    local output
    output=$(cd "$EXAMPLES/08-build-from-dockerfile" && "$BIN" config 2>&1) || {
        fail "test_08_build_from_dockerfile: config command failed"
        return
    }
    local ok=true
    assert_contains "$output" "build" "test_08_build_from_dockerfile" || ok=false
    if $ok; then
        pass "test_08_build_from_dockerfile"
    fi
}

test_09_multi_file_override() {
    local output
    output=$("$BIN" \
        -f "$EXAMPLES/09-multi-file-override/apptainer-compose.yml" \
        -f "$EXAMPLES/09-multi-file-override/apptainer-compose.override.yml" \
        config 2>&1) || {
        fail "test_09_multi_file_override: config command failed"
        return
    }
    local ok=true
    # Verify both services are present after merging two files
    assert_contains "$output" "web" "test_09_multi_file_override" || ok=false
    assert_contains "$output" "api" "test_09_multi_file_override" || ok=false
    if $ok; then
        pass "test_09_multi_file_override"
    fi
}

test_10_profiles() {
    # Config always shows all services (profile filtering happens at runtime)
    local output
    output=$(cd "$EXAMPLES/10-profiles" && "$BIN" config 2>&1) || {
        fail "test_10_profiles: config command failed"
        return
    }
    local ok=true
    # All services should be present in config output
    assert_contains "$output" "app" "test_10_profiles" || ok=false
    assert_contains "$output" "db" "test_10_profiles" || ok=false
    assert_contains "$output" "debug-shell" "test_10_profiles" || ok=false
    assert_contains "$output" "prometheus" "test_10_profiles" || ok=false
    # Profile assignments should be visible
    assert_contains "$output" "debug" "test_10_profiles (profiles field)" || ok=false
    assert_contains "$output" "monitoring" "test_10_profiles (profiles field)" || ok=false
    if $ok; then
        pass "test_10_profiles"
    fi
}

test_11_resource_limits() {
    local output
    output=$(cd "$EXAMPLES/11-resource-limits" && "$BIN" config 2>&1) || {
        fail "test_11_resource_limits: config command failed"
        return
    }
    local ok=true
    assert_contains "$output" "cpus" "test_11_resource_limits" || ok=false
    assert_contains "$output" "memory" "test_11_resource_limits" || ok=false
    if $ok; then
        pass "test_11_resource_limits"
    fi
}

test_12_web_database() {
    local output
    output=$(cd "$EXAMPLES/12-web-database" && "$BIN" config 2>&1) || {
        fail "test_12_web_database: config command failed"
        return
    }
    local ok=true
    assert_contains "$output" "db" "test_12_web_database" || ok=false
    assert_contains "$output" "redis" "test_12_web_database" || ok=false
    assert_contains "$output" "api" "test_12_web_database" || ok=false
    assert_contains "$output" "worker" "test_12_web_database" || ok=false
    assert_contains "$output" "nginx" "test_12_web_database" || ok=false
    if $ok; then
        pass "test_12_web_database"
    fi
}

test_13_apptainer_extensions() {
    local output
    output=$(cd "$EXAMPLES/13-apptainer-extensions" && "$BIN" config 2>&1) || {
        fail "test_13_apptainer_extensions: config command failed"
        return
    }
    local ok=true
    assert_contains "$output" "x-apptainer" "test_13_apptainer_extensions" || \
    assert_contains "$output" "apptainer" "test_13_apptainer_extensions" || ok=false
    if $ok; then
        pass "test_13_apptainer_extensions"
    fi
}

test_14_dns_and_networking() {
    local output
    output=$(cd "$EXAMPLES/14-dns-and-networking" && "$BIN" config 2>&1) || {
        fail "test_14_dns_and_networking: config command failed"
        return
    }
    local ok=true
    assert_contains "$output" "dns" "test_14_dns_and_networking" || ok=false
    assert_contains "$output" "hostname" "test_14_dns_and_networking" || ok=false
    assert_contains "$output" "aliases" "test_14_dns_and_networking" || ok=false
    if $ok; then
        pass "test_14_dns_and_networking"
    fi
}

test_15_full_stack_app() {
    # Basic config: verify all default services
    local output
    output=$(cd "$EXAMPLES/15-full-stack-app" && "$BIN" config 2>&1) || {
        fail "test_15_full_stack_app: config command failed"
        return
    }
    local ok=true
    assert_contains "$output" "db" "test_15_full_stack_app" || ok=false
    assert_contains "$output" "redis" "test_15_full_stack_app" || ok=false
    assert_contains "$output" "api" "test_15_full_stack_app" || ok=false
    assert_contains "$output" "worker" "test_15_full_stack_app" || ok=false
    assert_contains "$output" "nginx" "test_15_full_stack_app" || ok=false

    # With --profile debug: verify debug services appear
    local debug_output
    debug_output=$(cd "$EXAMPLES/15-full-stack-app" && "$BIN" --profile debug config 2>&1) || {
        fail "test_15_full_stack_app: config with --profile debug failed"
        return
    }
    assert_contains "$debug_output" "pgadmin" "test_15_full_stack_app (debug profile)" || ok=false
    assert_contains "$debug_output" "redis-commander" "test_15_full_stack_app (debug profile)" || ok=false

    if $ok; then
        pass "test_15_full_stack_app"
    fi
}

test_16_ext3_volumes() {
    local output
    output=$(cd "$EXAMPLES/16-ext3-volumes" && "$BIN" config 2>&1) || {
        fail "test_16_ext3_volumes: config command failed"
        return
    }
    local ok=true
    assert_contains "$output" "appdata" "test_16_ext3_volumes" || ok=false
    assert_contains "$output" "ext3" "test_16_ext3_volumes" || ok=false
    assert_contains "$output" "writer" "test_16_ext3_volumes" || ok=false
    assert_contains "$output" "reader" "test_16_ext3_volumes" || ok=false
    if $ok; then
        pass "test_16_ext3_volumes"
    fi
}

test_version() {
    local output
    output=$("$BIN" version 2>&1) || {
        fail "test_version: version command failed"
        return
    }
    local ok=true
    # Version output should contain some version-like string
    if echo "$output" | grep -qiE '(version|[0-9]+\.[0-9]+)'; then
        pass "test_version"
    else
        fail "test_version: output does not contain version info"
    fi
}

test_help() {
    local output
    output=$("$BIN" --help 2>&1) || {
        fail "test_help: --help command failed"
        return
    }
    local ok=true
    if echo "$output" | grep -qiE '(usage|apptainer-compose|USAGE)'; then
        pass "test_help"
    else
        fail "test_help: output does not contain usage info"
    fi
}

test_pull_and_up() {
    # This test requires apptainer to be installed
    if ! command -v apptainer &>/dev/null; then
        skip "test_pull_and_up: apptainer not found"
        return
    fi

    local compose_file="$EXAMPLES/00-hello-world/apptainer-compose.yml"
    local ok=true

    # Pull
    echo "  Pulling images..."
    if ! "$BIN" -f "$compose_file" pull 2>&1; then
        fail "test_pull_and_up: pull failed"
        return
    fi

    # Up (detached)
    echo "  Starting services..."
    if ! "$BIN" -f "$compose_file" up -d 2>&1; then
        fail "test_pull_and_up: up -d failed"
        ok=false
    fi

    # Ps
    echo "  Checking running services..."
    "$BIN" -f "$compose_file" ps 2>&1 || true

    # Down
    echo "  Stopping services..."
    if ! "$BIN" -f "$compose_file" down 2>&1; then
        fail "test_pull_and_up: down failed"
        ok=false
    fi

    if $ok; then
        pass "test_pull_and_up"
    fi
}

# ---------------------------------------------------------------------------
# Run all tests
# ---------------------------------------------------------------------------

echo -e "${BOLD}Running config parsing tests...${RESET}"
echo ""

run_test test_00_hello_world
run_test test_01_web_server
run_test test_02_multi_service
run_test test_03_environment_variables
run_test test_04_volumes
run_test test_05_depends_on
run_test test_06_healthcheck
run_test test_07_gpu_nvidia
run_test test_08_build_from_dockerfile
run_test test_09_multi_file_override
run_test test_10_profiles
run_test test_11_resource_limits
run_test test_12_web_database
run_test test_13_apptainer_extensions
run_test test_14_dns_and_networking
run_test test_15_full_stack_app
run_test test_16_ext3_volumes

echo -e "${BOLD}Running CLI tests...${RESET}"
echo ""

run_test test_version
run_test test_help

echo -e "${BOLD}Running lifecycle tests...${RESET}"
echo ""

run_test test_pull_and_up

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

TOTAL=$((PASS_COUNT + FAIL_COUNT + SKIP_COUNT))

echo -e "${BOLD}=== Summary ===${RESET}"
echo -e "  Total:   $TOTAL"
echo -e "  ${GREEN}Passed:  $PASS_COUNT${RESET}"
echo -e "  ${RED}Failed:  $FAIL_COUNT${RESET}"
echo -e "  ${YELLOW}Skipped: $SKIP_COUNT${RESET}"
echo ""

if [ "$FAIL_COUNT" -gt 0 ]; then
    echo -e "${RED}${BOLD}FAILED${RESET}"
    exit 1
else
    echo -e "${GREEN}${BOLD}ALL TESTS PASSED${RESET}"
    exit 0
fi
