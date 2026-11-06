//! 정보판(HUD): 화면 맨 위에 크게. 왼쪽 = 시간대 이름 + 24시간 시계(HH:MM, D14), 가운데 = N일째 · 버틴 날, 오른쪽 = 코인.
//! 정보판 줄 바로 아래: 왼쪽에 체력 하트와 숫자(D2), 오른쪽에 빨리감기(D12)가 켜져 있으면 "빨리감기 x10" 판.
//! 값은 모두 logic(clock, wallet, health)이 준 그대로 보여 준다.

use super::coin::draw_coin;
use super::screen::VIRTUAL_WIDTH;
use super::text::TextPainter;
use super::touch::draw_star;
use crate::config::{DAY_PERIODS, DEV_FAST_FORWARD_SCALE};
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
/// 빨리감기 판: 정보판 줄 바로 아래. 개발용이고 판이 위에 그려져서 지나가는 해와 겹쳐도 글자는 보인다.
const FAST_PANEL_TOP: f32 = PANEL_TOP + PANEL_HEIGHT + 8.0;
const FAST_PANEL_HEIGHT: f32 = 48.0;
const FAST_PANEL_RIGHT: f32 = 775.0;
const FAST_ICON_H: f32 = 24.0;
const FAST_PAD_LEFT: f32 = 20.0;
const FAST_ICON_TEXT_GAP: f32 = 12.0;

/// 체력 판: 빨리감기 판과 같은 줄, 터치 "고르기" 버튼(x 12~168)의 오른쪽.
const HEALTH_LEFT: f32 = 180.0;
const HEALTH_PAD: f32 = 22.0;
const HEALTH_TEXT_GAP: f32 = 12.0;
/// 하트 줄의 가장 긴 폭. 최대 체력이 늘면 하트가 작아져서 이 안에 들어온다 (빨리감기 판과 안 겹치게).
const HEARTS_MAX_WIDTH: f32 = 200.0;
const HEART_MAX_STEP: f32 = 28.0;
const HEART_FULL: Color = Color::new(0.96, 0.27, 0.38, 1.0);
const HEART_EMPTY: Color = Color::new(0.0, 0.0, 0.0, 0.35);
const HEART_SPARKLE: Color = Color::new(1.0, 0.96, 0.70, 1.0);

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

/// 하트 하나: 동그라미 두 개 + 아래로 뾰족한 세모. `size`는 폭.
fn draw_heart(c: Vec2, size: f32, color: Color) {
    let bump = size * 0.27;
    draw_circle(c.x - size * 0.24, c.y - size * 0.12, bump, color);
    draw_circle(c.x + size * 0.24, c.y - size * 0.12, bump, color);
    draw_triangle(c + vec2(-size * 0.5, -size * 0.02), c + vec2(size * 0.5, -size * 0.02), c + vec2(0.0, size * 0.5), color);
}

/// 하트 한 칸 폭: 최대 체력이 커져도 하트 줄이 `HEARTS_MAX_WIDTH`를 넘지 않는다.
fn heart_step(max_hp: u32) -> f32 {
    (HEARTS_MAX_WIDTH / max_hp.max(1) as f32).min(HEART_MAX_STEP)
}

/// 체력 판: 최대 체력만큼 하트 + "남은/최대" 숫자. `sparkle`(1 막 시작 ~ 0 끝)이면 찬 하트가 반짝인다 (D27).
pub fn draw_health(text: &TextPainter, hp: u32, max_hp: u32, sparkle: f32) {
    let step = heart_step(max_hp);
    let label = format!("{hp}/{max_hp}");
    let hearts_w = step * max_hp as f32;
    let w = HEALTH_PAD + hearts_w + HEALTH_TEXT_GAP + text.width(&label, MID_TEXT) + HEALTH_PAD;
    draw_pill(HEALTH_LEFT, FAST_PANEL_TOP, w, FAST_PANEL_HEIGHT, PANEL);
    let mid_y = FAST_PANEL_TOP + FAST_PANEL_HEIGHT / 2.0;
    let grow = 1.0 + 0.3 * (sparkle * std::f32::consts::PI).sin();
    for i in 0..max_hp {
        let c = vec2(HEALTH_LEFT + HEALTH_PAD + step * (i as f32 + 0.5), mid_y);
        if i < hp {
            draw_heart(c, step * 0.86 * grow, HEART_FULL);
            if sparkle > 0.0 {
                draw_star(c, step * 0.5 * sparkle, sparkle * 4.0 + i as f32, Color { a: sparkle, ..HEART_SPARKLE });
            }
        } else {
            draw_heart(c, step * 0.86, HEART_EMPTY);
        }
    }
    text.left(&label, HEALTH_LEFT + HEALTH_PAD + hearts_w + HEALTH_TEXT_GAP, mid_y + 11.0, MID_TEXT, WHITE);
}

pub fn draw_hud(text: &TextPainter, clock: &Clock, coins: u32, fast_forward: bool) {
    let mid_y = PANEL_TOP + PANEL_HEIGHT / 2.0;
    let baseline = PANEL_TOP + TEXT_BASELINE;

    // 왼쪽: 시간대 이름 + 24시간 시계. 판 폭은 숫자가 바뀌어도 흔들리지 않게 고정 견본으로 잰다.
    let (hour, minute) = clock.hour_minute();
    let time_text = format!("{} {}", period_name(clock.time_of_day_minutes()), hh_mm(hour, minute));
    // 숫자마다 폭이 다른 글꼴이어도 넘치지 않게 가장 넓은 견본에 맞춘다.
    let sample_w = DAY_PERIODS
        .iter()
        .flat_map(|(_, name)| ["00:00", "88:88"].map(|t| format!("{name} {t}")))
        .map(|s| text.width(&s, BIG_TEXT))
        .fold(0.0, f32::max);
    let left_w = ICON_TEXT_GAP + sample_w + PANEL_PAD_RIGHT;
    draw_pill(MARGIN, PANEL_TOP, left_w, PANEL_HEIGHT, PANEL);
    let icon = vec2(MARGIN + ICON_X, mid_y);
    match clock.phase() {
        Phase::Day => draw_sun_icon(icon, 12.0),
        Phase::Night => draw_moon_icon(icon, 17.0),
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
    fn hearts_fit_as_max_health_grows() {
        assert_eq!(heart_step(7), HEART_MAX_STEP);
        for max_hp in [7, 12, 17, 100] {
            assert!(heart_step(max_hp) * max_hp as f32 <= HEARTS_MAX_WIDTH + 0.01, "{max_hp}");
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
        c.update(599.9);
        assert_eq!(show(&c), "17:59");
        c.update(0.1);
        assert_eq!(show(&c), "18:00");
        c.update(300.0);
        assert_eq!(show(&c), "00:00");
    }
}
