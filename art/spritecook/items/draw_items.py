# Clean redraw of 4 child-drawn items. Draw at S x, then downsample.
from PIL import Image, ImageDraw, ImageFilter, ImageChops
import os

OUT = os.path.dirname(os.path.abspath(__file__))  # writes item-*.png next to this script (art/spritecook/items)
S = 4                      # supersample factor
INK = (34, 36, 48, 255)    # outline color (same family as hero outline)
OW = 2.6                   # outline width in final px


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
    def __init__(self, w, h):
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
        """Paint color inside region ∩ m (no outline) — for shade/highlight."""
        self.paint(ImageChops.multiply(m, region), color)

    def line(self, pts, color, width, curve=True):
        d = ImageDraw.Draw(self.im)
        d.line(sc(pts), fill=color, width=int(width * S), joint="curve")
        r = width * S / 2
        for x, y in (pts[0], pts[-1]):
            d.ellipse((x * S - r, y * S - r, x * S + r, y * S + r), fill=color)

    def sparkle(self, cx, cy, r, color=(255, 255, 255, 255)):
        k = r * 0.28
        pts = [(cx, cy - r), (cx + k, cy - k), (cx + r, cy), (cx + k, cy + k),
               (cx, cy + r), (cx - k, cy + k), (cx - r, cy), (cx - k, cy - k)]
        ImageDraw.Draw(self.im).polygon(sc(pts), fill=color)

    def save(self, name):
        out = self.im.resize((self.w, self.h), Image.LANCZOS)
        p = os.path.join(OUT, name)
        out.save(p)
        return p


# ---------------- 다이아래더라이트 곡괭이 ----------------
def pickaxe():
    p = Pic(132, 120)
    # orange handle, slight lean like the child's drawing
    handle = p.poly([(46, 18), (58, 18), (66, 115), (54, 116)])
    p.part(handle, (242, 160, 48, 255))
    p.clip(handle, p.poly([(54, 18), (60, 18), (68, 116), (61, 116)]), (205, 122, 30, 255))
    p.line([(50, 40), (55, 100)], (255, 205, 120, 255), 1.4)
    # black head: arch, short left arm, longer right arm
    head_pts = [(3, 58), (46, 8), (58, 7), (93, 19), (129, 35), (124, 47),
                (90, 32), (56, 22), (50, 22), (14, 64)]
    head = p.poly(head_pts)
    p.part(head, (44, 46, 56, 255))
    # soft dark-grey top facet so it reads as metal, not a hole
    p.clip(head, p.poly([(8, 56), (47, 11), (57, 10), (92, 21), (126, 37), (126, 30), (92, 13), (58, 2), (44, 2), (0, 54)]), (78, 82, 98, 255))
    # light-blue diamond: top of the arch and the right tip (as the child coloured)
    cap = p.poly([(34, 22), (47, 8), (58, 7), (74, 12), (66, 19), (56, 16), (49, 16), (40, 26)])
    p.clip(head, cap, (120, 210, 240, 255))
    p.clip(head, p.poly([(42, 12), (48, 8), (58, 7), (66, 10), (57, 12), (49, 12)]), (200, 240, 255, 255))
    tip = p.poly([(110, 20), (140, 30), (140, 60), (106, 44)])
    p.clip(head, tip, (120, 210, 240, 255))
    p.clip(head, p.poly([(112, 27), (130, 34), (128, 38), (110, 31)]), (200, 240, 255, 255))
    # thin ink line where diamond meets black
    p.line([(40, 26), (49, 16), (56, 16), (66, 19), (74, 12)], INK, 1.2)
    p.line([(110, 25), (106, 41)], INK, 1.2)
    p.sparkle(53, 11, 5.5)
    p.sparkle(121, 33, 4)
    return p.save("item-diamond-pickaxe.png")


# ---------------- 자동차 (트럭 모양) ----------------
def car():
    p = Pic(196, 92)
    tire = (52, 54, 64, 255)
    hub = (196, 200, 210, 255)
    wheels = [(32, 79), (141, 79), (163, 79)]   # child drew 1 at back, 2 at front
    for cx, cy in wheels:
        p.part(p.ellipse((cx - 9.5, cy - 9.5, cx + 9.5, cy + 9.5)), tire)
    for cx, cy in wheels:
        p.part(p.ellipse((cx - 4, cy - 4, cx + 4, cy + 4)), hub, ow=1.2)
    # body: long low back, tall cab in front, rounded front
    front = catmull([(160, 5), (176, 9), (185, 24), (187, 44), (185, 64)], closed=False, n=8)
    body_pts = [(7, 42), (100, 37), (100, 5)] + front + [(181, 74), (173, 76), (12, 76), (6, 70)]
    body = p.poly(body_pts)
    p.part(body, (214, 217, 222, 255))
    p.clip(body, p.poly([(0, 64), (196, 62), (196, 92), (0, 92)]), (184, 188, 196, 255))
    p.line([(10, 45), (97, 40)], (242, 243, 246, 255), 1.6)
    p.line([(104, 9), (158, 9)], (242, 243, 246, 255), 1.6)
    # one window (left uncoloured by the child -> near-white glass)
    win = p.poly([(114, 14), (155, 12), (161, 17), (159, 41), (114, 43)])
    p.part(win, (246, 249, 252, 255), ow=2.0)
    p.line([(122, 36), (134, 18)], (214, 224, 234, 255), 2.2)
    # yellow light at the back (top of low part), red light at the front bottom
    yl = p.poly([(7, 42), (22, 41), (22, 54), (7, 55)])
    p.part(ImageChops.multiply(yl, body), (250, 200, 60, 255), ow=1.6)
    rl = p.poly([(170, 50), (190, 50), (190, 72), (170, 72)])
    p.part(ImageChops.multiply(rl, body), (214, 54, 78, 255), ow=1.6)
    return p.save("item-car.png")


# ---------------- 주사기총 ----------------
def syringe():
    p = Pic(132, 40)
    # thin needle first so the body covers its root
    p.line([(104, 21), (114, 20), (123, 18), (130, 19)], INK, 2.4)
    p.line([(106, 20.6), (114, 19.7), (123, 17.8), (129, 18.8)], (150, 156, 168, 255), 0.9)
    body = p.poly([(17, 6), (40, 4), (66, 7), (88, 6), (104, 7), (106, 20), (108, 34), (17, 35)])
    p.part(body, (143, 211, 240, 255))
    p.clip(body, p.poly([(0, 29), (132, 28), (132, 40), (0, 40)]), (106, 184, 222, 255))
    p.line([(22, 9), (40, 8), (66, 10)], (220, 244, 255, 255), 1.3)
    liquid = p.poly(catmull([(22, 20), (40, 14.5), (70, 13.5), (100, 18), (103, 21), (90, 26), (60, 28), (32, 26)], n=8))
    p.part(liquid, (181, 224, 74, 255), ow=1.4)
    p.clip(liquid, p.poly([(20, 23.5), (110, 23.5), (110, 31), (20, 31)]), (143, 190, 46, 255))
    handle = p.poly([(3, 36), (15, 2), (19, 2), (19, 36)])
    p.part(handle, (214, 46, 70, 255))
    p.clip(handle, p.poly([(15, 2), (19, 2), (19, 36), (15, 36)]), (170, 30, 54, 255))
    return p.save("item-syringe-gun.png")


# ---------------- 엔더열매 ----------------
def ender_fruit():
    p = Pic(132, 78)
    stem = p.poly([(86, 2), (98, 2), (98, 15), (86, 15)])
    p.part(stem, (242, 163, 58, 255))
    outline = [(87, 12), (106, 13), (119, 22), (124, 40), (119, 58), (103, 70), (70, 75),
               (38, 74), (15, 68), (8, 57), (11, 46), (27, 50), (52, 52), (70, 45), (82, 32)]
    fruit = p.poly(catmull(outline, n=10))
    p.part(fruit, (226, 72, 108, 255))
    # pink upper glow and deeper red belly (red + pink as the child coloured)
    p.clip(fruit, p.poly(catmull([(90, 14), (112, 20), (118, 38), (104, 40), (92, 30)], n=6)), (244, 128, 160, 255))
    p.clip(fruit, p.poly([(10, 50), (40, 55), (70, 50), (40, 52)]), (244, 128, 160, 255))
    p.clip(fruit, p.poly(catmull([(4, 62), (40, 66), (90, 64), (124, 46), (130, 80), (0, 80)], n=6)), (192, 44, 84, 255))
    # gloss highlight
    p.paint(p.ellipse((101, 20, 108, 33)), (255, 236, 242, 255))
    p.paint(p.ellipse((20, 55, 34, 59)), (252, 170, 192, 255))
    return p.save("item-ender-fruit.png")


if __name__ == "__main__":
    os.makedirs(OUT, exist_ok=True)
    for f in (pickaxe, car, syringe, ender_fruit):
        print(f())
