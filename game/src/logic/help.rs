//! 물음표(?) 버튼과 조작 안내 한 장의 규칙 (US-122). 보임·열림·닫힘만 안다. 그림은 화면(로건)이 그린다.
//!
//! - 버튼은 플레이를 시작하고 `HELP_BUTTON_SECONDS`초 동안만 보인다. 그 뒤에는 안 보이고 다시 볼 길도 지금은 없다 (임시-136).
//! - 건너뛴 사람·이미 본 사람·튜토리얼 상태와 상관없이 누구에게나 보인다 (스토리: "플레이를 시작하면 보인다").
//!   튜토리얼의 "다 봤다" 상태는 이 파일이 건드리지 않는다.
//! - 세는 시간은 게임 시계(빨리감기)가 아니라 **실제 시간**이고, 멈춤 창·가방·제작대 같은 창이 열려 시간이 멈춘 동안은 세지 않는다
//!   (멈춤 창에서 오래 쉬었다가 돌아왔더니 버튼이 없어지는 일이 없게). 판이 새로 시작하면 새로 센다 (`Help::new`).
//! - 조작 안내가 열려 있는 동안 **게임은 멈춘다** (가방·멈춤 창과 같은 방식. 글을 천천히 읽는 동안 몬스터에게 맞지 않게).
//! - 안내가 열린 채로 30초가 지나도 안내는 그대로 열려 있고, 닫으면 버튼은 이미 없다.

use crate::config::{HELP_BUTTON_SECONDS, HELP_LINES_PC, HELP_LINES_TOUCH};

#[derive(Debug, Clone, PartialEq)]
pub struct Help {
    /// 플레이한 실제 초 (멈춘 동안은 안 센다)
    played_seconds: f32,
    open: bool,
    /// `?qa`처럼 안내를 끈 판: 버튼이 없다
    no_guide: bool,
}

impl Help {
    /// 플레이를 시작할 때 만든다. `no_guide`가 true면 버튼이 아예 없다 (`?qa`, US-123).
    pub fn new(no_guide: bool) -> Help {
        Help { played_seconds: 0.0, open: false, no_guide }
    }

    /// 한 프레임. `dt`는 실제 초. `time_stopped`가 true면 (멈춤 창·가방·제작대·상점·문제 창이 열림) 세지 않는다.
    /// 조작 안내가 열려 있는 동안도 세지 않는다.
    pub fn update(&mut self, dt: f32, time_stopped: bool) {
        if !time_stopped && !self.open {
            self.played_seconds += dt.max(0.0);
        }
    }

    /// 물음표 버튼이 보이는지: 처음 `HELP_BUTTON_SECONDS`초 동안만 (30.0초가 되는 순간 안 보인다).
    pub fn button_visible(&self) -> bool {
        !self.no_guide && self.played_seconds < HELP_BUTTON_SECONDS
    }

    /// 버튼을 눌렀다. 보일 때만 열리고 true. 안 보이는데 눌렀거나 이미 열려 있으면 아무 일도 없다 (false).
    pub fn press_button(&mut self) -> bool {
        if self.open || !self.button_visible() {
            return false;
        }
        self.open = true;
        true
    }

    /// 조작 안내가 열려 있다 (그리기, 닫기 버튼 받기).
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// 닫기. 열려 있었으면 true. 하던 판은 그대로 이어진다 (판이 새로 시작되지 않는다).
    pub fn close(&mut self) -> bool {
        std::mem::replace(&mut self.open, false)
    }

    /// 열려 있는 동안 게임 시간을 멈춰야 하는지 (`is_open`과 같다. 멈춤 조건에 이 함수를 넣는다).
    pub fn pauses_game(&self) -> bool {
        self.open
    }

    /// 안내 줄들. `touch`가 true면 폰, 아니면 PC. 지금 게임의 실제 키·버튼과 같다 (config.rs).
    /// 한 줄은 (무엇을 하는지, 어떻게 하는지).
    pub fn lines(touch: bool) -> &'static [(&'static str, &'static str)] {
        if touch {
            &HELP_LINES_TOUCH
        } else {
            &HELP_LINES_PC
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    #[test]
    fn button_is_visible_for_thirty_seconds_then_gone() {
        let mut help = Help::new(false);
        assert!(help.button_visible(), "시작하자마자 보인다");
        help.update(HELP_BUTTON_SECONDS - 0.01, false);
        assert!(help.button_visible(), "29.99초");
        help.update(0.02, false);
        assert!(!help.button_visible(), "30초가 넘으면 안 보인다");
        // 한 번 사라지면 다시 안 나온다
        for _ in 0..50 {
            help.update(1.0, true);
            help.update(1.0, false);
            assert!(!help.button_visible());
        }
    }

    #[test]
    fn exactly_thirty_seconds_is_gone() {
        let mut help = Help::new(false);
        help.update(HELP_BUTTON_SECONDS, false);
        assert!(!help.button_visible());
    }

    #[test]
    fn thirty_seconds_is_the_config_value() {
        assert_eq!(HELP_BUTTON_SECONDS, 30.0, "부모님이 정한 값");
    }

    #[test]
    fn frozen_time_does_not_count() {
        let mut help = Help::new(false);
        for _ in 0..1000 {
            help.update(10.0, true);
        }
        assert!(help.button_visible(), "멈춰 있는 동안은 안 센다");
        help.update(HELP_BUTTON_SECONDS - 1.0, false);
        assert!(help.button_visible());
    }

    #[test]
    fn negative_dt_never_gives_extra_time() {
        let mut help = Help::new(false);
        help.update(HELP_BUTTON_SECONDS - 1.0, false);
        help.update(-1000.0, false);
        help.update(1.5, false);
        assert!(!help.button_visible());
    }

    #[test]
    fn pressing_opens_closing_goes_back_and_open_time_is_not_counted() {
        let mut help = Help::new(false);
        assert!(!help.is_open());
        assert!(help.press_button());
        assert!(help.is_open() && help.pauses_game());
        assert!(!help.press_button(), "이미 열려 있으면 또 열리지 않는다");
        // 열려 있는 동안 시간이 아무리 흘러도 버튼 시간은 안 간다
        for _ in 0..100 {
            help.update(10.0, false);
        }
        assert!(help.close());
        assert!(!help.is_open() && !help.pauses_game());
        assert!(help.button_visible(), "읽는 동안은 시간이 안 갔다");
        assert!(!help.close(), "닫혀 있는데 또 닫으면 아무 일도 없다");
        assert!(help.press_button(), "닫은 뒤 30초 안이면 다시 볼 수 있다");
    }

    #[test]
    fn press_after_thirty_seconds_does_nothing() {
        let mut help = Help::new(false);
        help.update(HELP_BUTTON_SECONDS + 1.0, false);
        assert!(!help.press_button());
        assert!(!help.is_open());
    }

    #[test]
    fn qa_mode_has_no_button() {
        let mut help = Help::new(true);
        assert!(!help.button_visible());
        assert!(!help.press_button());
        assert!(!help.is_open());
    }

    #[test]
    fn open_help_stays_open_if_it_was_opened_before_thirty_seconds() {
        let mut help = Help::new(false);
        help.update(HELP_BUTTON_SECONDS - 0.1, false);
        assert!(help.press_button());
        help.update(60.0, false);
        assert!(help.is_open(), "열린 채로 있고, 닫을 때까지 읽을 수 있다");
        help.close();
        assert!(help.button_visible(), "열린 동안은 시간이 안 갔다");
    }

    #[test]
    fn new_game_starts_the_thirty_seconds_again() {
        let mut help = Help::new(false);
        help.update(HELP_BUTTON_SECONDS + 5.0, false);
        assert!(!help.button_visible());
        let next = Help::new(false);
        assert!(next.button_visible());
    }

    #[test]
    fn lines_cover_the_six_actions_for_pc_and_phone() {
        for touch in [false, true] {
            let lines = Help::lines(touch);
            let names: Vec<&str> = lines.iter().map(|l| l.0).collect();
            for must in ["걷기", "점프", "공격", "캐기", "가방", "멈춤"] {
                assert!(names.contains(&must), "{must} (touch={touch})");
            }
            assert!(lines.len() <= HELP_LINES_MAX, "한 장에 들어갈 만큼만");
            for l in lines {
                assert!(!l.0.is_empty() && !l.1.is_empty());
                assert!(l.1.chars().count() <= 22, "{}", l.1);
                assert!(!l.1.contains('피') && !l.1.contains('죽'), "{}", l.1);
            }
        }
        assert_ne!(Help::lines(false), Help::lines(true), "PC와 폰은 따로");
    }

    #[test]
    fn pc_lines_use_the_real_keys() {
        let pc: Vec<(&str, &str)> = Help::lines(false).iter().map(|l| (l.0, l.1)).collect();
        let how = |what: &str| pc.iter().find(|(w, _)| *w == what).unwrap().1;
        assert!(how("걷기").contains('←') && how("걷기").contains('→'));
        assert!(how("점프").contains("Space"));
        assert!(how("공격").contains('Q'));
        assert!(how("캐기").contains('E'));
        assert!(how("가방").contains('I'));
        assert!(how("멈춤").contains("Esc"));
    }
}
