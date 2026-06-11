# Release Information

## NPM Distribution Layout

This project uses a split-package strategy for npm distribution:

*   **Platform-specific binary packages**: Published under the `@directree` scope (e.g., `@directree/darwin-arm64`, `@directree/linux-x64`). These contain only the compiled native binaries for the respective operating system and CPU architecture.
*   **User-facing wrapper package**: Published as `contextree`. This package resolves the host environment at runtime and spawns the correct platform-specific binary.

## Release Fixes

### Version 1.0.1 / 0.2.1

*   **Executable Permissions**: Fixed an issue where the installed native binary did not have executable permissions (causing `EACCES` errors).
    *   Added `chmod +x` during the GitHub Actions build steps.
    *   Added runtime self-healing permission checks in the wrapper JS shim (`npm/directree/bin/directree.js`) to ensure the binary is marked executable before execution.
