//! M8 깜짝 미션 시험 (수산나): 낮 미션(US-084), 밤 미션(US-085), 로건이 그릴 상태(US-086).
//! 기준: stories/backlog.md [임시-49]~[임시-55], docs/02-plan.md D81·D103.
//! 숫자는 config 상수로 적어서 임시 값을 바꿔도 시험을 고치지 않아도 된다. 반복은 모두 횟수가 정해져 있다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::coins::Coin;
use logic::combat::CombatEvent;
use logic::game::{Game, GameEvent, GatherEvent};
use logic::gathering::Gathering;
use logic::math::MathQuestionMaker;
use logic::mission::{mission_fail_text, mission_success_text, MissionEvent, MissionKind, MissionState, Missions};
use logic::monster::Zombie;
use logic::player::{Facing, PlayerInput};
use logic::shop::{Armor, SELLABLE};
use logic::skeleton::{Skeleton, SkeletonStage};
use logic::weapon::Weapons;
use logic::world::World;
use macroquad::math::{Rect, Vec2};

const FRAME: f32 = 1.0 / 60.0;
const SEED: u32 = 7;
const A_LITTLE: f32 = 0.05;
const MANY_SEEDS: u32 = 1000;

fn idle() -> PlayerInput {
    PlayerInput::default()
}

fn attack() -> PlayerInput {
    PlayerInput { attack_pressed: true, ..Default::default() }
}

/// 벽 두 개 사이에 갇힌 짧은 맵. 몬스터가 닿지 않아 며칠을 넘겨도 쓰러지지 않는다 (m6_math와 같다).
fn quiet_game(seed: u32) -> Game {
    let wall = |x| Rect::new(x, 0.0, 10.0, GROUND_Y);
    let walled = World { width: WORLD_WIDTH, ground_y: GROUND_Y, obstacles: vec![wall(100.0), wall(180.0)] };
    let mut g = Game::on_world(CharacterKind::Niro, seed, walled);
    g.combat.skeleton_chance = 0.0;
    g
}

/// `kind` 미션만 켜고 지금 그 미션 하나를 낸다 (낮 종류면 낮 미션, 밤 종류면 밤 미션).
fn only_mission(g: &mut Game, kind: MissionKind) {
    g.missions.turn_off_all();
    g.missions.turned_on = vec![kind];
    let day = g.clock.day_number();
    if kind.is_day_mission() {
        g.missions.give_day_mission(day);
    } else {
        g.missions.give_night_mission(day);
    }
    g.take_mission_events();
}

fn answer_right_and_close(g: &mut Game) {
    let answer = g.math.question().expect("문제가 열려 있다").answer;
    g.math.type_and_submit(answer);
    g.update(&idle(), MATH_RIGHT_SHOW_SECONDS + A_LITTLE);
    assert!(!g.math.is_open());
}

/// 다음 18:00이나 06:00을 넘긴다. 밤 문제가 열리면 맞혀서 닫고 한 프레임 더 간다 (밤 미션은 이때 나온다).
fn to_next_phase(g: &mut Game) -> Vec<GameEvent> {
    g.combat.zombies.list.clear();
    g.combat.skeletons.clear();
    let left = g.clock.seconds_left_in_phase();
    let mut events = g.update(&idle(), left + A_LITTLE);
    if g.is_waiting_to_open_quiz() {
        events.extend(g.update(&idle(), QUIZ_WAIT_AFTER_DAY_MISSION_FAIL_SECONDS + A_LITTLE));
    }
    if g.math.is_open() {
        answer_right_and_close(g);
    }
    events.extend(g.update(&idle(), FRAME));
    events
}

/// 몸에 닿는 새 코인 하나를 놓고 줍는다. 맵의 다른 코인은 치운다.
fn pick_one_coin(g: &mut Game) -> Vec<GameEvent> {
    g.coins.list = vec![Coin::above_box(Rect::new(600.0, GROUND_Y - 48.0, 48.0, 48.0))];
    let rect = g.coins.list[0].rect;
    g.player.pos = Vec2::new(rect.x, rect.y);
    g.update(&idle(), 0.0)
}

fn mission_count(g: &Game) -> u32 {
    g.missions.current().expect("미션이 있다").count
}

fn mission_state(g: &Game) -> MissionState {
    g.missions.current().expect("미션이 있다").state
}

fn appeared(events: &[MissionEvent]) -> Vec<MissionKind> {
    events
        .iter()
        .filter_map(|e| match e {
            MissionEvent::Appeared { kind, .. } => Some(*kind),
            _ => None,
        })
        .collect()
}

fn hurt_amount(events: &[GameEvent]) -> u32 {
    events
        .iter()
        .map(|e| match e {
            GameEvent::Combat(CombatEvent::PlayerHurt { amount, .. }) => *amount,
            _ => 0,
        })
        .sum()
}

/// 체력을 `hp`로 맞춘다.
fn set_hp(g: &mut Game, hp: u32) {
    let now = g.combat.health.hp();
    assert!(hp <= now);
    g.combat.health.lose_from_quiz(now - hp);
}

fn zombie_in_front(g: &mut Game) {
    let mut zombie = Zombie::new(g.player.rect().right() + 5.0, GROUND_Y, false);
    zombie.hp = 1;
    g.combat.zombies.list.push(zombie);
    g.player.facing = Facing::Right;
}

/// 밤이 된 짧은 맵. 낮 미션은 없고, 밤 문제가 닫힌 뒤 `kind` 밤 미션이 나와 있다.
fn night_game(kind: MissionKind) -> Game {
    night_game_on(quiet_game(SEED), kind)
}

/// `night_game`과 같지만 맵을 고른다 (쿵은 긴 맵 해변에서만 된다).
fn night_game_on(mut g: Game, kind: MissionKind) -> Game {
    g.missions.turn_off_all();
    g.missions.turned_on = vec![kind];
    let left = g.clock.seconds_left_in_phase();
    g.update(&idle(), left + A_LITTLE);
    answer_right_and_close(&mut g);
    g.update(&idle(), FRAME); // 밤 미션이 나온다. 싸움도 밤인 줄 안다 (아침 체력 채우기)
    assert!(g.clock.is_night());
    assert_eq!(g.missions.going_kind(), Some(kind));
    g.combat.zombies.list.clear();
    g.combat.skeletons.clear();
    g.take_mission_events();
    g
}

// ---- US-084 낮 미션 ----

#[test]
fn new_game_has_day_1_mission_then_one_every_morning_for_7_days() {
    let mut g = quiet_game(SEED);
    let first = g.take_mission_events();
    assert_eq!(appeared(&first).len(), 1, "새 게임에 바로 하나");
    assert!(appeared(&first)[0].is_day_mission());
    let mut day_missions = 1;
    let mut night_missions = 0;
    for _ in 0..DAYS_TO_CLEAR * 2 {
        to_next_phase(&mut g);
        for kind in appeared(&g.take_mission_events()) {
            if kind.is_day_mission() {
                day_missions += 1;
            } else {
                night_missions += 1;
            }
        }
    }
    assert!(g.is_cleared());
    assert_eq!((day_missions, night_missions), (DAYS_TO_CLEAR, DAYS_TO_CLEAR), "7일 동안 낮 7번, 밤 7번");
}

#[test]
fn day_goals_follow_the_mission_table() {
    let goal = |kind: MissionKind, day| kind.goal_on_day(day);
    use MissionKind::*;
    assert_eq!([goal(Coins, 1), goal(Trees, 1), goal(Ores, 1), goal(FindMiss, 1)], [10, 3, 2, 1]);
    assert_eq!([goal(Coins, 2), goal(Trees, 2), goal(Ores, 2)], [12, 4, 3]);
    assert_eq!([goal(Coins, 99), goal(Trees, 99), goal(Ores, 99), goal(FindMiss, 99)], [20, 6, 5, 1]);
}

#[test]
fn goals_come_from_config() {
    let table = [
        (MissionKind::Coins, MISSION_COINS_START, MISSION_COINS_PER_DAY, MISSION_COINS_MAX),
        (MissionKind::Trees, MISSION_TREES_START, MISSION_TREES_PER_DAY, MISSION_TREES_MAX),
        (MissionKind::Ores, MISSION_ORES_START, MISSION_ORES_PER_DAY, MISSION_ORES_MAX),
        (MissionKind::FindMiss, MISSION_FIND_MISS_START, MISSION_FIND_MISS_PER_DAY, MISSION_FIND_MISS_MAX),
        (MissionKind::Monsters, MISSION_MONSTERS_START, MISSION_MONSTERS_PER_DAY, MISSION_MONSTERS_MAX),
        (MissionKind::KeepHealth, MISSION_KEEP_HEALTH_START, MISSION_KEEP_HEALTH_PER_DAY, MISSION_KEEP_HEALTH_MAX),
    ];
    for (kind, start, per_day, max) in table {
        for day in 1..=10 {
            assert_eq!(kind.goal_on_day(day), (start + per_day * (day - 1)).min(max), "{kind:?} {day}일째");
        }
    }
    assert_eq!(MISSION_REWARD_COINS, 100);
    assert_eq!(MISSION_FAIL_HP_LOSS, 1);
}

/// 같은 차례(낮1, 밤1, 낮2, …)로 7일 동안 뽑은 종류.
fn seven_days_of(seed: u32) -> Vec<MissionKind> {
    let mut missions = Missions::new(seed);
    let mut kinds = Vec::new();
    for day in 1..=DAYS_TO_CLEAR {
        missions.give_day_mission(day);
        kinds.push(missions.current().unwrap().kind);
        missions.end_day_mission();
        missions.give_night_mission(day);
        kinds.push(missions.current().unwrap().kind);
        missions.end_night_mission(PLAYER_START_HP);
    }
    kinds
}

#[test]
fn thousand_seeds_show_every_kind_and_never_repeat_yesterday() {
    let mut seen = Vec::new();
    for seed in 0..MANY_SEEDS {
        let kinds = seven_days_of(seed);
        let days: Vec<MissionKind> = kinds.iter().step_by(2).copied().collect();
        let nights: Vec<MissionKind> = kinds.iter().skip(1).step_by(2).copied().collect();
        assert!(days.iter().all(|k| k.is_day_mission()) && nights.iter().all(|k| !k.is_day_mission()));
        assert!(days.windows(2).all(|w| w[0] != w[1]), "시드 {seed}: 어제 낮과 같은 종류");
        assert!(nights.windows(2).all(|w| w[0] != w[1]), "시드 {seed}: 어젯밤과 같은 종류");
        for kind in kinds {
            if !seen.contains(&kind) {
                seen.push(kind);
            }
        }
    }
    for kind in MissionKind::DAY_KINDS.into_iter().chain(MissionKind::NIGHT_KINDS) {
        assert!(seen.contains(&kind), "{kind:?}가 한 번도 안 나왔다");
    }
}

#[test]
fn same_seed_same_missions_in_same_order() {
    for seed in [0, 1, 7, 12345] {
        assert_eq!(seven_days_of(seed), seven_days_of(seed));
    }
    // 게임 안에서 나오는 차례도 같다 (첫 낮 미션).
    let a = Game::new(CharacterKind::Niro, 3).missions.current().copied();
    let b = Game::new(CharacterKind::Ellie, 3).missions.current().copied();
    assert_eq!(a, b);
    assert_eq!(a.unwrap().kind, seven_days_of(3)[0]);
}

#[test]
fn mission_draws_do_not_change_boxes_miss_weapons_ores_or_math() {
    for seed in [1, 7, 42] {
        let plain = Game::new(CharacterKind::Niro, seed);
        let mut drawn = Game::new(CharacterKind::Niro, seed);
        for day in 1..=20 {
            drawn.missions.give_day_mission(day);
            drawn.missions.give_night_mission(day);
        }
        // 상자 자리: 미션을 넣기 전과 같은 맵 만들기 그대로.
        assert_eq!(plain.world.obstacles, World::long_map(seed).obstacles);
        assert_eq!(drawn.world.obstacles, plain.world.obstacles);

        // 미스 무기: 떨어지는 차례가 미션 없는 Weapons와 같다.
        let world = World::long_map(seed);
        let mut fresh = Weapons::new(seed);
        let (mut a, mut b) = (plain, drawn);
        for _ in 0..3 {
            fresh.drop_one(500.0, &world);
            a.weapons.drop_one(500.0, &world);
            b.weapons.drop_one(500.0, &world);
        }
        let kinds = |w: &Weapons| w.dropped.iter().map(|d| d.kind).collect::<Vec<_>>();
        assert_eq!(kinds(&a.weapons), kinds(&fresh));
        assert_eq!(kinds(&b.weapons), kinds(&fresh));

        // 광석: 덩이를 다 캐면 나오는 광물 차례가 같다.
        let mut fresh_gathering = Gathering::on_world(&world, &World::cave(), seed);
        for _ in 0..5 {
            let lump = fresh_gathering.ore_lumps[0].rect;
            let mine = |gathering: &mut Gathering| {
                gathering.start_mining(0);
                gathering.keep_mining(1000.0, true, lump).map(|(material, _)| material)
            };
            let expected = mine(&mut fresh_gathering);
            assert!(expected.is_some());
            assert_eq!(mine(&mut a.gathering), expected);
            assert_eq!(mine(&mut b.gathering), expected);
        }

        // 수학 문제: 미션 없는 문제 만들기와 같은 차례.
        let mut maker = MathQuestionMaker::new(seed);
        for _ in 0..5 {
            let expected = maker.next();
            for g in [&mut a, &mut b] {
                g.math.add_waiting();
                g.math.open_next_waiting();
                assert_eq!(g.math.question().cloned(), expected);
                answer_right_and_close(g);
            }
        }
    }
}

#[test]
fn only_counts_after_the_mission_appears() {
    let mut g = Game::on_world(CharacterKind::Niro, SEED, World::m1());
    g.missions.turn_off_all();
    pick_one_coin(&mut g);
    pick_one_coin(&mut g);
    assert_eq!(g.coins.picked_total, 2);
    g.missions.turned_on = vec![MissionKind::Coins];
    g.missions.give_day_mission(1);
    assert_eq!(mission_count(&g), 0, "나오기 전에 주운 코인은 세지 않는다");
    pick_one_coin(&mut g);
    assert_eq!(mission_count(&g), 1);
}

#[test]
fn coins_count_box_coins_only_even_at_5000() {
    let mut g = Game::on_world(CharacterKind::Niro, SEED, World::m1());
    only_mission(&mut g, MissionKind::Coins);
    // 광물을 팔아 받은 코인은 세지 않는다.
    g.gathering.bag.add(SELLABLE[0]);
    g.sell(SELLABLE[0]).unwrap();
    assert_eq!(mission_count(&g), 0);
    // 코인이 5,000이어서 지갑이 그대로여도 주우면 1 센다.
    g.wallet.add(COIN_MAX);
    pick_one_coin(&mut g);
    assert_eq!(g.wallet.coins(), COIN_MAX);
    assert_eq!(mission_count(&g), 1);
}

/// 긴 맵에서 `kind` 미션만 켠 게임. 시민은 치워서 펑이 없게 한다.
fn long_map_game(kind: MissionKind) -> Game {
    let mut g = Game::new(CharacterKind::Niro, SEED);
    g.citizens.list.clear();
    only_mission(&mut g, kind);
    g
}

/// 몸 가운데를 `rect` 가운데에 두고 땅에 세운다.
fn stand_at(g: &mut Game, rect: Rect) {
    g.player.pos = Vec2::new(rect.center().x - g.player.size.x / 2.0, GROUND_Y - g.player.size.y);
    g.player.vel = Vec2::ZERO;
}

/// 캐기 버튼을 `seconds`초 누르고 뗀다. 다 캔 횟수를 돌려준다.
fn hold_mine(g: &mut Game, seconds: f32) -> usize {
    let mut mined = 0;
    g.mine_held = true;
    for _ in 0..(seconds / FRAME).ceil() as usize {
        g.update(&idle(), FRAME);
        mined += g.take_gather_events().iter().filter(|e| matches!(e, GatherEvent::Mined { .. })).count();
    }
    g.mine_held = false;
    g.update(&idle(), FRAME);
    mined
}

#[test]
fn trees_count_only_when_fully_mined() {
    let mut g = long_map_game(MissionKind::Trees);
    let tree = g.gathering.trees[0].rect;
    stand_at(&mut g, tree);
    let needed = g.gathering.bag.mine_seconds();
    assert_eq!(hold_mine(&mut g, needed / 2.0), 0, "캐다가 그만두면");
    assert_eq!(mission_count(&g), 0, "세지 않는다");
    assert_eq!(hold_mine(&mut g, needed + 0.5), 1);
    assert_eq!(mission_count(&g), 1);
}

#[test]
fn ores_count_one_whatever_comes_out() {
    let mut g = long_map_game(MissionKind::Ores);
    g.in_cave = true;
    let needed = g.gathering.bag.mine_seconds();
    for index in 0..2 {
        let lump = g.gathering.ore_lumps[index].rect;
        stand_at(&mut g, lump);
        assert_eq!(hold_mine(&mut g, needed + 0.5), 1);
    }
    assert_eq!(mission_count(&g), 2);
    assert_eq!(mission_state(&g), MissionState::Succeeded, "1일째 광석은 2개");
}

#[test]
fn finding_miss_once_is_success() {
    let mut g = long_map_game(MissionKind::FindMiss);
    let before = g.wallet.coins();
    let miss = g.miss.as_ref().expect("긴 맵에는 미스가 있다");
    assert!(miss.is_hiding());
    let rect = miss.rect(&g.world);
    g.player.pos = Vec2::new(rect.x, rect.y);
    let events = g.update(&idle(), 0.0);
    assert!(events.iter().any(|e| matches!(e, GameEvent::MissFound { .. })));
    assert_eq!(mission_state(&g), MissionState::Succeeded);
    assert_eq!(g.wallet.coins(), before + MISSION_REWARD_COINS);
}

#[test]
fn reaching_n_is_instant_success_with_100_coins_and_no_more_counting() {
    let mut g = Game::on_world(CharacterKind::Niro, SEED, World::m1());
    only_mission(&mut g, MissionKind::Coins);
    let goal = MissionKind::Coins.goal_on_day(1);
    for _ in 0..goal - 1 {
        pick_one_coin(&mut g);
    }
    assert_eq!(mission_state(&g), MissionState::Going);
    assert!(g.take_mission_events().is_empty());
    pick_one_coin(&mut g);
    assert_eq!(g.take_mission_events(), vec![MissionEvent::Succeeded { kind: MissionKind::Coins }]);
    assert_eq!(g.wallet.coins(), goal + MISSION_REWARD_COINS);
    assert_eq!(g.coins.picked_total, goal + MISSION_REWARD_COINS, "보상 코인은 점수의 획득한 동전 수에 든다");
    assert_eq!(g.score(), (goal + MISSION_REWARD_COINS) * SCORE_PER_COIN);
    assert!(g.mission_now().is_none(), "끝난 뒤에는 진행 글이 없다");
    // 더 주워도 세지 않고 보상도 없다. 그날 낮에 새 미션도 없다.
    pick_one_coin(&mut g);
    assert_eq!(mission_count(&g), goal);
    assert_eq!(g.wallet.coins(), goal + 1 + MISSION_REWARD_COINS);
    g.update(&idle(), g.clock.seconds_left_in_phase() - 1.0);
    assert!(g.take_mission_events().is_empty());
}

#[test]
fn reward_stops_at_5000_but_score_gets_100() {
    for (start, after) in [(COIN_MAX - 50, COIN_MAX), (COIN_MAX, COIN_MAX)] {
        let mut g = Game::on_world(CharacterKind::Niro, SEED, World::m1());
        only_mission(&mut g, MissionKind::Coins);
        g.wallet.add(start - 1);
        let goal = MissionKind::Coins.goal_on_day(1);
        for _ in 0..goal {
            pick_one_coin(&mut g);
        }
        assert_eq!(mission_state(&g), MissionState::Succeeded);
        assert_eq!(g.wallet.coins(), after);
        assert_eq!(g.coins.picked_total, goal + MISSION_REWARD_COINS);
    }
}

#[test]
fn day_mission_fails_at_1800_before_the_quiz_and_costs_1_hp() {
    let mut g = quiet_game(SEED);
    let (coins, score) = (g.wallet.coins(), g.score());
    g.take_mission_events();
    let left = g.clock.seconds_left_in_phase();
    let events = g.update(&idle(), left + A_LITTLE);
    let mission_events = g.take_mission_events();
    assert!(matches!(mission_events[0], MissionEvent::Failed { .. }));
    assert_eq!(hurt_amount(&events), MISSION_FAIL_HP_LOSS);
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP - MISSION_FAIL_HP_LOSS);
    let hurt_at = events.iter().position(|e| matches!(e, GameEvent::Combat(CombatEvent::PlayerHurt { .. }))).unwrap();
    let night_at = events.iter().position(|e| *e == GameEvent::NightStarted).unwrap();
    assert!(hurt_at < night_at, "낮 미션이 먼저 끝난다");
    // BUG-034 (부모님 결정 2026-11-15): 실패 글을 먼저 2초 보여 주고 그다음 잠깐 퀴즈.
    assert!(!g.math.is_open(), "실패 글이 먼저다");
    assert!(g.is_waiting_to_open_quiz());
    g.update(&idle(), QUIZ_WAIT_AFTER_DAY_MISSION_FAIL_SECONDS + A_LITTLE);
    assert!(g.math.is_open(), "그다음 잠깐 퀴즈");
    assert_eq!((g.wallet.coins(), g.score()), (coins, score), "코인과 점수는 그대로");
}

// ---- BUG-034 낮 미션 실패 글 먼저, 그다음 퀴즈 ----

/// 몬스터를 막는 벽이 없는 맵 (BUG-025처럼 몬스터가 나오는지 보려고).
fn open_game() -> Game {
    let world = World { width: WORLD_WIDTH, ground_y: GROUND_Y, obstacles: vec![] };
    Game::on_world(CharacterKind::Niro, 3, world)
}

/// 18:00 직전 1초까지 보내고 몬스터를 치운 뒤, 한 프레임씩 18:00이 올 때까지 간다 (최대 2초).
fn step_to_six_pm(g: &mut Game) -> Vec<GameEvent> {
    g.update(&idle(), g.clock.seconds_left_in_phase() - 1.0);
    g.combat.zombies.list.clear();
    g.combat.skeletons.clear();
    for _ in 0..(2.0 / FRAME) as usize {
        let events = g.update(&idle(), FRAME);
        if events.contains(&GameEvent::NightStarted) {
            return events;
        }
    }
    panic!("2초 안에 18:00이 오지 않았다");
}

fn monsters_on_map(g: &Game) -> usize {
    g.combat.zombies.list.len() + g.combat.skeletons.list.len()
}

#[test]
fn bug034_failed_day_mission_waits_2_seconds_before_the_quiz() {
    assert_eq!(QUIZ_WAIT_AFTER_DAY_MISSION_FAIL_SECONDS, MISSION_FAIL_SHOW_SECONDS);
    let mut g = open_game();
    g.take_mission_events();
    step_to_six_pm(&mut g);
    assert!(g.take_mission_events().iter().any(|e| matches!(e, MissionEvent::Failed { .. })));
    let clock_left = g.clock.seconds_left_in_phase();
    let hp = g.combat.health.hp();

    // 2초 바로 전까지 한 프레임씩: 퀴즈 없음, 몬스터 없음, 시계는 18:00에 멈춤, 밤 미션도 아직.
    let frames_before = ((QUIZ_WAIT_AFTER_DAY_MISSION_FAIL_SECONDS - A_LITTLE) / FRAME) as usize;
    for _ in 0..frames_before {
        g.update(&idle(), FRAME);
        assert!(!g.math.is_open(), "2초 동안은 퀴즈가 열리지 않는다");
        assert!(g.is_waiting_to_open_quiz());
        assert_eq!(monsters_on_map(&g), 0, "퀴즈보다 먼저 몬스터가 나오지 않는다 (BUG-025)");
    }
    assert_eq!(g.clock.seconds_left_in_phase(), clock_left, "시계는 18:00에서 멈춰 있다");
    assert_eq!(g.combat.health.hp(), hp);
    assert!(appeared(&g.take_mission_events()).is_empty(), "밤 미션은 퀴즈 뒤에 나온다");

    // 조금 더 가면 퀴즈가 열린다. 그때까지도 몬스터는 없다.
    for _ in 0..(1.0 / FRAME) as usize {
        g.update(&idle(), FRAME);
        if g.math.is_open() {
            break;
        }
    }
    assert!(g.math.is_open(), "2초 뒤 잠깐 퀴즈가 열린다");
    assert!(!g.is_waiting_to_open_quiz());
    assert_eq!(monsters_on_map(&g), 0);

    // 퀴즈를 닫은 뒤에 밤 미션이 나온다 (지금과 같다).
    answer_right_and_close(&mut g);
    g.update(&idle(), FRAME);
    assert_eq!(appeared(&g.take_mission_events()).len(), 1);
}

#[test]
fn bug034_succeeded_or_no_day_mission_opens_the_quiz_right_away() {
    for has_mission in [true, false] {
        let mut g = open_game();
        if has_mission {
            only_mission(&mut g, MissionKind::Coins);
            let goal = g.missions.current().unwrap().goal;
            assert!(g.missions.add(MissionKind::Coins, goal), "낮 미션 성공");
        } else {
            g.missions.turn_off_all();
        }
        step_to_six_pm(&mut g);
        assert!(!g.is_waiting_to_open_quiz());
        assert!(g.math.is_open(), "18:00에 바로 퀴즈");
        assert_eq!(monsters_on_map(&g), 0);
    }
}

#[test]
fn bug034_wait_uses_real_seconds_even_in_fast_forward() {
    let mut g = quiet_game(SEED);
    g.update(&idle(), g.clock.seconds_left_in_phase() + A_LITTLE);
    assert!(g.is_waiting_to_open_quiz());
    g.update_with_time_scale(&idle(), 1.0, DEV_FAST_FORWARD_SCALE);
    assert!(!g.math.is_open(), "빨리감기여도 실제 1초는 아직이다");
    g.update_with_time_scale(&idle(), 1.0 + A_LITTLE, DEV_FAST_FORWARD_SCALE);
    assert!(g.math.is_open());
}

#[test]
fn failing_with_1_hp_goes_down_and_no_quiz_opens() {
    let mut g = quiet_game(SEED);
    set_hp(&mut g, 1);
    let left = g.clock.seconds_left_in_phase();
    let events = g.update(&idle(), left + A_LITTLE);
    assert!(events.contains(&GameEvent::Combat(CombatEvent::PlayerDown)));
    assert!(g.combat.health.is_down() && g.is_over());
    assert!(!g.math.is_open());
    assert!(g.mission_now().is_none(), "쓰러진 화면에는 미션이 없다");
}

#[test]
fn armor_does_not_stop_the_fail_hp_loss() {
    let mut g = quiet_game(SEED);
    g.shop.put_on_armor(Armor::Diamond);
    let left = g.clock.seconds_left_in_phase();
    g.update(&idle(), left + A_LITTLE);
    assert!(g.shop.armor().is_some(), "18:00에도 갑옷을 입고 있다");
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP - MISSION_FAIL_HP_LOSS);
}

#[test]
fn shop_crafting_and_quiz_stop_mission_time() {
    for open in 0..3 {
        let mut g = quiet_game(SEED);
        match open {
            0 => g.shop.open(),
            1 => g.crafting.open(),
            _ => {
                g.math.add_waiting();
                g.math.open_next_waiting();
            }
        }
        for _ in 0..3 {
            g.update(&idle(), DAY_SECONDS);
        }
        assert!(!g.clock.is_night(), "18:00이 오지 않는다");
        assert!(g.mission_now().is_some(), "미션이 끝나지 않는다");
        assert_eq!(g.combat.health.hp(), PLAYER_START_HP);
    }
}

#[test]
fn fast_forward_counts_the_same() {
    let mut g = Game::on_world(CharacterKind::Niro, SEED, World::m1());
    only_mission(&mut g, MissionKind::Coins);
    g.coins.list = vec![Coin::above_box(Rect::new(600.0, GROUND_Y - 48.0, 48.0, 48.0))];
    let rect = g.coins.list[0].rect;
    g.player.pos = Vec2::new(rect.x, rect.y);
    g.update_with_time_scale(&idle(), FRAME, DEV_FAST_FORWARD_SCALE);
    assert_eq!(mission_count(&g), 1);
}

#[test]
fn new_game_starts_missions_from_the_beginning() {
    let mut g = quiet_game(SEED);
    set_hp(&mut g, 1);
    g.update(&idle(), g.clock.seconds_left_in_phase() + A_LITTLE);
    assert!(g.is_over());
    let again = quiet_game(SEED);
    let mission = again.mission_now().expect("새 게임 1일째 낮 미션");
    assert_eq!((mission.day, mission.count, mission.state), (1, 0, MissionState::Going));
}

#[test]
fn turned_off_kind_never_appears() {
    for seed in 0..MANY_SEEDS {
        let mut missions = Missions::new(seed);
        missions.turned_on.retain(|k| *k != MissionKind::Coins && *k != MissionKind::KeepHealth);
        for day in 1..=DAYS_TO_CLEAR {
            missions.give_day_mission(day);
            assert_ne!(missions.current().unwrap().kind, MissionKind::Coins);
            missions.give_night_mission(day);
            assert_eq!(missions.current().unwrap().kind, MissionKind::Monsters, "밤에 남은 종류는 하나뿐이라 매일 같다");
        }
    }
    let mut none = Missions::new(1);
    none.turned_on.clear();
    none.give_day_mission(1);
    assert!(none.current().is_none());
}

#[test]
fn practice_map_without_miss_has_no_find_miss_mission() {
    for seed in 0..200 {
        let g = Game::on_world(CharacterKind::Niro, seed, World::m1());
        assert!(!g.missions.turned_on.contains(&MissionKind::FindMiss));
    }
}

// ---- US-085 밤 미션 ----

#[test]
fn night_mission_comes_after_the_quiz_closes_once_per_night() {
    let mut g = quiet_game(SEED);
    g.take_mission_events();
    let left = g.clock.seconds_left_in_phase();
    g.update(&idle(), left + A_LITTLE);
    // 낮 미션 실패 글을 먼저 보여 준다 (BUG-034).
    g.update(&idle(), QUIZ_WAIT_AFTER_DAY_MISSION_FAIL_SECONDS + A_LITTLE);
    assert!(g.math.is_open());
    g.take_mission_events();
    g.update(&idle(), 1.0);
    assert!(appeared(&g.take_mission_events()).is_empty(), "퀴즈가 열린 동안은 나오지 않는다");
    answer_right_and_close(&mut g);
    g.update(&idle(), FRAME);
    let kinds = appeared(&g.take_mission_events());
    assert_eq!(kinds.len(), 1);
    assert!(!kinds[0].is_day_mission());
    for _ in 0..30 {
        g.update(&idle(), FRAME);
    }
    assert!(appeared(&g.take_mission_events()).is_empty(), "같은 밤에 두 번 나오지 않는다");
}

#[test]
fn night_goals_follow_the_mission_table() {
    use MissionKind::*;
    assert_eq!([Monsters.goal_on_day(1), Monsters.goal_on_day(2), Monsters.goal_on_day(99)], [5, 6, 10]);
    assert_eq!([KeepHealth.goal_on_day(1), KeepHealth.goal_on_day(7)], [4, 4]);
}

#[test]
fn monsters_count_hands_poison_and_skeleton_touch() {
    let mut g = night_game(MissionKind::Monsters);
    // 맨손으로
    zombie_in_front(&mut g);
    g.update(&attack(), FRAME);
    assert_eq!(mission_count(&g), 1);
    // 독약으로 (밤에 화면 끝에서 나온 좀비는 치운다)
    g.combat.zombies.list.clear();
    let mut zombie = Zombie::new(g.player.rect().right() + 40.0, GROUND_Y, false);
    zombie.hp = 1;
    g.combat.zombies.list.push(zombie);
    assert_eq!(g.combat.spray_poison(g.player.rect().center()), 1);
    for _ in 0..((POISON_TICK_SECONDS + 0.5) / FRAME) as usize {
        g.update(&idle(), FRAME);
        if g.combat.zombies.list.is_empty() {
            break;
        }
    }
    assert_eq!(mission_count(&g), 2);
    // 스켈레톤에 닿아 쓰러뜨리기
    let body = g.player.rect();
    g.combat.skeletons.list.push(Skeleton::new(body.x, GROUND_Y, SkeletonStage::Normal));
    g.update(&idle(), FRAME);
    assert_eq!(mission_count(&g), 3);
}

#[test]
fn citizen_blast_and_sweeps_do_not_count() {
    let mut g = night_game(MissionKind::Monsters);
    // 폭발 시민
    zombie_in_front(&mut g);
    let at = g.combat.zombies.list[0].rect().center();
    let far = Rect::new(at.x + 2000.0, 0.0, 10.0, 10.0);
    let blast = g.combat.citizen_blast(at, far);
    assert!(blast.iter().any(|e| matches!(e, CombatEvent::ZombieDefeated { .. })));
    g.update(&idle(), FRAME);
    assert_eq!(mission_count(&g), 0);
    // 주사기총 (한꺼번에 쓰러뜨리기)
    zombie_in_front(&mut g);
    zombie_in_front(&mut g);
    g.gathering.bag.syringe_guns = 1;
    let (given_before, defeated_before) = (g.combat.monsters_given_experience, g.combat.monsters_defeated);
    assert!(g.fire_syringe_gun());
    g.update(&idle(), FRAME);
    assert!(g.combat.zombies.list.is_empty());
    assert!(g.combat.monsters_defeated > defeated_before, "주사기총 점수는 들어간다");
    assert_eq!(g.combat.monsters_given_experience, given_before, "미션이 세는 수는 그대로");
    assert_eq!(mission_count(&g), 0);

    // 쿵 (긴 맵 해변, 엔더열매 힘)
    let mut g = night_game_on(Game::new(CharacterKind::Niro, SEED), MissionKind::Monsters);
    let beach_x = (SLAM_BEACH_LEFT + SLAM_BEACH_RIGHT) / 2.0;
    g.player.pos = Vec2::new(beach_x - PLAYER_WIDTH / 2.0, GROUND_Y - PLAYER_HEIGHT);
    g.player.vel = Vec2::ZERO;
    g.gathering.bag.has_ender_power = true;
    assert!(g.slam_power_here().is_some());
    g.combat.zombies.list.push(Zombie::new(beach_x + 200.0, GROUND_Y, false));
    let (given_before, defeated_before) = (g.combat.monsters_given_experience, g.combat.monsters_defeated);
    g.mine_held = true;
    g.update(&idle(), FRAME);
    assert!(g.combat.zombies.list.is_empty(), "쿵으로 쓰러졌다");
    assert!(g.combat.monsters_defeated > defeated_before, "쿵 점수는 들어간다");
    assert_eq!(g.combat.monsters_given_experience, given_before, "미션이 세는 수는 그대로");
    assert_eq!(mission_count(&g), 0);
}

#[test]
fn n_monsters_is_instant_success_with_100_coins() {
    let mut g = night_game(MissionKind::Monsters);
    let goal = MissionKind::Monsters.goal_on_day(1);
    let coins = g.wallet.coins();
    for _ in 0..goal {
        zombie_in_front(&mut g);
        g.update(&attack(), FRAME);
    }
    assert_eq!(g.take_mission_events(), vec![MissionEvent::Succeeded { kind: MissionKind::Monsters }]);
    assert_eq!(g.wallet.coins(), coins + MISSION_REWARD_COINS);
    assert!(g.clock.is_night(), "06:00까지 기다리지 않는다");
}

/// 06:00을 넘기고 이번 업데이트의 이벤트를 돌려준다.
fn to_morning(g: &mut Game) -> Vec<GameEvent> {
    g.combat.zombies.list.clear();
    let left = g.clock.seconds_left_in_phase();
    g.update(&idle(), left + A_LITTLE)
}

#[test]
fn keep_health_looks_at_hp_before_the_morning_refill() {
    let goal = MissionKind::KeepHealth.goal_on_day(1);
    // 체력 3으로 06:00 → 실패. 체력이 찬 뒤 다시 많아져도 실패다.
    let mut g = night_game(MissionKind::KeepHealth);
    set_hp(&mut g, goal - 1);
    let events = to_morning(&mut g);
    let mission_events = g.take_mission_events();
    assert_eq!(mission_events, vec![MissionEvent::Failed { kind: MissionKind::KeepHealth }]);
    let hurt_at = events.iter().position(|e| matches!(e, GameEvent::Combat(CombatEvent::PlayerHurt { .. }))).unwrap();
    let refill_at = events.iter().position(|e| *e == GameEvent::Combat(CombatEvent::HealthRefilled)).unwrap();
    assert!(hurt_at < refill_at, "실패 체력 -1이 아침 채우기보다 먼저");
    assert_eq!(g.combat.health.hp(), g.combat.health.max_hp());

    // 체력 4로 06:00 → 성공, 코인 100.
    let mut g = night_game(MissionKind::KeepHealth);
    set_hp(&mut g, goal);
    let coins = g.wallet.coins();
    let events = to_morning(&mut g);
    assert_eq!(g.take_mission_events()[0], MissionEvent::Succeeded { kind: MissionKind::KeepHealth });
    assert_eq!(hurt_amount(&events), 0);
    assert_eq!(g.wallet.coins(), coins + MISSION_REWARD_COINS);
}

#[test]
fn night_fail_costs_1_hp_then_refill_and_can_go_down_at_1() {
    let mut g = night_game(MissionKind::Monsters);
    let (coins, score) = (g.wallet.coins(), g.score());
    let events = to_morning(&mut g);
    assert_eq!(hurt_amount(&events), MISSION_FAIL_HP_LOSS);
    assert_eq!(g.combat.health.hp(), g.combat.health.max_hp(), "그 뒤 아침 채우기");
    assert_eq!((g.wallet.coins(), g.score()), (coins, score));

    let mut g = night_game(MissionKind::Monsters);
    set_hp(&mut g, 1);
    let events = to_morning(&mut g);
    assert!(events.contains(&GameEvent::Combat(CombatEvent::PlayerDown)));
    assert!(!events.contains(&GameEvent::Combat(CombatEvent::HealthRefilled)), "채우기 전에 쓰러졌다");
    assert!(g.is_over());
    assert!(appeared(&g.take_mission_events()).is_empty(), "쓰러진 뒤 새 낮 미션은 없다");
}

/// 7일째 밤 미션이 나온 게임. 밤 7번째 미션은 `kind`로 정한다.
fn seventh_night(kind: MissionKind) -> Game {
    let mut g = quiet_game(SEED);
    for _ in 0..DAYS_TO_CLEAR * 2 - 1 {
        if g.clock.day_number() == DAYS_TO_CLEAR && !g.clock.is_night() {
            g.missions.turned_on = vec![kind];
        }
        to_next_phase(&mut g);
    }
    assert!(g.clock.is_night() && g.clock.day_number() == DAYS_TO_CLEAR);
    assert_eq!(g.missions.going_kind(), Some(kind));
    g.take_mission_events();
    g
}

#[test]
fn clear_morning_success_gives_100_coins_into_clear_score() {
    let mut g = seventh_night(MissionKind::KeepHealth);
    let (coins, earned) = (g.wallet.coins(), g.coins.picked_total);
    let events = to_morning(&mut g);
    assert!(events.contains(&GameEvent::GameCleared));
    assert_eq!(g.wallet.coins(), coins + MISSION_REWARD_COINS);
    assert_eq!(g.coins.picked_total, earned + MISSION_REWARD_COINS);
    assert!(appeared(&g.take_mission_events()).is_empty(), "클리어 아침에는 새 미션이 없다");
}

#[test]
fn clear_morning_failure_does_not_cost_hp() {
    let mut g = seventh_night(MissionKind::Monsters);
    set_hp(&mut g, 1);
    let events = to_morning(&mut g);
    assert_eq!(hurt_amount(&events), 0);
    assert!(g.is_cleared(), "쓰러지지 않고 클리어");
    assert!(!g.combat.health.is_down());
}

#[test]
fn going_down_at_night_ends_the_mission() {
    let mut g = night_game(MissionKind::Monsters);
    assert!(g.mission_now().is_some());
    set_hp(&mut g, 0);
    assert!(g.mission_now().is_none());
}

// ---- US-086 로건이 그릴 상태와 글 ----

#[test]
fn mission_now_and_texts_for_the_screen() {
    let mut g = Game::on_world(CharacterKind::Niro, SEED, World::m1());
    only_mission(&mut g, MissionKind::Coins);
    pick_one_coin(&mut g);
    let now = g.mission_now().unwrap();
    assert_eq!((now.kind, now.count, now.goal, now.state), (MissionKind::Coins, 1, 10, MissionState::Going));
    assert_eq!(now.progress_text(), "깜짝 미션! 코인 1/10 (밤 전까지)");
    assert_eq!(MissionKind::Coins.miss_says_text(MissionKind::Coins.goal_on_day(2)), "깜짝 미션이야! 밤이 되기 전에 코인 12개를 모아 줘!");
    assert_eq!(MissionKind::FindMiss.progress_text(0, 1), "깜짝 미션! 미스 찾기 0/1 (밤 전까지)");
    assert_eq!(MissionKind::Monsters.progress_text(2, 5), "깜짝 미션! 몬스터 2/5 (아침 전까지)");
    assert_eq!(mission_success_text(), "미션 성공! 코인 100개를 받았어요!");
    assert_eq!(mission_fail_text(), "아쉽다! 다음 미션도 해 보자");
    assert_eq!((MISSION_MISS_SAYS_SECONDS, MISSION_SUCCESS_SHOW_SECONDS, MISSION_FAIL_SHOW_SECONDS), (3.0, 3.0, 2.0));

    // 체력 지키기는 지금 체력이 보인다.
    let mut g = night_game(MissionKind::KeepHealth);
    set_hp(&mut g, 6);
    let now = g.mission_now().unwrap();
    assert_eq!(now.progress_text(), "깜짝 미션! 체력 4 이상 지키기, 지금 6 (아침까지)");
}

#[test]
fn appeared_event_carries_todays_goal() {
    let mut g = quiet_game(SEED);
    let first = g.take_mission_events();
    let MissionEvent::Appeared { kind, goal } = first[0] else { panic!("나옴이 먼저") };
    assert_eq!(goal, kind.goal_on_day(1));
    let m = g.mission_now().unwrap();
    assert_eq!(m.kind.miss_says_text(m.goal), kind.miss_says_text(goal));
}

#[test]
fn texts_have_no_scary_words() {
    let mut texts: Vec<String> = Vec::new();
    for kind in MissionKind::DAY_KINDS.into_iter().chain(MissionKind::NIGHT_KINDS) {
        texts.push(kind.miss_says_text(5));
        texts.push(kind.progress_text(1, 5));
    }
    texts.push(mission_success_text());
    texts.push(mission_fail_text().to_string());
    for text in texts {
        assert!(!text.contains("잡") && !text.contains("죽"), "{text}");
        assert!(!text.contains('{'), "빈칸이 남았다: {text}");
    }
    assert!(MissionKind::Monsters.miss_says_text(5).contains("물리쳐 줘"));
}
