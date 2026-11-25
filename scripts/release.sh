#!/usr/bin/env bash
# Package update procedure (see readme.md):
#   0. cargo test
#   1. update workspace version
#   2. update scal-core version
#   3. update flake.nix version
#   4. git commit and push
#   5. cargo publish: core, runtime, ipc
#
# Usage: scripts/release.sh <new-version> [options]
#   --macros <ver>   also bump scal-ipc-macros to <ver> and publish it first
#   --skip-tests     skip step 0
#   --no-nix         don't wrap cargo in `nix develop` (use your current env)
#   --no-publish     stop after step 4
#   --dry-run        only print what would change (no edits, no commit/publish)
#   -y, --yes        don't ask for confirmation before commit/push/publish
set -euo pipefail

cd "$(dirname "$0")/.."

usage() { sed -n '2,/^set -/p' "$0" | sed '$d' | sed 's/^# \{0,1\}//'; exit "${1:-1}"; }

NEW=""
MACROS=""
SKIP_TESTS=0
NO_PUBLISH=0
DRY=0
YES=0
USE_NIX=1

while [[ $# -gt 0 ]]; do
  case "$1" in
    --macros) MACROS="${2:?--macros needs a version}"; shift 2 ;;
    --skip-tests) SKIP_TESTS=1; shift ;;
    --no-nix) USE_NIX=0; shift ;;
    --no-publish) NO_PUBLISH=1; shift ;;
    --dry-run) DRY=1; shift ;;
    -y|--yes) YES=1; shift ;;
    -h|--help) usage 0 ;;
    -*) echo "unknown option: $1" >&2; usage ;;
    *) [[ -z "$NEW" ]] || { echo "version given twice" >&2; usage; }; NEW="$1"; shift ;;
  esac
done

[[ -n "$NEW" ]] || usage

SEMVER='^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$'
[[ "$NEW" =~ $SEMVER ]] || { echo "not a valid version: $NEW" >&2; exit 1; }
if [[ -n "$MACROS" ]]; then
  [[ "$MACROS" =~ $SEMVER ]] || { echo "not a valid version: $MACROS" >&2; exit 1; }
fi

# cargo needs the system libs (alsa, ffmpeg, libclang...) from the flake's dev
# shell, both for tests and for the build `cargo publish` does to verify.
# Skipped when already inside a nix shell or when nix is unavailable.
if [[ $USE_NIX -eq 1 && -z "${IN_NIX_SHELL:-}" ]] && command -v nix >/dev/null; then
  CARGO=(nix develop --command cargo)
else
  CARGO=(cargo)
fi

run() { echo "+ $*"; if [[ $DRY -eq 0 ]]; then "$@"; fi; }

confirm() {
  [[ $YES -eq 1 || $DRY -eq 1 ]] && return 0
  read -r -p "$1 [y/N] " ans
  [[ "$ans" == [yY]* ]] || { echo "aborted"; exit 1; }
}

CURRENT=$(sed -n '/^\[workspace.package\]/,/^\[/{s/^version *= *"\(.*\)"/\1/p}' Cargo.toml)
echo "workspace version: $CURRENT -> $NEW"
[[ -n "$MACROS" ]] && echo "scal-ipc-macros:   -> $MACROS"

if [[ $DRY -eq 0 && -n "$(git status --porcelain)" ]]; then
  echo "working tree is not clean; commit or stash first (cargo publish needs a clean tree)" >&2
  exit 1
fi

# 0. tests
if [[ $SKIP_TESTS -eq 0 ]]; then
  run "${CARGO[@]}" test --workspace
fi

# 1. workspace version
run sed -i '/^\[workspace.package\]/,/^\[/s/^version *= *".*"/version = "'"$NEW"'"/' Cargo.toml

# 2. scal-core version (workspace dependency requirement)
run sed -i 's|^\(scal-core *= *{.*version *= *\)"[^"]*"|\1"'"$NEW"'"|' Cargo.toml

if [[ -n "$MACROS" ]]; then
  run sed -i 's|^\(scal-ipc-macros *= *{.*version *= *\)"[^"]*"|\1"'"$MACROS"'"|' Cargo.toml
  run sed -i '0,/^version *= *".*"/s//version = "'"$MACROS"'"/' scal-ipc-macros/Cargo.toml
fi

# 3. flake.nix (the scal-runtime package version)
run sed -i '/pname = "scal-runtime";/,/version/s/version = "[^"]*";/version = "'"$NEW"'";/' flake.nix

# keep Cargo.lock in sync with the new versions (flake uses it)
run "${CARGO[@]}" update --workspace

if [[ $DRY -eq 0 ]]; then
  echo
  git --no-pager diff --stat
  # sanity: every touched version must now be the new one
  grep -q "^version = \"$NEW\"" Cargo.toml || { echo "Cargo.toml version update failed" >&2; exit 1; }
  grep -q "version = \"$NEW\";" flake.nix || { echo "flake.nix version update failed" >&2; exit 1; }
  grep -qE "^scal-core *=.*version *= *\"$NEW\"" Cargo.toml || { echo "scal-core requirement update failed" >&2; exit 1; }
fi

# 4. commit and push
confirm "Commit and push v$NEW?"
run git add Cargo.toml Cargo.lock flake.nix scal-ipc-macros/Cargo.toml
run git commit -m "version $NEW"
run git push

# 5. publish
if [[ $NO_PUBLISH -eq 1 ]]; then
  echo "skipping publish (--no-publish)"
  exit 0
fi
confirm "cargo publish core, runtime, ipc?"
if [[ -n "$MACROS" ]]; then
  run "${CARGO[@]}" publish -p scal-ipc-macros
  [[ $DRY -eq 0 ]] && sleep 15   # let the crates.io index catch up
fi
run "${CARGO[@]}" publish -p scal-core
[[ $DRY -eq 0 ]] && sleep 15
run "${CARGO[@]}" publish -p scal-runtime
run "${CARGO[@]}" publish -p scal-ipc

echo "released $NEW"
