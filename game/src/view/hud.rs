//! 정보판(HUD), 롤플레잉 게임처럼 놓는다 (US-091, D93, 임시-72). 특정 게임 그림은 쓰지 않고 우리 그림으로.
//! - 왼쪽 위: 고른 캐릭터 얼굴(아이 손그림을 잘라 씀), 빨간 체력 막대와 숫자, 작은 레벨, 그 아래 하얀 경험치 막대.
//! - 그 아래 한 줄: 효과 중인 것(자동차·스케이트·갑옷 남은 시간·엔더열매 힘) 작은 그림, 그 오른쪽에 가진 포션·독약·주사기총·엔더열매 칸.
//!   폰은 그 칸을 누르면 바로 쓴다 (`pocket_tap_rects`). PC는 칸에 1·2·R·3 키 글자.
//! - 오른쪽 위: N일째·코인, 그 아래 시계. 맨 오른쪽 구석은 가방 버튼 자리 (view::bag).
//! - 아래 가운데: 지금 든 무기 칸 하나.
//!
//! 재료·광물 개수와 무기 목록은 가방에서 본다. 값은 모두 logic이 준 그대로 보여 준다.

use super::assets::{face_source, Assets, ItemPicture, M6Art};
use super::coin::draw_coin;
use super::items::draw_item_picture;
use super::screen::VIRTUAL_WIDTH;
use super::shop::{armor_time_text, draw_armor, draw_pocket, draw_skates, Pocket};
use super::sprites::fit_size;
use super::text::TextPainter;
use super::touch::{draw_star, BACK_BUTTON};
use crate::config::{DAY_PERIODS, EXPERIENCE_PER_LEVEL};
use crate::logic::character::CharacterKind;
use crate::logic::clock::{Clock, Phase};
use crate::logic::game::Game;
use crate::logic::shop::Armor;
use crate::logic::weapon::Weapon;
use macroquad::prelude::*;

/// 알약 모양 판 (반투명 남색): 하늘이 밝든 어둡든 흰 글자가 잘 보이게.
pub const PANEL: Color = Color::new(0.10, 0.13, 0.32, 0.55);
const SUN_ICON: Color = Color::new(1.0, 0.82, 0.22, 1.0);
const MOON_ICON: Color = Color::new(1.0, 0.95, 0.70, 1.0);
const FAST_COLOR: Color = Color::new(1.0, 0.62, 0.15, 1.0);

// ---- 왼쪽 위: 얼굴 + 막대 ----
pub const FACE_CENTER: Vec2 = Vec2::new(52.0, 48.0);
pub const FACE_RADIUS: f32 = 38.0;
/// 얼굴 뒤 하늘색: 하얀 얼굴(종이색)이 판 색에 묻히지 않게
const FACE_BACK: Color = Color::new(0.68, 0.85, 1.0, 1.0);
/// 체력 막대 (아이 확정: 빨간색). 숫자(5/7)는 막대 안 가운데.
pub const HEALTH_BAR: Rect = Rect::new(100.0, 14.0, 210.0, 28.0);
const HEALTH_RED: Color = Color::new(0.93, 0.18, 0.22, 1.0);
const BAR_EMPTY: Color = Color::new(0.0, 0.0, 0.0, 0.40);
const BAR_LINE: Color = Color::new(0.10, 0.12, 0.25, 1.0);
const HEALTH_TEXT: f32 = 24.0;
const HEART_SPARKLE: Color = Color::new(1.0, 0.96, 0.70, 1.0);
/// 레벨 글 (부모님 확정: 작게). 체력 막대 오른쪽.
const LEVEL_TEXT: f32 = 22.0;
const LEVEL_GAP: f32 = 8.0;
/// 경험치 막대 (아이 확정: 하얀색, 밝은 하늘에서도 보이게 얇은 어두운 테두리 — 임시)
pub const XP_BAR: Rect = Rect::new(100.0, 48.0, 210.0, 10.0);
const XP_WHITE: Color = Color::new(1.0, 1.0, 1.0, 1.0);
const XP_LINE_WIDTH: f32 = 1.5;
/// 얼굴·막대 뒤 판 (왼쪽 끝, 위, 높이). 오른쪽 끝은 레벨 글 뒤.
const LEFT_PANEL_X: f32 = 10.0;
const LEFT_PANEL_Y: f32 = 8.0;
const LEFT_PANEL_H: f32 = 58.0;

// ---- 효과 줄과 쓰는 물건 칸 (체력 막대 아래) ----
/// 효과 그림·물건 칸의 가운데 높이
pub const ROW_MID_Y: f32 = 80.0;
pub const EFFECTS_LEFT: f32 = 100.0;
const EFFECT_ICON: f32 = 30.0;
const EFFECT_SLOT: f32 = 32.0;
/// 갑옷 남은 시간 글 자리 ("1분 12초"가 들어가는 폭). 늘 같은 폭이라 물건 칸 자리가 글자 폭 없이 정해진다.
const ARMOR_TIME_W: f32 = 84.0;
const ARMOR_TIME_TEXT: f32 = 22.0;
const EFFECT_GAP: f32 = 6.0;
/// 쓰는 물건 칸: 그림 칸, 누르는 자리(폰, 740×360에서도 48 화면 픽셀이 넘게), 칸 사이
pub const POCKET_CELL: f32 = 42.0;
pub const POCKET_TAP_W: f32 = 76.0;
pub const POCKET_TAP_H: f32 = 74.0;
const POCKET_STEP: f32 = 78.0;
const POCKET_COUNT_TEXT: f32 = 22.0;
const POCKET_KEY_SIZE: f32 = 18.0;
const POCKET_KEY_FILL: Color = Color::new(1.0, 0.62, 0.15, 1.0);
const POCKET_TAP_RING: Color = Color::new(1.0, 1.0, 1.0, 0.7);

// ---- 오른쪽 위: 날짜·코인, 시계 ----
/// 오른쪽 판들의 오른쪽 끝 (그 오른쪽은 가방 버튼 자리)
pub const RIGHT_COLUMN_RIGHT: f32 = 862.0;
const RIGHT_ROW_H: f32 = 40.0;
const RIGHT_ROW1_Y: f32 = 8.0;
const RIGHT_ROW2_Y: f32 = 52.0;
const RIGHT_TEXT: f32 = 26.0;
const RIGHT_PAD: f32 = 14.0;
const RIGHT_ICON_W: f32 = 34.0;
const RIGHT_GAP: f32 = 6.0;

// ---- 아래 가운데: 지금 든 무기 ----
pub const HELD_SLOT: Rect = Rect::new(VIRTUAL_WIDTH / 2.0 - 32.0, 466.0, 64.0, 64.0);
const HELD_ICON: f32 = 44.0;
const FIST: Color = Color::new(1.0, 0.93, 0.78, 1.0);
const FIST_LINE: Color = Color::new(0.13, 0.16, 0.36, 1.0);

/// 시각(00:00부터 분) → 시간대 이름 (D31, `config::DAY_PERIODS` 표).
pub fn period_name(minutes: u32) -> &'static str {
    DAY_PERIODS.iter().rev().find(|(start, _)| minutes / 60 >= *start).map_or(DAY_PERIODS[0].1, |(_, name)| name)
}

/// 24시간 시계의 (시, 분) → "HH:MM". 두 자리씩. 시는 23, 분은 59를 넘지 않는다.
pub fn hh_mm(hour: u32, minute: u32) -> String {
    format!("{:02}:{:02}", hour.min(23), minute.min(59))
}

/// 둥근 끝 판.
pub(super) fn draw_pill(x: f32, y: f32, w: f32, h: f32, color: Color) {
    let r = h / 2.0;
    draw_rectangle(x + r, y, (w - h).max(0.0), h, color);
    // 반원 두 개 대신 원 두 개를 그리면 겹치는 곳이 더 진해지므로 반원 부채를 그린다.
    draw_half_disc(vec2(x + r, y + r), r, true, color);
    draw_half_disc(vec2(x + w - r, y + r), r, false, color);
}

fn draw_half_disc(c: Vec2, r: f32, left: bool, color: Color) {
    const STEPS: usize = 16;
    let start = if left { std::f32::consts::FRAC_PI_2 } else { -std::f32::consts::FRAC_PI_2 };
    for i in 0..STEPS {
        let a0 = start + i as f32 / STEPS as f32 * std::f32::consts::PI;
        let a1 = start + (i + 1) as f32 / STEPS as f32 * std::f32::consts::PI;
        draw_triangle(c, c + vec2(a0.cos(), a0.sin()) * r, c + vec2(a1.cos(), a1.sin()) * r, color);
    }
}

/// 해 아이콘: 동그라미 + 빛살 8개.
fn draw_sun_icon(c: Vec2, r: f32) {
    for i in 0..8 {
        let a = i as f32 / 8.0 * std::f32::consts::TAU;
        let dir = vec2(a.cos(), a.sin());
        draw_line(c.x + dir.x * r * 1.25, c.y + dir.y * r * 1.25, c.x + dir.x * r * 1.7, c.y + dir.y * r * 1.7, 2.5, SUN_ICON);
    }
    draw_circle(c.x, c.y, r, SUN_ICON);
}

/// 초승달 아이콘: 큰 원에서 오른쪽으로 비킨 원을 뺀 모양을 가로줄로 칠한다 (판 색과 상관없이 깔끔).
fn draw_moon_icon(c: Vec2, r: f32) {
    let cut_r = r * 0.85;
    let cut_dx = r * 0.55;
    let rows = (r * 2.0).ceil() as i32;
    for i in 0..rows {
        let y = -r + i as f32 + 0.5;
        let a = (r * r - y * y).max(0.0).sqrt();
        let right = if y.abs() < cut_r { (cut_dx - (cut_r * cut_r - y * y).sqrt()).min(a) } else { a };
        if right > -a {
            draw_rectangle(c.x - a, c.y + y - 0.5, right + a, 1.0, MOON_ICON);
        }
    }
}

/// 하트 하나: 동그라미 두 개 + 아래로 뾰족한 세모. `size`는 폭.
pub(super) fn draw_heart(c: Vec2, size: f32, color: Color) {
    let bump = size * 0.27;
    draw_circle(c.x - size * 0.24, c.y - size * 0.12, bump, color);
    draw_circle(c.x + size * 0.24, c.y - size * 0.12, bump, color);
    draw_triangle(c + vec2(-size * 0.5, -size * 0.02), c + vec2(size * 0.5, -size * 0.02), c + vec2(0.0, size * 0.5), color);
}

/// 체력 막대가 찬 비율 (0~1): 지금 체력 / 최대 체력.
pub fn health_share(hp: u32, max_hp: u32) -> f32 {
    (hp as f32 / max_hp.max(1) as f32).clamp(0.0, 1.0)
}

/// 경험치 막대가 찬 비율 (0~1): 다음 레벨까지 모은 만큼. 레벨이 오르면 logic이 경험치를 0으로 돌려서 막대가 빈다.
pub fn experience_share(experience: u32) -> f32 {
    (experience as f32 / EXPERIENCE_PER_LEVEL.max(1) as f32).clamp(0.0, 1.0)
}

/// 캐릭터마다 얼굴 둘레 색: Niro 노랑, Ellie 파랑, Logan 빨강 (원화 옷 색).
fn face_ring(kind: CharacterKind) -> Color {
    match kind {
        CharacterKind::Niro => Color::new(1.0, 0.82, 0.20, 1.0),
        CharacterKind::Ellie => Color::new(0.35, 0.62, 1.0, 1.0),
        CharacterKind::Logan => Color::new(0.90, 0.20, 0.25, 1.0),
    }
}

/// 왼쪽 위 고른 캐릭터 얼굴: 동그란 판 안에 원화 얼굴을 잘라 넣는다.
pub fn draw_face(assets: &Assets, kind: CharacterKind) {
    let c = FACE_CENTER;
    draw_circle(c.x, c.y, FACE_RADIUS, FACE_BACK);
    let source = face_source(kind);
    let fit = FACE_RADIUS * 1.7 / source.w.max(source.h);
    let size = vec2(source.w, source.h) * fit;
    draw_texture_ex(
        assets.character(kind),
        c.x - size.x / 2.0,
        c.y - size.y / 2.0 + 2.0,
        WHITE,
        DrawTextureParams { dest_size: Some(size), source: Some(source), ..Default::default() },
    );
    draw_circle_lines(c.x, c.y, FACE_RADIUS, 4.0, face_ring(kind));
}

/// 왼쪽 위 얼굴·막대 판의 오른쪽 끝 (레벨 글 뒤). 미션 글이 이 오른쪽에 들어갈 수 있는지 볼 때도 쓴다 (US-086).
pub fn left_panel_right(text: &TextPainter, level: u32) -> f32 {
    HEALTH_BAR.right() + LEVEL_GAP + text.width(&format!("레벨 {level}"), LEVEL_TEXT) + 14.0
}

/// 오른쪽 위 첫 줄(N일째·코인) 판의 왼쪽 끝. `draw_coins_and_clock`과 같은 셈 (US-086 미션 글 자리).
pub fn right_row1_left(text: &TextPainter, day: u32, coins: u32) -> f32 {
    let coin_w = RIGHT_PAD + RIGHT_ICON_W + text.width(&coins.to_string(), RIGHT_TEXT) + RIGHT_PAD;
    let day_w = RIGHT_PAD * 2.0 + text.width(&format!("{day}일째"), RIGHT_TEXT);
    RIGHT_COLUMN_RIGHT - coin_w - RIGHT_GAP - day_w
}

/// 줄어든 체력 자리가 옅게 남았다가 줄어드는 색 (미션 실패 등으로 체력 막대 1칸이 주는 모습, D103)
const HEALTH_LOST: Color = Color::new(1.0, 0.80, 0.82, 0.9);

/// 체력 막대 + 숫자, 작은 레벨, 하얀 경험치 막대. `sparkle`(1 막 시작 ~ 0 끝)이면 체력 막대 위로 별이 반짝인다 (D27).
/// `lost_hp`(0이면 없음): 방금 준 체력만큼 막대 끝에 옅은 칸이 남아 줄어든다.
pub fn draw_bars(text: &TextPainter, hp: u32, max_hp: u32, sparkle: f32, level: u32, experience: u32, lost_hp: f32) {
    let level_label = format!("레벨 {level}");
    let level_x = HEALTH_BAR.right() + LEVEL_GAP;
    let panel_right = left_panel_right(text, level);
    draw_pill(LEFT_PANEL_X, LEFT_PANEL_Y, panel_right - LEFT_PANEL_X, LEFT_PANEL_H, PANEL);

    let r = HEALTH_BAR;
    draw_rectangle(r.x, r.y, r.w, r.h, BAR_EMPTY);
    let filled = r.w * health_share(hp, max_hp);
    draw_rectangle(r.x, r.y, filled, r.h, HEALTH_RED);
    if lost_hp > 0.0 {
        let lost = (r.w * lost_hp / max_hp.max(1) as f32).min(r.w - filled);
        draw_rectangle(r.x + filled, r.y, lost, r.h, HEALTH_LOST);
    }
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, BAR_LINE);
    text.centered(&format!("{hp}/{max_hp}"), r.center().x, r.bottom() - 6.0, HEALTH_TEXT, WHITE);
    if sparkle > 0.0 {
        for i in 0..5 {
            let p = vec2(r.x + r.w * (i as f32 + 0.5) / 5.0, r.center().y);
            draw_star(p, 10.0 * sparkle, sparkle * 4.0 + i as f32, Color { a: sparkle, ..HEART_SPARKLE });
        }
    }
    text.left(&level_label, level_x, r.bottom() - 6.0, LEVEL_TEXT, WHITE);

    let x = XP_BAR;
    draw_rectangle(x.x, x.y, x.w, x.h, BAR_EMPTY);
    draw_rectangle(x.x, x.y, x.w * experience_share(experience), x.h, XP_WHITE);
    draw_rectangle_lines(x.x, x.y, x.w, x.h, XP_LINE_WIDTH, BAR_LINE);
}

/// 효과 줄에 넣을 것: 지금 효과 중인 것만 (타고 있는 자동차, 신은 스케이트, 입은 갑옷과 남은 시간, 엔더열매 힘).
#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Car,
    Skates,
    Armor(Armor, String),
    EnderPower,
}

/// 지금 보일 효과들 (없으면 빈 목록). `time_scale`은 갑옷 남은 시간을 실제 기다릴 시간으로 바꾸는 데 쓴다.
pub fn effects_now(game: &Game, time_scale: f32) -> Vec<Effect> {
    let bag = &game.gathering.bag;
    let mut list = Vec::new();
    if bag.riding_car {
        list.push(Effect::Car);
    }
    if game.shop.has_skates() {
        list.push(Effect::Skates);
    }
    if let Some(armor) = game.shop.armor() {
        list.push(Effect::Armor(armor, armor_time_text(game.shop.armor_seconds_left(), time_scale)));
    }
    if bag.has_ender_power {
        list.push(Effect::EnderPower);
    }
    list
}

fn effect_width(effect: &Effect) -> f32 {
    match effect {
        Effect::Armor(..) => EFFECT_SLOT + 4.0 + ARMOR_TIME_W,
        _ => EFFECT_SLOT,
    }
}

/// 효과 줄 폭 (0이면 줄이 없다). 글자 폭을 재지 않아서 그리기 전(입력 처리)에도 쓸 수 있다.
fn effects_width(effects: &[Effect]) -> f32 {
    if effects.is_empty() {
        return 0.0;
    }
    effects.iter().map(effect_width).sum::<f32>() + EFFECT_GAP * (effects.len() - 1) as f32
}

/// 가진 쓰는 물건 (`Pocket::ALL` 순서, 0개는 빠진다).
pub fn owned_pockets(game: &Game) -> Vec<Pocket> {
    Pocket::ALL.into_iter().filter(|p| p.count(game) > 0).collect()
}

/// 물건 칸 하나와 그 누르는 자리. `effects_w`는 효과 줄 폭, `index`는 왼쪽부터 몇 번째인지.
fn pocket_cell(effects_w: f32, index: usize) -> (Rect, Rect) {
    let tap_left = EFFECTS_LEFT + if effects_w > 0.0 { effects_w + EFFECT_GAP } else { 0.0 } + POCKET_STEP * index as f32;
    let tap = Rect::new(tap_left, ROW_MID_Y - POCKET_TAP_H / 2.0, POCKET_TAP_W, POCKET_TAP_H);
    let cell = Rect::new(tap.center().x - POCKET_CELL / 2.0, ROW_MID_Y - POCKET_CELL / 2.0, POCKET_CELL, POCKET_CELL);
    (cell, tap)
}

/// 폰에서 누르면 쓰는 자리 (가진 것만, 효과 줄 오른쪽). 그림 칸보다 크다.
pub fn pocket_tap_rects(game: &Game) -> Vec<(Pocket, Rect)> {
    let w = effects_width(&effects_now(game, 1.0));
    owned_pockets(game).into_iter().enumerate().map(|(i, p)| (p, pocket_cell(w, i).1)).collect()
}

/// 효과 그림 하나.
fn draw_effect(text: &TextPainter, art: &M6Art, effect: &Effect, x: f32, t: f32) {
    let c = vec2(x + EFFECT_SLOT / 2.0, ROW_MID_Y);
    match effect {
        Effect::Car => draw_item_picture(art, ItemPicture::Car, c, EFFECT_ICON * 1.2),
        Effect::Skates => draw_skates(c, EFFECT_ICON),
        Effect::Armor(armor, time) => {
            draw_armor(*armor, c, EFFECT_ICON, t);
            text.left(time, x + EFFECT_SLOT + 4.0, ROW_MID_Y + 8.0, ARMOR_TIME_TEXT, WHITE);
        }
        Effect::EnderPower => {
            // 엔더열매 힘 (아이: 금빛 기운): 금빛 동그라미 위에 열매
            let p = 0.5 + 0.5 * (t * 4.0).sin();
            draw_circle(c.x, c.y, EFFECT_ICON * 0.55, Color::new(1.0, 0.82, 0.20, 0.35 + 0.25 * p));
            draw_item_picture(art, ItemPicture::EnderFruit, c, EFFECT_ICON * 0.9);
        }
    }
}

/// 체력 막대 아래 한 줄: 효과 그림들, 그 오른쪽 가진 물건 칸 (개수와 함께).
/// 폰(`touch_mode`)이면 "눌러요" 동그라미, PC면 쓰는 키 글자(1·2·R·3).
pub fn draw_effects_and_pockets(text: &TextPainter, game: &Game, art: &M6Art, time_scale: f32, touch_mode: bool) {
    let t = get_time() as f32;
    let effects = effects_now(game, time_scale);
    let ew = effects_width(&effects);
    if ew > 0.0 {
        draw_pill(EFFECTS_LEFT - 8.0, ROW_MID_Y - 20.0, ew + 16.0, 40.0, PANEL);
        let mut x = EFFECTS_LEFT;
        for effect in &effects {
            draw_effect(text, art, effect, x, t);
            x += effect_width(effect) + EFFECT_GAP;
        }
    }
    for (i, pocket) in owned_pockets(game).into_iter().enumerate() {
        let (cell, _) = pocket_cell(ew, i);
        draw_rectangle(cell.x, cell.y, cell.w, cell.h, PANEL);
        let c = cell.center();
        if touch_mode {
            draw_circle_lines(c.x, c.y, POCKET_CELL / 2.0 + 3.0, 3.0, POCKET_TAP_RING);
        }
        draw_pocket(pocket, art, c, POCKET_CELL * 0.78);
        // 개수 (주사기총도 여러 개 가질 수 있다)
        text.left(&pocket.count(game).to_string(), cell.right() - 6.0, cell.bottom() + 4.0, POCKET_COUNT_TEXT, WHITE);
        if !touch_mode {
            let key = vec2(cell.x + 2.0, cell.y + 4.0);
            draw_circle(key.x, key.y, POCKET_KEY_SIZE * 0.55, POCKET_KEY_FILL);
            text.centered(super::input::pocket_key_label(pocket), key.x, key.y + POCKET_KEY_SIZE * 0.36, POCKET_KEY_SIZE, WHITE);
        }
    }
}

/// 오른쪽 판 한 칸: 오른쪽 끝을 `right`에 붙인다. 돌려주는 값: 판 왼쪽 끝.
fn right_pill(text: &TextPainter, right: f32, y: f32, label: &str, icon: impl FnOnce(Vec2)) -> f32 {
    let w = RIGHT_PAD + RIGHT_ICON_W + text.width(label, RIGHT_TEXT) + RIGHT_PAD;
    let x = right - w;
    draw_pill(x, y, w, RIGHT_ROW_H, PANEL);
    icon(vec2(x + RIGHT_PAD + RIGHT_ICON_W / 2.0 - 4.0, y + RIGHT_ROW_H / 2.0));
    text.left(label, x + RIGHT_PAD + RIGHT_ICON_W, y + RIGHT_ROW_H - 11.0, RIGHT_TEXT, WHITE);
    x
}

/// 오른쪽 위: (N일째 · 코인) 한 줄, 그 아래 시계 (시간대 이름 + 24시간 시계). 빨리감기(F, 숨은 기능)는 화면에 안 보인다 (2026-11-15).
pub fn draw_coins_and_clock(text: &TextPainter, clock: &Clock, coins: u32) {
    let t = get_time() as f32;
    let coin_left = right_pill(text, RIGHT_COLUMN_RIGHT, RIGHT_ROW1_Y, &coins.to_string(), |c| draw_coin(c, 13.0, 0.0, t));
    let day = format!("{}일째", clock.day_number());
    let w = RIGHT_PAD * 2.0 + text.width(&day, RIGHT_TEXT);
    draw_pill(coin_left - RIGHT_GAP - w, RIGHT_ROW1_Y, w, RIGHT_ROW_H, PANEL);
    text.left(&day, coin_left - RIGHT_GAP - w + RIGHT_PAD, RIGHT_ROW1_Y + RIGHT_ROW_H - 11.0, RIGHT_TEXT, WHITE);

    let (hour, minute) = clock.hour_minute();
    let time_text = format!("{} {}", period_name(clock.time_of_day_minutes()), hh_mm(hour, minute));
    let phase = clock.phase();
    right_pill(text, RIGHT_COLUMN_RIGHT, RIGHT_ROW2_Y, &time_text, |c| match phase {
        Phase::Day => draw_sun_icon(c, 8.0),
        Phase::Night => draw_moon_icon(c, 11.0),
    });
}

/// 아래 가운데: 지금 든 무기 칸 하나. 맨손이면 주먹.
/// `level`: 그 무기의 강화 횟수 (화면에는 횟수+1을 "Lv n"으로, 맨손은 없음), `flash`: 강화 반짝임 (1~0).
pub fn draw_held_weapon(text: &TextPainter, art: &M6Art, held: Weapon, level: u32, flash: f32) {
    super::upgrade::draw_held_flash(flash, get_time() as f32);
    let r = HELD_SLOT;
    let c = r.center();
    draw_circle(c.x, c.y, r.w / 2.0, PANEL);
    draw_circle_lines(c.x, c.y, r.w / 2.0 - 1.0, 3.0, FAST_COLOR);
    match art.weapon(held) {
        Some(texture) => {
            let size = fit_size(texture, HELD_ICON);
            draw_texture_ex(texture, c.x - size.x / 2.0, c.y - size.y / 2.0, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
        }
        None => {
            draw_circle(c.x, c.y, 14.0, FIST_LINE);
            draw_circle(c.x, c.y, 11.5, FIST);
        }
    }
    super::upgrade::draw_held_level(text, held, level);
}

/// 위 정보판의 아래 끝. 큰 알림 글은 그 아래에 놓인다 (BUG-031, 임시-78).
/// `row`: 효과 줄이나 물건 칸이 있다. `touch_mode`: 폰 "멈춤" 버튼(왼쪽, 정보판 아래)까지 넣는다.
pub fn hud_bottom(row: bool, touch_mode: bool) -> f32 {
    let left = if row { ROW_MID_Y + POCKET_CELL / 2.0 } else { FACE_CENTER.y + FACE_RADIUS };
    let right = RIGHT_ROW2_Y + RIGHT_ROW_H;
    let pause = if touch_mode { BACK_BUTTON.bottom() } else { 0.0 };
    left.max(right).max(pause)
}

/// 효과 줄이나 물건 칸이 하나라도 있으면 true.
pub fn has_row(game: &Game) -> bool {
    !effects_now(game, 1.0).is_empty() || !owned_pockets(game).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::shop::{Potion, ShopItem};
    use crate::logic::world::World;
    use crate::view::bag::bag_button_rect;
    use crate::view::touch::{button_touches_rect, play_buttons_with_mine, JOYSTICK_RADIUS};

    #[test]
    fn clock_text_is_hh_mm_two_digits_each() {
        assert_eq!(hh_mm(6, 0), "06:00");
        assert_eq!(hh_mm(9, 36), "09:36");
        assert_eq!(hh_mm(0, 12), "00:12");
        assert_eq!(hh_mm(0, 0), "00:00");
        assert_eq!(hh_mm(23, 59), "23:59");
        assert_eq!(hh_mm(24, 60), "23:59", "24:00이나 세 자리는 안 나온다");
    }

    #[test]
    fn period_names_change_on_the_hour() {
        for (hour, minute, name) in [
            (0, 0, "새벽"),
            (5, 59, "새벽"),
            (6, 0, "아침"),
            (9, 59, "아침"),
            (10, 0, "낮"),
            (13, 59, "낮"),
            (14, 0, "오후"),
            (17, 59, "오후"),
            (18, 0, "저녁"),
            (20, 59, "저녁"),
            (21, 0, "밤"),
            (23, 59, "밤"),
        ] {
            assert_eq!(period_name(hour * 60 + minute), name, "{hour}:{minute}");
        }
    }

    #[test]
    fn clock_text_follows_logic_clock() {
        let show = |c: &Clock| {
            let (h, m) = c.hour_minute();
            hh_mm(h, m)
        };
        let mut c = Clock::new();
        assert_eq!(show(&c), "06:00");
        c.update(299.9);
        assert_eq!(show(&c), "17:59");
        c.update(0.1);
        assert_eq!(show(&c), "18:00");
        c.update(300.0);
        assert_eq!(show(&c), "00:00");
    }

    #[test]
    fn health_bar_fills_by_hp_and_experience_bar_empties_on_level_up() {
        assert_eq!(health_share(5, 7), 5.0 / 7.0);
        assert_eq!(health_share(7, 7), 1.0);
        assert_eq!(health_share(0, 7), 0.0);
        assert_eq!(health_share(3, 0), 1.0, "최대가 0이어도 넘치지 않는다");
        assert_eq!(experience_share(0), 0.0);
        assert_eq!(experience_share(2), 2.0 / EXPERIENCE_PER_LEVEL as f32);
        assert_eq!(experience_share(EXPERIENCE_PER_LEVEL + 3), 1.0);
        // 실제 게임: 경험치가 다 차서 레벨이 오르면 막대가 비어서 다시 찬다
        let mut g = Game::on_world(CharacterKind::Niro, 7, World::m1());
        let before = g.combat.level;
        g.combat.level_up();
        assert_eq!(g.combat.level, before + 1);
        assert_eq!(experience_share(g.combat.experience), 0.0);
    }

    #[test]
    fn colors_are_the_kids_choice() {
        const { assert!(HEALTH_RED.r > 0.8 && HEALTH_RED.g < 0.3 && HEALTH_RED.b < 0.3, "체력 막대 빨간색") };
        const { assert!(XP_WHITE.r == 1.0 && XP_WHITE.g == 1.0 && XP_WHITE.b == 1.0, "경험치 막대 하얀색") };
        const { assert!(BAR_LINE.r < 0.3 && BAR_LINE.g < 0.3 && BAR_LINE.b < 0.3 && XP_LINE_WIDTH <= 2.0, "얇은 어두운 테두리") };
    }

    fn full_game() -> Game {
        let mut g = Game::on_world(CharacterKind::Niro, 7, World::m1());
        g.wallet.add(3000);
        for item in [ShopItem::Skates, ShopItem::RainbowArmor, ShopItem::HealthPotion, ShopItem::HealthPotion, ShopItem::PoisonPotion] {
            g.buy(item).unwrap();
        }
        let bag = &mut g.gathering.bag;
        bag.riding_car = true;
        bag.has_ender_power = true;
        bag.syringe_guns = 1;
        bag.ender_fruits = 2;
        g
    }

    #[test]
    fn effects_show_only_what_is_on() {
        let mut g = Game::on_world(CharacterKind::Ellie, 7, World::m1());
        assert!(effects_now(&g, 1.0).is_empty() && !has_row(&g), "아무것도 없으면 줄도 없다");
        g.gathering.bag.riding_car = true;
        assert_eq!(effects_now(&g, 1.0), vec![Effect::Car]);
        g.gathering.bag.riding_car = false;
        g.gathering.bag.has_ender_power = true;
        assert_eq!(effects_now(&g, 1.0), vec![Effect::EnderPower]);
        let all = effects_now(&full_game(), 10.0);
        assert_eq!(all.len(), 4);
        assert!(matches!(&all[2], Effect::Armor(Armor::Rainbow, time) if time == "8초"), "갑옷은 남은 시간도: {all:?}");
    }

    #[test]
    fn pocket_cells_show_only_what_you_have() {
        let mut g = Game::on_world(CharacterKind::Niro, 7, World::m1());
        assert!(pocket_tap_rects(&g).is_empty());
        g.gathering.bag.ender_fruits = 1;
        assert_eq!(pocket_tap_rects(&g).iter().map(|(p, _)| *p).collect::<Vec<_>>(), vec![Pocket::EnderFruit]);
        g.gathering.bag.ender_fruits = 0;
        assert!(pocket_tap_rects(&g).is_empty(), "0개가 되면 사라진다");
        let all = pocket_tap_rects(&full_game());
        assert_eq!(all.iter().map(|(p, _)| *p).collect::<Vec<_>>(), Pocket::ALL.to_vec());
        assert_eq!(all[0].0, Pocket::Potion(Potion::Health));
    }

    /// 화면 세 크기 (가로, 세로): 가상 1픽셀이 몇 화면 픽셀인지
    const SCREENS: [(f32, f32); 3] = [(960.0, 540.0), (812.0, 375.0), (740.0, 360.0)];

    fn scale(w: f32, h: f32) -> f32 {
        f32::min(w / VIRTUAL_WIDTH, h / 540.0)
    }

    #[test]
    fn phone_pocket_spots_are_big_and_clear_of_everything() {
        // 아이템을 다 가졌을 때, 그리고 효과 없이 물건만 가졌을 때 (물건 칸이 왼쪽 끝으로 붙는다)
        let pockets_only = {
            let mut g = Game::on_world(CharacterKind::Niro, 7, World::m1());
            g.wallet.add(1000);
            g.buy(ShopItem::HealthPotion).unwrap();
            g.buy(ShopItem::PoisonPotion).unwrap();
            g.gathering.bag.syringe_guns = 2;
            g.gathering.bag.ender_fruits = 1;
            g
        };
        for g in [full_game(), pockets_only] {
            let rects = pocket_tap_rects(&g);
            let effects = effects_now(&g, 1.0);
            let effects_end = EFFECTS_LEFT + effects_width(&effects);
            for (i, (pocket, r)) in rects.iter().enumerate() {
                for (w, h) in SCREENS {
                    assert!(r.w.min(r.h) * scale(w, h) >= 48.0, "{pocket:?} {w}×{h}");
                }
                assert!(r.y > HEALTH_BAR.bottom(), "체력 막대와 안 겹친다");
                assert!(effects.is_empty() || r.x >= effects_end, "효과 그림 오른쪽");
                assert!(!r.overlaps(&BACK_BUTTON), "멈춤 버튼과 안 겹친다");
                assert!(!r.overlaps(&bag_button_rect(true)), "가방 버튼과 안 겹친다");
                // 오른쪽 위 시계 판("아침 06:00", 26 크기)은 약 x 670부터라 그보다 왼쪽 (화면에서 눈으로 한 번 더 본다)
                assert!(r.right() <= 660.0, "오른쪽 위 시계 판보다 왼쪽: {}", r.right());
                for b in play_buttons_with_mine(true, true) {
                    assert!(!button_touches_rect(&b, *r), "{pocket:?}가 {:?} 버튼과 겹친다", b.kind);
                }
                for (_, other) in &rects[i + 1..] {
                    assert!(!r.overlaps(other), "누르는 자리끼리 안 겹친다");
                }
            }
        }
    }

    #[test]
    fn hud_is_about_one_line_and_leaves_the_play_field() {
        // 아이템을 다 가져도 정보판은 위쪽 약 1/5 안이다 (폰은 멈춤 버튼까지)
        assert!(hud_bottom(true, false) <= 104.0);
        assert!(hud_bottom(true, true) <= 180.0);
        let head = crate::config::GROUND_Y - crate::config::PLAYER_HEIGHT;
        assert!(hud_bottom(true, true) < head - 100.0, "서 있는 캐릭터와 한참 떨어져 있다");
        // 아래 무기 칸은 화면 안, 땅 아래 모래 위, 폰 버튼·조이스틱 안내와 안 겹친다
        assert!(HELD_SLOT.y > crate::config::GROUND_Y && HELD_SLOT.bottom() <= 540.0);
        for b in play_buttons_with_mine(true, true) {
            assert!(!button_touches_rect(&b, HELD_SLOT), "{:?}", b.kind);
        }
        const { assert!(HELD_SLOT.x > 150.0 + JOYSTICK_RADIUS) };
        // 오른쪽 판은 가방 버튼 왼쪽
        assert!(RIGHT_COLUMN_RIGHT < bag_button_rect(true).x && RIGHT_COLUMN_RIGHT < bag_button_rect(false).x);
        // 얼굴·체력 판은 멈춤 버튼 위
        const { assert!(FACE_CENTER.y + FACE_RADIUS < BACK_BUTTON.y && ROW_MID_Y + POCKET_CELL / 2.0 < BACK_BUTTON.y) };
    }

    #[test]
    fn small_text_is_still_readable_on_a_small_phone() {
        for size in [HEALTH_TEXT, LEVEL_TEXT, ARMOR_TIME_TEXT, POCKET_COUNT_TEXT, RIGHT_TEXT] {
            assert!(size * scale(740.0, 360.0) >= 14.0, "{size}");
        }
    }
}
