//! 잠깐 퀴즈의 영어 낱말 문제 (US-087·088, D82·D106). 규칙은 `logic::english`와 `logic::math::MathQuiz`에 있고,
//! 여기서는 보기 4개(2×2)와 묻는 그림·낱말을 그리고, 누른 자리를 칸 번호(0~3)로 바꾼다.
//! 창 판·제목·미스·말풍선은 수학과 같은 것을 `view::math`가 그린다. 숫자판은 그리지 않는다.
//!
//! 그림은 헤일리가 그린 `assets/sprites/english/en-<낱말>.png`를 config 낱말 표(`ENGLISH_WORDS`) 차례로 시작할 때 한 번에 불러온다.
//! 파일이 없거나 읽지 못하면 그 칸은 빈 칸에 "?"로 그린다 (게임은 멈추지 않는다). Tree는 게임의 나무 그림이다.

use super::gather::draw_tree;
use super::hud::draw_pill;
use super::text::TextPainter;
use crate::config::{ENGLISH_CHOICE_COUNT, ENGLISH_GAME_TREE_PICTURE, ENGLISH_WORDS};
use crate::logic::english::{EnglishKind, EnglishQuestion};
use crate::logic::math::{MathQuiz, MathResult};
use macroquad::prelude::*;
use std::collections::HashMap;

/// 영어 그림이 있는 폴더 (`EnglishWord::picture`가 이 폴더 안의 파일 이름이다)
pub const ENGLISH_PICTURE_DIR: &str = "assets/sprites/english";

/// 보기 칸 하나 크기와 사이 ([임시-61]: 72픽셀 이상, 사이 12픽셀 이상)
pub const CHOICE_W: f32 = 300.0;
pub const CHOICE_H: f32 = 112.0;
pub const CHOICE_GAP: f32 = 16.0;
/// 보기 4개(2×2)의 왼쪽 위. 창 판 가운데(565)에 맞춘다.
const GRID_LEFT: f32 = 565.0 - CHOICE_GAP / 2.0 - CHOICE_W;
const GRID_TOP: f32 = 282.0;
/// 묻는 그림·낱말 칸 (말풍선 오른쪽)
pub const CUE: Rect = Rect::new(676.0, 84.0, 260.0, 128.0);
/// 묻는 글 ("이 그림은 영어로 뭘까?")과 남은 기회 글: 같은 줄, 묻는 글은 왼쪽, 기회 글은 오른쪽 끝에 붙인다
pub const ASK_X: f32 = 196.0;
pub const ASK_Y: f32 = 262.0;
pub const ASK_SIZE: f32 = 30.0;
pub const TRIES_RIGHT: f32 = 936.0;
pub const TRIES_SIZE: f32 = 26.0;
/// 낱말 글 크기: 묻는 칸의 낱말, 보기 칸의 낱말
pub const CUE_WORD_SIZE: f32 = 52.0;
pub const CHOICE_WORD_SIZE: f32 = 44.0;
/// 보기 칸 왼쪽 위의 번호 동그라미 (PC 숫자 키 1~4)
const BADGE_RADIUS: f32 = 20.0;
const BADGE_SIZE: f32 = 28.0;
/// 번호 동그라미 자리를 뺀 보기 칸 안쪽 (그림·낱말이 여기 들어간다)
pub fn choice_inner(r: Rect) -> Rect {
    let left = r.x + BADGE_RADIUS * 2.0 + 16.0;
    Rect::new(left, r.y + 8.0, r.right() - 12.0 - left, r.h - 16.0)
}

const CARD_FILL: Color = Color::new(0.35, 0.68, 1.0, 1.0);
const PICTURE_FILL: Color = Color::new(1.0, 0.98, 0.90, 1.0);
const CUE_WORD_FILL: Color = Color::new(0.30, 0.80, 0.42, 1.0);
const BADGE_FILL: Color = Color::new(0.20, 0.28, 0.55, 1.0);
const UNKNOWN_TEXT: Color = Color::new(0.55, 0.58, 0.70, 1.0);
const STAR: Color = Color::new(1.0, 0.86, 0.22, 1.0);
/// 흐린 칸 위에 덮는 색 (창 판 색). 그림·낱말·나무가 모두 같이 흐려진다.
const DIM_OVER: Color = Color::new(0.20, 0.28, 0.55, 0.68);

/// 영어 그림들. 그림 이름(`EnglishWord::picture`) → 그림. 없는 그림은 들어 있지 않다.
pub struct EnglishArt {
    pictures: HashMap<&'static str, Texture2D>,
}

impl EnglishArt {
    /// 낱말 표의 그림(`names` 차례)을 미리 받아 둔 파일 내용으로 만든다 (Tree는 코드로 그리므로 뺀다).
    /// 없거나 읽지 못한 그림은 건너뛴다.
    pub fn from_files(names: &[&'static str], files: Vec<super::assets::FileResult>) -> EnglishArt {
        let mut pictures = HashMap::new();
        for (name, file) in names.iter().zip(files) {
            let Ok(bytes) = file else { continue };
            let Ok(image) = Image::from_file_with_format(&bytes, Some(ImageFormat::Png)) else { continue };
            let texture = Texture2D::from_image(&image);
            texture.set_filter(FilterMode::Linear);
            pictures.insert(*name, texture);
        }
        EnglishArt { pictures }
    }

    fn picture(&self, name: &str) -> Option<&Texture2D> {
        self.pictures.get(name)
    }
}

/// 불러올 그림 파일 이름들 (낱말 표 차례, Tree 빼고).
pub fn picture_files() -> impl Iterator<Item = &'static str> {
    ENGLISH_WORDS.iter().map(|(_, picture)| *picture).filter(|p| *p != ENGLISH_GAME_TREE_PICTURE)
}

pub fn picture_path(name: &str) -> String {
    format!("{ENGLISH_PICTURE_DIR}/{name}")
}

/// 보기 칸 자리. 0 = 1번(왼쪽 위), 1 = 2번(오른쪽 위), 2 = 3번(왼쪽 아래), 3 = 4번(오른쪽 아래).
pub fn choice_rect(slot: usize) -> Rect {
    let (col, row) = ((slot % 2) as f32, (slot / 2) as f32);
    Rect::new(GRID_LEFT + col * (CHOICE_W + CHOICE_GAP), GRID_TOP + row * (CHOICE_H + CHOICE_GAP), CHOICE_W, CHOICE_H)
}

/// 가상 화면의 이 자리를 누르면 어느 보기인지 (사이를 누르면 None).
pub fn choice_slot_at(point: Vec2) -> Option<usize> {
    (0..ENGLISH_CHOICE_COUNT).find(|&slot| choice_rect(slot).contains(point))
}

/// 숫자 키 1~4 → 보기 칸 0~3. 다른 숫자는 None.
pub fn slot_for_digit(digit: u32) -> Option<usize> {
    (1..=ENGLISH_CHOICE_COUNT as u32).contains(&digit).then(|| digit as usize - 1)
}

/// 한 칸을 어떻게 보여 줄지.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoiceLook {
    /// 고를 수 있다
    Normal,
    /// 틀려서 흐려졌거나, 결과 글을 보여 주는 동안 다른 칸 (다시 눌러도 아무 일이 없다)
    Dim,
    /// 맞힌 칸 또는 두 번 다 틀린 뒤 알려 주는 정답 칸: 반짝인다
    Shine,
}

pub fn choice_look(quiz: &MathQuiz, question: &EnglishQuestion, slot: usize) -> ChoiceLook {
    match quiz.result() {
        Some(MathResult::Right | MathResult::Wrong) if slot == question.answer => ChoiceLook::Shine,
        Some(MathResult::Right | MathResult::Wrong) => ChoiceLook::Dim,
        _ if quiz.is_crossed_out(slot) => ChoiceLook::Dim,
        _ => ChoiceLook::Normal,
    }
}

/// 영어 문제 창의 아래쪽: 묻는 그림·낱말, 묻는 글, 남은 기회, 보기 4개.
pub fn draw_english(text: &TextPainter, quiz: &MathQuiz, question: &EnglishQuestion, art: &EnglishArt) {
    let shown = question.answer_word();
    // 묻는 칸: (가) 그림 하나, (나) 낱말 하나
    match question.kind {
        EnglishKind::PictureToWord => {
            draw_pill(CUE.x, CUE.y, CUE.w, CUE.h, PICTURE_FILL);
            draw_picture(text, art, shown.picture, square_in(CUE, 8.0));
        }
        EnglishKind::WordToPicture => {
            draw_pill(CUE.x, CUE.y, CUE.w, CUE.h, CUE_WORD_FILL);
            text.centered(shown.word, CUE.center().x, CUE.center().y + CUE_WORD_SIZE * 0.36, CUE_WORD_SIZE, WHITE);
        }
    }
    draw_rectangle_lines(CUE.x, CUE.y, CUE.w, CUE.h, 4.0, STAR);

    text.left(question.kind.ask_text(), ASK_X, ASK_Y, ASK_SIZE, WHITE);
    if quiz.result() != Some(MathResult::Wrong) {
        // 기회를 다 쓰면 "0번 남았어" 대신 말풍선의 정답만 보인다 (수학과 같다)
        let tries = quiz.tries_left_text();
        text.left(&tries, TRIES_RIGHT - text.width(&tries, TRIES_SIZE), ASK_Y, TRIES_SIZE, WHITE);
    }

    let blink = (get_time() * 4.0).fract() < 0.5;
    for (slot, choice) in question.choices.iter().enumerate() {
        let r = choice_rect(slot);
        let look = choice_look(quiz, question, slot);
        let fill = if look == ChoiceLook::Shine && blink { STAR } else { CARD_FILL };
        draw_pill(r.x, r.y, r.w, r.h, fill);
        let inner = choice_inner(r);
        match question.kind {
            EnglishKind::PictureToWord => {
                text.centered(choice.word, inner.center().x, inner.center().y + CHOICE_WORD_SIZE * 0.36, CHOICE_WORD_SIZE, WHITE);
            }
            EnglishKind::WordToPicture => {
                let square = square_in(inner, 0.0);
                draw_rectangle(square.x, square.y, square.w, square.h, PICTURE_FILL);
                draw_picture(text, art, choice.picture, square_in(square, 4.0));
            }
        }
        // 번호 (PC 숫자 키 1~4)
        let badge = vec2(r.x + BADGE_RADIUS + 10.0, r.y + BADGE_RADIUS + 10.0);
        draw_circle(badge.x, badge.y, BADGE_RADIUS, BADGE_FILL);
        text.centered(&(slot + 1).to_string(), badge.x, badge.y + BADGE_SIZE * 0.36, BADGE_SIZE, WHITE);
        match look {
            ChoiceLook::Dim => draw_rectangle(r.x, r.y, r.w, r.h, DIM_OVER),
            ChoiceLook::Shine => draw_rectangle_lines(r.x - 3.0, r.y - 3.0, r.w + 6.0, r.h + 6.0, 6.0, STAR),
            ChoiceLook::Normal => {}
        }
    }
}

/// `r` 가운데에 `margin`만큼 안쪽으로 들인 정사각형.
pub fn square_in(r: Rect, margin: f32) -> Rect {
    let side = (r.w.min(r.h) - margin * 2.0).max(0.0);
    Rect::new(r.center().x - side / 2.0, r.center().y - side / 2.0, side, side)
}

/// 그림 하나를 정사각형 `r` 안에 비율을 지켜 그린다. Tree는 게임의 나무, 없는 그림은 "?".
fn draw_picture(text: &TextPainter, art: &EnglishArt, name: &str, r: Rect) {
    if name == ENGLISH_GAME_TREE_PICTURE {
        draw_tree(tree_rect(r), true, 0.0);
        return;
    }
    let Some(texture) = art.picture(name) else {
        let size = r.h * 0.6;
        text.centered("?", r.center().x, r.center().y + size * 0.36, size, UNKNOWN_TEXT);
        return;
    };
    let scale = (r.w / texture.width()).min(r.h / texture.height());
    let size = vec2(texture.width(), texture.height()) * scale;
    let params = DrawTextureParams { dest_size: Some(size), ..Default::default() };
    draw_texture_ex(texture, r.center().x - size.x / 2.0, r.center().y - size.y / 2.0, WHITE, params);
}

/// `draw_tree`에 넘길 네모: 나뭇잎 동그라미가 네모 밖(옆·위)으로 나오므로, 나무 전체가 `square` 안에 들게 줄인다.
pub fn tree_rect(square: Rect) -> Rect {
    let s = square.w;
    Rect::new(square.center().x - s * 0.31, square.y + s * 0.12, s * 0.62, s * 0.85)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::screen::{Letterbox, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

    fn font() -> fontdue::Font {
        let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/Jua-Regular.ttf")).unwrap();
        fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).unwrap()
    }

    /// Jua 글꼴로 잰 글 폭 (테두리 몫을 조금 더한다)
    fn width(font: &fontdue::Font, text: &str, size: f32) -> f32 {
        text.chars().map(|c| font.metrics(c, size).advance_width).sum::<f32>() + size * 0.2
    }

    fn apart(a: Rect, b: Rect) -> f32 {
        (a.x - b.right()).max(b.x - a.right()).max(a.y - b.bottom()).max(b.y - a.bottom())
    }

    #[test]
    fn jua_font_has_every_english_letter() {
        // 없는 글자는 네모(□)로 나온다. 글리프 0번이 "없는 글자"다.
        let font = font();
        for c in ('a'..='z').chain('A'..='Z').chain(['?']) {
            assert_ne!(font.lookup_glyph_index(c), 0, "Jua에 {c}가 없어요");
            assert!(font.metrics(c, 40.0).width > 0, "{c} 그림이 비었어요");
        }
    }

    #[test]
    fn choices_are_big_apart_and_inside_the_panel() {
        const { assert!(CHOICE_W >= 72.0 && CHOICE_H >= 72.0 && CHOICE_GAP >= 12.0) };
        let panel = crate::view::math::PANEL;
        let rects: Vec<Rect> = (0..ENGLISH_CHOICE_COUNT).map(choice_rect).collect();
        for (i, a) in rects.iter().enumerate() {
            assert!(a.x >= panel.x && a.right() <= panel.right() && a.y >= panel.y && a.bottom() <= panel.bottom(), "{a:?}");
            assert!(a.y > ASK_Y + 8.0, "묻는 글·기회 글 아래");
            for b in &rects[i + 1..] {
                assert!(apart(*a, *b) >= 12.0, "{a:?} {b:?}");
            }
            // 그림 칸도 72픽셀 이상
            assert!(square_in(choice_inner(*a), 0.0).w >= 72.0);
        }
        assert!(square_in(CUE, 8.0).w >= 72.0);
        // 1번 왼쪽 위, 2번 오른쪽 위, 3번 왼쪽 아래, 4번 오른쪽 아래
        assert!(rects[0].x < rects[1].x && rects[0].y == rects[1].y);
        assert!(rects[2].y > rects[0].y && rects[2].x == rects[0].x && rects[3].x == rects[1].x);
        assert!(CUE.x >= panel.x && CUE.right() <= panel.right() && CUE.bottom() < ASK_Y - ASK_SIZE);
        assert!(crate::view::touch::BACK_BUTTON.right() < rects[0].x);
    }

    #[test]
    fn tapping_a_choice_finds_its_slot_and_gaps_find_nothing() {
        for slot in 0..ENGLISH_CHOICE_COUNT {
            assert_eq!(choice_slot_at(choice_rect(slot).center()), Some(slot));
        }
        let between = vec2(choice_rect(0).right() + CHOICE_GAP / 2.0, choice_rect(0).center().y);
        assert_eq!(choice_slot_at(between), None);
        assert_eq!(choice_slot_at(CUE.center()), None, "묻는 그림을 눌러도 고르지 않는다");
        assert_eq!([1, 2, 3, 4].map(slot_for_digit), [Some(0), Some(1), Some(2), Some(3)]);
        assert_eq!(slot_for_digit(0), None);
        assert_eq!(slot_for_digit(5), None);
    }

    #[test]
    fn every_word_fits_its_box_and_texts_do_not_overlap() {
        let font = font();
        let inner = choice_inner(choice_rect(0));
        for (word, _) in ENGLISH_WORDS {
            assert!(width(&font, word, CHOICE_WORD_SIZE) <= inner.w, "보기 칸의 {word}");
            assert!(width(&font, word, CUE_WORD_SIZE) <= CUE.w - 16.0, "묻는 칸의 {word}");
        }
        let longest_ask = [EnglishKind::PictureToWord, EnglishKind::WordToPicture]
            .map(|k| width(&font, k.ask_text(), ASK_SIZE))
            .into_iter()
            .fold(0.0, f32::max);
        // 기회 글은 처음이 가장 길다 ("기회가 2번 남았어")
        let tries = width(&font, &crate::config::MATH_TRIES_LEFT_TEXT.replace('□', "2"), TRIES_SIZE);
        assert!(ASK_X + longest_ask + 12.0 <= TRIES_RIGHT - tries, "묻는 글과 기회 글이 겹치지 않는다");
        // 정답 글 말풍선은 두 줄 안 (가장 긴 낱말)
        let longest = ENGLISH_WORDS.iter().map(|(w, _)| *w).max_by_key(|w| w.len()).unwrap();
        let says = crate::config::ENGLISH_WRONG_TEXT.replace('□', longest);
        assert!(crate::view::math::bubble_line_count(&says, |s| width(&font, s, crate::view::math::BUBBLE_SIZE)) <= 2, "{says}");
    }

    #[test]
    fn whole_window_stays_on_small_and_big_screens() {
        // 가상 화면(960×540)을 비율 그대로 줄이고 키우므로, 판 안에 있으면 어느 창 크기에서도 잘리지 않는다.
        for (w, h) in [(740.0, 360.0), (812.0, 375.0), (960.0, 540.0), (1200.0, 700.0)] {
            let lb = Letterbox::for_window(w, h);
            for r in (0..ENGLISH_CHOICE_COUNT).map(choice_rect).chain([CUE]) {
                let top_left = lb.offset + vec2(r.x, r.y) * lb.scale;
                let bottom_right = lb.offset + vec2(r.right(), r.bottom()) * lb.scale;
                assert!(top_left.x >= 0.0 && top_left.y >= 0.0 && bottom_right.x <= w && bottom_right.y <= h, "{w}×{h} {r:?}");
                // 폰에서도 손가락으로 누를 만한 크기 (창 픽셀 40 이상)
                assert!(r.h * lb.scale >= 40.0, "{w}×{h}");
            }
            assert!(VIRTUAL_WIDTH * lb.scale <= w + 0.01 && VIRTUAL_HEIGHT * lb.scale <= h + 0.01);
        }
    }

    #[test]
    fn tree_picture_stays_inside_its_square() {
        // draw_tree: 줄기 아래 끝 = r 아래, 나뭇잎은 가운데 ± 0.68w, 맨 위는 r.y + 0.36h - 0.65w
        let square = Rect::new(100.0, 100.0, 96.0, 96.0);
        let r = tree_rect(square);
        assert!(r.center().x - r.w * 0.68 >= square.x && r.center().x + r.w * 0.68 <= square.right());
        assert!(r.y + r.h * 0.36 - r.w * 0.65 >= square.y && r.bottom() <= square.bottom());
    }

    #[test]
    fn every_picture_file_is_there_and_readable() {
        // 낱말 표에 있는 그림만 (표에서 뺀 낱말의 그림은 불러오지 않는다). Tree는 파일이 없다 (게임 나무).
        let mut missing = Vec::new();
        for name in picture_files() {
            let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), picture_path(name));
            match std::fs::read(&path) {
                Ok(bytes) => assert!(Image::from_file_with_format(&bytes, Some(ImageFormat::Png)).is_ok(), "{name}을 읽지 못했어요"),
                Err(_) => missing.push(name),
            }
        }
        assert_eq!(picture_files().count(), ENGLISH_WORDS.len() - 1);
        assert!(missing.is_empty(), "그림이 없어요: {missing:?}");
    }

    #[test]
    fn crossed_out_and_end_looks() {
        use crate::logic::english::EnglishQuestionMaker;
        use crate::logic::math::MathQuestionMaker;
        let mut quiz = MathQuiz::with_subjects(MathQuestionMaker::new(1), EnglishQuestionMaker::new(1), 1, 100);
        quiz.add_waiting();
        quiz.open_next_waiting();
        let question = quiz.english_question().unwrap().clone();
        let looks = |quiz: &MathQuiz| (0..4).map(|s| choice_look(quiz, &question, s)).collect::<Vec<_>>();
        assert!(looks(&quiz).iter().all(|l| *l == ChoiceLook::Normal));
        let wrong = (0..4).find(|&s| s != question.answer).unwrap();
        quiz.pick_choice(wrong);
        assert_eq!(looks(&quiz)[wrong], ChoiceLook::Dim, "틀린 칸은 흐리다");
        assert_eq!(looks(&quiz)[question.answer], ChoiceLook::Normal);
        let wrong2 = (0..4).find(|&s| s != question.answer && s != wrong).unwrap();
        quiz.pick_choice(wrong2);
        let end = looks(&quiz);
        assert_eq!(end[question.answer], ChoiceLook::Shine, "두 번 다 틀리면 정답 칸이 반짝인다");
        assert_eq!(end.iter().filter(|l| **l == ChoiceLook::Dim).count(), 3);
    }
}
