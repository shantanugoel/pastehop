#!/bin/sh

set -eu

REPO="${REPO:-shantanugoel/pastehop}"
BIN_NAME="${BIN_NAME:-ph}"
VERSION="${VERSION:-latest}"

log() {
  printf '%s\n' "$*"
}

fail() {
  printf 'Error: %s\n' "$*" >&2
  exit 1
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command: $1"
}

download_to() {
  url="$1"
  dest="$2"

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$dest"
    return
  fi

  if command -v wget >/dev/null 2>&1; then
    wget -qO "$dest" "$url"
    return
  fi

  fail "missing required command: curl or wget"
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64|amd64) printf 'x86_64-unknown-linux-gnu\n' ;;
        aarch64|arm64) printf 'aarch64-unknown-linux-gnu\n' ;;
        *) fail "unsupported Linux architecture: $arch" ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64|amd64) printf 'x86_64-apple-darwin\n' ;;
        arm64|aarch64) printf 'aarch64-apple-darwin\n' ;;
        *) fail "unsupported macOS architecture: $arch" ;;
      esac
      ;;
    *)
      fail "unsupported operating system: $os"
      ;;
  esac
}

resolve_bin_dir() {
  if [ -n "${BIN_DIR:-}" ]; then
    printf '%s\n' "$BIN_DIR"
    return
  fi

  if [ "$(id -u)" -eq 0 ]; then
    printf '/usr/local/bin\n'
    return
  fi

  printf '%s/.local/bin\n' "$HOME"
}

main() {
  need_cmd uname
  need_cmd tar
  need_cmd mktemp
  need_cmd install

  target="$(detect_target)"
  archive_name="${BIN_NAME}-${target}.tar.gz"

  if [ "$VERSION" = "latest" ]; then
    download_url="https://github.com/${REPO}/releases/latest/download/${archive_name}"
  else
    release_tag="$VERSION"
    case "$release_tag" in
      v*) ;;
      *) release_tag="v${release_tag}" ;;
    esac

    download_url="https://github.com/${REPO}/releases/download/${release_tag}/${archive_name}"
  fi

  bin_dir="$(resolve_bin_dir)"
  tmp_dir="$(mktemp -d)"
  trap 'rm -rf "$tmp_dir"' EXIT INT HUP TERM

  mkdir -p "$bin_dir"

  archive_path="${tmp_dir}/${archive_name}"
  log "Downloading ${download_url}"
  download_to "$download_url" "$archive_path"

  tar -xzf "$archive_path" -C "$tmp_dir"
  install -m 755 "${tmp_dir}/${BIN_NAME}" "${bin_dir}/${BIN_NAME}"

  log "Installed ${BIN_NAME} to ${bin_dir}/${BIN_NAME}"

  case ":$PATH:" in
    *":${bin_dir}:"*) ;;
    *)
      log "Add ${bin_dir} to your PATH if it is not already available there."
      ;;
  esac
}

main "$@"
