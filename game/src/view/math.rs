//! 미스의 수학 문제 창 (US-076, D67). 규칙은 `logic::math::MathQuiz`에 있고, 여기서는 그 상태를 그리고
//! 숫자판·숫자 키를 `press_digit`·`erase`·`submit`으로 넘긴다.
//! 왼쪽: 미스와 말풍선, 문제 문장, 넣은 답 칸, 남은 기회. 오른쪽: 큰 숫자판 12개.

use super::gameover::{draw_confetti, miss_dance};
use super::hud::draw_pill;
use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::text::TextPainter;
use crate::config::{MATH_MISS_ASK_TEXT, MATH_RIGHT_SHOW_SECONDS, MATH_TITLE_TEXT, MATH_TRY_AGAIN_TEXT};
use crate::logic::math::{MathQuiz, MathResult};
use macroquad::prelude::*;

/// 숫자판 버튼 하나가 하는 일.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathKey {
    Digit(u32),
    Erase,
    Submit,
}

impl MathKey {
    /// 화면에 쓰는 글.
    fn label(self) -> String {
        match self {
            MathKey::Digit(d) => d.to_string(),
            MathKey::Erase => "지우기".to_string(),
            MathKey::Submit => "확인".to_string(),
        }
    }

    /// 누른 버튼을 문제에 넘긴다.
    pub fn apply(self, quiz: &mut MathQuiz) {
        match self {
            MathKey::Digit(d) => quiz.press_digit(d),
            MathKey::Erase => quiz.erase(),
            MathKey::Submit => {
                quiz.submit();
            }
        }
    }
}

/// 숫자판 버튼 크기와 사이 ([임시-32]: 72픽셀 이상, 사이 12픽셀 이상)
pub const PAD_BUTTON: f32 = 80.0;
pub const PAD_GAP: f32 = 12.0;
/// 숫자판 왼쪽 위
const PAD_LEFT: f32 = 676.0;
const PAD_TOP: f32 = 138.0;
const PAD_ROWS: [[MathKey; 3]; 4] = [
    [MathKey::Digit(1), MathKey::Digit(2), MathKey::Digit(3)],
    [MathKey::Digit(4), MathKey::Digit(5), MathKey::Digit(6)],
    [MathKey::Digit(7), MathKey::Digit(8), MathKey::Digit(9)],
    [MathKey::Erase, MathKey::Digit(0), MathKey::Submit],
];

/// 창 판. 왼쪽은 터치 "◀ 고르기" 버튼(x 12~168)이 보이도록 비운다.
const PANEL: Rect = Rect::new(178.0, 8.0, 774.0, 524.0);
/// 왼쪽 칸 (말풍선, 문제 문장, 답 칸): 판 왼쪽 안쪽 ~ 숫자판 앞
const LEFT_X: f32 = 196.0;
const LEFT_RIGHT: f32 = PAD_LEFT - 18.0;
const TITLE_Y: f32 = 62.0;
const MISS_FEET: Vec2 = Vec2::new(250.0, 212.0);
const MISS_HEIGHT: f32 = 112.0;
const BUBBLE: Rect = Rect::new(318.0, 92.0, LEFT_RIGHT - 318.0, 104.0);
const BUBBLE_SIZE: f32 = 28.0;
/// 문제 문장: 이 자리에서 시작해 줄마다 내려간다. 마지막 줄은 답 칸 위.
pub const QUESTION_TOP: f32 = 228.0;
pub const QUESTION_SIZE: f32 = 30.0;
pub const QUESTION_LINE: f32 = 38.0;
const ANSWER_BOX: Rect = Rect::new(LEFT_X, 420.0, 260.0, 72.0);
const TRIES_Y: f32 = 522.0;

const PANEL_FILL: Color = Color::new(0.20, 0.28, 0.55, 0.94);
const DIM: Color = Color::new(0.06, 0.08, 0.20, 0.55);
const BUBBLE_FILL: Color = Color::new(1.0, 1.0, 1.0, 0.95);
const BUBBLE_TEXT: Color = Color::new(0.20, 0.24, 0.50, 1.0);
const DIGIT_FILL: Color = Color::new(0.35, 0.68, 1.0, 1.0);
const ERASE_FILL: Color = Color::new(1.0, 0.62, 0.20, 1.0);
const SUBMIT_FILL: Color = Color::new(0.30, 0.80, 0.42, 1.0);
const BOX_FILL: Color = Color::new(1.0, 1.0, 1.0, 0.95);
const STAR: Color = Color::new(1.0, 0.86, 0.22, 1.0);

/// 숫자판 버튼 자리.
pub fn pad_buttons() -> impl Iterator<Item = (Rect, MathKey)> {
    PAD_ROWS.iter().enumerate().flat_map(|(row, keys)| {
        keys.iter().enumerate().map(move |(col, key)| {
            let x = PAD_LEFT + col as f32 * (PAD_BUTTON + PAD_GAP);
            let y = PAD_TOP + row as f32 * (PAD_BUTTON + PAD_GAP);
            (Rect::new(x, y, PAD_BUTTON, PAD_BUTTON), *key)
        })
    })
}

/// 가상 화면의 이 자리를 누르면 어느 버튼인지.
pub fn pad_key_at(point: Vec2) -> Option<MathKey> {
    pad_buttons().find(|(r, _)| r.contains(point)).map(|(_, k)| k)
}

/// 띄어쓰기에서 줄을 바꿔 `max_width` 안에 들게 한다. 한 낱말이 너무 길면 글자 단위로 자른다.
pub fn wrap_lines(text: &str, max_width: f32, width: impl Fn(&str) -> f32) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let joined = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if width(&joined) <= max_width {
            line = joined;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        for c in word.chars() {
            let next = format!("{line}{c}");
            if !line.is_empty() && width(&next) > max_width {
                lines.push(std::mem::replace(&mut line, c.to_string()));
            } else {
                line = next;
            }
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// 말풍선 줄: "너무 아쉽다!" 뒤에서 먼저 줄을 바꾼다.
fn bubble_lines(says: &str, width: impl Fn(&str) -> f32) -> Vec<String> {
    says.split_inclusive("! ").flat_map(|part| wrap_lines(part, BUBBLE.w - BUBBLE.h * 0.6, &width)).collect()
}

/// 미스가 말풍선에 하는 말. 맞힘·다시·정답 글은 logic이 준다.
fn miss_says(quiz: &MathQuiz) -> String {
    match quiz.result() {
        Some(MathResult::TryAgain) => MATH_TRY_AGAIN_TEXT.to_string(),
        Some(_) => quiz.result_text().unwrap_or_default(),
        None => MATH_MISS_ASK_TEXT.to_string(),
    }
}

/// 문제 창 전체. 판은 왼쪽 터치 "◀ 고르기" 자리를 가리지 않는다 (main이 그 버튼을 그린다).
pub fn draw_math(text: &TextPainter, quiz: &MathQuiz, miss: &Texture2D) {
    let Some(question) = quiz.question() else { return };
    let right = quiz.result() == Some(MathResult::Right);
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, DIM);
    draw_rectangle(PANEL.x, PANEL.y, PANEL.w, PANEL.h, PANEL_FILL);
    let cheer_seconds = MATH_RIGHT_SHOW_SECONDS - quiz.result_seconds_left();
    if right {
        draw_confetti(cheer_seconds);
    }
    text.centered(MATH_TITLE_TEXT, PANEL.x + PANEL.w / 2.0, TITLE_Y, 48.0, STAR);

    // 미스: 평소에는 살짝 숨 쉬고, 맞히면 두 팔을 들고 콩콩 뛴다
    draw_miss(miss, if right { Some(cheer_seconds) } else { None });
    draw_pill(BUBBLE.x, BUBBLE.y, BUBBLE.w, BUBBLE.h, BUBBLE_FILL);
    draw_triangle(vec2(BUBBLE.x + 8.0, BUBBLE.y + 60.0), vec2(BUBBLE.x + 30.0, BUBBLE.y + 74.0), vec2(BUBBLE.x - 22.0, BUBBLE.y + 88.0), BUBBLE_FILL);
    let says = miss_says(quiz);
    let (size, color) = if right { (40.0, STAR) } else { (BUBBLE_SIZE, BUBBLE_TEXT) };
    let lines = bubble_lines(&says, |s| text.width(s, size));
    let first = BUBBLE.y + BUBBLE.h / 2.0 + size * 0.38 - (lines.len() as f32 - 1.0) * size * 0.6;
    for (i, line) in lines.iter().enumerate() {
        text.centered(line, BUBBLE.x + BUBBLE.w / 2.0, first + i as f32 * size * 1.2, size, color);
    }

    // 문제 문장 (식은 보여 주지 않는다)
    let lines = wrap_lines(&question.text, LEFT_RIGHT - LEFT_X, |s| text.width(s, QUESTION_SIZE));
    for (i, line) in lines.iter().enumerate() {
        text.left(line, LEFT_X, QUESTION_TOP + QUESTION_SIZE + i as f32 * QUESTION_LINE, QUESTION_SIZE, WHITE);
    }

    // 넣은 답 칸과 남은 기회
    let b = ANSWER_BOX;
    draw_rectangle(b.x, b.y, b.w, b.h, BOX_FILL);
    draw_rectangle_lines(b.x, b.y, b.w, b.h, 5.0, STAR);
    let typed = if quiz.typed().is_empty() && quiz.result().is_none() && (get_time() * 2.0).fract() < 0.5 { "_" } else { quiz.typed() };
    text.centered(typed, b.x + b.w / 2.0, b.y + b.h / 2.0 + 19.0, 54.0, WHITE);
    if quiz.result() != Some(MathResult::Wrong) {
        // 기회를 다 쓰면 "0번 남았어" 대신 말풍선의 정답만 보인다
        text.left(&quiz.tries_left_text(), LEFT_X, TRIES_Y, 26.0, WHITE);
    }

    // 숫자판. 맞힘·정답 글이 나오는 동안은 흐리게
    let alpha = if matches!(quiz.result(), Some(MathResult::Right | MathResult::Wrong)) { 0.35 } else { 1.0 };
    for (r, key) in pad_buttons() {
        let fill = match key {
            MathKey::Digit(_) => DIGIT_FILL,
            MathKey::Erase => ERASE_FILL,
            MathKey::Submit => SUBMIT_FILL,
        };
        draw_pill(r.x, r.y, r.w, r.h, Color { a: alpha, ..fill });
        let size = if matches!(key, MathKey::Digit(_)) { 46.0 } else { 26.0 };
        text.centered(&key.label(), r.x + r.w / 2.0, r.y + r.h / 2.0 + size * 0.36, size, Color { a: alpha, ..WHITE });
    }
}

/// 미스 그림. `cheer`가 있으면 그 초만큼 축하 중: 클리어 화면처럼 콩콩 뛰고, 두 팔을 들어 흔든다.
fn draw_miss(texture: &Texture2D, cheer: Option<f32>) {
    let (lift, squash) = match cheer {
        Some(t) => {
            let dance = miss_dance(t);
            (dance.lift.min(40.0), dance.squash)
        }
        None => (0.0, 0.03 * (get_time() as f32 * 3.0).sin()),
    };
    let height = MISS_HEIGHT * (1.0 - squash);
    let width = MISS_HEIGHT * (1.0 + squash) * texture.width() / texture.height();
    let top = MISS_FEET.y - lift - height;
    if let Some(t) = cheer {
        // 몸 뒤에서 나오는 팔 (그림의 작은 손 자리 조금 위)
        for side in [-1.0, 1.0] {
            let wave = (t * 10.0 + side).sin() * 0.12;
            let shoulder = vec2(MISS_FEET.x + side * width * 0.30, top + height * 0.55);
            let hand = shoulder + vec2(side * (width * 0.30 + wave * width), -height * (0.42 + wave));
            draw_line(shoulder.x, shoulder.y, hand.x, hand.y, 9.0, BLACK);
            draw_line(shoulder.x, shoulder.y, hand.x, hand.y, 5.0, WHITE);
            draw_circle(hand.x, hand.y, 8.0, BLACK);
            draw_circle(hand.x, hand.y, 5.5, WHITE);
        }
    }
    let params = DrawTextureParams { dest_size: Some(vec2(width, height)), ..Default::default() };
    draw_texture_ex(texture, MISS_FEET.x - width / 2.0, top, WHITE, params);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        MATH_ADD_THREE_STORIES, MATH_ADD_TWO_STORIES, MATH_MULTIPLY_STORIES, MATH_STORY_NAMES, MATH_SUBTRACT_STORIES,
    };
    use crate::logic::math::{fill_story, MathQuestionMaker};
    use crate::view::touch::BACK_BUTTON;

    /// 한글 한 글자는 글자 크기보다 넓지 않다. 그래서 "글자 수 × 크기"는 실제 폭보다 넉넉한 어림이다.
    fn wide(size: f32) -> impl Fn(&str) -> f32 {
        move |s: &str| s.chars().count() as f32 * size
    }

    #[test]
    fn pad_buttons_are_big_apart_and_on_screen() {
        let buttons: Vec<(Rect, MathKey)> = pad_buttons().collect();
        assert_eq!(buttons.len(), 12);
        for d in 0..10 {
            assert!(buttons.iter().any(|(_, k)| *k == MathKey::Digit(d)));
        }
        const { assert!(PAD_BUTTON >= 72.0 && PAD_GAP >= 12.0) };
        for (i, (a, _)) in buttons.iter().enumerate() {
            assert!(a.x >= PANEL.x && a.right() <= PANEL.right() && a.y >= PANEL.y && a.bottom() <= PANEL.bottom(), "{a:?}");
            assert!(a.x >= LEFT_RIGHT, "왼쪽 칸과 겹치지 않는다");
            for (b, _) in &buttons[i + 1..] {
                let apart = (a.x - b.right()).max(b.x - a.right()).max(a.y - b.bottom()).max(b.y - a.bottom());
                assert!(apart >= PAD_GAP, "{a:?} {b:?}");
            }
        }
        assert_eq!(pad_key_at(buttons[11].0.center()), Some(MathKey::Submit));
        assert_eq!(pad_key_at(vec2(PAD_LEFT + PAD_BUTTON + PAD_GAP / 2.0, PAD_TOP + 10.0)), None, "사이는 아무 버튼도 아니다");
    }

    #[test]
    fn back_button_stays_outside_the_panel() {
        assert!(BACK_BUTTON.right() < PANEL.x);
    }

    #[test]
    fn longest_question_fits_above_the_answer_box() {
        let stories = [MATH_MULTIPLY_STORIES, MATH_ADD_TWO_STORIES, MATH_ADD_THREE_STORIES, MATH_SUBTRACT_STORIES];
        let mut longest = String::new();
        for story in stories.iter().flat_map(|s| s.iter()) {
            for name in MATH_STORY_NAMES {
                let text = fill_story(story, name, [999, 999, 999]);
                if text.chars().count() > longest.chars().count() {
                    longest = text;
                }
            }
        }
        let lines = wrap_lines(&longest, LEFT_RIGHT - LEFT_X, wide(QUESTION_SIZE));
        let bottom = QUESTION_TOP + QUESTION_SIZE + (lines.len() as f32 - 1.0) * QUESTION_LINE;
        assert!(bottom + 8.0 < ANSWER_BOX.y, "{} 줄: {longest}", lines.len());
        assert_eq!(lines.join(" "), longest, "글자를 빼먹지 않는다");
    }

    #[test]
    fn miss_words_fit_in_two_lines_of_the_bubble() {
        let mut quiz = MathQuiz::with_maker(MathQuestionMaker::new(3));
        quiz.add_waiting();
        quiz.open_next_waiting();
        for text in [MATH_MISS_ASK_TEXT, MATH_TRY_AGAIN_TEXT, "너무 아쉽다! 정답은 999야"] {
            assert!(bubble_lines(text, wide(BUBBLE_SIZE)).len() <= 2, "{text}");
        }
        assert_eq!(miss_says(&quiz), MATH_MISS_ASK_TEXT);
        assert_eq!(bubble_lines(MATH_TRY_AGAIN_TEXT, wide(1.0)), ["너무 아쉽다!", "다시 해 볼까?"]);
    }

    #[test]
    fn wrap_breaks_at_spaces_and_cuts_long_words() {
        assert_eq!(wrap_lines("가나 다라 마", 5.0, wide(1.0)), ["가나 다라", "마"]);
        assert_eq!(wrap_lines("가나다라마바", 4.0, wide(1.0)), ["가나다라", "마바"]);
        assert!(wrap_lines("", 4.0, wide(1.0)).is_empty());
    }

    #[test]
    fn keys_go_to_the_quiz() {
        let mut quiz = MathQuiz::with_maker(MathQuestionMaker::new(3));
        quiz.add_waiting();
        quiz.open_next_waiting();
        let answer = quiz.question().unwrap().answer;
        MathKey::Digit(9).apply(&mut quiz);
        MathKey::Erase.apply(&mut quiz);
        assert_eq!(quiz.typed(), "");
        for c in answer.to_string().chars() {
            MathKey::Digit(c.to_digit(10).unwrap()).apply(&mut quiz);
        }
        MathKey::Submit.apply(&mut quiz);
        assert_eq!(quiz.result(), Some(MathResult::Right));
        assert_eq!(miss_says(&quiz), crate::config::MATH_RIGHT_TEXT);
    }
}
