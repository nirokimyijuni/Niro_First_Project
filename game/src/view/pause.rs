//! 멈춤 창 (로건, 부모님 요청 2026-11-14): 게임 중 Esc나 폰 "멈춤" 버튼을 누르면 게임이 멈추고 작은 창이 뜬다.
//! "이어서 하기"(Esc를 다시 눌러도 된다) / "처음으로 (캐릭터 고르기)".
//! 멈추는 것은 main이 그동안 `Game::update`를 부르지 않는 것이라 logic은 바꾸지 않는다 (세로 안내와 같은 방법).

use super::hud::draw_pill;
use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::text::TextPainter;
use macroquad::prelude::*;

const PANEL: Rect = Rect::new(250.0, 90.0, 460.0, 380.0);
pub const RESUME_BUTTON: Rect = Rect::new(290.0, 190.0, 380.0, 110.0);
pub const QUIT_BUTTON: Rect = Rect::new(290.0, 320.0, 380.0, 110.0);

// 글은 config.rs에 모은다 (REVIEW-020 3번). 여기서는 가리키기만 한다.
pub const PAUSE_TITLE_TEXT: &str = crate::config::PAUSE_TITLE_TEXT;
pub const RESUME_TEXT: &str = crate::config::PAUSE_RESUME_TEXT;
pub const QUIT_TEXT: &str = crate::config::PAUSE_QUIT_TEXT;
pub const QUIT_SUB_TEXT: &str = crate::config::PAUSE_QUIT_SUB_TEXT;

const DIM: Color = Color::new(0.0, 0.0, 0.0, 0.45);
const PANEL_FILL: Color = Color::new(0.10, 0.13, 0.32, 0.92);
const PANEL_LINE: Color = Color::new(1.0, 1.0, 1.0, 0.7);
const RESUME_FILL: Color = Color::new(1.0, 0.66, 0.12, 1.0);
const QUIT_FILL: Color = Color::new(0.30, 0.62, 0.90, 1.0);
const TITLE: Color = Color::new(1.0, 0.86, 0.22, 1.0);

/// 멈춤 창에서 누른 것.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseTap {
    Resume,
    Quit,
}

/// 누른 곳(가상 좌표)이 어느 버튼인지. 빈 곳은 None (창 밖을 눌러도 아무 일 없다).
pub fn pause_tap_at(point: Vec2) -> Option<PauseTap> {
    if RESUME_BUTTON.contains(point) {
        Some(PauseTap::Resume)
    } else if QUIT_BUTTON.contains(point) {
        Some(PauseTap::Quit)
    } else {
        None
    }
}

/// 멈춤 창 그리기 (화면 좌표). `touch_mode`가 아니면 "이어서 하기" 아래에 "Esc"를 작게 쓴다.
pub fn draw_pause(text: &TextPainter, touch_mode: bool) {
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, DIM);
    draw_rectangle(PANEL.x, PANEL.y, PANEL.w, PANEL.h, PANEL_FILL);
    draw_rectangle_lines(PANEL.x, PANEL.y, PANEL.w, PANEL.h, 5.0, PANEL_LINE);
    text.centered(PAUSE_TITLE_TEXT, PANEL.center().x, PANEL.y + 66.0, 44.0, TITLE);

    let r = RESUME_BUTTON;
    draw_pill(r.x, r.y, r.w, r.h, RESUME_FILL);
    // 이어서 하기: 오른쪽을 가리키는 세모 (놀이 버튼)
    let tri = vec2(r.x + 62.0, r.center().y);
    draw_triangle(tri + vec2(-16.0, -22.0), tri + vec2(-16.0, 22.0), tri + vec2(22.0, 0.0), WHITE);
    let (y, small) = if touch_mode { (r.center().y + 16.0, None) } else { (r.center().y + 6.0, Some("Esc")) };
    text.centered(RESUME_TEXT, r.center().x + 24.0, y, 46.0, WHITE);
    if let Some(key) = small {
        text.centered(key, r.center().x + 24.0, r.bottom() - 10.0, 24.0, WHITE);
    }

    let q = QUIT_BUTTON;
    draw_pill(q.x, q.y, q.w, q.h, QUIT_FILL);
    text.centered(QUIT_TEXT, q.center().x, q.center().y + 2.0, 42.0, WHITE);
    text.centered(QUIT_SUB_TEXT, q.center().x, q.bottom() - 14.0, 26.0, WHITE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_are_big_apart_and_inside_the_window() {
        for b in [RESUME_BUTTON, QUIT_BUTTON] {
            assert!(PANEL.contains(b.point()) && PANEL.contains(vec2(b.right(), b.bottom())), "{b:?} 창 밖");
            // 740×360 폰(0.667배)에서도 70 화면 픽셀 넘게
            assert!(b.h * 360.0 / 540.0 >= 70.0 && b.w >= 300.0);
        }
        assert!(RESUME_BUTTON.intersect(QUIT_BUTTON).is_none(), "두 버튼이 안 겹친다");
        assert!(PANEL.right() <= VIRTUAL_WIDTH && PANEL.bottom() <= VIRTUAL_HEIGHT);
    }

    #[test]
    fn taps_pick_resume_or_quit_and_nothing_else() {
        assert_eq!(pause_tap_at(RESUME_BUTTON.center()), Some(PauseTap::Resume));
        assert_eq!(pause_tap_at(QUIT_BUTTON.center()), Some(PauseTap::Quit));
        assert_eq!(pause_tap_at(vec2(20.0, 20.0)), None, "창 밖은 아무 일 없다");
        assert_eq!(pause_tap_at(PANEL.point() + vec2(10.0, 10.0)), None, "제목 자리도 아무 일 없다");
    }

    #[test]
    fn words_are_easy_korean() {
        assert_eq!((RESUME_TEXT, QUIT_TEXT, QUIT_SUB_TEXT), ("이어서 하기", "처음으로", "(캐릭터 고르기)"));
    }
}
