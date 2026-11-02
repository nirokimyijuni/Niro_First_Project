//! 화면 그리기: 타이틀, 캐릭터 고르기, 플레이. 좌표는 모두 가상 화면(960×540) 기준.
//! 배경은 해변: 하늘·바다 빛(sky.rs) → 하얀 물결 → 모래. 바다는 그림일 뿐 규칙은 없다.
//! 플레이 중에는 logic 시계의 `sky_darkness`(0 한낮 ~ 1 한밤)로 낮 그림과 밤 그림을 부드럽게 섞는다
//! (D9, D10): 노을 때 하늘 아래가 주황·분홍, 해는 수평선 아래로, 밤에는 남색 하늘에 별과 달.

use super::assets::Assets;
use super::coin::{draw_coins, CoinPops};
use super::hud::draw_hud;
use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::sky::{self, mix, night, Canvas, GpuCanvas, SkyArt, SkyLook, MENU_SKY, PLAY_SKY};
use super::sprites::{self, draw_character, max_half_width, PLAY_SPRITE_HEIGHT};
use super::text::TextPainter;
use crate::logic::character::CharacterKind;
use crate::logic::game::Game;
use crate::logic::player::{Facing, Motion};
use macroquad::prelude::*;

// ---- 색 ----
/// 모래: 아이 그림의 흰 얼굴이 묻히지 않게 너무 하얗지 않은 따뜻한 베이지
const SAND: Color = Color::new(0.90, 0.76, 0.52, 1.0);
/// 화면 아래쪽 모래는 조금 더 진하게 (그라데이션)
const SAND_DEEP: Color = Color::new(0.84, 0.68, 0.45, 1.0);
/// 물에 젖은 모래 (물결 바로 아래)
const SAND_WET: Color = Color::new(0.74, 0.59, 0.40, 1.0);
/// 모래 알갱이: 어두운 알갱이와 밝은 알갱이 (반투명)
const SAND_DOT: Color = Color::new(0.62, 0.47, 0.28, 0.45);
const SAND_SPARK: Color = Color::new(1.0, 0.95, 0.82, 0.55);
const SAND_DOT_COUNT: u32 = 90;
const SHELL: Color = Color::new(1.0, 0.72, 0.74, 1.0);
const SHELL_LINE: Color = Color::new(0.80, 0.45, 0.50, 1.0);
const FOAM: Color = Color::new(1.0, 1.0, 1.0, 0.95);
/// 거품 아래 모래 위로 얇게 번지는 물 (반투명)
const WASH: Color = Color::new(0.85, 0.95, 1.0, 0.35);
const WOOD: Color = Color::new(0.80, 0.56, 0.30, 1.0);
const WOOD_DARK: Color = Color::new(0.52, 0.33, 0.15, 1.0);
const WOOD_EDGE: Color = Color::new(0.30, 0.18, 0.08, 1.0);
/// 글자는 흰색 + 진한 테두리 (text.rs)
const TEXT_COLOR: Color = WHITE;
const CARD: Color = Color::new(1.0, 1.0, 1.0, 0.55);
const CARD_PICKED: Color = Color::new(1.0, 0.98, 0.85, 0.9);
const PICK_BORDER: Color = Color::new(1.0, 0.70, 0.10, 1.0);

/// 밤에 캐릭터에 곱하는 색. 배경보다 훨씬 덜 어둡게 해서 캐릭터가 잘 보이게.
const CHARACTER_NIGHT_TINT: Color = Color::new(0.82, 0.84, 0.97, 1.0);
/// 밤에 얼마나 어두워지는지 (1 = 밤빛 그대로, 0 = 그대로). 상자는 덜 어둡게.
const BOX_NIGHT_AMOUNT: f32 = 0.7;
const FOAM_NIGHT_AMOUNT: f32 = 0.7;
const SAND_NIGHT_AMOUNT: f32 = 0.85;

/// 젖은 모래가 마른 모래로 바뀌는 폭
const WET_SAND_THICKNESS: f32 = 16.0;

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

// ---- 배경 (해변) ----

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

/// 아래가 평평한 반원 (부채꼴 조개 몸통). 모래 위에 덧칠하지 않고 반원만 그린다.
fn draw_half_disc_up(c: Vec2, r: f32, color: Color) {
    const STEPS: usize = 14;
    for i in 0..STEPS {
        let a0 = std::f32::consts::PI * (1.0 + i as f32 / STEPS as f32);
        let a1 = std::f32::consts::PI * (1.0 + (i + 1) as f32 / STEPS as f32);
        draw_triangle(c, c + vec2(a0.cos(), a0.sin()) * r, c + vec2(a1.cos(), a1.sin()) * r, color);
    }
}

fn draw_shell(c: Vec2, s: f32, d: f32) {
    let (shell, line) = (night(SHELL, d), night(SHELL_LINE, d));
    // 부채 모양 조개: 반원 + 줄무늬 + 아래 꼭지
    draw_half_disc_up(c, 9.0 * s, shell);
    draw_triangle(c + vec2(-4.0, 0.0) * s, c + vec2(4.0, 0.0) * s, c + vec2(0.0, 5.0) * s, shell);
    for dx in [-5.0, 0.0, 5.0] {
        draw_line(c.x, c.y + 3.0 * s, c.x + dx * s, c.y - 7.0 * s, 1.5, line);
    }
}

/// 바다(sky.rs) → 모래 그라데이션 → 알갱이·조개 → 물결 거품.
fn draw_ground(art: &SkyArt, look: &SkyLook, ground_y: f32) {
    let d = look.d;
    let mut canvas = GpuCanvas { art };
    sky::draw_sea(&mut canvas, look, ground_y);
    // 모래는 조금 덜 어둡게: 캐릭터가 서 있는 바닥이 너무 깜깜하지 않게
    let sd = d * SAND_NIGHT_AMOUNT;
    // 젖은 모래 → 마른 모래 → 아래쪽은 조금 진하게 (띠 없이 부드럽게)
    canvas.vgradient(
        0.0,
        VIRTUAL_WIDTH,
        &[
            (ground_y, night(SAND_WET, sd)),
            (ground_y + WET_SAND_THICKNESS, night(SAND, sd)),
            (VIRTUAL_HEIGHT, night(SAND_DEEP, sd)),
        ],
    );
    // 모래 알갱이: 크기·밝기가 제각각인 작은 점 (늘 같은 자리)
    for i in 0..SAND_DOT_COUNT {
        let x = hash01(i * 3) * VIRTUAL_WIDTH;
        let y = ground_y + WET_SAND_THICKNESS + 4.0 + hash01(i * 3 + 1) * (VIRTUAL_HEIGHT - ground_y - WET_SAND_THICKNESS - 6.0);
        let r = 0.8 + hash01(i * 3 + 2) * 1.8;
        let c = if i % 4 == 0 { SAND_SPARK } else { SAND_DOT };
        draw_circle(x, y, r, night(c, sd));
    }
    // 조개 몇 개
    for (x, y, s) in [(90.0, 488.0, 1.0), (430.0, 520.0, 0.8), (650.0, 478.0, 1.1), (900.0, 505.0, 0.9)] {
        draw_shell(vec2(x, y), s, sd);
    }
    // 거품 아래로 얇게 번지는 물
    let surge = (time() * 0.9).sin() * 5.0;
    let wash = night(WASH, d * FOAM_NIGHT_AMOUNT);
    canvas.vgradient(0.0, VIRTUAL_WIDTH, &[(ground_y - 2.0 + surge, wash), (ground_y + 14.0 + surge, Color { a: 0.0, ..wash })]);
    draw_foam(ground_y, d);
}

/// 번호 → 0 이상 1 미만의 늘 같은 수 (모래 알갱이 자리)
fn hash01(n: u32) -> f32 {
    let mut x = n.wrapping_mul(0x2C1B_3C6D) ^ 0x297A_2D39;
    x ^= x >> 15;
    x = x.wrapping_mul(0x8EBC_6AF1);
    x ^= x >> 13;
    (x >> 8) as f32 / (1u32 << 24) as f32
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

/// 하늘 + 바다 + 모래. 타이틀·고르기·플레이가 모두 같은 하늘을 쓴다.
fn draw_background(assets: &Assets, look: &SkyLook, ground_y: f32) {
    sky::draw_sky(&mut GpuCanvas { art: &assets.sky }, look);
    draw_ground(&assets.sky, look, ground_y);
}

// ---- 타이틀 ----

pub fn draw_title(assets: &Assets, text: &TextPainter, ground_y: f32) {
    draw_background(assets, &SkyLook::new(0.0, &MENU_SKY, time()), ground_y);
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
    draw_background(assets, &SkyLook::new(0.0, &MENU_SKY, time()), ground_y);
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
    draw_background(assets, &SkyLook::new(d, &PLAY_SKY, time()), game.world.ground_y);
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
        assert!(sky::HORIZON_Y < crate::config::GROUND_Y);
        assert!(sky::HORIZON_Y + 40.0 < crate::config::GROUND_Y, "안개가 모래까지 내려오지 않는다");
    }
}
