#!/bin/sh
set -eu

repository="Castrozan/avatar"
version="${AVATAR_VERSION:-latest}"
install_directory="${AVATAR_INSTALL_DIRECTORY:-$HOME/.local/bin}"

fail() {
  printf 'install.sh: %s\n' "$1" >&2
  exit 1
}

detect_target() {
  case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) echo x86_64-unknown-linux-gnu ;;
    Linux-aarch64 | Linux-arm64) echo aarch64-unknown-linux-gnu ;;
    Darwin-x86_64) echo x86_64-apple-darwin ;;
    Darwin-arm64 | Darwin-aarch64) echo aarch64-apple-darwin ;;
    *) fail "unsupported platform $(uname -s) $(uname -m)" ;;
  esac
}

release_base_url() {
  if [ -n "${AVATAR_RELEASE_BASE_URL:-}" ]; then
    echo "$AVATAR_RELEASE_BASE_URL"
  elif [ "$version" = latest ]; then
    echo "https://github.com/$repository/releases/latest/download"
  else
    echo "https://github.com/$repository/releases/download/$version"
  fi
}

data_directory() {
  if [ -n "${AVATAR_DATA_DIRECTORY:-}" ]; then
    echo "$AVATAR_DATA_DIRECTORY"
  elif [ "$(uname -s)" = Darwin ]; then
    echo "$HOME/Library/Application Support/avatar"
  else
    echo "${XDG_DATA_HOME:-$HOME/.local/share}/avatar"
  fi
}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d ' ' -f 1
  else
    shasum -a 256 "$1" | cut -d ' ' -f 1
  fi
}

download() {
  curl --fail --silent --show-error --location --output "$2" "$1" || fail "could not download $1"
}

verify_checksum() {
  expected="$(cut -d ' ' -f 1 "$2")"
  actual="$(sha256_of "$1")"
  [ -n "$expected" ] && [ "$expected" = "$actual" ] || fail "checksum mismatch for $1"
}

install_portraits() {
  portraits="$(data_directory)/renderer/portraits"
  mkdir -p "$portraits"
  cp "$1"/avatars/* "$portraits"/
}

main() {
  command -v curl >/dev/null 2>&1 || fail "curl is required"
  target="$(detect_target)"
  archive="avatar-$target.tar.gz"
  work="$(mktemp -d)"
  trap 'rm -rf "$work"' EXIT
  base="$(release_base_url)"
  download "$base/$archive" "$work/$archive"
  download "$base/$archive.sha256" "$work/$archive.sha256"
  verify_checksum "$work/$archive" "$work/$archive.sha256"
  tar -xzf "$work/$archive" -C "$work"
  mkdir -p "$install_directory"
  install -m 755 "$work/avatar-$target/avatar" "$install_directory/avatar"
  install_portraits "$work/avatar-$target"
  printf 'installed %s/avatar\n' "$install_directory"
  printf 'next: avatar setup\n'
}

main "$@"
