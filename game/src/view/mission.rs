//! 깜짝 미션 화면 (US-086, D81, D103, [임시-55]). 규칙은 `logic::mission`에 있고 여기서는 보여 주기만 한다.
//!
//! - 미션이 나오면 미스 얼굴과 말풍선이 `MISSION_MISS_SAYS_SECONDS`초.
//! - 성공하면 성공 글 `MISSION_SUCCESS_SHOW_SECONDS`초와 반짝이, 실패하면 실패 글 `MISSION_FAIL_SHOW_SECONDS`초.
//! - 그 밖에는 하는 중인 미션의 진행 글이 한 줄 늘 보인다.
//!
//! 자리: 위쪽 가운데. 정보판 왼쪽 위 판과 오른쪽 위 판 사이에 들어가면 거기, 좁으면 글자를 줄이지 않고
//! 정보판 아래(물건 칸 누르는 자리 아래) 한 줄로 내려간다. 폰 멈춤 버튼과 가방 버튼 사이에만 놓는다.
//! 말풍선·성공·실패 글은 한 번에 하나씩 차례로 보인다 (06:00에 실패 글 뒤에 새 미션 말풍선).
//! 게임은 멈추지 않는다. 창(퀴즈·상점·제작대)이 열린 동안은 효과처럼 시간이 흐르지 않는다.

use super::bag::bag_button_rect;
use super::hud::{draw_pill, PANEL, POCKET_TAP_H, ROW_MID_Y};
use super::screen::VIRTUAL_WIDTH;
use super::sprites::fit_size;
use super::text::{TextPainter, TEXT_ABOVE, TEXT_BELOW};
use super::touch::{draw_star, BACK_BUTTON};
use crate::config::{MISSION_FAIL_SHOW_SECONDS, MISSION_MISS_SAYS_SECONDS, MISSION_SUCCESS_SHOW_SECONDS};
use crate::logic::mission::{mission_fail_text, mission_success_text, Mission, MissionEvent};
use macroquad::prelude::*;
use std::collections::VecDeque;

/// 진행 글 크기 (740×360에서 16 화면 픽셀).
pub const PROGRESS_SIZE: f32 = 24.0;
/// 미스 말풍선 글 크기.
pub const SAYS_SIZE: f32 = 24.0;
/// 성공·실패 글 크기.
pub const RESULT_SIZE: f32 = 28.0;
/// 판 안쪽 가로 여백.
const PAD: f32 = 14.0;
/// 정보판 사이 띠: 위 끝과 높이 (물건 칸 누르는 자리 위 끝보다 위).
const TOP_BAND_Y: f32 = 8.0;
const TOP_BAND_H: f32 = 32.0;
/// 정보판 사이에 넣을 때 양쪽 판과의 틈.
const SIDE_GAP: f32 = 10.0;
/// 정보판 아래 한 줄: 물건 칸 누르는 자리 아래 끝에서 이만큼 아래.
const BELOW_GAP: f32 = 6.0;
pub const BELOW_TOP: f32 = ROW_MID_Y + POCKET_TAP_H / 2.0 + BELOW_GAP;
const LINE_H: f32 = 36.0;
const RESULT_H: f32 = 42.0;
/// 말풍선: 미스 얼굴 동그라미 반지름, 얼굴과 풍선 틈, 줄 간격(글자 크기에 대한 비율), 위아래 여백, 모서리
const FACE_R: f32 = 28.0;
const FACE_GAP: f32 = 10.0;
const SAYS_LINE_STEP: f32 = 1.2;
const SAYS_PAD_Y: f32 = 10.0;
const CORNER: f32 = 14.0;
/// 폰 멈춤 버튼(왼쪽)·가방 버튼(오른쪽)과의 틈
const BUTTON_GAP: f32 = 8.0;

const FACE_BACK: Color = Color::new(0.68, 0.85, 1.0, 1.0);
const FACE_RING: Color = Color::new(1.0, 0.62, 0.80, 1.0);
/// 말풍선: 게임의 다른 말풍선처럼 진한 남색 판에 하얀 글자
const BUBBLE: Color = Color::new(0.13, 0.16, 0.36, 0.85);
const SUCCESS_COLOR: Color = Color::new(1.0, 0.85, 0.25, 1.0);
const FAIL_COLOR: Color = Color::new(1.0, 1.0, 1.0, 1.0);
const SPARKLE: Color = Color::new(1.0, 0.96, 0.60, 1.0);
/// 성공할 때 오른쪽 위 코인 판 둘레에도 반짝 (코인이 100 늘었다)
const COIN_SPARKLE_AT: Vec2 = Vec2::new(800.0, 28.0);

/// 미션 글이 놓일 수 있는 가로 범위: 폰 멈춤 버튼 오른쪽 ~ 가방 버튼 왼쪽. 화면 가운데(480)를 가운데로 둔다.
pub fn center_x() -> f32 {
    VIRTUAL_WIDTH / 2.0
}

pub fn max_width() -> f32 {
    let left = BACK_BUTTON.right() + BUTTON_GAP;
    let right = bag_button_rect(true).x.min(bag_button_rect(false).x) - BUTTON_GAP;
    2.0 * (center_x() - left).min(right - center_x())
}

/// 잠깐 보이는 알림 하나.
#[derive(Debug, Clone, PartialEq)]
pub enum Banner {
    /// 미스 얼굴과 말풍선 (미스가 알려 주는 말).
    MissSays(String),
    Success,
    Fail,
}

impl Banner {
    pub fn seconds(&self) -> f32 {
        match self {
            Banner::MissSays(_) => MISSION_MISS_SAYS_SECONDS,
            Banner::Success => MISSION_SUCCESS_SHOW_SECONDS,
            Banner::Fail => MISSION_FAIL_SHOW_SECONDS,
        }
    }
}

/// 미션 알림의 차례와 시간 (화면 전용).
#[derive(Default)]
pub struct MissionView {
    now: Option<(Banner, f32)>,
    waiting: VecDeque<Banner>,
}

impl MissionView {
    /// logic이 준 미션 일을 받는다. `cleared`(클리어한 아침)면 실패 글은 띄우지 않는다 (감독 결정).
    pub fn on_event(&mut self, event: MissionEvent, cleared: bool) {
        let banner = match event {
            MissionEvent::Appeared { kind, goal } => Banner::MissSays(kind.miss_says_text(goal)),
            MissionEvent::Succeeded { .. } => Banner::Success,
            MissionEvent::Failed { .. } if cleared => return,
            MissionEvent::Failed { .. } => Banner::Fail,
        };
        if self.now.is_none() {
            self.now = Some((banner, 0.0));
        } else {
            self.waiting.push_back(banner);
        }
    }

    /// 진짜 시간 `dt`만큼 흐른다. 끝난 알림은 지우고 다음 알림을 꺼낸다.
    pub fn update(&mut self, dt: f32) {
        if let Some((banner, age)) = self.now.as_mut() {
            *age += dt;
            if *age >= banner.seconds() {
                self.now = self.waiting.pop_front().map(|b| (b, 0.0));
            }
        }
    }

    /// 지금 보이는 알림과 지난 시간.
    pub fn banner(&self) -> Option<(&Banner, f32)> {
        self.now.as_ref().map(|(b, age)| (b, *age))
    }
}

/// 이번 프레임에 그릴 것과 자리 (화면 좌표).
#[derive(Debug, Clone, PartialEq)]
pub enum MissionLook {
    Progress { rect: Rect, text: String },
    MissSays { rect: Rect, lines: Vec<String> },
    Result { rect: Rect, success: bool, age: f32 },
}

impl MissionLook {
    pub fn rect(&self) -> Rect {
        match self {
            MissionLook::Progress { rect, .. } | MissionLook::MissSays { rect, .. } | MissionLook::Result { rect, .. } => *rect,
        }
    }
}

/// 진행 글 판의 자리. `gap_left`~`gap_right`(정보판 왼쪽 판 오른쪽 끝 ~ 오른쪽 위 판 왼쪽 끝) 사이에 들어가면 거기,
/// 아니면 글자를 줄이지 않고 정보판 아래 한 줄로.
pub fn progress_rect(text_w: f32, gap_left: f32, gap_right: f32) -> Rect {
    let w = text_w + PAD * 2.0;
    let (left, right) = (gap_left + SIDE_GAP, gap_right - SIDE_GAP);
    if right - left >= w {
        Rect::new((left + right - w) / 2.0, TOP_BAND_Y, w, TOP_BAND_H)
    } else {
        Rect::new(center_x() - w / 2.0, BELOW_TOP, w, LINE_H)
    }
}

/// 말풍선 글이 들어갈 가장 큰 폭.
pub fn says_max_text_width() -> f32 {
    max_width() - FACE_R * 2.0 - FACE_GAP - PAD * 2.0
}

/// 말풍선 글을 한 줄, 넘치면 두 줄로 나눈다 (가운데에 가까운 띄어쓰기에서). `width`는 글 폭을 재는 함수.
pub fn split_lines(words: &str, max_w: f32, width: impl Fn(&str) -> f32) -> Vec<String> {
    if width(words) <= max_w {
        return vec![words.to_string()];
    }
    let best = words
        .char_indices()
        .filter(|(_, c)| *c == ' ')
        .map(|(i, _)| (words[..i].to_string(), words[i + 1..].to_string()))
        .min_by(|a, b| width(&a.0).max(width(&a.1)).total_cmp(&width(&b.0).max(width(&b.1))));
    match best {
        Some((first, second)) => vec![first, second],
        None => vec![words.to_string()],
    }
}

/// 말풍선(얼굴 포함) 자리. `text_w`는 가장 긴 줄 폭.
pub fn says_rect(text_w: f32, lines: usize) -> Rect {
    let w = FACE_R * 2.0 + FACE_GAP + text_w + PAD * 2.0;
    let text_h = SAYS_SIZE * (TEXT_ABOVE + TEXT_BELOW) + lines.saturating_sub(1) as f32 * SAYS_SIZE * SAYS_LINE_STEP;
    let h = (text_h + SAYS_PAD_Y * 2.0).max(FACE_R * 2.0);
    Rect::new(center_x() - w / 2.0, BELOW_TOP, w, h)
}

pub fn result_rect(text_w: f32) -> Rect {
    let w = text_w + PAD * 2.0;
    Rect::new(center_x() - w / 2.0, BELOW_TOP, w, RESULT_H)
}

/// 지금 그릴 미션 화면. 알림이 있으면 알림, 없으면 하는 중인 미션의 진행 글, 둘 다 없으면 None.
/// `gap`: 정보판 왼쪽 판 오른쪽 끝과 오른쪽 위 첫 줄 판 왼쪽 끝 (hud가 잰다).
pub fn look(view: &MissionView, mission: Option<&Mission>, gap: (f32, f32), width: impl Fn(&str, f32) -> f32) -> Option<MissionLook> {
    if let Some((banner, age)) = view.banner() {
        return Some(match banner {
            Banner::MissSays(words) => {
                let lines = split_lines(words, says_max_text_width(), |s| width(s, SAYS_SIZE));
                let text_w = lines.iter().map(|l| width(l, SAYS_SIZE)).fold(0.0, f32::max);
                MissionLook::MissSays { rect: says_rect(text_w, lines.len()), lines }
            }
            Banner::Success => MissionLook::Result { rect: result_rect(width(&mission_success_text(), RESULT_SIZE)), success: true, age },
            Banner::Fail => MissionLook::Result { rect: result_rect(width(mission_fail_text(), RESULT_SIZE)), success: false, age },
        });
    }
    let mission = mission?;
    let text = mission.progress_text();
    Some(MissionLook::Progress { rect: progress_rect(width(&text, PROGRESS_SIZE), gap.0, gap.1), text })
}

/// 둥근 네모 판 (말풍선).
fn draw_round_rect(r: Rect, corner: f32, color: Color) {
    let c = corner.min(r.w / 2.0).min(r.h / 2.0);
    draw_rectangle(r.x + c, r.y, r.w - c * 2.0, r.h, color);
    draw_rectangle(r.x, r.y + c, c, r.h - c * 2.0, color);
    draw_rectangle(r.right() - c, r.y + c, c, r.h - c * 2.0, color);
    for (x, y) in [(r.x + c, r.y + c), (r.right() - c, r.y + c), (r.x + c, r.bottom() - c), (r.right() - c, r.bottom() - c)] {
        draw_circle(x, y, c, color);
    }
}

/// 나타날 때·사라질 때 부드럽게 (0~1).
fn fade(age: f32, seconds: f32) -> f32 {
    (age / 0.2).min((seconds - age) / 0.3).clamp(0.0, 1.0)
}

pub fn draw(text: &TextPainter, look: &MissionLook, miss: &Texture2D, banner_age: f32) {
    let t = get_time() as f32;
    match look {
        MissionLook::Progress { rect, text: words } => {
            draw_pill(rect.x, rect.y, rect.w, rect.h, PANEL);
            text.centered(words, rect.center().x, rect.y + rect.h / 2.0 + PROGRESS_SIZE * 0.36, PROGRESS_SIZE, WHITE);
        }
        MissionLook::MissSays { rect, lines } => {
            let a = fade(banner_age, MISSION_MISS_SAYS_SECONDS);
            // 미스 얼굴 (지금 퀴즈 창의 미스 그림)
            let face = vec2(rect.x + FACE_R, rect.y + FACE_R);
            draw_circle(face.x, face.y, FACE_R, Color { a, ..FACE_BACK });
            let size = fit_size(miss, FACE_R * 1.7);
            let bob = (t * 4.0).sin() * 1.5;
            draw_texture_ex(miss, face.x - size.x / 2.0, face.y - size.y / 2.0 + bob, Color { a, ..WHITE }, DrawTextureParams { dest_size: Some(size), ..Default::default() });
            draw_circle_lines(face.x, face.y, FACE_R, 3.0, Color { a, ..FACE_RING });
            // 말풍선과 꼬리 (얼굴 쪽)
            let bubble = Rect::new(rect.x + FACE_R * 2.0 + FACE_GAP, rect.y, rect.w - FACE_R * 2.0 - FACE_GAP, rect.h);
            let color = Color { a: BUBBLE.a * a, ..BUBBLE };
            draw_round_rect(bubble, CORNER, color);
            draw_triangle(vec2(bubble.x, face.y - 8.0), vec2(bubble.x, face.y + 8.0), vec2(bubble.x - FACE_GAP, face.y), color);
            let text_h = SAYS_SIZE * (TEXT_ABOVE + TEXT_BELOW) + (lines.len().saturating_sub(1)) as f32 * SAYS_SIZE * SAYS_LINE_STEP;
            let mut baseline = bubble.center().y - text_h / 2.0 + SAYS_SIZE * TEXT_ABOVE;
            for line in lines {
                text.centered(line, bubble.center().x, baseline, SAYS_SIZE, Color { a, ..WHITE });
                baseline += SAYS_SIZE * SAYS_LINE_STEP;
            }
        }
        MissionLook::Result { rect, success, age } => {
            let seconds = if *success { MISSION_SUCCESS_SHOW_SECONDS } else { MISSION_FAIL_SHOW_SECONDS };
            let a = fade(*age, seconds);
            draw_pill(rect.x, rect.y, rect.w, rect.h, Color { a: PANEL.a * a, ..PANEL });
            let (words, color) = if *success { (mission_success_text(), SUCCESS_COLOR) } else { (mission_fail_text().to_string(), FAIL_COLOR) };
            text.centered(&words, rect.center().x, rect.y + rect.h / 2.0 + RESULT_SIZE * 0.36, RESULT_SIZE, Color { a, ..color });
            if *success {
                // 반짝이: 글 판 둘레와 코인 판 둘레
                for i in 0..8 {
                    let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                    let x = rect.center().x + side * (rect.w / 2.0 + 8.0 + (i / 2) as f32 * 14.0);
                    let y = rect.center().y + ((t * 3.0 + i as f32).sin()) * rect.h * 0.4;
                    let twinkle = 0.5 + 0.5 * (t * 6.0 + i as f32 * 1.3).sin();
                    draw_star(vec2(x, y), 6.0 + 4.0 * twinkle, t * 2.0 + i as f32, Color { a: a * (0.4 + 0.6 * twinkle), ..SPARKLE });
                }
                for i in 0..4 {
                    let p = COIN_SPARKLE_AT + vec2((i as f32 - 1.5) * 26.0, ((t * 5.0 + i as f32).sin()) * 12.0);
                    let twinkle = 0.5 + 0.5 * (t * 7.0 + i as f32).sin();
                    draw_star(p, 5.0 + 4.0 * twinkle, t * 3.0 + i as f32, Color { a: a * twinkle, ..SPARKLE });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GROUND_Y, PLAYER_HEIGHT};
    use crate::logic::character::CharacterKind;
    use crate::logic::game::Game;
    use crate::logic::mission::{MissionKind, MissionState};
    use crate::view::hud::{HEALTH_BAR, HELD_SLOT, XP_BAR};
    use crate::view::touch::{button_touches_rect, play_buttons_with_mine};

    const FRAME: f32 = 1.0 / 60.0;

    /// 글 폭 어림 (시험에는 글꼴이 없다): 한글은 글자 크기만큼, 띄어쓰기는 0.3, 영문·숫자·기호는 0.6만큼. 실제 Jua보다 넉넉하다.
    fn est(s: &str, size: f32) -> f32 {
        s.chars().map(|c| if c == ' ' { 0.3 } else if c.is_ascii() { 0.6 } else { 1.0 }).sum::<f32>() * size
    }

    fn all_kinds() -> [MissionKind; 6] {
        [MissionKind::Coins, MissionKind::Trees, MissionKind::Ores, MissionKind::FindMiss, MissionKind::Monsters, MissionKind::KeepHealth]
    }

    fn mission(kind: MissionKind, count: u32, goal: u32) -> Mission {
        Mission { kind, goal, count, state: MissionState::Going, day: 7 }
    }

    /// 정보판 사이 띠 (보통 게임: 레벨 1, 1일째, 코인 몇 개). 실제로는 hud가 잰다.
    const USUAL_GAP: (f32, f32) = (395.0, 660.0);

    /// 지금 쓰는 모든 미션 글의 자리 (가장 큰 수로)
    fn every_look() -> Vec<MissionLook> {
        let mut list = Vec::new();
        for kind in all_kinds() {
            // 가장 큰 수. 체력 지키기의 지금 체력은 레벨이 올라 두 자리일 수 있다
            let goal = kind.goal_on_day(99);
            let now = if kind == MissionKind::KeepHealth { 99 } else { goal };
            let m = mission(kind, now, goal);
            list.push(look(&MissionView::default(), Some(&m), USUAL_GAP, est).unwrap());
            for event in [MissionEvent::Appeared { kind, goal: 20 }, MissionEvent::Succeeded { kind }, MissionEvent::Failed { kind }] {
                let mut view = MissionView::default();
                view.on_event(event, false);
                list.push(look(&view, Some(&m), USUAL_GAP, est).unwrap());
            }
        }
        list
    }

    #[test]
    fn miss_says_for_three_seconds_then_progress_shows() {
        let mut view = MissionView::default();
        let m = mission(MissionKind::Coins, 3, 12);
        view.on_event(MissionEvent::Appeared { kind: MissionKind::Coins, goal: 12 }, false);
        assert_eq!(view.banner().map(|(b, _)| b.clone()), Some(Banner::MissSays("깜짝 미션이야! 밤이 되기 전에 코인 12개를 모아 줘!".into())));
        assert!(matches!(look(&view, Some(&m), USUAL_GAP, est), Some(MissionLook::MissSays { .. })));
        for _ in 0..(MISSION_MISS_SAYS_SECONDS / FRAME) as usize - 2 {
            view.update(FRAME);
        }
        assert!(view.banner().is_some(), "3초 동안 보인다");
        for _ in 0..4 {
            view.update(FRAME);
        }
        assert!(view.banner().is_none());
        match look(&view, Some(&m), USUAL_GAP, est) {
            Some(MissionLook::Progress { text, .. }) => assert_eq!(text, "깜짝 미션! 코인 3/12 (밤 전까지)"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn success_three_seconds_fail_two_seconds() {
        for (event, seconds) in [(MissionEvent::Succeeded { kind: MissionKind::Trees }, 3.0), (MissionEvent::Failed { kind: MissionKind::Trees }, 2.0)] {
            let mut view = MissionView::default();
            view.on_event(event, false);
            view.update(seconds - 0.05);
            assert!(view.banner().is_some(), "{event:?}");
            view.update(0.1);
            assert!(view.banner().is_none(), "{event:?}");
            // 끝난 미션은 진행 글이 없다 (logic의 mission_now가 None)
            assert_eq!(look(&view, None, USUAL_GAP, est), None);
        }
        assert_eq!(MISSION_SUCCESS_SHOW_SECONDS, 3.0);
        assert_eq!(MISSION_FAIL_SHOW_SECONDS, 2.0);
        assert_eq!(MISSION_MISS_SAYS_SECONDS, 3.0);
    }

    #[test]
    fn fail_then_new_mission_come_one_after_another() {
        // 06:00: 밤 미션 실패 글 2초 뒤에 낮 미션 말풍선 3초
        let mut view = MissionView::default();
        view.on_event(MissionEvent::Failed { kind: MissionKind::Monsters }, false);
        view.on_event(MissionEvent::Appeared { kind: MissionKind::Trees, goal: 4 }, false);
        assert_eq!(view.banner().map(|(b, _)| b.clone()), Some(Banner::Fail));
        view.update(2.0);
        assert!(matches!(view.banner(), Some((Banner::MissSays(_), age)) if age == 0.0));
        view.update(3.0);
        assert!(view.banner().is_none());
    }

    #[test]
    fn no_fail_text_on_the_clear_morning() {
        let mut view = MissionView::default();
        view.on_event(MissionEvent::Failed { kind: MissionKind::KeepHealth }, true);
        assert!(view.banner().is_none(), "클리어 화면 위에 실패 글을 띄우지 않는다");
    }

    #[test]
    fn new_game_shows_the_day_one_mission_from_miss() {
        let mut g = Game::new(CharacterKind::Niro, 7);
        let mut view = MissionView::default();
        for e in g.take_mission_events() {
            view.on_event(e, g.is_cleared());
        }
        let m = g.mission_now().expect("1일째 낮 미션");
        match view.banner() {
            Some((Banner::MissSays(words), _)) => assert_eq!(*words, m.kind.miss_says_text(m.goal)),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn keep_health_shows_hp_now() {
        let m = mission(MissionKind::KeepHealth, 6, 4);
        match look(&MissionView::default(), Some(&m), USUAL_GAP, est) {
            Some(MissionLook::Progress { text, .. }) => assert_eq!(text, "깜짝 미션! 체력 4 이상 지키기, 지금 6 (아침까지)"),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn progress_goes_between_panels_only_if_it_fits() {
        let wide = progress_rect(200.0, 300.0, 700.0);
        assert_eq!(wide.y, TOP_BAND_Y, "넓으면 정보판 사이 위쪽 줄");
        assert!(wide.x >= 310.0 && wide.right() <= 690.0);
        let narrow = progress_rect(400.0, 395.0, 660.0);
        assert_eq!((narrow.y, narrow.w), (BELOW_TOP, 400.0 + PAD * 2.0), "좁으면 글자를 줄이지 않고 아래 줄");
        assert_eq!(narrow.center().x, center_x());
        // 정보판 사이 띠는 물건 칸 누르는 자리보다 위
        const { assert!(TOP_BAND_Y + TOP_BAND_H < ROW_MID_Y - POCKET_TAP_H / 2.0) };
    }

    #[test]
    fn long_miss_words_split_into_two_lines_that_fit() {
        for kind in all_kinds() {
            let words = kind.miss_says_text(20);
            let lines = split_lines(&words, says_max_text_width(), |s| est(s, SAYS_SIZE));
            assert!(lines.len() <= 2, "{words}");
            assert_eq!(lines.join(" "), words, "글자가 빠지지 않는다");
            for l in &lines {
                assert!(est(l, SAYS_SIZE) <= says_max_text_width(), "{l}");
            }
        }
        assert_eq!(split_lines("짧은 말", 500.0, |s| est(s, 24.0)), vec!["짧은 말".to_string()]);
    }

    #[test]
    fn mission_text_stays_clear_of_hud_buttons_and_tags() {
        // 다 가진 게임의 물건 칸 누르는 자리
        let mut full = Game::new(CharacterKind::Niro, 7);
        full.gathering.bag.syringe_guns = 1;
        full.gathering.bag.ender_fruits = 1;
        full.gathering.bag.riding_car = true;
        full.gathering.bag.has_ender_power = true;
        let pockets: Vec<Rect> = crate::view::hud::pocket_tap_rects(&full).into_iter().map(|(_, r)| r).collect();
        let head = GROUND_Y - PLAYER_HEIGHT;
        for l in every_look() {
            let r = l.rect();
            assert!(r.x >= 0.0 && r.right() <= VIRTUAL_WIDTH && r.y >= 0.0, "화면 안: {l:?}");
            assert!(r.w <= max_width() + 0.1, "멈춤·가방 버튼 사이 폭: {l:?}");
            assert!(!r.overlaps(&BACK_BUTTON), "폰 멈춤 버튼: {l:?}");
            for touch in [false, true] {
                assert!(!r.overlaps(&bag_button_rect(touch)), "가방 버튼: {l:?}");
            }
            assert!(!r.overlaps(&HEALTH_BAR) && !r.overlaps(&XP_BAR), "체력·경험치 막대: {l:?}");
            assert!(!r.overlaps(&HELD_SLOT), "무기 칸: {l:?}");
            for p in &pockets {
                assert!(!r.overlaps(p), "쓰는 것 칸 누르는 자리: {l:?}");
            }
            for b in play_buttons_with_mine(true, true) {
                assert!(!button_touches_rect(&b, r), "{:?} 버튼: {l:?}", b.kind);
            }
            assert!(r.bottom() < head - 60.0, "서 있는 캐릭터와 머리 위 E 표시보다 한참 위: {l:?}");
        }
    }

    #[test]
    fn usable_tags_stay_below_the_mission_text() {
        // 맵을 걸어 다니며 머리 위 E 표시(집 문·나무·동굴 입구 등)가 미션 글 아래에 있는지
        let lowest = every_look().iter().map(|l| l.rect().bottom()).fold(0.0, f32::max);
        let mut g = Game::new(CharacterKind::Niro, 7);
        let mut seen = 0;
        let width = g.world.width;
        let steps = (width / 20.0) as usize;
        for i in 0..steps.min(2000) {
            g.player.pos = vec2(i as f32 * 20.0, GROUND_Y - PLAYER_HEIGHT);
            g.update(&Default::default(), 0.0);
            let camera = g.camera_left();
            if let Some(tag) = crate::view::gather::usable_tag_screen_rect(&g, camera + VIRTUAL_WIDTH / 2.0, camera) {
                seen += 1;
                assert!(tag.y > lowest, "E 표시 {tag:?}, 미션 글 아래 끝 {lowest}");
            }
        }
        assert!(seen > 0, "E 표시를 하나는 봐야 의미가 있다");
    }

    #[test]
    fn health_bar_shows_the_lost_hp_shrinking() {
        use crate::logic::combat::CombatEvent;
        let mut effects = crate::view::effects::Effects::new();
        assert_eq!(effects.lost_hp(), 0.0);
        effects.on_combat(CombatEvent::PlayerHurt { amount: 1, hp_left: 5 });
        assert_eq!(effects.lost_hp(), 1.0, "막 줄면 1칸이 옅게 남는다");
        effects.update(0.5);
        assert!(effects.lost_hp() > 0.0 && effects.lost_hp() < 1.0, "점점 줄어든다");
        effects.update(0.6);
        assert_eq!(effects.lost_hp(), 0.0);
    }

    #[test]
    fn text_is_readable_on_a_small_phone() {
        let small = (740.0 / VIRTUAL_WIDTH).min(360.0 / 540.0);
        for size in [PROGRESS_SIZE, SAYS_SIZE, RESULT_SIZE] {
            assert!(size * small >= 14.0, "{size}");
        }
    }

    #[test]
    fn mission_words_use_only_letters_jua_has() {
        // Jua에 없는 글자(가운데 점, 긴 줄표, 화살표)를 쓰지 않는다. "잡기", "죽는다"도 없다.
        let mut all: Vec<String> = vec![mission_success_text(), mission_fail_text().to_string()];
        for kind in all_kinds() {
            all.push(kind.miss_says_text(5));
            all.push(kind.progress_text(1, 5));
        }
        for s in all {
            assert!(!s.contains(['·', '—', '↑', '…']), "{s}");
            assert!(!s.contains("잡기") && !s.contains("죽는"), "{s}");
        }
    }
}
