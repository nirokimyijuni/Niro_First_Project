//! 정보판(HUD): 화면 맨 위에 크게. 왼쪽 = 낮/밤 + 남은 시간, 가운데 = N일째 · 버틴 날, 오른쪽 = 코인.
//! 빨리감기(D12)가 켜져 있으면 정보판 줄 바로 아래 오른쪽(해 그림 왼쪽)에 "빨리감기 x10" 판.
//! 값은 모두 logic(clock, wallet)이 준 그대로 보여 준다.

use super::coin::draw_coin;
use super::screen::VIRTUAL_WIDTH;
use super::text::TextPainter;
use crate::config::DEV_FAST_FORWARD_SCALE;
use crate::logic::clock::{Clock, Phase};
use macroquad::prelude::*;

/// 알약 모양 판 (반투명 남색): 하늘이 밝든 어둡든 흰 글자가 잘 보이게.
const PANEL: Color = Color::new(0.10, 0.13, 0.32, 0.55);
const SUN_ICON: Color = Color::new(1.0, 0.82, 0.22, 1.0);
const MOON_ICON: Color = Color::new(1.0, 0.95, 0.70, 1.0);
const FAST_COLOR: Color = Color::new(1.0, 0.62, 0.15, 1.0);

const MARGIN: f32 = 12.0;
const PANEL_TOP: f32 = 10.0;
const PANEL_HEIGHT: f32 = 56.0;
/// 판 안 글자 바닥선 (판 위쪽에서)
const TEXT_BASELINE: f32 = 41.0;
const BIG_TEXT: f32 = 38.0;
const MID_TEXT: f32 = 34.0;
/// 판 왼쪽 끝 → 아이콘 가운데, 판 왼쪽 끝 → 글자 시작
const ICON_X: f32 = 32.0;
const ICON_TEXT_GAP: f32 = 58.0;
const PANEL_PAD_RIGHT: f32 = 20.0;
/// 빨리감기 판: 정보판 줄 바로 아래. 해(가운데 x 850, 빛 반지름 약 59)와 겹치지 않게
/// 판 오른쪽 끝을 해 빛 왼쪽(791)보다 조금 왼쪽에 맞춘다.
const FAST_PANEL_TOP: f32 = PANEL_TOP + PANEL_HEIGHT + 8.0;
const FAST_PANEL_HEIGHT: f32 = 48.0;
const FAST_PANEL_RIGHT: f32 = 775.0;
const FAST_ICON_H: f32 = 24.0;
const FAST_PAD_LEFT: f32 = 20.0;
const FAST_ICON_TEXT_GAP: f32 = 12.0;

/// 남은 초 → "MM:SS". 올림해서 시작할 때 10:00, 끝나기 직전 00:01. 0 아래·99:59 위로는 안 나간다.
pub fn mm_ss(seconds_left: f32) -> String {
    let total = seconds_left.max(0.0).ceil().min(99.0 * 60.0 + 59.0) as u32;
    format!("{:02}:{:02}", total / 60, total % 60)
}

/// 둥근 끝 판.
fn draw_pill(x: f32, y: f32, w: f32, h: f32, color: Color) {
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
        draw_line(c.x + dir.x * r * 1.25, c.y + dir.y * r * 1.25, c.x + dir.x * r * 1.7, c.y + dir.y * r * 1.7, 3.0, SUN_ICON);
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

/// 빨리감기 아이콘 ⏩: 세모 두 개.
fn draw_fast_icon(x: f32, mid_y: f32, h: f32, color: Color) {
    for dx in [0.0, h * 0.7] {
        draw_triangle(vec2(x + dx, mid_y - h / 2.0), vec2(x + dx, mid_y + h / 2.0), vec2(x + dx + h * 0.7, mid_y), color);
    }
}

pub fn draw_hud(text: &TextPainter, clock: &Clock, coins: u32, fast_forward: bool) {
    let mid_y = PANEL_TOP + PANEL_HEIGHT / 2.0;
    let baseline = PANEL_TOP + TEXT_BASELINE;

    // 왼쪽: 낮/밤 + 남은 시간. 판 폭은 숫자가 바뀌어도 흔들리지 않게 고정 견본으로 잰다.
    let label = match clock.phase() {
        Phase::Day => "낮",
        Phase::Night => "밤",
    };
    let is_night = clock.is_night();
    let time_text = format!("{label} {}", mm_ss(clock.seconds_left_in_phase()));
    let left_w = ICON_TEXT_GAP + text.width("밤 00:00", BIG_TEXT) + PANEL_PAD_RIGHT;
    draw_pill(MARGIN, PANEL_TOP, left_w, PANEL_HEIGHT, PANEL);
    let icon = vec2(MARGIN + ICON_X, mid_y);
    if is_night {
        draw_moon_icon(icon, 17.0);
    } else {
        draw_sun_icon(icon, 12.0);
    }
    text.left(&time_text, MARGIN + ICON_TEXT_GAP, baseline, BIG_TEXT, WHITE);

    // 가운데: N일째   버틴 날 N
    let day_text = format!("{}일째    버틴 날 {}", clock.day_number(), clock.nights_survived());
    let center_w = text.width(&day_text, MID_TEXT) + 48.0;
    draw_pill(VIRTUAL_WIDTH / 2.0 - center_w / 2.0, PANEL_TOP, center_w, PANEL_HEIGHT, PANEL);
    text.centered(&day_text, VIRTUAL_WIDTH / 2.0, baseline - 2.0, MID_TEXT, WHITE);

    // 오른쪽: 코인 아이콘 + 숫자 (오른쪽 끝에 붙인다)
    let coin_text = coins.to_string();
    let right_w = ICON_TEXT_GAP + text.width(&coin_text, BIG_TEXT).max(text.width("0", BIG_TEXT)) + PANEL_PAD_RIGHT;
    let right_x = VIRTUAL_WIDTH - MARGIN - right_w;
    draw_pill(right_x, PANEL_TOP, right_w, PANEL_HEIGHT, PANEL);
    draw_coin(vec2(right_x + ICON_X, mid_y), 16.0, 0.0, get_time() as f32);
    text.left(&coin_text, right_x + ICON_TEXT_GAP, baseline, BIG_TEXT, WHITE);

    if fast_forward {
        let label = format!("빨리감기 x{}", DEV_FAST_FORWARD_SCALE as u32);
        // 다른 정보판처럼 반투명 남색 판 위에: 낮·노을 하늘에서도 또렷하게.
        let icon_w = FAST_ICON_H * 1.4;
        let w = FAST_PAD_LEFT + icon_w + FAST_ICON_TEXT_GAP + text.width(&label, MID_TEXT) + PANEL_PAD_RIGHT;
        let x = FAST_PANEL_RIGHT - w;
        let fast_mid_y = FAST_PANEL_TOP + FAST_PANEL_HEIGHT / 2.0;
        draw_pill(x, FAST_PANEL_TOP, w, FAST_PANEL_HEIGHT, PANEL);
        draw_fast_icon(x + FAST_PAD_LEFT, fast_mid_y, FAST_ICON_H, FAST_COLOR);
        text.left(&label, x + FAST_PAD_LEFT + icon_w + FAST_ICON_TEXT_GAP, fast_mid_y + 11.0, MID_TEXT, WHITE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_text_is_always_two_digits_each() {
        assert_eq!(mm_ss(600.0), "10:00");
        assert_eq!(mm_ss(599.5), "10:00");
        assert_eq!(mm_ss(599.0), "09:59");
        assert_eq!(mm_ss(545.0), "09:05");
        assert_eq!(mm_ss(6.2), "00:07");
        assert_eq!(mm_ss(0.0), "00:00");
        assert_eq!(mm_ss(-3.0), "00:00");
    }
}
