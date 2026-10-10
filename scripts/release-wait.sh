#!/usr/bin/env bash
#
# Wait until the GitHub Release for a tag exists — and, if patterns are given,
# until it has an asset matching each — then exit 0.
#
#   scripts/release-wait.sh <tag> [asset-pattern...]
#
# release.yml creates the release. The workflows that attach a package to it or
# publish from it start on the same tag push and run in parallel, so they wait
# here instead of racing it — only one workflow ever creates a release, which is
# what keeps a tag from getting two. They stop waiting as soon as the Release
# run for the tag has failed, rather than at the timeout.
#
# Needs gh with GH_TOKEN and GITHUB_REPOSITORY; the job needs `actions: read`.
# RELEASE_WAIT_TIMEOUT (seconds, default 2700) bounds the wait.

set -euo pipefail

tag="${1:?usage: $0 <tag> [asset-pattern...]}"
shift
repo="${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is not set}"
deadline=$(( SECONDS + ${RELEASE_WAIT_TIMEOUT:-2700} ))

has_all_assets() {
    local names="$1" pattern name found
    for pattern in "$@"; do
        found=0
        while IFS= read -r name; do
            # shellcheck disable=SC2053 # the pattern is meant to glob
            [[ -n "$name" && "$name" == $pattern ]] && { found=1; break; }
        done <<< "$names"
        (( found )) || return 1
    done
    return 0
}

while true; do
    if names="$(gh release view "$tag" -R "$repo" --json assets --jq '.assets[].name' 2>/dev/null)"; then
        if has_all_assets "$names" "$@"; then
            echo "Release $tag is published${1:+ with $*}."
            exit 0
        fi
    fi
    conclusion="$(gh run list -R "$repo" --workflow release.yml --branch "$tag" --limit 1 \
                  --json conclusion --jq '.[0].conclusion // ""' 2>/dev/null || true)"
    case "$conclusion" in
        failure|cancelled|timed_out|startup_failure)
            echo "::error::The Release workflow for $tag ended in '$conclusion'; nothing to attach to."
            exit 1 ;;
    esac
    if (( SECONDS >= deadline )); then
        echo "::error::Release $tag${1:+ with $*} did not appear in time."
        exit 1
    fi
    echo "Waiting for release $tag${1:+ with $*}..."
    sleep 20
done
