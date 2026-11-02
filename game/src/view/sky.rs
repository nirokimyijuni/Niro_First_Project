//! 하늘과 바다 빛 (D9, D10, D15 A안): 매끄러운 하늘 그라데이션, 빛무리가 있는 해·달,
//! 다양한 별, 가장자리가 부드러운 구름 세 겹(패럴랙스), 바다 위 해·달 빛 기둥과 반짝임, 수평선 안개.
//!
//! - 그림 파일을 내려받지 않는다. 해·달·구름·빛 그림은 게임이 시작할 때 한 번만 코드로 만든다
//!   ([`SkyImages::generate`] → [`SkyArt`]). 매 프레임에는 만든 그림을 늘이고 색을 곱해서 그리기만 한다.
//! - 나중에 아이·엄마가 그린 해·달·구름으로 바꾸려면 [`SkyArt`]의 텍스처만 그림 파일로 바꿔 끼우면 된다
//!   (그리는 함수 [`draw_sun`], [`draw_moon`], [`draw_cloud`]는 그대로).
//! - logic이 주는 값은 `sky_darkness`(0 한낮 ~ 1 한밤) 하나만 쓴다. 게임 규칙은 없다.
//! - 색·크기·빠르기는 그림 값이라 config.rs가 아니라 이 파일 위쪽 상수에 모았다.
//! - 그리는 일은 [`Canvas`]를 거친다: 게임에서는 GPU([`GpuCanvas`]), 테스트에서는 CPU로 PNG를 뽑아 본다.

use super::screen::VIRTUAL_WIDTH;
use macroquad::prelude::*;

// ================= 그림 값 (색·크기·빠르기) =================

/// 수평선 y (하늘과 바다가 만나는 곳). 땅(GROUND_Y=440)보다 위.
pub const HORIZON_Y: f32 = 330.0;

// ---- 하늘 그라데이션: 위 / 가운데 / 수평선 세 색 ----
const DAY_SKY: [Color; 3] = [
    Color::new(0.24, 0.55, 0.92, 1.0),
    Color::new(0.47, 0.75, 0.97, 1.0),
    Color::new(0.80, 0.92, 0.99, 1.0),
];
/// 밤하늘 남색 (D10)
const NIGHT_SKY: [Color; 3] = [
    Color::new(0.015, 0.03, 0.12, 1.0),
    Color::new(0.05, 0.08, 0.25, 1.0),
    Color::new(0.13, 0.18, 0.40, 1.0),
];
/// 노을·새벽 하늘: 위는 보라, 가운데 분홍, 수평선은 주황
const DUSK_SKY: [Color; 3] = [
    Color::new(0.34, 0.30, 0.62, 1.0),
    Color::new(0.93, 0.50, 0.62, 1.0),
    Color::new(1.0, 0.62, 0.30, 1.0),
];
/// 가운데 색이 놓이는 높이 (0 하늘 맨 위 ~ 1 수평선)
const SKY_MID_AT: f32 = 0.55;
/// 노을빛이 하늘 맨 위까지 얼마나 번지는지 (수평선은 늘 가장 세다)
const DUSK_TOP_AMOUNT: f32 = 0.35;
/// 하늘 그라데이션 꼭짓점 줄 수. 줄 사이는 GPU가 픽셀마다 부드럽게 섞으므로 띠가 생기지 않는다.
const SKY_ROWS: usize = 28;

// ---- 해 ----
pub const SUN_RADIUS: f32 = 38.0;
/// 노을 때 해가 이만큼 더 커진다 (0.3 = 30%)
const SUNSET_GROW: f32 = 0.3;
const SUN_EDGE_DAY: Color = Color::new(1.0, 0.84, 0.36, 1.0);
const SUN_EDGE_SET: Color = Color::new(1.0, 0.42, 0.16, 1.0);
const SUN_CORE_DAY: Color = Color::new(1.0, 1.0, 0.94, 0.95);
const SUN_CORE_SET: Color = Color::new(1.0, 0.80, 0.45, 0.70);
const SUN_HALO_DAY: Color = Color::new(1.0, 0.92, 0.62, 1.0);
const SUN_HALO_SET: Color = Color::new(1.0, 0.48, 0.22, 1.0);
/// 빛무리 크기 (반지름의 몇 배인 지름) 와 진하기: 넓은 빛, 가운데 빛, 바로 둘레 빛
const SUN_HALO_WIDE: (f32, f32) = (9.0, 0.36);
const SUN_HALO_MID: (f32, f32) = (4.2, 0.45);
const SUN_HALO_NEAR: (f32, f32) = (2.7, 0.75);
/// 노을 때 수평선을 따라 옆으로 길게 번지는 빛 (가로, 세로, 진하기)
const DUSK_BAND: (f32, f32, f32) = (1100.0, 260.0, 0.55);

// ---- 달 ----
pub const MOON_RADIUS: f32 = 30.0;
const MOON_HALO: Color = Color::new(0.85, 0.90, 1.0, 1.0);
const MOON_HALO_WIDE: (f32, f32) = (6.5, 0.16);
const MOON_HALO_NEAR: (f32, f32) = (2.8, 0.28);

// ---- 별 ----
const STAR_COUNT: u32 = 110;
const STAR: Color = Color::new(1.0, 0.98, 0.90, 1.0);
/// 별 반지름: 대부분 작고, 몇 개만 크다
const STAR_RADIUS: (f32, f32) = (0.7, 2.7);
/// 이 반지름보다 큰 별은 십자 반짝
const STAR_CROSS_FROM: f32 = 2.0;

// ---- 구름 ----
const CLOUD_DAY: Color = WHITE;
const CLOUD_DUSK: Color = Color::new(1.0, 0.70, 0.62, 1.0);
const CLOUD_NIGHT: Color = Color::new(0.22, 0.25, 0.40, 1.0);
/// 구름 그림 아래쪽 그늘 색 (그림에 미리 칠해 둔다)
const CLOUD_SHADOW: [f32; 3] = [0.72, 0.78, 0.90];
const CLOUD_KINDS: usize = 3;

/// 구름 한 겹. 먼 겹일수록 작고·흐리고(하늘색이 섞이고)·느리다.
struct CloudLayer {
    /// 구름 그림(256×128)을 몇 배로 그릴지
    scale: f32,
    /// 옆으로 흐르는 빠르기 (픽셀/초)
    speed: f32,
    alpha: f32,
    /// 하늘색이 섞이는 정도 (멀리 있는 느낌)
    haze: f32,
    /// (처음 x, 가운데 y, 구름 모양 번호)
    clouds: &'static [(f32, f32, usize)],
}

const CLOUD_LAYERS: [CloudLayer; 3] = [
    // 먼 구름: 수평선 가까이
    CloudLayer { scale: 0.42, speed: 3.0, alpha: 0.80, haze: 0.45, clouds: &[(60.0, 272.0, 0), (380.0, 286.0, 1), (690.0, 266.0, 2), (930.0, 290.0, 1)] },
    // 가운데 구름
    CloudLayer { scale: 0.70, speed: 7.0, alpha: 0.92, haze: 0.18, clouds: &[(220.0, 200.0, 1), (640.0, 178.0, 2)] },
    // 가까운 구름: 크고 빠르게
    CloudLayer { scale: 1.05, speed: 13.0, alpha: 0.97, haze: 0.0, clouds: &[(40.0, 96.0, 2), (560.0, 74.0, 0)] },
];

// ---- 바다 ----
const SEA_FAR: Color = Color::new(0.09, 0.40, 0.76, 1.0);
const SEA_MID: Color = Color::new(0.14, 0.55, 0.82, 1.0);
const SEA_NEAR: Color = Color::new(0.24, 0.73, 0.85, 1.0);
const NIGHT_SEA_FAR: Color = Color::new(0.03, 0.08, 0.24, 1.0);
const NIGHT_SEA_NEAR: Color = Color::new(0.07, 0.20, 0.38, 1.0);
/// 노을 때 먼 바다에 비치는 주황빛
const SEA_DUSK: Color = Color::new(0.78, 0.44, 0.44, 1.0);
const SEA_DUSK_AMOUNT: f32 = 0.55;
/// 수평선 안개 (낮 / 노을 / 밤) 와 진하기
const HAZE_DAY: Color = Color::new(0.93, 0.97, 1.0, 0.55);
const HAZE_DUSK: Color = Color::new(1.0, 0.74, 0.55, 0.55);
const HAZE_NIGHT: Color = Color::new(0.22, 0.27, 0.48, 0.35);
/// 해 빛 기둥 색 (낮 / 노을)과 달빛 기둥 색
const SUN_PATH_DAY: Color = Color::new(1.0, 0.97, 0.82, 1.0);
const SUN_PATH_SET: Color = Color::new(1.0, 0.62, 0.30, 1.0);
const MOON_PATH: Color = Color::new(0.95, 0.95, 0.85, 1.0);
/// 빛 기둥 세기: 해가 높이 떠 있을 때 / 노을 때 / 달
const SUN_PATH_DAY_STRENGTH: f32 = 0.35;
const SUN_PATH_SET_STRENGTH: f32 = 0.95;
const MOON_PATH_STRENGTH: f32 = 0.75;
/// 바다 여기저기 반짝이는 물결 개수와 색 (낮 / 밤)
const GLITTER_COUNT: u32 = 40;
const GLITTER_DAY: Color = Color::new(1.0, 1.0, 1.0, 0.55);
const GLITTER_NIGHT: Color = Color::new(0.95, 0.93, 0.80, 0.28);

// ---- 해·달 자리 ----

/// 해·달 자리. 메뉴 화면은 오른쪽 위에 해. 플레이는 위쪽에 정보판이 있어서 조금 아래.
pub struct SkyLayout {
    pub sun: (f32, f32),
    pub moon: (f32, f32),
}
pub const MENU_SKY: SkyLayout = SkyLayout { sun: (870.0, 80.0), moon: (150.0, 120.0) };
pub const PLAY_SKY: SkyLayout = SkyLayout { sun: (850.0, 150.0), moon: (170.0, 140.0) };

// ================= 작은 도우미 =================

pub fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t)
}

pub fn with_alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}

/// 밤빛으로 어둡게: `amount` 0 = 그대로, 1 = 푸르스름하게 어두운 밤 색. 투명도는 그대로.
pub fn night(c: Color, amount: f32) -> Color {
    let dark = Color::new(c.r * 0.32, c.g * 0.38, c.b * 0.55 + 0.06, c.a);
    mix(c, dark, amount.clamp(0.0, 1.0))
}

/// 노을·새벽 빛의 세기: 어둠이 반쯤일 때 가장 세고, 한낮·한밤에는 0.
pub fn dusk_glow(darkness: f32) -> f32 {
    let d = darkness.clamp(0.0, 1.0);
    4.0 * d * (1.0 - d)
}

/// `from` 아래로는 0, `to` 위로는 1, 사이는 부드럽게. (`from > to`이면 거꾸로)
pub fn ramp(x: f32, from: f32, to: f32) -> f32 {
    let t = ((x - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// 번호 → 0 이상 1 미만의 늘 같은 "무작위" 수 (별 자리·반짝임 순서용)
fn hash(n: u32) -> f32 {
    let mut x = n.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 16;
    x = x.wrapping_mul(0x7FEB_352D);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846C_A68B);
    x ^= x >> 16;
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// 부드러운 값 노이즈 (0~1): 격자점마다 정해진 수를 부드럽게 이어 붙인다.
fn value_noise(x: f32, y: f32) -> f32 {
    let (xi, yi) = (x.floor(), y.floor());
    let (tx, ty) = (x - xi, y - yi);
    let (sx, sy) = (tx * tx * (3.0 - 2.0 * tx), ty * ty * (3.0 - 2.0 * ty));
    let at = |i: f32, j: f32| hash(((xi + i) as i32 as u32).wrapping_mul(7919) ^ ((yi + j) as i32 as u32).wrapping_mul(104_729));
    lerp(lerp(at(0.0, 0.0), at(1.0, 0.0), sx), lerp(at(0.0, 1.0), at(1.0, 1.0), sx), sy)
}

/// 세 색 그라데이션 (가운데 색은 `mid_at`). 구간 안은 부드럽게 섞어서 꺾인 자국이 없게.
fn three_stops(c: &[Color; 3], t: f32, mid_at: f32) -> Color {
    if t < mid_at {
        mix(c[0], c[1], ramp(t, 0.0, mid_at))
    } else {
        mix(c[1], c[2], ramp(t, mid_at, 1.0))
    }
}

/// 하늘 색: `t` 0 = 맨 위 ~ 1 = 수평선.
fn sky_color(t: f32, d: f32, dusk: f32) -> Color {
    let base = mix(three_stops(&DAY_SKY, t, SKY_MID_AT), three_stops(&NIGHT_SKY, t, SKY_MID_AT), d);
    let warm = three_stops(&DUSK_SKY, t, SKY_MID_AT);
    let amount = dusk * (DUSK_TOP_AMOUNT + (1.0 - DUSK_TOP_AMOUNT) * t.powf(1.4));
    mix(base, warm, amount)
}

/// 가운데가 `c`이고 지름이 `size`인 정사각형
fn square(c: Vec2, size: f32) -> Rect {
    Rect::new(c.x - size / 2.0, c.y - size / 2.0, size, size)
}

// ================= 코드로 만드는 그림 =================

/// 빛 그림: 가운데 흰색, 둘레로 갈수록 자연스럽게 투명해진다 (해·달 빛무리, 별, 반짝임).
const GLOW_SIZE: u16 = 128;
/// 둥근 판 (해 몸통): 가장자리만 살짝 부드럽다. 판 반지름 = 그림 반 크기의 이 비율
const DISC_SIZE: u16 = 128;
const DISC_FILL: f32 = 62.0 / 64.0;
/// 달 그림
const MOON_SIZE: u16 = 128;
const MOON_FILL: f32 = 60.0 / 64.0;
/// 빛 기둥: 위(수평선)는 진하고 아래로 사라지며, 옆으로는 부드럽게
const BEAM_SIZE: (u16, u16) = (64, 128);
/// 구름 그림
const CLOUD_SIZE: (u16, u16) = (256, 128);

/// 어떤 그림인지
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Tex {
    Glow,
    Disc,
    Moon,
    Beam,
    Cloud(usize),
}

/// GPU 없이 만든 그림들 (테스트에서도 만들 수 있다).
pub struct SkyImages {
    pub glow: Image,
    pub disc: Image,
    pub moon: Image,
    pub beam: Image,
    pub clouds: Vec<Image>,
}

fn smoothstep(e0: f32, e1: f32, x: f32) -> f32 {
    ramp(x, e0, e1)
}

/// 픽셀마다 `f(x, y)` → [r, g, b, a] (0~1)로 그림을 만든다. x, y는 픽셀 가운데 좌표.
fn make_image(w: u16, h: u16, f: impl Fn(f32, f32) -> [f32; 4]) -> Image {
    let mut img = Image::gen_image_color(w, h, Color::new(1.0, 1.0, 1.0, 0.0));
    for y in 0..h as usize {
        for x in 0..w as usize {
            let p = f(x as f32 + 0.5, y as f32 + 0.5);
            let i = (y * w as usize + x) * 4;
            for k in 0..4 {
                img.bytes[i + k] = (p[k].clamp(0.0, 1.0) * 255.0 + 0.5) as u8;
            }
        }
    }
    img
}

/// 가우스처럼 부드럽게 줄다가 r = 1에서 딱 0이 되는 곡선
fn soft_falloff(r: f32, sharpness: f32) -> f32 {
    if r >= 1.0 {
        return 0.0;
    }
    let edge = (-sharpness).exp();
    (((-sharpness * r * r).exp()) - edge) / (1.0 - edge)
}

fn gen_glow() -> Image {
    let half = GLOW_SIZE as f32 / 2.0;
    make_image(GLOW_SIZE, GLOW_SIZE, |x, y| {
        let r = ((x - half).powi(2) + (y - half).powi(2)).sqrt() / half;
        [1.0, 1.0, 1.0, soft_falloff(r, 4.5)]
    })
}

fn gen_disc() -> Image {
    let half = DISC_SIZE as f32 / 2.0;
    let radius = half * DISC_FILL;
    make_image(DISC_SIZE, DISC_SIZE, |x, y| {
        let dist = ((x - half).powi(2) + (y - half).powi(2)).sqrt();
        let n = (dist / radius).min(1.0);
        // 가장자리가 조금 어두운 둥근 공 느낌 (색은 그릴 때 곱한다)
        let shade = 1.0 - 0.10 * n * n;
        let alpha = (radius - dist + 0.75).clamp(0.0, 1.5) / 1.5;
        [shade, shade, shade, alpha]
    })
}

/// 달: 크림색 공 + 부드러운 회색 무늬(바다·분화구) + 둘레가 살짝 어둡게.
fn gen_moon() -> Image {
    let half = MOON_SIZE as f32 / 2.0;
    let radius = half * MOON_FILL;
    // 큰 무늬 (x, y, 크기, 진하기). 달 반지름 = 1 기준
    const MARIA: [(f32, f32, f32, f32); 6] = [
        (-0.28, -0.22, 0.34, 0.26),
        (0.18, 0.12, 0.28, 0.22),
        (0.02, -0.40, 0.20, 0.15),
        (-0.12, 0.40, 0.22, 0.15),
        (0.42, -0.18, 0.16, 0.12),
        (-0.48, 0.18, 0.14, 0.10),
    ];
    // 작은 분화구 (x, y, 크기)
    const CRATERS: [(f32, f32, f32); 6] =
        [(0.35, 0.42, 0.10), (-0.40, -0.48, 0.08), (0.55, 0.15, 0.07), (-0.05, 0.05, 0.06), (0.15, -0.62, 0.06), (-0.62, -0.05, 0.07)];
    make_image(MOON_SIZE, MOON_SIZE, |x, y| {
        let (dx, dy) = ((x - half) / radius, (y - half) / radius);
        let r2 = dx * dx + dy * dy;
        let dist = r2.sqrt() * radius;
        let alpha = (radius - dist + 0.75).clamp(0.0, 1.5) / 1.5;
        let mut dark = 0.0;
        for (mx, my, s, k) in MARIA {
            let q = ((dx - mx).powi(2) + (dy - my).powi(2)) / (s * s);
            dark += k * (-q * 1.6).exp();
        }
        let mut light = 0.0;
        for (cx, cy, s) in CRATERS {
            let q = ((dx - cx).powi(2) + (dy - cy).powi(2)) / (s * s);
            dark += 0.10 * (-q * 2.5).exp();
            // 오른쪽 아래 테두리가 살짝 밝다
            let q2 = ((dx - cx - s * 0.45).powi(2) + (dy - cy - s * 0.45).powi(2)) / (s * s * 0.5);
            light += 0.06 * (-q2 * 2.0).exp();
        }
        // 잔잔한 얼룩 (부드러운 값 노이즈 두 겹)
        let grain = 0.07 * (value_noise(dx * 5.0 + 3.0, dy * 5.0 + 7.0) - 0.5)
            + 0.04 * (value_noise(dx * 11.0 + 1.0, dy * 11.0 + 2.0) - 0.5);
        let limb = 1.0 - 0.22 * r2.min(1.0).powf(1.5);
        let v = ((1.0 - dark + light + grain) * limb).clamp(0.0, 1.0);
        [v * 1.0, v * 0.97, v * 0.88, alpha]
    })
}

fn gen_beam() -> Image {
    let (w, h) = (BEAM_SIZE.0 as f32, BEAM_SIZE.1 as f32);
    make_image(BEAM_SIZE.0, BEAM_SIZE.1, |x, y| {
        let side = soft_falloff(((x - w / 2.0) / (w / 2.0)).abs(), 3.0);
        let down = (1.0 - y / h).powf(1.6);
        [1.0, 1.0, 1.0, side * down]
    })
}

/// 구름: 둥근 덩어리 여러 개의 "밀도"를 더해 부드러운 테두리를 만들고, 아래쪽은 평평하게 그늘지게.
fn gen_cloud(kind: usize) -> Image {
    let (w, h) = (CLOUD_SIZE.0 as f32, CLOUD_SIZE.1 as f32);
    let seed = kind as u32 * 101 + 7;
    let base_y = h * 0.74;
    let mut blobs: Vec<(f32, f32, f32)> = Vec::new();
    let n = 7 + kind % 3;
    for i in 0..n {
        let t = i as f32 / (n - 1) as f32;
        let bell = (t * std::f32::consts::PI).sin();
        let r = 20.0 + 22.0 * bell.powf(0.7) + hash(seed + i as u32 * 3) * 5.0;
        let cx = 46.0 + (w - 92.0) * t + (hash(seed + i as u32 * 3 + 1) - 0.5) * 8.0;
        let cy = base_y - r * 0.5 + (hash(seed + i as u32 * 3 + 2) - 0.5) * 6.0;
        blobs.push((cx, cy, r));
    }
    // 위쪽 뭉게뭉게
    for j in 0..3 {
        let cx = w * (0.36 + 0.14 * j as f32) + (hash(seed + 50 + j) - 0.5) * 20.0;
        let r = 20.0 + hash(seed + 60 + j) * 12.0;
        blobs.push((cx, base_y - 30.0 - r * 0.6, r));
    }
    let top_y = blobs.iter().map(|b| b.1 - b.2).fold(h, f32::min);
    let density = |x: f32, y: f32| -> f32 {
        let mut f = 0.0;
        for &(cx, cy, r) in &blobs {
            let q = ((x - cx).powi(2) + (y - cy).powi(2)) / (r * r);
            if q < 1.0 {
                f += (1.0 - q) * (1.0 - q);
            }
        }
        // 바닥은 평평하게 잘리되 부드럽게
        f * smoothstep(base_y + 10.0, base_y - 6.0, y)
    };
    // 1) 투명도: 밀도로 부드러운 테두리 → 2) 한 번 더 흐리게
    let (wi, hi) = (CLOUD_SIZE.0 as usize, CLOUD_SIZE.1 as usize);
    let mut alpha = vec![0.0f32; wi * hi];
    for y in 0..hi {
        for x in 0..wi {
            alpha[y * wi + x] = smoothstep(0.18, 0.85, density(x as f32 + 0.5, y as f32 + 0.5));
        }
    }
    for _ in 0..2 {
        alpha = box_blur(&alpha, wi, hi);
    }
    make_image(CLOUD_SIZE.0, CLOUD_SIZE.1, |x, y| {
        let a = alpha[(y as usize) * wi + x as usize];
        // 아래로 갈수록 그늘, 위쪽은 하얗게
        let t = ((y - top_y) / (base_y - top_y)).clamp(0.0, 1.0);
        let shade = smoothstep(0.35, 1.05, t) * 0.9;
        let c = |k: usize| lerp(1.0, CLOUD_SHADOW[k], shade);
        // 가장자리 바깥(투명한 곳)도 색을 채워 두어 늘여 그릴 때 테두리가 거뭇해지지 않게
        [c(0), c(1), c(2), a]
    })
}

fn box_blur(src: &[f32], w: usize, h: usize) -> Vec<f32> {
    let mut out = vec![0.0; src.len()];
    for y in 0..h {
        for x in 0..w {
            let mut sum = 0.0;
            let mut n = 0.0;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let (xx, yy) = (x as i32 + dx, y as i32 + dy);
                    if xx >= 0 && yy >= 0 && (xx as usize) < w && (yy as usize) < h {
                        sum += src[yy as usize * w + xx as usize];
                        n += 1.0;
                    }
                }
            }
            out[y * w + x] = sum / n;
        }
    }
    out
}

impl SkyImages {
    pub fn generate() -> SkyImages {
        SkyImages {
            glow: gen_glow(),
            disc: gen_disc(),
            moon: gen_moon(),
            beam: gen_beam(),
            clouds: (0..CLOUD_KINDS).map(gen_cloud).collect(),
        }
    }

    #[cfg(test)]
    pub fn get(&self, tex: Tex) -> &Image {
        match tex {
            Tex::Glow => &self.glow,
            Tex::Disc => &self.disc,
            Tex::Moon => &self.moon,
            Tex::Beam => &self.beam,
            Tex::Cloud(i) => &self.clouds[i % self.clouds.len()],
        }
    }
}

/// GPU에 올린 하늘 그림. 게임 시작 때 한 번만 만든다.
/// 아이·엄마 그림으로 바꾸려면 이 텍스처를 `load_texture`로 불러온 그림으로 바꿔 끼우면 된다.
pub struct SkyArt {
    pub glow: Texture2D,
    pub sun_disc: Texture2D,
    pub moon: Texture2D,
    pub beam: Texture2D,
    pub clouds: Vec<Texture2D>,
}

fn to_texture(img: &Image) -> Texture2D {
    let t = Texture2D::from_image(img);
    t.set_filter(FilterMode::Linear);
    t
}

impl SkyArt {
    pub fn new() -> SkyArt {
        let images = SkyImages::generate();
        SkyArt {
            glow: to_texture(&images.glow),
            sun_disc: to_texture(&images.disc),
            moon: to_texture(&images.moon),
            beam: to_texture(&images.beam),
            clouds: images.clouds.iter().map(to_texture).collect(),
        }
    }

    fn get(&self, tex: Tex) -> &Texture2D {
        match tex {
            Tex::Glow => &self.glow,
            Tex::Disc => &self.sun_disc,
            Tex::Moon => &self.moon,
            Tex::Beam => &self.beam,
            Tex::Cloud(i) => &self.clouds[i % self.clouds.len()],
        }
    }
}

// ================= 그리는 곳 =================

/// 그리는 곳. 게임은 GPU, 테스트는 CPU 그림판.
pub trait Canvas {
    /// 가로 `x`~`x + w`에 세로 그라데이션. `stops`는 (y, 색), y가 커지는 순서.
    fn vgradient(&mut self, x: f32, w: f32, stops: &[(f32, Color)]);
    /// 그림을 `dest` 네모에 늘여서, 색 `tint`를 곱해 그린다.
    fn image(&mut self, tex: Tex, dest: Rect, tint: Color);
}

pub struct GpuCanvas<'a> {
    pub art: &'a SkyArt,
}

impl Canvas for GpuCanvas<'_> {
    fn vgradient(&mut self, x: f32, w: f32, stops: &[(f32, Color)]) {
        if stops.len() < 2 {
            return;
        }
        // 꼭짓점 색 그라데이션: GPU가 픽셀마다 부드럽게 섞는다 (띠 없음).
        let mut vertices = Vec::with_capacity(stops.len() * 2);
        let mut indices = Vec::with_capacity((stops.len() - 1) * 6);
        for (i, &(y, c)) in stops.iter().enumerate() {
            vertices.push(Vertex::new(x, y, 0.0, 0.0, 0.0, c));
            vertices.push(Vertex::new(x + w, y, 0.0, 1.0, 0.0, c));
            if i > 0 {
                let b = (i as u16 - 1) * 2;
                indices.extend_from_slice(&[b, b + 1, b + 2, b + 1, b + 3, b + 2]);
            }
        }
        draw_mesh(&Mesh { vertices, indices, texture: None });
    }

    fn image(&mut self, tex: Tex, dest: Rect, tint: Color) {
        if tint.a <= 0.002 || dest.w <= 0.0 || dest.h <= 0.0 {
            return;
        }
        draw_texture_ex(
            self.art.get(tex),
            dest.x,
            dest.y,
            tint,
            DrawTextureParams { dest_size: Some(vec2(dest.w, dest.h)), ..Default::default() },
        );
    }
}

// ================= 이번 프레임의 하늘 =================

/// 해나 달 하나
#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub pos: Vec2,
    pub radius: f32,
    /// 노을 정도 (해: 0 한낮 ~ 1 노을) / 보이는 정도 (달)
    pub amount: f32,
}

impl Body {
    /// 수평선 위로 얼마나 나와 있는지 (1 = 다 나옴, 0 = 다 잠김)
    pub fn above_horizon(&self) -> f32 {
        ((HORIZON_Y - (self.pos.y - self.radius)) / (self.radius * 2.0)).clamp(0.0, 1.0)
    }
}

/// `sky_darkness`와 시간으로 정한 이번 프레임의 하늘 모습.
pub struct SkyLook {
    pub d: f32,
    pub dusk: f32,
    pub time: f32,
    pub sun: Option<Body>,
    pub moon: Option<Body>,
}

impl SkyLook {
    pub fn new(darkness: f32, layout: &SkyLayout, time: f32) -> SkyLook {
        let d = darkness.clamp(0.0, 1.0);
        // 해: 어두워질수록 주황빛으로 커지며 수평선 아래로 (바다가 나중에 그려져서 가려진다).
        let sun = (d < 1.0).then(|| {
            let set = ramp(d, 0.05, 0.6);
            let radius = SUN_RADIUS * (1.0 + SUNSET_GROW * set);
            let sink = ramp(d, 0.0, 0.85);
            let y = lerp(layout.sun.1, HORIZON_Y + radius * 1.8, sink);
            Body { pos: vec2(layout.sun.0, y), radius, amount: set }
        });
        // 보름달: 해가 지는 동안 수평선에서 떠오르고, 새벽에 다시 내려가며 사라진다.
        let moon_alpha = ramp(d, 0.3, 0.9);
        let moon = (moon_alpha > 0.0).then(|| {
            let rise = ramp(d, 0.2, 1.0);
            let y = lerp(HORIZON_Y + MOON_RADIUS * 2.0, layout.moon.1, rise);
            Body { pos: vec2(layout.moon.0, y), radius: MOON_RADIUS, amount: moon_alpha }
        });
        SkyLook { d, dusk: dusk_glow(d), time, sun, moon }
    }

    fn sky_at(&self, y: f32) -> Color {
        sky_color((y / HORIZON_Y).clamp(0.0, 1.0), self.d, self.dusk)
    }
}

// ================= 하늘 그리기 =================

/// 하늘 전체: 그라데이션 → 노을빛 → 별 → 달 → 해 → 구름 세 겹 (먼 것부터).
pub fn draw_sky(c: &mut impl Canvas, look: &SkyLook) {
    let stops: Vec<(f32, Color)> = (0..=SKY_ROWS)
        .map(|i| {
            let t = i as f32 / SKY_ROWS as f32;
            (t * HORIZON_Y, sky_color(t, look.d, look.dusk))
        })
        .collect();
    c.vgradient(0.0, VIRTUAL_WIDTH, &stops);

    // 노을·새벽: 해 쪽 수평선을 따라 따뜻한 빛이 옆으로 길게 번진다
    if look.dusk > 0.0 {
        let x = look.sun.map(|s| s.pos.x).unwrap_or(VIRTUAL_WIDTH * 0.75);
        let (w, h, a) = DUSK_BAND;
        let tint = with_alpha(DUSK_SKY[2], a * look.dusk);
        c.image(Tex::Glow, Rect::new(x - w / 2.0, HORIZON_Y - h / 2.0, w, h), tint);
    }

    draw_stars(c, look);
    if let Some(moon) = look.moon {
        draw_moon(c, &moon);
    }
    if let Some(sun) = look.sun {
        draw_sun(c, &sun, look.d);
    }
    for layer in &CLOUD_LAYERS {
        for &(x0, y, kind) in layer.clouds {
            let w = CLOUD_SIZE.0 as f32 * layer.scale;
            let h = CLOUD_SIZE.1 as f32 * layer.scale;
            let x = (x0 + look.time * layer.speed).rem_euclid(VIRTUAL_WIDTH + w) - w;
            draw_cloud(c, kind, Rect::new(x, y - h / 2.0, w, h), cloud_tint(look, layer, y));
        }
    }
}

/// 구름 색: 낮엔 흰색, 노을엔 분홍·주황빛, 밤엔 어두운 남회색. 먼 구름일수록 하늘색이 섞인다.
fn cloud_tint(look: &SkyLook, layer: &CloudLayer, y: f32) -> Color {
    let base = mix(mix(CLOUD_DAY, CLOUD_NIGHT, look.d), CLOUD_DUSK, look.dusk * 0.6);
    let c = mix(base, look.sky_at(y), layer.haze);
    with_alpha(c, layer.alpha * (1.0 - 0.15 * look.d))
}

/// 구름 하나. `kind`번 구름 그림을 `dest`에 그린다.
pub fn draw_cloud(c: &mut impl Canvas, kind: usize, dest: Rect, tint: Color) {
    c.image(Tex::Cloud(kind), dest, tint);
}

/// 해: 넓은 빛무리 → 가운데 빛 → 둘레 빛 → 몸통(가장자리 따뜻한 색) → 하얗게 빛나는 가운데.
pub fn draw_sun(c: &mut impl Canvas, sun: &Body, darkness: f32) {
    let set = sun.amount;
    let r = sun.radius;
    let p = sun.pos;
    // 해가 수평선에 잠길수록, 밤이 될수록 빛무리도 사라진다
    let fade = lerp(0.25, 1.0, sun.above_horizon()) * (1.0 - ramp(darkness, 0.6, 1.0));
    let halo = mix(SUN_HALO_DAY, SUN_HALO_SET, set);
    for (size, alpha) in [SUN_HALO_WIDE, SUN_HALO_MID, SUN_HALO_NEAR] {
        // 노을 때는 빛무리가 더 크고 진하다
        let size = size * (1.0 + 0.25 * set);
        let alpha = alpha * (1.0 + 0.3 * set) * fade;
        c.image(Tex::Glow, square(p, r * size), with_alpha(halo, alpha.min(1.0)));
    }
    c.image(Tex::Disc, square(p, r * 2.0 / DISC_FILL), mix(SUN_EDGE_DAY, SUN_EDGE_SET, set));
    c.image(Tex::Glow, square(p, r * 1.7), mix(SUN_CORE_DAY, SUN_CORE_SET, set));
}

/// 달: 은은한 빛무리 두 겹 → 무늬 있는 달 그림.
pub fn draw_moon(c: &mut impl Canvas, moon: &Body) {
    let a = moon.amount;
    let r = moon.radius;
    for (size, alpha) in [MOON_HALO_WIDE, MOON_HALO_NEAR] {
        c.image(Tex::Glow, square(moon.pos, r * size), with_alpha(MOON_HALO, alpha * a));
    }
    c.image(Tex::Moon, square(moon.pos, r * 2.0 / MOON_FILL), with_alpha(WHITE, a));
}

/// 별: 자리·크기·밝기는 번호로 정해져 늘 같고, 하나하나 다른 빠르기로 부드럽게 반짝인다.
/// 수평선 가까이는 흐리게.
fn draw_stars(c: &mut impl Canvas, look: &SkyLook) {
    let alpha = ramp(look.d, 0.45, 0.95);
    if alpha <= 0.0 {
        return;
    }
    for i in 0..STAR_COUNT {
        let x = hash(i * 5) * VIRTUAL_WIDTH;
        let y = 14.0 + hash(i * 5 + 1) * (HORIZON_Y - 60.0);
        let size = hash(i * 5 + 2).powi(3);
        let r = lerp(STAR_RADIUS.0, STAR_RADIUS.1, size);
        let bright = 0.45 + 0.55 * hash(i * 5 + 3);
        let speed = 0.8 + 1.6 * hash(i * 5 + 4);
        let twinkle = 0.72 + 0.28 * (look.time * speed + i as f32 * 1.7).sin();
        let low = 1.0 - 0.7 * ramp(y, HORIZON_Y - 140.0, HORIZON_Y - 30.0);
        let a = alpha * bright * twinkle * low;
        c.image(Tex::Glow, square(vec2(x, y), r * 5.0), with_alpha(STAR, a));
        if r > STAR_CROSS_FROM {
            let len = r * 10.0 * (0.8 + 0.2 * twinkle);
            let tint = with_alpha(STAR, a * 0.6);
            c.image(Tex::Glow, Rect::new(x - len / 2.0, y - 1.0, len, 2.0), tint);
            c.image(Tex::Glow, Rect::new(x - 1.0, y - len / 2.0, 2.0, len), tint);
        }
    }
}

// ================= 바다 그리기 =================

/// 바다: 그라데이션 → 수평선 안개 → 해·달 빛 기둥 → 여기저기 반짝이는 물결.
/// (하얀 물결 거품과 모래는 screens.rs)
pub fn draw_sea(c: &mut impl Canvas, look: &SkyLook, ground_y: f32) {
    let d = look.d;
    let warm = look.dusk * SEA_DUSK_AMOUNT;
    let far = mix(mix(SEA_FAR, NIGHT_SEA_FAR, d), SEA_DUSK, warm);
    let mid = mix(mix(SEA_MID, NIGHT_SEA_FAR, d), SEA_DUSK, warm * 0.5);
    let near = mix(mix(SEA_NEAR, NIGHT_SEA_NEAR, d), SEA_DUSK, warm * 0.2);
    let sea_h = ground_y - HORIZON_Y;
    c.vgradient(0.0, VIRTUAL_WIDTH, &[(HORIZON_Y, far), (HORIZON_Y + sea_h * 0.35, mid), (ground_y + 2.0, near)]);

    // 수평선 안개: 하늘 아래와 먼 바다를 같이 흐리게 덮는다
    let haze = mix(mix(HAZE_DAY, HAZE_NIGHT, d), HAZE_DUSK, look.dusk);
    let clear = with_alpha(haze, 0.0);
    c.vgradient(
        0.0,
        VIRTUAL_WIDTH,
        &[
            (HORIZON_Y - 34.0, clear),
            (HORIZON_Y, haze),
            (HORIZON_Y + 10.0, with_alpha(haze, haze.a * 0.35)),
            (HORIZON_Y + 32.0, clear),
        ],
    );

    if let Some(sun) = look.sun {
        let strength = sun.above_horizon() * lerp(SUN_PATH_DAY_STRENGTH, SUN_PATH_SET_STRENGTH, sun.amount);
        draw_light_path(c, &sun, mix(SUN_PATH_DAY, SUN_PATH_SET, sun.amount), strength, look.time, ground_y);
    }
    if let Some(moon) = look.moon {
        let strength = moon.amount * moon.above_horizon() * MOON_PATH_STRENGTH;
        draw_light_path(c, &moon, MOON_PATH, strength, look.time + 40.0, ground_y);
    }

    // 여기저기 반짝이는 물결: 나타났다 사라졌다 한다. 가까울수록 길게.
    let glitter = mix(GLITTER_DAY, GLITTER_NIGHT, d);
    for i in 0..GLITTER_COUNT {
        let depth = hash(i * 7 + 1).powf(0.8);
        let y = HORIZON_Y + 6.0 + depth * (sea_h - 16.0);
        let x = hash(i * 7) * VIRTUAL_WIDTH + (look.time * 0.6 + i as f32).sin() * 6.0;
        let pulse = ((look.time * (0.6 + hash(i * 7 + 2)) + hash(i * 7 + 3) * 6.3).sin()).max(0.0).powi(2);
        let len = (10.0 + hash(i * 7 + 4) * 16.0) * (0.5 + depth);
        c.image(Tex::Glow, Rect::new(x - len / 2.0, y - 1.5, len, 3.0), with_alpha(glitter, glitter.a * pulse));
    }
}

/// 해·달 아래 바다에 비치는 빛 기둥: 수평선에서 진하고 아래로 갈수록 넓어지며 흐려지는
/// 부드러운 빛 + 흔들리는 짧은 빛 줄.
fn draw_light_path(c: &mut impl Canvas, body: &Body, color: Color, strength: f32, time: f32, ground_y: f32) {
    if strength <= 0.01 {
        return;
    }
    let sea_h = ground_y - HORIZON_Y;
    let x = body.pos.x;
    let w = body.radius * 2.6;
    c.image(Tex::Beam, Rect::new(x - w / 2.0, HORIZON_Y, w, sea_h), with_alpha(color, 0.45 * strength));
    let mut y = HORIZON_Y + 3.0;
    let mut row = 0u32;
    while y < ground_y - 6.0 {
        let depth = (y - HORIZON_Y) / sea_h;
        let half = body.radius * 0.5 + depth * body.radius * 1.4;
        for k in 0..2u32 {
            let seed = row * 2 + k;
            let off = (hash(seed * 3 + 11) - 0.5) * 2.0 * half + (time * (0.7 + hash(seed)) + seed as f32).sin() * 5.0;
            let len = (8.0 + hash(seed * 3 + 12) * 22.0) * (0.6 + depth);
            let twinkle = 0.45 + 0.55 * (time * (1.2 + hash(seed * 3 + 13)) + seed as f32 * 2.1).sin().abs();
            let a = strength * (1.0 - 0.55 * depth) * twinkle;
            c.image(Tex::Glow, Rect::new(x + off - len / 2.0, y - 1.5, len, 3.0), with_alpha(color, a.min(1.0)));
        }
        y += 4.0 + depth * 3.0;
        row += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sky_is_plain_day_at_zero_and_navy_at_one() {
        // 한낮·한밤에는 노을빛이 섞이지 않는다 → 새벽이 끝나면 낮 하늘과 똑같다.
        assert_eq!(dusk_glow(0.0), 0.0);
        assert_eq!(dusk_glow(1.0), 0.0);
        assert!(dusk_glow(0.5) > 0.9);
        assert_eq!(sky_color(0.0, 0.0, 0.0), DAY_SKY[0]);
        let n = sky_color(1.0, 1.0, 0.0);
        let close = |a: f32, b: f32| (a - b).abs() < 1e-5;
        assert!(close(n.r, NIGHT_SKY[2].r) && close(n.g, NIGHT_SKY[2].g) && close(n.b, NIGHT_SKY[2].b), "{n:?}");
        assert_eq!(night(WHITE, 0.0), WHITE);
    }

    #[test]
    fn sky_gradient_has_no_jumps() {
        // 띠가 보이지 않게: 바로 옆 줄끼리 색 차이가 아주 작다 (어둠·노을 어느 때나).
        for d in [0.0, 0.3, 0.5, 0.8, 1.0] {
            let dusk = dusk_glow(d);
            let mut prev = sky_color(0.0, d, dusk);
            for i in 1..=540 {
                let c = sky_color(i as f32 / 540.0, d, dusk);
                let jump = (c.r - prev.r).abs().max((c.g - prev.g).abs()).max((c.b - prev.b).abs());
                assert!(jump < 0.02, "d={d} i={i} jump={jump}");
                prev = c;
            }
        }
    }

    #[test]
    fn sun_sets_fully_below_horizon_at_night() {
        // 해가 다 내려가면 바다에 가려진다 (커진 노을 해도).
        let look = SkyLook::new(0.9, &PLAY_SKY, 0.0);
        let sun = look.sun.expect("아직 해가 있다");
        assert!(sun.pos.y - sun.radius > HORIZON_Y);
        assert_eq!(sun.above_horizon(), 0.0);
        assert!(SkyLook::new(1.0, &PLAY_SKY, 0.0).sun.is_none());
        // 한낮엔 달이 없고 해는 다 떠 있다
        let noon = SkyLook::new(0.0, &PLAY_SKY, 0.0);
        assert!(noon.moon.is_none());
        assert_eq!(noon.sun.unwrap().above_horizon(), 1.0);
        // 한밤 달은 다 떠 있다
        assert_eq!(SkyLook::new(1.0, &PLAY_SKY, 0.0).moon.unwrap().above_horizon(), 1.0);
        // 플레이 화면 해·달은 정보판(맨 위 약 70픽셀) 아래.
        assert!(PLAY_SKY.sun.1 - SUN_RADIUS > 70.0);
        assert!(PLAY_SKY.moon.1 - MOON_RADIUS > 70.0);
    }

    #[test]
    fn generated_images_fade_out_at_edges() {
        // 늘여 그릴 때 네모 테두리가 보이지 않도록 그림 가장자리는 투명해야 한다.
        let images = SkyImages::generate();
        for tex in [Tex::Glow, Tex::Disc, Tex::Moon, Tex::Cloud(0), Tex::Cloud(1), Tex::Cloud(2)] {
            let img = images.get(tex);
            let (w, h) = (img.width as usize, img.height as usize);
            let a = |x: usize, y: usize| img.bytes[(y * w + x) * 4 + 3];
            for x in 0..w {
                assert!(a(x, 0) <= 2 && a(x, h - 1) <= 2, "{tex:?} 위아래 테두리");
            }
            for y in 0..h {
                assert!(a(0, y) <= 2 && a(w - 1, y) <= 2, "{tex:?} 옆 테두리");
            }
            // 가운데는 보인다
            let visible = (0..w * h).filter(|i| img.bytes[i * 4 + 3] > 128).count();
            assert!(visible * 10 > w * h, "{tex:?} 보이는 부분이 너무 적다");
        }
    }

    #[test]
    fn clouds_are_softer_at_edge_than_inside() {
        // 단색 덩어리가 아니라 테두리에 반투명 픽셀이 충분히 있다.
        let images = SkyImages::generate();
        for img in &images.clouds {
            let n = (img.width as usize) * (img.height as usize);
            let soft = (0..n).filter(|i| (20..235).contains(&img.bytes[i * 4 + 3])).count();
            let solid = (0..n).filter(|i| img.bytes[i * 4 + 3] >= 235).count();
            assert!(soft * 8 > solid, "soft {soft} solid {solid}");
        }
    }

    #[test]
    fn far_clouds_are_smaller_and_slower() {
        for pair in CLOUD_LAYERS.windows(2) {
            assert!(pair[0].scale < pair[1].scale && pair[0].speed < pair[1].speed && pair[0].haze > pair[1].haze);
        }
    }

    // ---------- CPU 그림판: 확인용 PNG ----------

    struct CpuCanvas<'a> {
        images: &'a SkyImages,
        w: usize,
        h: usize,
        px: Vec<[f32; 3]>,
    }

    impl CpuCanvas<'_> {
        fn blend(&mut self, x: usize, y: usize, c: [f32; 3], a: f32) {
            let p = &mut self.px[y * self.w + x];
            for k in 0..3 {
                p[k] = p[k] * (1.0 - a) + c[k] * a;
            }
        }
    }

    impl Canvas for CpuCanvas<'_> {
        fn vgradient(&mut self, x0: f32, w: f32, stops: &[(f32, Color)]) {
            for y in 0..self.h {
                let fy = y as f32 + 0.5;
                if fy < stops[0].0 || fy > stops[stops.len() - 1].0 {
                    continue;
                }
                let i = stops.windows(2).position(|s| fy >= s[0].0 && fy <= s[1].0).unwrap_or(0);
                let (a, b) = (stops[i], stops[i + 1]);
                let c = mix(a.1, b.1, ((fy - a.0) / (b.0 - a.0).max(0.001)).clamp(0.0, 1.0));
                for x in (x0.max(0.0) as usize)..((x0 + w).min(self.w as f32) as usize) {
                    self.blend(x, y, [c.r, c.g, c.b], c.a);
                }
            }
        }

        fn image(&mut self, tex: Tex, dest: Rect, tint: Color) {
            let img = self.images.get(tex);
            let (iw, ih) = (img.width as usize, img.height as usize);
            let sample = |u: f32, v: f32, k: usize| -> f32 {
                // 쌍선형 (가장자리는 붙잡기)
                let fx = (u * iw as f32 - 0.5).clamp(0.0, iw as f32 - 1.0);
                let fy = (v * ih as f32 - 0.5).clamp(0.0, ih as f32 - 1.0);
                let (x0, y0) = (fx as usize, fy as usize);
                let (x1, y1) = ((x0 + 1).min(iw - 1), (y0 + 1).min(ih - 1));
                let (tx, ty) = (fx - x0 as f32, fy - y0 as f32);
                let g = |x: usize, y: usize| img.bytes[(y * iw + x) * 4 + k] as f32 / 255.0;
                lerp(lerp(g(x0, y0), g(x1, y0), tx), lerp(g(x0, y1), g(x1, y1), tx), ty)
            };
            let xs = dest.x.max(0.0) as usize..((dest.x + dest.w).min(self.w as f32).max(0.0) as usize);
            let ys = dest.y.max(0.0) as usize..((dest.y + dest.h).min(self.h as f32).max(0.0) as usize);
            for y in ys {
                for x in xs.clone() {
                    let u = (x as f32 + 0.5 - dest.x) / dest.w;
                    let v = (y as f32 + 0.5 - dest.y) / dest.h;
                    let a = sample(u, v, 3) * tint.a;
                    if a > 0.0 {
                        let c = [sample(u, v, 0) * tint.r, sample(u, v, 1) * tint.g, sample(u, v, 2) * tint.b];
                        self.blend(x, y, c, a.min(1.0));
                    }
                }
            }
        }
    }

    /// 확인용: 하늘·바다를 CPU로 그려 PNG로 저장한다.
    /// `SKY_PREVIEW_DIR=폴더 cargo test sky_preview -- --ignored`
    #[test]
    #[ignore]
    fn export_sky_preview() {
        let dir = std::env::var("SKY_PREVIEW_DIR").expect("SKY_PREVIEW_DIR");
        let images = SkyImages::generate();
        let ground_y = crate::config::GROUND_Y;
        for (name, d, layout) in [
            ("menu", 0.0, &MENU_SKY),
            ("day", 0.0, &PLAY_SKY),
            ("dusk_early", 0.3, &PLAY_SKY),
            ("dusk", 0.5, &PLAY_SKY),
            ("dusk_late", 0.7, &PLAY_SKY),
            ("night", 1.0, &PLAY_SKY),
        ] {
            let (w, h) = (960usize, 540usize);
            let mut c = CpuCanvas { images: &images, w, h, px: vec![[0.9, 0.76, 0.52]; w * h] };
            let look = SkyLook::new(d, layout, 3.0);
            draw_sky(&mut c, &look);
            draw_sea(&mut c, &look, ground_y);
            // export_png는 위아래를 뒤집어 저장하므로 미리 뒤집는다.
            let mut img = Image::gen_image_color(w as u16, h as u16, BLACK);
            for y in 0..h {
                for x in 0..w {
                    let p = c.px[y * w + x];
                    let i = ((h - 1 - y) * w + x) * 4;
                    for k in 0..3 {
                        img.bytes[i + k] = (p[k].clamp(0.0, 1.0) * 255.0) as u8;
                    }
                }
            }
            img.export_png(&format!("{dir}/sky_{name}.png"));
        }
        for (name, tex) in [("moon", Tex::Moon), ("cloud0", Tex::Cloud(0)), ("cloud1", Tex::Cloud(1)), ("cloud2", Tex::Cloud(2))] {
            let src = images.get(tex);
            let (w, h) = (src.width as usize, src.height as usize);
            let mut img = Image::gen_image_color(w as u16, h as u16, BLACK);
            for y in 0..h {
                for x in 0..w {
                    let s = (y * w + x) * 4;
                    let a = src.bytes[s + 3] as f32 / 255.0;
                    let i = ((h - 1 - y) * w + x) * 4;
                    // 하늘색 바탕 위에 올려서 본다
                    for (k, bg) in [0.45f32, 0.72, 0.95].into_iter().enumerate() {
                        img.bytes[i + k] = ((src.bytes[s + k] as f32 / 255.0 * a + bg * (1.0 - a)) * 255.0) as u8;
                    }
                }
            }
            img.export_png(&format!("{dir}/tex_{name}.png"));
        }
    }
}
