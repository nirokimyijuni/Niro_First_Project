//! M5 스켈레톤과 레벨업 규칙 테스트 (수산나).
//! 기준: stories/backlog.md US-045~US-049, docs/02-plan.md D2, D3, D27, D32, D35. REVIEW-006 2번.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)] // 이 테스트에서 안 쓰는 logic 함수(좀비 번쩍임 등)가 있다
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::combat::{Combat, CombatEvent};
use logic::game::{Game, GameEvent};
use logic::monster::{Zombie, ZombieState};
use logic::player::{Facing, PlayerInput};
use logic::skeleton::{Projectile, ProjectileKind, Skeleton, SkeletonStage, SkeletonState};
use logic::world::World;
use macroquad::math::Rect;

const FRAME: f32 = 1.0 / 60.0;
const SEED: u32 = 7;
/// 첫째 작은 상자 앞에 막힌 왼쪽 스켈레톤의 x (실제 맵).
const X_BLOCKED_BY_SMALL_BOX: f32 = 252.0;

/// 상자 자리가 고정된 맵의 새 게임. 여기 규칙들은 랜덤 상자와 상관없다 (D36은 d36_random_boxes.rs).
fn m1_game(kind: CharacterKind) -> Game {
    Game::on_world(kind, SEED, World::m1())
}

fn flat_world() -> World {
    World { width: WORLD_WIDTH, ground_y: GROUND_Y, obstacles: vec![] }
}

/// 큰 상자 하나만 있는 맵. 상자 왼쪽 면은 x=560.
fn big_box_world() -> World {
    World { obstacles: vec![Rect::new(560.0, GROUND_Y - 64.0, 64.0, 64.0)], ..flat_world() }
}

/// 땅 위 x에 서 있는 플레이어의 몸.
fn player_at(x: f32) -> Rect {
    Rect::new(x, GROUND_Y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT)
}

/// 하늘 높이 있어서 공격에 안 맞는 플레이어. 쏘는 간격만 볼 때 쓴다.
fn player_in_the_sky(x: f32) -> Rect {
    Rect::new(x, 0.0, PLAYER_WIDTH, PLAYER_HEIGHT)
}

/// `box_`의 윗면에 서 있는 플레이어의 몸.
fn player_on(box_: Rect) -> Rect {
    Rect::new(box_.x, box_.y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT)
}

/// 맨손 공격 한 번.
fn hands_attack(c: &mut Combat, player: Rect, facing: Facing) -> Vec<CombatEvent> {
    c.attack_with(player, facing, HANDS_ATTACK, PLAYER_ATTACK_RANGE)
}

fn run(c: &mut Combat, seconds: f32, is_night: bool, player: Rect, world: &World) -> Vec<CombatEvent> {
    let mut events = Vec::new();
    for _ in 0..(seconds / FRAME).round() as usize {
        events.extend(c.update(FRAME, is_night, player, world));
    }
    events
}

/// 밤이고 몬스터가 없다. 다음 몬스터는 20초 뒤 좀비다.
fn empty_night() -> Combat {
    let mut c = Combat::new(SEED);
    c.skeleton_chance = 0.0;
    run(&mut c, FRAME, true, player_at(480.0), &flat_world());
    c.zombies.list.clear();
    c
}

fn night_with(x: f32, stage: SkeletonStage) -> Combat {
    let mut c = empty_night();
    c.skeletons.list.push(Skeleton::new(x, GROUND_Y, stage));
    c
}

fn shot(x: f32, direction: f32, kind: ProjectileKind, attack: u32, bounced: bool) -> Projectile {
    let mut p = Projectile::new(x, direction, kind, attack, &flat_world());
    p.bounced = bounced;
    p
}

fn arrow(x: f32, direction: f32, bounced: bool) -> Projectile {
    shot(x, direction, ProjectileKind::Arrow, SKELETON_NORMAL_ATTACK, bounced)
}

fn count(events: &[CombatEvent], wanted: fn(&CombatEvent) -> bool) -> usize {
    events.iter().filter(|e| wanted(e)).count()
}

fn fired(events: &[CombatEvent]) -> usize {
    count(events, |e| matches!(e, CombatEvent::ProjectileFired { .. }))
}

fn hurts(events: &[CombatEvent]) -> usize {
    count(events, |e| matches!(e, CombatEvent::PlayerHurt { .. }))
}

// ---- REVIEW-006 2번 ----

#[test]
fn feet_on_big_box_touching_zombie_head_do_not_hurt() {
    let w = World::m1();
    let box_ = w.obstacles[1];
    assert_eq!(box_.h, ZOMBIE_HEIGHT, "이 경우는 좀비 키와 상자 높이가 같아서 생긴다");
    let mut c = empty_night();
    c.zombies.list.push(Zombie::new(box_.x - ZOMBIE_WIDTH, GROUND_Y, false));
    let player = Rect::new(box_.x - 20.0, box_.y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT);
    let events = run(&mut c, 0.5, true, player, &w);
    assert_eq!(hurts(&events), 0, "발과 머리가 선으로만 닿았다");

    let sunk_a_little = Rect::new(player.x, player.y + 1.0, player.w, player.h);
    assert_eq!(hurts(&run(&mut c, FRAME, true, sunk_a_little, &w)), 1, "정말 겹치면 맞는다");
}

// ---- US-045 경험치와 레벨 ----

/// 체력이 1 남은 좀비를 앞에 놓고 때린다.
fn defeat_a_zombie(c: &mut Combat) -> Vec<CombatEvent> {
    let player = player_at(300.0);
    let mut zombie = Zombie::new(player.x + PLAYER_WIDTH, GROUND_Y, false);
    zombie.hp = 1;
    c.zombies.list.push(zombie);
    hands_attack(c, player, Facing::Right)
}

#[test]
fn starts_at_level_1_with_no_experience() {
    let c = Combat::new(SEED);
    assert_eq!((c.level, c.experience), (1, 0));
}

#[test]
fn five_defeats_level_up_and_experience_starts_over() {
    let mut c = empty_night();
    for i in 1..EXPERIENCE_PER_LEVEL {
        assert!(!defeat_a_zombie(&mut c).contains(&CombatEvent::LevelUp { level: 2 }));
        assert_eq!((c.level, c.experience), (1, i));
    }
    assert!(defeat_a_zombie(&mut c).contains(&CombatEvent::LevelUp { level: 2 }));
    assert_eq!((c.level, c.experience), (2, 0));
    assert_eq!(c.health.max_hp(), 12);
    for _ in 0..EXPERIENCE_PER_LEVEL {
        defeat_a_zombie(&mut c);
    }
    assert_eq!((c.level, c.experience, c.health.max_hp()), (3, 0, 17));
}

#[test]
fn level_up_fills_health_right_away() {
    let mut c = empty_night();
    for _ in 0..EXPERIENCE_PER_LEVEL {
        defeat_a_zombie(&mut c);
    }
    assert_eq!(c.health.hp(), 12, "아침을 기다리지 않는다");
}

#[test]
fn hit_that_does_not_defeat_gives_no_experience() {
    let mut c = empty_night();
    let player = player_at(300.0);
    c.zombies.list.push(Zombie::new(player.x + PLAYER_WIDTH, GROUND_Y, false));
    hands_attack(&mut c, player, Facing::Right);
    assert_eq!(c.experience, 0);
}

#[test]
fn popped_zombie_gives_no_experience() {
    let w = flat_world();
    let far = player_at(900.0);
    let mut c = empty_night();
    c.zombies.list.push(Zombie::new(0.0, GROUND_Y, true));
    let events = run(&mut c, ZOMBIE_FLOAT_START_SECONDS + ZOMBIE_FLOAT_SECONDS + 0.5, true, far, &w);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::ZombiePopped { .. })), 1);
    assert_eq!(c.experience, 0);
}

#[test]
fn skeleton_defeated_by_attack_gives_experience() {
    let player = player_at(300.0);
    let mut c = night_with(player.x + PLAYER_WIDTH, SkeletonStage::Normal);
    for _ in 0..SKELETON_HP - 1 {
        let events = hands_attack(&mut c, player, Facing::Right);
        assert_eq!(count(&events, |e| matches!(e, CombatEvent::SkeletonHit { .. })), 1);
    }
    assert_eq!(c.experience, 0);
    let events = hands_attack(&mut c, player, Facing::Right);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::SkeletonDefeated { .. })), 1);
    assert!(c.skeletons.list.is_empty());
    assert_eq!(c.experience, 1);
}

#[test]
fn one_swing_can_cross_the_level_line_with_two_monsters() {
    let player = player_at(300.0);
    let mut c = night_with(player.x + PLAYER_WIDTH, SkeletonStage::Normal);
    c.skeletons.list[0].hp = 1;
    c.experience = EXPERIENCE_PER_LEVEL - 1;
    let mut zombie = Zombie::new(player.x + PLAYER_WIDTH, GROUND_Y, false);
    zombie.hp = 1;
    c.zombies.list.push(zombie);
    let events = hands_attack(&mut c, player, Facing::Right);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::LevelUp { .. })), 1);
    assert_eq!((c.level, c.experience), (2, 1));
}

#[test]
fn zombie_stays_hp_3_attack_2_after_level_ups() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = empty_night();
    c.level_up();
    c.level_up();
    c.zombies.list.push(Zombie::new(player.x, GROUND_Y, false));
    assert_eq!(c.zombies.list[0].hp, 3);
    let events = c.update(FRAME, true, player, &w);
    assert!(events.contains(&CombatEvent::PlayerHurt { amount: 2, hp_left: 15 }));
}

#[test]
fn level_up_comes_through_game() {
    let mut g = m1_game(CharacterKind::Niro);
    g.combat.skeleton_chance = 0.0;
    g.clock.update(DAY_SECONDS);
    g.combat.experience = EXPERIENCE_PER_LEVEL - 1;
    g.update(&PlayerInput::default(), FRAME);
    g.combat.zombies.list.clear();
    let mut zombie = Zombie::new(g.player.pos.x + PLAYER_WIDTH, GROUND_Y, false);
    zombie.hp = 1;
    g.combat.zombies.list.push(zombie);
    let events = g.update(&PlayerInput { attack_pressed: true, ..Default::default() }, FRAME);
    assert!(events.contains(&GameEvent::Combat(CombatEvent::LevelUp { level: 2 })));
}

// ---- 단계와 나오기 (D35) ----

#[test]
fn stage_values_match_d35() {
    assert_eq!(SkeletonStage::Normal.attack(), 2);
    assert_eq!(SkeletonStage::Half.attack(), 3);
    assert_eq!(SkeletonStage::Full.attack(), 4);
    for stage in [SkeletonStage::Normal, SkeletonStage::Half, SkeletonStage::Full] {
        assert_eq!(Skeleton::new(0.0, GROUND_Y, stage).hp, 3);
    }
}

/// 몬스터가 쌓이게 플레이어 양옆을 큰 벽으로 막은 맵.
fn walled_world() -> World {
    let mut w = flat_world();
    w.obstacles.push(Rect::new(300.0, 0.0, 10.0, GROUND_Y));
    w.obstacles.push(Rect::new(650.0, 0.0, 10.0, GROUND_Y));
    w
}

fn appeared_stages(events: &[CombatEvent]) -> Vec<SkeletonStage> {
    events
        .iter()
        .filter_map(|e| match e {
            CombatEvent::SkeletonAppeared { stage, .. } => Some(*stage),
            _ => None,
        })
        .collect()
}

#[test]
fn each_level_brings_only_its_stage() {
    let wanted = [(1, SkeletonStage::Normal), (2, SkeletonStage::Half), (3, SkeletonStage::Full), (6, SkeletonStage::Full)];
    for (level, stage) in wanted {
        let mut stages = Vec::new();
        for seed in 0..20 {
            let mut c = Combat::new(seed);
            c.level = level;
            // 레벨이 오르기 전에 끝나게 두 차례만 본다.
            let seconds = ZOMBIE_SPAWN_SECONDS * 2.0 + 1.0;
            stages.extend(appeared_stages(&run(&mut c, seconds, true, player_at(480.0), &walled_world())));
        }
        assert!(!stages.is_empty());
        assert!(stages.iter().all(|s| *s == stage), "레벨 {level}: {stages:?}");
    }
}

#[test]
fn skeletons_and_zombies_mix_at_night() {
    let (mut zombies, mut skeletons) = (0, 0);
    for seed in 0..50 {
        let mut c = Combat::new(seed);
        let events = run(&mut c, FRAME, true, player_at(480.0), &flat_world());
        zombies += count(&events, |e| matches!(e, CombatEvent::ZombieAppeared { .. }));
        skeletons += count(&events, |e| matches!(e, CombatEvent::SkeletonAppeared { .. }));
    }
    assert_eq!(zombies + skeletons, 50 * MONSTERS_PER_SPAWN, "마리마다 둘 중 하나를 뽑는다");
    assert!((20..=80).contains(&skeletons), "100마리 중 {skeletons}마리");
}

#[test]
fn no_skeletons_in_the_daytime() {
    let mut c = Combat::new(SEED);
    let events = run(&mut c, 60.0, false, player_at(480.0), &flat_world());
    assert!(c.skeletons.list.is_empty());
    assert!(events.is_empty());
}

#[test]
fn skeletons_appear_only_at_screen_edges() {
    let w = flat_world();
    for seed in 0..30 {
        let mut c = Combat::new(seed);
        c.skeleton_chance = 1.0;
        let events = c.update(FRAME, true, player_at(480.0), &w);
        assert_eq!(appeared_stages(&events).len(), MONSTERS_PER_SPAWN);
        for skeleton in &c.skeletons.list {
            let x = skeleton.pos.x;
            assert!(x < 5.0 || x > w.width - SKELETON_WIDTH - 5.0, "seed {seed}: x={x}");
        }
    }
}

#[test]
fn screen_never_holds_more_than_the_limit() {
    let mut reached_the_limit = false;
    for seed in 0..10 {
        let w = walled_world();
        let player = player_at(480.0);
        let mut c = Combat::new(seed);
        let (mut most_monsters, mut most_zombies) = (0, 0);
        for _ in 0..(200.0 / FRAME) as usize {
            c.update(FRAME, true, player, &w);
            most_monsters = most_monsters.max(c.zombies.list.len() + c.skeletons.list.len());
            most_zombies = most_zombies.max(c.zombies.list.len());
        }
        assert!(most_monsters <= MONSTER_MAX_ON_SCREEN, "seed {seed}");
        assert!(most_zombies <= ZOMBIE_MAX_ON_SCREEN, "seed {seed}");
        reached_the_limit |= most_monsters == MONSTER_MAX_ON_SCREEN;
    }
    assert!(reached_the_limit);
}

#[test]
fn a_skeleton_comes_when_zombies_are_full() {
    let mut c = empty_night();
    for _ in 0..ZOMBIE_MAX_ON_SCREEN {
        c.zombies.list.push(Zombie::new(0.0, GROUND_Y, false));
    }
    run(&mut c, ZOMBIE_SPAWN_SECONDS + 1.0, true, player_at(480.0), &walled_world());
    assert_eq!((c.zombies.list.len(), c.skeletons.list.len()), (ZOMBIE_MAX_ON_SCREEN, MONSTERS_PER_SPAWN));
}

#[test]
fn morning_removes_skeletons_and_shots_and_next_night_starts_at_once() {
    let w = flat_world();
    let player = player_at(480.0);
    let mut c = night_with(0.0, SkeletonStage::Normal);
    c.skeletons.projectiles.push(arrow(100.0, 1.0, false));
    c.update(FRAME, false, player, &w);
    assert!(c.skeletons.list.is_empty() && c.skeletons.projectiles.is_empty());

    c.update(FRAME, true, player, &w);
    assert_eq!(c.zombies.list.len() + c.skeletons.list.len(), MONSTERS_PER_SPAWN);
}

#[test]
fn same_seed_gives_the_same_night_with_skeletons() {
    let w = World::m1();
    let night = |seed| {
        let mut c = Combat::new(seed);
        let events = run(&mut c, 100.0, true, player_at(120.0), &w);
        (events, c.health.hp())
    };
    assert_eq!(night(42), night(42));
}

// ---- 걷기: 상자를 못 넘는다 ----

#[test]
fn every_stage_is_stopped_by_a_small_box() {
    let w = World::m1();
    for stage in [SkeletonStage::Normal, SkeletonStage::Half, SkeletonStage::Full] {
        let mut c = night_with(0.0, stage);
        // 도착하고 바로 본다. 더 지나면 완전체는 튕겨 온 총알에 쓰러진다 (D36).
        run(&mut c, 5.0, true, Rect::new(900.0, 376.0, 48.0, 64.0), &w);
        assert_eq!(c.skeletons.list[0].pos.x, X_BLOCKED_BY_SMALL_BOX, "{stage:?}");
        assert_ne!(c.skeletons.list[0].state, SkeletonState::Walking);
    }
}

#[test]
fn shooters_stop_at_shooting_distance_but_half_keeps_walking() {
    let w = flat_world();
    let player = player_at(800.0);
    let mut shooter = night_with(0.0, SkeletonStage::Normal);
    run(&mut shooter, 2.0, true, player, &w);
    assert_eq!(shooter.skeletons.list[0].state, SkeletonState::Walking);
    run(&mut shooter, 8.0, true, player, &w);
    let gap = player.center().x - shooter.skeletons.list[0].rect().center().x;
    assert!((gap - SKELETON_SHOOT_DISTANCE).abs() < 2.0, "{gap}");
    assert_eq!(shooter.skeletons.list[0].state, SkeletonState::Standing);

    let mut half = night_with(300.0, SkeletonStage::Half);
    run(&mut half, 6.0, true, player, &w);
    assert!(half.skeletons.list[0].pos.x > 500.0, "절반은 멈추지 않는다");
}

#[test]
fn skeleton_walks_slower_than_player() {
    assert!(SKELETON_WALK_SPEED < PLAYER_WALK_SPEED);
}

// ---- US-046 화살, US-048 총알 ----

#[test]
fn normal_skeleton_shoots_an_arrow_every_3_seconds() {
    let w = flat_world();
    let player = player_in_the_sky(800.0);
    let mut c = night_with(100.0, SkeletonStage::Normal);
    assert_eq!(fired(&run(&mut c, ARROW_INTERVAL_SECONDS - 0.1, true, player, &w)), 0);
    assert_eq!(fired(&run(&mut c, 0.2, true, player, &w)), 1);
    assert_eq!(fired(&run(&mut c, ARROW_INTERVAL_SECONDS - 0.2, true, player, &w)), 0);
    assert_eq!(fired(&run(&mut c, 0.2, true, player, &w)), 1);
}

#[test]
fn full_skeleton_shoots_a_bullet_every_2_seconds() {
    let w = flat_world();
    let player = player_in_the_sky(800.0);
    let mut c = night_with(100.0, SkeletonStage::Full);
    let events = run(&mut c, BULLET_INTERVAL_SECONDS * 3.0 + 0.1, true, player, &w);
    assert_eq!(fired(&events), 3);
    assert!(events.iter().any(|e| matches!(e, CombatEvent::ProjectileFired { kind: ProjectileKind::Bullet, .. })));
}

#[test]
fn shots_fly_toward_the_player_side() {
    let w = flat_world();
    let mut c = night_with(500.0, SkeletonStage::Normal);
    run(&mut c, ARROW_INTERVAL_SECONDS + 0.05, true, player_at(100.0), &w);
    let shot = &c.skeletons.projectiles[0];
    assert_eq!((shot.direction, shot.kind, shot.bounced), (-1.0, ProjectileKind::Arrow, false));
}

#[test]
fn arrow_takes_2_and_disappears() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = empty_night();
    c.skeletons.projectiles.push(arrow(player.x - 30.0, 1.0, false));
    let events = run(&mut c, 0.1, true, player, &w);
    assert!(events.contains(&CombatEvent::PlayerHurt { amount: 2, hp_left: 5 }));
    assert!(c.skeletons.projectiles.is_empty());
}

#[test]
fn bullet_from_full_skeleton_takes_4() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = night_with(0.0, SkeletonStage::Full);
    let events = run(&mut c, BULLET_INTERVAL_SECONDS + 1.0, true, player, &w);
    assert!(events.contains(&CombatEvent::PlayerHurt { amount: 4, hp_left: 3 }));
}

#[test]
fn second_shot_during_safety_does_not_hurt() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = empty_night();
    c.skeletons.projectiles.push(arrow(player.x - 30.0, 1.0, false));
    c.skeletons.projectiles.push(arrow(player.x - 60.0, 1.0, false));
    let events = run(&mut c, 0.3, true, player, &w);
    assert_eq!(hurts(&events), 1);
    assert!(c.skeletons.projectiles.is_empty(), "닿은 화살은 사라진다");
}

#[test]
fn two_shots_in_one_moment_take_the_stronger_only() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = empty_night();
    c.skeletons.projectiles.push(arrow(player.x, 1.0, false));
    c.skeletons.projectiles.push(shot(player.x, 1.0, ProjectileKind::Bullet, SKELETON_FULL_ATTACK, false));
    let events = c.update(FRAME, true, player, &w);
    assert!(events.contains(&CombatEvent::PlayerHurt { amount: 4, hp_left: 3 }));
    assert_eq!(hurts(&events), 1);
}

/// 진짜 플레이어가 점프로 피한다. `gap`은 공격 앞쪽과 몸 사이.
fn jump_over(new_shot: fn(f32) -> Projectile, gap: f32) -> (u32, usize) {
    let mut g = m1_game(CharacterKind::Niro);
    g.combat.skeleton_chance = 0.0;
    g.clock.update(DAY_SECONDS);
    g.player.pos.x = 400.0;
    g.update(&PlayerInput::default(), FRAME);
    g.combat.zombies.list.clear();
    g.combat.skeletons.projectiles.push(new_shot(g.player.pos.x + PLAYER_WIDTH + gap));

    g.update(&PlayerInput { jump_pressed: true, ..Default::default() }, FRAME);
    for _ in 0..120 {
        g.update(&PlayerInput::default(), FRAME);
    }
    (g.combat.health.hp(), g.combat.skeletons.projectiles.len())
}

#[test]
fn jumping_avoids_arrows_and_bullets() {
    let (hp, _) = jump_over(|x| arrow(x, -1.0, false), 60.0);
    assert_eq!(hp, PLAYER_START_HP);
    let (hp, _) = jump_over(|x| shot(x, -1.0, ProjectileKind::Bullet, SKELETON_FULL_ATTACK, false), 100.0);
    assert_eq!(hp, PLAYER_START_HP);
}

#[test]
fn shots_disappear_at_the_screen_edge() {
    let w = flat_world();
    let far = player_at(100.0);
    let mut c = empty_night();
    c.skeletons.projectiles.push(arrow(800.0, 1.0, false));
    run(&mut c, 2.0, true, far, &w);
    assert!(c.skeletons.projectiles.is_empty());
    c.skeletons.projectiles.push(arrow(30.0, -1.0, false));
    run(&mut c, 1.0, true, player_at(600.0), &w);
    assert!(c.skeletons.projectiles.is_empty());
}

#[test]
fn touching_a_shooting_skeleton_does_not_hurt() {
    let w = flat_world();
    let player = player_at(300.0);
    for stage in [SkeletonStage::Normal, SkeletonStage::Full] {
        let mut c = night_with(player.x, stage);
        assert_eq!(hurts(&run(&mut c, 0.5, true, player, &w)), 0, "{stage:?}");
    }
}

// ---- 화살 높이: 작은 상자는 넘고 큰 상자에서 튕긴다 ----

#[test]
fn flight_height_is_between_small_and_big_box() {
    let w = World::m1();
    let shot = arrow(0.0, 1.0, false).rect();
    let (small, big) = (w.obstacles[0], w.obstacles[1]);
    assert!(shot.y + shot.h < small.y, "작은 상자 윗면보다 위");
    assert!(shot.y > big.y, "큰 상자 윗면보다 아래");
    assert_eq!(w.obstacles[2].h, small.h);
}

#[test]
fn blocked_skeleton_arrow_flies_over_small_box_to_player_beyond() {
    let w = World::m1();
    let player = player_at(420.0);
    let mut c = night_with(X_BLOCKED_BY_SMALL_BOX, SkeletonStage::Normal);
    let events = run(&mut c, ARROW_INTERVAL_SECONDS + 1.0, true, player, &w);
    assert!(events.contains(&CombatEvent::PlayerHurt { amount: 2, hp_left: 5 }));
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::ProjectileBounced { .. })), 0);
}

#[test]
fn blocked_skeleton_bullet_flies_over_small_box_too() {
    let w = World::m1();
    let player = player_at(420.0);
    let mut c = night_with(X_BLOCKED_BY_SMALL_BOX, SkeletonStage::Full);
    let events = run(&mut c, BULLET_INTERVAL_SECONDS + 1.0, true, player, &w);
    assert!(events.contains(&CombatEvent::PlayerHurt { amount: 4, hp_left: 3 }));
}

#[test]
fn standing_on_a_small_box_is_not_safe_but_on_the_big_box_is() {
    let w = World::m1();
    let mut c = empty_night();
    let small = player_on(w.obstacles[2]);
    c.skeletons.projectiles.push(arrow(small.x - 40.0, 1.0, false));
    assert_eq!(hurts(&run(&mut c, 0.3, true, small, &w)), 1);

    let mut c = empty_night();
    let big = player_on(w.obstacles[1]);
    c.skeletons.projectiles.push(arrow(big.x + 100.0, -1.0, false));
    assert_eq!(hurts(&run(&mut c, 1.0, true, big, &w)), 0);
}

// ---- US-049 튕김 ----

#[test]
fn big_box_bounces_the_shot_back() {
    let w = big_box_world();
    let far = player_at(800.0);
    let mut c = empty_night();
    c.skeletons.projectiles.push(arrow(500.0, 1.0, false));
    let events = run(&mut c, 0.5, true, far, &w);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::ProjectileBounced { .. })), 1);
    let shot = &c.skeletons.projectiles[0];
    assert!(shot.bounced);
    assert_eq!(shot.direction, -1.0);
    assert!(shot.pos.x + shot.rect().w < 560.0, "상자 앞으로 돌아왔다");
}

#[test]
fn shot_from_the_right_bounces_off_the_right_face() {
    let w = big_box_world();
    let mut c = empty_night();
    c.skeletons.projectiles.push(arrow(650.0, -1.0, false));
    run(&mut c, 0.3, true, player_at(100.0), &w);
    let shot = &c.skeletons.projectiles[0];
    assert_eq!(shot.direction, 1.0);
    assert!(shot.pos.x > 624.0);
}

#[test]
fn bounce_happens_once_then_next_box_removes_the_shot() {
    let mut w = big_box_world();
    w.obstacles.push(Rect::new(400.0, GROUND_Y - 64.0, 64.0, 64.0));
    let mut c = empty_night();
    c.skeletons.projectiles.push(arrow(500.0, 1.0, false));
    let events = run(&mut c, 1.0, true, player_at(800.0), &w);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::ProjectileBounced { .. })), 1);
    assert!(c.skeletons.projectiles.is_empty());
}

#[test]
fn bounced_shot_disappears_after_its_range() {
    let w = big_box_world();
    let mut c = empty_night();
    c.skeletons.projectiles.push(arrow(540.0, 1.0, false));
    let seconds_of_range = PROJECTILE_BOUNCED_RANGE / ARROW_SPEED;
    let far = player_at(800.0);
    run(&mut c, seconds_of_range - 0.1, true, far, &w);
    assert_eq!(c.skeletons.projectiles.len(), 1);
    run(&mut c, 0.3, true, far, &w);
    assert!(c.skeletons.projectiles.is_empty(), "화면 안인데도 사라진다");
}

#[test]
fn long_frame_does_not_pass_through_the_big_box() {
    let w = big_box_world();
    let mut c = empty_night();
    c.skeletons.projectiles.push(shot(540.0, 1.0, ProjectileKind::Bullet, SKELETON_FULL_ATTACK, false));
    c.update(MAX_FRAME_SECONDS, true, player_at(800.0), &w);
    assert_eq!(c.skeletons.projectiles[0].direction, -1.0);
}

#[test]
fn bounced_shot_hurts_the_player_by_its_own_attack() {
    let w = big_box_world();
    let player = player_at(430.0);
    let mut c = empty_night();
    c.skeletons.projectiles.push(shot(540.0, 1.0, ProjectileKind::Bullet, SKELETON_FULL_ATTACK, false));
    let events = run(&mut c, 0.5, true, player, &w);
    assert!(events.contains(&CombatEvent::PlayerHurt { amount: 4, hp_left: 3 }));
}

#[test]
fn unbounced_shot_passes_through_skeletons() {
    let w = flat_world();
    let mut c = night_with(300.0, SkeletonStage::Half);
    c.skeletons.projectiles.push(arrow(250.0, 1.0, false));
    let events = run(&mut c, 0.5, true, player_at(900.0), &w);
    assert_eq!(c.skeletons.list[0].hp, SKELETON_HP);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::SkeletonDodged { .. })), 0);
}

#[test]
fn skeleton_pressed_against_big_box_is_hit_by_a_bounce_and_can_fall() {
    let w = big_box_world();
    let far = player_at(800.0);
    let mut c = night_with(560.0 - SKELETON_WIDTH, SkeletonStage::Half);
    c.skeletons.list[0].hp = 2;
    c.skeletons.projectiles.push(arrow(450.0, 1.0, false));
    let events = run(&mut c, 0.5, true, far, &w);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::SkeletonHit { hp_left: 1, .. })), 1);
    assert!(c.skeletons.projectiles.is_empty(), "맞은 화살은 사라진다");

    c.skeletons.projectiles.push(arrow(450.0, 1.0, false));
    let events = run(&mut c, 0.5, true, far, &w);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::SkeletonDefeated { .. })), 1);
    assert!(c.skeletons.list.is_empty());
    assert_eq!(c.experience, 1, "튕긴 공격으로 쓰러져도 경험치를 준다");
}

#[test]
fn level_up_by_a_bounce_comes_with_an_event() {
    let w = big_box_world();
    let mut c = night_with(560.0 - SKELETON_WIDTH, SkeletonStage::Half);
    c.skeletons.list[0].hp = 1;
    c.experience = EXPERIENCE_PER_LEVEL - 1;
    c.skeletons.projectiles.push(arrow(450.0, 1.0, false));
    let events = run(&mut c, 0.5, true, player_at(800.0), &w);
    assert!(events.contains(&CombatEvent::LevelUp { level: 2 }));
}

/// 맵의 왼쪽 스켈레톤이 쏜 공격이 큰 상자에서 튕겨 돌아온다.
fn own_shot_comes_back(stage: SkeletonStage) {
    let w = World::m1();
    let far = player_at(900.0);
    let mut c = night_with(X_BLOCKED_BY_SMALL_BOX, stage);
    let events = run(&mut c, 6.0, true, far, &w);
    assert!(count(&events, |e| matches!(e, CombatEvent::ProjectileBounced { .. })) >= 1, "{stage:?}");
    assert!(count(&events, |e| matches!(e, CombatEvent::SkeletonDodged { .. })) >= 1, "{stage:?}");
    assert_eq!(c.skeletons.list[0].hp, SKELETON_HP, "{stage:?}: 피해서 맞지 않는다");
    c.skeletons.projectiles.clear();
    run(&mut c, 2.0 * SKELETON_DODGE_JUMP_SPEED / GRAVITY, true, far, &w);
    assert_eq!(c.skeletons.list[0].pos.y, GROUND_Y - SKELETON_HEIGHT, "{stage:?}: 내려왔다");
}

#[test]
fn skeleton_jumps_over_its_own_arrow() {
    own_shot_comes_back(SkeletonStage::Normal);
}

#[test]
fn skeleton_jumps_too_late_for_its_own_bullet_d36() {
    let w = World::m1();
    let mut c = night_with(X_BLOCKED_BY_SMALL_BOX, SkeletonStage::Full);
    let events = run(&mut c, 3.5, true, player_at(900.0), &w);
    assert!(count(&events, |e| matches!(e, CombatEvent::SkeletonDodged { .. })) >= 1, "뛰기는 뛴다");
    assert!(c.skeletons.list[0].hp < SKELETON_HP, "총알이 더 빨라서 맞는다");
}

#[test]
fn skeleton_does_not_shoot_or_walk_while_dodging() {
    let w = flat_world();
    let far = player_at(900.0);
    let mut c = night_with(300.0, SkeletonStage::Normal);
    run(&mut c, ARROW_INTERVAL_SECONDS - 0.1, true, far, &w);
    let x = c.skeletons.list[0].pos.x;
    let right_edge = x + SKELETON_WIDTH;
    c.skeletons.projectiles.push(arrow(right_edge + 30.0, -1.0, true));

    let events = run(&mut c, 0.6, true, far, &w);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::SkeletonDodged { .. })), 1);
    assert_eq!(c.skeletons.list[0].state, SkeletonState::Dodging);
    assert_eq!(fired(&events), 0);
    assert_eq!(c.skeletons.list[0].pos.x, x);

    let landing = 2.0 * SKELETON_DODGE_JUMP_SPEED / GRAVITY - 0.6;
    let events = run(&mut c, landing + 0.3, true, far, &w);
    assert_ne!(c.skeletons.list[0].state, SkeletonState::Dodging);
    assert_eq!(fired(&events), 1, "내려온 뒤에 다시 쏜다");
}

#[test]
fn far_or_passing_bounced_shot_is_not_dodged() {
    let w = flat_world();
    let far = player_at(900.0);
    let mut c = night_with(300.0, SkeletonStage::Half);
    c.skeletons.projectiles.push(arrow(500.0, 1.0, true));
    c.skeletons.projectiles.push(arrow(200.0, -1.0, true));
    let events = run(&mut c, 0.1, true, far, &w);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::SkeletonDodged { .. })), 0);
}

// ---- US-047 손 ----

#[test]
fn half_skeleton_never_shoots() {
    let w = flat_world();
    let mut c = night_with(0.0, SkeletonStage::Half);
    let events = run(&mut c, 5.0, true, player_at(900.0), &w);
    assert_eq!(fired(&events), 0);
    assert!(c.skeletons.projectiles.is_empty());
}

#[test]
fn hand_hits_for_3_when_touching_then_safety() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = night_with(player.x + 10.0, SkeletonStage::Half);
    let events = c.update(FRAME, true, player, &w);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::SkeletonHandHit { .. })), 1);
    assert!(events.contains(&CombatEvent::PlayerHurt { amount: 3, hp_left: 4 }));

    let events = run(&mut c, PLAYER_HURT_INVINCIBLE_SECONDS - 0.1, true, player, &w);
    assert_eq!(hurts(&events), 0);
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::SkeletonHandHit { .. })), 0);
    assert_eq!(hurts(&run(&mut c, 0.2, true, player, &w)), 1);
}

#[test]
fn hand_does_not_reach_far_away() {
    let w = flat_world();
    let player = player_at(600.0);
    let mut c = night_with(100.0, SkeletonStage::Half);
    assert_eq!(hurts(&run(&mut c, 1.0, true, player, &w)), 0);
}

#[test]
fn player_attack_reaches_every_stage_and_three_hits_defeat() {
    let player = player_at(300.0);
    for stage in [SkeletonStage::Normal, SkeletonStage::Half, SkeletonStage::Full] {
        let mut c = night_with(player.x + PLAYER_WIDTH + PLAYER_ATTACK_RANGE - 1.0, stage);
        for _ in 0..3 {
            hands_attack(&mut c, player, Facing::Right);
        }
        assert!(c.skeletons.list.is_empty(), "{stage:?}");
    }
}

#[test]
fn attack_does_not_reach_behind_or_too_far() {
    let player = player_at(300.0);
    let mut c = night_with(player.x - PLAYER_WIDTH, SkeletonStage::Normal);
    assert!(hands_attack(&mut c, player, Facing::Right).is_empty());
    let mut c = night_with(player.x + PLAYER_WIDTH + PLAYER_ATTACK_RANGE + 5.0, SkeletonStage::Normal);
    assert!(hands_attack(&mut c, player, Facing::Right).is_empty());
}

#[test]
fn hit_skeleton_flashes_for_a_moment() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = night_with(player.x + PLAYER_WIDTH, SkeletonStage::Normal);
    hands_attack(&mut c, player, Facing::Right);
    assert!(c.skeletons.list[0].is_flashing());
    run(&mut c, SKELETON_HIT_FLASH_SECONDS + 0.05, true, player, &w);
    assert!(!c.skeletons.list[0].is_flashing());
}

// ---- 쓰러진 뒤 ----

#[test]
fn nothing_happens_after_going_down_by_a_shot() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = night_with(0.0, SkeletonStage::Full);
    let mut events = Vec::new();
    let mut frames = 0;
    while !c.health.is_down() {
        events.extend(c.update(FRAME, true, player, &w));
        frames += 1;
        assert!(frames < 36_000, "10분 안에 쓰러져야 한다");
    }
    assert_eq!(events.last(), Some(&CombatEvent::PlayerDown));
    assert_eq!(count(&events, |e| matches!(e, CombatEvent::PlayerDown)), 1);

    let frozen: Vec<f32> = c.skeletons.projectiles.iter().map(|p| p.pos.x).collect();
    let x = c.skeletons.list[0].pos.x;
    assert!(run(&mut c, 3.0, true, player, &w).is_empty());
    assert!(run(&mut c, 3.0, false, player, &w).is_empty(), "아침이 와도 움직이지 않는다");
    assert_eq!(c.skeletons.projectiles.iter().map(|p| p.pos.x).collect::<Vec<f32>>(), frozen);
    assert_eq!(c.skeletons.list[0].pos.x, x);
    assert!(hands_attack(&mut c, player, Facing::Right).is_empty());
}

#[test]
fn hand_that_downs_the_player_ends_the_events() {
    let w = flat_world();
    let player = player_at(300.0);
    let mut c = night_with(player.x, SkeletonStage::Half);
    c.skeletons.list.push(Skeleton::new(player.x, GROUND_Y, SkeletonStage::Half));
    let mut events = Vec::new();
    let mut frames = 0;
    while !c.health.is_down() {
        events.extend(c.update(FRAME, true, player, &w));
        frames += 1;
        assert!(frames < 36_000, "10분 안에 쓰러져야 한다");
    }
    assert_eq!(events.last(), Some(&CombatEvent::PlayerDown));
    assert_eq!(hurts(&events), 3, "7 → 4 → 1 → 0");
}

#[test]
fn floating_zombie_state_is_not_changed_by_skeletons() {
    let w = flat_world();
    let mut c = night_with(0.0, SkeletonStage::Normal);
    c.zombies.list.push(Zombie::new(0.0, GROUND_Y, true));
    run(&mut c, ZOMBIE_FLOAT_START_SECONDS + 0.1, true, player_at(900.0), &w);
    assert_eq!(c.zombies.list[0].state, ZombieState::Floating);
}

// ---- 아이가 정한 규칙: 레벨이 오르면 체력이 바로 다 찬다 ----

/// 체력이 깎인 채 경험치 하나만 더 있으면 레벨이 오르는 싸움.
fn hurt_one_defeat_from_level_up() -> Combat {
    let mut c = Combat::new(SEED);
    c.skeleton_chance = 0.0;
    c.experience = EXPERIENCE_PER_LEVEL - 1;
    let w = flat_world();
    c.zombies.list.push(Zombie::new(100.0, GROUND_Y, false));
    c.update(FRAME, true, Rect::new(100.0, GROUND_Y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT), &w);
    assert!(c.health.hp() < PLAYER_START_HP);
    c.zombies.list.clear();
    c
}

#[test]
fn level_up_event_comes_with_health_refilled() {
    let mut c = hurt_one_defeat_from_level_up();
    let player = Rect::new(100.0, GROUND_Y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT);
    c.zombies.list.push(Zombie::new(player.x + PLAYER_WIDTH + 10.0, GROUND_Y, false));
    c.zombies.list[0].hp = 1;
    let events = hands_attack(&mut c, player, Facing::Right);
    assert_eq!(c.health.hp(), PLAYER_START_HP + PLAYER_HP_PER_LEVEL);
    assert_eq!(c.health.hp(), c.health.max_hp());
    let level_up_at = events.iter().position(|e| *e == CombatEvent::LevelUp { level: 2 }).expect("레벨 업");
    assert_eq!(events.get(level_up_at + 1), Some(&CombatEvent::HealthRefilled), "{events:?}");
}

#[test]
fn level_up_never_happens_after_going_down() {
    let mut c = Combat::new(SEED);
    c.skeleton_chance = 0.0;
    c.experience = EXPERIENCE_PER_LEVEL - 1;
    let w = flat_world();
    let player = Rect::new(100.0, GROUND_Y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT);
    for _ in 0..(20.0 / FRAME) as usize {
        if c.health.is_down() {
            break;
        }
        c.zombies.list = vec![Zombie::new(100.0, GROUND_Y, false)];
        c.update(FRAME, true, player, &w);
    }
    assert!(c.health.is_down());

    c.zombies.list = vec![Zombie::new(player.x + PLAYER_WIDTH + 10.0, GROUND_Y, false)];
    c.zombies.list[0].hp = 1;
    assert!(hands_attack(&mut c, player, Facing::Right).is_empty());
    c.level_up();
    assert_eq!((c.level, c.health.hp()), (1, 0), "쓰러지면 레벨도 체력도 그대로다");
}
