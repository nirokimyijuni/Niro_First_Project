# English quiz pictures (US-088, [임시-60]). Drawn on this PC, 0 credits.
# Same method as ../items/draw_items.py: draw at S x, then downsample.
# 128x128 transparent PNG, ink outline, no letters, 6~8px margin around the object.
# Usage:  python draw_english.py            -> draw every word below
#         python draw_english.py kite cone  -> draw only those words
#         python draw_english.py --preview  -> also write english-preview.png
from PIL import Image, ImageDraw, ImageFilter, ImageChops
import os
import sys

OUT = os.path.dirname(os.path.abspath(__file__))  # writes en-*.png next to this script
S = 4                      # supersample factor
W = H = 128                # final size
INK = (34, 36, 48, 255)    # outline color (same as items/draw_items.py)
OW = 2.6                   # outline width in final px
WHITE = (255, 255, 255, 255)


def sc(pts):
    return [(x * S, y * S) for x, y in pts]


def catmull(pts, closed=True, n=10):
    out = []
    m = len(pts)
    rng = range(m) if closed else range(m - 1)
    for i in rng:
        p0 = pts[(i - 1) % m] if closed or i > 0 else pts[i]
        p1 = pts[i]
        p2 = pts[(i + 1) % m]
        p3 = pts[(i + 2) % m] if closed or i + 2 < m else pts[(i + 1) % m]
        for k in range(n):
            t = k / n
            t2, t3 = t * t, t * t * t
            x = 0.5 * ((2 * p1[0]) + (-p0[0] + p2[0]) * t + (2 * p0[0] - 5 * p1[0] + 4 * p2[0] - p3[0]) * t2 + (-p0[0] + 3 * p1[0] - 3 * p2[0] + p3[0]) * t3)
            y = 0.5 * ((2 * p1[1]) + (-p0[1] + p2[1]) * t + (2 * p0[1] - 5 * p1[1] + 4 * p2[1] - p3[1]) * t2 + (-p0[1] + 3 * p1[1] - 3 * p2[1] + p3[1]) * t3)
            out.append((x, y))
    if not closed:
        out.append(pts[-1])
    return out


class Pic:
    def __init__(self, w=W, h=H):
        self.w, self.h = w, h
        self.im = Image.new("RGBA", (w * S, h * S), (0, 0, 0, 0))

    def mask(self):
        return Image.new("L", self.im.size, 0)

    def poly(self, pts):
        m = self.mask()
        ImageDraw.Draw(m).polygon(sc(pts), fill=255)
        return m

    def ellipse(self, box):
        m = self.mask()
        x0, y0, x1, y1 = box
        ImageDraw.Draw(m).ellipse((x0 * S, y0 * S, x1 * S, y1 * S), fill=255)
        return m

    def circle(self, cx, cy, r):
        return self.ellipse((cx - r, cy - r, cx + r, cy + r))

    def rrect(self, box, r):
        m = self.mask()
        x0, y0, x1, y1 = box
        ImageDraw.Draw(m).rounded_rectangle((x0 * S, y0 * S, x1 * S, y1 * S), radius=r * S, fill=255)
        return m

    @staticmethod
    def dilate(m, w):
        r = w * S
        b = m.filter(ImageFilter.GaussianBlur(r / 2.0))
        return b.point(lambda v: 255 if v > 6 else 0)

    def paint(self, m, color):
        layer = Image.new("RGBA", self.im.size, color)
        self.im.paste(layer, (0, 0), m)

    def part(self, m, fill, ow=OW):
        """Outlined, flat-filled part drawn over what is already there."""
        if ow:
            self.paint(self.dilate(m, ow), INK)
        self.paint(m, fill)

    def clip(self, m, region, color):
        """Paint color inside region and m (no outline) - for shade/highlight."""
        self.paint(ImageChops.multiply(m, region), color)

    def line(self, pts, color, width):
        d = ImageDraw.Draw(self.im)
        d.line(sc(pts), fill=color, width=int(width * S), joint="curve")
        r = width * S / 2
        for x, y in (pts[0], pts[-1]):
            d.ellipse((x * S - r, y * S - r, x * S + r, y * S + r), fill=color)

    def inked_line(self, pts, color, width, ow=OW * 0.8):
        """A thick stroke with its own ink outline (tails, strings, sticks)."""
        self.line(pts, INK, width + 2 * ow)
        self.line(pts, color, width)

    def clip_line(self, m, pts, color, width):
        """Line painted only inside mask m (texture lines)."""
        lm = self.mask()
        d = ImageDraw.Draw(lm)
        d.line(sc(pts), fill=255, width=int(width * S), joint="curve")
        self.clip(m, lm, color)

    def sparkle(self, cx, cy, r, color=WHITE):
        k = r * 0.28
        pts = [(cx, cy - r), (cx + k, cy - k), (cx + r, cy), (cx + k, cy + k),
               (cx, cy + r), (cx - k, cy + k), (cx - r, cy), (cx - k, cy - k)]
        ImageDraw.Draw(self.im).polygon(sc(pts), fill=color)

    def save(self, word, margin=7):
        # fit: crop to the drawn object, scale so its longer side leaves `margin` px, center it
        big = self.im
        box = big.getchannel("A").point(lambda v: 255 if v > 8 else 0).getbbox()
        obj = big.crop(box)
        room = (self.w - 2 * margin) * S
        k = room / max(obj.width, obj.height)
        obj = obj.resize((max(1, round(obj.width * k)), max(1, round(obj.height * k))), Image.LANCZOS)
        big = Image.new("RGBA", big.size, (0, 0, 0, 0))
        big.alpha_composite(obj, ((big.width - obj.width) // 2, (big.height - obj.height) // 2))
        out = big.resize((self.w, self.h), Image.LANCZOS)
        name = "en-%s.png" % word
        out.save(os.path.join(OUT, name))
        return name, out


# ================= words =================

def acorn():
    p = Pic()
    brown, brown_d = (176, 112, 58, 255), (140, 84, 40, 255)
    tan, tan_d, tan_l = (222, 168, 96, 255), (190, 132, 66, 255), (246, 210, 150, 255)
    # nut (light, pointy bottom)
    nut = p.poly(catmull([(30, 58), (98, 58), (100, 80), (88, 104), (64, 120), (40, 104), (28, 80)], n=10))
    p.part(nut, tan)
    p.clip(nut, p.poly([(70, 50), (110, 50), (110, 124), (64, 124)]), tan_d)
    p.paint(p.ellipse((40, 70, 50, 92)), tan_l)
    # stem
    p.inked_line([(64, 30), (66, 18), (72, 9)], brown_d, 6)
    # cap (dark, bumpy texture)
    cap = p.poly(catmull([(20, 62), (26, 40), (46, 28), (64, 25), (82, 28), (102, 40), (108, 62), (64, 68)], n=10))
    p.part(cap, brown)
    for y in (38, 48, 58):
        for x in range(24, 108, 12):
            xo = x + (6 if y == 48 else 0)
            p.clip_line(cap, [(xo, y), (xo + 5, y + 5)], brown_d, 2.2)
            p.clip_line(cap, [(xo + 5, y), (xo, y + 5)], brown_d, 2.2)
    p.clip(cap, p.poly([(20, 64), (108, 64), (108, 70), (20, 70)]), brown_d)
    return p.save("acorn")


def kite():
    p = Pic()
    red, red_d = (230, 70, 80, 255), (196, 44, 60, 255)
    yel, yel_d = (252, 206, 64, 255), (232, 172, 40, 255)
    blue = (70, 150, 230, 255)
    top, right, bot, left = (52, 8), (102, 38), (66, 90), (14, 50)
    c = (58, 44)  # where the two sticks cross
    # tail first (behind the kite)
    tail = catmull([bot, (60, 100), (78, 106), (92, 112), (104, 106), (118, 116)], closed=False, n=8)
    p.line(tail, INK, 2.4)
    for bx, by in ((72, 103), (98, 110)):
        bow = p.poly([(bx - 10, by - 7), (bx, by), (bx - 10, by + 7)])
        bow2 = p.poly([(bx + 10, by - 7), (bx, by), (bx + 10, by + 7)])
        p.part(ImageChops.lighter(bow, bow2), blue, ow=2.0)
    body = p.poly([top, right, bot, left])
    p.part(body, red)
    p.clip(body, p.poly([top, right, c]), yel)
    p.clip(body, p.poly([right, bot, c]), red_d)
    p.clip(body, p.poly([bot, left, c]), yel_d)
    # sticks
    p.line([top, bot], INK, 2.2)
    p.line([left, right], INK, 2.2)
    p.line([(56, 16), (62, 30)], (255, 236, 140, 255), 1.8)
    return p.save("kite")


def cone():
    p = Pic()
    waffle, waffle_d = (228, 170, 92, 255), (190, 126, 56, 255)
    pink, pink_d, pink_l = (248, 150, 180, 255), (226, 108, 146, 255), (255, 210, 224, 255)
    # waffle cone
    cn = p.poly([(36, 60), (92, 60), (66, 120), (62, 120)])
    p.part(cn, waffle)
    for k in range(-4, 8):
        x = 28 + k * 11
        p.clip_line(cn, [(x, 56), (x + 50, 124)], waffle_d, 2.0)
        p.clip_line(cn, [(x + 50, 56), (x, 124)], waffle_d, 2.0)
    # scoop with wavy bottom edge
    scoop_pts = catmull([(28, 58), (26, 40), (40, 22), (64, 15), (88, 22), (102, 40), (100, 58),
                         (92, 66), (84, 60), (74, 68), (64, 61), (54, 68), (44, 60), (36, 66)], n=10)
    sc_m = p.poly(scoop_pts)
    p.part(sc_m, pink)
    p.clip(sc_m, p.poly([(70, 10), (110, 10), (110, 72), (70, 72)]), pink_d)
    p.paint(p.ellipse((40, 30, 54, 42)), pink_l)
    # cherry on top
    p.inked_line([(66, 18), (70, 10), (78, 6)], (90, 150, 60, 255), 2.0, ow=1.2)
    ch = p.circle(64, 20, 7)
    p.part(ch, (214, 40, 60, 255), ow=2.0)
    p.paint(p.circle(61.5, 17.5, 2), (255, 190, 196, 255))
    return p.save("cone")


def sky():
    p = Pic()
    blue, blue_l = (110, 186, 244, 255), (160, 214, 250, 255)
    panel = p.rrect((8, 8, 120, 120), 16)
    p.part(panel, blue)
    p.clip(panel, p.poly([(0, 84), (128, 84), (128, 128), (0, 128)]), blue_l)
    # sun with rays (top right)
    sx, sy = 82, 44
    sun_y = (255, 208, 56, 255)
    for i in range(8):
        import math
        a = i * math.pi / 4
        x0, y0 = sx + 23 * math.cos(a), sy + 23 * math.sin(a)
        x1, y1 = sx + 31 * math.cos(a), sy + 31 * math.sin(a)
        rm = p.mask()
        ImageDraw.Draw(rm).line(sc([(x0, y0), (x1, y1)]), fill=255, width=int(5 * S))
        p.clip(panel, rm, (255, 184, 40, 255))
    p.part(ImageChops.multiply(p.circle(sx, sy, 18), panel), sun_y, ow=2.0)
    p.paint(p.circle(sx - 6, sy - 6, 4.5), (255, 240, 170, 255))
    # small white cloud (bottom left)
    cl = p.poly(catmull([(20, 94), (24, 82), (36, 78), (44, 70), (58, 72), (64, 82), (72, 86), (70, 96), (44, 98)], n=8))
    p.part(ImageChops.multiply(cl, panel), WHITE, ow=2.0)
    p.clip(cl, p.poly([(10, 92), (80, 92), (80, 100), (10, 100)]), (226, 236, 246, 255))
    return p.save("sky")


def watch():
    p = Pic()
    strap, strap_d = (70, 110, 200, 255), (50, 84, 170, 255)
    metal, metal_d = (200, 206, 216, 255), (160, 166, 180, 255)
    # strap top and bottom
    st = p.rrect((46, 8, 82, 120), 8)
    p.part(st, strap)
    p.clip(st, p.poly([(70, 0), (90, 0), (90, 128), (70, 128)]), strap_d)
    for y in (100, 108):
        p.part(p.circle(64, y, 2.4), (30, 40, 70, 255), ow=0)
    # crown (knob on the right)
    p.part(p.rrect((96, 58, 106, 70), 2), metal)
    # case and face
    case = p.circle(64, 64, 34)
    p.part(case, metal)
    p.clip(case, p.poly([(64, 64), (128, 30), (128, 128), (40, 128)]), metal_d)
    face = p.circle(64, 64, 27)
    p.part(face, WHITE, ow=2.0)
    import math
    for i in range(12):
        a = i * math.pi / 6
        r0 = 20 if i % 3 else 17
        p.line([(64 + r0 * math.sin(a), 64 - r0 * math.cos(a)),
                (64 + 23 * math.sin(a), 64 - 23 * math.cos(a))], INK, 2.2 if i % 3 == 0 else 1.4)
    # hands: 10 o'clock and 2 o'clock-ish (friendly face)
    p.line([(64, 64), (54, 50)], INK, 3.2)
    p.line([(64, 64), (78, 47)], INK, 2.4)
    p.part(p.circle(64, 64, 3.2), (230, 70, 80, 255), ow=1.2)
    p.sparkle(48, 44, 4)
    return p.save("watch")


def phone():
    p = Pic()
    body_c, body_d = (52, 58, 78, 255), (36, 40, 56, 255)
    scr, scr_d = (150, 210, 245, 255), (110, 180, 230, 255)
    body = p.rrect((34, 8, 94, 120), 12)
    p.part(body, body_c)
    p.clip(body, p.poly([(80, 0), (100, 0), (100, 128), (80, 128)]), body_d)
    screen = p.rrect((40, 20, 88, 104), 4)
    p.part(screen, scr, ow=1.4)
    p.clip(screen, p.poly([(40, 80), (88, 60), (88, 104), (40, 104)]), scr_d)
    # app squares (no letters)
    cols = [(240, 96, 96, 255), (255, 200, 60, 255), (110, 200, 110, 255),
            (170, 120, 230, 255), (255, 150, 60, 255), (80, 150, 240, 255),
            (240, 120, 180, 255), (90, 200, 200, 255), (255, 255, 255, 255)]
    k = 0
    for row in range(3):
        for col in range(3):
            x = 45 + col * 14
            y = 27 + row * 15
            p.part(p.rrect((x, y, x + 10, y + 10), 3), cols[k], ow=1.0)
            k += 1
    # speaker slit and home button
    p.paint(p.rrect((56, 12.5, 72, 15.5), 1.5), (20, 22, 30, 255))
    p.part(p.circle(64, 112, 4), (90, 98, 120, 255), ow=1.2)
    p.line([(46, 76), (58, 64)], (220, 240, 255, 255), 2.0)
    return p.save("phone")


# ---------- helpers for the remaining words ----------
import math


def ell(p, cx, cy, rx, ry):
    return p.ellipse((cx - rx, cy - ry, cx + rx, cy + ry))


def add(*ms):
    r = ms[0]
    for m in ms[1:]:
        r = ImageChops.lighter(r, m)
    return r


def sub(a, b):
    return ImageChops.subtract(a, b)


def right_of(p, x):
    return p.poly([(x, -50), (200, -50), (200, 200), (x, 200)])


def below(p, y):
    return p.poly([(-50, y), (200, y), (200, 200), (-50, 200)])


def stroke_mask(p, pts, width):
    m = p.mask()
    d = ImageDraw.Draw(m)
    d.line(sc(pts), fill=255, width=int(width * S), joint="curve")
    r = width * S / 2
    for x, y in (pts[0], pts[-1]):
        d.ellipse((x * S - r, y * S - r, x * S + r, y * S + r), fill=255)
    return m


def rot(pts, deg, cx=64, cy=64):
    a = math.radians(deg)
    ca, sa = math.cos(a), math.sin(a)
    return [(cx + (x - cx) * ca - (y - cy) * sa, cy + (x - cx) * sa + (y - cy) * ca) for x, y in pts]


def heart(p, cx, cy, r):
    return add(p.circle(cx - r * 0.5, cy - r * 0.2, r * 0.55), p.circle(cx + r * 0.5, cy - r * 0.2, r * 0.55),
               p.poly([(cx - r * 1.02, cy), (cx + r * 1.02, cy), (cx, cy + r * 1.05)]))


# ---------- clothes: Apron, Bib, Vest, Jacket (kept very different) ----------

def apron():
    # long kitchen apron: neck loop, side ties, big skirt, white pocket
    p = Pic()
    red, red_d = (232, 92, 84, 255), (196, 62, 62, 255)
    p.inked_line(catmull([(48, 30), (50, 12), (64, 6), (78, 12), (80, 30)], closed=False, n=8), red_d, 4.5)
    p.inked_line([(38, 62), (18, 56), (8, 66)], red_d, 4.5)
    p.inked_line([(90, 62), (110, 56), (120, 66)], red_d, 4.5)
    body = p.poly([(44, 26), (84, 26), (84, 54), (90, 62), (98, 112), (64, 120), (30, 112), (38, 62), (44, 54)])
    p.part(body, red)
    p.clip(body, right_of(p, 76), red_d)
    p.clip(body, p.poly([(0, 56), (128, 56), (128, 64), (0, 64)]), red_d)
    pocket = p.rrect((48, 76, 80, 98), 4)
    p.part(pocket, WHITE, ow=2.0)
    p.line([(50, 32), (50, 50)], (250, 150, 140, 255), 2.0)
    return p.save("apron")


def bib():
    # small round baby bib with a neck hole, white trim and a heart
    p = Pic()
    blue = (130, 200, 240, 255)
    big = p.ellipse((18, 18, 110, 122))
    hole = p.circle(64, 22, 21)
    b = sub(big, hole)
    p.part(b, blue)
    p.clip(b, sub(big, p.ellipse((24, 24, 104, 116))), WHITE)
    p.clip(b, p.circle(64, 22, 26), WHITE)
    p.part(heart(p, 64, 76, 18), (250, 120, 150, 255), ow=2.0)
    p.part(p.circle(36, 34, 4), (255, 214, 80, 255), ow=1.4)
    return p.save("bib")


def vest():
    # sleeveless knit vest: deep V neck, open front, buttons, no sleeves
    p = Pic()
    g, g_d = (96, 168, 96, 255), (70, 136, 74, 255)
    v = p.poly([(40, 10), (52, 10), (64, 58), (76, 10), (88, 10), (90, 28), (98, 40), (108, 44),
                (108, 112), (70, 120), (64, 114), (58, 120), (20, 112), (20, 44), (30, 40), (38, 28)])
    p.part(v, g)
    p.clip(v, right_of(p, 92), g_d)
    p.clip(v, below(p, 106), g_d)
    p.line([(64, 58), (64, 116)], INK, 2.2)
    for y in (72, 88, 104):
        p.part(p.circle(71, y, 3.6), (240, 214, 120, 255), ow=1.4)
    p.line([(30, 84), (46, 84)], INK, 1.8)
    p.line([(82, 84), (98, 84)], INK, 1.8)
    return p.save("vest")


def jacket():
    # zip-up jacket with long sleeves and collar
    p = Pic()
    b, b_d, b_l = (66, 120, 210, 255), (46, 92, 176, 255), (120, 168, 236, 255)
    for pts in ([(38, 16), (22, 30), (8, 100), (26, 106), (40, 52)],
                [(90, 16), (106, 30), (120, 100), (102, 106), (88, 52)]):
        sl = p.poly(pts)
        p.part(sl, b)
    p.clip(add(p.poly([(38, 16), (22, 30), (8, 100), (26, 106), (40, 52)]),
               p.poly([(90, 16), (106, 30), (120, 100), (102, 106), (88, 52)])), below(p, 92), b_d)
    body = p.poly([(40, 14), (54, 10), (64, 22), (74, 10), (88, 14), (94, 24), (94, 118), (34, 118), (34, 24)])
    p.part(body, b)
    p.clip(body, below(p, 108), b_d)
    for pts in ([(54, 10), (64, 24), (52, 34), (44, 14)], [(74, 10), (64, 24), (76, 34), (84, 14)]):
        p.part(p.poly(pts), b_l, ow=1.8)
    p.line([(64, 24), (64, 118)], (200, 206, 216, 255), 2.6)
    p.line([(64, 24), (64, 118)], INK, 1.0)
    p.part(p.rrect((61, 30, 67, 42), 2), (220, 224, 232, 255), ow=1.4)
    p.line([(42, 80), (52, 90)], INK, 1.8)
    p.line([(86, 80), (76, 90)], INK, 1.8)
    return p.save("jacket")


# ---------- people and toys: Baby, Doll, King, Cheek ----------
SKIN, SKIN_D = (255, 214, 180, 255), (240, 186, 150, 255)
PINK_CHEEK = (250, 150, 166, 255)


def baby():
    # baby wrapped in a blanket (swaddle) with one hair curl
    p = Pic()
    bl, bl_d, bl_l = (150, 200, 245, 255), (110, 168, 228, 255), (196, 226, 252, 255)
    hood = p.circle(64, 44, 38)
    p.part(hood, bl)
    bundle = p.poly(catmull([(28, 58), (100, 58), (106, 90), (88, 118), (64, 122), (40, 118), (22, 90)], n=10))
    p.part(bundle, bl)
    p.clip(bundle, p.poly([(20, 64), (108, 64), (64, 112)]), bl_l)
    p.line([(26, 66), (64, 110), (102, 66)], bl_d, 2.0)
    head = p.circle(64, 46, 27)
    p.part(head, SKIN)
    p.inked_line([(60, 22), (63, 15), (70, 16), (68, 22)], (180, 120, 70, 255), 2.4, ow=1.2)
    for x in (53, 75):
        p.paint(ell(p, x, 46, 3, 3.8), INK)
        p.paint(p.circle(x - 1, 44.5, 1.1), WHITE)
    for x in (45, 83):
        p.paint(ell(p, x, 56, 6, 4), PINK_CHEEK)
    p.line(catmull([(57, 60), (64, 64), (71, 60)], closed=False, n=6), INK, 1.8)
    return p.save("baby")


def doll():
    # rag doll: yarn hair loops, button eyes, stitched smile, patched dress
    p = Pic()
    yarn, yarn_d = (244, 140, 60, 255), (214, 106, 40, 255)
    cloth = (250, 228, 198, 255)
    dress, dress_d = (150, 110, 210, 255), (122, 84, 184, 255)
    for i in range(11):
        a = math.radians(-200 + i * 22)
        p.part(p.circle(64 + 25 * math.cos(a), 36 + 25 * math.sin(a), 8), yarn, ow=1.8)
    for lx in (54, 74):  # legs with striped socks
        leg = p.rrect((lx - 5, 92, lx + 5, 118), 3)
        p.part(leg, WHITE, ow=2.0)
        for y in (98, 106):
            p.clip(leg, p.poly([(0, y), (128, y), (128, y + 4), (0, y + 4)]), (230, 80, 90, 255))
        p.part(p.rrect((lx - 7, 114, lx + 7, 122), 4), (60, 60, 80, 255), ow=1.6)
    p.inked_line([(48, 66), (34, 88)], cloth, 7)
    p.inked_line([(80, 66), (94, 88)], cloth, 7)
    dr = p.poly([(48, 60), (80, 60), (94, 100), (34, 100)])
    p.part(dr, dress)
    p.clip(dr, right_of(p, 78), dress_d)
    patch = p.poly([(52, 76), (66, 74), (68, 88), (54, 90)])
    p.part(patch, (255, 214, 80, 255), ow=1.6)
    head = p.circle(64, 38, 21)
    p.part(head, cloth)
    for x in (56, 72):
        p.part(p.circle(x, 36, 4.6), (52, 70, 140, 255), ow=1.4)
        p.paint(p.circle(x - 1.5, 36, 1.0), WHITE)
        p.paint(p.circle(x + 1.5, 36, 1.0), WHITE)
    for x in (51, 77):
        p.paint(p.circle(x, 45, 3.5), PINK_CHEEK)
    smile = catmull([(56, 47), (64, 51), (72, 47)], closed=False, n=6)
    p.line(smile, INK, 1.4)
    for x, y in ((58, 46.5), (64, 49.5), (70, 46.5)):
        p.line([(x, y - 2.2), (x, y + 2.2)], INK, 1.1)
    return p.save("doll")


def king():
    # king: gold crown with jewels, beard, red robe with white fur trim
    p = Pic()
    gold, gold_d = (252, 200, 60, 255), (220, 160, 30, 255)
    robe, robe_d = (210, 50, 70, 255), (172, 34, 56, 255)
    beard = (150, 96, 56, 255)
    rb = p.poly(catmull([(10, 124), (16, 96), (40, 82), (88, 82), (112, 96), (118, 124)], n=8))
    p.part(rb, robe)
    p.clip(rb, right_of(p, 84), robe_d)
    fur = add(p.poly([(30, 84), (98, 84), (96, 96), (32, 96)]), p.rrect((56, 84, 72, 128), 2))
    p.clip(rb, fur, WHITE)
    for x, y in ((40, 90), (52, 90), (76, 90), (88, 90), (64, 104), (64, 116)):
        p.paint(ell(p, x, y, 1.6, 2.6), INK)
    p.part(p.circle(64, 58, 24), SKIN)
    bd = p.poly(catmull([(42, 66), (52, 70), (64, 70), (76, 70), (86, 66), (84, 82), (74, 94), (64, 97), (54, 94), (44, 82)], n=8))
    p.part(bd, beard, ow=2.0)
    p.part(p.poly(catmull([(50, 70), (64, 66), (78, 70), (64, 74)], n=6)), (120, 72, 40, 255), ow=1.2)
    p.paint(ell(p, 64, 80, 5, 2.4), (200, 70, 80, 255))
    for x in (55, 73):
        p.paint(p.circle(x, 54, 2.8), INK)
    crown = p.poly([(38, 42), (38, 12), (50, 26), (64, 6), (78, 26), (90, 12), (90, 42)])
    p.part(crown, gold)
    p.clip(crown, below(p, 34), gold_d)
    for x, c in ((50, (60, 140, 230, 255)), (64, (230, 60, 80, 255)), (78, (60, 140, 230, 255))):
        p.part(p.circle(x, 36, 3.6), c, ow=1.2)
    for x, y in ((38, 12), (64, 6), (90, 12)):
        p.part(p.circle(x, y, 3.4), gold, ow=1.4)
    return p.save("king")


def cheek():
    # a smiling face where the round pink cheeks are the main thing
    p = Pic()
    hair = (120, 76, 44, 255)
    p.part(p.ellipse((14, 8, 114, 104)), hair)
    face = p.ellipse((22, 22, 106, 120))
    p.part(face, SKIN)
    bang = p.poly(catmull([(20, 50), (30, 16), (64, 8), (98, 16), (108, 50), (84, 34), (64, 40), (44, 34)], n=8))
    p.part(bang, hair, ow=2.0)
    for x in (48, 80):
        p.paint(ell(p, x, 66, 3.2, 4.4), INK)
        p.paint(p.circle(x - 1, 64, 1.2), WHITE)
    for x in (38, 90):
        c = p.circle(x, 86, 12)
        p.part(c, (248, 120, 146, 255), ow=1.6)
        p.paint(p.circle(x - 4, 82, 3), (255, 200, 210, 255))
    p.line(catmull([(56, 94), (64, 99), (72, 94)], closed=False, n=6), INK, 2.0)
    return p.save("cheek")


# ---------- home things ----------

def fan():
    p = Pic()
    grey, grey_d = (190, 198, 210, 255), (150, 158, 174, 255)
    blade = (120, 190, 240, 255)
    p.part(p.rrect((59, 62, 69, 112), 3), grey)
    base = p.ellipse((30, 104, 98, 122))
    p.part(base, grey)
    p.clip(base, below(p, 115), grey_d)
    cx, cy = 64, 44
    p.paint(p.circle(cx, cy, 36), (236, 244, 250, 255))
    for k in range(3):
        a = math.radians(-90 + k * 120)
        pts = [(cx, cy)]
        for t, w in ((0.3, 10), (0.7, 14), (1.0, 6)):
            pass
        b = p.poly(catmull([(cx, cy),
                            (cx + 18 * math.cos(a - 0.6), cy + 18 * math.sin(a - 0.6)),
                            (cx + 30 * math.cos(a - 0.15), cy + 30 * math.sin(a - 0.15)),
                            (cx + 30 * math.cos(a + 0.3), cy + 30 * math.sin(a + 0.3)),
                            (cx + 16 * math.cos(a + 0.5), cy + 16 * math.sin(a + 0.5))], n=8))
        p.part(b, blade, ow=1.6)
    for k in range(12):
        a = math.radians(k * 30)
        p.line([(cx + 8 * math.cos(a), cy + 8 * math.sin(a)), (cx + 35 * math.cos(a), cy + 35 * math.sin(a))],
               (110, 118, 136, 255), 1.0)
    ring = sub(p.circle(cx, cy, 38), p.circle(cx, cy, 33))
    p.part(ring, grey)
    p.part(p.circle(cx, cy, 8), (240, 100, 100, 255), ow=1.8)
    return p.save("fan")


def lamp():
    p = Pic()
    shade, shade_d = (255, 214, 90, 255), (236, 178, 50, 255)
    teal, teal_d = (80, 176, 176, 255), (56, 144, 148, 255)
    base = p.ellipse((34, 108, 94, 122))
    p.part(base, teal)
    p.part(p.rrect((60, 52, 68, 70), 2), (180, 186, 200, 255))
    vase = p.poly(catmull([(54, 66), (74, 66), (82, 88), (74, 110), (54, 110), (46, 88)], n=8))
    p.part(vase, teal)
    p.clip(vase, right_of(p, 70), teal_d)
    p.inked_line([(84, 58), (84, 76)], (200, 170, 80, 255), 1.4, ow=0.8)
    p.part(p.circle(84, 79, 3), (250, 210, 80, 255), ow=1.2)
    sh = p.poly([(38, 12), (90, 12), (108, 60), (20, 60)])
    p.part(sh, shade)
    p.clip(sh, p.poly([(76, 0), (128, 0), (128, 70), (90, 70)]), shade_d)
    p.clip(sh, below(p, 54), shade_d)
    p.line([(44, 20), (32, 52)], (255, 240, 180, 255), 2.4)
    return p.save("lamp")


def paper():
    p = Pic()
    pg = p.poly([(26, 8), (84, 8), (102, 26), (102, 120), (26, 120)])
    p.part(pg, WHITE)
    for y in range(40, 116, 12):
        p.clip_line(pg, [(30, y), (100, y)], (150, 196, 240, 255), 1.6)
    p.clip_line(pg, [(38, 8), (38, 120)], (240, 120, 120, 255), 1.6)
    p.clip(pg, right_of(p, 96), (232, 236, 244, 255))
    p.part(p.poly([(84, 8), (84, 26), (102, 26)]), (222, 228, 238, 255), ow=2.0)
    return p.save("paper")


def radio():
    p = Pic()
    red, red_d = (226, 82, 72, 255), (190, 56, 54, 255)
    p.inked_line([(86, 40), (108, 8)], (180, 186, 200, 255), 2.4, ow=1.4)
    p.part(p.circle(108, 8, 4), (180, 186, 200, 255), ow=1.6)
    p.inked_line([(34, 40), (38, 26), (90, 26), (94, 40)], (70, 70, 90, 255), 4.0, ow=1.6)
    body = p.rrect((10, 38, 118, 112), 12)
    p.part(body, red)
    p.clip(body, below(p, 102), red_d)
    sp = p.circle(42, 74, 24)
    p.part(sp, (70, 72, 88, 255), ow=2.0)
    for gx in range(-3, 4):
        for gy in range(-3, 4):
            if gx * gx + gy * gy <= 10:
                p.paint(p.circle(42 + gx * 6, 74 + gy * 6, 1.8), (150, 154, 170, 255))
    p.part(p.rrect((76, 50, 108, 68), 3), (255, 240, 200, 255), ow=1.8)
    p.line([(92, 52), (88, 66)], (230, 60, 60, 255), 1.6)
    for x in (82, 102):
        p.part(p.circle(x, 88, 6.5), (250, 220, 120, 255), ow=1.8)
    for x in (22, 98):
        p.part(p.rrect((x, 110, x + 10, 118), 2), (70, 70, 90, 255), ow=1.4)
    return p.save("radio")


def table():
    p = Pic()
    wood, wood_d, wood_l = (196, 130, 72, 255), (160, 100, 52, 255), (226, 168, 104, 255)
    for x in (36, 86):
        p.part(p.rrect((x, 46, x + 8, 100), 2), wood_d)
    for x in (14, 102):
        p.part(p.rrect((x, 50, x + 12, 120), 3), wood)
    top = p.poly([(24, 28), (104, 28), (120, 46), (8, 46)])
    p.part(top, wood_l)
    p.clip_line(top, [(30, 36), (70, 36)], wood, 1.6)
    p.clip_line(top, [(60, 41), (104, 41)], wood, 1.6)
    band = p.rrect((8, 46, 120, 58), 2)
    p.part(band, wood)
    p.clip(band, below(p, 54), wood_d)
    return p.save("table")


def dish():
    p = Pic()
    p.part(p.ellipse((8, 36, 120, 108)), (210, 216, 228, 255))
    plate = p.ellipse((8, 28, 120, 100))
    p.part(plate, (248, 250, 253, 255))
    p.clip(plate, sub(plate, p.ellipse((18, 36, 110, 92))), (110, 160, 230, 255))
    well = p.ellipse((32, 44, 96, 84))
    p.part(well, (234, 238, 246, 255), ow=1.4)
    p.sparkle(44, 54, 5)
    return p.save("dish")


def knife():
    # kitchen knife lying on a slant
    p = Pic()
    blade = rot([(44, 53), (110, 53), (126, 56), (118, 64), (96, 72), (44, 75)], -35)
    hnd = rot([(4, 56), (40, 56), (42, 58), (42, 70), (40, 72), (4, 72), (2, 70), (2, 58)], -35)
    bol = rot([(38, 52), (46, 52), (46, 76), (38, 76)], -35)
    b = p.poly(blade)
    p.part(b, (214, 222, 234, 255))
    p.clip(b, p.poly(rot([(40, 66), (130, 60), (130, 80), (40, 80)], -35)), (176, 186, 204, 255))
    p.part(p.poly(hnd), (60, 62, 78, 255))
    p.part(p.poly(bol), (170, 176, 190, 255), ow=1.8)
    for x in (12, 22, 32):
        (rx, ry), = rot([(x, 64)], -35)
        p.paint(p.circle(rx, ry, 2.4), (210, 214, 224, 255))
    (sx, sy), = rot([(80, 58)], -35)
    p.sparkle(sx, sy, 4.5)
    return p.save("knife")


# ---------- nature ----------

def maple():
    # big red maple leaf
    p = Pic()
    red, red_d = (228, 76, 50, 255), (190, 50, 40, 255)
    p.inked_line([(64, 92), (64, 122)], (140, 70, 40, 255), 4.0)
    right = [(71, 24), (79, 20), (76, 46), (92, 32), (96, 40), (114, 36), (108, 54), (120, 60),
             (92, 78), (96, 88), (70, 84), (67, 96)]
    left = [(128 - x, y) for x, y in reversed(right)]
    leaf = p.poly([(64, 6)] + right + left)
    p.part(leaf, red)
    p.clip(leaf, right_of(p, 64), red_d)
    for end in ((64, 14), (108, 40), (20, 40), (92, 82), (36, 82)):
        p.clip_line(leaf, [(64, 90), end], (150, 34, 30, 255), 1.6)
    return p.save("maple")


def pine():
    # tall pointy pine tree (3 stacked triangles)
    p = Pic()
    g, g_d = (52, 146, 84, 255), (36, 112, 66, 255)
    p.part(p.rrect((56, 98, 72, 122), 2), (150, 96, 56, 255))
    for pts in ([(64, 52), (110, 104), (18, 104)], [(64, 28), (98, 74), (30, 74)], [(64, 4), (88, 46), (40, 46)]):
        t = p.poly(pts)
        p.part(t, g)
        p.clip(t, right_of(p, 64), g_d)
    return p.save("pine")


def seed():
    # three striped sunflower seeds (no cap, unlike the acorn)
    p = Pic()

    def one(cx, cy, ang, L, Wd):
        pts = rot(catmull([(cx, cy - L), (cx + Wd, cy - L * 0.1), (cx + Wd * 0.7, cy + L * 0.6),
                           (cx, cy + L), (cx - Wd * 0.7, cy + L * 0.6), (cx - Wd, cy - L * 0.1)], n=10), ang, cx, cy)
        m = p.poly(pts)
        p.part(m, (70, 72, 84, 255))
        for dx in (-Wd * 0.45, 0, Wd * 0.45):
            p.clip_line(m, rot([(cx + dx * 0.6, cy - L), (cx + dx, cy + L)], ang, cx, cy), (226, 228, 236, 255), 2.2)

    one(34, 80, -30, 26, 15)
    one(96, 82, 30, 26, 15)
    one(64, 52, 0, 40, 22)
    return p.save("seed")


def fire():
    # campfire: flames on crossed logs
    p = Pic()
    log, log_l = (150, 92, 52, 255), (220, 170, 110, 255)
    p.inked_line([(16, 110), (112, 98)], log, 12)
    p.inked_line([(16, 98), (112, 112)], log, 12)
    for x, y in ((16, 110), (112, 112)):
        p.paint(p.circle(x, y, 4), log_l)
    outer = p.poly(catmull([(30, 100), (26, 72), (36, 50), (44, 62), (50, 30), (62, 8), (70, 34), (80, 22),
                            (88, 50), (100, 66), (98, 100)], n=10))
    p.part(outer, (238, 80, 50, 255))
    mid = p.poly(catmull([(40, 100), (38, 78), (48, 64), (54, 74), (60, 42), (72, 58), (78, 48), (88, 74), (88, 100)], n=10))
    p.part(mid, (250, 156, 50, 255), ow=0)
    inn = p.poly(catmull([(50, 100), (50, 86), (58, 74), (64, 62), (70, 74), (78, 86), (78, 100)], n=10))
    p.part(inn, (255, 228, 100, 255), ow=0)
    return p.save("fire")


def smoke():
    # grey smoke puffs rising, no fire
    p = Pic()
    g, g_l, g_d = (164, 170, 182, 255), (206, 210, 220, 255), (136, 142, 156, 255)
    for cl in ([(58, 112, 9), (68, 106, 8)],
               [(46, 82, 13), (64, 76, 15), (80, 84, 11)],
               [(36, 44, 17), (60, 30, 21), (88, 40, 18), (64, 52, 17)]):
        m = add(*[p.circle(x, y, r) for x, y, r in cl])
        p.part(m, g)
        p.clip(m, below(p, max(y for x, y, r in cl) + 2), g_d)
        x, y, r = cl[0]
        p.paint(p.circle(x - r * 0.3, y - r * 0.3, r * 0.4), g_l)
    return p.save("smoke")


def rose():
    p = Pic()
    g, g_d = (80, 160, 80, 255), (56, 128, 62, 255)
    red, red_d = (222, 50, 76, 255), (176, 30, 58, 255)
    p.inked_line([(64, 60), (62, 122)], g, 5)
    for pts in ([(62, 100), (44, 88), (34, 96), (46, 104)], [(63, 86), (82, 76), (92, 82), (80, 92)]):
        p.part(p.poly(catmull(pts, n=8)), g, ow=2.0)
    p.part(p.poly([(58, 70), (48, 58), (64, 64), (80, 58), (70, 70)]), g_d, ow=1.8)
    bloom = p.poly(catmull([(34, 40), (40, 18), (64, 10), (88, 18), (94, 40), (82, 62), (64, 68), (46, 62)], n=10))
    p.part(bloom, red)
    cup = p.poly(catmull([(36, 44), (64, 58), (92, 44), (84, 62), (64, 68), (44, 62)], n=8))
    p.clip(bloom, cup, red_d)
    p.line(catmull([(52, 30), (64, 22), (76, 30), (70, 42), (58, 40), (60, 32), (68, 32)], closed=False, n=8), red_d, 2.4)
    p.line(catmull([(40, 36), (52, 48), (64, 50), (78, 46), (90, 36)], closed=False, n=8), red_d, 2.0)
    return p.save("rose")


def beehive():
    # straw beehive (skep) with a dark door, no bees
    p = Pic()
    h, h_d = (240, 184, 64, 255), (210, 146, 40, 255)
    p.part(ell(p, 64, 18, 10, 7), h)
    for box in ((46, 20, 82, 42), (34, 36, 94, 60), (26, 54, 102, 80), (20, 74, 108, 100), (14, 94, 114, 120)):
        b = p.rrect(box, 11)
        p.part(b, h)
        p.clip(b, right_of(p, 84), h_d)
    door = sub(p.ellipse((52, 98, 76, 130)), below(p, 119))
    p.paint(door, (70, 46, 30, 255))
    return p.save("beehive")


def hole():
    # dark hole in a patch of ground (no animal)
    p = Pic()
    grass = p.ellipse((4, 40, 124, 116))
    p.part(grass, (120, 190, 90, 255))
    dirt = p.ellipse((18, 52, 110, 106))
    p.part(dirt, (176, 122, 76, 255), ow=1.8)
    hl = p.ellipse((30, 60, 98, 96))
    p.part(hl, (44, 32, 30, 255), ow=1.8)
    p.clip(hl, below(p, 80), (28, 20, 20, 255))
    for x, y, r in ((22, 50, 4), (106, 52, 3.5), (14, 66, 3)):
        p.part(p.circle(x, y, r), (176, 122, 76, 255), ow=1.4)
    for x in (24, 100):
        p.inked_line([(x, 44), (x - 3, 34)], (120, 190, 90, 255), 2.0, ow=1.2)
        p.inked_line([(x + 4, 44), (x + 6, 32)], (120, 190, 90, 255), 2.0, ow=1.2)
    return p.save("hole")


# ---------- animals ----------

def kangaroo():
    p = Pic()
    fur, fur_d, belly = (198, 132, 76, 255), (166, 104, 56, 255), (240, 200, 150, 255)
    tail = p.poly([(48, 84), (54, 98), (30, 110), (6, 120), (4, 115), (26, 101), (40, 86)])
    p.part(tail, fur)
    p.part(p.ellipse((50, 106, 100, 120)), fur_d)
    p.part(p.ellipse((40, 64, 80, 112)), fur)
    body = p.poly(catmull([(48, 62), (60, 40), (76, 34), (88, 42), (88, 68), (80, 96), (62, 104), (46, 92)], n=10))
    p.part(body, fur)
    p.clip(body, p.ellipse((64, 46, 92, 102)), belly)
    p.part(p.circle(80, 74, 7), fur, ow=1.6)  # joey peeking from the pouch
    p.paint(p.circle(82, 72, 1.4), INK)
    p.clip(body, p.poly([(60, 78), (96, 78), (96, 104), (60, 104)]), belly)
    p.line(catmull([(64, 79), (78, 82), (92, 78)], closed=False, n=6), INK, 1.6)
    p.inked_line([(84, 52), (94, 62), (98, 58)], fur, 4.0)
    for pts in ([(78, 18), (74, 0), (86, 14)], [(86, 16), (88, 0), (94, 16)]):
        p.part(p.poly(pts), fur)
    head = add(p.ellipse((74, 14, 100, 40)), p.ellipse((88, 20, 116, 36)))
    p.part(head, fur)
    p.paint(p.circle(91, 24, 2.6), INK)
    p.paint(p.circle(113, 26, 2.2), INK)
    return p.save("kangaroo")


def elephant():
    p = Pic()
    g, g_d, g_e = (170, 178, 194, 255), (140, 148, 166, 255), (150, 158, 178, 255)
    p.inked_line([(108, 62), (118, 78)], g, 2.4)
    for box in ((44, 80, 60, 118), (62, 84, 76, 116), (84, 84, 98, 116), (100, 80, 114, 118)):
        leg = p.rrect(box, 4)
        p.part(leg, g)
    body = p.ellipse((36, 34, 116, 98))
    p.part(body, g)
    p.clip(body, below(p, 84), g_d)
    p.inked_line(catmull([(20, 56), (12, 78), (14, 96), (22, 106), (30, 100)], closed=False, n=8), g, 10)
    p.part(p.circle(34, 48, 24), g)
    p.part(p.ellipse((42, 26, 76, 76)), g_e)
    p.inked_line([(22, 66), (12, 72)], WHITE, 3.0, ow=1.2)
    p.paint(p.circle(26, 42, 2.8), INK)
    for x in (48, 66, 88, 104):
        p.paint(ell(p, x + 4, 115, 3, 2), (220, 224, 232, 255))
    return p.save("elephant")


def hen():
    p = Pic()
    br, br_d, br_l = (204, 120, 64, 255), (166, 90, 44, 255), (230, 160, 100, 255)
    yel = (250, 196, 60, 255)
    tail = p.poly([(10, 26), (26, 36), (22, 20), (38, 40), (36, 24), (48, 48), (30, 66)])
    p.part(tail, br_d)
    for x in (56, 72):
        p.inked_line([(x, 92), (x, 114)], yel, 3.0, ow=1.4)
        p.inked_line([(x - 6, 116), (x + 8, 116)], yel, 3.0, ow=1.4)
    body = p.poly(catmull([(28, 56), (44, 44), (70, 48), (84, 40), (90, 60), (82, 84), (62, 98), (38, 92), (26, 74)], n=10))
    p.part(body, br)
    wing = p.ellipse((40, 58, 76, 84))
    p.part(wing, br_l, ow=2.0)
    p.clip_line(wing, [(46, 70), (72, 70)], br_d, 1.4)
    for x, y, r in ((84, 14, 5.5), (92, 11, 6.5), (100, 15, 5.5)):
        p.part(p.circle(x, y, r), (226, 50, 60, 255), ow=1.8)
    p.part(p.circle(90, 32, 16), br)
    p.part(ell(p, 101, 46, 4, 6), (226, 50, 60, 255), ow=1.6)
    p.part(p.poly([(104, 28), (118, 34), (104, 40)]), yel, ow=1.8)
    p.paint(p.circle(94, 28, 2.6), INK)
    return p.save("hen")


def lizard():
    p = Pic()
    g, g_d, spot = (110, 190, 80, 255), (78, 150, 60, 255), (240, 220, 90, 255)
    p.inked_line(catmull([(64, 82), (66, 100), (58, 114), (44, 118), (36, 110), (42, 102)], closed=False, n=8), g, 8)
    for pts in ([(56, 40), (40, 32), (34, 22)], [(72, 40), (88, 32), (94, 22)],
                [(56, 72), (40, 80), (34, 92)], [(72, 72), (88, 80), (94, 92)]):
        p.inked_line(pts, g, 6)
        p.part(p.circle(pts[-1][0], pts[-1][1], 4), g, ow=1.6)
    body = p.ellipse((48, 28, 80, 90))
    p.part(body, g)
    p.clip(body, p.poly([(61, 28), (67, 28), (67, 90), (61, 90)]), g_d)
    for x, y in ((56, 50), (72, 60), (56, 72), (70, 80)):
        p.paint(p.circle(x, y, 3), spot)
    p.part(p.ellipse((50, 4, 78, 34)), g)
    for x in (55, 73):
        p.part(p.circle(x, 15, 3.6), WHITE, ow=1.4)
        p.paint(p.circle(x, 15, 1.8), INK)
    return p.save("lizard")


def mole():
    # mole standing (no hole): dark velvet fur, pink nose, big pink paws
    p = Pic()
    fur, fur_d, fur_l = (96, 88, 104, 255), (72, 66, 82, 255), (128, 120, 138, 255)
    pink = (250, 160, 176, 255)
    for x in (46, 82):
        p.part(ell(p, x, 116, 10, 5), pink, ow=1.8)
    body = p.ellipse((22, 26, 106, 118))
    p.part(body, fur)
    p.clip(body, p.ellipse((40, 60, 88, 116)), fur_l)
    for cx in (22, 106):
        paw = ell(p, cx, 82, 15, 12)
        p.part(paw, pink, ow=2.0)
        for k in (-6, 0, 6):
            p.line([(cx + k, 76), (cx + k * 1.3, 70)], (250, 250, 250, 255), 2.0)
    p.part(ell(p, 64, 56, 10, 7.5), pink, ow=1.8)
    for x in (48, 80):
        p.paint(p.circle(x, 44, 2.4), INK)
    for dx in (-1, 1):
        for dy in (-3, 3):
            p.line([(64 + dx * 14, 58 + dy), (64 + dx * 30, 56 + dy * 2.5)], INK, 1.0)
    return p.save("mole")


def insect():
    # ladybug from above: 6 legs, 2 feelers
    p = Pic()
    red, red_d = (226, 52, 52, 255), (190, 34, 40, 255)
    blk = (44, 44, 56, 255)
    for y, d in ((54, -8), (72, 0), (92, 8)):
        p.inked_line([(40, y), (14, y + d)], blk, 3.0, ow=1.0)
        p.inked_line([(88, y), (114, y + d)], blk, 3.0, ow=1.0)
    for sx in (-1, 1):
        p.inked_line([(64 + sx * 6, 22), (64 + sx * 14, 6)], blk, 2.4, ow=1.0)
        p.part(p.circle(64 + sx * 15, 5, 3.4), blk, ow=1.0)
    p.part(p.circle(64, 28, 14), blk)
    body = p.circle(64, 74, 40)
    p.part(body, red)
    p.clip(body, right_of(p, 90), red_d)
    p.line([(64, 36), (64, 113)], INK, 2.2)
    for x, y, r in ((46, 56, 7), (82, 56, 7), (40, 82, 6.5), (88, 82, 6.5), (52, 102, 5), (76, 102, 5)):
        p.paint(p.circle(x, y, r), blk)
    for x in (58, 70):
        p.paint(p.circle(x, 22, 2.4), WHITE)
    p.paint(ell(p, 40, 46, 4, 6), (250, 130, 130, 255))
    return p.save("insect")


# ---------- outdoor and sticks: Hose, Rope, Pole, Mop, Net, Slide, Tent ----------

def hose():
    # green garden hose coil with a nozzle spraying water
    p = Pic()
    g = (70, 170, 80, 255)
    pts = []
    for i in range(0, 76):
        t = i / 75 * 2.6 * 2 * math.pi
        pts.append((58 + 40 * math.cos(t), 80 + 22 * math.sin(t) - t * 2.6))
    for k in range(0, len(pts) - 1, 15):
        p.inked_line(pts[k:k + 16], g, 8)
    end = pts[-1]
    p.inked_line([end, (92, 40), (98, 30)], g, 8)
    nz = p.poly(rot([(92, 18), (104, 18), (106, 34), (90, 34)], 35, 98, 26))
    p.part(nz, (250, 196, 60, 255))
    for x, y, r in ((110, 12, 4), (118, 22, 3.6), (120, 8, 3.2), (108, 2, 3)):
        p.part(p.circle(x, y, r), (110, 190, 250, 255), ow=1.4)
    return p.save("hose")


def rope():
    # thick braided rope with a knot and frayed ends
    p = Pic()
    tan, tan_d = (214, 170, 110, 255), (170, 124, 70, 255)
    path = catmull([(18, 24), (44, 14), (70, 30), (68, 58), (44, 74), (48, 100), (82, 110), (110, 98)], closed=False, n=10)
    p.line(path, INK, 12 + 2 * OW * 0.8)
    m = stroke_mask(p, path, 12)
    p.paint(m, tan)
    for k in range(-140, 260, 8):
        p.clip_line(m, [(k, 0), (k - 128, 128)], tan_d, 2.0)
    for (x, y), sgn in (((18, 24), -1), ((110, 98), 1)):
        for d in (-6, 0, 6):
            p.inked_line([(x, y + d * 0.6), (x + sgn * 10, y + d * 1.2)], tan, 2.4, ow=1.0)
    kn = p.circle(57, 66, 11)
    p.part(kn, tan)
    p.clip_line(kn, [(48, 60), (66, 72)], tan_d, 2.2)
    p.clip_line(kn, [(48, 72), (66, 60)], tan_d, 2.2)
    return p.save("rope")


def pole():
    # tall plain pole with a round top and a stone base
    p = Pic()
    shaft = p.rrect((56, 16, 72, 108), 3)
    p.part(shaft, (120, 146, 186, 255))
    p.clip(shaft, right_of(p, 66), (92, 116, 160, 255))
    p.line([(60, 22), (60, 100)], (180, 200, 228, 255), 1.8)
    p.part(p.circle(64, 12, 9), (252, 200, 60, 255))
    p.paint(p.circle(61, 9, 2.5), (255, 240, 170, 255))
    base = p.rrect((40, 104, 88, 122), 3)
    p.part(base, (176, 180, 190, 255))
    p.clip(base, below(p, 116), (146, 150, 162, 255))
    return p.save("pole")


def mop():
    # mop: wooden stick with a shaggy white head
    p = Pic()
    p.inked_line([(102, 6), (60, 82)], (200, 140, 80, 255), 6)
    for k in range(9):
        x0 = 44 + k * 4
        x1 = 14 + k * 11
        p.inked_line(catmull([(x0, 86), (x0 + (x1 - x0) * 0.5 + (4 if k % 2 else -4), 104), (x1, 120)], closed=False, n=6),
                     (236, 238, 244, 255), 6, ow=1.6)
    p.part(p.rrect((40, 78, 80, 92), 4), (80, 140, 220, 255))
    return p.save("mop")


def net():
    # butterfly net: stick, red hoop, mesh bag
    p = Pic()
    p.inked_line([(8, 122), (56, 46)], (200, 140, 80, 255), 6)
    bag = p.poly(catmull([(58, 36), (110, 30), (110, 66), (98, 96), (84, 110), (70, 98), (60, 66)], n=10))
    p.part(bag, (244, 246, 250, 255), ow=2.0)
    for k in range(-60, 200, 9):
        p.clip_line(bag, [(k, 0), (k + 90, 128)], (150, 160, 180, 255), 1.2)
        p.clip_line(bag, [(k, 128), (k + 90, 0)], (150, 160, 180, 255), 1.2)
    ring = sub(p.ellipse((52, 16, 116, 54)), p.ellipse((58, 22, 110, 48)))
    p.part(ring, (230, 70, 70, 255), ow=1.8)
    return p.save("net")


def slide():
    p = Pic()
    blue, yel, red, red_l = (70, 140, 230, 255), (252, 206, 64, 255), (232, 80, 80, 255), (250, 140, 130, 255)
    p.inked_line([(14, 122), (26, 30)], blue, 5)
    p.inked_line([(36, 122), (46, 30)], blue, 5)
    for y in (50, 70, 90, 110):
        x0 = 14 + (122 - y) / 92 * 12
        p.inked_line([(x0 + 3, y), (x0 + 21, y)], blue, 3.4, ow=1.2)
    p.inked_line(catmull([(26, 30), (28, 14), (46, 12), (50, 24)], closed=False, n=6), blue, 3.4, ow=1.2)
    p.part(p.rrect((22, 24, 58, 34), 3), yel)
    chute = p.poly([(48, 26), (60, 22), (124, 104), (124, 118), (108, 118)])
    p.part(chute, red)
    p.line([(60, 23), (123, 104)], red_l, 2.4)
    p.inked_line([(100, 104), (100, 122)], (170, 176, 190, 255), 3.0, ow=1.2)
    return p.save("slide")


def tent():
    # orange camping tent with a dark door
    p = Pic()
    o, o_d = (246, 140, 60, 255), (210, 106, 40, 255)
    p.line([(18, 108), (6, 118)], INK, 1.6)
    p.line([(122, 102), (126, 116)], INK, 1.6)
    side = p.poly([(64, 14), (98, 10), (124, 100), (108, 108)])
    p.part(side, o_d)
    front = p.poly([(64, 14), (108, 108), (18, 108)])
    p.part(front, o)
    door = p.poly([(64, 42), (82, 108), (46, 108)])
    p.paint(door, (64, 52, 64, 255))
    p.part(p.poly([(64, 42), (46, 108), (36, 108)]), (255, 196, 120, 255), ow=1.6)
    p.inked_line([(64, 14), (64, 4)], (150, 96, 56, 255), 2.4, ow=1.2)
    return p.save("tent")


# ---------- food: Cheese, Ice, Candy ----------

def cheese():
    p = Pic()
    top_c, front_c, hole_c = (255, 226, 112, 255), (250, 196, 60, 255), (224, 156, 40, 255)
    A, B, C, h = (8, 64), (92, 30), (120, 46), 40
    front = p.poly([A, C, (C[0], C[1] + h), (A[0], A[1] + h)])
    p.part(front, front_c)
    top = p.poly([A, B, C])
    p.part(top, top_c)
    for x, y, rx, ry in ((40, 86, 6, 5), (74, 78, 7, 6), (102, 86, 5, 4.5), (60, 96, 4, 3.5)):
        p.clip(front, ell(p, x, y, rx, ry), hole_c)
    for x, y, rx, ry in ((70, 48, 5, 3), (96, 44, 4, 2.4)):
        p.clip(top, ell(p, x, y, rx, ry), hole_c)
    return p.save("cheese")


def ice():
    # three light-blue ice cubes
    p = Pic()
    f, t, s = (170, 220, 250, 255), (222, 244, 255, 255), (128, 194, 236, 255)

    def cube(x, y, z):
        p.part(add(p.poly([(x, y), (x + z, y), (x + z, y + z), (x, y + z)]),
                   p.poly([(x, y), (x + z * 0.35, y - z * 0.3), (x + z * 1.35, y - z * 0.3), (x + z, y)]),
                   p.poly([(x + z, y), (x + z * 1.35, y - z * 0.3), (x + z * 1.35, y + z * 0.7), (x + z, y + z)])), f)
        p.paint(p.poly([(x, y), (x + z * 0.35, y - z * 0.3), (x + z * 1.35, y - z * 0.3), (x + z, y)]), t)
        p.paint(p.poly([(x + z, y), (x + z * 1.35, y - z * 0.3), (x + z * 1.35, y + z * 0.7), (x + z, y + z)]), s)
        p.line([(x, y), (x + z, y), (x + z * 1.35, y - z * 0.3)], INK, 1.2)
        p.line([(x + z, y), (x + z, y + z)], INK, 1.2)
        p.line([(x + z * 0.15, y + z * 0.75), (x + z * 0.15, y + z * 0.2), (x + z * 0.45, y + z * 0.2)], WHITE, 2.4)

    cube(46, 34, 36)
    cube(10, 72, 38)
    cube(60, 78, 36)
    return p.save("ice")


def candy():
    # wrapped candy with twisted ends and stripes
    p = Pic()
    pk, pk_l = (246, 100, 146, 255), (252, 176, 202, 255)
    for sx in (-1, 1):
        w = p.poly([(64 + sx * 26, 64), (64 + sx * 58, 38), (64 + sx * 52, 64), (64 + sx * 58, 90)])
        p.part(w, pk_l)
        p.line([(64 + sx * 30, 64), (64 + sx * 50, 50)], (230, 130, 166, 255), 1.6)
        p.line([(64 + sx * 30, 64), (64 + sx * 50, 78)], (230, 130, 166, 255), 1.6)
    c = p.ellipse((32, 40, 96, 88))
    p.part(c, pk)
    for k in range(-40, 160, 14):
        p.clip_line(c, [(k, 30), (k + 30, 100)], WHITE, 4.0)
    p.paint(ell(p, 50, 52, 6, 3.5), (255, 220, 232, 255))
    return p.save("candy")


def bone():
    p = Pic()
    m = add(p.rrect((28, 54, 100, 74), 4), p.circle(28, 50, 14), p.circle(28, 78, 14),
            p.circle(100, 50, 14), p.circle(100, 78, 14))
    p.part(m, (250, 242, 222, 255))
    p.clip(m, below(p, 70), (226, 214, 186, 255))
    p.line([(38, 60), (88, 60)], WHITE, 2.4)
    return p.save("bone")


def feet():
    # two bare feet seen from below, five toes each
    p = Pic()
    sole = [(20, 52), (34, 44), (52, 46), (56, 62), (50, 86), (48, 108), (38, 120), (26, 116), (24, 96), (18, 74)]
    toes = [(50, 34, 7.5), (38, 31, 5.5), (29, 33, 5), (21, 38, 4.2), (15, 46, 3.6)]
    for mir in (False, True):
        fx = (lambda x: 128 - x) if mir else (lambda x: x)
        for x, y, r in toes:
            p.part(p.circle(fx(x) + (0 if mir else 0), y, r), SKIN, ow=2.0)
        m = p.poly(catmull([(fx(x), y) for x, y in sole], n=10))
        p.part(m, SKIN)
        p.clip(m, p.circle(fx(38), 106, 10), SKIN_D)
        p.clip(m, p.circle(fx(40), 58, 10), SKIN_D)
    return p.save("feet")


def teeth():
    # open smiling mouth showing top and bottom rows of teeth
    p = Pic()
    lips = p.poly(catmull([(8, 64), (34, 38), (64, 44), (94, 38), (120, 64), (94, 94), (64, 102), (34, 94)], n=10))
    p.part(lips, (230, 90, 110, 255))
    inside = p.poly(catmull([(22, 64), (40, 52), (64, 54), (88, 52), (106, 64), (88, 84), (64, 90), (40, 84)], n=10))
    p.part(inside, (120, 30, 50, 255), ow=1.6)
    p.clip(inside, ell(p, 64, 82, 18, 8), (240, 120, 140, 255))
    up = p.poly([(0, 0), (128, 0), (128, 66), (0, 66)])
    lo = p.poly([(0, 78), (128, 78), (128, 128), (0, 128)])
    p.clip(inside, up, WHITE)
    p.clip(inside, sub(lo, ell(p, 64, 82, 18, 8)), WHITE)
    for x in range(34, 100, 10):
        p.clip_line(inside, [(x, 50), (x, 66)], (150, 156, 176, 255), 1.2)
    p.clip_line(inside, [(20, 66), (110, 66)], (150, 156, 176, 255), 1.2)
    p.paint(ell(p, 40, 46, 6, 2.6), (250, 170, 186, 255))
    return p.save("teeth")


WORDS = {
    "acorn": acorn, "apron": apron, "baby": baby, "fan": fan, "jacket": jacket, "kangaroo": kangaroo,
    "lamp": lamp, "maple": maple, "paper": paper, "radio": radio, "table": table, "beehive": beehive,
    "cheek": cheek, "cheese": cheese, "elephant": elephant, "feet": feet, "hen": hen, "net": net,
    "seed": seed, "teeth": teeth, "vest": vest, "dish": dish, "fire": fire, "kite": kite, "knife": knife,
    "lizard": lizard, "pine": pine, "slide": slide, "bib": bib, "bone": bone, "cone": cone, "doll": doll,
    "hole": hole, "hose": hose, "mole": mole, "mop": mop, "phone": phone, "pole": pole, "rope": rope,
    "rose": rose, "smoke": smoke, "ice": ice, "king": king, "sky": sky, "watch": watch, "candy": candy,
    "insect": insect, "tent": tent,
}


def margins(img):
    x0, y0, x1, y1 = img.getchannel("A").point(lambda v: 255 if v > 8 else 0).getbbox()
    return x0, y0, W - x1, H - y1


def preview(names, out="english-preview.png", cols=3):
    cell, pad, lab = 128, 12, 18
    rows = (len(names) + cols - 1) // cols
    sheet = Image.new("RGBA", (cols * (cell + pad) + pad, rows * (cell + pad + lab) + pad), (236, 238, 242, 255))
    d = ImageDraw.Draw(sheet)
    for i, n in enumerate(names):
        im = Image.open(os.path.join(OUT, "en-%s.png" % n))
        x = pad + (i % cols) * (cell + pad)
        y = pad + (i // cols) * (cell + pad + lab)
        # checker so transparency is visible
        for cy in range(0, cell, 16):
            for cx in range(0, cell, 16):
                c = (255, 255, 255, 255) if (cx + cy) // 16 % 2 == 0 else (222, 226, 232, 255)
                d.rectangle((x + cx, y + cy, x + cx + 15, y + cy + 15), fill=c)
        sheet.alpha_composite(im, (x, y))
        d.text((x + 4, y + cell + 3), n, fill=(40, 40, 50, 255))  # label only on the preview sheet
    path = os.path.join(OUT, out)
    sheet.save(path)
    return path


if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    names = args or list(WORDS)
    for n in names:
        fname, img = WORDS[n]()
        print(fname, "margins L,T,R,B =", margins(img))
    if "--preview" in sys.argv:
        print(preview(names, "english-preview-all.png", 8) if len(names) > 6 else preview(names))
