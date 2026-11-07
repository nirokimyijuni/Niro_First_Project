//! US-016 "정보판 시계가 24시간 시계처럼 앞으로 간다" 규칙 테스트 (엘리).
//! 기준: stories/backlog.md US-016 완료 조건 + docs/02-plan.md D13, D14.
//! 화면 글자(view/hud.rs의 hh_mm)는 bin 쪽이라 여기서 못 부른다 → 같은 모양 "{:02}:{:02}"로 흉내 낸다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)] // 이 테스트에서 안 쓰는 logic 함수(체력·번쩍임 등)가 있다
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::clock::{Clock, Phase};

const FRAME: f32 = 1.0 / 60.0;

fn hm(c: &Clock) -> String {
    let (h, m) = c.hour_minute();
    format!("{:02}:{:02}", h, m)
}

/// 실제 `secs`초를 프레임 단위로 흘린다.
fn run(c: &mut Clock, secs: f32) {
    let n = (secs / FRAME).round() as u32;
    for _ in 0..n {
        c.update(FRAME);
    }
}

#[test]
fn starts_at_0600_day_1_nights_0() {
    let c = Clock::new();
    assert_eq!(hm(&c), "06:00");
    assert_eq!(c.phase(), Phase::Day);
    assert_eq!((c.day_number(), c.nights_survived()), (1, 0));
}

#[test]
fn config_has_day_start_and_night_start() {
    assert_eq!((DAY_START_HOUR, NIGHT_START_HOUR), (6, 18));
    assert_eq!((DAY_SECONDS, NIGHT_SECONDS), (300.0, 600.0));
}

#[test]
fn one_real_minute_of_day_is_about_144_game_minutes() {
    let mut c = Clock::new();
    run(&mut c, 60.0);
    // 06:00 + 144분 = 08:24 (프레임 반올림 때문에 1분 모자랄 수 있음)
    let m = c.time_of_day_minutes();
    assert!((503..=504).contains(&m), "실제 1분 뒤 {}", hm(&c));
}

#[test]
fn night_starts_at_1800_and_day_number_stays() {
    let mut c = Clock::new();
    c.update(DAY_SECONDS - 0.01);
    assert_eq!((hm(&c).as_str(), c.phase()), ("17:59", Phase::Day));
    c.update(0.02);
    assert_eq!((hm(&c).as_str(), c.phase()), ("18:00", Phase::Night));
    assert_eq!((c.day_number(), c.nights_survived()), (1, 0));
}

#[test]
fn midnight_wraps_to_0000_without_new_day() {
    let mut c = Clock::new();
    c.update(DAY_SECONDS);
    c.update(NIGHT_SECONDS / 2.0 - 0.6); // 23:59 쯤
    assert_eq!(hm(&c), "23:59");
    c.update(1.0);
    assert_eq!(hm(&c), "00:00");
    assert_eq!(c.phase(), Phase::Night);
    assert_eq!((c.day_number(), c.nights_survived()), (1, 0));
}

#[test]
fn morning_at_0600_is_day_2_then_day_3() {
    let mut c = Clock::new();
    c.update(DAY_SECONDS);
    c.update(NIGHT_SECONDS - 0.01);
    assert_eq!((hm(&c).as_str(), c.phase()), ("05:59", Phase::Night));
    assert_eq!(c.day_number(), 1);
    c.update(0.02);
    assert_eq!((hm(&c).as_str(), c.phase()), ("06:00", Phase::Day));
    assert_eq!((c.day_number(), c.nights_survived()), (2, 1));
    c.update(DAY_SECONDS);
    c.update(NIGHT_SECONDS);
    assert_eq!(hm(&c), "06:00");
    assert_eq!((c.day_number(), c.nights_survived()), (3, 2));
}

/// 빨리감기(dt x 10)로 3바퀴 넘게: 언제나 두 자리 "HH:MM"(00~23, 00~59),
/// 자정 말고는 줄지 않고, 낮이면 06:00~17:59, 밤이면 18:00~05:59.
#[test]
fn fast_forward_four_days_always_valid_and_forward() {
    let mut c = Clock::new();
    let mut prev = c.time_of_day_minutes();
    let mut wraps = 0;
    // 4일 + 1초 (f32 더하기 오차로 마지막 아침 바로 앞에서 멈추지 않게)
    let frames = ((DAY_SECONDS + NIGHT_SECONDS) * 4.0 / (FRAME * 10.0)) as u32 + 6;
    for _ in 0..frames {
        c.update(FRAME * 10.0);
        let (h, m) = c.hour_minute();
        assert!(h < 24 && m < 60, "{}", hm(&c));
        assert_eq!(hm(&c).len(), 5);
        let now = c.time_of_day_minutes();
        if now < prev {
            assert!(prev >= 23 * 60 && now < 60, "거꾸로 감: {} -> {}", prev, now);
            wraps += 1;
        }
        let day_range = DAY_START_HOUR * 60..NIGHT_START_HOUR * 60;
        match c.phase() {
            Phase::Day => assert!(day_range.contains(&now), "낮인데 {}", hm(&c)),
            Phase::Night => assert!(!day_range.contains(&now), "밤인데 {}", hm(&c)),
        }
        prev = now;
    }
    assert_eq!(wraps, 4);
    assert_eq!(c.day_number(), 5);
}

#[test]
fn sunset_and_dawn_start_at_1537_and_0448() {
    let mut c = Clock::new();
    c.update(DAY_SECONDS - 60.0 - 0.5);
    assert_eq!(c.sky_darkness(), 0.0, "노을 전 {}", hm(&c));
    c.update(1.0);
    assert_eq!(hm(&c), "15:37");
    assert!(c.sky_darkness() > 0.0);
    c.update(60.0); // 밤
    c.update(NIGHT_SECONDS - 60.0 - 1.0);
    assert_eq!(c.sky_darkness(), 1.0, "새벽 전 {}", hm(&c));
    c.update(1.0);
    assert_eq!(hm(&c), "04:48");
    c.update(1.0);
    assert!(c.sky_darkness() < 1.0);
}
