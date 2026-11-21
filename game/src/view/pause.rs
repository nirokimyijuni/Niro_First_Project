//! 멈춤 창 (로건, 부모님 요청 2026-11-14): 게임 중 Esc나 폰 "멈춤" 버튼을 누르면 게임이 멈추고 작은 창이 뜬다.
//! "이어서 하기"(Esc를 다시 눌러도 된다) / "처음으로 (캐릭터 고르기)".
//! 멈추는 것은 main이 그동안 `Game::update`를 부르지 않는 것이라 logic은 바꾸지 않는다 (세로 안내와 같은 방법).

use super::hud::draw_pill;
use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::text::TextPainter;
use crate::config::{PAUSE_EFFECTS_LABEL, PAUSE_MUSIC_LABEL, PAUSE_SOUND_OFF_TEXT, PAUSE_SOUND_ON_TEXT, SOUND_STEP_LABELS, SOUND_VOLUME_STEPS};
use crate::logic::character::CharacterKind;
use macroquad::prelude::*;

// 소리 줄(켜짐/꺼짐, 음악 4칸, 효과음 4칸)이 들어가도록 창을 위아래로 늘리고 두 큰 버튼은 조금 낮췄다 (US-112).
const PANEL: Rect = Rect::new(250.0, 4.0, 460.0, 520.0);
pub const RESUME_BUTTON: Rect = Rect::new(290.0, 66.0, 380.0, 86.0);
pub const QUIT_BUTTON: Rect = Rect::new(290.0, 170.0, 380.0, 86.0);
/// 소리 켜기/끄기 버튼
pub const SOUND_TOGGLE: Rect = Rect::new(290.0, 274.0, 380.0, 68.0);
/// 음악 줄·효과음 줄: 왼쪽에 이름, 오른쪽에 4칸 (끔·작게·보통·크게)
const MUSIC_ROW_Y: f32 = 352.0;
const EFFECTS_ROW_Y: f32 = 430.0;
const SOUND_ROW_H: f32 = 68.0;
const CELL_X: f32 = 380.0;
const CELL_W: f32 = 68.0;
const CELL_GAP: f32 = 4.0;

// 글은 config.rs에 모은다 (REVIEW-020 3번). 여기서는 가리키기만 한다.
pub const PAUSE_TITLE_TEXT: &str = crate::config::PAUSE_TITLE_TEXT;
pub const RESUME_TEXT: &str = crate::config::PAUSE_RESUME_TEXT;
pub const QUIT_TEXT: &str = crate::config::PAUSE_QUIT_TEXT;
pub const QUIT_SUB_TEXT: &str = crate::config::PAUSE_QUIT_SUB_TEXT;

const DIM: Color = Color::new(0.0, 0.0, 0.0, 0.45);
const PANEL_FILL: Color = Color::new(0.10, 0.13, 0.32, 0.92);
const PANEL_LINE: Color = Color::new(1.0, 1.0, 1.0, 0.7);
/// 고른 칸 (↑/↓·Enter, 부모님 요청 2026-11-15): 주황 바탕. 다른 칸은 파랑.
pub const PICKED_FILL: Color = Color::new(1.0, 0.66, 0.12, 1.0);
const OTHER_FILL: Color = Color::new(0.30, 0.62, 0.90, 1.0);
const PICK_RING_COLOR: Color = WHITE;
const PICK_RING: f32 = 7.0;
const TITLE: Color = Color::new(1.0, 0.86, 0.22, 1.0);

/// 멈춤 창 옆에서 기다려 주는 친구 (아이 결정 2026-11-20): 창이 열릴 때마다 넷 중 하나를 랜덤으로 고른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseBuddy {
    Hero(CharacterKind),
    Miss,
}

impl PauseBuddy {
    pub const ALL: [PauseBuddy; 4] =
        [PauseBuddy::Hero(CharacterKind::Niro), PauseBuddy::Hero(CharacterKind::Ellie), PauseBuddy::Hero(CharacterKind::Logan), PauseBuddy::Miss];

    /// 아무 수(`roll`, 예: 지금 시각 밀리초)로 넷 중 하나를 고른다.
    pub fn pick(roll: u32) -> PauseBuddy {
        // 섞어서 밀리초가 비슷해도 고르게 나오게 한다
        let mixed = roll.wrapping_mul(0x9E37_79B9) >> 16;
        PauseBuddy::ALL[mixed as usize % PauseBuddy::ALL.len()]
    }
}

/// 친구가 서는 칸 (창 오른쪽 바깥, 창·버튼과 안 겹친다). 발은 칸 아래 가운데.
const BUDDY_BOX: Rect = Rect::new(PANEL.x + PANEL.w + 20.0, 200.0, 210.0, 230.0);
/// 통통: 살짝만 뛴다 (가상 픽셀), 한 번 뛰는 데 걸리는 초
const BUDDY_HOP: f32 = 10.0;
const BUDDY_HOP_SECONDS: f32 = 0.9;

/// `t`초에 친구가 땅에서 뜬 높이 (0 ~ `BUDDY_HOP`). 부드럽게 통통.
fn buddy_lift(t: f32) -> f32 {
    ((t / BUDDY_HOP_SECONDS).fract() * std::f32::consts::PI).sin().abs() * BUDDY_HOP
}

/// 그림 비율을 지키며 `BUDDY_BOX`(뛰는 높이 빼고) 안에 들어가는 크기.
fn buddy_size(texture_w: f32, texture_h: f32) -> Vec2 {
    let room = vec2(BUDDY_BOX.w, BUDDY_BOX.h - BUDDY_HOP);
    let k = (room.x / texture_w.max(1.0)).min(room.y / texture_h.max(1.0));
    vec2(texture_w * k, texture_h * k)
}

/// 기다려 주는 친구 그리기. `seconds`: 통통 뛰는 시간.
fn draw_buddy(texture: &Texture2D, seconds: f32) {
    let size = buddy_size(texture.width(), texture.height());
    let feet = vec2(BUDDY_BOX.center().x, BUDDY_BOX.bottom() - buddy_lift(seconds));
    draw_texture_ex(texture, feet.x - size.x / 2.0, feet.y - size.y, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
}

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

/// 멈춤 창에서 이번 프레임에 "새로 누른" 키 (꾹 누르고 있던 키는 false. 창이 열릴 때 누르던 ↓로 바로 옮겨지지 않는다).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PauseKeys {
    pub up: bool,
    pub down: bool,
    /// Enter 또는 Space
    pub confirm: bool,
    /// Esc
    pub back: bool,
}

/// 멈춤 창이 받은 입력 (부모님 요청 2026-11-15).
/// ↑/↓는 `picked`(고른 칸)를 옮기고 Enter·Space는 고른 칸을 누른다. Esc는 언제나 이어서 하기.
/// 마우스·손가락으로 누른 버튼은 고른 칸과 상관없이 그 버튼이다.
pub fn pause_choose(picked: &mut PauseTap, keys: PauseKeys, tap: Option<Vec2>) -> Option<PauseTap> {
    if keys.back {
        return Some(PauseTap::Resume);
    }
    if keys.up && !keys.down {
        *picked = PauseTap::Resume;
    } else if keys.down && !keys.up {
        *picked = PauseTap::Quit;
    }
    if keys.confirm {
        return Some(*picked);
    }
    tap.and_then(pause_tap_at)
}

/// 멈춤 창의 소리 버튼에서 누른 것.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PauseSoundTap {
    /// 소리 켜기/끄기
    Toggle,
    /// 음악 크기 칸 (`SOUND_VOLUME_STEPS` 번호)
    Music(usize),
    /// 효과음 크기 칸
    Effects(usize),
}

impl PauseSoundTap {
    /// 크기 칸을 누르면 소리도 같이 켠다 (꺼진 채로 눌러도 칸이 바뀌고 소리가 난다). 켜기/끄기 버튼은 그대로 뒤집기만 한다.
    pub fn turns_sound_on(self) -> bool {
        !matches!(self, PauseSoundTap::Toggle)
    }
}

/// 소리 크기 칸 하나의 네모. `row_y`는 줄의 위쪽, `i`는 0~3.
fn cell_rect(row_y: f32, i: usize) -> Rect {
    Rect::new(CELL_X + i as f32 * (CELL_W + CELL_GAP), row_y, CELL_W, SOUND_ROW_H)
}

/// 누른 곳(가상 좌표)이 소리 버튼 중 어디인지. 빈 곳은 None.
pub fn pause_sound_tap_at(point: Vec2) -> Option<PauseSoundTap> {
    if SOUND_TOGGLE.contains(point) {
        return Some(PauseSoundTap::Toggle);
    }
    for i in 0..SOUND_VOLUME_STEPS.len() {
        if cell_rect(MUSIC_ROW_Y, i).contains(point) {
            return Some(PauseSoundTap::Music(i));
        }
        if cell_rect(EFFECTS_ROW_Y, i).contains(point) {
            return Some(PauseSoundTap::Effects(i));
        }
    }
    None
}

/// 고른 칸은 주황 바탕 + 흰 테두리, 다른 칸은 파랑 바탕.
fn draw_choice(r: Rect, picked: bool) {
    if picked {
        draw_pill(r.x - PICK_RING, r.y - PICK_RING, r.w + PICK_RING * 2.0, r.h + PICK_RING * 2.0, PICK_RING_COLOR);
    }
    draw_pill(r.x, r.y, r.w, r.h, if picked { PICKED_FILL } else { OTHER_FILL });
}

/// 멈춤 창 그리기 (화면 좌표). `touch_mode`가 아니면 "이어서 하기" 아래에 "Esc"를 작게 쓴다.
/// `picked` 칸이 주황색이다 (처음은 이어서 하기라 예전 모습 그대로).
/// `buddy`: 창 옆에서 기다려 주는 친구 그림과 통통 뛰는 시간(초).
/// `sound`: (소리 켜짐, 음악 %, 효과음 %). 아래 소리 줄에 보인다.
pub fn draw_pause(text: &TextPainter, touch_mode: bool, picked: PauseTap, buddy: (&Texture2D, f32), sound: (bool, u8, u8)) {
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, DIM);
    draw_buddy(buddy.0, buddy.1);
    draw_rectangle(PANEL.x, PANEL.y, PANEL.w, PANEL.h, PANEL_FILL);
    draw_rectangle_lines(PANEL.x, PANEL.y, PANEL.w, PANEL.h, 5.0, PANEL_LINE);
    text.centered(PAUSE_TITLE_TEXT, PANEL.center().x, PANEL.y + 50.0, 40.0, TITLE);

    let r = RESUME_BUTTON;
    draw_choice(r, picked == PauseTap::Resume);
    // 이어서 하기: 오른쪽을 가리키는 세모 (놀이 버튼)
    let tri = vec2(r.x + 62.0, r.center().y);
    draw_triangle(tri + vec2(-16.0, -20.0), tri + vec2(-16.0, 20.0), tri + vec2(22.0, 0.0), WHITE);
    let (y, small) = if touch_mode { (r.center().y + 14.0, None) } else { (r.center().y + 4.0, Some("Esc")) };
    text.centered(RESUME_TEXT, r.center().x + 24.0, y, 42.0, WHITE);
    if let Some(key) = small {
        text.centered(key, r.center().x + 24.0, r.bottom() - 8.0, 20.0, WHITE);
    }

    let q = QUIT_BUTTON;
    draw_choice(q, picked == PauseTap::Quit);
    text.centered(QUIT_TEXT, q.center().x, q.center().y + 2.0, 38.0, WHITE);
    text.centered(QUIT_SUB_TEXT, q.center().x, q.bottom() - 10.0, 22.0, WHITE);

    // 소리 줄 (US-112): 켜짐/꺼짐 버튼, 음악 4칸, 효과음 4칸
    let (on, music, effects) = sound;
    let t = SOUND_TOGGLE;
    draw_pill(t.x, t.y, t.w, t.h, if on { Color::new(0.20, 0.62, 0.35, 1.0) } else { Color::new(0.45, 0.45, 0.50, 1.0) });
    let label = if on { PAUSE_SOUND_ON_TEXT } else { PAUSE_SOUND_OFF_TEXT };
    text.centered(label, t.center().x, t.center().y + 14.0, 40.0, WHITE);
    if !touch_mode {
        text.left("M", t.right() - 52.0, t.center().y + 9.0, 26.0, WHITE);
    }
    for (row_y, name, percent) in [(MUSIC_ROW_Y, PAUSE_MUSIC_LABEL, music), (EFFECTS_ROW_Y, PAUSE_EFFECTS_LABEL, effects)] {
        text.left(name, 292.0, row_y + SOUND_ROW_H / 2.0 + 9.0, 24.0, WHITE);
        let chosen = super::audio::volume_cell_for(percent);
        for i in 0..SOUND_VOLUME_STEPS.len() {
            let c = cell_rect(row_y, i);
            draw_rectangle(c.x, c.y, c.w, c.h, if i == chosen && on { PICKED_FILL } else { OTHER_FILL });
            text.centered(SOUND_STEP_LABELS[i], c.center().x, c.center().y + 9.0, 26.0, WHITE);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buttons_are_big_apart_and_inside_the_window() {
        for b in [RESUME_BUTTON, QUIT_BUTTON] {
            assert!(PANEL.contains(b.point()) && PANEL.contains(vec2(b.right(), b.bottom())), "{b:?} 창 밖");
            // 740×360 폰(0.667배)에서도 55 화면 픽셀 넘게 (소리 줄이 들어가며 110 → 86으로 낮췄다)
            assert!(b.h * 360.0 / 540.0 >= 55.0 && b.w >= 300.0);
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

    fn keys(up: bool, down: bool, confirm: bool, back: bool) -> PauseKeys {
        PauseKeys { up, down, confirm, back }
    }

    #[test]
    fn arrows_move_the_pick_and_enter_chooses_it() {
        // 처음은 이어서 하기 (main이 창을 열 때 Resume으로 둔다)
        let mut p = PauseTap::Resume;
        assert_eq!(pause_choose(&mut p, keys(false, false, true, false), None), Some(PauseTap::Resume), "바로 Enter면 이어서");
        assert_eq!(pause_choose(&mut p, keys(false, true, false, false), None), None, "↓는 옮기기만");
        assert_eq!(p, PauseTap::Quit);
        assert_eq!(pause_choose(&mut p, keys(false, true, false, false), None), None);
        assert_eq!(p, PauseTap::Quit, "맨 아래에서 ↓는 그대로 (돌아 올라가지 않는다)");
        assert_eq!(pause_choose(&mut p, keys(false, false, true, false), None), Some(PauseTap::Quit), "Enter·Space로 처음으로");
        assert_eq!(pause_choose(&mut p, keys(true, false, false, false), None), None);
        assert_eq!(p, PauseTap::Resume, "↑로 다시 이어서 하기");
        assert_eq!(pause_choose(&mut p, keys(true, false, false, false), None), None);
        assert_eq!(p, PauseTap::Resume, "맨 위에서 ↑는 그대로");
        // 같은 프레임에 ↓와 Enter: 옮긴 칸을 고른다
        let mut p = PauseTap::Resume;
        assert_eq!(pause_choose(&mut p, keys(false, true, true, false), None), Some(PauseTap::Quit));
        // ↑↓를 함께 누르면 움직이지 않는다
        let mut p = PauseTap::Quit;
        pause_choose(&mut p, keys(true, true, false, false), None);
        assert_eq!(p, PauseTap::Quit);
    }

    #[test]
    fn esc_always_resumes_and_taps_still_work() {
        let mut p = PauseTap::Quit;
        assert_eq!(pause_choose(&mut p, keys(false, false, false, true), None), Some(PauseTap::Resume), "처음으로를 골라 둬도 Esc는 이어서");
        let mut p = PauseTap::Resume;
        assert_eq!(pause_choose(&mut p, PauseKeys::default(), Some(QUIT_BUTTON.center())), Some(PauseTap::Quit), "클릭·터치는 고른 칸과 상관없이");
        let mut p = PauseTap::Quit;
        assert_eq!(pause_choose(&mut p, PauseKeys::default(), Some(RESUME_BUTTON.center())), Some(PauseTap::Resume));
        assert_eq!(pause_choose(&mut p, PauseKeys::default(), Some(vec2(5.0, 5.0))), None, "빈 곳은 그대로 멈춰 있다");
        assert_eq!(pause_choose(&mut p, PauseKeys::default(), None), None, "아무것도 안 누르면 그대로");
        assert_eq!(p, PauseTap::Quit, "누르지 않으면 고른 칸도 그대로");
    }

    #[test]
    fn picked_button_is_orange_and_ring_stays_inside() {
        assert!(PICKED_FILL.r > 0.9 && PICKED_FILL.g > 0.5 && PICKED_FILL.g < 0.8 && PICKED_FILL.b < 0.3, "주황");
        for b in [RESUME_BUTTON, QUIT_BUTTON] {
            assert!(b.y - PICK_RING > PANEL.y && b.bottom() + PICK_RING < PANEL.bottom(), "테두리가 창 안");
        }
        assert!(RESUME_BUTTON.bottom() + PICK_RING < QUIT_BUTTON.y, "테두리가 다른 버튼에 안 닿는다");
    }

    #[test]
    fn buddy_is_one_of_four_and_all_four_come_up() {
        let mut seen = [false; 4];
        for roll in 0..400u32 {
            let b = PauseBuddy::pick(roll);
            let i = PauseBuddy::ALL.iter().position(|x| *x == b).expect("넷 중 하나");
            seen[i] = true;
        }
        assert_eq!(seen, [true; 4], "Niro, Ellie, Logan, 미스 모두 나온다");
        assert!(PauseBuddy::ALL.contains(&PauseBuddy::Miss));
    }

    #[test]
    fn buddy_stands_beside_the_window_on_screen_and_hops_only_a_little() {
        assert!(BUDDY_BOX.x > PANEL.right(), "창과 버튼에 안 겹친다");
        assert!(BUDDY_BOX.right() <= VIRTUAL_WIDTH && BUDDY_BOX.bottom() <= VIRTUAL_HEIGHT && BUDDY_BOX.y >= 0.0, "화면 안 (740×360도 같은 가상 화면)");
        // 740×360 폰(0.667배)에서도 잘 보이는 크기
        assert!(BUDDY_BOX.h * 360.0 / 540.0 >= 120.0);
        for (w, h) in [(100.0, 300.0), (300.0, 100.0), (200.0, 200.0), (0.0, 0.0)] {
            let s = buddy_size(w, h);
            assert!(s.x <= BUDDY_BOX.w + 0.01 && s.y + BUDDY_HOP <= BUDDY_BOX.h + 0.01, "{w}×{h} → {s:?} 칸 안");
        }
        for i in 0..300 {
            let lift = buddy_lift(i as f32 * 0.01);
            assert!((0.0..=BUDDY_HOP + 0.01).contains(&lift), "살짝만 통통: {lift}");
        }
        assert!(buddy_lift(0.0) < 0.01, "땅에서 시작");
        const { assert!(BUDDY_HOP <= 12.0) };
    }

    #[test]
    fn sound_buttons_fit_in_the_window_do_not_overlap_and_are_touchable() {
        let mut all: Vec<Rect> = vec![SOUND_TOGGLE];
        for i in 0..SOUND_VOLUME_STEPS.len() {
            all.push(cell_rect(MUSIC_ROW_Y, i));
            all.push(cell_rect(EFFECTS_ROW_Y, i));
        }
        all.push(RESUME_BUTTON);
        all.push(QUIT_BUTTON);
        for (i, a) in all.iter().enumerate() {
            assert!(PANEL.contains(a.point()) && PANEL.contains(vec2(a.right(), a.bottom())), "{a:?} 창 밖");
            // 740×360 폰에서 44 화면 픽셀 넘게 (손가락 크기)
            assert!(a.h * 360.0 / 540.0 >= 44.0 && a.w * 740.0 / 960.0 >= 44.0, "{a:?} 너무 작다");
            for b in all.iter().skip(i + 1) {
                assert!(a.intersect(*b).is_none(), "{a:?}와 {b:?}가 겹친다");
            }
        }
        // 이름 글자 자리(왼쪽)와 첫 칸도 안 겹친다
        assert!(cell_rect(MUSIC_ROW_Y, 0).x > 292.0 + 3.0 * 24.0 + 8.0, "효과음 세 글자가 첫 칸 앞에 든다");
        // 고르기 테두리(7)가 소리 줄에 닿지 않는다
        assert!(QUIT_BUTTON.bottom() + PICK_RING < SOUND_TOGGLE.y);
        assert!(PANEL.right() <= VIRTUAL_WIDTH && PANEL.bottom() <= VIRTUAL_HEIGHT);
        // 친구 자리와 안 겹친다
        assert!(BUDDY_BOX.x > PANEL.right());
    }

    #[test]
    fn volume_cells_turn_the_sound_on_but_the_toggle_does_not() {
        assert!(PauseSoundTap::Music(2).turns_sound_on());
        assert!(PauseSoundTap::Effects(0).turns_sound_on());
        assert!(!PauseSoundTap::Toggle.turns_sound_on());
    }

    #[test]
    fn sound_taps_are_found_and_do_not_steal_the_big_buttons() {
        assert_eq!(pause_sound_tap_at(SOUND_TOGGLE.center()), Some(PauseSoundTap::Toggle));
        for i in 0..4 {
            assert_eq!(pause_sound_tap_at(cell_rect(MUSIC_ROW_Y, i).center()), Some(PauseSoundTap::Music(i)));
            assert_eq!(pause_sound_tap_at(cell_rect(EFFECTS_ROW_Y, i).center()), Some(PauseSoundTap::Effects(i)));
        }
        assert_eq!(pause_sound_tap_at(RESUME_BUTTON.center()), None);
        assert_eq!(pause_sound_tap_at(QUIT_BUTTON.center()), None);
        assert_eq!(pause_tap_at(SOUND_TOGGLE.center()), None, "소리 버튼은 이어서/처음으로가 아니다");
        assert_eq!(pause_sound_tap_at(vec2(5.0, 5.0)), None);
    }

    #[test]
    fn words_are_easy_korean() {
        assert_eq!((RESUME_TEXT, QUIT_TEXT, QUIT_SUB_TEXT), ("이어서 하기", "처음으로", "(캐릭터 고르기)"));
    }
}
