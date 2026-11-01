//! 화면 그리기: 타이틀, 캐릭터 고르기, 플레이. 좌표는 모두 가상 화면(960×540) 기준.
//! 배경은 해변: 하늘(해·구름) → 먼 바다(수평선) → 하얀 물결 → 모래. 바다는 그림일 뿐 규칙은 없다.

use super::assets::Assets;
use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::sprites::{self, draw_character, max_half_width, PLAY_SPRITE_HEIGHT};
use super::text::TextPainter;
use crate::logic::character::CharacterKind;
use crate::logic::player::{Facing, Motion, Player};
use crate::logic::world::World;
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
/// 글자는 흰색 + 진한 테두리 (text.rs)
const TEXT_COLOR: Color = WHITE;
const CARD: Color = Color::new(1.0, 1.0, 1.0, 0.55);
const CARD_PICKED: Color = Color::new(1.0, 0.98, 0.85, 0.9);
const PICK_BORDER: Color = Color::new(1.0, 0.70, 0.10, 1.0);

/// 수평선 y (하늘과 바다가 만나는 곳). 땅(GROUND_Y=440)보다 위.
const HORIZON_Y: f32 = 330.0;
/// 젖은 모래 띠 두께
const WET_SAND_THICKNESS: f32 = 10.0;

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

// ---- 배경 (해변) ----

fn draw_sky() {
    // 위는 진하고 아래는 연한 하늘 (가로 띠 여러 개로 그라데이션 흉내)
    let bands = 10;
    let h = HORIZON_Y / bands as f32;
    for i in 0..bands {
        let c = mix(SKY_TOP, SKY_BOTTOM, i as f32 / (bands - 1) as f32);
        draw_rectangle(0.0, i as f32 * h, VIRTUAL_WIDTH, h + 1.0, c);
    }
    // 해: 오른쪽 위, 제목과 겹치지 않는 곳
    let sun = vec2(870.0, 70.0);
    draw_circle(sun.x, sun.y, 56.0 + (time() * 1.5).sin() * 3.0, SUN_GLOW);
    draw_circle(sun.x, sun.y, 40.0, SUN);
    // 천천히 흘러가는 구름
    for (i, (x0, y, s)) in [(120.0, 80.0, 1.0), (520.0, 50.0, 0.8), (820.0, 150.0, 1.2)].into_iter().enumerate() {
        let x = (x0 + time() * (8.0 + i as f32 * 3.0)) % (VIRTUAL_WIDTH + 200.0) - 100.0;
        draw_circle(x, y, 26.0 * s, CLOUD);
        draw_circle(x + 30.0 * s, y - 12.0 * s, 32.0 * s, CLOUD);
        draw_circle(x + 64.0 * s, y, 24.0 * s, CLOUD);
        draw_rectangle(x, y, 64.0 * s, 24.0 * s, CLOUD);
    }
}

fn draw_sea(ground_y: f32) {
    let bands = 8;
    let h = (ground_y - HORIZON_Y) / bands as f32;
    for i in 0..bands {
        let c = mix(SEA_FAR, SEA_NEAR, i as f32 / (bands - 1) as f32);
        draw_rectangle(0.0, HORIZON_Y + i as f32 * h, VIRTUAL_WIDTH, h + 1.0, c);
    }
    // 반짝이는 물결 줄 몇 개 (옆으로 천천히 흔들림)
    for i in 0..9 {
        let y = HORIZON_Y + 14.0 + (i % 3) as f32 * 26.0 + (i / 3) as f32 * 8.0;
        let x = (i as f32 * 113.0 + (time() * 0.8 + i as f32).sin() * 10.0) % VIRTUAL_WIDTH;
        draw_line(x, y, x + 28.0 + (i % 2) as f32 * 14.0, y, 3.0, SEA_SHINE);
    }
}

/// 바다와 모래가 만나는 곳의 하얀 물결: 둥근 거품이 천천히 밀려왔다 빠진다.
fn draw_foam(ground_y: f32) {
    let surge = (time() * 0.9).sin() * 5.0; // 밀려왔다(아래) 빠졌다(위)
    let mut x = -20.0;
    let mut i = 0;
    while x < VIRTUAL_WIDTH + 20.0 {
        let wobble = (x * 0.03 + time() * 1.6).sin() * 3.0;
        let y = ground_y - 6.0 + surge + wobble;
        draw_circle(x, y, 9.0 + (i % 3) as f32 * 2.0, FOAM);
        x += 16.0;
        i += 1;
    }
    draw_rectangle(0.0, ground_y - 6.0 + surge, VIRTUAL_WIDTH, 6.0, FOAM);
}

fn draw_shell(c: Vec2, s: f32) {
    // 부채 모양 조개: 반원 + 줄무늬
    draw_circle(c.x, c.y, 9.0 * s, SHELL);
    draw_rectangle(c.x - 10.0 * s, c.y, 20.0 * s, 10.0 * s, SAND);
    draw_triangle(c + vec2(-4.0, 0.0) * s, c + vec2(4.0, 0.0) * s, c + vec2(0.0, 5.0) * s, SHELL);
    for dx in [-5.0, 0.0, 5.0] {
        draw_line(c.x, c.y + 3.0 * s, c.x + dx * s, c.y - 7.0 * s, 1.5, SHELL_LINE);
    }
}

fn draw_ground(ground_y: f32) {
    draw_sea(ground_y);
    // 모래
    draw_rectangle(0.0, ground_y, VIRTUAL_WIDTH, VIRTUAL_HEIGHT - ground_y, SAND);
    draw_rectangle(0.0, ground_y, VIRTUAL_WIDTH, WET_SAND_THICKNESS, SAND_WET);
    // 모래 알갱이
    for i in 0..45 {
        let x = (i as f32 * 97.0) % VIRTUAL_WIDTH + 10.0;
        let y = ground_y + WET_SAND_THICKNESS + 8.0 + ((i * 37) % 80) as f32;
        if y < VIRTUAL_HEIGHT - 4.0 {
            draw_circle(x, y, 1.5 + (i % 3) as f32, SAND_DOT);
        }
    }
    // 조개 몇 개
    for (x, y, s) in [(90.0, 488.0, 1.0), (430.0, 520.0, 0.8), (650.0, 478.0, 1.1), (900.0, 505.0, 0.9)] {
        draw_shell(vec2(x, y), s);
    }
    draw_foam(ground_y);
}

/// 나무 상자: 테두리 + 가로 판자 줄 + X자 버팀목.
fn draw_wood_box(r: Rect) {
    let edge = (r.w.min(r.h) * 0.12).max(4.0);
    draw_rectangle(r.x, r.y, r.w, r.h, WOOD);
    draw_line(r.x + edge, r.y + edge, r.x + r.w - edge, r.y + r.h - edge, edge * 0.8, WOOD_DARK);
    draw_line(r.x + r.w - edge, r.y + edge, r.x + edge, r.y + r.h - edge, edge * 0.8, WOOD_DARK);
    draw_rectangle_lines(r.x + edge / 2.0, r.y + edge / 2.0, r.w - edge, r.h - edge, edge, WOOD_DARK);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, Color::new(0.30, 0.18, 0.08, 1.0));
}

// ---- 타이틀 ----

pub fn draw_title(assets: &Assets, text: &TextPainter, ground_y: f32) {
    draw_sky();
    draw_ground(ground_y);
    for (i, kind) in CharacterKind::ALL.into_iter().enumerate() {
        let pose = sprites::pose(Motion::Idle, time() + i as f32 * 0.37, false);
        draw_character(assets.character(kind), vec2(CARD_CENTERS_X[i], ground_y + 4.0), 150.0, Facing::Right, pose);
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
    draw_sky();
    draw_ground(ground_y);
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
        draw_character(assets.character(kind), vec2(r.x + r.w / 2.0, CARD_FEET_Y), height, Facing::Right, pose);
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

pub fn draw_play(assets: &Assets, text: &TextPainter, world: &World, player: &Player) {
    draw_sky();
    draw_ground(world.ground_y);
    for b in &world.obstacles {
        draw_wood_box(*b);
    }
    let texture = assets.character(player.kind);
    let r = player.rect();
    let half = max_half_width(texture, PLAY_SPRITE_HEIGHT);
    let feet = vec2(feet_x_on_screen(r.x + r.w / 2.0, half), r.y + r.h);
    let pose = sprites::pose(player.motion, time(), player.vel.y < 0.0);
    draw_character(texture, feet, PLAY_SPRITE_HEIGHT, player.facing, pose);

    text.centered("화살표: 걷기   Space: 점프   Esc: 캐릭터 고르기", VIRTUAL_WIDTH / 2.0, 40.0, 28.0, TEXT_COLOR);
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
}
