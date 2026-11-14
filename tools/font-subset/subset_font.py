"""한글 글꼴을 게임이 쓰는 글자만 남긴 작은 글꼴로 만든다 (부모 결정 2026-11-14).

사용법: python subset_font.py <원본.ttf> <결과.ttf> <글자를 모을 폴더나 파일>...
  예) python tools/font-subset/subset_font.py game/assets/fonts/Jua-Regular.ttf \
          dist/assets/fonts/Jua-Regular.ttf game/src web

- 모으는 글자: 주어진 폴더의 .rs/.html/.js/.css/.toml/.md 파일의 모든 글자 + 안전용 글자
  (ASCII 전체, 기본 문장부호·기호).
- 검사: 원본 글꼴에 있는 글자가 결과 글꼴에서 빠지면 그 글자를 보여 주고 실패(종료 코드 1).
  원본 글꼴에 처음부터 없는 글자(예: ①, →, 이모지)는 원래도 네모라서 경고만 한다.
- 필요한 것: fonttools (pip install fonttools)
"""
import os
import sys

from fontTools import subset
from fontTools.ttLib import TTFont

TEXT_EXT = {".rs", ".html", ".js", ".css", ".toml", ".md", ".txt"}

# 안전용 글자: ASCII 전체(퀴즈 이름·영어 퀴즈 대비) + 자주 쓰는 문장부호·기호
SAFETY = "".join(chr(c) for c in range(0x20, 0x7F)) + "·…“”‘’「」『』×÷±°%~—–、。！？：；（）［］"


def collect_chars(paths):
    chars = set()
    for p in paths:
        files = []
        if os.path.isdir(p):
            for root, _dirs, names in os.walk(p):
                files += [os.path.join(root, n) for n in names
                          if os.path.splitext(n)[1].lower() in TEXT_EXT]
        else:
            files.append(p)
        for f in files:
            with open(f, encoding="utf-8", errors="ignore") as fh:
                chars.update(fh.read())
    # 줄바꿈·탭 같은 제어 문자는 그리지 않는다
    return {c for c in chars if ord(c) >= 0x20}


def main():
    if len(sys.argv) < 4:
        print(__doc__)
        return 2
    src, out, sources = sys.argv[1], sys.argv[2], sys.argv[3:]
    wanted = collect_chars(sources) | set(SAFETY)

    original_cmap = TTFont(src).getBestCmap()
    in_font = {c for c in wanted if ord(c) in original_cmap}
    not_in_font = sorted(wanted - in_font)

    opts = subset.Options()
    opts.layout_features = ["*"]
    opts.name_IDs = ["*"]
    opts.name_languages = ["*"]
    opts.notdef_outline = True
    opts.glyph_names = False
    font = TTFont(src)
    sub = subset.Subsetter(opts)
    sub.populate(unicodes=[ord(c) for c in in_font])
    sub.subset(font)
    os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
    font.save(out)

    # 빠진 글자 검사: 결과 글꼴을 다시 열어 확인한다
    result_cmap = TTFont(out).getBestCmap()
    missing = sorted(c for c in in_font if ord(c) not in result_cmap)

    before, after = os.path.getsize(src), os.path.getsize(out)
    print(f"글꼴 줄이기: {before:,} -> {after:,} bytes, 글자 {len(in_font)}개")
    if not_in_font:
        shown = " ".join(f"{c}(U+{ord(c):04X})" for c in not_in_font[:40])
        print(f"경고: 원본 글꼴에도 없는 글자 {len(not_in_font)}개 (게임에 나오면 네모): {shown}")
    if missing:
        shown = " ".join(f"{c}(U+{ord(c):04X})" for c in missing)
        print(f"오류: 줄인 글꼴에서 빠진 글자가 있어요: {shown}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
