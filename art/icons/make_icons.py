# 미스 빼꼼 아이콘: 원본(art/concept/icon-miss-peek.png)에서 잰 좌표 그대로 벡터처럼 다시 그려 크기별로 만든다.
from PIL import Image, ImageDraw, ImageChops
import os, shutil

# 프로젝트 폴더 = 이 스크립트(art/icons/make_icons.py)에서 두 칸 위
ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
SRC = os.path.join(ROOT, 'art', 'concept', 'icon-miss-peek.png')
OUT = os.path.join(ROOT, 'art', 'icons')
WEB = os.path.join(ROOT, 'web', 'icons')

BLUE = (168, 213, 234, 255)   # 원본 하늘색 그대로
BLACK = (0, 0, 0, 255)
WHITE = (255, 255, 255, 255)
GRAY = (88, 88, 88, 255)      # 원본 몸·손 테두리

# 원본 좌표(픽셀)에서 잰 모양
RECT = (12.5, 9.5, 346.5, 340.0, 50)      # 둥근 네모 x0,y0,x1,y1,반지름
TBAR = (84.0, 143.5, 274.5, 194.5, 9)     # T 가로막대
TSTEM = (153.5, 190.0, 205.5, 280.0)      # T 기둥 (몸 뒤로 들어감)
BODY = (180.5, 341.2, 110.0)              # 몸 동그라미 중심, 반지름
BODY_BOTTOM = 345.5                        # 몸 아래 선
LINE_W = 1.5
EYES = [(140.0, 302.0, 9.6, 21.7), (220.8, 302.0, 9.6, 21.7)]
HANDS = [(62.0, 327.0, 19.4), (298.8, 327.0, 19.2)]


def render(w, h, x0, y0, scale, full_bg=False, ss=8, snap=False):
    """원본 좌표 (x0,y0)가 왼쪽 위, 1원본픽셀 = scale 출력픽셀."""
    W, H = w * ss, h * ss
    k = scale * ss
    P = lambda x, y: ((x - x0) * k, (y - y0) * k)
    # 작은 크기: T와 눈을 출력 픽셀 칸에 맞춰 또렷하게 (모양·위치는 그대로)
    S = (lambda x, y: (round((x - x0) * scale) * ss, round((y - y0) * scale) * ss)) if snap else P
    img = Image.new('RGBA', (W, H), BLUE if full_bg else (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    # 하늘색 둥근 네모
    x1, y1, x2, y2, r = RECT
    if not full_bg:
        d.rounded_rectangle([*P(x1, y1), *P(x2, y2)], radius=r * k, fill=BLUE)
    # T 안테나
    a, b, c, e, r = TBAR
    d.rounded_rectangle([*S(a, b), *S(c, e)], radius=r * k, fill=BLACK)
    a, b, c, e = TSTEM
    d.rectangle([*S(a, b), *S(c, e)], fill=BLACK)
    # 몸 (아래 선에서 잘림)
    body = Image.new('RGBA', (W, H), (0, 0, 0, 0))
    bd = ImageDraw.Draw(body)
    cx, cy, R = BODY
    o = LINE_W / 2
    bd.ellipse([*P(cx - R - o, cy - R - o), *P(cx + R + o, cy + R + o)], fill=GRAY)
    bd.ellipse([*P(cx - R + o, cy - R + o), *P(cx + R - o, cy + R - o)], fill=WHITE)
    cut = int(round((BODY_BOTTOM + o - y0) * k))
    if cut < H:
        bd.rectangle([0, cut, W, H], fill=(0, 0, 0, 0))
    bd.rectangle([*P(cx - R, BODY_BOTTOM - o), *P(cx + R, BODY_BOTTOM + o)], fill=GRAY)
    img.alpha_composite(body)
    d = ImageDraw.Draw(img)
    # 눈
    for ex, ey, rx, ry in EYES:
        if snap:
            px = (ex - x0) * scale; py = (ey - y0) * scale
            ew = max(1, round(2 * rx * scale)); eh = max(2, round(2 * ry * scale))
            l = round(px - ew / 2); t = round(py - eh / 2)
            box = [l * ss, t * ss, (l + ew) * ss - 1, (t + eh) * ss - 1]
            if ew <= 2:
                d.rectangle(box, fill=BLACK)
            else:
                d.ellipse(box, fill=BLACK)
        else:
            d.ellipse([*P(ex - rx, ey - ry), *P(ex + rx, ey + ry)], fill=BLACK)
    # 손
    for hx, hy, hr in HANDS:
        d.ellipse([*P(hx - hr - o, hy - hr - o), *P(hx + hr + o, hy + hr + o)], fill=GRAY)
        d.ellipse([*P(hx - hr + o, hy - hr + o), *P(hx + hr - o, hy + hr - o)], fill=WHITE)
    return img.resize((w, h), Image.BOX if snap else Image.LANCZOS)


# 정사각 틀: 원본에서 x 10.5~348.5, y 9.5~347.5 (338칸)
BX, BY, BL = 10.5, 9.5, 338.0


def icon(size, full_bg=False):
    return render(size, size, BX, BY, size / BL, full_bg, snap=size <= 48)


def maskable(size):
    # 캐릭터 가장 먼 점(손 아래 모서리)까지 171칸 → 안전 원(반지름 40%) 안에 들어가게
    cx, cy = 180.5, 245.5
    scale = (size * 0.40) / 171.5
    half = size / 2 / scale
    return render(size, size, cx - half, cy - half, scale, full_bg=True)


def flatten_white(im):
    bg = Image.new('RGBA', im.size, (255, 255, 255, 255))
    bg.alpha_composite(im)
    return bg.convert('RGB')


os.makedirs(OUT, exist_ok=True)
os.makedirs(WEB, exist_ok=True)
files = {}
files['favicon-16.png'] = icon(16)
files['favicon-32.png'] = icon(32)
files['apple-touch-icon.png'] = icon(180, full_bg=True)
files['icon-192.png'] = icon(192)
files['icon-512.png'] = icon(512)
files['icon-maskable-512.png'] = maskable(512)
for name, im in files.items():
    im.save(os.path.join(OUT, name), optimize=True)
# apple: 투명 없이
Image.open(os.path.join(OUT, 'apple-touch-icon.png')).convert('RGB').save(os.path.join(OUT, 'apple-touch-icon.png'), optimize=True)
Image.open(os.path.join(OUT, 'icon-maskable-512.png')).convert('RGB').save(os.path.join(OUT, 'icon-maskable-512.png'), optimize=True)
# ico: 16·32·48 각각 따로 그린 것
i48 = icon(48)
i16, i32 = files['favicon-16.png'], files['favicon-32.png']
i48.save(os.path.join(OUT, 'favicon.ico'), format='ICO', sizes=[(16, 16), (32, 32), (48, 48)],
         append_images=[i16, i32])
for name in list(files) + ['favicon.ico']:
    shutil.copyfile(os.path.join(OUT, name), os.path.join(WEB, name))

# 비교 그림: 원본 vs 다시 그린 것(같은 크기·같은 틀) vs 차이
src = Image.open(SRC).convert('RGBA')
sw, sh = src.size
re = flatten_white(render(sw, sh, 0, 0, 1.0))
diff = ImageChops.difference(src.convert('RGB'), re)
diff = diff.point(lambda v: min(255, v * 3))
cmp_ = Image.new('RGB', (sw * 3 + 40, sh + 40), (255, 255, 255))
for i, im in enumerate([src.convert('RGB'), re, ImageChops.invert(diff)]):
    cmp_.paste(im, (10 + i * (sw + 10), 30))
ImageDraw.Draw(cmp_).text((10, 8), 'original  |  redraw (same size)  |  difference x3 (dark = differs)', fill=(0, 0, 0))
cmp_.save(os.path.join(OUT, 'icons-compare-original.png'))
import statistics
px = list(ImageChops.difference(src.convert('RGB'), re).convert('L').get_flattened_data())
print('mean diff', sum(px) / len(px), 'pixels >40:', sum(1 for v in px if v > 40), 'of', len(px))

# 미리보기: 실제 크기 + 16·32·48 확대(픽셀 그대로)
pad = 20
tiles = [('16', i16, 1), ('32', i32, 1), ('48', i48, 1), ('16 x8', i16, 8), ('32 x4', i32, 4), ('48 x3', i48, 3),
         ('apple 180', Image.open(os.path.join(OUT, 'apple-touch-icon.png')).convert('RGBA'), 1),
         ('192', files['icon-192.png'], 1),
         ('512', files['icon-512.png'], 1),
         ('maskable 512', Image.open(os.path.join(OUT, 'icon-maskable-512.png')).convert('RGBA'), 1)]
row1 = tiles[:6]
row2 = tiles[6:]
def tsize(t):
    return t[1].width * t[2]
W1 = sum(tsize(t) + pad for t in row1) + pad
W2 = sum(tsize(t) + pad for t in row2) + pad
H1 = max(tsize(t) for t in row1) + 40
H2 = 512 + 40
canvas = Image.new('RGBA', (max(W1, W2), H1 + H2 + pad), (236, 236, 236, 255))
dr = ImageDraw.Draw(canvas)
x = pad
for label, im, z in row1:
    big = im.resize((im.width * z, im.height * z), Image.NEAREST) if z > 1 else im
    canvas.alpha_composite(big, (x, 30))
    dr.text((x, 10), label, fill=(0, 0, 0, 255))
    x += big.width + pad
x = pad
for label, im, z in row2:
    canvas.alpha_composite(im, (x, H1 + 30))
    if label.startswith('maskable'):
        # 안전 원(80%) 표시
        r = 512 * 0.40
        dr.ellipse([x + 256 - r, H1 + 30 + 256 - r, x + 256 + r, H1 + 30 + 256 + r], outline=(255, 0, 0, 255), width=2)
    dr.text((x, H1 + 10), label, fill=(0, 0, 0, 255))
    x += im.width + pad
canvas.convert('RGB').save(os.path.join(OUT, 'icons-preview.png'))
print('ok')
