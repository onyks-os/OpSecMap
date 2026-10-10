#!/usr/bin/env bash
#
# Download release assets and verify each against its Sigstore bundle, bound to
# release.yml at the tag, before anything else touches them.
#
#   scripts/release-fetch.sh <tag> <dest-dir> <asset-pattern>...
#
# The publishing workflows upload these exact bytes to a package registry. They
# do not rebuild: a rebuild would put an artifact on the registry that is not
# the one the release signature attests to. The bundles are left in
# <dest-dir>/.sigstore/ so that the directory holds only the artifacts.
#
# Needs gh (GH_TOKEN, GITHUB_REPOSITORY) and the sigstore CLI.

set -euo pipefail

tag="${1:?usage: $0 <tag> <dest-dir> <asset-pattern>...}"
dest="${2:?usage: $0 <tag> <dest-dir> <asset-pattern>...}"
shift 2
(( $# > 0 )) || { echo "no asset patterns given" >&2; exit 2; }
repo="${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is not set}"
identity="https://github.com/${repo}/.github/workflows/release.yml@refs/tags/${tag}"

mkdir -p "$dest/.sigstore"
args=()
for pattern in "$@"; do
    args+=(-p "$pattern" -p "${pattern}.sigstore.json")
done
gh release download "$tag" -R "$repo" -D "$dest" --clobber "${args[@]}"

shopt -s nullglob
count=0
for f in "$dest"/*; do
    [ -f "$f" ] || continue
    case "$f" in *.sigstore.json) continue ;; esac
    [ -f "$f.sigstore.json" ] || { echo "::error::$(basename "$f") has no Sigstore bundle"; exit 1; }
    sigstore verify identity --cert-identity "$identity" \
        --cert-oidc-issuer https://token.actions.githubusercontent.com \
        --bundle "$f.sigstore.json" "$f"
    mv "$f.sigstore.json" "$dest/.sigstore/"
    echo "verified $(basename "$f")"
    count=$((count + 1))
done
(( count > 0 )) || { echo "::error::No release asset matched: $*"; exit 1; }
