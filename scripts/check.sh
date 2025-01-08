#!/usr/bin/env bash
set -euo pipefail

# Ensure we're in the project root
cd "$(dirname "$0")/.."

echo "==> Setting up environment..."

# Install uv if not already installed
if ! command -v uv &> /dev/null; then
    echo "==> Installing uv..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
fi

# Create and activate a virtual environment if it doesn't exist
if [ ! -d ".venv" ]; then
    echo "==> Creating virtual environment..."
    python3 -m venv .venv
fi
source .venv/bin/activate

echo "==> Installing dependencies..."
# Install dependencies using uv
uv pip install -U pytest maturin

echo "==> Installing pytest from submodule..."
# Install pytest from submodule
cd tests/pytest
uv pip install -e .
cd ../..

echo "==> Building Rust collector..."
# Clean any old builds
rm -rf target/wheels/
# Build our collector with debug output
RUST_BACKTRACE=1 maturin develop -v

echo "==> Installing rytest in development mode..."
uv pip install -e .

echo "==> Running pytest collection tests..."
# Run pytest's collection tests with our collector
PYTHONPATH=tests/pytest pytest tests/pytest/testing/test_collection.py -p rytest.collect -x
