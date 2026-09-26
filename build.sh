#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

NDK="${ANDROID_NDK_HOME:-${ANDROID_NDK:-/opt/android-ndk-r21}}"
if [ ! -f "$NDK/build/cmake/android.toolchain.cmake" ]; then
    echo "error: NDK CMake toolchain not found under $NDK (set ANDROID_NDK_HOME)" >&2
    exit 1
fi

ABIS="${ANDROID_ABIS:-armeabi-v7a}"
API="${ANDROID_API:-24}"
BUILD_ROOT="${BUILD_ROOT:-/tmp/opencode/watchcheckcode_build}"

rm -rf "$BUILD_ROOT"

for abi in $ABIS; do
    cmake -S "$PROJECT_DIR" -B "$BUILD_ROOT/$abi" \
        -G Ninja \
        -DCMAKE_TOOLCHAIN_FILE="$NDK/build/cmake/android.toolchain.cmake" \
        -DANDROID_ABI="$abi" \
        -DANDROID_PLATFORM="android-$API" \
        -DCMAKE_BUILD_TYPE=Release
    cmake --build "$BUILD_ROOT/$abi"
done

mkdir -p "$PROJECT_DIR/prebuilt"
for abi in $ABIS; do
    mkdir -p "$PROJECT_DIR/prebuilt/$abi"
    find "$BUILD_ROOT/$abi" -name "libwatchcheckcode.so" \
        -exec cp {} "$PROJECT_DIR/prebuilt/$abi/" \;
done

echo "built: $PROJECT_DIR/prebuilt/"
