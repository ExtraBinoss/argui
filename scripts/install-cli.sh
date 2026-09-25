#!/bin/sh
# Install the latest verified Argui CLI release on Linux or macOS.
set -eu

repo='https://github.com/ExtraBinoss/argui'
for tool in curl tar mktemp grep awk install; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    printf 'argui installer: %s is required.\n' "$tool" >&2
    exit 1
  fi
done

case "$(uname -s)" in
  Linux) system='unknown-linux-gnu' ;;
  Darwin) system='apple-darwin' ;;
  *) printf 'argui installer: this script supports Linux and macOS. Windows: see %s#install-the-cli\n' "$repo" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  x86_64|amd64) architecture='x86_64' ;;
  aarch64|arm64) architecture='aarch64' ;;
  *) printf 'argui installer: unsupported CPU architecture: %s\n' "$(uname -m)" >&2; exit 1 ;;
esac
target="$architecture-$system"

if [ -n "${ARGUI_VERSION:-}" ]; then
  version="$ARGUI_VERSION"
else
  latest=$(curl -fsSL --retry 3 -o /dev/null -w '%{url_effective}' "$repo/releases/latest") || {
    printf 'argui installer: could not find the latest release.\n' >&2
    exit 1
  }
  version=${latest##*/}
fi
if ! printf '%s\n' "$version" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$'; then
  printf 'argui installer: invalid release version: %s\n' "$version" >&2
  exit 1
fi

asset="argui-cli-$version-$target.tar.gz"
url="$repo/releases/download/$version/$asset"
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT HUP INT TERM
if ! curl -fsSL --retry 3 "$url" -o "$temporary/$asset"; then
  printf 'argui installer: no CLI archive for %s on %s. See %s/releases\n' "$version" "$target" "$repo" >&2
  exit 1
fi
curl -fsSL --retry 3 "$url.sha256" -o "$temporary/$asset.sha256" || {
  printf 'argui installer: checksum is missing for %s.\n' "$asset" >&2
  exit 1
}
expected=$(awk '{print $1}' "$temporary/$asset.sha256")
case "$expected" in
  *[!0-9a-f]*|'') printf 'argui installer: invalid SHA-256 file.\n' >&2; exit 1 ;;
esac
if [ "${#expected}" -ne 64 ]; then
  printf 'argui installer: invalid SHA-256 length.\n' >&2
  exit 1
fi
if command -v sha256sum >/dev/null 2>&1; then
  actual=$(sha256sum "$temporary/$asset" | awk '{print $1}')
elif command -v shasum >/dev/null 2>&1; then
  actual=$(shasum -a 256 "$temporary/$asset" | awk '{print $1}')
else
  printf 'argui installer: SHA-256 tool required (sha256sum or shasum).\n' >&2
  exit 1
fi
if [ "$actual" != "$expected" ]; then
  printf 'argui installer: checksum mismatch; installation stopped.\n' >&2
  exit 1
fi

tar -xzf "$temporary/$asset" -C "$temporary" argui
destination=${ARGUI_INSTALL_DIR:-"$HOME/.local/bin"}
mkdir -p "$destination"
install -m 755 "$temporary/argui" "$destination/argui"
printf 'Installed Argui CLI %s at %s/argui\n' "$version" "$destination"
case ":$PATH:" in
  *":$destination:"*) ;;
  *) printf 'Add %s to PATH to use `argui` from any directory.\n' "$destination" ;;
esac
