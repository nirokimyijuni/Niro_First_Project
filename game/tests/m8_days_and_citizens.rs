//! 부모님 요청 2026-11-21 시험 (수산나):
//! 1. 날이 지날수록 밤에 몬스터가 많아진다 ([임시] 날별 표는 config.rs).
//! 2. 일반 시민(미스 시민)이 상자에 끼지 않는다.
//! 숫자는 config 상수로 적고, 반복은 모두 횟수가 정해져 있다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::boxes::BoxMonster;
use logic::character::CharacterKind;
use logic::combat::{Combat, CombatEvent, MonstersOnDay};
use logic::game::Game;
use logic::mission::MissionKind;
use logic::player::PlayerInput;
use logic::world::{really_overlaps, World};
use macroquad::math::Rect;

const FRAME: f32 = 1.0 / 60.0;
const SEED: u32 = 7;

// ---- 1. 날별 몬스터 ----

#[test]
fn monsters_on_each_day_follow_the_table() {
    // (날, 간격, 한 번에, 화면 최대, 좀비 최대). config.rs 표와 같다.
    let table = [
        (1, 10.0, 2, 6, 4),
        (2, 9.3, 2, 7, 4),
        (3, 8.6, 2, 8, 5),
        (4, 7.9, 3, 9, 6),
        (5, 7.2, 3, 10, 6),
        (6, 6.5, 3, 11, 7),
        (7, 6.0, 4, 12, 8),
        (8, 6.0, 4, 12, 8),
        (30, 6.0, 4, 12, 8),
    ];
    for (day, seconds, per_spawn, max, zombies) in table {
        let today = MonstersOnDay::for_day(day);
        assert!((today.spawn_seconds - seconds).abs() < 0.001, "{day}일째 {today:?}");
        assert_eq!((today.per_spawn, today.max_on_screen, today.max_zombies), (per_spawn, max, zombies), "{day}일째");
    }
}

#[test]
fn first_day_is_the_same_as_before() {
    let first = MonstersOnDay::for_day(1);
    assert_eq!(first.spawn_seconds, ZOMBIE_SPAWN_SECONDS);
    assert_eq!(first.per_spawn, MONSTERS_PER_SPAWN);
    assert_eq!(first.max_on_screen, MONSTER_MAX_ON_SCREEN);
    assert_eq!(first.max_zombies, ZOMBIE_MAX_ON_SCREEN);
    assert_eq!(MonstersOnDay::for_day(0), first, "0일째는 없지만 1일째로 본다");
}

#[test]
fn monsters_never_get_fewer_as_days_pass() {
    for day in 1..=DAYS_TO_CLEAR + 3 {
        let (today, next) = (MonstersOnDay::for_day(day), MonstersOnDay::for_day(day + 1));
        assert!(next.spawn_seconds <= today.spawn_seconds && next.spawn_seconds >= SPAWN_SECONDS_SHORTEST);
        assert!(next.per_spawn >= today.per_spawn && next.per_spawn <= MONSTERS_PER_SPAWN_MOST);
        assert!(next.max_on_screen >= today.max_on_screen && next.max_on_screen <= MONSTER_MAX_MOST);
        assert!(next.max_zombies >= today.max_zombies && next.max_zombies <= next.max_on_screen);
        assert!(next.per_spawn <= next.max_on_screen);
    }
}

#[test]
fn monster_mission_fits_each_night() {
    // 깜짝 미션 "몬스터 N마리": 그날 화면 최대보다 크지 않고, 밤 동안 나올 수 있는 수보다도 많지 않다.
    for day in 1..=DAYS_TO_CLEAR {
        let n = MissionKind::Monsters.goal_on_day(day) as usize;
        let today = MonstersOnDay::for_day(day);
        assert!(n <= today.max_on_screen, "{day}일째 밤 N {n} > 화면 최대 {}", today.max_on_screen);
        let turns = (NIGHT_SECONDS / today.spawn_seconds) as usize;
        assert!(n <= turns * today.per_spawn, "{day}일째 밤에 다 나오지 못한다");
    }
}

/// 땅만 있는 넓은 맵.
fn open_world() -> World {
    World { width: 3000.0, ground_y: GROUND_Y, obstacles: Vec::new() }
}

/// 밤에 `seconds`초 돌리며 몬스터를 하나도 쓰러뜨리지 않는다. 돌려주는 값: 가장 많았던 (모두, 좀비), 나온 차례 시각들.
fn run_night(c: &mut Combat, seconds: f32) -> (usize, usize, Vec<(f32, usize)>) {
    // 주인공은 아주 멀리, 공중에 둔다 (몬스터가 닿지 않게). 쓰러지면 안 되므로 갑옷을 입힌다.
    c.health.set_armored(true);
    let player = Rect::new(1500.0, -10_000.0, PLAYER_WIDTH, PLAYER_HEIGHT);
    let world = open_world();
    let (mut most, mut most_zombies, mut turns) = (0, 0, Vec::new());
    for frame in 0..(seconds / FRAME) as usize {
        let events = c.update(FRAME, true, player, &world);
        let appeared = events
            .iter()
            .filter(|e| matches!(e, CombatEvent::ZombieAppeared { .. } | CombatEvent::SkeletonAppeared { .. }))
            .count();
        if appeared > 0 {
            turns.push((frame as f32 * FRAME, appeared));
        }
        most = most.max(c.zombies.list.len() + c.skeletons.list.len());
        most_zombies = most_zombies.max(c.zombies.list.len());
    }
    (most, most_zombies, turns)
}

#[test]
fn later_nights_have_more_monsters_and_faster() {
    for day in [1, 4, 7] {
        let today = MonstersOnDay::for_day(day);
        let mut c = Combat::new(SEED);
        c.day_number = day;
        let (most, most_zombies, turns) = run_night(&mut c, 60.0);
        assert_eq!(most, today.max_on_screen, "{day}일째: 화면 최대까지 찬다");
        assert!(most_zombies <= today.max_zombies, "{day}일째");
        assert_eq!(turns[0].1, today.per_spawn, "{day}일째: 첫 차례에 한 번에 나오는 수");
        assert!((turns[1].0 - turns[0].0 - today.spawn_seconds).abs() < 0.1, "{day}일째 간격 {turns:?}");
    }
}

#[test]
fn box_monsters_use_the_same_limit_as_the_day() {
    for day in [1, 7] {
        let today = MonstersOnDay::for_day(day);
        let mut c = Combat::new(SEED);
        c.day_number = day;
        let mut appeared = 0;
        for i in 0..today.max_on_screen + 5 {
            if c.spawn_from_box(100.0 + i as f32 * 60.0, GROUND_Y, BoxMonster::Zombie).is_some() {
                appeared += 1;
            }
        }
        assert_eq!(appeared, today.max_on_screen, "{day}일째: 상자 몬스터도 그날 화면 최대까지만");
        assert_eq!(c.zombies.list.len(), today.max_zombies, "{day}일째: 좀비가 차면 스켈레톤이 나온다");
    }
}

#[test]
fn game_tells_combat_which_day_it_is() {
    let mut g = Game::new(CharacterKind::Niro, SEED);
    g.missions.turn_off_all();
    g.update(&PlayerInput::default(), FRAME);
    assert_eq!(g.combat.day_number, 1);
    g.clock.update(DAY_SECONDS + NIGHT_SECONDS + DAY_SECONDS + NIGHT_SECONDS);
    g.update(&PlayerInput::default(), FRAME);
    assert_eq!(g.clock.day_number(), 3);
    assert_eq!(g.combat.day_number, 3);
    assert_eq!(g.combat.monsters_today(), MonstersOnDay::for_day(3));
}

// ---- 2. 일반 시민이 상자에 끼지 않는다 ----

/// 서 있는 상자(맵의 막는 것) 중 일반 시민과 겹치는 것이 있으면 그 (시민 번호, 상자).
fn walker_stuck_in_wall(g: &Game) -> Option<(usize, Rect)> {
    g.citizens.walkers.iter().enumerate().find_map(|(i, walker)| {
        g.world.monster_walls().find(|wall| really_overlaps(*wall, walker.rect)).map(|wall| (i, wall))
    })
}

/// `walker` 시민과 같은 장소에 있는, 땅 위에 바로 놓인 부술 수 있는 상자 번호.
fn ground_box_in_place_of(g: &Game, walker: usize) -> Option<usize> {
    let place = (g.citizens.walkers[walker].rect.center().x / SCREEN_WIDTH) as usize;
    let place_left = place as f32 * SCREEN_WIDTH;
    g.boxes.list.iter().position(|b| {
        b.rect.y + b.rect.h == g.world.ground_y
            && place_left <= b.rect.x
            && b.rect.x + b.rect.w <= place_left + SCREEN_WIDTH
    })
}

#[test]
fn box_coming_back_at_six_am_waits_for_a_walking_citizen() {
    // 재현 (REVIEW-021 못 본 것): 부서진 상자 자리에 일반 시민이 서 있을 때 06:00 뒤 상자가 돌아오면 시민이 상자 안에 낀다.
    let mut tried = 0;
    for seed in 0..40 {
        let mut g = Game::new(CharacterKind::Niro, seed);
        g.missions.turn_off_all();
        g.combat.health.set_armored(true);
        for walker in 0..g.citizens.walkers.len() {
            let Some(index) = ground_box_in_place_of(&g, walker) else { continue };
            tried += 1;
            let rect = g.boxes.list[index].rect;
            let mut events = Vec::new();
            g.break_box(index, &mut events);
            // 시민이 빈 상자 자리 가운데에 걸어 들어와 있다.
            g.citizens.walkers[walker].rect.x = rect.center().x - g.citizens.walkers[walker].rect.w / 2.0;
            g.boxes.morning_came();
            g.update(&PlayerInput::default(), FRAME);
            assert_eq!(walker_stuck_in_wall(&g), None, "seed {seed}, 시민 {walker}: 상자가 시민 위에 돌아왔다");
        }
    }
    assert!(tried >= 10, "시험해 본 경우가 너무 적다: {tried}");
}

#[test]
fn box_comes_back_after_the_walking_citizen_steps_away() {
    let mut g = Game::new(CharacterKind::Niro, SEED);
    g.missions.turn_off_all();
    g.combat.health.set_armored(true);
    let walker = (0..g.citizens.walkers.len()).find(|w| ground_box_in_place_of(&g, *w).is_some()).unwrap();
    let index = ground_box_in_place_of(&g, walker).unwrap();
    let rect = g.boxes.list[index].rect;
    let mut events = Vec::new();
    g.break_box(index, &mut events);
    g.citizens.walkers[walker].rect.x = rect.center().x - g.citizens.walkers[walker].rect.w / 2.0;
    g.boxes.morning_came();
    // 걷기 6초·멈춤 2초를 몇 번 지나는 동안 (정해진 프레임 수) 상자는 시민이 비켜난 뒤에만 돌아온다.
    let mut came_back = false;
    for _ in 0..(60.0 / FRAME) as usize {
        g.update(&PlayerInput::default(), FRAME);
        assert_eq!(walker_stuck_in_wall(&g), None);
        came_back |= g.world.obstacles.contains(&rect);
    }
    assert!(came_back, "시민이 비켜나면 상자가 돌아온다");
}

#[test]
fn walking_citizen_inside_a_box_steps_out() {
    // 어쩌다 이미 겹쳐 있어도 (낀 채로는 양쪽이 다 막혀 못 움직였다) 바로 가장 가까운 빈 땅으로 비켜 선다.
    for seed in 0..20 {
        let mut g = Game::new(CharacterKind::Niro, seed);
        g.missions.turn_off_all();
        g.combat.health.set_armored(true);
        for walker in 0..g.citizens.walkers.len() {
            let Some(index) = ground_box_in_place_of(&g, walker) else { continue };
            let rect = g.boxes.list[index].rect;
            g.citizens.walkers[walker].rect.x = rect.center().x - g.citizens.walkers[walker].rect.w / 2.0;
            assert!(walker_stuck_in_wall(&g).is_some());
            g.update(&PlayerInput::default(), FRAME);
            assert_eq!(walker_stuck_in_wall(&g), None, "seed {seed}, 시민 {walker}");
            let place = (rect.center().x / SCREEN_WIDTH) as usize as f32;
            let body = g.citizens.walkers[walker].rect;
            assert!(place * SCREEN_WIDTH <= body.x && body.x + body.w <= (place + 1.0) * SCREEN_WIDTH, "자기 장소 안");
            assert!((body.center().x - rect.center().x).abs() <= rect.w / 2.0 + body.w / 2.0 + 16.0, "가까운 곳으로: {body:?} {rect:?}");
        }
    }
}

#[test]
fn walking_citizens_never_start_or_walk_inside_a_box() {
    // 처음 자리와 걷는 길 (D99 상자 자리 포함): 여러 seed에서 상자를 모두 부수고 06:00을 지나며 걸어도 끼지 않는다.
    for seed in 0..12 {
        let mut g = Game::new(CharacterKind::Niro, seed);
        g.missions.turn_off_all();
        g.combat.health.set_armored(true);
        assert_eq!(walker_stuck_in_wall(&g), None, "seed {seed}: 처음 자리");
        let mut events = Vec::new();
        for index in 0..g.boxes.list.len() {
            g.break_box(index, &mut events);
        }
        for second in 0..40 {
            if second == 20 {
                g.boxes.morning_came();
            }
            for _ in 0..60 {
                g.update(&PlayerInput::default(), FRAME);
                assert_eq!(walker_stuck_in_wall(&g), None, "seed {seed}, {second}초");
            }
        }
    }
}

#[test]
fn walker_body_starts_at_the_same_place_as_before() {
    // 처음 자리 찾기를 함수로 나눴어도 자리는 그대로다: 장소 가운데에서 가장 가까운 빈 땅.
    let g = Game::new(CharacterKind::Niro, SEED);
    for (place, walker) in g.citizens.walkers.iter().enumerate() {
        let place_left = place as f32 * SCREEN_WIDTH;
        assert!(place_left <= walker.rect.x && walker.rect.x + walker.rect.w <= place_left + SCREEN_WIDTH);
        assert_eq!(walker.rect.y + walker.rect.h, g.world.ground_y);
    }
}
