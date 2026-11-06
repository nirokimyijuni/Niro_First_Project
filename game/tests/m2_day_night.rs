//! M2 "낮과 밤" 규칙 테스트 (엘리).
//! 기준: stories/backlog.md US-009, US-012, US-013, US-014, US-015 완료 조건 + docs/02-plan.md D1, D11, D13.
//!
//! m1_movement.rs처럼 logic/와 config.rs 소스를 붙여 와서 검사한다.
//! 빨리감기(main.rs의 update_with_time_scale)와 화면 시계 글자(view/hud.rs의 hh_mm)는
//! bin 쪽 코드라 여기서 못 부른다 → 그쪽 단위 테스트(main.rs, hud.rs)와 브라우저에서 확인한다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)] // 이 테스트에서 안 쓰는 logic 함수(체력·번쩍임 등)가 있다
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::clock::{Clock, ClockEvent, Phase};
use logic::coins::Wallet;
use logic::game::{Game, GameEvent};
use logic::player::PlayerInput;
use macroquad::math::Vec2;

const FRAME: f32 = 1.0 / 60.0;
const SEED: u32 = 7;
const NONE: PlayerInput = PlayerInput { left: false, right: false, jump_pressed: false, attack_pressed: false };

/// 구간(낮/밤)에서 남은 초 (올림). 화면에는 안 나오고 낮/밤 길이 확인에만 쓴다.
/// 화면 시계(HH:MM, 06:00에서 시작해 앞으로 감)는 m2_clock_24h.rs가 지킨다 (US-016).
fn shown_seconds(c: &Clock) -> u32 {
    c.seconds_left_in_phase().max(0.0).ceil() as u32
}

/// 아무 코인에도 안 닿는 곳(시작 자리, 땅 위 왼쪽)으로 플레이어를 옮긴다.
fn park_far(g: &mut Game) {
    g.player.pos = Vec2::new(0.0, g.world.ground_y - g.player.size.y);
    g.player.vel = Vec2::ZERO;
}

/// 코인 `i`에 몸을 딱 대 놓는다 (공중에 떠 있어도 한 프레임이면 줍는다).
fn touch_coin(g: &mut Game, i: usize) {
    g.player.pos = g.coins.list[i].rect.point();
    g.player.vel = Vec2::ZERO;
}

// ---------- US-009 시계 ----------

#[test]
fn us009_starts_day_10_00_and_ticks_one_second_per_second() {
    let mut c = Clock::new();
    assert_eq!(c.phase(), Phase::Day);
    assert_eq!(shown_seconds(&c), 600, "10:00");
    // 프레임 단위로 1분 흘리면 09:00
    for _ in 0..60 * 60 {
        c.update(FRAME);
    }
    let s = shown_seconds(&c);
    assert!(s == 540 || s == 541, "1분 뒤 09:00 근처여야 함: {s}");
}

#[test]
fn us009_clock_never_negative_and_never_over_10_00_through_many_cycles() {
    // 빨리감기 프레임(1/60 × 10)으로 10일 돌려도 남은 시간이 0~600 사이, 낮/밤이 번갈아 나온다.
    let mut c = Clock::new();
    let mut events = Vec::new();
    let mut last_phase = c.phase();
    let mut switches = 0;
    for _ in 0..(20 * 60 * 60 * 10 / 10) {
        // 10일 = 12000초 / 10배 = 1200초 = 72000 프레임
        events.extend(c.update(FRAME * DEV_FAST_FORWARD_SCALE));
        let left = c.seconds_left_in_phase();
        assert!((0.0..=600.0).contains(&left), "남은 시간 범위 밖: {left}");
        if c.phase() != last_phase {
            switches += 1;
            last_phase = c.phase();
        }
    }
    assert!(switches >= 19, "10일 동안 낮/밤이 19번 이상 바뀌어야 함: {switches}");
    assert_eq!(events.len(), switches);
    // 이벤트가 밤, 아침, 밤, 아침 … 순서
    for (i, e) in events.iter().enumerate() {
        if i % 2 == 0 {
            assert_eq!(*e, ClockEvent::NightStarted);
        } else {
            assert_eq!(*e, ClockEvent::DayStarted { day_number: (i as u32 + 1) / 2 + 1 });
        }
    }
}

#[test]
fn us009_night_lasts_10_minutes_too() {
    let mut c = Clock::new();
    c.update(DAY_SECONDS);
    assert!(c.is_night());
    assert_eq!(shown_seconds(&c), 600);
    assert!(c.update(NIGHT_SECONDS - 0.5).is_empty());
    assert!(c.is_night());
    assert_eq!(c.update(0.5), vec![ClockEvent::DayStarted { day_number: 2 }]);
}

#[test]
fn us009_clock_runs_while_walking_jumping_or_standing() {
    let inputs = [
        NONE,
        PlayerInput { right: true, ..NONE },
        PlayerInput { jump_pressed: true, ..NONE },
    ];
    for inp in inputs {
        let mut g = Game::new(CharacterKind::Ellie, SEED);
        for _ in 0..60 * 30 {
            g.update(&inp, FRAME);
        }
        let used = DAY_SECONDS - g.clock.seconds_left_in_phase();
        assert!((used - 30.0).abs() < 0.05, "30초 흘러야 함: {used}");
    }
}

// ---------- US-011 하늘 (논리 부분) ----------

#[test]
fn us011_dusk_starts_at_9_minutes_and_dawn_at_19_minutes() {
    let mut c = Clock::new();
    c.update(DAY_SECONDS - DUSK_SECONDS - 0.5);
    assert_eq!(c.sky_darkness(), 0.0, "노을 전에는 한낮");
    c.update(1.0);
    assert!(c.sky_darkness() > 0.0, "01:00 지나면 노을 시작");
    c.update(DUSK_SECONDS); // 밤 0.5초
    assert!(c.is_night());
    c.update(NIGHT_SECONDS - DAWN_SECONDS - 1.0);
    assert_eq!(c.sky_darkness(), 1.0, "새벽 전에는 한밤");
    c.update(1.0);
    assert!(c.sky_darkness() < 1.0, "01:00 지나면 새벽 시작");
}

#[test]
fn us011_dawn_gets_lighter_little_by_little() {
    let mut c = Clock::new();
    c.update(DAY_SECONDS + NIGHT_SECONDS - DAWN_SECONDS);
    let mut last = c.sky_darkness();
    for _ in 0..(DAWN_SECONDS as usize - 1) {
        c.update(1.0);
        let now = c.sky_darkness();
        assert!(now < last, "새벽 동안 계속 밝아진다");
        assert!(last - now < 0.1, "한 번에 확 바뀌지 않는다");
        last = now;
    }
}

// ---------- US-012 N일째 / 버틴 날 ----------

#[test]
fn us012_day_counter_only_goes_up_in_the_morning() {
    let mut c = Clock::new();
    assert_eq!((c.day_number(), c.nights_survived()), (1, 0));
    c.update(DAY_SECONDS); // 밤
    assert!(c.is_night());
    assert_eq!((c.day_number(), c.nights_survived()), (1, 0), "밤이 돼도 그대로");
    c.update(NIGHT_SECONDS); // 아침
    assert_eq!((c.day_number(), c.nights_survived()), (2, 1));
    c.update(DAY_SECONDS);
    assert_eq!((c.day_number(), c.nights_survived()), (2, 1), "낮→밤은 그대로");
    c.update(NIGHT_SECONDS);
    assert_eq!((c.day_number(), c.nights_survived()), (3, 2));
}

// ---------- US-013 코인 줍기 ----------

#[test]
fn us013_three_coins_one_on_each_of_three_boxes() {
    let g = Game::new(CharacterKind::Niro, SEED);
    assert_eq!(g.world.obstacles.len(), 3);
    assert_eq!(g.coins.list.len(), 3);
    for (coin, b) in g.coins.list.iter().zip(&g.world.obstacles) {
        assert!(coin.is_visible());
        assert!(coin.rect.x >= b.x && coin.rect.x + coin.rect.w <= b.x + b.w, "상자 폭 안");
        assert!(coin.rect.y + coin.rect.h <= b.y, "상자 위");
    }
    assert_eq!(g.wallet.coins(), 0);
}

#[test]
fn us013_collect_all_three_by_walking_and_jumping() {
    // 실제 걷기·점프로 세 상자 위에 차례로 올라가 코인 3개를 줍는다.
    let mut g = Game::new(CharacterKind::Niro, SEED);
    for i in 0..3 {
        let b = g.world.obstacles[i];
        let target = b.x + b.w / 2.0 - g.player.size.x / 2.0;
        let mut got = false;
        for _ in 0..60 * 8 {
            let x = g.player.pos.x;
            let on_box = g.player.on_ground && (g.player.pos.y + g.player.size.y - b.y).abs() < 0.5;
            let inp = if on_box && (x - target).abs() < 4.0 {
                NONE
            } else if x < target - 2.0 {
                PlayerInput { right: true, jump_pressed: g.player.on_ground && x > b.x - 90.0, ..NONE }
            } else if x > target + 2.0 {
                PlayerInput { left: true, ..NONE }
            } else {
                PlayerInput { jump_pressed: g.player.on_ground, ..NONE }
            };
            g.update(&inp, FRAME);
            if !g.coins.list[i].is_visible() {
                got = true;
                break;
            }
        }
        assert!(got, "상자 {i} 코인을 못 주움");
        assert_eq!(g.wallet.coins(), i as u32 + 1);
    }
    assert_eq!(g.wallet.coins(), 3);
}

#[test]
fn us013_standing_on_coin_counts_once() {
    let mut g = Game::new(CharacterKind::Logan, SEED);
    touch_coin(&mut g, 0);
    let mut picked = 0;
    for _ in 0..60 * 5 {
        // 5초 동안 코인 자리에 계속 붙어 있는다 (공중이라도 위치를 다시 맞춘다)
        touch_coin(&mut g, 0);
        for e in g.update(&NONE, FRAME) {
            if let GameEvent::CoinsPicked { count } = e {
                picked += count;
            }
        }
    }
    assert_eq!(picked, 1);
    assert_eq!(g.wallet.coins(), 1);
}

#[test]
fn us013_coins_can_be_picked_at_night() {
    let mut g = Game::new(CharacterKind::Ellie, SEED);
    park_far(&mut g);
    g.update(&NONE, DAY_SECONDS + 1.0);
    assert!(g.clock.is_night());
    touch_coin(&mut g, 1);
    g.update(&NONE, FRAME);
    assert_eq!(g.wallet.coins(), 1);
}

#[test]
fn us013_wallet_4999_plus_two_coins_is_5000_and_stays() {
    let mut g = Game::new(CharacterKind::Niro, SEED);
    g.wallet.add(4999);
    touch_coin(&mut g, 0);
    g.update(&NONE, FRAME);
    assert_eq!(g.wallet.coins(), 5000);
    touch_coin(&mut g, 2);
    g.update(&NONE, FRAME);
    assert_eq!(g.wallet.coins(), 5000, "5,001이 되면 안 됨");
    assert!(!g.coins.list[2].is_visible(), "5,000이어도 코인은 사라진다");

    let mut w = Wallet::new();
    for _ in 0..6000 {
        w.add(1);
    }
    assert_eq!(w.coins(), COIN_MAX);
}

// ---------- US-014 다시 생기기 ----------

#[test]
fn us014_each_coin_has_its_own_10_second_timer() {
    let mut g = Game::new(CharacterKind::Niro, SEED);
    // 0초에 코인0, 3초에 코인1, 6초에 코인2를 줍는다.
    let mut picked_at = [0.0f32; 3];
    let mut t = 0.0;
    for i in 0..3 {
        touch_coin(&mut g, i);
        g.update(&NONE, 0.0);
        picked_at[i] = t;
        park_far(&mut g);
        for _ in 0..180 {
            g.update(&NONE, FRAME);
            t += FRAME;
        }
    }
    assert_eq!(g.wallet.coins(), 3);
    // 이제 t ≈ 9초. 각 코인은 자기 주운 때 + 10초에 생긴다.
    let mut back_at = [None::<f32>; 3];
    for _ in 0..60 * 12 {
        g.update(&NONE, FRAME);
        t += FRAME;
        for i in 0..3 {
            if back_at[i].is_none() && g.coins.list[i].is_visible() {
                back_at[i] = Some(t);
            }
        }
    }
    for i in 0..3 {
        let b = back_at[i].expect("다시 생겨야 함");
        let wait = b - picked_at[i];
        assert!((wait - COIN_RESPAWN_SECONDS).abs() < 0.1, "코인 {i}: {wait}초 뒤 생김");
    }
    // 같은 자리, 3개 그대로 (겹쳐 생기지 않음)
    let fresh = Game::new(CharacterKind::Niro, SEED);
    assert_eq!(g.coins.list.len(), 3);
    for (a, b) in g.coins.list.iter().zip(&fresh.coins.list) {
        assert_eq!(a.rect, b.rect);
    }
    assert_eq!(g.wallet.coins(), 3, "멀리 있었으니 더 안 늘어남");
}

#[test]
fn us014_fast_forward_dt_respawns_in_about_one_second() {
    // main.rs의 빨리감기는 코인에 dt × 10을 넣는다 → 실제 1초(60프레임)면 다시 생겨야 한다.
    let mut g = Game::new(CharacterKind::Niro, SEED);
    touch_coin(&mut g, 0);
    g.update(&NONE, 0.0);
    park_far(&mut g);
    let mut frames = 0;
    while !g.coins.list[0].is_visible() {
        g.coins.update(FRAME * DEV_FAST_FORWARD_SCALE, g.player.rect());
        frames += 1;
        assert!(frames < 200);
    }
    assert!((57..=63).contains(&frames), "약 1초(60프레임): {frames}");
}

// ---------- US-015 새 게임 ----------

#[test]
fn us015_new_game_after_night_and_coins_is_fresh() {
    let mut g = Game::new(CharacterKind::Niro, SEED);
    touch_coin(&mut g, 0);
    g.update(&NONE, 0.0);
    g.update(&NONE, DAY_SECONDS * 3.0 + 10.0); // 2일째 밤
    assert!(g.clock.is_night());
    assert_eq!(g.clock.day_number(), 2);
    assert!(g.wallet.coins() >= 1);

    // Esc → 다시 고르기 → 새 게임은 Game::new (main.rs start_play)
    let g = Game::new(CharacterKind::Logan, SEED);
    assert_eq!(g.clock.phase(), Phase::Day);
    assert_eq!(shown_seconds(&g.clock), 600);
    assert_eq!((g.clock.day_number(), g.clock.nights_survived()), (1, 0));
    assert_eq!(g.wallet.coins(), 0);
    assert_eq!(g.clock.sky_darkness(), 0.0);
    assert!(g.coins.list.iter().all(|c| c.is_visible()));
}
