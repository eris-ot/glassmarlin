#!/usr/bin/env bash
# Build a relocatable runtime bundle for the host platform:
#
#   bundle/<TARGET_OS>/bundle.tar.zst
#
# Contents:
#   python/         — relocatable CPython 3.12 (python-build-standalone)
#   marlinspike/    — the marlinspike source tree (a clone or copy)
#
# The Tauri binary includes this archive via include_bytes! at compile time
# and extracts it to the user-data dir on first launch.
#
# Requires:
#   * uv (https://docs.astral.sh/uv/)
#   * curl
#   * tar with zstd support (or system zstd CLI)
#
# Env vars:
#   MARLINSPIKE_SRC      — path to a local checkout of eris-ot/marlinspike.
#                          Defaults to ../marlinspike.
#   MARLINSPIKE_DPI_SRC  — path to a local checkout of eris-ot/marlinspike-dpi.
#                          Defaults to ../marlinspike-dpi.
#                          Pass MARLINSPIKE_DPI_SRC=skip to omit the DPI engine
#                          (the Python engine will fall back to its built-in
#                          parser path — degraded but not broken).
#   PYTHON_VERSION       — defaults to 3.12.7.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MARLINSPIKE_SRC="${MARLINSPIKE_SRC:-${ROOT}/../marlinspike}"
MARLINSPIKE_DPI_SRC="${MARLINSPIKE_DPI_SRC:-${ROOT}/../marlinspike-dpi}"
PYTHON_VERSION="${PYTHON_VERSION:-3.12.7}"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
  Darwin)
    TARGET_OS="macos"
    case "${ARCH}" in
      arm64)   PBS_TAG="cpython-${PYTHON_VERSION}+20241016-aarch64-apple-darwin-install_only.tar.gz" ;;
      x86_64)  PBS_TAG="cpython-${PYTHON_VERSION}+20241016-x86_64-apple-darwin-install_only.tar.gz" ;;
      *) echo "unsupported macOS arch: ${ARCH}" >&2; exit 1 ;;
    esac
    ;;
  Linux)
    TARGET_OS="linux"
    case "${ARCH}" in
      x86_64)  PBS_TAG="cpython-${PYTHON_VERSION}+20241016-x86_64-unknown-linux-gnu-install_only.tar.gz" ;;
      aarch64) PBS_TAG="cpython-${PYTHON_VERSION}+20241016-aarch64-unknown-linux-gnu-install_only.tar.gz" ;;
      *) echo "unsupported Linux arch: ${ARCH}" >&2; exit 1 ;;
    esac
    ;;
  MINGW*|MSYS*|CYGWIN*)
    TARGET_OS="windows"
    PBS_TAG="cpython-${PYTHON_VERSION}+20241016-x86_64-pc-windows-msvc-install_only.tar.gz"
    ;;
  *)
    echo "unsupported OS: ${OS}" >&2
    exit 1
    ;;
esac

PBS_URL="https://github.com/astral-sh/python-build-standalone/releases/download/20241016/${PBS_TAG}"
BUNDLE_DIR="${ROOT}/bundle/${TARGET_OS}"
WORK_DIR="${BUNDLE_DIR}/build"

if [[ ! -d "${MARLINSPIKE_SRC}" ]]; then
  echo "MARLINSPIKE_SRC=${MARLINSPIKE_SRC} not found" >&2
  echo "Set MARLINSPIKE_SRC to the path of your eris-ot/marlinspike checkout." >&2
  exit 1
fi

if ! command -v uv >/dev/null 2>&1; then
  echo "uv not found — install from https://docs.astral.sh/uv/ first" >&2
  exit 1
fi

echo "[1/5] Cleaning ${BUNDLE_DIR}"
rm -rf "${BUNDLE_DIR}"
mkdir -p "${WORK_DIR}"

echo "[2/5] Fetching python-build-standalone (${PBS_TAG})"
curl -fSL "${PBS_URL}" -o "${WORK_DIR}/python.tar.gz"
tar -xzf "${WORK_DIR}/python.tar.gz" -C "${WORK_DIR}"
mv "${WORK_DIR}/python" "${WORK_DIR}/python.runtime"
mv "${WORK_DIR}/python.runtime" "${BUNDLE_DIR}/python"

echo "[3/5] Installing marlinspike + deps into bundled site-packages"
case "${TARGET_OS}" in
  windows)
    BUNDLED_PY="${BUNDLE_DIR}/python/python.exe"
    SITE_PACKAGES="${BUNDLE_DIR}/python/Lib/site-packages"
    ;;
  *)
    BUNDLED_PY="${BUNDLE_DIR}/python/bin/python3"
    SITE_PACKAGES="${BUNDLE_DIR}/python/lib/python3.12/site-packages"
    ;;
esac

# Use the bundled interpreter so binary wheels match its ABI.
uv pip install --python "${BUNDLED_PY}" --target "${SITE_PACKAGES}" \
  "${MARLINSPIKE_SRC}" \
  flask flask-sqlalchemy flask-limiter psycopg2-binary PyYAML zstandard \
  flask-migrate alembic

echo "[4/5] Copying marlinspike asset directories"
DEST="${BUNDLE_DIR}/marlinspike"
mkdir -p "${DEST}"
# Source tree the runtime needs at MARLINSPIKE_PROJECT_ROOT:
#   - rules/, presets/, data/oui.json, migrations/, plugins/
# The package itself is installed into site-packages above.
for path in rules presets plugins migrations; do
  if [[ -d "${MARLINSPIKE_SRC}/${path}" ]]; then
    cp -R "${MARLINSPIKE_SRC}/${path}" "${DEST}/${path}"
  fi
done
mkdir -p "${DEST}/data"
if [[ -f "${MARLINSPIKE_SRC}/data/oui.json" ]]; then
  cp "${MARLINSPIKE_SRC}/data/oui.json" "${DEST}/data/oui.json"
fi

echo "[5/6] Building marlinspike-dpi (Rust DPI engine — drops the tshark/libpcap dependency)"
if [[ "${MARLINSPIKE_DPI_SRC}" == "skip" ]]; then
  echo "  Skipping per MARLINSPIKE_DPI_SRC=skip — engine will fall back to built-in parser"
elif [[ ! -d "${MARLINSPIKE_DPI_SRC}" ]]; then
  echo "  MARLINSPIKE_DPI_SRC=${MARLINSPIKE_DPI_SRC} not found" >&2
  echo "  Set MARLINSPIKE_DPI_SRC to the path of your eris-ot/marlinspike-dpi checkout," >&2
  echo "  or set MARLINSPIKE_DPI_SRC=skip to build without it (degraded engine path)." >&2
  exit 1
elif ! command -v cargo >/dev/null 2>&1; then
  echo "  cargo not found — install Rust toolchain from https://rustup.rs/ first" >&2
  exit 1
else
  case "${TARGET_OS}" in
    windows) DPI_BIN_NAME="marlinspike-dpi.exe" ;;
    *)       DPI_BIN_NAME="marlinspike-dpi" ;;
  esac
  (
    cd "${MARLINSPIKE_DPI_SRC}"
    cargo build --release --bin marlinspike-dpi
  )
  DPI_DEST="${DEST}/dpi"
  mkdir -p "${DPI_DEST}"
  cp "${MARLINSPIKE_DPI_SRC}/target/release/${DPI_BIN_NAME}" "${DPI_DEST}/${DPI_BIN_NAME}"
  ls -lh "${DPI_DEST}/${DPI_BIN_NAME}"
fi

echo "[6/6] Packing bundle.tar.zst"
cd "${BUNDLE_DIR}"
rm -rf build
tar --use-compress-program="zstd -19 --long=27 -T0" -cf bundle.tar.zst python marlinspike
ls -lh bundle.tar.zst

echo ""
echo "Bundle ready: ${BUNDLE_DIR}/bundle.tar.zst"
echo "Next step: cargo build --release  (build.rs reads GLASSMARLIN_TARGET_OS_STAMP=${TARGET_OS})"
