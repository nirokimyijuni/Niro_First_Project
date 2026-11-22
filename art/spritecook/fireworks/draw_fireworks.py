# 폭죽 그림 9개 (새 기록 화면용). 이 PC에서 코드로 그림, SpriteCook 크레딧 0.
# english/draw_english.py와 같은 방법: 크게(S배) 그린 뒤 LANCZOS로 줄이고, 어두운 테두리(INK)를 두른다.
# 실행:  python draw_fireworks.py            -> 개별 PNG 9개 + ../fireworks-sheet.png
#        python draw_fireworks.py --preview  -> 확대 미리보기(fireworks-preview.png)도 만든다
from PIL import Image, ImageDraw, ImageFilter
import math
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))   # 개별 PNG는 이 폴더에
SHEET = os.path.join(os.path.dirname(HERE), "fireworks-sheet.png")
S = 8                       # 크게 그리는 배율
N = 64                      # 한 개 크기(64x64)
INK = (34, 36, 48, 255)     # 테두리 색 (english/items와 같음)
OW = 1.6                    # 테두리 굵기(최종 픽셀)

# gameover.rs CONFETTI_COLORS (0~1 -> 0~255)
COLORS = {
    "yellow": (255, 219, 56),
    "pink": (255, 140, 179),
    "blue": (115, 217, 255),
}


def lighten(c, t=0.55):
    return tuple(round(v + (255 - v) * t) for v in c) + (255,)


class Pic:
    def __init__(self):
        self.size = N * S
        self.m = Image.new("L", (self.size, self.size), 0)
        self.d = ImageDraw.Draw(self.m)
        self.hi = Image.new("L", (self.size, self.size), 0)   # 밝은 반짝 점
        self.hd = ImageDraw.Draw(self.hi)

    def circle(self, x, y, r):
        self.d.ellipse(((x - r) * S, (y - r) * S, (x + r) * S, (y + r) * S), fill=255)

    def poly(self, pts):
        self.d.polygon([(x * S, y * S) for x, y in pts], fill=255)

    def glint(self, x, y, r):
        self.hd.ellipse(((x - r) * S, (y - r) * S, (x + r) * S, (y + r) * S), fill=255)

    def render(self, color):
        k = int(OW * S) * 2 + 1
        outer = self.m.filter(ImageFilter.MaxFilter(k))
        big = Image.new("RGBA", self.m.size, (0, 0, 0, 0))
        big.paste(Image.new("RGBA", self.m.size, INK), mask=outer)
        big.paste(Image.new("RGBA", self.m.size, color + (255,)), mask=self.m)
        hi = ImageChops_multiply(self.hi, self.m)
        big.paste(Image.new("RGBA", self.m.size, lighten(color)), mask=hi)
        return big.resize((N, N), Image.LANCZOS)


def ImageChops_multiply(a, b):
    from PIL import ImageChops
    return ImageChops.multiply(a, b)


def star_pts(cx, cy, n, r_out, r_in, rot=-90):
    pts = []
    for i in range(n * 2):
        a = math.radians(rot + i * 180 / n)
        r = r_out if i % 2 == 0 else r_in
        pts.append((cx + r * math.cos(a), cy + r * math.sin(a)))
    return pts


def burst_star():
    """작은 별 모양 터짐: 8갈래 뾰족한 별 + 사이사이 동그란 반짝 점."""
    p = Pic()
    p.poly(star_pts(32, 32, 8, 22, 9))
    for i in range(8):
        a = math.radians(-90 + 22.5 + i * 45)
        p.circle(32 + 27 * math.cos(a), 32 + 27 * math.sin(a), 2.6)
    p.circle(32, 32, 6)
    p.glint(32, 32, 3.2)
    return p


def burst_flower():
    """동그란 꽃 모양 터짐: 둥근 꽃잎 8개 + 가운데 큰 동그라미 + 바깥 작은 점."""
    p = Pic()
    for i in range(8):
        a = math.radians(-90 + i * 45)
        p.circle(32 + 16 * math.cos(a), 32 + 16 * math.sin(a), 6.6)
    p.circle(32, 32, 14)
    for i in range(8):
        a = math.radians(-90 + 22.5 + i * 45)
        p.circle(32 + 27.5 * math.cos(a), 32 + 27.5 * math.sin(a), 2.4)
    p.glint(32, 32, 6)
    return p


def sparkle_fall():
    """길게 떨어지는 반짝 줄기: 위쪽 작은 별에서 5갈래가 휘어 내려오고 끝에 동그란 방울."""
    p = Pic()
    sx, sy = 32, 13
    for ang in (-62, -31, 0, 31, 62):
        a = math.radians(ang)
        steps = 16
        for i in range(steps + 1):
            t = i / steps
            x = sx + math.sin(a) * 25 * t
            y = sy + math.cos(a) * 8 * t + 31 * t * t
            r = 3.0 - 1.5 * t
            p.circle(x, y, r)
        p.circle(sx + math.sin(a) * 25, sy + math.cos(a) * 8 + 31, 3.3)
    p.poly(star_pts(sx, sy, 4, 9, 3.4))
    p.circle(sx, sy, 4)
    p.glint(sx, sy, 2.2)
    return p


SHAPES = [("burst_star", burst_star), ("burst_flower", burst_flower), ("sparkle_fall", sparkle_fall)]


def main():
    sheet = Image.new("RGBA", (N * 3, N * 3), (0, 0, 0, 0))
    for row, (name, fn) in enumerate(SHAPES):
        for col, (cname, c) in enumerate(COLORS.items()):
            img = fn().render(c)
            img.save(os.path.join(HERE, f"{name}_{cname}.png"))
            sheet.paste(img, (col * N, row * N))
    sheet.save(SHEET)
    if "--preview" in sys.argv:
        bg = Image.new("RGBA", (sheet.width * 4, sheet.height * 4), (28, 30, 60, 255))
        bg.alpha_composite(sheet.resize(bg.size, Image.NEAREST))
        bg.save(os.path.join(HERE, "fireworks-preview.png"))


if __name__ == "__main__":
    main()
