//! M4 좀비 카스테라 규칙 테스트 (수산나).
//! 기준: stories/backlog.md US-039~US-043, docs/02-plan.md D2, D3, D27, D32.

#[path = "../src/config.rs"]
mod config;
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::combat::{attack_box, Combat, CombatEvent};
use logic::game::{Game, GameEvent};
use logic::monster::{Zombie, ZombieState};
use logic::player::{Facing, PlayerInput};
use logic::world::World;
use macroquad::math::Rect;

const FRAME: f32 = 1.0 / 60.0;
const SEED: u32 = 7;

fn flat_world() -> World {
    World { width: WORLD_WIDTH, ground_y: GROUND_Y, obstacles: vec![] }
}

/// 땅 위 x에 서 있는 플레이어의 몸.
fn player_at(x: f32) -> Rect {
    Rect::new(x, GROUND_Y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT)
}

/// `seconds`초를 프레임씩 흘린다.
fn run(c: &mut Combat, seconds: f32, is_night: bool, player: Rect, world: &World) -> Vec<CombatEvent> {
    let mut events = Vec::new();
    for _ in 0..(seconds / FRAME).round() as usize {
        events.extend(c.update(FRAME, is_night, player, world));
    }
    events
}

/// 좀비 한 마리만 놓고 시작한다. 다음 좀비가 끼어들지 않게 스폰 시간을 먼저 흘린다.
fn night_with_one_zombie(x: f32, will_float: bool) -> Combat {
    let mut c = Combat::new(SEED);
    run(&mut c, FRAME, true, player_at(480.0), &flat_world());
    c.zombies.list.clear();
    c.zombies.list.push(Zombie::new(x, GROUND_Y, will_float));
    c
}

fn count(events: &[CombatEvent], wanted: fn(&CombatEvent) -> bool) -> usize {
    events.iter().filter(|e| wanted(e)).count()
}

// ---- US-039 나타나고 다가오기 ----

#[test]
fn no_zombies_in_the_daytime() {
    let mut c = Combat::new(SEED);
    let events = run(&mut c, 60.0, false, player_at(480.0), &flat_world());
    assert!(c.zombies.list.is_empty());
    assert!(events.is_empty());
}

#[test]
fn first_zombie_appears_at_screen_edge_as_night_starts() {
    let w = flat_world();
    let mut c = Combat::new(SEED);
    let events = c.update(FRAME, true, player_at(480.0), &w);
    assert_eq!(c.zombies.list.len(), 1);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::ZombieAppeared { .. })), 1);
    let z = c.zombies.list[0].rect();
    assert!(z.x < 5.0 || z.x + z.w > w.width - 5.0, "화면 끝에서 나온다: x={}", z.x);
    assert_eq!(z.y + z.h, GROUND_Y, "발이 땅에 있다");
}

/// 플레이어(x=480)의 양옆을 벽으로 막은 맵. 좀비가 플레이어에게 닿지 못한다.
fn walled_world() -> World {
    let mut w = flat_world();
    w.obstacles.push(Rect::new(300.0, 0.0, 10.0, GROUND_Y));
    w.obstacles.push(Rect::new(650.0, 0.0, 10.0, GROUND_Y));
    w
}

#[test]
fn one_zombie_every_20_seconds() {
    let w = walled_world();
    let player = player_at(480.0);
    let appeared = |events: &[CombatEvent]| count(events, |e| matches!(e, CombatEvent::ZombieAppeared { .. }));
    let mut c = Combat::new(SEED);
    assert_eq!(appeared(&run(&mut c, 1.0, true, player, &w)), 1);
    assert_eq!(appeared(&run(&mut c, 18.0, true, player, &w)), 0);
    assert_eq!(appeared(&run(&mut c, 2.0, true, player, &w)), 1);
}

#[test]
fn never_more_than_3_zombies() {
    let w = walled_world();
    let player = player_at(480.0);
    let mut c = Combat::new(SEED);
    let mut most = 0;
    for _ in 0..(NIGHT_SECONDS / FRAME) as usize {
        c.update(FRAME, true, player, &w);
        most = most.max(c.zombies.list.len());
    }
    assert_eq!(most, ZOMBIE_MAX_ON_SCREEN);
}

#[test]
fn after_a_full_screen_next_zombie_comes_20_seconds_after_one_is_gone() {
    let w = walled_world();
    let player = player_at(480.0);
    let mut c = night_with_one_zombie(0.0, false);
    c.zombies.list.push(Zombie::new(0.0, GROUND_Y, false));
    c.zombies.list.push(Zombie::new(0.0, GROUND_Y, false));
    run(&mut c, 30.0, true, player, &w);
    assert_eq!(c.zombies.list.len(), 3, "꽉 차 있으면 시간을 세지 않는다");

    c.zombies.list.pop();
    run(&mut c, ZOMBIE_SPAWN_SECONDS - 1.0, true, player, &w);
    assert_eq!(c.zombies.list.len(), 2);
    run(&mut c, 2.0, true, player, &w);
    assert_eq!(c.zombies.list.len(), 3);
}

#[test]
fn zombie_walks_toward_player_slower_than_player() {
    assert!(ZOMBIE_WALK_SPEED < PLAYER_WALK_SPEED);
    let w = flat_world();
    let mut c = night_with_one_zombie(0.0, false);
    run(&mut c, 1.0, true, player_at(600.0), &w);
    let moved = c.zombies.list[0].pos.x;
    assert!((moved - ZOMBIE_WALK_SPEED).abs() < 2.0, "1초에 걷는 속도만큼: {moved}");

    let mut c = night_with_one_zombie(800.0, false);
    run(&mut c, 1.0, true, player_at(100.0), &w);
    assert!(c.zombies.list[0].pos.x < 800.0, "왼쪽에 있으면 왼쪽으로");
}

#[test]
fn zombie_is_stopped_by_a_box_and_stays_on_the_ground() {
    let w = World::m1();
    let box_ = w.obstacles[0];
    let mut c = night_with_one_zombie(0.0, false);
    run(&mut c, 10.0, true, player_at(480.0), &w);
    let z = c.zombies.list[0].rect();
    assert_eq!(z.x + z.w, box_.x, "상자 왼쪽 면에서 멈춘다");
    assert_eq!(z.y + z.h, GROUND_Y);
    assert_eq!(c.health.hp(), PLAYER_START_HP);
}

#[test]
fn long_frame_does_not_pass_through_a_box() {
    let w = World::m1();
    let box_ = w.obstacles[0];
    let mut c = night_with_one_zombie(box_.x - ZOMBIE_WIDTH - 10.0, false);
    c.update(2.5, true, player_at(480.0), &w);
    assert_eq!(c.zombies.list[0].pos.x + ZOMBIE_WIDTH, box_.x);
}

#[test]
fn zombie_next_to_a_box_does_not_jump_across_it() {
    let w = World::m1();
    let box_ = w.obstacles[0];
    // 상자 오른쪽 면에 붙어 있고, 플레이어도 그 자리에 있다.
    let x = box_.x + box_.w;
    let mut c = night_with_one_zombie(x, false);
    run(&mut c, 0.5, true, player_at(x), &w);
    assert_eq!(c.zombies.list[0].pos.x, x);
}

#[test]
fn zombie_does_not_appear_right_next_to_the_player() {
    let w = flat_world();
    for seed in 0..50 {
        for player_x in [0.0, w.width - PLAYER_WIDTH] {
            let player = player_at(player_x);
            let mut c = Combat::new(seed);
            c.update(FRAME, true, player, &w);
            let gap = (c.zombies.list[0].rect().center().x - player.center().x).abs();
            assert!(gap > ZOMBIE_SPAWN_MIN_DISTANCE, "seed {seed}: {gap}");
        }
    }
}

#[test]
fn both_screen_edges_are_used() {
    let w = flat_world();
    let lefts = (0..100)
        .filter(|&seed| {
            let mut c = Combat::new(seed);
            c.update(FRAME, true, player_at(480.0), &w);
            c.zombies.list[0].pos.x < 5.0 // 나온 프레임에 벌써 조금 걸었다
        })
        .count();
    assert!((20..=80).contains(&lefts), "왼쪽 {lefts}마리");
}

#[test]
fn morning_removes_zombies_and_next_night_starts_with_a_new_one() {
    let w = flat_world();
    let player = player_at(480.0);
    let mut c = night_with_one_zombie(0.0, false);
    c.update(FRAME, false, player, &w);
    assert!(c.zombies.list.is_empty());
    run(&mut c, 30.0, false, player, &w);
    assert!(c.zombies.list.is_empty());
    c.update(FRAME, true, player, &w);
    assert_eq!(c.zombies.list.len(), 1);
}

// ---- US-040 공격 ----

/// 플레이어 오른쪽에 바짝 붙기 직전의 좀비 위치.
fn x_in_reach_on_right(player: Rect) -> f32 {
    player.x + player.w + PLAYER_ATTACK_RANGE - 10.0
}

#[test]
fn three_hits_defeat_a_zombie() {
    assert_eq!(ZOMBIE_HP, 3);
    let player = player_at(300.0);
    let mut c = night_with_one_zombie(x_in_reach_on_right(player), false);

    let events = c.player_attack(player, Facing::Right);
    assert!(matches!(events[..], [CombatEvent::ZombieHit { hp_left: 2, .. }]));
    assert!(c.zombies.list[0].is_flashing(), "맞으면 반짝인다");

    let events = c.player_attack(player, Facing::Right);
    assert!(matches!(events[..], [CombatEvent::ZombieHit { hp_left: 1, .. }]));

    let events = c.player_attack(player, Facing::Right);
    assert!(matches!(events[..], [CombatEvent::ZombieDefeated { .. }]));
    assert!(c.zombies.list.is_empty());
}

#[test]
fn flash_ends_soon() {
    let player = player_at(300.0);
    let mut c = night_with_one_zombie(x_in_reach_on_right(player), false);
    c.player_attack(player, Facing::Right);
    run(&mut c, ZOMBIE_HIT_FLASH_SECONDS + 0.05, true, player, &flat_world());
    assert!(!c.zombies.list[0].is_flashing());
}

#[test]
fn far_zombie_is_not_hit() {
    let player = player_at(300.0);
    let mut c = night_with_one_zombie(player.x + player.w + PLAYER_ATTACK_RANGE + 1.0, false);
    assert!(c.player_attack(player, Facing::Right).is_empty());
    assert_eq!(c.zombies.list[0].hp, ZOMBIE_HP);
}

#[test]
fn zombie_behind_the_player_is_not_hit() {
    let player = player_at(300.0);
    let mut c = night_with_one_zombie(x_in_reach_on_right(player), false);
    assert!(c.player_attack(player, Facing::Left).is_empty());

    // 몸이 겹쳐 있어도 등 뒤는 맞지 않는다.
    c.zombies.list[0].pos.x = player.x + 10.0;
    assert!(c.player_attack(player, Facing::Left).is_empty());
    assert_eq!(c.player_attack(player, Facing::Right).len(), 1);
}

#[test]
fn attack_box_is_in_front_of_the_player() {
    let player = player_at(300.0);
    let right = attack_box(player, Facing::Right);
    assert_eq!((right.x, right.w), (player.x + player.w, PLAYER_ATTACK_RANGE));
    let left = attack_box(player, Facing::Left);
    assert_eq!(left.x + left.w, player.x);
}

#[test]
fn one_attack_hits_every_zombie_in_reach() {
    let player = player_at(300.0);
    let x = x_in_reach_on_right(player);
    let mut c = night_with_one_zombie(x, false);
    c.zombies.list.push(Zombie::new(x, GROUND_Y, false));
    assert_eq!(c.player_attack(player, Facing::Right).len(), 2);
}

#[test]
fn zombie_hp_stays_3_at_higher_levels() {
    let mut c = night_with_one_zombie(0.0, false);
    c.level_up();
    c.level_up();
    assert_eq!(c.zombies.list[0].hp, ZOMBIE_HP);
    assert_eq!(Zombie::new(0.0, GROUND_Y, false).hp, 3);
}

// ---- US-041 체력 ----

#[test]
fn player_starts_with_7_and_gains_5_per_level() {
    let mut c = Combat::new(SEED);
    assert_eq!((c.health.hp(), c.health.max_hp(), c.level), (7, 7, 1));
    c.level_up();
    assert_eq!(c.health.max_hp(), 12);
    c.level_up();
    assert_eq!((c.health.max_hp(), c.level), (17, 3));
}

#[test]
fn touching_a_zombie_takes_2_hp_then_1_second_of_safety() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = night_with_one_zombie(player.x, false);

    let events = c.update(FRAME, true, player, &w);
    assert!(events.contains(&CombatEvent::PlayerHurt { amount: 2, hp_left: 5 }));
    assert!(c.health.is_invincible());

    let events = run(&mut c, PLAYER_HURT_INVINCIBLE_SECONDS - 0.1, true, player, &w);
    assert_eq!(c.health.hp(), 5, "1초 동안은 다시 맞지 않는다");
    assert!(events.is_empty());
}

#[test]
fn hp_stops_at_0_and_game_over_comes_once() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = night_with_one_zombie(player.x, false);
    c.zombies.list.push(Zombie::new(player.x, GROUND_Y, false));

    let events = run(&mut c, 5.0, true, player, &w);
    assert_eq!(c.health.hp(), 0, "7 - 2 - 2 - 2 - 2는 0에서 멈춘다");
    assert!(c.health.is_down());
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::PlayerHurt { .. })), 4);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::PlayerDown)), 1);
}

#[test]
fn after_going_down_nothing_moves() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = night_with_one_zombie(player.x, false);
    run(&mut c, 5.0, true, player, &w);
    assert!(c.health.is_down());

    let far = player_at(900.0);
    let x_before = c.zombies.list[0].pos.x;
    assert!(run(&mut c, 30.0, true, far, &w).is_empty());
    assert_eq!(c.zombies.list[0].pos.x, x_before);
    assert!(run(&mut c, 1.0, false, far, &w).is_empty(), "아침이 와도 다시 일어나지 않는다");
    assert_eq!(c.health.hp(), 0);
    assert!(c.player_attack(player, Facing::Right).is_empty());
}

#[test]
fn zombie_attack_stays_2_at_higher_levels_d32() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = night_with_one_zombie(player.x, false);
    c.level_up();
    c.level_up();
    let events = c.update(FRAME, true, player, &w);
    assert!(events.contains(&CombatEvent::PlayerHurt { amount: ZOMBIE_ATTACK, hp_left: 5 }));
}

#[test]
fn morning_refills_health_d27() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = night_with_one_zombie(player.x, false);
    c.update(FRAME, true, player, &w);
    assert_eq!(c.health.hp(), 5);

    let events = c.update(FRAME, false, player, &w);
    assert_eq!(c.health.hp(), PLAYER_START_HP);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::HealthRefilled)), 1);
    assert!(run(&mut c, 5.0, false, player, &w).is_empty(), "낮 동안 또 차지 않는다");
}

#[test]
fn morning_refills_up_to_the_grown_max() {
    let w = flat_world();
    let mut c = night_with_one_zombie(0.0, false);
    c.level_up();
    c.update(FRAME, false, player_at(480.0), &w);
    assert_eq!(c.health.hp(), 12);
}

// ---- US-042 날아오른 좀비 ----

#[test]
fn floating_zombie_rises_then_pops_without_hurting_anyone() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = night_with_one_zombie(0.0, true);
    let far = player_at(900.0);
    run(&mut c, ZOMBIE_FLOAT_START_SECONDS - 0.1, true, far, &w);
    assert_eq!(c.zombies.list[0].state, ZombieState::Walking);
    run(&mut c, 0.2, true, far, &w);
    assert_eq!(c.zombies.list[0].state, ZombieState::Floating);

    // 떠오른 좀비 바로 아래로 플레이어가 와도 맞지 않는다.
    c.zombies.list[0].pos.x = player.x;
    let ground_top = GROUND_Y - ZOMBIE_HEIGHT;
    let events = run(&mut c, 1.0, true, player, &w);
    assert!(c.zombies.list[0].pos.y < ground_top, "둥실 떠오른다");
    assert_eq!(c.zombies.list[0].pos.x, player.x, "떠 있는 동안에는 걸어오지 않는다");
    assert_eq!(c.health.hp(), PLAYER_START_HP, "떠 있는 동안에는 공격하지 않는다");
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::PlayerHurt { .. })), 0);
}

#[test]
fn pop_happens_5_seconds_after_rising_and_gives_no_defeat() {
    let w = flat_world();
    let far = player_at(900.0);
    let mut c = night_with_one_zombie(0.0, true);
    let events = run(&mut c, ZOMBIE_FLOAT_START_SECONDS + ZOMBIE_FLOAT_SECONDS - 0.2, true, far, &w);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::ZombiePopped { .. })), 0);

    let events = run(&mut c, 0.5, true, far, &w);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::ZombiePopped { .. })), 1);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::ZombieDefeated { .. })), 0, "경험치 없음");
    assert_eq!(c.health.hp(), PLAYER_START_HP);
}

#[test]
fn pop_right_above_the_player_does_not_hurt() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = night_with_one_zombie(player.x, true);
    c.zombies.list[0].state = ZombieState::Floating;
    let events = run(&mut c, ZOMBIE_FLOAT_SECONDS + 0.1, true, player, &w);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::ZombiePopped { .. })), 1);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::PlayerHurt { .. })), 0);
}

#[test]
fn next_zombie_can_come_after_a_pop() {
    let w = flat_world();
    let far = player_at(900.0);
    let mut c = night_with_one_zombie(0.0, true);
    c.zombies.list.push(Zombie::new(0.0, GROUND_Y, true));
    c.zombies.list.push(Zombie::new(0.0, GROUND_Y, true));
    let events = run(&mut c, ZOMBIE_FLOAT_START_SECONDS + ZOMBIE_FLOAT_SECONDS + ZOMBIE_SPAWN_SECONDS + 1.0, true, far, &w);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::ZombiePopped { .. })), 3);
    assert!(count(&events, |e| matches!(e, CombatEvent::ZombieAppeared { .. })) >= 1);
}

#[test]
fn about_one_in_three_zombies_float() {
    let w = flat_world();
    let far = player_at(900.0); // 10초를 걸어도 닿지 않는 자리
    let total = 300;
    let floaters = (0..total)
        .filter(|&seed| {
            let mut c = Combat::new(seed);
            c.update(FRAME, true, far, &w);
            run(&mut c, ZOMBIE_FLOAT_START_SECONDS + 0.1, true, far, &w);
            c.zombies.list[0].state == ZombieState::Floating
        })
        .count();
    assert!((70..=130).contains(&floaters), "300마리 중 {floaters}마리");
}

// ---- 시드 ----

#[test]
fn same_seed_gives_the_same_night() {
    let w = flat_world();
    let player = player_at(480.0);
    let night = |seed| {
        let mut c = Combat::new(seed);
        run(&mut c, 100.0, true, player, &w)
            .iter()
            .filter_map(|e| match e {
                CombatEvent::ZombieAppeared { at } => Some(at.x),
                _ => None,
            })
            .collect::<Vec<f32>>()
    };
    assert_eq!(night(42), night(42));
    assert!((0..20).any(|seed| night(seed) != night(42)), "시드가 다르면 달라질 수 있다");
}

// ---- Game에 연결 ----

/// 밤이 시작된 게임. 좀비는 테스트가 직접 놓는다.
fn night_game() -> Game {
    let mut g = Game::new(CharacterKind::Niro, SEED);
    g.clock.update(DAY_SECONDS);
    g.update(&PlayerInput::default(), FRAME);
    g.combat.zombies.list.clear();
    g
}

fn zombie_at(g: &mut Game, x: f32) {
    g.combat.zombies.list.push(Zombie::new(x, GROUND_Y, false));
}

#[test]
fn new_game_has_full_health_and_no_zombies() {
    let g = Game::new(CharacterKind::Ellie, SEED);
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP);
    assert!(g.combat.zombies.list.is_empty());
}

#[test]
fn zombie_appears_in_the_same_frame_night_starts() {
    let mut g = Game::new(CharacterKind::Niro, SEED);
    g.clock.update(DAY_SECONDS - FRAME / 2.0);
    let events = g.update(&PlayerInput::default(), FRAME);
    assert_eq!(events[0], GameEvent::NightStarted);
    assert!(matches!(events[1], GameEvent::Combat(CombatEvent::ZombieAppeared { .. })));
}

#[test]
fn attack_button_hits_through_game_once_per_press() {
    let mut g = night_game();
    let x = g.player.pos.x + PLAYER_WIDTH + 20.0;
    zombie_at(&mut g, x);
    let attack = PlayerInput { attack_pressed: true, ..Default::default() };
    let events = g.update(&attack, FRAME);
    assert!(matches!(events[..], [GameEvent::Combat(CombatEvent::ZombieHit { hp_left: 2, .. })]));
    g.update(&PlayerInput::default(), FRAME);
    assert_eq!(g.combat.zombies.list[0].hp, 2, "버튼을 안 누르면 맞지 않는다");
}

#[test]
fn morning_through_game_clears_zombies_and_refills_health() {
    let mut g = night_game();
    let x = g.player.pos.x;
    zombie_at(&mut g, x);
    g.update(&PlayerInput::default(), FRAME);
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP - ZOMBIE_ATTACK);

    g.clock.update(g.clock.seconds_left_in_phase() - FRAME / 2.0);
    let events = g.update(&PlayerInput::default(), FRAME);
    assert!(events.contains(&GameEvent::DayStarted { day_number: 2 }));
    assert!(events.contains(&GameEvent::Combat(CombatEvent::HealthRefilled)));
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP);
    assert!(g.combat.zombies.list.is_empty());
}

#[test]
fn after_player_down_clock_player_coins_and_zombies_all_stop() {
    let mut g = night_game();
    let x = g.player.pos.x;
    zombie_at(&mut g, x);
    let mut events = Vec::new();
    for _ in 0..(5.0 / FRAME) as usize {
        events.extend(g.update(&PlayerInput::default(), FRAME));
    }
    assert!(g.combat.health.is_down());
    assert_eq!(events.iter().filter(|e| **e == GameEvent::Combat(CombatEvent::PlayerDown)).count(), 1);

    g.player.pos = g.coins.list[0].rect.point(); // 코인 위에 놓아도 줍지 않는다
    let (pos, left) = (g.player.pos, g.clock.seconds_left_in_phase());
    let press_all = PlayerInput { left: false, right: true, jump_pressed: true, attack_pressed: true };
    for _ in 0..120 {
        assert!(g.update(&press_all, FRAME).is_empty());
    }
    assert_eq!(g.player.pos, pos);
    assert_eq!(g.clock.seconds_left_in_phase(), left);
    assert_eq!(g.wallet.coins(), 0);
}
