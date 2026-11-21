//! 물음표(?) 버튼과 조작 안내 한 장 그림 (US-122). 보임·열림·닫힘 규칙은 `logic::help`에 있고 여기서는 그리고 눌림만 본다.
//!
//! - 버튼: 왼쪽 위 얼굴·물건 칸 바로 아래 (PC), 폰은 "멈춤" 버튼 바로 아래. 노란 동그라미에 큰 "?".
//! - 안내 한 장: 화면을 어둡게 하고 가운데 카드에 그림 여섯 개(걷기·점프·공격·캐기·가방·멈춤)와 짧은 글, 큰 "알았어요!" 버튼.
//!   새 그림 파일 없이 기존 모양(화살표·별·곡괭이)과 글자로 그린다. 모양은 임시이고 아이에게 물어볼 거리다.
//! - 눌린 손가락은 조이스틱·버튼으로 새지 않는다 (main이 `tutorial::claim_skip_finger`와 같은 길로 가려낸다).

use super::hud::draw_pill;
use super::mission::{draw_round_rect, split_lines};
use super::text::TextPainter;
use super::touch::{draw_arrow_head, draw_pickaxe_shape, draw_star};
use crate::config::*;
use macroquad::prelude::*;

fn rgba(c: [f32; 4]) -> Color {
    Color::new(c[0], c[1], c[2], c[3])
}

/// 물음표 버튼의 네모 (누르는 자리). 그림은 이 안의 동그라미.
pub fn button_rect(touch_mode: bool) -> Rect {
    let (top, size) = if touch_mode { (HELP_BUTTON_TOP_TOUCH, HELP_BUTTON_SIZE_TOUCH) } else { (HELP_BUTTON_TOP_PC, HELP_BUTTON_SIZE_PC) };
    Rect::new(HELP_BUTTON_LEFT, top, size, size)
}

/// 안내 한 장 카드.
pub fn sheet_rect() -> Rect {
    Rect::new(HELP_SHEET[0], HELP_SHEET[1], HELP_SHEET[2], HELP_SHEET[3])
}

/// "알았어요!" 닫기 버튼.
pub fn close_rect() -> Rect {
    Rect::new(HELP_CLOSE[0], HELP_CLOSE[1], HELP_CLOSE[2], HELP_CLOSE[3])
}

/// 줄 하나가 놓이는 칸: 두 칸씩 세 줄, 카드 안 제목 아래.
const CELL_W: f32 = 372.0;
const CELL_H: f32 = 100.0;
const CELLS_TOP: f32 = 110.0;
const CELL_LEFT: f32 = 100.0;
const CELL_GAP_X: f32 = 8.0;
const ICON_RADIUS: f32 = 38.0;
const LABEL_SIZE: f32 = 36.0;
const HOW_SIZE: f32 = 22.0;
const TITLE_SIZE: f32 = 46.0;
/// 그림과 글 사이
const TEXT_LEFT_GAP: f32 = 18.0;

/// 줄 번호의 칸 (두 칸씩 가로로, 위에서 아래로).
pub fn cell_rect(index: usize) -> Rect {
    let (col, row) = (index % 2, index / 2);
    Rect::new(CELL_LEFT + col as f32 * (CELL_W + CELL_GAP_X), CELLS_TOP + row as f32 * CELL_H, CELL_W, CELL_H - 4.0)
}

/// 글이 들어가는 폭.
fn how_width() -> f32 {
    CELL_W - ICON_RADIUS * 2.0 - TEXT_LEFT_GAP - 8.0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpIcon {
    Walk,
    Jump,
    Attack,
    Mine,
    Bag,
    Pause,
    /// 모르는 이름의 줄 (글만 있어도 별 하나는 붙는다)
    Other,
}

/// 줄의 첫 글(무엇을 하는지)로 그림을 고른다.
pub fn icon_for(label: &str) -> HelpIcon {
    match label {
        "걷기" => HelpIcon::Walk,
        "점프" => HelpIcon::Jump,
        "공격" => HelpIcon::Attack,
        "캐기" => HelpIcon::Mine,
        "가방" => HelpIcon::Bag,
        "멈춤" => HelpIcon::Pause,
        _ => HelpIcon::Other,
    }
}

/// 글꼴(Jua)에 없는 화살표 글자(← → ↑ ↓)를 낱말로 바꾼다. config 글은 그대로 두고 그릴 때만 바꾼다 (제작대의 ①~④와 같은 방법).
pub fn readable(how: &str) -> String {
    how.replace("← →", "왼쪽/오른쪽").replace('←', "왼쪽").replace('→', "오른쪽").replace('↑', "위쪽").replace('↓', "아래쪽")
}

/// 눌렸는지: 마우스 클릭이나 새로 댄 손가락이 `rect` 안일 때 (키는 없다).
pub fn pressed(click: Option<Vec2>, new_finger: Option<Vec2>, rect: Rect) -> bool {
    click.is_some_and(|p| rect.contains(p)) || new_finger.is_some_and(|p| rect.contains(p))
}

/// 물음표 버튼을 그린다 (화면 좌표). 눈에 띄게 살짝 숨쉬듯 커졌다 작아진다.
pub fn draw_button(text: &TextPainter, touch_mode: bool) {
    let r = button_rect(touch_mode);
    let t = get_time() as f32;
    let center = r.center();
    let radius = r.w / 2.0 - 4.0 + 2.0 * (t * 3.0).sin();
    draw_circle(center.x + 2.0, center.y + 4.0, radius, Color::new(0.0, 0.0, 0.0, 0.25));
    draw_circle(center.x, center.y, radius, rgba(HELP_BUTTON_COLOR));
    draw_circle_lines(center.x, center.y, radius, 5.0, WHITE);
    text.centered("?", center.x, center.y + r.w * 0.2, r.w * 0.68, rgba(HELP_BUTTON_MARK_COLOR));
}

fn draw_icon(icon: HelpIcon, c: Vec2) {
    let r = ICON_RADIUS;
    draw_circle(c.x, c.y, r, Color::new(0.30, 0.38, 0.70, 1.0));
    draw_circle_lines(c.x, c.y, r, 4.0, rgba(HELP_BUTTON_COLOR));
    match icon {
        HelpIcon::Walk => {
            for side in [-1.0, 1.0] {
                draw_arrow_head(c + vec2(side * r * 0.62, 0.0), vec2(side, 0.0), r * 0.46, WHITE);
            }
            draw_line(c.x - r * 0.3, c.y, c.x + r * 0.3, c.y, 5.0, WHITE);
        }
        HelpIcon::Jump => {
            draw_arrow_head(c + vec2(0.0, -r * 0.62), vec2(0.0, -1.0), r * 0.5, WHITE);
            draw_line(c.x, c.y - r * 0.2, c.x, c.y + r * 0.5, 6.0, WHITE);
        }
        HelpIcon::Attack => draw_star(c, r * 0.72, 0.0, rgba(HELP_BUTTON_COLOR)),
        HelpIcon::Mine => draw_pickaxe_shape(c + vec2(0.0, r * 0.05), r * 1.5, Color::new(0.93, 0.72, 0.42, 1.0), WHITE),
        HelpIcon::Bag => {
            // 장난감 상자: 몸통과 뚜껑
            draw_rectangle(c.x - r * 0.52, c.y - r * 0.12, r * 1.04, r * 0.7, WHITE);
            draw_rectangle(c.x - r * 0.6, c.y - r * 0.42, r * 1.2, r * 0.3, rgba(HELP_BUTTON_COLOR));
            draw_rectangle(c.x - r * 0.08, c.y - r * 0.12, r * 0.16, r * 0.26, rgba(HELP_BUTTON_MARK_COLOR));
        }
        HelpIcon::Pause => {
            for dx in [-0.26, 0.26] {
                draw_rectangle(c.x + dx * r - r * 0.12, c.y - r * 0.5, r * 0.24, r, WHITE);
            }
        }
        HelpIcon::Other => draw_star(c, r * 0.6, 0.0, WHITE),
    }
}

/// 조작 안내 한 장을 그린다 (화면 좌표). `lines`는 `logic::help::Help::lines`의 (무엇, 어떻게).
pub fn draw_sheet(text: &TextPainter, lines: &[(&str, &str)]) {
    draw_rectangle(0.0, 0.0, super::screen::VIRTUAL_WIDTH, super::screen::VIRTUAL_HEIGHT, Color::new(0.0, 0.0, 0.0, 0.5));
    let card = sheet_rect();
    draw_round_rect(card, 28.0, Color::new(0.10, 0.13, 0.32, 0.97));
    text.centered(HELP_TITLE_TEXT, card.center().x, card.y + 68.0, TITLE_SIZE, rgba(TUTORIAL_CHEER_COLOR));
    for (i, (label, how)) in lines.iter().take(HELP_LINES_MAX).enumerate() {
        let cell = cell_rect(i);
        let icon_center = vec2(cell.x + ICON_RADIUS + 4.0, cell.center().y);
        draw_icon(icon_for(label), icon_center);
        let left = icon_center.x + ICON_RADIUS + TEXT_LEFT_GAP;
        let how = readable(how);
        let how_lines = split_lines(&how, how_width(), |s| text.width(s, HOW_SIZE));
        // 글 묶음(큰 글 + 작은 글 한두 줄)을 칸 가운데 높이에 맞춘다
        let block_h = LABEL_SIZE * 0.95 + how_lines.len() as f32 * HOW_SIZE * 1.15;
        let mut baseline = cell.center().y - block_h / 2.0 + LABEL_SIZE * 0.85;
        text.left(label, left, baseline, LABEL_SIZE, WHITE);
        baseline += LABEL_SIZE * 0.25;
        for line in &how_lines {
            baseline += HOW_SIZE * 1.15;
            text.left(line, left, baseline, HOW_SIZE, Color::new(0.85, 0.92, 1.0, 1.0));
        }
    }
    let close = close_rect();
    draw_pill(close.x, close.y, close.w, close.h, rgba(HELP_BUTTON_COLOR));
    text.centered(HELP_CLOSE_TEXT, close.center().x, close.center().y + 14.0, 40.0, rgba(HELP_BUTTON_MARK_COLOR));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::help::Help;
    use crate::view::bag::bag_button_rect;
    use crate::view::hud::{FACE_CENTER, FACE_RADIUS, HEALTH_BAR, POCKET_TAP_H, ROW_MID_Y};
    use crate::view::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
    use crate::view::touch::{play_buttons_with_mine, BACK_BUTTON, JOYSTICK_RADIUS};

    fn fake_width(s: &str, size: f32) -> f32 {
        s.chars().count() as f32 * size
    }

    #[test]
    fn button_sits_right_under_the_profile_and_stays_clear_of_everything() {
        for touch in [false, true] {
            let b = button_rect(touch);
            assert!(b.x >= 0.0 && b.right() <= VIRTUAL_WIDTH && b.bottom() <= VIRTUAL_HEIGHT);
            // 얼굴 동그라미·체력 막대 아래, 물건 칸(누르는 자리) 아래
            assert!(b.y >= FACE_CENTER.y + FACE_RADIUS, "얼굴 아래");
            assert!(b.y >= ROW_MID_Y + POCKET_TAP_H / 2.0, "물건 칸 아래: {}", b.y);
            assert!(b.y > HEALTH_BAR.bottom());
            // 폰 "멈춤" 버튼, 가방 버튼과 겹치지 않는다
            if touch {
                assert!(!b.overlaps(&BACK_BUTTON), "멈춤 버튼");
                assert!(b.y >= BACK_BUTTON.bottom(), "멈춤 버튼 바로 아래");
            }
            assert!(!b.overlaps(&bag_button_rect(touch)));
            // 말풍선 카드(왼쪽 끝 172)와 안 겹친다
            assert!(b.right() < 172.0);
            // 오른쪽 버튼(공격·캐기·바꾸기)과 멀다
            for button in play_buttons_with_mine(true, true) {
                let nearest = vec2(button.center.x.clamp(b.x, b.right()), button.center.y.clamp(b.y, b.bottom()));
                assert!(nearest.distance(button.center) > button.radius, "{:?}", button.kind);
            }
        }
    }

    #[test]
    fn phone_button_is_big_enough_for_a_thumb() {
        // 폰 667×375는 가상 화면을 0.695배로 줄인다. 그래도 48 화면 픽셀 이상 (다른 폰 버튼들과 같은 기준)
        let b = button_rect(true);
        assert!(b.w * (667.0 / 960.0) >= 48.0, "{}", b.w);
        assert!(b.h * (375.0 / 540.0) >= 48.0, "{}", b.h);
    }

    #[test]
    fn button_stays_off_the_joystick_start_hint_circle() {
        // 조이스틱 처음 자리(150, 400, 반지름 85)의 위 끝은 315 — 폰 버튼 아래 끝보다 아래다
        assert!(button_rect(true).bottom() < 400.0 - JOYSTICK_RADIUS);
    }

    #[test]
    fn sheet_holds_every_line_and_the_close_button() {
        let card = sheet_rect();
        assert!(card.x >= 0.0 && card.right() <= VIRTUAL_WIDTH && card.y >= 0.0 && card.bottom() <= VIRTUAL_HEIGHT);
        let close = close_rect();
        assert!(card.contains(close.point()) && card.contains(vec2(close.right() - 0.1, close.bottom() - 0.1)), "닫기 버튼은 카드 안");
        assert!(close.h * (375.0 / 540.0) >= 48.0, "폰에서도 누르기 쉬운 크기");
        for touch in [false, true] {
            let lines = Help::lines(touch);
            assert!(lines.len() <= HELP_LINES_MAX.min(6), "한 장에 {} 줄까지", HELP_LINES_MAX.min(6));
            for (i, (label, how)) in lines.iter().enumerate() {
                let cell = cell_rect(i);
                assert!(card.contains(cell.point()) && card.contains(vec2(cell.right() - 0.1, cell.bottom() - 0.1)), "{label} 칸이 카드 밖");
                assert!(cell.bottom() <= close.y, "{label} 칸이 닫기 버튼 위쪽에 있다");
                // 글은 한 줄이나 두 줄로 칸에 들어간다
                let how = readable(how);
                let how_lines = split_lines(&how, how_width(), |s| fake_width(s, HOW_SIZE));
                assert!(how_lines.len() <= 2, "{how}");
                for line in &how_lines {
                    assert!(fake_width(line, HOW_SIZE) <= how_width(), "{line}");
                }
                assert!(fake_width(label, LABEL_SIZE) <= how_width(), "{label}");
                let block_h = LABEL_SIZE * 0.95 + how_lines.len() as f32 * HOW_SIZE * 1.15;
                assert!(block_h <= CELL_H - 6.0, "{label} 글 높이 {block_h}");
                assert_ne!(icon_for(label), HelpIcon::Other, "{label}에 맞는 그림이 있어야 한다");
            }
        }
    }

    #[test]
    fn arrow_glyphs_the_font_cannot_draw_become_words() {
        // 글꼴에 없는 글자는 네모(□)로 보인다 (실제 화면에서 확인한 일). 그릴 글에는 화살표가 없어야 한다.
        for touch in [false, true] {
            for (label, how) in Help::lines(touch) {
                let shown = readable(how);
                assert!(!shown.contains(['←', '→', '↑', '↓']), "{label}: {shown}");
            }
        }
        assert_eq!(readable("← → 화살표 키"), "왼쪽/오른쪽 화살표 키");
        assert_eq!(readable("Space 키 또는 ↑ 키"), "Space 키 또는 위쪽 키");
        assert_eq!(readable("Q 키"), "Q 키", "화살표가 없으면 그대로");
    }

    #[test]
    fn cells_do_not_overlap_each_other() {
        for a in 0..6 {
            for b in (a + 1)..6 {
                assert!(!cell_rect(a).overlaps(&cell_rect(b)), "{a} {b}");
            }
        }
    }

    #[test]
    fn close_button_is_clear_of_the_help_button_so_the_opening_finger_cannot_close_it() {
        // 버튼을 누른 손가락이 다음 프레임에 "눌림"으로 한 번 더 읽혀도 닫기 버튼이 아니다
        for touch in [false, true] {
            assert!(!button_rect(touch).overlaps(&close_rect()));
        }
    }

    #[test]
    fn pressed_by_click_or_new_finger_inside_only() {
        let r = button_rect(true);
        let inside = r.center();
        let outside = vec2(500.0, 300.0);
        assert!(pressed(Some(inside), None, r));
        assert!(pressed(None, Some(inside), r));
        assert!(!pressed(Some(outside), Some(outside), r));
        assert!(!pressed(None, None, r));
    }
}
