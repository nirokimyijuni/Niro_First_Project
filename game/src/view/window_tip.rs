//! 창 한 줄 안내 그림 (US-121). 규칙(처음 열었나·5초·창마다 한 번·저장)은 `logic::window_tips`에 있고 여기서는 보여 주기만 한다.
//!
//! - 창이 열리는 순간(`open_window`가 바뀔 때)과 깜짝 미션이 나올 때 main이 규칙을 부르고, 이 파일은 글을 한 줄 카드로 그린다.
//! - 카드는 창마다 정해진 빈 곳에 놓는다 (`config::WINDOW_TIP_SPOTS`). 글이 길면 글자를 조금 줄여 한 줄로 맞춘다.
//! - 카드는 그리기만 한다: 눌러도 아무 일도 없고 창 기능·게임 시간을 막지 않는다.

use super::hud::draw_pill;
use super::text::TextPainter;
use super::touch::draw_star;
use crate::config::*;
use crate::logic::game::Game;
use crate::logic::math::QuizSubject;
use crate::logic::window_tips::TipWindow;
use macroquad::prelude::*;

/// 글을 줄여도 이보다 작아지지는 않는다. 못 넣으면 시험이 실패해서 자리나 글을 고치게 한다.
pub const MIN_TEXT_SIZE: f32 = 19.0;

/// 지금 열려 있는 안내 창 (퀴즈·제작대·상점·가방). 없으면 None. 이 값이 바뀌는 프레임이 "창이 열리고 닫힌 순간"이다.
pub fn open_window(game: &Game, bag_open: bool) -> Option<TipWindow> {
    match game.math.subject() {
        Some(QuizSubject::Math) => Some(TipWindow::MathQuiz),
        Some(QuizSubject::English) => Some(TipWindow::EnglishQuiz),
        None if game.crafting.is_open() => Some(TipWindow::Crafting),
        None if game.shop.is_open() => Some(TipWindow::Shop),
        None if bag_open => Some(TipWindow::Bag),
        None => None,
    }
}

/// 카드 자리: `text_w`는 글 폭(글자를 줄인 뒤). 가운데 x는 창마다 정해져 있다.
pub fn card_rect(window: TipWindow, text_w: f32) -> Rect {
    let (center_x, top, max_w) = WINDOW_TIP_SPOTS[window.index()];
    let w = (text_w + WINDOW_TIP_PAD * 2.0 + WINDOW_TIP_ICON_W).min(max_w);
    Rect::new(center_x - w / 2.0, top, w, WINDOW_TIP_HEIGHT)
}

/// 글자 크기: 기본 크기로 자리에 들어가면 그대로, 아니면 줄여서 한 줄에 맞춘다 (`MIN_TEXT_SIZE`보다 작아지지 않는다).
/// `width`는 글 폭을 재는 함수 (기본 크기 기준이 아니라 크기를 받는다).
pub fn text_size(window: TipWindow, words: &str, width: impl Fn(&str, f32) -> f32) -> f32 {
    let (_, _, max_w) = WINDOW_TIP_SPOTS[window.index()];
    let room = max_w - WINDOW_TIP_PAD * 2.0 - WINDOW_TIP_ICON_W;
    let natural = width(words, WINDOW_TIP_TEXT_SIZE);
    if natural <= room {
        WINDOW_TIP_TEXT_SIZE
    } else {
        (WINDOW_TIP_TEXT_SIZE * room / natural).max(MIN_TEXT_SIZE)
    }
}

/// 깜짝 미션 안내는 게임 화면 위에 뜨므로 큰 알림 글이 그 아래로 쌓여야 한다. 그 아래 끝 (다른 안내는 None).
pub fn bottom(current: Option<TipWindow>) -> Option<f32> {
    current.filter(|w| *w == TipWindow::SurpriseMission).map(|w| WINDOW_TIP_SPOTS[w.index()].1 + WINDOW_TIP_HEIGHT + TUTORIAL_BUBBLE_GAP_BELOW)
}

/// 한 줄 카드를 그린다 (화면 좌표).
pub fn draw(text: &TextPainter, window: TipWindow, touch_mode: bool) {
    let words = window.text(touch_mode);
    let size = text_size(window, words, |s, size| text.width(s, size));
    let card = card_rect(window, text.width(words, size));
    let t = get_time() as f32;
    let color = |c: [f32; 4]| Color::new(c[0], c[1], c[2], c[3]);
    draw_pill(card.x, card.y, card.w, card.h, color(TUTORIAL_BUBBLE_COLOR));
    // 왼쪽에 살짝 반짝이는 별 (안내 표시)
    let star = vec2(card.x + WINDOW_TIP_PAD + WINDOW_TIP_ICON_W / 2.0 - 8.0, card.center().y);
    draw_star(star, 10.0 + 1.5 * (t * 5.0).sin(), t, color(TUTORIAL_CHEER_COLOR));
    let text_center = card.x + WINDOW_TIP_PAD + WINDOW_TIP_ICON_W + (card.w - WINDOW_TIP_PAD * 2.0 - WINDOW_TIP_ICON_W) / 2.0;
    text.centered(words, text_center, card.center().y + size * 0.34, size, color(TUTORIAL_TEXT_COLOR));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

    /// 한글은 폭이 글자 크기와 거의 같다. 시험은 이 값으로 잰다.
    fn fake_width(s: &str, size: f32) -> f32 {
        s.chars().count() as f32 * size
    }

    #[test]
    fn every_card_stays_inside_the_screen() {
        for window in TipWindow::ALL {
            let (x, y, w) = WINDOW_TIP_SPOTS[window.index()];
            let card = card_rect(window, 10_000.0);
            assert!(card.w <= w + 0.01);
            assert!(card.x >= 0.0 && card.right() <= VIRTUAL_WIDTH && y >= 0.0 && card.bottom() <= VIRTUAL_HEIGHT, "{window:?} {x}");
        }
    }

    #[test]
    fn every_text_fits_on_one_line_without_getting_tiny() {
        for window in TipWindow::ALL {
            for touch in [false, true] {
                let words = window.text(touch);
                assert!(words.chars().count() <= 26, "한 줄 안내는 25자 안팎: {words}");
                let size = text_size(window, words, fake_width);
                assert!(size >= MIN_TEXT_SIZE, "{window:?} {words}");
                let (_, _, max_w) = WINDOW_TIP_SPOTS[window.index()];
                let card = card_rect(window, fake_width(words, size));
                assert!(fake_width(words, size) + WINDOW_TIP_PAD * 2.0 + WINDOW_TIP_ICON_W <= max_w + 0.01, "{window:?} {words}: 글이 카드에 안 들어간다");
                assert!(card.w <= max_w + 0.01);
            }
        }
    }

    /// 창마다 눌러야 하는 칸·버튼들 (카드가 가리면 안 되는 곳).
    fn must_stay_visible(window: TipWindow) -> Vec<Rect> {
        use crate::view::{bag, craft, english, math, shop};
        match window {
            TipWindow::MathQuiz => math::pad_buttons().map(|(r, _)| r).collect(),
            TipWindow::EnglishQuiz => (0..4).map(english::choice_rect).chain([english::CUE]).collect(),
            TipWindow::Crafting => {
                let mut cells: Vec<Rect> = (0..3).flat_map(|row| (0..4).map(move |column| craft::cell_rect(row, column))).collect();
                cells.extend([craft::CLOSE, craft::RESULT]);
                cells
            }
            TipWindow::Shop => {
                let mut cells: Vec<Rect> = (0..5).map(shop::buy_rect).chain((0..3).map(shop::sell_rect)).collect();
                cells.push(shop::CLOSE);
                cells
            }
            TipWindow::Bag => {
                let mut cells: Vec<Rect> = (0..3).flat_map(|row| (0..9).map(move |col| bag::slot_rect(row, col))).collect();
                cells.push(bag::CLOSE);
                cells
            }
            TipWindow::SurpriseMission => Vec::new(),
        }
    }

    #[test]
    fn cards_never_cover_the_buttons_and_cells_of_their_window() {
        for window in TipWindow::ALL {
            // 가장 넓을 때의 카드로 잰다 (글이 길수록 넓다)
            let card = card_rect(window, 10_000.0);
            for cell in must_stay_visible(window) {
                assert!(!card.overlaps(&cell), "{window:?} 카드 {card:?} 가 {cell:?} 를 가린다");
            }
        }
    }

    #[test]
    fn card_keeps_off_the_phone_buttons_during_the_mission_tip() {
        use crate::view::touch::{play_buttons_with_mine, BACK_BUTTON};
        let card = card_rect(TipWindow::SurpriseMission, 10_000.0);
        assert!(!card.overlaps(&BACK_BUTTON));
        assert!(!card.overlaps(&crate::view::help::button_rect(true)) && !card.overlaps(&crate::view::help::button_rect(false)));
        for button in play_buttons_with_mine(true, true) {
            let nearest = vec2(button.center.x.clamp(card.x, card.right()), button.center.y.clamp(card.y, card.bottom()));
            assert!(nearest.distance(button.center) > button.radius, "{:?}", button.kind);
        }
    }

    #[test]
    fn short_text_keeps_its_normal_size() {
        let size = text_size(TipWindow::Shop, "짧아요", fake_width);
        assert_eq!(size, WINDOW_TIP_TEXT_SIZE);
    }

    #[test]
    fn only_the_mission_tip_pushes_big_messages_down() {
        assert!(bottom(Some(TipWindow::SurpriseMission)).is_some());
        for window in [TipWindow::MathQuiz, TipWindow::EnglishQuiz, TipWindow::Crafting, TipWindow::Shop, TipWindow::Bag] {
            assert_eq!(bottom(Some(window)), None);
        }
        assert_eq!(bottom(None), None);
    }

    #[test]
    fn open_window_follows_the_game_windows() {
        use crate::logic::character::CharacterKind;
        let game = Game::new(CharacterKind::Niro, 3);
        assert_eq!(open_window(&game, false), None);
        assert_eq!(open_window(&game, true), Some(TipWindow::Bag));
    }
}
