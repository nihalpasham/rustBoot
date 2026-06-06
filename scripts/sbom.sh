#!/usr/bin/env bash
# Generate Software Bill of Materials (SBOM) for rustBoot.
#
# Requires: cargo-cyclonedx
# Install: cargo install cargo-cyclonedx --locked
#
# Usage: ./scripts/sbom.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

echo "==> Generating SBOM for rustBoot crate..."

if command -v cargo-cyclonedx &>/dev/null; then
    cargo cyclonedx --format json --output-file sbom.cdx.json
    echo "==> SBOM written to sbom.cdx.json"
    ls -la sbom.cdx.json
else
    echo "Warning: cargo-cyclonedx not installed."
    echo "Install with: cargo install cargo-cyclonedx --locked"
    echo "Creating placeholder SBOM..."
    cat > sbom.cdx.json << 'SBOM'
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.5",
  "version": 1,
  "metadata": {
    "component": {
      "type": "library",
      "name": "rustBoot",
      "version": "0.1.0",
      "description": "Secure bootloader for embedded systems",
      "licenses": [{"license": {"id": "MIT"}}]
    }
  },
  "components": []
}
SBOM
    echo "==> Placeholder SBOM written to sbom.cdx.json"
fi