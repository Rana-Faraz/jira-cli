#!/bin/sh
set -eu

REPO="${JIRA_CLI_REPO:-Rana-Faraz/jira-cli}"
BIN_DIR="${JIRA_CLI_INSTALL_DIR:-/usr/local/bin}"
VERSION=""
USE_SUDO="auto"

usage() {
  cat <<'EOF'
Install jira-cli from GitHub Releases.

Usage: install.sh [options]

Options:
  -b, --bin-dir DIR     Install into DIR (default: /usr/local/bin)
  -v, --version TAG     Install a specific tag, for example v0.1.0
  -r, --repo REPO       GitHub repo in owner/name form
      --no-sudo         Never attempt sudo when BIN_DIR is not writable
  -h, --help            Show this help
EOF
}

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "error: required command not found: $1" >&2
    exit 1
  fi
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    -b|--bin-dir)
      BIN_DIR="$2"
      shift 2
      ;;
    -v|--version)
      VERSION="$2"
      shift 2
      ;;
    -r|--repo)
      REPO="$2"
      shift 2
      ;;
    --no-sudo)
      USE_SUDO="false"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

need_cmd curl
need_cmd tar
need_cmd install
need_cmd uname
need_cmd mktemp
need_cmd find
need_cmd sed
need_cmd head

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux)
      case "$arch" in
        x86_64|amd64) echo "x86_64-unknown-linux-gnu" ;;
        *)
          echo "error: unsupported Linux architecture: $arch" >&2
          exit 1
          ;;
      esac
      ;;
    Darwin)
      case "$arch" in
        x86_64) echo "x86_64-apple-darwin" ;;
        arm64|aarch64) echo "aarch64-apple-darwin" ;;
        *)
          echo "error: unsupported macOS architecture: $arch" >&2
          exit 1
          ;;
      esac
      ;;
    *)
      echo "error: unsupported operating system: $os" >&2
      exit 1
      ;;
  esac
}

latest_version() {
  curl -fsSLI -o /dev/null -w '%{url_effective}' "https://github.com/$REPO/releases/latest" \
    | sed 's#.*/tag/##'
}

install_binary() {
  source_bin="$1"

  if [ -w "$BIN_DIR" ] || { [ ! -e "$BIN_DIR" ] && [ -w "$(dirname "$BIN_DIR")" ]; }; then
    install -d "$BIN_DIR"
    install -m 0755 "$source_bin" "$BIN_DIR/jira"
    return
  fi

  if [ "$USE_SUDO" != "false" ] && command -v sudo >/dev/null 2>&1; then
    sudo install -d "$BIN_DIR"
    sudo install -m 0755 "$source_bin" "$BIN_DIR/jira"
    return
  fi

  echo "error: cannot write to $BIN_DIR; rerun with a writable --bin-dir" >&2
  exit 1
}

TARGET="$(detect_target)"

if [ -z "$VERSION" ]; then
  VERSION="$(latest_version)"
fi

if [ -z "$VERSION" ]; then
  echo "error: could not resolve a release version from GitHub" >&2
  exit 1
fi

ASSET="jira-cli-${VERSION}-${TARGET}.tar.gz"
URL="https://github.com/$REPO/releases/download/${VERSION}/${ASSET}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM

echo "Downloading $URL"
curl -fsSL "$URL" -o "$TMP_DIR/$ASSET"
tar -xzf "$TMP_DIR/$ASSET" -C "$TMP_DIR"

SOURCE_BIN="$(find "$TMP_DIR" -type f -name jira | head -n 1)"
if [ -z "$SOURCE_BIN" ]; then
  echo "error: downloaded archive did not contain a jira binary" >&2
  exit 1
fi

install_binary "$SOURCE_BIN"

echo "jira-cli ${VERSION} installed to $BIN_DIR/jira"
