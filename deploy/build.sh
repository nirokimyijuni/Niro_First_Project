#!/usr/bin/env bash
# wasm release 빌드 후 dist/에 웹 파일을 모은다.
# 사용법 (프로젝트 루트 어디서든): bash deploy/build.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

cd "$ROOT/game"
cargo test
cargo build --release --target wasm32-unknown-unknown

# 상대 경로면 cargo처럼 지금 위치(game/) 기준이다. 아래 cp까지 cd 하지 않는다.
WASM="${CARGO_TARGET_DIR:-$ROOT/game/target}/wasm32-unknown-unknown/release/miss-game.wasm"
DIST="$ROOT/dist"

rm -rf "$DIST"
mkdir -p "$DIST"
cp "$ROOT/web/mq_js_bundle.js" "$DIST/"
cp "$WASM" "$DIST/miss-game.wasm"

# 캐시 문제 방지: 브라우저가 옛 .js/.wasm을 계속 쓰지 않도록 dist/index.html 안의 주소에 버전 꼬리를 붙인다.
# 원본 web/index.html은 그대로 두고 dist 쪽만 바꾼다. 버전 = git 짧은 해시 + 빌드 시각(UTC).
GIT_HASH="$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo nogit)"
VERSION="${GIT_HASH}-$(date -u +%Y%m%d%H%M%S)"
sed -e "s|src=\"mq_js_bundle.js\"|src=\"mq_js_bundle.js?v=${VERSION}\"|" \
    -e "s|load(\"miss-game.wasm\")|load(\"miss-game.wasm?v=${VERSION}\")|" \
    -e "s|^\([[:space:]]*\)<script src=\"mq_js_bundle.js|\1<script>window.GAME_VERSION = \"${VERSION}\"; console.log(\"게임 버전: ${VERSION}\");</script>\n\1<script src=\"mq_js_bundle.js|" \
    "$ROOT/web/index.html" > "$DIST/index.html"
# US-031: 같은 버전 값을 window.GAME_VERSION 과 콘솔에도 남긴다 (화면 표시는 로건 담당).
if ! grep -q "mq_js_bundle.js?v=${VERSION}" "$DIST/index.html" \
   || ! grep -q "miss-game.wasm?v=${VERSION}" "$DIST/index.html" \
   || ! grep -q "window.GAME_VERSION = \"${VERSION}\"" "$DIST/index.html"; then
    echo "오류: dist/index.html에 버전 꼬리를 붙이지 못했어요 (web/index.html의 주소 모양이 바뀌었나요?)" >&2
    exit 1
fi
echo "버전 꼬리: ?v=${VERSION}"
# 소개 페이지와 공유 그림 (로건, 2026-11-21). 대표 주소 자리는 index.html처럼 아래에서 채운다.
cp "$ROOT/web/about.html" "$DIST/about.html"
cp "$ROOT/web/og-image.png" "$DIST/og-image.png"
# 아이콘 (로건, 2026-11-21): 탭 아이콘, 홈 화면 아이콘, 앱 설명서(manifest)
cp "$ROOT/web/favicon.ico" "$DIST/favicon.ico"
cp "$ROOT/web/manifest.webmanifest" "$DIST/manifest.webmanifest"
cp -r "$ROOT/web/icons" "$DIST/icons"
# 그림·글꼴·소리: 게임이 "assets/..." 경로로 불러오므로 dist/assets 로 복사한다 (.gitkeep 제외)
cp -r "$ROOT/game/assets" "$DIST/assets"
find "$DIST/assets" -name .gitkeep -delete

# 구글 검색 준비 (부모 결정 2026-11-21): robots.txt와 sitemap.xml을 dist 맨 위에 넣는다.
# 대표 주소는 deploy/seo/site-url.txt 한 곳에만 둔다 (끝에 / 포함). web/의 canonical·og·twitter 주소도
# __SITE_URL__ 자리표시로 두고 여기서 채운다. AWS에도 같은 파일이 올라가지만 주소는 대표 주소만 가리킨다.
# lastmod는 빌드 날짜(UTC).
SITE_URL="$(tr -d '\r\n' < "$ROOT/deploy/seo/site-url.txt")"
case "$SITE_URL" in
    https://*/) ;;
    *) echo "오류: deploy/seo/site-url.txt는 https://로 시작하고 /로 끝나야 해요: $SITE_URL" >&2; exit 1 ;;
esac
for f in index.html about.html; do
    sed -i "s|__SITE_URL__|${SITE_URL}|g" "$DIST/$f"
    if grep -q "__SITE_URL__" "$DIST/$f"; then
        echo "오류: dist/$f에 대표 주소 자리표시가 남았어요" >&2
        exit 1
    fi
done
LASTMOD="$(date -u +%Y-%m-%d)"
for f in robots.txt sitemap.xml; do
    sed -e "s|__SITE_URL__|${SITE_URL}|g" -e "s|__LASTMOD__|${LASTMOD}|g" \
        "$ROOT/deploy/seo/$f" > "$DIST/$f"
    if grep -q "__[A-Z_]*__" "$DIST/$f"; then
        echo "오류: dist/$f에 채우지 못한 자리가 남았어요" >&2
        exit 1
    fi
done
# Search Console 소유 확인용 HTML 파일(google로 시작)이 있으면 그대로 dist 맨 위에 둔다. 비밀 값 아님.
for f in "$ROOT"/deploy/seo/google*.html; do
    if [ -e "$f" ]; then cp "$f" "$DIST/"; fi
done

# 글꼴 줄이기 (부모 결정 2026-11-14): game/src와 web/의 글자 + 안전용 글자만 남긴 글꼴로
# dist 쪽 파일을 같은 이름으로 바꾼다. 원본 game/assets/fonts/Jua-Regular.ttf는 그대로.
# 빠진 글자가 있으면 스크립트가 실패해서 빌드도 멈춘다. 파이썬은 PYTHON으로 고른다.
# (이 PC의 그냥 `python`은 가짜라서, 로컬에서는 진짜 파이썬 경로를 PYTHON에 넣어 준다.)
if [ -z "${PYTHON:-}" ]; then
    if [ -x /c/Users/SDS/AppData/Local/Programs/Python/Python313/python.exe ]; then
        PYTHON=/c/Users/SDS/AppData/Local/Programs/Python/Python313/python.exe
    else
        PYTHON=python3
    fi
fi
# fonttools가 없을 때: 로컬은 원래 글꼴로 계속, CI(GitHub Actions)는 실패 → 배포판은 늘 작은 글꼴.
if "$PYTHON" -c "import fontTools.subset" >/dev/null 2>&1; then
    "$PYTHON" "$ROOT/tools/font-subset/subset_font.py" \
        "$ROOT/game/assets/fonts/Jua-Regular.ttf" \
        "$DIST/assets/fonts/Jua-Regular.ttf" \
        "$ROOT/game/src" "$ROOT/web"
elif [ -n "${CI:-}" ]; then
    echo "오류: fonttools 없음 (CI에서는 글꼴을 꼭 줄여야 해요. pip install fonttools)" >&2
    exit 1
else
    echo "경고: fonttools 없음: 원래 글꼴을 그대로 씀" >&2
fi

SIZE=$(wc -c < "$DIST/miss-game.wasm")
echo "dist 준비 완료: $DIST"
echo "wasm 크기: $SIZE bytes"
if [ "$SIZE" -gt $((5 * 1024 * 1024)) ]; then
    echo "경고: wasm이 5MB를 넘습니다. 모바일에서 느릴 수 있어요." >&2
fi
