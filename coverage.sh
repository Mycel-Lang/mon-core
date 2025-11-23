#!/usr/bin/env bash
set -euo pipefail

# Clean old coverage files
rm -f coverage-*.profraw .coverage
rm -rf coverage_html

# Run tests with source-based coverage
echo "[*] Running tests with coverage instrumentation..."
RUSTFLAGS="-C instrument-coverage" \
LLVM_PROFILE_FILE="coverage-%p-%m.profraw" \
cargo test

# Merge raw profiling data
echo "[*] Merging raw coverage data..."
llvm-profdata merge -sparse coverage-*.profraw -o .coverage

# Find ALL test binaries
echo "[*] Finding test binaries..."
BINARIES=""
for file in target/debug/deps/mon_core-*; do
    if [ -f "$file" ] && [ -x "$file" ] && [[ ! "$file" =~ \.d$ ]]; then
        BINARIES="$BINARIES --object $file"
    fi
done

echo "[*] Using binaries:$BINARIES"

# Generate HTML report
echo "[*] Generating HTML coverage report..."
llvm-cov show \
    $BINARIES \
    --instr-profile=".coverage" \
    --show-line-counts-or-regions \
    --format=html \
    --ignore-filename-regex='\.cargo' \
    --ignore-filename-regex='\.rustup' \
    --output-dir=coverage_html

# Generate summary report
echo ""
echo "[*] Coverage Summary (mon-core only):"
llvm-cov report \
    $BINARIES \
    --instr-profile=".coverage" \
    --ignore-filename-regex='\.cargo' \
    --ignore-filename-regex='\.rustup'

echo ""
echo "[*] Done! Open coverage_html/index.html in a browser to view the detailed report."
