#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUT_DIR="${SCRIPT_DIR}/out"

if [ "${1:-}" = "clean" ]; then
    echo "Cleaning build outputs..."
    rm -rf "${OUT_DIR}" "${SCRIPT_DIR}/target"
    echo "Clean complete."
    exit 0
fi

PROFILE="${1:-release}"
case "$PROFILE" in
    debug|release) ;;
    *)
        echo "error: profile must be 'debug' or 'release'" >&2
        exit 2
        ;;
esac

ANDROID_NDK_HOME="${ANDROID_NDK_HOME:-${ANDROID_NDK_ROOT:-/home/rapli/android-ndk-r26b}}"
if [ ! -d "$ANDROID_NDK_HOME" ]; then
    echo "error: Android NDK not found at $ANDROID_NDK_HOME" >&2
    exit 1
fi

NDK_BIN="$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin"
if [ ! -x "$NDK_BIN/aarch64-linux-android26-clang" ]; then
    echo "error: Android NDK clang toolchain not found under $NDK_BIN" >&2
    exit 1
fi

export CC_aarch64_linux_android="$NDK_BIN/aarch64-linux-android26-clang"
export CXX_aarch64_linux_android="$NDK_BIN/aarch64-linux-android26-clang++"
export AR_aarch64_linux_android="$NDK_BIN/llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$CC_aarch64_linux_android"

CARGO_BIN="${CARGO:-cargo}"
if [ -x "$HOME/.cargo/bin/cargo" ]; then
    CARGO_BIN="$HOME/.cargo/bin/cargo"
fi

echo "=== Building OPlus Charger Bridge for Transsion ($PROFILE, Rust aarch64) ==="

cd "$SCRIPT_DIR"

if [ "$PROFILE" = "release" ]; then
    "$CARGO_BIN" build --target aarch64-linux-android --release
    BIN="target/aarch64-linux-android/release/vendor-oplus-hardware-charger-service"
else
    "$CARGO_BIN" build --target aarch64-linux-android
    BIN="target/aarch64-linux-android/debug/vendor-oplus-hardware-charger-service"
fi

if [ "$PROFILE" = "release" ]; then
    echo "==> Stripping binary..."
    "$NDK_BIN/llvm-strip" "$BIN"
fi

mkdir -p "$OUT_DIR"
cp -f "$BIN" "$OUT_DIR/vendor.oplus.hardware.charger-service"
cp -f "$BIN" "$OUT_DIR/vendor.oplus.hardware.charger-V1-service"
cp -f "$BIN" "$OUT_DIR/vendor.oplus.hardware.charger-V11-service"
cp -f "${SCRIPT_DIR}/vendor.oplus.hardware.charger-service.rc" "${OUT_DIR}/vendor.oplus.hardware.charger-service.rc"

# Create ready-to-deploy archive
tar -czf "${OUT_DIR}/charger_package.tar.gz" -C "${OUT_DIR}" \
    vendor.oplus.hardware.charger-service \
    vendor.oplus.hardware.charger-V1-service \
    vendor.oplus.hardware.charger-V11-service \
    vendor.oplus.hardware.charger-service.rc

echo ""
echo "=== Build Succeeded! Outputs in ${OUT_DIR} ==="
ls -lh "${OUT_DIR}"
file "${OUT_DIR}/vendor.oplus.hardware.charger-service"
