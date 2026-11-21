//! 튜토리얼 말풍선 화면 (US-119·120). 규칙(단계·칭찬·건너뛰기)은 `logic::tutorial`에 있고 여기서는 보여 주기만 한다.
//!
//! - 위쪽 가운데에 말풍선 카드: 왼쪽에 캐릭터 Niro 얼굴(어떤 캐릭터를 골라도 Niro가 말한다, 임시-134), 글, 오른쪽에 "건너뛰기".
//! - 자리는 글과 상관없이 고정이다 (입력과 그리기가 같은 자리를 쓴다). PC는 미션 진행 줄 아래, 폰은 멈춤 버튼 아래.
//!   게임 속(몬스터·미스·캐릭터)은 화면 아래쪽 땅 위라서 가리지 않는다 (점프해도 머리가 카드 아래에 있다).
//! - 말풍선이 떠도 게임은 멈추지 않는다. 건너뛰기를 누른 손가락은 조이스틱·버튼으로 새지 않는다 (`claim_skip_finger`).

use super::assets::{face_source, Assets};
use super::effects::Effects;
use super::gather::GatherView;
use super::hud::draw_pill;
use super::mission::{draw_round_rect, split_lines};
use super::screen::VIRTUAL_WIDTH;
use super::text::{TextPainter, TEXT_ABOVE, TEXT_BELOW};
use super::touch::{draw_star, Finger};
use crate::config::*;
use crate::logic::character::CharacterKind;
use crate::logic::game::Game;
use macroquad::prelude::*;

fn rgba(c: [f32; 4]) -> Color {
    Color::new(c[0], c[1], c[2], c[3])
}

/// 말풍선 카드 전체 (얼굴 포함).
pub fn card_rect(touch_mode: bool) -> Rect {
    let top = if touch_mode { TUTORIAL_BUBBLE_TOP_TOUCH } else { TUTORIAL_BUBBLE_TOP_PC };
    Rect::new((VIRTUAL_WIDTH - TUTORIAL_BUBBLE_WIDTH) / 2.0, top, TUTORIAL_BUBBLE_WIDTH, TUTORIAL_BUBBLE_HEIGHT)
}

/// 얼굴 오른쪽의 풍선 몸통.
fn body_rect(touch_mode: bool) -> Rect {
    let card = card_rect(touch_mode);
    let left = card.x + TUTORIAL_FACE_RADIUS * 2.0 + TUTORIAL_FACE_GAP;
    Rect::new(left, card.y, card.right() - left, card.h)
}

/// 건너뛰기 버튼: 풍선 몸통 오른쪽 안.
pub fn skip_rect(touch_mode: bool) -> Rect {
    let body = body_rect(touch_mode);
    Rect::new(body.right() - TUTORIAL_PAD - TUTORIAL_SKIP_WIDTH, body.y + (body.h - TUTORIAL_SKIP_HEIGHT) / 2.0, TUTORIAL_SKIP_WIDTH, TUTORIAL_SKIP_HEIGHT)
}

/// 글이 들어가는 자리: 풍선 몸통 안, 건너뛰기 왼쪽.
pub fn text_rect(touch_mode: bool) -> Rect {
    let body = body_rect(touch_mode);
    let left = body.x + TUTORIAL_PAD;
    let right = skip_rect(touch_mode).x - TUTORIAL_PAD;
    Rect::new(left, body.y, right - left, body.h)
}

/// 말풍선 자리를 다른 안내가 쓰고 있는가: 미스가 미션을 말하는 3초, 처음 나무를 캤을 때 곡괭이 안내 4초.
/// 그리기(`bubble_text`)·건너뛰기 받기·logic의 칭찬 시간 멈춤(`Tutorial::set_notice_up`, BUG-040)이 모두 이 한 값을 쓴다.
pub fn other_notice_up(effects: &Effects, gathered: &GatherView) -> bool {
    effects.mission.miss_is_talking() || gathered.recipe_hint_showing()
}

/// 지금 말풍선에 쓸 글. 칭찬 중이면 칭찬 글, 단계가 진행 중이면 그 단계 글, 아니면(시작 전·건너뜀·끝) None.
/// `other_notice`: 미스가 미션을 말하는 3초, 처음 나무를 캤을 때 곡괭이 안내 4초 동안은 같은 위쪽 가운데 자리를 그 안내가 먼저 쓴다.
/// 그동안만 말풍선을 숨긴다 (게임·단계는 그대로 간다).
pub fn bubble_text(game: &Game, touch_mode: bool, other_notice: bool) -> Option<&'static str> {
    if other_notice {
        return None;
    }
    let step = game.tutorial.current_step()?;
    Some(if game.tutorial.is_cheering() { TUTORIAL_CHEER_TEXT } else { step.text(touch_mode) })
}

/// 큰 알림 글이 말풍선 아래로 쌓이게 알려 주는 말풍선 아래 끝. 말풍선이 없으면 None.
pub fn bottom(game: &Game, touch_mode: bool, other_notice: bool) -> Option<f32> {
    bubble_text(game, touch_mode, other_notice).map(|_| card_rect(touch_mode).bottom() + TUTORIAL_BUBBLE_GAP_BELOW)
}

/// 글을 한 줄, 넘치면 두 줄로 (가운데에 가까운 띄어쓰기에서).
pub fn wrap(words: &str, max_w: f32, width: impl Fn(&str) -> f32) -> Vec<String> {
    split_lines(words, max_w, width)
}

/// 건너뛰기를 눌렀는지: PC 키, 마우스 클릭, 새로 댄 손가락 중 하나가 버튼 안이면 true.
pub fn skip_pressed(key: bool, click: Option<Vec2>, new_finger: Option<Vec2>, rect: Rect) -> bool {
    key || click.is_some_and(|p| rect.contains(p)) || new_finger.is_some_and(|p| rect.contains(p))
}

/// 새로 댄 손가락 중 건너뛰기 버튼 안에서 시작한 손가락. 이 손가락은 조이스틱·버튼에 넘기지 않는다.
pub fn claim_skip_finger(fingers: &[Finger], rect: Rect) -> Option<Finger> {
    fingers.iter().find(|f| f.phase == TouchPhase::Started && rect.contains(f.pos)).copied()
}

/// 건너뛰기에 쓴 손가락을 뺀 손가락 목록 (조이스틱·버튼용).
pub fn without_finger(fingers: &[Finger], claimed: Option<u64>) -> Vec<Finger> {
    fingers.iter().filter(|f| Some(f.id) != claimed).copied().collect()
}

/// 말풍선 카드를 그린다 (화면 좌표). `words`는 `bubble_text`가 준 글, `skip_key`는 PC에서 버튼 옆에 보이는 키 글자.
pub fn draw(assets: &Assets, painter: &TextPainter, words: &str, cheering: bool, touch_mode: bool, skip_key: &str) {
    let t = get_time() as f32;
    let card = card_rect(touch_mode);
    let body = body_rect(touch_mode);
    let face = vec2(card.x + TUTORIAL_FACE_RADIUS, card.y + card.h / 2.0);
    // Niro 얼굴 (아이 원화에서 잘라 낸 그림 그대로)
    draw_circle(face.x, face.y, TUTORIAL_FACE_RADIUS, rgba(TUTORIAL_FACE_BACK_COLOR));
    let source = face_source(CharacterKind::Niro);
    let fit = TUTORIAL_FACE_RADIUS * 1.7 / source.w.max(source.h);
    let size = vec2(source.w, source.h) * fit;
    let bob = (t * TUTORIAL_FACE_BOB_SPEED).sin() * TUTORIAL_FACE_BOB;
    draw_texture_ex(
        assets.character(CharacterKind::Niro),
        face.x - size.x / 2.0,
        face.y - size.y / 2.0 + 2.0 + bob,
        WHITE,
        DrawTextureParams { dest_size: Some(size), source: Some(source), ..Default::default() },
    );
    draw_circle_lines(face.x, face.y, TUTORIAL_FACE_RADIUS, 3.0, rgba(TUTORIAL_FACE_RING_COLOR));
    // 풍선과 꼬리 (얼굴 쪽)
    let color = rgba(TUTORIAL_BUBBLE_COLOR);
    draw_round_rect(body, TUTORIAL_CORNER, color);
    draw_triangle(vec2(body.x, face.y - 8.0), vec2(body.x, face.y + 8.0), vec2(body.x - TUTORIAL_FACE_GAP, face.y), color);
    // 글 (칸 안에서 가운데 정렬, 최대 두 줄)
    let area = text_rect(touch_mode);
    let lines = wrap(words, area.w, |s| painter.width(s, TUTORIAL_TEXT_SIZE));
    let step = TUTORIAL_TEXT_SIZE * TUTORIAL_LINE_STEP;
    let text_h = TUTORIAL_TEXT_SIZE * (TEXT_ABOVE + TEXT_BELOW) + lines.len().saturating_sub(1) as f32 * step;
    let mut baseline = area.center().y - text_h / 2.0 + TUTORIAL_TEXT_SIZE * TEXT_ABOVE;
    let word_color = rgba(if cheering { TUTORIAL_CHEER_COLOR } else { TUTORIAL_TEXT_COLOR });
    for line in &lines {
        painter.centered(line, area.center().x, baseline, TUTORIAL_TEXT_SIZE, word_color);
        baseline += step;
    }
    if cheering {
        // 짝짝! 카드 위 두 모서리에서 별이 반짝
        for (i, spin) in [0.0_f32, 2.1].into_iter().enumerate() {
            let c = vec2(card.x + 2.0 + i as f32 * (card.w - 4.0), card.y + 4.0);
            let r = 7.0 + 2.0 * (t * 6.0 + spin).sin();
            draw_star(c, r, t * 2.0 + spin, rgba(TUTORIAL_CHEER_COLOR));
        }
    }
    // 건너뛰기 버튼
    let skip = skip_rect(touch_mode);
    draw_pill(skip.x, skip.y, skip.w, skip.h, rgba(TUTORIAL_SKIP_FILL_COLOR));
    let label_x = if touch_mode { skip.center().x } else { skip.center().x + 10.0 };
    painter.centered(TUTORIAL_SKIP_TEXT, label_x, skip.center().y + TUTORIAL_SKIP_TEXT_SIZE * 0.36, TUTORIAL_SKIP_TEXT_SIZE, rgba(TUTORIAL_SKIP_TEXT_COLOR));
    if !touch_mode {
        // PC: 누르는 키를 작은 네모로 (폰에는 키가 없다)
        let chip = Rect::new(skip.x + 12.0, skip.center().y - 12.0, 24.0, 24.0);
        draw_rectangle(chip.x, chip.y, chip.w, chip.h, rgba(TUTORIAL_SKIP_TEXT_COLOR));
        painter.centered(skip_key, chip.center().x, chip.center().y + 6.5, 18.0, rgba(TUTORIAL_SKIP_FILL_COLOR));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::bag::bag_button_rect;
    use crate::view::gather::POPUP_TOP;
    use crate::view::hud::{hud_bottom, HELD_SLOT};
    use crate::view::mission::BELOW_TOP;
    use crate::view::screen::VIRTUAL_HEIGHT;
    use crate::view::touch::{play_buttons_with_mine, BACK_BUTTON};

    /// 한글은 폭이 글자 크기와 거의 같다. 시험은 조금 넉넉하게 1.0으로 잰다.
    fn fake_width(s: &str, size: f32) -> f32 {
        s.chars().count() as f32 * size
    }

    fn circle_touches(center: Vec2, radius: f32, r: Rect) -> bool {
        let nearest = vec2(center.x.clamp(r.x, r.right()), center.y.clamp(r.y, r.bottom()));
        nearest.distance(center) <= radius
    }

    #[test]
    fn card_stays_inside_the_screen_and_holds_the_button() {
        for touch in [false, true] {
            let card = card_rect(touch);
            assert!(card.x >= 0.0 && card.right() <= VIRTUAL_WIDTH && card.y >= 0.0 && card.bottom() <= VIRTUAL_HEIGHT);
            let skip = skip_rect(touch);
            assert!(card.contains(skip.point()) && card.contains(vec2(skip.right() - 0.1, skip.bottom() - 0.1)), "건너뛰기는 카드 안");
            let text = text_rect(touch);
            assert!(text.w > 300.0, "글 칸이 너무 좁다: {}", text.w);
            assert!(text.right() <= skip.x, "글과 건너뛰기가 겹치지 않는다");
        }
    }

    #[test]
    fn skip_button_is_big_enough_for_a_phone_thumb() {
        // 폰 667×375는 가상 화면을 0.695배로 줄인다. 그래도 가로 48 화면 픽셀 이상이다.
        let skip = skip_rect(true);
        assert!(skip.w * (667.0 / 960.0) >= 48.0, "{}", skip.w);
        assert!(skip.h * (375.0 / 540.0) >= 36.0, "{}", skip.h);
    }

    #[test]
    fn card_keeps_off_hud_and_buttons() {
        for touch in [false, true] {
            let card = card_rect(touch);
            // 위: 정보판 줄(물건 칸)과 미션 진행 줄(BELOW_TOP 아래 한 줄 36) 아래. 아래: 캔 것 팝업 위
            assert!(card.y >= hud_bottom(true, false));
            assert!(card.y >= BELOW_TOP + 42.0, "미션 진행 줄(36)·성공/실패 글(42)이 먼저다");
            assert!(card.bottom() <= POPUP_TOP, "캔 것 팝업({POPUP_TOP})을 가리지 않는다: {}", card.bottom());
            // 폰 멈춤 버튼·가방 버튼·든 무기 칸·물건 누르는 칸과 겹치지 않는다
            assert!(!card.overlaps(&BACK_BUTTON), "멈춤 버튼");
            assert!(!card.overlaps(&bag_button_rect(touch)));
            assert!(!card.overlaps(&HELD_SLOT));
            // 땅 위 캐릭터가 가장 높이 뛰어도(96) 머리 위 끝은 카드 아래다
            assert!(card.bottom() < GROUND_Y - PLAYER_HEIGHT - 96.0);
        }
        // 폰의 오른쪽 버튼(공격·바꾸기·캐기)은 아래쪽이라 카드와 멀다
        for button in play_buttons_with_mine(true, true) {
            assert!(!circle_touches(button.center, button.radius + 12.0, card_rect(true)), "{:?}", button.kind);
        }
    }

    #[test]
    fn every_text_fits_in_the_card_in_at_most_two_lines() {
        let all = TUTORIAL_TEXT_PC.iter().chain(TUTORIAL_TEXT_TOUCH.iter()).chain(std::iter::once(&TUTORIAL_CHEER_TEXT));
        for touch in [false, true] {
            let area = text_rect(touch);
            for words in all.clone() {
                let lines = wrap(words, area.w, |s| fake_width(s, TUTORIAL_TEXT_SIZE));
                assert!(lines.len() <= 2, "{words}");
                for line in &lines {
                    assert!(fake_width(line, TUTORIAL_TEXT_SIZE) <= area.w, "{line}");
                }
                let text_h = TUTORIAL_TEXT_SIZE * (TEXT_ABOVE + TEXT_BELOW) + (lines.len() - 1) as f32 * TUTORIAL_TEXT_SIZE * TUTORIAL_LINE_STEP;
                assert!(text_h <= area.h - 6.0, "글 높이 {text_h} / 칸 {}", area.h);
            }
        }
    }

    #[test]
    fn skip_by_key_click_or_new_finger_only_inside_the_button() {
        let rect = skip_rect(true);
        let inside = rect.center();
        let outside = vec2(10.0, 10.0);
        assert!(skip_pressed(true, None, None, rect));
        assert!(skip_pressed(false, Some(inside), None, rect));
        assert!(skip_pressed(false, None, Some(inside), rect));
        assert!(!skip_pressed(false, Some(outside), Some(outside), rect));
        assert!(!skip_pressed(false, None, None, rect));
    }

    #[test]
    fn the_finger_that_pressed_skip_never_reaches_the_joystick() {
        let rect = skip_rect(true);
        let on_button = Finger { id: 7, phase: TouchPhase::Started, pos: rect.center() };
        let other = Finger { id: 8, phase: TouchPhase::Started, pos: vec2(100.0, 400.0) };
        let moving_over = Finger { id: 9, phase: TouchPhase::Moved, pos: rect.center() };
        let fingers = [on_button, other, moving_over];
        let claimed = claim_skip_finger(&fingers, rect).map(|f| f.id);
        assert_eq!(claimed, Some(7), "새로 댄 손가락만 건너뛰기가 된다 (움직여 들어온 손가락은 아니다)");
        let left = without_finger(&fingers, claimed);
        assert_eq!(left.iter().map(|f| f.id).collect::<Vec<_>>(), vec![8, 9]);
        assert_eq!(without_finger(&fingers, None).len(), 3);
    }

    #[test]
    fn bubble_text_follows_the_logic_state() {
        use crate::logic::settings::Settings;
        let mut game = Game::new(CharacterKind::Ellie, 1);
        assert_eq!(bubble_text(&game, false, false), None, "시작 전에는 없다");
        assert_eq!(bottom(&game, false, false), None);
        game.start_tutorial(&Settings::default(), false);
        assert_eq!(bubble_text(&game, false, false), Some(TUTORIAL_TEXT_PC[0]));
        assert_eq!(bubble_text(&game, true, false), Some(TUTORIAL_TEXT_TOUCH[0]), "폰 문구");
        assert_eq!(bubble_text(&game, false, true), None, "미스가 미션을 말하는 동안은 숨는다");
        assert_eq!(bottom(&game, false, true), None);
        assert!(bottom(&game, true, false).unwrap() > card_rect(true).bottom());
        game.skip_tutorial();
        assert_eq!(bubble_text(&game, false, false), None, "건너뛰면 사라진다");
    }

    #[test]
    fn phone_texts_name_phone_controls_and_pc_texts_name_real_keys() {
        for words in TUTORIAL_TEXT_TOUCH {
            assert!(!words.contains(" 키"), "폰 문구에 키 이름이 나오면 안 된다: {words}");
        }
        assert!(TUTORIAL_TEXT_PC[1].contains('Q'), "공격은 Q");
        assert!(TUTORIAL_TEXT_PC[2].contains('E'), "캐기는 E");
        assert!(TUTORIAL_TEXT_PC[3].contains("스페이스"), "점프는 스페이스");
        assert!(TUTORIAL_TEXT_TOUCH[2].contains(MINE_BUTTON_TEXT), "폰 캐기 버튼 글자와 같다");
    }
}
