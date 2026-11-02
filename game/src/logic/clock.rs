//! 낮/밤 시계. 1일째 낮에서 시작해서 낮 → 밤 → 낮 … 을 되풀이한다.
//!
//! 빨리감기(D12)는 시계가 따로 알 필요가 없다. 부르는 쪽이 `dt × DEV_FAST_FORWARD_SCALE`을 넣으면 된다.


use crate::config::{
    DAWN_SECONDS, DAY_GAME_MINUTES, DAY_SECONDS, DAY_START_HOUR, DUSK_SECONDS, MINUTES_PER_GAME_DAY,
    NIGHT_GAME_MINUTES, NIGHT_SECONDS,
};

/// 지금이 낮인지 밤인지.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Day,
    Night,
}

/// 시계가 알려 주는 일.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockEvent {
    /// 해가 지고 밤이 시작됐다.
    NightStarted,
    /// 밤을 버티고 아침이 왔다. `day_number`는 새로 시작한 날 (2, 3, 4 …).
    DayStarted { day_number: u32 },
}

pub struct Clock {
    phase: Phase,
    /// 지금 구간(낮 또는 밤)이 시작된 뒤 흐른 초.
    seconds_in_phase: f32,
    /// 몇 번째 날인지. 1부터.
    day_number: u32,
}

impl Clock {
    /// 1일째 낮, 0초에서 시작.
    pub fn new() -> Clock {
        Clock { phase: Phase::Day, seconds_in_phase: 0.0, day_number: 1 }
    }

    /// 시간을 `dt`초 흘려 보낸다. 그 사이에 낮/밤이 바뀐 일을 순서대로 모두 돌려준다.
    /// `dt`가 아주 커서 여러 번 바뀌어도 하나도 빠뜨리지 않는다.
    pub fn update(&mut self, dt: f32) -> Vec<ClockEvent> {
        let mut events = Vec::new();
        self.seconds_in_phase += dt.max(0.0);
        while self.seconds_in_phase >= self.phase_length() {
            self.seconds_in_phase -= self.phase_length();
            match self.phase {
                Phase::Day => {
                    self.phase = Phase::Night;
                    events.push(ClockEvent::NightStarted);
                }
                Phase::Night => {
                    self.phase = Phase::Day;
                    self.day_number += 1;
                    events.push(ClockEvent::DayStarted { day_number: self.day_number });
                }
            }
        }
        events
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn is_night(&self) -> bool {
        self.phase == Phase::Night
    }

    /// 지금 구간(낮 또는 밤)이 끝날 때까지 남은 초. 화면 시계에 쓴다.
    pub fn seconds_left_in_phase(&self) -> f32 {
        (self.phase_length() - self.seconds_in_phase).max(0.0)
    }

    /// 몇 번째 날인지 (1부터).
    pub fn day_number(&self) -> u32 {
        self.day_number
    }

    /// 버틴 밤의 수 (D4 기록). 밤이 끝나고 아침이 올 때마다 1씩 는다.
    pub fn nights_survived(&self) -> u32 {
        self.day_number - 1
    }

    /// 하늘이 얼마나 어두운지: 0 = 한낮, 1 = 한밤.
    /// 낮의 마지막 DUSK_SECONDS 동안 0 → 1 (노을), 밤의 마지막 DAWN_SECONDS 동안 1 → 0 (새벽).
    /// 그래서 밤이 시작되는 순간 딱 1, 아침이 오는 순간 딱 0이다.
    pub fn sky_darkness(&self) -> f32 {
        let left = self.seconds_left_in_phase();
        match self.phase {
            Phase::Day => {
                if left < DUSK_SECONDS {
                    smooth(1.0 - left / DUSK_SECONDS)
                } else {
                    0.0
                }
            }
            Phase::Night => {
                if left < DAWN_SECONDS {
                    smooth(left / DAWN_SECONDS)
                } else {
                    1.0
                }
            }
        }
    }

    /// 24시간 시계의 지금 시각을 분으로 (D14). 00:00 = 0, 23:59 = 1439.
    /// 낮은 06:00~18:00, 밤은 18:00~06:00. 낮·밤 길이가 달라도 각각 12시간에 맞춰 늘이거나 줄인다.
    /// 분은 내림이라 화면 시계가 뒤로 가는 일이 없다.
    pub fn time_of_day_minutes(&self) -> u32 {
        // 오늘 아침(06:00)부터 몇 분 지났는지
        let minutes_since_morning = match self.phase {
            Phase::Day => minutes_into_phase(self.seconds_in_phase, DAY_SECONDS, DAY_GAME_MINUTES),
            Phase::Night => {
                DAY_GAME_MINUTES + minutes_into_phase(self.seconds_in_phase, NIGHT_SECONDS, NIGHT_GAME_MINUTES)
            }
        };
        (DAY_START_HOUR * 60 + minutes_since_morning) % MINUTES_PER_GAME_DAY
    }

    /// 24시간 시계의 (시, 분). 예: 오후 3시 7분 → (15, 7).
    pub fn hour_minute(&self) -> (u32, u32) {
        let m = self.time_of_day_minutes();
        (m / 60, m % 60)
    }

    fn phase_length(&self) -> f32 {
        match self.phase {
            Phase::Day => DAY_SECONDS,
            Phase::Night => NIGHT_SECONDS,
        }
    }
}

impl Default for Clock {
    fn default() -> Clock {
        Clock::new()
    }
}

/// 구간이 시작된 뒤 `seconds`초가 지났을 때, 게임 시계로 몇 분 지났는지 (내림).
/// 구간(실제 `phase_seconds`초)이 게임 시계 `phase_minutes`분에 맞도록 늘이거나 줄인다.
/// 구간이 끝나기 전에는 마지막 분을 넘지 않는다 → 낮인데 18:00이 보이는 일이 없다.
fn minutes_into_phase(seconds: f32, phase_seconds: f32, phase_minutes: u32) -> u32 {
    let minutes = (seconds.max(0.0) / phase_seconds * phase_minutes as f32).floor() as u32;
    minutes.min(phase_minutes - 1)
}

/// 0~1 값을 처음과 끝이 살살 바뀌게 만든다 (smoothstep).
fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_on_day_one_in_daytime() {
        let c = Clock::new();
        assert_eq!(c.phase(), Phase::Day);
        assert_eq!(c.day_number(), 1);
        assert_eq!(c.nights_survived(), 0);
        assert_eq!(c.seconds_left_in_phase(), DAY_SECONDS);
        assert_eq!(c.sky_darkness(), 0.0);
    }

    #[test]
    fn night_starts_after_600_seconds() {
        let mut c = Clock::new();
        assert!(c.update(599.0).is_empty());
        assert_eq!(c.phase(), Phase::Day);
        assert_eq!(c.update(1.0), vec![ClockEvent::NightStarted]);
        assert!(c.is_night());
        assert_eq!(c.day_number(), 1);
        assert_eq!(c.seconds_left_in_phase(), NIGHT_SECONDS);
    }

    #[test]
    fn second_day_after_1200_seconds() {
        let mut c = Clock::new();
        let mut events = Vec::new();
        // 한 프레임씩 20분.
        for _ in 0..(1200 * 60) {
            events.extend(c.update(1.0 / 60.0));
        }
        // f32 오차로 살짝 모자랄 수 있어서 1초 더 흘린다 (다음 일은 한참 뒤라 괜찮다).
        events.extend(c.update(1.0));
        assert_eq!(events, vec![ClockEvent::NightStarted, ClockEvent::DayStarted { day_number: 2 }]);
        assert_eq!(c.phase(), Phase::Day);
        assert_eq!(c.day_number(), 2);
        assert_eq!(c.nights_survived(), 1);
    }

    #[test]
    fn huge_dt_gives_every_event_in_order() {
        let mut c = Clock::new();
        // 3000초 = 낮(600) 밤(600) 낮(600) 밤(600) 낮(600) → 3일째 밤 시작
        let events = c.update(3000.0);
        assert_eq!(
            events,
            vec![
                ClockEvent::NightStarted,
                ClockEvent::DayStarted { day_number: 2 },
                ClockEvent::NightStarted,
                ClockEvent::DayStarted { day_number: 3 },
                ClockEvent::NightStarted,
            ]
        );
        assert!(c.is_night());
        assert_eq!(c.day_number(), 3);
        assert_eq!(c.nights_survived(), 2);
    }

    #[test]
    fn fast_forward_is_just_bigger_dt() {
        use crate::config::DEV_FAST_FORWARD_SCALE;
        let mut c = Clock::new();
        // 빨리감기를 켜고 60초 동안 플레이하면 600초가 흐른다 → 밤.
        let mut events = Vec::new();
        for _ in 0..(60 * 60) {
            events.extend(c.update(1.0 / 60.0 * DEV_FAST_FORWARD_SCALE));
        }
        events.extend(c.update(1.0));
        assert_eq!(events, vec![ClockEvent::NightStarted]);
    }

    #[test]
    fn sky_darkness_noon_midnight_and_between() {
        let mut c = Clock::new();
        c.update(DAY_SECONDS / 2.0);
        assert_eq!(c.sky_darkness(), 0.0, "한낮");

        // 노을 한가운데
        c.update(DAY_SECONDS / 2.0 - DUSK_SECONDS / 2.0);
        let dusk = c.sky_darkness();
        assert!(dusk > 0.0 && dusk < 1.0, "노을 중에는 사이값: {dusk}");

        // 밤 시작 순간 = 1
        c.update(DUSK_SECONDS / 2.0);
        assert!(c.is_night());
        assert_eq!(c.sky_darkness(), 1.0);

        c.update(NIGHT_SECONDS / 2.0);
        assert_eq!(c.sky_darkness(), 1.0, "한밤");

        // 새벽 한가운데
        c.update(NIGHT_SECONDS / 2.0 - DAWN_SECONDS / 2.0);
        let dawn = c.sky_darkness();
        assert!(dawn > 0.0 && dawn < 1.0, "새벽 중에는 사이값: {dawn}");

        // 아침 = 0
        c.update(DAWN_SECONDS / 2.0);
        assert_eq!(c.phase(), Phase::Day);
        assert_eq!(c.sky_darkness(), 0.0);
    }

    #[test]
    fn sky_gets_darker_little_by_little_during_dusk() {
        let mut c = Clock::new();
        c.update(DAY_SECONDS - DUSK_SECONDS);
        let mut last = c.sky_darkness();
        for _ in 0..(DUSK_SECONDS as usize - 1) {
            c.update(1.0);
            let now = c.sky_darkness();
            assert!(now > last, "노을 동안 계속 어두워진다");
            assert!(now - last < 0.1, "한 번에 확 바뀌지 않는다");
            last = now;
        }
    }

    #[test]
    fn negative_dt_does_nothing() {
        let mut c = Clock::new();
        assert!(c.update(-5.0).is_empty());
        assert_eq!(c.seconds_left_in_phase(), DAY_SECONDS);
    }

    // ---------- D14 24시간 시계 ----------

    #[test]
    fn clock_starts_at_six_in_the_morning() {
        let c = Clock::new();
        assert_eq!(c.time_of_day_minutes(), 6 * 60);
        assert_eq!(c.hour_minute(), (6, 0));
    }

    #[test]
    fn noon_after_300_seconds() {
        let mut c = Clock::new();
        c.update(300.0);
        assert_eq!(c.hour_minute(), (12, 0));
    }

    #[test]
    fn one_real_second_is_1_2_game_minutes() {
        let mut c = Clock::new();
        c.update(5.0); // 6분
        assert_eq!(c.hour_minute(), (6, 6));
        c.update(1.0); // 7.2분 → 내림 7분
        assert_eq!(c.hour_minute(), (6, 7));
    }

    #[test]
    fn evening_six_right_when_night_starts() {
        let mut c = Clock::new();
        c.update(599.9);
        assert_eq!(c.phase(), Phase::Day);
        assert_eq!(c.hour_minute(), (17, 59), "낮의 끝은 17:59");
        c.update(0.1);
        assert!(c.is_night());
        assert_eq!(c.hour_minute(), (18, 0));
    }

    #[test]
    fn midnight_at_900_seconds_and_three_at_1050() {
        let mut c = Clock::new();
        c.update(900.0);
        assert_eq!(c.hour_minute(), (0, 0), "자정");
        assert_eq!(c.day_number(), 1, "자정에는 날이 안 바뀐다");
        c.update(150.0);
        assert_eq!(c.hour_minute(), (3, 0));
        assert_eq!(c.day_number(), 1);
    }

    #[test]
    fn six_again_and_day_two_after_1200_seconds() {
        let mut c = Clock::new();
        c.update(1200.0);
        assert_eq!(c.phase(), Phase::Day);
        assert_eq!(c.hour_minute(), (6, 0));
        assert_eq!(c.day_number(), 2);
        assert_eq!(c.nights_survived(), 1);
    }

    #[test]
    fn clock_never_goes_backward_frame_by_frame() {
        // 하루(실제 20분)를 60프레임/초로 흘리며, 자정을 넘을 때만 빼고 시각이 줄지 않는지 본다.
        let mut c = Clock::new();
        let mut last = c.time_of_day_minutes();
        let mut wraps = 0;
        for _ in 0..(1200 * 60) {
            c.update(1.0 / 60.0);
            let now = c.time_of_day_minutes();
            assert!(now < MINUTES_PER_GAME_DAY);
            if now < last {
                wraps += 1;
                assert_eq!((last, now), (MINUTES_PER_GAME_DAY - 1, 0), "자정 넘김만 허용");
            } else {
                assert!(now - last <= 1, "한 프레임에 1분 넘게 뛰지 않는다");
            }
            last = now;
        }
        assert_eq!(wraps, 1, "하루에 자정은 한 번");
    }

    #[test]
    fn big_dt_keeps_time_of_day_right() {
        let mut c = Clock::new();
        c.update(3000.0); // 3일째 밤 시작
        assert_eq!(c.hour_minute(), (18, 0));
        assert_eq!(c.day_number(), 3);
        c.update(1200.0 * 10.0 + 300.0); // 10일 더 + 300초 → 자정
        assert_eq!(c.hour_minute(), (0, 0));
        assert_eq!(c.day_number(), 13);
    }

    #[test]
    fn negative_dt_does_not_change_time() {
        let mut c = Clock::new();
        c.update(-5.0);
        assert_eq!(c.hour_minute(), (6, 0));
    }
}
