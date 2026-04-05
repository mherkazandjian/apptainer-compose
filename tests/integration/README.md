# Integration Tests

Integration tests for apptainer-compose. These tests validate YAML parsing,
config merging, profile filtering, and the full pull/up/down lifecycle against
all 16 example compose files.

## Requirements

- **Docker** on the host (all tests run inside containers)
- No other dependencies needed; the Makefile handles everything

## Running

Build the binary and run integration tests in a privileged apptainer container:

```bash
make test-integration
```

Or run both unit and integration tests together:

```bash
make test integration=1
```

## What the tests cover

| Category | Tests | Description |
|---|---|---|
| Config parsing | `test_00` through `test_15` | Runs `apptainer-compose config` against each example and verifies expected service names, fields, and values appear in the output |
| Multi-file merge | `test_09` | Validates that `-f base.yml -f override.yml` correctly merges override values |
| Profile filtering | `test_10`, `test_15` | Verifies `--profile` includes/excludes the right services |
| CLI basics | `test_version`, `test_help` | Ensures `version` and `--help` produce expected output |
| Lifecycle | `test_pull_and_up` | Pulls an image, starts a service, checks status, and tears down (requires Apptainer; skipped if unavailable) |

## Architecture

Tests run inside the `kaczmarj/apptainer` Docker image with `--privileged` so
that Apptainer can function. The project is bind-mounted at `/workspace` and the
pre-built static binary is invoked directly.

Config-based tests are fast (no Apptainer needed) and validate the YAML parsing
and interpolation logic. The lifecycle test exercises the actual Apptainer
driver.
