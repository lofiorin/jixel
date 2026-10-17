#!/usr/bin/env bash
# Prepares a fresh clone for building.
#
#   scripts/bootstrap.sh
#
# 1. Checks out libjxl (a git submodule) and only the three third-party
#    libraries it needs: brotli, highway and skcms. libjxl has many more
#    submodules (test data, benchmarks…) that we do not need.
# 2. Checks that the required tools are installed.
# 3. Fetches the Flutter packages and generates the translations.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

say() { printf '\033[1m==> %s\033[0m\n' "$*"; }
warn() { printf '\033[33mwarning:\033[0m %s\n' "$*"; }

say "Fetching libjxl"
git submodule update --init --depth 1 core/crates/jixel-libjxl-sys/libjxl
(
  cd core/crates/jixel-libjxl-sys/libjxl
  git submodule update --init --depth 1 third_party/brotli third_party/highway third_party/skcms
)

say "Checking tools"
missing=0
for tool in cargo cmake; do
  if ! command -v "$tool" >/dev/null; then
    warn "$tool not found (see docs/en/building.md)"
    missing=1
  fi
done
command -v rustup >/dev/null || warn "rustup not found: the Flutter app build needs it to add mobile targets"
command -v flutter >/dev/null || warn "flutter not found: only the Rust core and CLI can be built"

if command -v flutter >/dev/null; then
  say "Preparing the Flutter app"
  (cd app && python3 tool/strings.py && flutter pub get && flutter gen-l10n && dart format --line-length 120 lib/l10n)
fi

if [ "$missing" -eq 0 ]; then
  say "Done. Try:  cargo run --release --manifest-path core/Cargo.toml -p jixel-cli -- --help"
fi
