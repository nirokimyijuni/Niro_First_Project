#!/usr/bin/env bash
# wasm release 빌드 후 dist/에 웹 파일을 모은다.
# 사용법 (프로젝트 루트 어디서든): bash deploy/build.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"

cd "$ROOT/game"
cargo test
cargo build --release --target wasm32-unknown-unknown

WASM="$ROOT/game/target/wasm32-unknown-unknown/release/miss-game.wasm"
DIST="$ROOT/dist"

rm -rf "$DIST"
mkdir -p "$DIST"
cp "$ROOT/web/index.html" "$ROOT/web/mq_js_bundle.js" "$DIST/"
cp "$WASM" "$DIST/miss-game.wasm"
# 그림·글꼴·소리: 게임이 "assets/..." 경로로 불러오므로 dist/assets 로 복사한다 (.gitkeep 제외)
cp -r "$ROOT/game/assets" "$DIST/assets"
find "$DIST/assets" -name .gitkeep -delete

SIZE=$(wc -c < "$DIST/miss-game.wasm")
echo "dist 준비 완료: $DIST"
echo "wasm 크기: $SIZE bytes"
if [ "$SIZE" -gt $((5 * 1024 * 1024)) ]; then
    echo "경고: wasm이 5MB를 넘습니다. 모바일에서 느릴 수 있어요." >&2
fi
