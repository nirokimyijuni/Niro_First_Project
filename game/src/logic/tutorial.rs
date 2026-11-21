//! 튜토리얼 말풍선 규칙 (US-119·120). 어느 단계인지, 직접 해냈는지, 건너뛰었는지만 안다.
//! 그림·글자 그리기·저장 쓰기는 화면(로건)이 한다. 시간은 `dt`(실제 초)로 받는다.
//!
//! 단계: ① 이동 → ② 공격(연습용 몬스터) → ③ 캐기 → ④ 점프 → ⑤ 미스 찾기.
//! 직접 해야 넘어간다 (시간이 지났다고 저절로 넘어가지 않는다). 예외 둘:
//! - ②는 연습용 몬스터가 없어지면(시간이 다 됨, 밤이 옴) 안 때려도 ③으로 넘어간다 (막히지 않게).
//! - ⑤는 말만 하고, `TUTORIAL_FIND_MISS_SECONDS`초 뒤 또는 미스를 찾으면 끝난다.
//! 건너뛰기는 한 번에 모두 끝낸다.

use super::settings::Settings;
use crate::config::{TUTORIAL_CHEER_SECONDS, TUTORIAL_FIND_MISS_SECONDS, TUTORIAL_TEXT_PC, TUTORIAL_TEXT_TOUCH, TUTORIAL_WALK_DISTANCE};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialStep {
    Move,
    Attack,
    Gather,
    Jump,
    FindMiss,
}

impl TutorialStep {
    /// 1부터 5까지 (① ~ ⑤).
    pub fn number(self) -> usize {
        match self {
            TutorialStep::Move => 1,
            TutorialStep::Attack => 2,
            TutorialStep::Gather => 3,
            TutorialStep::Jump => 4,
            TutorialStep::FindMiss => 5,
        }
    }

    /// 말풍선 글 (임시 문구는 config.rs). `touch`가 true면 폰, 아니면 키보드 문구.
    pub fn text(self, touch: bool) -> &'static str {
        let texts = if touch { &TUTORIAL_TEXT_TOUCH } else { &TUTORIAL_TEXT_PC };
        texts[self.number() - 1]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialState {
    /// 시작 안 함 (이미 본 사람, `?qa`, 아직 시작 전)
    NotStarted,
    /// 단계 하나가 진행 중
    Running(TutorialStep),
    /// 건너뜀
    Skipped,
    /// 끝까지 다 봄
    Finished,
}

/// 화면(로건)이 보고 소리·저장을 할 일. 한 번씩만 온다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialEvent {
    /// 새 말풍선이 나온다.
    StepStarted(TutorialStep),
    /// 단계를 직접 해냈다 ("잘했어요!").
    StepDone(TutorialStep),
    /// 끝까지 다 봤다. `Tutorial::settings_after`로 저장할 값을 얻는다.
    Finished,
    /// 건너뛰었다. 저장할 값은 같다.
    Skipped,
}

/// 이번 프레임에 게임에서 일어난 일. `Game`이 채운다 (순수 값이라 시험에서 직접 만들 수 있다).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TutorialFacts {
    /// 이번 프레임에 걸은 거리 (픽셀, 0 이상)
    pub walked: f32,
    /// 이번 프레임에 점프를 시작했다
    pub jumped: bool,
    /// 이번 프레임에 연습용 몬스터를 맞혔다
    pub hit_practice: bool,
    /// 이번 프레임에 나무나 광석을 다 캤다
    pub mined: bool,
    /// 이번 프레임에 미스를 찾았다
    pub found_miss: bool,
    /// 연습용 몬스터가 지금 맵에 있다
    pub practice_present: bool,
    /// 지금 연습용 몬스터를 내보내도 된다 (1일째 낮이고 동굴 밖)
    pub practice_allowed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tutorial {
    state: TutorialState,
    walked: f32,
    step_seconds: f32,
    /// "잘했어요!"를 보여 주는 남은 초. 있는 동안은 다음 말풍선이 아직 안 나온다.
    cheer_left: Option<f32>,
    /// 다른 안내(미스 미션 말, 곡괭이 안내)가 말풍선 자리를 쓰는 중 (BUG-040). 화면이 매 프레임 `set_notice_up`으로 알려 준다.
    notice_up: bool,
}

impl Default for Tutorial {
    fn default() -> Self {
        Tutorial::new()
    }
}

impl Tutorial {
    pub fn new() -> Tutorial {
        Tutorial { state: TutorialState::NotStarted, walked: 0.0, step_seconds: 0.0, cheer_left: None, notice_up: false }
    }

    /// 시작한다. 이미 본 사람(`settings.tutorial_done`)이나 안내 없음(`?qa`, `no_guide`)이면 시작하지 않고 None이다.
    /// 이미 시작했거나 끝났으면 아무 일도 없다.
    pub fn start(&mut self, settings: &Settings, no_guide: bool) -> Option<TutorialEvent> {
        if self.state != TutorialState::NotStarted || settings.tutorial_done || no_guide {
            return None;
        }
        self.state = TutorialState::Running(TutorialStep::Move);
        Some(TutorialEvent::StepStarted(TutorialStep::Move))
    }

    pub fn state(&self) -> TutorialState {
        self.state
    }

    /// 지금 말풍선 단계. 진행 중이 아니면 None. (잘했어요 보여 주는 동안은 방금 끝낸 단계이고 `is_cheering`이 true다.)
    pub fn current_step(&self) -> Option<TutorialStep> {
        match self.state {
            TutorialState::Running(step) => Some(step),
            _ => None,
        }
    }

    pub fn is_cheering(&self) -> bool {
        self.cheer_left.is_some()
    }

    /// 다른 안내가 말풍선 자리를 쓰는 동안 true로 알려 준다 (`view::tutorial::bubble_text`가 말풍선을 숨기는 그 조건).
    /// true인 동안은 "잘했어요!" 시간과 ⑤의 사라지는 시간이 흐르지 않는다: 안 보이는 사이에 칭찬이 끝나 버리지 않고,
    /// 안내가 끝난 뒤 칭찬을 처음부터 다 보여 준다 (BUG-040). 단계를 해냈는지 보는 일은 그대로 한다.
    /// 매 프레임 부르면 된다 (false로 돌아와야 시간이 다시 흐른다).
    pub fn set_notice_up(&mut self, notice_up: bool) {
        self.notice_up = notice_up;
    }

    /// 건너뛰기/끝남이면 true. 어떤 단계도 더 나오지 않는다.
    pub fn is_over(&self) -> bool {
        matches!(self.state, TutorialState::Skipped | TutorialState::Finished)
    }

    /// 건너뛰기. 진행 중일 때만 되고, 한 번에 모두 끝난다.
    pub fn skip(&mut self) -> Option<TutorialEvent> {
        self.current_step()?;
        self.state = TutorialState::Skipped;
        self.cheer_left = None;
        Some(TutorialEvent::Skipped)
    }

    /// 한 프레임. 단계를 직접 해냈는지 `facts`로 본다. 생긴 일을 차례로 돌려준다.
    pub fn update(&mut self, dt: f32, facts: &TutorialFacts) -> Vec<TutorialEvent> {
        let mut events = Vec::new();
        let TutorialState::Running(step) = self.state else {
            return events;
        };
        let dt = dt.max(0.0);
        if let Some(left) = self.cheer_left {
            if self.notice_up {
                return events; // 칭찬이 가려져 있는 동안은 기다린다
            }
            let left = left - dt;
            if left > 0.0 {
                self.cheer_left = Some(left);
            } else {
                self.cheer_left = None;
                self.go_after(step, facts, &mut events);
            }
            return events;
        }
        match step {
            TutorialStep::Move => {
                self.walked += facts.walked.max(0.0);
                if self.walked >= TUTORIAL_WALK_DISTANCE {
                    self.done(step, &mut events);
                }
            }
            TutorialStep::Attack => {
                if facts.hit_practice {
                    self.done(step, &mut events);
                } else if !facts.practice_present {
                    // 연습용 몬스터가 사라졌다: 안 때려도 막히지 않는다.
                    self.go_after(step, facts, &mut events);
                }
            }
            TutorialStep::Gather => {
                if facts.mined {
                    self.done(step, &mut events);
                }
            }
            TutorialStep::Jump => {
                if facts.jumped {
                    self.done(step, &mut events);
                }
            }
            TutorialStep::FindMiss => {
                if !self.notice_up {
                    self.step_seconds += dt;
                }
                if facts.found_miss || self.step_seconds >= TUTORIAL_FIND_MISS_SECONDS {
                    self.state = TutorialState::Finished;
                    events.push(TutorialEvent::Finished);
                }
            }
        }
        events
    }

    /// 저장할 설정: 건너뛰었거나 끝났으면 `tutorial_done`을 켠 값, 아니면 그대로. 저장 쓰기는 화면이 한다.
    pub fn settings_after(&self, settings: &Settings) -> Settings {
        Settings { tutorial_done: settings.tutorial_done || self.is_over(), ..*settings }
    }

    /// 쓰러져서 "다시하기"를 할 때 새 판에 넘길 값: 이미 시작했으면 다시 나오지 않는다. 저장은 하지 않는다 (새로고침하면 나올 수 있다).
    pub fn for_next_game(&self) -> Tutorial {
        let mut next = Tutorial::new();
        if self.state != TutorialState::NotStarted {
            next.state = TutorialState::Finished;
        }
        next
    }

    fn done(&mut self, step: TutorialStep, events: &mut Vec<TutorialEvent>) {
        self.cheer_left = Some(TUTORIAL_CHEER_SECONDS);
        events.push(TutorialEvent::StepDone(step));
    }

    /// `step` 다음 말풍선으로. ②는 연습용 몬스터를 내보내도 될 때만 나온다.
    fn go_after(&mut self, step: TutorialStep, facts: &TutorialFacts, events: &mut Vec<TutorialEvent>) {
        let next = match step {
            TutorialStep::Move if facts.practice_allowed => TutorialStep::Attack,
            TutorialStep::Move | TutorialStep::Attack => TutorialStep::Gather,
            TutorialStep::Gather => TutorialStep::Jump,
            TutorialStep::Jump => TutorialStep::FindMiss,
            TutorialStep::FindMiss => {
                self.state = TutorialState::Finished;
                events.push(TutorialEvent::Finished);
                return;
            }
        };
        self.state = TutorialState::Running(next);
        self.step_seconds = 0.0;
        events.push(TutorialEvent::StepStarted(next));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_STEPS: [TutorialStep; 5] =
        [TutorialStep::Move, TutorialStep::Attack, TutorialStep::Gather, TutorialStep::Jump, TutorialStep::FindMiss];

    fn started() -> Tutorial {
        let mut t = Tutorial::new();
        assert_eq!(t.start(&Settings::default(), false), Some(TutorialEvent::StepStarted(TutorialStep::Move)));
        t
    }

    fn day_one() -> TutorialFacts {
        TutorialFacts { practice_allowed: true, ..Default::default() }
    }

    /// "잘했어요" 시간까지 흘려 다음 말풍선이 나오게 한다.
    fn finish_cheer(t: &mut Tutorial, facts: &TutorialFacts) -> Vec<TutorialEvent> {
        t.update(TUTORIAL_CHEER_SECONDS + 0.01, facts)
    }

    #[test]
    fn starts_only_for_new_player_without_qa() {
        let mut t = Tutorial::new();
        assert_eq!(t.state(), TutorialState::NotStarted);
        let seen = Settings { tutorial_done: true, ..Settings::default() };
        assert_eq!(t.start(&seen, false), None);
        assert_eq!(t.state(), TutorialState::NotStarted);
        assert_eq!(t.start(&Settings::default(), true), None, "?qa");
        assert_eq!(t.state(), TutorialState::NotStarted);
        assert!(t.start(&Settings::default(), false).is_some());
        assert_eq!(t.state(), TutorialState::Running(TutorialStep::Move));
        assert_eq!(t.start(&Settings::default(), false), None, "두 번 시작하지 않는다");
    }

    #[test]
    fn not_started_does_nothing() {
        let mut t = Tutorial::new();
        let facts = TutorialFacts { walked: 9999.0, jumped: true, mined: true, ..day_one() };
        assert!(t.update(1.0, &facts).is_empty());
        assert_eq!(t.state(), TutorialState::NotStarted);
        assert_eq!(t.skip(), None);
    }

    #[test]
    fn move_needs_real_walking_not_time() {
        let mut t = started();
        for _ in 0..100 {
            assert!(t.update(10.0, &day_one()).is_empty(), "시간만으로는 안 넘어간다");
        }
        assert_eq!(t.current_step(), Some(TutorialStep::Move));
        let step = TutorialFacts { walked: TUTORIAL_WALK_DISTANCE / 4.0, ..day_one() };
        for _ in 0..3 {
            assert!(t.update(0.016, &step).is_empty());
        }
        assert_eq!(t.update(0.016, &step), vec![TutorialEvent::StepDone(TutorialStep::Move)]);
        assert!(t.is_cheering());
    }

    #[test]
    fn negative_walk_never_goes_backwards() {
        let mut t = started();
        t.update(0.016, &TutorialFacts { walked: -500.0, ..day_one() });
        t.update(0.016, &TutorialFacts { walked: TUTORIAL_WALK_DISTANCE - 1.0, ..day_one() });
        assert_eq!(t.current_step(), Some(TutorialStep::Move));
        assert!(!t.is_cheering());
    }

    #[test]
    fn cheer_comes_before_next_step() {
        let mut t = started();
        t.update(0.016, &TutorialFacts { walked: TUTORIAL_WALK_DISTANCE, ..day_one() });
        assert!(t.is_cheering());
        assert_eq!(t.current_step(), Some(TutorialStep::Move));
        assert!(t.update(TUTORIAL_CHEER_SECONDS / 2.0, &day_one()).is_empty());
        let events = finish_cheer(&mut t, &day_one());
        assert_eq!(events, vec![TutorialEvent::StepStarted(TutorialStep::Attack)]);
        assert!(!t.is_cheering());
    }

    #[test]
    fn full_walk_through_in_order() {
        let mut t = started();
        let present = TutorialFacts { practice_present: true, ..day_one() };
        t.update(0.016, &TutorialFacts { walked: TUTORIAL_WALK_DISTANCE, ..present });
        finish_cheer(&mut t, &present);
        assert_eq!(t.current_step(), Some(TutorialStep::Attack));
        // 맞히지 않고 시간만 흘러도 연습용 몬스터가 있는 동안은 그대로
        for _ in 0..50 {
            assert!(t.update(5.0, &present).is_empty());
        }
        t.update(0.016, &TutorialFacts { hit_practice: true, ..present });
        finish_cheer(&mut t, &present);
        assert_eq!(t.current_step(), Some(TutorialStep::Gather));
        // 점프는 ③ 다음이다: 지금 뛰어도 안 센다
        assert!(t.update(0.016, &TutorialFacts { jumped: true, ..present }).is_empty());
        t.update(0.016, &TutorialFacts { mined: true, ..present });
        finish_cheer(&mut t, &present);
        assert_eq!(t.current_step(), Some(TutorialStep::Jump));
        assert!(t.update(0.016, &TutorialFacts { mined: true, ..present }).is_empty());
        t.update(0.016, &TutorialFacts { jumped: true, ..present });
        finish_cheer(&mut t, &present);
        assert_eq!(t.current_step(), Some(TutorialStep::FindMiss));
        let events = t.update(0.016, &TutorialFacts { found_miss: true, ..present });
        assert_eq!(events, vec![TutorialEvent::Finished]);
        assert_eq!(t.state(), TutorialState::Finished);
    }

    #[test]
    fn facts_during_cheer_do_not_count_for_next_step() {
        let mut t = started();
        t.update(0.016, &TutorialFacts { walked: TUTORIAL_WALK_DISTANCE, ..day_one() });
        // 잘했어요 동안 한 일은 ②로 안 센다
        assert!(t.update(0.1, &TutorialFacts { hit_practice: true, practice_present: true, ..day_one() }).is_empty());
        finish_cheer(&mut t, &day_one());
        assert_eq!(t.current_step(), Some(TutorialStep::Attack));
    }

    #[test]
    fn attack_step_skipped_when_practice_not_allowed() {
        let mut t = started();
        let night = TutorialFacts { practice_allowed: false, ..Default::default() };
        t.update(0.016, &TutorialFacts { walked: TUTORIAL_WALK_DISTANCE, ..night });
        let events = finish_cheer(&mut t, &night);
        assert_eq!(events, vec![TutorialEvent::StepStarted(TutorialStep::Gather)]);
    }

    #[test]
    fn attack_step_does_not_block_when_practice_monster_is_gone() {
        let mut t = started();
        t.update(0.016, &TutorialFacts { walked: TUTORIAL_WALK_DISTANCE, ..day_one() });
        finish_cheer(&mut t, &day_one());
        assert_eq!(t.current_step(), Some(TutorialStep::Attack));
        let gone = TutorialFacts { practice_present: false, ..day_one() };
        assert_eq!(t.update(0.016, &gone), vec![TutorialEvent::StepStarted(TutorialStep::Gather)]);
        assert!(!t.is_cheering(), "안 때렸으니 잘했어요는 없다");
    }

    #[test]
    fn hit_beats_gone_in_same_frame() {
        let mut t = started();
        t.update(0.016, &TutorialFacts { walked: TUTORIAL_WALK_DISTANCE, ..day_one() });
        finish_cheer(&mut t, &day_one());
        let both = TutorialFacts { hit_practice: true, practice_present: false, ..day_one() };
        assert_eq!(t.update(0.016, &both), vec![TutorialEvent::StepDone(TutorialStep::Attack)]);
    }

    #[test]
    fn find_miss_ends_by_timer_without_cheer() {
        let mut t = started();
        t.state = TutorialState::Running(TutorialStep::FindMiss);
        assert!(t.update(TUTORIAL_FIND_MISS_SECONDS - 1.0, &day_one()).is_empty());
        assert_eq!(t.update(1.01, &day_one()), vec![TutorialEvent::Finished]);
        assert!(t.is_over());
        assert!(!t.is_cheering());
    }

    #[test]
    fn skip_ends_everything_at_once() {
        for step in ALL_STEPS {
            let mut t = started();
            t.state = TutorialState::Running(step);
            assert_eq!(t.skip(), Some(TutorialEvent::Skipped));
            assert_eq!(t.state(), TutorialState::Skipped);
            assert_eq!(t.current_step(), None);
            let all = TutorialFacts {
                walked: 9999.0,
                jumped: true,
                hit_practice: true,
                mined: true,
                found_miss: true,
                practice_present: true,
                practice_allowed: true,
            };
            for _ in 0..20 {
                assert!(t.update(100.0, &all).is_empty(), "건너뛴 뒤에는 어떤 단계도 안 나온다");
            }
            assert_eq!(t.skip(), None, "두 번째 건너뛰기는 아무 일도 없다");
        }
    }

    #[test]
    fn skip_during_cheer_also_works() {
        let mut t = started();
        t.update(0.016, &TutorialFacts { walked: TUTORIAL_WALK_DISTANCE, ..day_one() });
        assert!(t.is_cheering());
        assert_eq!(t.skip(), Some(TutorialEvent::Skipped));
        assert!(!t.is_cheering());
        assert!(t.update(10.0, &day_one()).is_empty());
    }

    #[test]
    fn skip_or_finish_turns_tutorial_done_on() {
        let fresh = Settings::default();
        assert!(!Tutorial::new().settings_after(&fresh).tutorial_done);
        assert!(!started().settings_after(&fresh).tutorial_done, "진행 중에는 아직");
        let mut skipped = started();
        skipped.skip();
        let after = skipped.settings_after(&fresh);
        assert!(after.tutorial_done);
        assert_eq!(Settings { tutorial_done: false, ..after }, fresh, "다른 값은 그대로");
        let mut finished = started();
        finished.state = TutorialState::Running(TutorialStep::FindMiss);
        finished.update(TUTORIAL_FIND_MISS_SECONDS + 1.0, &day_one());
        assert!(finished.settings_after(&fresh).tutorial_done);
        // 이미 켜져 있으면 안 꺼진다
        let seen = Settings { tutorial_done: true, ..fresh };
        assert!(Tutorial::new().settings_after(&seen).tutorial_done);
    }

    #[test]
    fn restart_after_down_does_not_show_again_but_new_player_still_can() {
        let running = started();
        let mut next = running.for_next_game();
        assert!(next.is_over());
        assert_eq!(next.start(&Settings::default(), false), None);
        assert_eq!(Tutorial::new().for_next_game().state(), TutorialState::NotStarted);
    }

    #[test]
    fn texts_exist_for_every_step_and_are_gentle() {
        for step in ALL_STEPS {
            for touch in [false, true] {
                let text = step.text(touch);
                assert!(!text.is_empty());
                assert!(text.chars().count() <= 25, "{text}");
                assert!(!text.contains('피') && !text.contains('죽'), "{text}");
            }
        }
        assert_ne!(TutorialStep::Move.text(false), TutorialStep::Move.text(true), "PC와 폰 문구는 따로");
        assert_eq!(TutorialStep::Move.number(), 1);
        assert_eq!(TutorialStep::FindMiss.number(), 5);
    }

    // ---- BUG-040: 다른 안내가 말풍선 자리를 쓰는 동안 칭찬 시간이 안 흐른다 ----

    #[test]
    fn cheer_timer_stops_while_another_notice_is_up() {
        let mut t = started();
        t.update(0.016, &TutorialFacts { walked: TUTORIAL_WALK_DISTANCE, ..day_one() });
        assert!(t.is_cheering());
        t.set_notice_up(true);
        for _ in 0..100 {
            assert!(t.update(1.0, &day_one()).is_empty(), "안내가 떠 있는 동안은 아무 일도 없다");
        }
        assert!(t.is_cheering());
        assert_eq!(t.current_step(), Some(TutorialStep::Move));
        // 안내가 끝나면 칭찬이 처음부터 다 보인다
        t.set_notice_up(false);
        assert!(t.update(TUTORIAL_CHEER_SECONDS - 0.1, &day_one()).is_empty());
        assert!(t.is_cheering());
        assert_eq!(t.update(0.2, &day_one()), vec![TutorialEvent::StepStarted(TutorialStep::Attack)]);
    }

    #[test]
    fn cheer_that_already_ran_a_bit_keeps_its_remaining_time() {
        let mut t = started();
        t.update(0.016, &TutorialFacts { walked: TUTORIAL_WALK_DISTANCE, ..day_one() });
        t.update(TUTORIAL_CHEER_SECONDS - 0.5, &day_one()); // 0.5초 남음
        t.set_notice_up(true);
        t.update(30.0, &day_one());
        t.set_notice_up(false);
        assert!(t.update(0.4, &day_one()).is_empty(), "남은 0.5초가 그대로");
        assert_eq!(t.update(0.2, &day_one()).len(), 1);
    }

    #[test]
    fn doing_a_step_still_counts_while_notice_is_up() {
        let mut t = started();
        t.set_notice_up(true);
        let events = t.update(0.016, &TutorialFacts { walked: TUTORIAL_WALK_DISTANCE, ..day_one() });
        assert_eq!(events, vec![TutorialEvent::StepDone(TutorialStep::Move)]);
        assert_eq!(t.skip(), Some(TutorialEvent::Skipped));
        assert!(!t.is_cheering());
    }

    #[test]
    fn find_miss_timer_also_waits_while_notice_hides_the_bubble() {
        let mut t = started();
        t.state = TutorialState::Running(TutorialStep::FindMiss);
        t.set_notice_up(true);
        for _ in 0..50 {
            assert!(t.update(10.0, &day_one()).is_empty());
        }
        assert_eq!(t.current_step(), Some(TutorialStep::FindMiss));
        // 미스를 찾았다면 안내 중이어도 끝난다
        assert_eq!(t.update(0.016, &TutorialFacts { found_miss: true, ..day_one() }), vec![TutorialEvent::Finished]);
        let mut t = started();
        t.state = TutorialState::Running(TutorialStep::FindMiss);
        t.set_notice_up(false);
        assert_eq!(t.update(TUTORIAL_FIND_MISS_SECONDS + 0.1, &day_one()), vec![TutorialEvent::Finished]);
    }

    #[test]
    fn notice_flag_starts_off_and_new_game_tutorial_is_clean() {
        assert!(!Tutorial::new().for_next_game().is_cheering());
        let mut fresh = Tutorial::new();
        fresh.start(&Settings::default(), false);
        fresh.update(0.016, &TutorialFacts { walked: TUTORIAL_WALK_DISTANCE, ..day_one() });
        assert_eq!(fresh.update(TUTORIAL_CHEER_SECONDS + 0.01, &day_one()).len(), 1, "처음 값은 안내 없음");    }
}
