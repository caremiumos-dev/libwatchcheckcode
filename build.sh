#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"

NDK="${ANDROID_NDK_HOME:-${ANDROID_NDK:-/opt/android-ndk-r21}}"
if [ ! -x "$NDK/ndk-build" ]; then
    echo "error: NDK not found under $NDK (set ANDROID_NDK_HOME)" >&2
    exit 1
fi

HOST_TAG="linux-x86_64"
BIN="$NDK/toolchains/llvm/prebuilt/$HOST_TAG/bin"
API="${ANDROID_API:-24}"
TARGETS="${ANDROID_TARGETS:-armv7-linux-androideabi}"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/opencode/watchcheckcode_cargo}"

abi_of() {
    case "$1" in
        armv7-linux-androideabi) echo "armeabi-v7a" ;;
        aarch64-linux-android)   echo "arm64-v8a" ;;
        i686-linux-android)      echo "x86" ;;
        x86_64-linux-android)    echo "x86_64" ;;
        *) echo "unknown" ;;
    esac
}

cc_of() {
    case "$1" in
        armv7-linux-androideabi) echo "armv7a-linux-androideabi" ;;
        aarch64-linux-android)   echo "aarch64-linux-android" ;;
        i686-linux-android)      echo "i686-linux-android" ;;
        x86_64-linux-android)    echo "x86_64-linux-android" ;;
        *) echo "" ;;
    esac
}

for target in $TARGETS; do
    abi="$(abi_of "$target")"
    cc="$(cc_of "$target")"
    if [ "$abi" = "unknown" ] || [ -z "$cc" ]; then
        echo "error: unsupported target $target" >&2
        exit 1
    fi

    var="CARGO_TARGET_$(echo "$target" | tr 'a-z-' 'A-Z_')_LINKER"
    export "$var=$BIN/${cc}${API}-clang"

    flags_var="CARGO_TARGET_$(echo "$target" | tr 'a-z-' 'A-Z_')_RUSTFLAGS"
    export "$flags_var=-C link-arg=-fuse-ld=lld"

    cargo build --release --target "$target" --manifest-path "$PROJECT_DIR/Cargo.toml"

    mkdir -p "$PROJECT_DIR/prebuilt/$abi"
    cp "$CARGO_TARGET_DIR/$target/release/libwatchcheckcode.so" \
        "$PROJECT_DIR/prebuilt/$abi/libwatchcheckcode.so"
done

echo "built: $PROJECT_DIR/prebuilt/"
