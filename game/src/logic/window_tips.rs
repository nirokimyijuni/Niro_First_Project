//! 창 한 줄 안내 규칙 (US-121). 여섯 창이 "처음 열릴 때 한 번만" 한 줄 안내를 보여 준다.
//! 어느 창이 열렸는지 받아서 "안내를 보여야 하나?"만 정한다. 그림·글자 그리기·저장 쓰기는 화면(로건)이 한다.
//!
//! - 창마다 따로 기억한다 (`WindowTipsSeen`, 저장하는 값). 수학 퀴즈와 영어 퀴즈는 서로 다른 창이다.
//! - 안내는 `WINDOW_TIP_SECONDS`초 뒤 저절로 사라지거나 창이 닫히면 사라진다. 창 기능을 막지 않는다.
//! - 보이지 않아야 할 때(`?qa`, 튜토리얼 말풍선이 떠 있는 동안)는 보여 주지도, "봤다"로 적지도 않는다.
//!   그래서 말풍선이 있을 때 연 창은 다음에 열 때 안내가 나온다 (안내를 못 보고 지나가지 않게).
//! - 건너뛰기(US-120)는 말풍선만 끈다. 한 줄 안내는 그대로 나온다 (임시-132).

use super::settings::Settings;
use crate::config::{WINDOW_TIP_SECONDS, WINDOW_TIP_TEXT_PC, WINDOW_TIP_TEXT_TOUCH};

/// 한 줄 안내가 나오는 창 여섯 곳 (임시-126).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipWindow {
    MathQuiz,
    EnglishQuiz,
    Crafting,
    Shop,
    Bag,
    SurpriseMission,
}

impl TipWindow {
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))] // 저장 읽고 쓰기는 웹(wasm) 화면(view/web.rs)과 시험에서만 쓴다
    pub const ALL: [TipWindow; 6] = [
        TipWindow::MathQuiz,
        TipWindow::EnglishQuiz,
        TipWindow::Crafting,
        TipWindow::Shop,
        TipWindow::Bag,
        TipWindow::SurpriseMission,
    ];

    /// 0부터 5까지 (`config`의 글 목록 순서이고, 저장 숫자의 몇 번째 칸인지).
    pub fn index(self) -> usize {
        match self {
            TipWindow::MathQuiz => 0,
            TipWindow::EnglishQuiz => 1,
            TipWindow::Crafting => 2,
            TipWindow::Shop => 3,
            TipWindow::Bag => 4,
            TipWindow::SurpriseMission => 5,
        }
    }

    /// 한 줄 안내 글 (임시 문구는 config.rs). `touch`가 true면 폰, 아니면 PC 문구.
    pub fn text(self, touch: bool) -> &'static str {
        let texts = if touch { &WINDOW_TIP_TEXT_TOUCH } else { &WINDOW_TIP_TEXT_PC };
        texts[self.index()]
    }

    fn bit(self) -> u8 {
        1 << self.index()
    }
}

/// 어느 창의 안내를 이미 봤는지 (저장하는 값). 창 하나가 숫자의 한 칸(비트)이다. 처음에는 모두 안 봤다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WindowTipsSeen(u8);

impl WindowTipsSeen {
    /// 모든 창을 본 숫자. 저장 숫자는 이보다 클 수 없다.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))] // 저장 읽고 쓰기는 웹(wasm) 화면(view/web.rs)과 시험에서만 쓴다
    pub const ALL_BITS: u8 = (1 << TipWindow::ALL.len()) - 1;

    /// 저장 숫자로 만든다. 여섯 칸 밖의 숫자면 None (이상한 값).
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))] // 저장 읽고 쓰기는 웹(wasm) 화면(view/web.rs)과 시험에서만 쓴다
    pub fn from_bits(bits: u8) -> Option<WindowTipsSeen> {
        (bits <= Self::ALL_BITS).then_some(WindowTipsSeen(bits))
    }

    /// 저장할 숫자 (0 ~ `ALL_BITS`).
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))] // 저장 읽고 쓰기는 웹(wasm) 화면(view/web.rs)과 시험에서만 쓴다
    pub fn bits(self) -> u8 {
        self.0
    }

    pub fn has_seen(self, window: TipWindow) -> bool {
        self.0 & window.bit() != 0
    }

    fn mark(&mut self, window: TipWindow) {
        self.0 |= window.bit();
    }
}

/// 한 줄 안내 규칙. 한 번 만들어 판이 바뀌어도 이어 쓴다 (다시하기를 해도 본 창은 또 나오지 않는다).
#[derive(Debug, Clone, PartialEq)]
pub struct WindowTips {
    seen: WindowTipsSeen,
    /// 지금 보이는 안내와 남은 초
    showing: Option<(TipWindow, f32)>,
}

impl Default for WindowTips {
    fn default() -> Self {
        WindowTips::new(WindowTipsSeen::default())
    }
}

impl WindowTips {
    pub fn new(seen: WindowTipsSeen) -> WindowTips {
        WindowTips { seen, showing: None }
    }

    /// 저장해 둔 설정에서 시작한다.
    pub fn from_settings(settings: &Settings) -> WindowTips {
        WindowTips::new(settings.window_tips_seen)
    }

    /// 창이 열렸다. 안내를 보여야 하면 그 창을 돌려주고(지금부터 `current`로 보인다) "봤다"로 적는다.
    /// - `no_guide`: `?qa`처럼 안내를 끈 판. 보이지도 적지도 않는다.
    /// - `tutorial_bubble_up`: 튜토리얼 말풍선이 지금 떠 있다(`Tutorial::current_step().is_some()`).
    ///   같은 순간에 겹치지 않게 이번에는 보이지도 적지도 않는다. 다음에 열면 나온다.
    pub fn window_opened(&mut self, window: TipWindow, no_guide: bool, tutorial_bubble_up: bool) -> Option<TipWindow> {
        if no_guide || tutorial_bubble_up || self.seen.has_seen(window) {
            return None;
        }
        self.seen.mark(window);
        self.showing = Some((window, WINDOW_TIP_SECONDS));
        Some(window)
    }

    /// 한 프레임. 안내가 떠 있으면 남은 초를 줄이고, 다 되면 사라진다. `dt`는 실제 초 (창이 열려 시간이 멈춰도 센다).
    pub fn update(&mut self, dt: f32) {
        if let Some((window, left)) = self.showing {
            let left = left - dt.max(0.0);
            self.showing = (left > 0.0).then_some((window, left));
        }
    }

    /// 지금 보이는 안내가 있는 창. 없으면 None.
    pub fn current(&self) -> Option<TipWindow> {
        self.showing.map(|(window, _)| window)
    }

    /// 창이 닫혔다. 떠 있던 안내는 사라진다.
    pub fn window_closed(&mut self) {
        self.showing = None;
    }

    /// 지금까지 본 창들.
    #[cfg_attr(not(test), allow(dead_code))] // 화면은 쓰지 않고 시험에서만 본 창을 확인한다
    pub fn seen(&self) -> WindowTipsSeen {
        self.seen
    }

    /// 저장할 설정: 본 창 표시만 바꾼 값 (다른 값은 그대로). 저장 쓰기는 화면이 하고,
    /// `window_opened`가 Some을 돌려줘서 `settings_after(&settings) != settings`일 때만 쓰면 된다.
    pub fn settings_after(&self, settings: &Settings) -> Settings {
        Settings { window_tips_seen: self.seen, ..*settings }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open(tips: &mut WindowTips, w: TipWindow) -> Option<TipWindow> {
        tips.window_opened(w, false, false)
    }

    #[test]
    fn first_open_shows_second_open_does_not_for_each_window() {
        let mut tips = WindowTips::default();
        for w in TipWindow::ALL {
            assert_eq!(open(&mut tips, w), Some(w), "처음");
            assert_eq!(tips.current(), Some(w));
            tips.window_closed();
            assert_eq!(open(&mut tips, w), None, "두 번째");
            assert_eq!(tips.current(), None);
        }
    }

    #[test]
    fn windows_are_remembered_separately() {
        let mut tips = WindowTips::default();
        assert!(open(&mut tips, TipWindow::MathQuiz).is_some());
        assert!(tips.seen().has_seen(TipWindow::MathQuiz));
        for w in TipWindow::ALL.into_iter().filter(|w| *w != TipWindow::MathQuiz) {
            assert!(!tips.seen().has_seen(w), "{w:?}");
        }
        assert!(open(&mut tips, TipWindow::EnglishQuiz).is_some(), "영어는 수학과 따로");
        assert_eq!(open(&mut tips, TipWindow::MathQuiz), None);
    }

    #[test]
    fn qa_mode_never_shows_and_never_remembers() {
        let mut tips = WindowTips::default();
        for _ in 0..10 {
            for w in TipWindow::ALL {
                assert_eq!(tips.window_opened(w, true, false), None);
            }
        }
        assert_eq!(tips.current(), None);
        assert_eq!(tips.seen(), WindowTipsSeen::default(), "?qa는 저장할 것을 바꾸지 않는다");
        assert!(open(&mut tips, TipWindow::Shop).is_some(), "?qa가 아닌 판에서는 나온다");
    }

    #[test]
    fn tutorial_bubble_holds_the_tip_back_until_next_time() {
        let mut tips = WindowTips::default();
        assert_eq!(tips.window_opened(TipWindow::Bag, false, true), None, "말풍선이 떠 있다");
        assert!(!tips.seen().has_seen(TipWindow::Bag), "못 봤으니 적지 않는다");
        assert_eq!(tips.window_opened(TipWindow::Bag, false, false), Some(TipWindow::Bag), "다음에 열면 나온다");
    }

    #[test]
    fn tip_disappears_after_its_seconds_or_when_window_closes() {
        let mut tips = WindowTips::default();
        open(&mut tips, TipWindow::Crafting);
        tips.update(WINDOW_TIP_SECONDS - 0.1);
        assert_eq!(tips.current(), Some(TipWindow::Crafting));
        tips.update(0.2);
        assert_eq!(tips.current(), None);
        open(&mut tips, TipWindow::Shop);
        tips.window_closed();
        assert_eq!(tips.current(), None);
        tips.update(-5.0); // 이상한 dt도 괜찮다
        assert_eq!(tips.current(), None);
    }

    #[test]
    fn bits_are_a_small_number_and_round_trip() {
        assert_eq!(WindowTipsSeen::ALL_BITS, 0b11_1111);
        assert_eq!(WindowTipsSeen::from_bits(0), Some(WindowTipsSeen::default()));
        for bits in 0..=WindowTipsSeen::ALL_BITS {
            assert_eq!(WindowTipsSeen::from_bits(bits).unwrap().bits(), bits);
        }
        for bad in [64u8, 100, 255] {
            assert_eq!(WindowTipsSeen::from_bits(bad), None);
        }
        let mut tips = WindowTips::default();
        for (i, w) in TipWindow::ALL.into_iter().enumerate() {
            assert_eq!(w.index(), i);
            open(&mut tips, w);
        }
        assert_eq!(tips.seen().bits(), WindowTipsSeen::ALL_BITS);
    }

    #[test]
    fn settings_after_only_changes_the_seen_windows() {
        let base = Settings { tutorial_done: true, music_percent: 10, ..Settings::default() };
        let mut tips = WindowTips::from_settings(&base);
        assert_eq!(tips.settings_after(&base), base, "아무것도 안 열었으면 그대로");
        open(&mut tips, TipWindow::Shop);
        let after = tips.settings_after(&base);
        assert!(after.window_tips_seen.has_seen(TipWindow::Shop));
        assert_eq!(Settings { window_tips_seen: base.window_tips_seen, ..after }, base);
        // 저장된 값에서 이어 가면 본 창은 또 안 나온다
        let mut again = WindowTips::from_settings(&after);
        assert_eq!(open(&mut again, TipWindow::Shop), None);
        assert!(open(&mut again, TipWindow::Bag).is_some());
    }

    #[test]
    fn texts_exist_are_short_gentle_and_do_not_give_answers() {
        for w in TipWindow::ALL {
            for touch in [false, true] {
                let t = w.text(touch);
                assert!(!t.is_empty());
                assert!(t.chars().count() <= 25, "{t}");
                assert!(!t.contains('피') && !t.contains('죽') && !t.contains("정답은"), "{t}");
            }
        }
        // 숫자·고르기가 다른 곳은 PC와 폰 글이 따로
        for w in [TipWindow::MathQuiz, TipWindow::EnglishQuiz, TipWindow::Crafting] {
            assert_ne!(w.text(false), w.text(true), "{w:?}");
        }
        assert_ne!(TipWindow::MathQuiz.text(false), TipWindow::EnglishQuiz.text(false), "수학과 영어는 다른 안내");
        assert_ne!(TipWindow::MathQuiz.text(true), TipWindow::EnglishQuiz.text(true));
        assert!(WINDOW_TIP_SECONDS > 0.0);
    }
}
