//! 화면 그리기: 타이틀, 캐릭터 고르기, 플레이. 좌표는 모두 가상 화면(960×540) 기준.
//! 배경은 해변: 하늘(해·구름) → 먼 바다(수평선) → 하얀 물결 → 모래. 바다는 그림일 뿐 규칙은 없다.
//! 플레이 중에는 logic 시계의 `sky_darkness`(0 한낮 ~ 1 한밤)로 낮 그림과 밤 그림을 부드럽게 섞는다
//! (D9, D10): 노을 때 하늘 아래가 주황·분홍, 해는 수평선 아래로, 밤에는 남색 하늘에 별과 달.

use super::assets::Assets;
use super::coin::{draw_coins, CoinPops};
use super::hud::draw_hud;
use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::sprites::{self, draw_character, max_half_width, PLAY_SPRITE_HEIGHT};
use super::text::TextPainter;
use crate::logic::character::CharacterKind;
use crate::logic::game::Game;
use crate::logic::player::{Facing, Motion};
use macroquad::prelude::*;

// ---- 색 ----
const SKY_TOP: Color = Color::new(0.45, 0.76, 0.98, 1.0);
const SKY_BOTTOM: Color = Color::new(0.76, 0.91, 1.0, 1.0);
const CLOUD: Color = Color::new(1.0, 1.0, 1.0, 0.9);
const SUN: Color = Color::new(1.0, 0.86, 0.30, 1.0);
const SUN_GLOW: Color = Color::new(1.0, 0.95, 0.60, 0.35);
/// 먼 바다(수평선 쪽)는 진하고, 모래 쪽으로 올수록 밝은 청록
const SEA_FAR: Color = Color::new(0.10, 0.42, 0.78, 1.0);
const SEA_NEAR: Color = Color::new(0.22, 0.72, 0.85, 1.0);
const SEA_SHINE: Color = Color::new(1.0, 1.0, 1.0, 0.35);
const FOAM: Color = Color::new(1.0, 1.0, 1.0, 0.95);
/// 모래: 아이 그림의 흰 얼굴이 묻히지 않게 너무 하얗지 않은 따뜻한 베이지
const SAND: Color = Color::new(0.90, 0.76, 0.52, 1.0);
/// 물에 젖은 모래 (물결 바로 아래)
const SAND_WET: Color = Color::new(0.78, 0.62, 0.40, 1.0);
const SAND_DOT: Color = Color::new(0.76, 0.60, 0.38, 1.0);
const SHELL: Color = Color::new(1.0, 0.72, 0.74, 1.0);
const SHELL_LINE: Color = Color::new(0.80, 0.45, 0.50, 1.0);
const WOOD: Color = Color::new(0.80, 0.56, 0.30, 1.0);
const WOOD_DARK: Color = Color::new(0.52, 0.33, 0.15, 1.0);
const WOOD_EDGE: Color = Color::new(0.30, 0.18, 0.08, 1.0);
/// 글자는 흰색 + 진한 테두리 (text.rs)
const TEXT_COLOR: Color = WHITE;
const CARD: Color = Color::new(1.0, 1.0, 1.0, 0.55);
const CARD_PICKED: Color = Color::new(1.0, 0.98, 0.85, 0.9);
const PICK_BORDER: Color = Color::new(1.0, 0.70, 0.10, 1.0);

// ---- 노을·밤 색 (D10) ----
/// 밤하늘 남색: 위는 진하게, 수평선 쪽은 조금 밝게
const NIGHT_SKY_TOP: Color = Color::new(0.04, 0.06, 0.22, 1.0);
const NIGHT_SKY_BOTTOM: Color = Color::new(0.12, 0.16, 0.40, 1.0);
/// 노을·새벽 때 하늘 아래쪽에 번지는 색 (수평선 가까이는 주황, 그 위는 분홍)
const DUSK_ORANGE: Color = Color::new(1.0, 0.56, 0.26, 1.0);
const DUSK_PINK: Color = Color::new(0.96, 0.52, 0.66, 1.0);
/// 지는 해는 주황빛
const SUNSET_SUN: Color = Color::new(1.0, 0.52, 0.22, 1.0);
const NIGHT_SEA_FAR: Color = Color::new(0.03, 0.09, 0.26, 1.0);
const NIGHT_SEA_NEAR: Color = Color::new(0.07, 0.20, 0.38, 1.0);
/// 밤 바다 물결 반짝임은 달빛 색
const MOON_SHINE: Color = Color::new(1.0, 0.95, 0.70, 0.40);
const MOON: Color = Color::new(1.0, 0.97, 0.82, 1.0);
const MOON_SPOT: Color = Color::new(0.90, 0.86, 0.68, 1.0);
const MOON_GLOW: Color = Color::new(1.0, 0.97, 0.80, 0.16);
const STAR: Color = Color::new(1.0, 0.98, 0.86, 1.0);
const STAR_COUNT: usize = 42;
/// 밤에 캐릭터에 곱하는 색. 배경보다 훨씬 덜 어둡게 해서 캐릭터가 잘 보이게.
const CHARACTER_NIGHT_TINT: Color = Color::new(0.82, 0.84, 0.97, 1.0);
/// 밤에 얼마나 어두워지는지 (1 = 밤빛 그대로, 0 = 그대로). 상자는 덜 어둡게.
const BOX_NIGHT_AMOUNT: f32 = 0.7;
const FOAM_NIGHT_AMOUNT: f32 = 0.7;
const SAND_NIGHT_AMOUNT: f32 = 0.85;

/// 수평선 y (하늘과 바다가 만나는 곳). 땅(GROUND_Y=440)보다 위.
const HORIZON_Y: f32 = 330.0;
/// 젖은 모래 띠 두께
const WET_SAND_THICKNESS: f32 = 10.0;

/// 해·달 자리. 메뉴 화면은 예전처럼 오른쪽 위에 해. 플레이는 위쪽에 정보판이 있어서 조금 아래.
struct SkyLayout {
    sun: (f32, f32),
    moon: (f32, f32),
}
const MENU_SKY: SkyLayout = SkyLayout { sun: (870.0, 70.0), moon: (150.0, 120.0) };
const PLAY_SKY: SkyLayout = SkyLayout { sun: (850.0, 150.0), moon: (170.0, 140.0) };
const SUN_RADIUS: f32 = 40.0;
const MOON_RADIUS: f32 = 30.0;

// ---- 캐릭터 고르기 화면 배치 ----
const CARD_CENTERS_X: [f32; 3] = [240.0, 480.0, 720.0];
const CARD_WIDTH: f32 = 220.0;
const CARD_TOP: f32 = 110.0;
const CARD_HEIGHT: f32 = 340.0;
const CARD_FEET_Y: f32 = 370.0;
const CARD_SPRITE_HEIGHT: f32 = 190.0;
/// 골라진 캐릭터는 이만큼 크게
const PICKED_GROW: f32 = 1.15;
/// "캐릭터를 골라요" 바닥선. 고른 카드 위 주황 세모와 겹치지 않게 위로 올렸다 (BUG-002).
const SELECT_TITLE_Y: f32 = 62.0;
/// 주황 세모 끝이 카드 위쪽에서 얼마나 떨어져 있는지, 높이, 통통 튀는 폭
const PICK_ARROW_GAP: f32 = 12.0;
const PICK_ARROW_HEIGHT: f32 = 20.0;
const PICK_ARROW_BOB: f32 = 4.0;

// ---- 플레이 화면 배치 ----
/// 조작 안내는 화면 아래 모래 위 (위쪽은 정보판 자리)
const PLAY_HINT_Y: f32 = 526.0;

/// 캐릭터 고르기 화면에서 `index`번째 카드 네모.
pub fn select_card_rect(index: usize) -> Rect {
    Rect::new(CARD_CENTERS_X[index] - CARD_WIDTH / 2.0, CARD_TOP, CARD_WIDTH, CARD_HEIGHT)
}

/// 누른 곳(가상 좌표)이 몇 번째 카드인지. 카드 밖이면 None.
pub fn select_card_at(point: Vec2) -> Option<usize> {
    (0..CharacterKind::ALL.len()).find(|&i| select_card_rect(i).contains(point))
}

fn time() -> f32 {
    get_time() as f32
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    Color::new(a.r + (b.r - a.r) * t, a.g + (b.g - a.g) * t, a.b + (b.b - a.b) * t, a.a + (b.a - a.a) * t)
}

/// 밤빛으로 어둡게: `amount` 0 = 그대로, 1 = 푸르스름하게 어두운 밤 색. 투명도는 그대로.
fn night(c: Color, amount: f32) -> Color {
    let dark = Color::new(c.r * 0.32, c.g * 0.38, c.b * 0.55 + 0.06, c.a);
    mix(c, dark, amount.clamp(0.0, 1.0))
}

/// 노을·새벽 빛의 세기: 어둠이 반쯤일 때 가장 세고, 한낮·한밤에는 0.
fn dusk_glow(darkness: f32) -> f32 {
    let d = darkness.clamp(0.0, 1.0);
    4.0 * d * (1.0 - d)
}

/// `from` 아래로는 0, `to` 위로는 1, 사이는 부드럽게.
fn ramp(x: f32, from: f32, to: f32) -> f32 {
    let t = ((x - from) / (to - from)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

// ---- 배경 (해변) ----

fn draw_sky(darkness: f32, layout: &SkyLayout) {
    let d = darkness.clamp(0.0, 1.0);
    let glow = dusk_glow(d);
    // 위는 진하고 아래는 연한 하늘 (가로 띠 여러 개로 그라데이션 흉내)
    let bands = 24;
    let h = HORIZON_Y / bands as f32;
    for i in 0..bands {
        let t = i as f32 / (bands - 1) as f32;
        let base = mix(mix(SKY_TOP, SKY_BOTTOM, t), mix(NIGHT_SKY_TOP, NIGHT_SKY_BOTTOM, t), d);
        let warm = mix(DUSK_PINK, DUSK_ORANGE, t);
        let c = mix(base, warm, glow * t.powf(1.5) * 0.85);
        draw_rectangle(0.0, i as f32 * h, VIRTUAL_WIDTH, h + 1.0, c);
    }

    draw_stars(ramp(d, 0.45, 0.95));
    draw_moon(layout, d);

    // 해: 어두워질수록 주황빛으로 수평선 아래로 내려간다 (바다가 나중에 그려져서 가려진다).
    if d < 1.0 {
        let sink = ramp(d, 0.0, 0.85);
        let sun = vec2(layout.sun.0, lerp(layout.sun.1, HORIZON_Y + SUN_RADIUS * 1.8, sink));
        let color = mix(SUN, SUNSET_SUN, ramp(d, 0.0, 0.6));
        let glow_color = Color { a: SUN_GLOW.a * (1.0 - d), ..mix(SUN_GLOW, SUNSET_SUN, d) };
        draw_circle(sun.x, sun.y, 56.0 + (time() * 1.5).sin() * 3.0, glow_color);
        draw_circle(sun.x, sun.y, SUN_RADIUS, color);
    }

    // 천천히 흘러가는 구름 (노을엔 분홍빛, 밤엔 어둡고 옅게)
    let cloud = mix(night(CLOUD, d * 0.9), DUSK_PINK, glow * 0.35);
    let cloud = Color { a: CLOUD.a + (1.0 - CLOUD.a) * d, ..cloud }; // 밤엔 겹친 자국이 안 보이게 불투명
    for (i, (x0, y, s)) in [(120.0, 80.0, 1.0), (520.0, 50.0, 0.8), (820.0, 150.0, 1.2)].into_iter().enumerate() {
        let x = (x0 + time() * (8.0 + i as f32 * 3.0)) % (VIRTUAL_WIDTH + 200.0) - 100.0;
        draw_circle(x, y, 26.0 * s, cloud);
        draw_circle(x + 30.0 * s, y - 12.0 * s, 32.0 * s, cloud);
        draw_circle(x + 64.0 * s, y, 24.0 * s, cloud);
        draw_rectangle(x, y, 64.0 * s, 24.0 * s, cloud);
    }
}

/// 별: 자리는 늘 같고(번호로 정함), 하나하나 다른 빠르기로 살짝 깜빡인다.
fn draw_stars(alpha: f32) {
    if alpha <= 0.0 {
        return;
    }
    for i in 0..STAR_COUNT {
        let x = (i as f32 * 197.3 + 31.0) % VIRTUAL_WIDTH;
        let y = 18.0 + ((i * 53 + 17) % 97) as f32 / 97.0 * (HORIZON_Y - 70.0);
        let speed = 1.3 + (i % 5) as f32 * 0.45;
        let twinkle = 0.55 + 0.45 * (time() * speed + i as f32 * 1.7).sin();
        let r = 1.6 + (i % 3) as f32 * 0.8;
        let c = Color { a: alpha * twinkle, ..STAR };
        draw_circle(x, y, r, c);
        if i % 6 == 0 {
            // 큰 별은 십자 반짝
            let len = r * 3.0 * twinkle;
            let c = Color { a: alpha * twinkle * 0.7, ..STAR };
            draw_line(x - len, y, x + len, y, 1.2, c);
            draw_line(x, y - len, x, y + len, 1.2, c);
        }
    }
}

/// 보름달: 해가 지는 동안 수평선에서 떠오르고, 새벽에 다시 내려가며 사라진다.
fn draw_moon(layout: &SkyLayout, d: f32) {
    let alpha = ramp(d, 0.3, 0.9);
    if alpha <= 0.0 {
        return;
    }
    let rise = ramp(d, 0.2, 1.0);
    let c = vec2(layout.moon.0, lerp(HORIZON_Y + MOON_RADIUS * 2.0, layout.moon.1, rise));
    draw_circle(c.x, c.y, MOON_RADIUS * 1.9, Color { a: MOON_GLOW.a * alpha, ..MOON_GLOW });
    draw_circle(c.x, c.y, MOON_RADIUS * 1.4, Color { a: MOON_GLOW.a * alpha, ..MOON_GLOW });
    draw_circle(c.x, c.y, MOON_RADIUS, Color { a: alpha, ..MOON });
    for (dx, dy, r) in [(-9.0, -6.0, 6.0), (8.0, 7.0, 4.5), (6.0, -12.0, 3.0), (-4.0, 12.0, 3.5)] {
        draw_circle(c.x + dx, c.y + dy, r, Color { a: alpha, ..MOON_SPOT });
    }
}

fn draw_sea(ground_y: f32, d: f32) {
    let bands = 8;
    let h = (ground_y - HORIZON_Y) / bands as f32;
    let far = mix(SEA_FAR, NIGHT_SEA_FAR, d);
    let near = mix(SEA_NEAR, NIGHT_SEA_NEAR, d);
    for i in 0..bands {
        let c = mix(far, near, i as f32 / (bands - 1) as f32);
        draw_rectangle(0.0, HORIZON_Y + i as f32 * h, VIRTUAL_WIDTH, h + 1.0, c);
    }
    // 반짝이는 물결 줄 몇 개 (옆으로 천천히 흔들림). 밤에는 달빛 색.
    let shine = mix(SEA_SHINE, MOON_SHINE, d);
    for i in 0..9 {
        let y = HORIZON_Y + 14.0 + (i % 3) as f32 * 26.0 + (i / 3) as f32 * 8.0;
        let x = (i as f32 * 113.0 + (time() * 0.8 + i as f32).sin() * 10.0) % VIRTUAL_WIDTH;
        draw_line(x, y, x + 28.0 + (i % 2) as f32 * 14.0, y, 3.0, shine);
    }
}

/// 바다와 모래가 만나는 곳의 하얀 물결: 둥근 거품이 천천히 밀려왔다 빠진다.
fn draw_foam(ground_y: f32, d: f32) {
    let foam = night(FOAM, d * FOAM_NIGHT_AMOUNT);
    let surge = (time() * 0.9).sin() * 5.0; // 밀려왔다(아래) 빠졌다(위)
    let mut x = -20.0;
    let mut i = 0;
    while x < VIRTUAL_WIDTH + 20.0 {
        let wobble = (x * 0.03 + time() * 1.6).sin() * 3.0;
        let y = ground_y - 6.0 + surge + wobble;
        draw_circle(x, y, 9.0 + (i % 3) as f32 * 2.0, foam);
        x += 16.0;
        i += 1;
    }
    draw_rectangle(0.0, ground_y - 6.0 + surge, VIRTUAL_WIDTH, 6.0, foam);
}

fn draw_shell(c: Vec2, s: f32, d: f32) {
    let (shell, sand, line) = (night(SHELL, d), night(SAND, d), night(SHELL_LINE, d));
    // 부채 모양 조개: 반원 + 줄무늬
    draw_circle(c.x, c.y, 9.0 * s, shell);
    draw_rectangle(c.x - 10.0 * s, c.y, 20.0 * s, 10.0 * s, sand);
    draw_triangle(c + vec2(-4.0, 0.0) * s, c + vec2(4.0, 0.0) * s, c + vec2(0.0, 5.0) * s, shell);
    for dx in [-5.0, 0.0, 5.0] {
        draw_line(c.x, c.y + 3.0 * s, c.x + dx * s, c.y - 7.0 * s, 1.5, line);
    }
}

fn draw_ground(ground_y: f32, d: f32) {
    // 모래는 조금 덜 어둡게: 캐릭터가 서 있는 바닥이 너무 깜깜하지 않게
    let sd = d * SAND_NIGHT_AMOUNT;
    draw_sea(ground_y, d);
    // 모래
    draw_rectangle(0.0, ground_y, VIRTUAL_WIDTH, VIRTUAL_HEIGHT - ground_y, night(SAND, sd));
    draw_rectangle(0.0, ground_y, VIRTUAL_WIDTH, WET_SAND_THICKNESS, night(SAND_WET, sd));
    // 모래 알갱이
    let dot = night(SAND_DOT, sd);
    for i in 0..45 {
        let x = (i as f32 * 97.0) % VIRTUAL_WIDTH + 10.0;
        let y = ground_y + WET_SAND_THICKNESS + 8.0 + ((i * 37) % 80) as f32;
        if y < VIRTUAL_HEIGHT - 4.0 {
            draw_circle(x, y, 1.5 + (i % 3) as f32, dot);
        }
    }
    // 조개 몇 개
    for (x, y, s) in [(90.0, 488.0, 1.0), (430.0, 520.0, 0.8), (650.0, 478.0, 1.1), (900.0, 505.0, 0.9)] {
        draw_shell(vec2(x, y), s, sd);
    }
    draw_foam(ground_y, d);
}

/// 나무 상자: 테두리 + 가로 판자 줄 + X자 버팀목. 밤에는 배경보다 덜 어둡게.
fn draw_wood_box(r: Rect, d: f32) {
    let amount = d * BOX_NIGHT_AMOUNT;
    let (wood, dark) = (night(WOOD, amount), night(WOOD_DARK, amount));
    let edge = (r.w.min(r.h) * 0.12).max(4.0);
    draw_rectangle(r.x, r.y, r.w, r.h, wood);
    draw_line(r.x + edge, r.y + edge, r.x + r.w - edge, r.y + r.h - edge, edge * 0.8, dark);
    draw_line(r.x + r.w - edge, r.y + edge, r.x + edge, r.y + r.h - edge, edge * 0.8, dark);
    draw_rectangle_lines(r.x + edge / 2.0, r.y + edge / 2.0, r.w - edge, r.h - edge, edge, dark);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, night(WOOD_EDGE, amount));
}

// ---- 타이틀 ----

pub fn draw_title(assets: &Assets, text: &TextPainter, ground_y: f32) {
    draw_sky(0.0, &MENU_SKY);
    draw_ground(ground_y, 0.0);
    for (i, kind) in CharacterKind::ALL.into_iter().enumerate() {
        let pose = sprites::pose(Motion::Idle, time() + i as f32 * 0.37, false);
        draw_character(assets.character(kind), vec2(CARD_CENTERS_X[i], ground_y + 4.0), 150.0, Facing::Right, pose, WHITE);
    }
    // 제목: 하늘 부분에서 살짝 둥실둥실. 자모마다 아이가 정한 색 (view/title.rs)
    let bob = (time() * 2.0).sin() * 6.0;
    assets.title.draw(VIRTUAL_WIDTH / 2.0, 150.0 + bob, 120.0);
    // 안내 문구는 천천히 깜빡
    let alpha = 0.65 + 0.35 * (time() * 3.0).sin();
    let hint = Color { a: alpha, ..TEXT_COLOR };
    text.centered("시작하려면 Space/Enter 또는 화면을 눌러요", VIRTUAL_WIDTH / 2.0, 240.0, 38.0, hint);
}

// ---- 캐릭터 고르기 ----

pub fn draw_select(assets: &Assets, text: &TextPainter, ground_y: f32, picked: usize) {
    draw_sky(0.0, &MENU_SKY);
    draw_ground(ground_y, 0.0);
    text.centered("캐릭터를 골라요", VIRTUAL_WIDTH / 2.0, SELECT_TITLE_Y, 56.0, TEXT_COLOR);

    for (i, kind) in CharacterKind::ALL.into_iter().enumerate() {
        let r = select_card_rect(i);
        let is_picked = i == picked;
        if is_picked {
            let pad = 8.0;
            draw_rectangle(r.x - pad, r.y - pad, r.w + pad * 2.0, r.h + pad * 2.0, CARD_PICKED);
            draw_rectangle_lines(r.x - pad, r.y - pad, r.w + pad * 2.0, r.h + pad * 2.0, 8.0, PICK_BORDER);
            // 위에서 콕콕 가리키는 세모
            let bob = (time() * 5.0).sin() * PICK_ARROW_BOB;
            let tip = vec2(r.x + r.w / 2.0, r.y - PICK_ARROW_GAP + bob);
            draw_triangle(
                tip,
                tip + vec2(-18.0, -PICK_ARROW_HEIGHT),
                tip + vec2(18.0, -PICK_ARROW_HEIGHT),
                PICK_BORDER,
            );
        } else {
            draw_rectangle(r.x, r.y, r.w, r.h, CARD);
        }
        let height = if is_picked { CARD_SPRITE_HEIGHT * PICKED_GROW } else { CARD_SPRITE_HEIGHT };
        let pose = sprites::pose(Motion::Idle, time() + i as f32 * 0.37, false);
        draw_character(assets.character(kind), vec2(r.x + r.w / 2.0, CARD_FEET_Y), height, Facing::Right, pose, WHITE);
        let name_size = if is_picked { 50.0 } else { 42.0 };
        text.centered(kind.name(), r.x + r.w / 2.0, r.y + r.h - 22.0, name_size, TEXT_COLOR);
    }

    text.centered("화살표로 고르고 Space/Enter, 또는 캐릭터를 눌러요", VIRTUAL_WIDTH / 2.0, 515.0, 30.0, TEXT_COLOR);
}

// ---- 플레이 ----

/// 그림을 그릴 발 x: 충돌 네모 가운데지만, 그림(옆으로 뻗은 팔 포함)이 화면 밖으로
/// 나가지 않게 화면 안으로 당긴다 (BUG-001). 규칙(충돌 네모)은 그대로다.
fn feet_x_on_screen(center_x: f32, half_width: f32) -> f32 {
    center_x.clamp(half_width, (VIRTUAL_WIDTH - half_width).max(half_width))
}

pub fn draw_play(assets: &Assets, text: &TextPainter, game: &Game, fast_forward: bool, pops: &CoinPops) {
    let d = game.clock.sky_darkness();
    draw_sky(d, &PLAY_SKY);
    draw_ground(game.world.ground_y, d);
    for b in &game.world.obstacles {
        draw_wood_box(*b, d);
    }
    draw_coins(&game.coins, d, time());

    let player = &game.player;
    let texture = assets.character(player.kind);
    let r = player.rect();
    let half = max_half_width(texture, PLAY_SPRITE_HEIGHT);
    let feet = vec2(feet_x_on_screen(r.x + r.w / 2.0, half), r.y + r.h);
    let pose = sprites::pose(player.motion, time(), player.vel.y < 0.0);
    let tint = mix(WHITE, CHARACTER_NIGHT_TINT, d);
    draw_character(texture, feet, PLAY_SPRITE_HEIGHT, player.facing, pose, tint);

    pops.draw(text);
    draw_hud(text, &game.clock, game.wallet.coins(), fast_forward);
    text.centered("화살표: 걷기   Space: 점프   Esc: 캐릭터 고르기", VIRTUAL_WIDTH / 2.0, PLAY_HINT_Y, 26.0, TEXT_COLOR);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tapping_a_card_picks_that_character() {
        for i in 0..3 {
            let r = select_card_rect(i);
            assert_eq!(select_card_at(r.center()), Some(i));
        }
        assert_eq!(select_card_at(vec2(10.0, 10.0)), None);
    }

    #[test]
    fn cards_fit_on_screen() {
        for i in 0..3 {
            let r = select_card_rect(i);
            assert!(r.x >= 0.0 && r.x + r.w <= VIRTUAL_WIDTH && r.y + r.h <= VIRTUAL_HEIGHT);
        }
    }

    #[test]
    fn sprite_is_pulled_inside_screen_edges() {
        // BUG-001: 반폭 40인 그림은 발 x가 40 ~ 920 안에 있어야 잘리지 않는다.
        assert_eq!(feet_x_on_screen(24.0, 40.0), 40.0);
        assert_eq!(feet_x_on_screen(VIRTUAL_WIDTH - 24.0, 40.0), VIRTUAL_WIDTH - 40.0);
        assert_eq!(feet_x_on_screen(300.0, 40.0), 300.0);
    }

    #[test]
    fn pick_arrow_stays_below_select_title() {
        // BUG-002: 세모가 가장 높이 튀어도 그 위 끝이 제목 바닥선보다 아래에 있다.
        let arrow_top = CARD_TOP - PICK_ARROW_GAP - PICK_ARROW_BOB - PICK_ARROW_HEIGHT;
        assert!(arrow_top > SELECT_TITLE_Y + 4.0, "세모 위 끝 {arrow_top}");
    }

    #[test]
    fn horizon_is_above_ground() {
        assert!(HORIZON_Y < crate::config::GROUND_Y);
    }

    #[test]
    fn sky_is_plain_day_at_zero_and_navy_at_one() {
        // 한낮·한밤에는 노을빛이 섞이지 않는다 → 새벽이 끝나면 낮 하늘과 똑같다.
        assert_eq!(dusk_glow(0.0), 0.0);
        assert_eq!(dusk_glow(1.0), 0.0);
        assert!(dusk_glow(0.5) > 0.9);
        assert_eq!(night(SAND, 0.0), SAND);
    }

    #[test]
    fn sun_sets_fully_below_horizon_at_night() {
        // 해(빛 둘레 포함)가 다 내려가면 바다에 가려진다.
        let lowest = HORIZON_Y + SUN_RADIUS * 1.8;
        assert!(lowest - SUN_RADIUS > HORIZON_Y);
        // 플레이 화면 해·달은 정보판(맨 위 약 70픽셀) 아래.
        assert!(PLAY_SKY.sun.1 - SUN_RADIUS > 70.0);
        assert!(PLAY_SKY.moon.1 - MOON_RADIUS > 70.0);
    }
}
