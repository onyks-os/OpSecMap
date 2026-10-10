#!/usr/bin/env bash
#
# Install a distribution package in a clean container, then run a command
# against what it installed.
#
#   scripts/smoke-test-package.sh <package> <image> <command...>
#
#   scripts/smoke-test-package.sh dist/tool_1.0.0-1_amd64.deb docker.io/library/debian:13 tool --version
#   scripts/smoke-test-package.sh dist/tool-1.0.0-1.x86_64.rpm registry.fedoraproject.org/fedora:44 tool --version
#   scripts/smoke-test-package.sh dist/tool-1.0.0-1-x86_64.pkg.tar.zst docker.io/library/archlinux:latest tool --version
#
# A package that was built but never installed is a package nobody has tested:
# a dependency its distribution does not provide, a binary linked against a newer
# libc, a file installed somewhere the program does not look — each installs or
# runs fine on the build machine and fails on the first user's. The install goes
# through the distribution's own package manager, so declared dependencies are
# resolved exactly as they will be for a user.
#
# Needs podman or docker. Starts no init system and needs no privileges.

set -euo pipefail

package="${1:?usage: $0 <package> <image> <command...>}"
image="${2:?usage: $0 <package> <image> <command...>}"
shift 2
(( $# > 0 )) || { echo "no command to run after installing" >&2; exit 2; }
[ -f "$package" ] || { echo "not a file: $package" >&2; exit 2; }
engine="$(command -v podman || command -v docker || true)"
[ -n "$engine" ] || { echo "podman or docker is required" >&2; exit 2; }

name="$(basename "$package")"
# The install commands run in the container, where $PKG is set; single quotes on purpose.
# shellcheck disable=SC2016
case "$name" in
    *.deb) install='apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq "/pkg/$PKG"' ;;
    *.rpm) install='dnf install -y -q "/pkg/$PKG"' ;;
    *.pkg.tar.zst) install='pacman -Sy --noconfirm --needed >/dev/null && pacman -U --noconfirm "/pkg/$PKG"' ;;
    *) echo "do not know how to install $name" >&2; exit 2 ;;
esac

echo "==> Installing $name in $image"
# The command is passed as positional arguments, never spliced into the script.
# shellcheck disable=SC2016 # expanded by the container's shell
"$engine" run --rm -v "$(cd "$(dirname "$package")" && pwd):/pkg:ro,z" -e PKG="$name" "$image" \
    sh -c "set -e; $install; echo '==> Running:' \"\$@\"; \"\$@\"" sh "$@"
echo "==> $name installs and runs on $image"
