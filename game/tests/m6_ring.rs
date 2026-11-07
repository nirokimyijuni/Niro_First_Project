//! US-067 고리 맵 시험 (수산나): 집 오른쪽이 해변 왼쪽으로 이어진다.
//! 기준: stories/backlog.md US-067, 임시-21, docs/02-plan.md D61.
//! 측정: `timeout 300 cargo test --test m6_ring -- --ignored --nocapture`

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::combat::{attack_box_with_range, is_reached, Combat, CombatEvent};
use logic::game::{Game, GameEvent};
use logic::monster::{spawn_x_on_side, walk_x, Zombie};
use logic::player::{Facing, Player, PlayerInput};
use logic::skeleton::{Projectile, ProjectileKind, Skeleton, SkeletonStage};
use logic::weapon::{DroppedWeapon, Weapon};
use logic::world::{nearest_copy, really_overlaps, shortest_dx, World};
use macroquad::math::{Rect, Vec2};

const FRAME: f32 = 1.0 / 60.0;
const MANY_SEEDS: u32 = 500;

fn flat_ring() -> World {
    World { width: MAP_WIDTH, ground_y: GROUND_Y, obstacles: vec![] }
}

fn ring_with(obstacles: Vec<Rect>) -> World {
    World { width: MAP_WIDTH, ground_y: GROUND_Y, obstacles }
}

fn box_at(x: f32) -> Rect {
    Rect::new(x, GROUND_Y - BOX_BIG_SIZE, BOX_BIG_SIZE, BOX_BIG_SIZE)
}

fn body_at(x: f32) -> Rect {
    Rect::new(x, GROUND_Y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT)
}

fn stand_at(x: f32, w: &World) -> Player {
    let mut p = Player::new(CharacterKind::Niro, Vec2::new(x, w.ground_y - PLAYER_HEIGHT));
    p.update(&PlayerInput::default(), FRAME, w);
    p
}

fn run(p: &mut Player, input: PlayerInput, seconds: f32, w: &World) {
    for _ in 0..(seconds / FRAME) as usize {
        p.update(&input, FRAME, w);
    }
}

fn hold_right() -> PlayerInput {
    PlayerInput { right: true, ..Default::default() }
}

fn hold_left() -> PlayerInput {
    PlayerInput { left: true, ..Default::default() }
}

fn night_combat() -> Combat {
    let mut c = Combat::new(7);
    c.skeleton_chance = 0.0;
    c
}

/// 밤의 한 프레임.
fn night_frame(c: &mut Combat, player: Rect, w: &World) -> Vec<CombatEvent> {
    c.update(FRAME, true, player, w)
}

// ---- 공통 도우미 ----

#[test]
fn shortest_distance_takes_the_short_way_around() {
    assert_eq!(shortest_dx(100.0, 3700.0), -240.0);
    assert_eq!(shortest_dx(3700.0, 100.0), 240.0);
    assert_eq!(shortest_dx(100.0, 200.0), 100.0);
    assert_eq!(shortest_dx(0.0, MAP_WIDTH / 2.0), MAP_WIDTH / 2.0);
    assert_eq!(shortest_dx(-10.0, MAP_WIDTH + 10.0), 20.0, "한 바퀴 넘은 값도 된다");
}

#[test]
fn m1_map_distances_never_change() {
    for from in [0.0, 0.1, 333.3, WORLD_WIDTH] {
        for to in [0.0, 0.7, 500.1, WORLD_WIDTH] {
            assert_eq!(shortest_dx(from, to), to - from, "오차 없이 그대로");
        }
    }
    let m1 = World::m1();
    assert_eq!(m1.wrap_x(-5.0), -5.0);
    assert_eq!(m1.keep_on_map(-5.0, PLAYER_WIDTH), 0.0);
    assert_eq!(m1.keep_on_map(9999.0, PLAYER_WIDTH), m1.width - PLAYER_WIDTH);
}

#[test]
fn wrapped_x_is_always_inside_the_map() {
    let w = flat_ring();
    for x in [-0.00001, -1.0, 0.0, 3839.9, MAP_WIDTH, MAP_WIDTH + 1.0, MAP_WIDTH * 3.0 + 5.0, -MAP_WIDTH * 2.0 - 5.0] {
        assert!((0.0..MAP_WIDTH).contains(&w.wrap_x(x)), "{x} -> {}", w.wrap_x(x));
    }
    assert_eq!(w.wrap_x(MAP_WIDTH + 5.0), 5.0);
    assert_eq!(w.wrap_x(-5.0), MAP_WIDTH - 5.0);
}

#[test]
fn nearest_copy_is_one_place_only() {
    // 화면이 이음매에 걸쳐 있어도 네모 하나는 한 자리에만 그려진다.
    let camera_left: f32 = -300.0;
    let middle = camera_left + SCREEN_WIDTH / 2.0;
    assert_eq!(nearest_copy(box_at(3700.0), middle).x, 3700.0 - MAP_WIDTH);
    assert_eq!(nearest_copy(box_at(100.0), middle).x, 100.0);
    assert_eq!(nearest_copy(box_at(-140.0), 3000.0).x, -140.0 + MAP_WIDTH);
}

// ---- 이어지기 ----

#[test]
fn walking_right_goes_through_the_seam_to_the_beach() {
    let w = flat_ring();
    let mut p = stand_at(MAP_WIDTH - 100.0, &w);
    assert_eq!(w.place_name_at(p.rect().center().x), "집");
    run(&mut p, hold_right(), 1.0, &w);
    assert!(p.pos.x < 300.0, "{}", p.pos.x);
    assert_eq!(w.place_name_at(p.rect().center().x), "해변");
}

#[test]
fn walking_left_goes_through_the_seam_to_the_house() {
    let w = flat_ring();
    let mut p = stand_at(50.0, &w);
    run(&mut p, hold_left(), 1.0, &w);
    assert!(p.pos.x > MAP_WIDTH - 300.0, "{}", p.pos.x);
    assert_eq!(w.place_name_at(p.rect().center().x), "집");
}

#[test]
fn body_on_the_seam_is_named_by_its_middle() {
    let w = flat_ring();
    assert_eq!(w.place_name_at(body_at(MAP_WIDTH - 10.0).center().x), "해변", "몸 가운데가 이음매를 넘었다");
    assert_eq!(w.place_name_at(body_at(MAP_WIDTH - 30.0).center().x), "집");
}

#[test]
fn start_spot_is_unchanged() {
    let g = Game::new(CharacterKind::Niro, 1);
    assert_eq!((g.player.pos.x, PLAYER_SPAWN_X), (120.0, 120.0));
    assert_eq!(g.place_name(), "해변");
}

#[test]
fn player_stays_at_the_same_screen_spot_while_crossing() {
    // 로건이 쓰는 방법 그대로: nearest_copy(몸, 화면 가운데).x - camera_left
    for input in [hold_right(), hold_left()] {
        let w = flat_ring();
        let start = if input.right { MAP_WIDTH - 100.0 } else { 50.0 };
        let mut p = stand_at(start, &w);
        let mut crossed = false;
        for _ in 0..60 {
            let before = p.pos.x;
            p.update(&input, FRAME, &w);
            let left = w.camera_left(p.rect().center().x);
            let screen_x = nearest_copy(p.rect(), left + SCREEN_WIDTH / 2.0).x - left;
            assert!((screen_x - (SCREEN_WIDTH - PLAYER_WIDTH) / 2.0).abs() < 0.01, "화면이 튄다: {screen_x}");
            crossed |= (p.pos.x - before).abs() > MAP_WIDTH / 2.0;
        }
        assert!(crossed);
    }
}

#[test]
fn crossing_keeps_clock_coins_health_level_and_weapons() {
    let mut g = Game::new(CharacterKind::Niro, 3);
    g.wallet.add(12);
    g.combat.level_up();
    g.switch_weapon();
    g.player.pos.x = MAP_WIDTH - 100.0;
    let (hp, level, held, day) = (g.combat.health.hp(), g.combat.level, g.weapons.held(), g.clock.day_number());
    let seconds_left = g.clock.seconds_left_in_phase();
    for _ in 0..60 {
        g.update(&hold_right(), FRAME);
    }
    assert!(g.player.pos.x < 300.0, "이음매를 넘었다");
    assert_eq!((g.combat.health.hp(), g.combat.level, g.weapons.held()), (hp, level, held));
    assert_eq!((g.wallet.coins(), g.clock.day_number()), (12, day));
    assert!(g.clock.seconds_left_in_phase() < seconds_left, "시계는 끊기지 않고 흐른다");
}

#[test]
fn no_box_sits_near_the_seam_on_any_map() {
    for seed in 0..MANY_SEEDS {
        let w = World::long_map(seed.wrapping_mul(7919));
        for b in w.obstacles.iter() {
            assert!(b.x >= BOX_AREA_LEFT && b.x + b.w <= MAP_WIDTH - (SCREEN_WIDTH - BOX_AREA_RIGHT), "seed {seed}: {b:?}");
        }
    }
}

// ---- 이음매의 상자와 구조물 ----

#[test]
fn a_box_just_past_the_seam_stops_a_player_coming_from_the_left() {
    let w = ring_with(vec![box_at(5.0)]);
    let mut p = stand_at(MAP_WIDTH - 150.0, &w);
    run(&mut p, hold_right(), 1.5, &w);
    assert_eq!(p.pos.x, MAP_WIDTH + 5.0 - PLAYER_WIDTH, "몸이 이음매에 걸친 채 상자 앞에 선다");
}

#[test]
fn a_box_just_before_the_seam_stops_a_player_coming_from_the_right() {
    let b = box_at(MAP_WIDTH - BOX_BIG_SIZE - 5.0);
    let w = ring_with(vec![b]);
    let mut p = stand_at(150.0, &w);
    run(&mut p, hold_left(), 1.5, &w);
    assert_eq!(p.pos.x, MAP_WIDTH - 5.0, "상자 오른쪽 면에 막혀 이음매에 걸친다");
}

#[test]
fn a_box_lying_across_the_seam_blocks_both_ways() {
    let b = box_at(MAP_WIDTH - BOX_BIG_SIZE / 2.0);
    let w = ring_with(vec![b]);
    let mut p = stand_at(MAP_WIDTH - 200.0, &w);
    run(&mut p, hold_right(), 1.5, &w);
    assert_eq!(p.pos.x, b.x - PLAYER_WIDTH);

    let mut p = stand_at(200.0, &w);
    run(&mut p, hold_left(), 1.5, &w);
    assert_eq!(p.pos.x, BOX_BIG_SIZE / 2.0);
}

#[test]
fn player_can_stand_on_a_box_past_the_seam_and_on_one_across_it() {
    for box_x in [5.0, MAP_WIDTH - BOX_BIG_SIZE / 2.0, MAP_WIDTH - BOX_BIG_SIZE - 5.0] {
        let b = box_at(box_x);
        let w = ring_with(vec![b]);
        // 몸 가운데가 상자 가운데에 오도록, 이음매 쪽에서 내려놓는다.
        let x = w.wrap_x(b.center().x - PLAYER_WIDTH / 2.0);
        let mut p = Player::new(CharacterKind::Niro, Vec2::new(x, 100.0));
        run(&mut p, PlayerInput::default(), 1.0, &w);
        assert!(p.on_ground, "{box_x}");
        assert_eq!(p.pos.y + PLAYER_HEIGHT, b.y, "{box_x}: 상자 위에 선다");
    }
}

#[test]
fn player_can_jump_over_a_box_past_the_seam() {
    let w = ring_with(vec![box_at(5.0)]);
    let mut p = stand_at(MAP_WIDTH - 150.0, &w);
    for _ in 0..(3.0 / FRAME) as usize {
        let before = p.pos.x;
        p.update(&hold_right(), FRAME, &w);
        if p.pos.x == before {
            p.update(&PlayerInput { jump_pressed: true, ..hold_right() }, FRAME, &w);
        }
    }
    assert!((200.0..MAP_WIDTH / 2.0).contains(&p.pos.x), "넘어갔다: {}", p.pos.x);
}

#[test]
fn a_box_past_the_seam_stops_a_monster() {
    let w = ring_with(vec![box_at(5.0)]);
    let mut x = MAP_WIDTH - 100.0;
    for _ in 0..600 {
        let body = Rect::new(x, GROUND_Y - ZOMBIE_HEIGHT, ZOMBIE_WIDTH, ZOMBIE_HEIGHT);
        x = walk_x(body, 200.0, 2.0, &w);
    }
    assert_eq!(x, MAP_WIDTH + 5.0 - ZOMBIE_WIDTH, "상자 앞에서 막힌다");
}

#[test]
fn a_box_before_the_seam_stops_a_monster_coming_the_other_way() {
    let w = ring_with(vec![box_at(MAP_WIDTH - BOX_BIG_SIZE - 5.0)]);
    let mut x = 100.0;
    for _ in 0..600 {
        let body = Rect::new(x, GROUND_Y - ZOMBIE_HEIGHT, ZOMBIE_WIDTH, ZOMBIE_HEIGHT);
        x = walk_x(body, MAP_WIDTH - 300.0, 2.0, &w);
    }
    assert_eq!(x, MAP_WIDTH - 5.0, "상자 오른쪽 면에서 막힌다");
}

#[test]
fn a_shot_bounces_off_a_box_past_the_seam() {
    let w = ring_with(vec![box_at(5.0)]);
    let mut c = night_combat();
    c.skeletons.projectiles.push(Projectile::new(MAP_WIDTH - 60.0, 1.0, ProjectileKind::Arrow, 2, &w));
    let player = body_at(MAP_WIDTH - 900.0);
    let mut bounced = false;
    for _ in 0..30 {
        let events = night_frame(&mut c, player, &w);
        bounced |= events.iter().any(|e| matches!(e, CombatEvent::ProjectileBounced { .. }));
        if bounced {
            break;
        }
    }
    assert!(bounced);
    let shot = c.skeletons.projectiles.iter().find(|s| s.bounced).unwrap();
    assert_eq!(shot.direction, -1.0);
    let box_face = MAP_WIDTH + 5.0 - ARROW_WIDTH;
    assert!((box_face - 10.0..=box_face).contains(&shot.pos.x), "상자 왼쪽 면에서 튕겨 돌아온다: {}", shot.pos.x);
}

// ---- 몬스터, 화살, 총알 ----

#[test]
fn monster_at_3700_walks_right_to_a_player_at_100() {
    let w = flat_ring();
    let body = Rect::new(3700.0 - ZOMBIE_WIDTH / 2.0, GROUND_Y - ZOMBIE_HEIGHT, ZOMBIE_WIDTH, ZOMBIE_HEIGHT);
    assert!(walk_x(body, 100.0, 2.0, &w) > body.x, "오른쪽으로 걷는다");
    // 고리 반대로도
    let body = Rect::new(100.0 - ZOMBIE_WIDTH / 2.0, GROUND_Y - ZOMBIE_HEIGHT, ZOMBIE_WIDTH, ZOMBIE_HEIGHT);
    assert!(walk_x(body, 3700.0, 2.0, &w) < body.x, "왼쪽으로 걷는다");
}

#[test]
fn zombies_cross_the_seam_both_ways_and_reach_the_player() {
    let w = flat_ring();
    for (zombie_x, player_x) in [(MAP_WIDTH - 200.0, 100.0), (100.0, MAP_WIDTH - 200.0)] {
        let mut c = night_combat();
        c.zombies.list.push(Zombie::new(zombie_x, GROUND_Y, false));
        let player = body_at(player_x);
        let mut hurt = false;
        let mut crossed = false;
        for _ in 0..(10.0 / FRAME) as usize {
            let before = c.zombies.list[0].pos.x;
            let events = night_frame(&mut c, player, &w);
            crossed |= (c.zombies.list[0].pos.x - before).abs() > MAP_WIDTH / 2.0;
            assert!((0.0..MAP_WIDTH).contains(&c.zombies.list[0].pos.x));
            hurt = events.iter().any(|e| matches!(e, CombatEvent::PlayerHurt { .. }));
            if hurt {
                break;
            }
        }
        assert!(crossed, "{zombie_x}: 넘을 때 사라지지 않는다");
        assert!(hurt, "{zombie_x}: 가까운 쪽으로 와서 닿는다");
    }
}

#[test]
fn skeleton_shoots_toward_the_near_side_and_the_arrow_crosses_the_seam() {
    let w = flat_ring();
    for (skeleton_x, player_x, direction) in [(MAP_WIDTH - 150.0, 80.0, 1.0), (80.0, MAP_WIDTH - 150.0, -1.0)] {
        let mut c = night_combat();
        c.skeletons.list.push(Skeleton::new(skeleton_x, GROUND_Y, SkeletonStage::Normal));
        let player = body_at(player_x);
        let mut hurt = false;
        let mut seen_direction = None;
        for _ in 0..((ARROW_INTERVAL_SECONDS + 3.0) / FRAME) as usize {
            let events = night_frame(&mut c, player, &w);
            seen_direction = seen_direction.or(c.skeletons.projectiles.first().map(|s| s.direction));
            hurt = events.iter().any(|e| matches!(e, CombatEvent::PlayerHurt { .. }));
            if hurt {
                break;
            }
        }
        assert_eq!(seen_direction, Some(direction), "{skeleton_x}: 가까운 쪽으로 쏜다");
        assert!(hurt, "{skeleton_x}: 화살이 이음매를 넘어 맞는다");
    }
}

#[test]
fn skeleton_stops_at_shooting_distance_measured_across_the_seam() {
    let w = flat_ring();
    let mut c = night_combat();
    c.skeletons.list.push(Skeleton::new(MAP_WIDTH - 100.0, GROUND_Y, SkeletonStage::Normal));
    let player = body_at(50.0);
    let before = c.skeletons.list[0].pos.x;
    night_frame(&mut c, player, &w);
    assert_eq!(c.skeletons.list[0].pos.x, before, "쏘기 좋은 거리라 걷지 않는다");
}

#[test]
fn skeleton_across_the_seam_counts_as_on_screen() {
    let w = flat_ring();
    assert!(w.is_on_screen(MAP_WIDTH - 100.0, 100.0));
    assert!(w.is_on_screen(100.0, MAP_WIDTH - 100.0));
    assert!(!w.is_on_screen(MAP_WIDTH - 500.0, 100.0));
    assert!(!w.is_on_screen(1000.0, 100.0));
}

#[test]
fn attacks_reach_across_the_seam_both_ways() {
    let target = |x: f32| Rect::new(x, GROUND_Y - ZOMBIE_HEIGHT, ZOMBIE_WIDTH, ZOMBIE_HEIGHT);
    let near_end = body_at(MAP_WIDTH - PLAYER_WIDTH - 5.0);
    assert!(is_reached(near_end, Facing::Right, PLAYER_ATTACK_RANGE, target(10.0)));
    assert!(!is_reached(near_end, Facing::Left, PLAYER_ATTACK_RANGE, target(10.0)), "등 뒤");
    let near_start = body_at(5.0);
    assert!(is_reached(near_start, Facing::Left, PLAYER_ATTACK_RANGE, target(MAP_WIDTH - 60.0)));
    assert!(!is_reached(near_start, Facing::Right, PLAYER_ATTACK_RANGE, target(MAP_WIDTH - 60.0)), "등 뒤");
    assert!(is_reached(body_at(MAP_WIDTH - 200.0), Facing::Right, GUN_RANGE, target(30.0)), "총");
    assert!(!is_reached(body_at(MAP_WIDTH - 500.0), Facing::Right, GUN_RANGE, target(30.0)), "총이 안 닿는 거리");
    // 몸이 이음매에 걸쳐 있을 때
    let on_seam = body_at(MAP_WIDTH - 20.0);
    assert!(is_reached(on_seam, Facing::Right, PLAYER_ATTACK_RANGE, target(40.0)));
    assert!(attack_box_with_range(on_seam, Facing::Right, PLAYER_ATTACK_RANGE).x > MAP_WIDTH - 1.0);
}

#[test]
fn a_real_attack_defeats_a_zombie_across_the_seam() {
    let mut c = night_combat();
    c.zombies.list.push(Zombie::new(10.0, GROUND_Y, false));
    let events = c.attack_with(body_at(MAP_WIDTH - PLAYER_WIDTH - 5.0), Facing::Right, ZOMBIE_HP, PLAYER_ATTACK_RANGE);
    assert!(events.iter().any(|e| matches!(e, CombatEvent::ZombieDefeated { .. })), "{events:?}");
    assert!(c.zombies.list.is_empty());
}

#[test]
fn hand_touch_and_blast_work_across_the_seam() {
    let w = flat_ring();
    // 손
    let mut c = night_combat();
    c.skeletons.list.push(Skeleton::new(MAP_WIDTH - SKELETON_WIDTH - 10.0, GROUND_Y, SkeletonStage::Half));
    let events = night_frame(&mut c, body_at(10.0), &w);
    assert!(events.iter().any(|e| matches!(e, CombatEvent::SkeletonHandHit { .. })), "손: {events:?}");

    // 닿기: 몸이 이음매에 걸친 좀비와 맵 왼쪽 주인공
    let mut c = night_combat();
    c.zombies.list.push(Zombie::new(MAP_WIDTH - 20.0, GROUND_Y, false));
    let events = night_frame(&mut c, body_at(5.0), &w);
    assert!(events.iter().any(|e| matches!(e, CombatEvent::PlayerHurt { .. })), "닿기: {events:?}");

    // 펑
    let mut c = night_combat();
    let at = Vec2::new(MAP_WIDTH - 30.0, GROUND_Y - PLAYER_HEIGHT / 2.0);
    c.zombies.list.push(Zombie::new(20.0, GROUND_Y, false));
    let events = c.citizen_blast(at, body_at(10.0));
    assert!(events.iter().any(|e| matches!(e, CombatEvent::PlayerHurt { .. })), "펑: 주인공 {events:?}");
    assert!(c.zombies.list.is_empty(), "펑: 몬스터");
    let events = c.citizen_blast(at, body_at(MAP_WIDTH / 2.0));
    assert!(events.is_empty(), "먼 곳은 안 맞는다");
}

#[test]
fn forgetting_distance_is_measured_the_short_way() {
    let w = flat_ring();
    let player = body_at(100.0);
    let mut c = night_combat();
    c.zombies.list.push(Zombie::new(MAP_WIDTH - 1000.0, GROUND_Y, false)); // 짧은 쪽으로 약 1,100
    c.zombies.list.push(Zombie::new(2000.0, GROUND_Y, false)); // 어느 쪽으로도 1,200이 넘는다
    c.skeletons.list.push(Skeleton::new(MAP_WIDTH - 1000.0, GROUND_Y, SkeletonStage::Normal));
    c.skeletons.list.push(Skeleton::new(2000.0, GROUND_Y, SkeletonStage::Normal));
    c.skeletons.projectiles.push(Projectile::new(MAP_WIDTH - 1000.0, -1.0, ProjectileKind::Arrow, 2, &w));
    c.skeletons.projectiles.push(Projectile::new(2000.0, 1.0, ProjectileKind::Arrow, 2, &w));
    night_frame(&mut c, player, &w);
    let near = |x: f32| shortest_dx(player.center().x, x).abs() <= FORGET_DISTANCE;
    assert_eq!(c.skeletons.list.len(), 1);
    assert_eq!(c.skeletons.projectiles.len(), 1);
    assert!(c.zombies.list.iter().all(|z| near(z.rect().center().x)));
    assert!(c.zombies.list.iter().any(|z| z.pos.x > MAP_WIDTH / 2.0), "이음매 너머 가까운 좀비는 남는다");
}

#[test]
fn bounced_shot_is_dodged_across_the_seam() {
    let w = flat_ring();
    let mut c = night_combat();
    c.skeletons.list.push(Skeleton::new(10.0, GROUND_Y, SkeletonStage::Normal));
    let mut shot = Projectile::new(MAP_WIDTH - 40.0, 1.0, ProjectileKind::Arrow, 2, &w);
    shot.bounced = true;
    c.skeletons.projectiles.push(shot);
    let events = night_frame(&mut c, body_at(MAP_WIDTH - 900.0), &w);
    assert!(events.iter().any(|e| matches!(e, CombatEvent::SkeletonDodged { .. })), "{events:?}");
}

// ---- 땅의 물건 ----

#[test]
fn a_weapon_falling_on_the_seam_stays_on_the_map_and_can_be_picked_up() {
    let w = flat_ring();
    let dropped = DroppedWeapon::above(Weapon::Sword, 5.0, &w);
    assert_eq!(dropped.rect.x, MAP_WIDTH + 5.0 - WEAPON_SIZE / 2.0, "왼쪽 몸이 이음매 너머로 감긴다");

    let mut g = Game::new(CharacterKind::Niro, 3);
    g.weapons.dropped.push(DroppedWeapon::above(Weapon::Axe, 5.0, &g.world));
    g.player.pos.x = 0.0;
    let mut picked = false;
    for _ in 0..(3.0 / FRAME) as usize {
        let events = g.update(&PlayerInput::default(), FRAME);
        picked |= events.contains(&GameEvent::WeaponPicked { kind: Weapon::Axe });
    }
    assert!(picked, "이음매에 걸친 무기를 맵 왼쪽에서 줍는다");
    assert!(g.weapons.dropped.is_empty());
}

#[test]
fn a_weapon_landing_past_the_seam_lands_on_the_box_there() {
    let w = ring_with(vec![box_at(MAP_WIDTH - BOX_BIG_SIZE / 2.0)]);
    let dropped = DroppedWeapon::above(Weapon::Sword, 5.0, &w);
    assert_eq!(w.surface_y_under(dropped.rect), GROUND_Y - BOX_BIG_SIZE, "이음매에 걸친 상자 위로 떨어진다");
}

#[test]
fn walking_citizens_stay_in_their_place() {
    let mut g = Game::new(CharacterKind::Niro, 11);
    g.player.pos.x = 300.0;
    for _ in 0..(180.0 / FRAME) as usize {
        g.citizens.walk(FRAME, &g.world);
        for (place, walker) in g.citizens.walkers.iter().enumerate() {
            let left = place as f32 * SCREEN_WIDTH;
            assert!(left <= walker.rect.x && walker.rect.x + walker.rect.w <= left + SCREEN_WIDTH, "{place}: {:?}", walker.rect);
        }
    }
}

// ---- 몬스터 나오기 ----

fn appeared_xs(events: &[CombatEvent]) -> Vec<f32> {
    events
        .iter()
        .filter_map(|e| match e {
            CombatEvent::ZombieAppeared { at } => Some(at.x),
            CombatEvent::SkeletonAppeared { at, .. } => Some(at.x),
            _ => None,
        })
        .collect()
}

/// `left` 쪽에 정말 나올 자리가 있는지. 없으면 `spawn_x_on_side`는 반대쪽을 준다.
fn side_is_open(left: bool, player: Rect, w: &World) -> bool {
    spawn_x_on_side(left, player, ZOMBIE_WIDTH, w, &[]).is_some_and(|x| (shortest_dx(player.center().x, x) < 0.0) == left)
}

/// 땅의 벽(상자, 구조물, 구멍)에 닿지 않은 주인공 자리들.
fn ground_spots(w: &World) -> Vec<Rect> {
    (0..MAP_WIDTH as usize)
        .step_by(97)
        .map(|x| body_at(x as f32))
        .filter(|body| !w.monster_walls().any(|wall| really_overlaps(wall, *body)))
        .collect()
}

#[test]
fn monsters_come_from_both_open_sides_on_every_map() {
    for seed in 0..MANY_SEEDS {
        let w = World::long_map(seed.wrapping_mul(7919));
        for player_x in [0.0, 30.0, PLAYER_SPAWN_X, MAP_WIDTH - PLAYER_WIDTH, MAP_WIDTH - 10.0] {
            let player = body_at(player_x);
            let xs = appeared_xs(&Combat::new(seed).update(FRAME, true, player, &w));
            assert!(xs.len() <= MONSTERS_PER_SPAWN);
            if side_is_open(true, player, &w) && side_is_open(false, player, &w) {
                assert_eq!(xs.len(), MONSTERS_PER_SPAWN, "seed {seed}, x {player_x}");
                let (a, b) = (shortest_dx(player.center().x, xs[0]), shortest_dx(player.center().x, xs[1]));
                assert!(a * b < 0.0, "seed {seed}, x {player_x}: 양쪽에서 1마리씩 {xs:?}");
            }
            let center = player.center().x;
            assert!(xs.iter().all(|x| shortest_dx(center, *x).abs() >= MONSTER_SPAWN_MIN_DISTANCE), "seed {seed}, x {player_x}: {xs:?}");
        }
    }
}

#[test]
fn a_monster_that_appears_can_always_walk_to_a_player_on_the_ground() {
    for seed in 0..100u32 {
        let w = World::long_map(seed.wrapping_mul(7919));
        for player in ground_spots(&w) {
            for from_left in [true, false] {
                let Some(mut x) = spawn_x_on_side(from_left, player, ZOMBIE_WIDTH, &w, &[]) else {
                    continue;
                };
                let body = |x: f32| Rect::new(x, GROUND_Y - ZOMBIE_HEIGHT, ZOMBIE_WIDTH, ZOMBIE_HEIGHT);
                assert!(!w.monster_walls().any(|wall| really_overlaps(wall, body(x))), "seed {seed}: 벽 안에서 나왔다 {x}");
                for _ in 0..(SCREEN_WIDTH / 24.0) as usize {
                    if really_overlaps(body(x), player) {
                        break;
                    }
                    x = walk_x(body(x), player.center().x, 24.0, &w);
                }
                assert!(really_overlaps(body(x), player), "seed {seed}, {player:?}: {x}에서 못 온다");
            }
        }
    }
}

#[test]
fn a_box_between_makes_the_monster_appear_on_the_players_side() {
    let b = box_at(2000.0);
    let w = ring_with(vec![b]);
    let player = body_at(2500.0);
    assert_eq!(spawn_x_on_side(true, player, ZOMBIE_WIDTH, &w, &[]), Some(b.x + b.w), "상자 바로 안쪽");
    let w = ring_with(vec![box_at(2900.0)]);
    let expected = 2900.0 - ZOMBIE_WIDTH;
    assert_eq!(spawn_x_on_side(false, player, ZOMBIE_WIDTH, &w, &[]), Some(expected));
}

#[test]
fn spawn_distance_is_measured_between_body_middles_across_the_seam() {
    // 몸 사이는 160보다 좁아도 가운데끼리 160이면 된다. 상자가 이음매 너머에 있다.
    let player = body_at(50.0);
    let box_x = MAP_WIDTH + player.center().x - MONSTER_SPAWN_MIN_DISTANCE - ZOMBIE_WIDTH / 2.0 - BOX_BIG_SIZE;
    let w = ring_with(vec![box_at(box_x)]);
    let x = spawn_x_on_side(true, player, ZOMBIE_WIDTH, &w, &[]).unwrap();
    assert_eq!(x, box_x + BOX_BIG_SIZE);
    assert_eq!(shortest_dx(player.center().x, x + ZOMBIE_WIDTH / 2.0), -MONSTER_SPAWN_MIN_DISTANCE);
}

#[test]
fn nearest_of_two_boxes_is_used_even_across_the_seam() {
    let w = ring_with(vec![box_at(MAP_WIDTH - 300.0), box_at(MAP_WIDTH - 100.0)]);
    let player = body_at(300.0);
    assert_eq!(spawn_x_on_side(true, player, ZOMBIE_WIDTH, &w, &[]), Some(MAP_WIDTH - 100.0 + BOX_BIG_SIZE));
}

#[test]
fn distance_limit_sends_the_monster_to_the_other_side_or_skips_the_turn() {
    let player = body_at(2500.0);
    // 몬스터 몸 가운데가 주인공 몸 가운데에서 딱 160 떨어지는 상자 자리.
    let exactly_far_enough = player.center().x - MONSTER_SPAWN_MIN_DISTANCE - ZOMBIE_WIDTH / 2.0 - BOX_BIG_SIZE;
    let w = ring_with(vec![box_at(exactly_far_enough)]);
    assert_eq!(spawn_x_on_side(true, player, ZOMBIE_WIDTH, &w, &[]), Some(exactly_far_enough + BOX_BIG_SIZE), "딱 160은 된다");

    let w = ring_with(vec![box_at(exactly_far_enough + 1.0)]);
    let x = spawn_x_on_side(true, player, ZOMBIE_WIDTH, &w, &[]).unwrap();
    assert!(x > player.x, "160보다 가까우면 반대쪽에서 나온다: {x}");

    let w = ring_with(vec![box_at(player.x - BOX_BIG_SIZE - 50.0), box_at(player.x + PLAYER_WIDTH + 50.0)]);
    assert_eq!(spawn_x_on_side(true, player, ZOMBIE_WIDTH, &w, &[]), None);
    assert_eq!(spawn_x_on_side(false, player, ZOMBIE_WIDTH, &w, &[]), None);
}

#[test]
fn skipped_turn_waits_for_the_next_turn() {
    let player = body_at(2500.0);
    let closed = ring_with(vec![box_at(player.x - BOX_BIG_SIZE - 50.0), box_at(player.x + PLAYER_WIDTH + 50.0)]);
    let mut c = night_combat();
    assert!(appeared_xs(&c.update(FRAME, true, player, &closed)).is_empty(), "양쪽 다 없으면 건너뛴다");
    let open = flat_ring();
    assert!(appeared_xs(&c.update(1.0, true, player, &open)).is_empty(), "바로 다시 나오지 않는다");
    let mut appeared = Vec::new();
    for _ in 0..(ZOMBIE_SPAWN_SECONDS / FRAME) as usize {
        appeared.extend(appeared_xs(&c.update(FRAME, true, player, &open)));
    }
    assert_eq!(appeared.len(), MONSTERS_PER_SPAWN, "다음 차례에 나온다");
}

#[test]
fn both_monsters_from_one_side_do_not_overlap_and_keep_the_distance() {
    // 왼쪽이 막혀 둘 다 오른쪽 차례가 된다.
    let player = body_at(2500.0);
    let w = ring_with(vec![box_at(player.x - BOX_BIG_SIZE - 50.0)]);
    let xs = appeared_xs(&night_combat().update(FRAME, true, player, &w));
    assert_eq!(xs.len(), MONSTERS_PER_SPAWN);
    assert!(xs.iter().all(|x| *x > player.x), "{xs:?}");
    assert!((xs[0] - xs[1]).abs() >= ZOMBIE_WIDTH, "{xs:?}");

    // 오른쪽 자리도 좁으면 한 마리만 나온다.
    let tight = player.center().x + MONSTER_SPAWN_MIN_DISTANCE + ZOMBIE_WIDTH + 10.0;
    let w = ring_with(vec![box_at(player.x - BOX_BIG_SIZE - 50.0), box_at(tight)]);
    assert_eq!(appeared_xs(&night_combat().update(FRAME, true, player, &w)).len(), 1);
}

#[test]
fn skeletons_use_the_same_spots() {
    let b = box_at(2000.0);
    let w = ring_with(vec![b]);
    let player = body_at(2500.0);
    assert_eq!(spawn_x_on_side(true, player, SKELETON_WIDTH, &w, &[]), Some(b.x + b.w));
    let mut c = Combat::new(7);
    c.skeleton_chance = 1.0;
    let xs = appeared_xs(&c.update(FRAME, true, player, &w));
    let left_x = b.x + b.w + SKELETON_WIDTH / 2.0;
    assert!(xs.contains(&left_x), "{xs:?}");
}

#[test]
fn player_on_a_box_gets_monsters_outside_the_touching_boxes_under_them() {
    let (a, b) = (box_at(2500.0), box_at(2500.0 + BOX_BIG_SIZE));
    let w = ring_with(vec![a, b]);
    let mut player = body_at(2500.0);
    player.y = a.y - PLAYER_HEIGHT;
    let left = spawn_x_on_side(true, player, ZOMBIE_WIDTH, &w, &[]).unwrap();
    let right = spawn_x_on_side(false, player, ZOMBIE_WIDTH, &w, &[]).unwrap();
    assert!(left + ZOMBIE_WIDTH <= a.x, "{left}");
    assert!(right >= b.x + b.w, "{right}");
}

#[test]
fn player_in_the_hole_gets_monsters_outside_the_hole() {
    let w = World::long_map(1);
    let hole = w.hole().unwrap();
    let player = Rect::new(hole.center().x - PLAYER_WIDTH / 2.0, GROUND_Y + HOLE_DEPTH - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT);
    for from_left in [true, false] {
        if let Some(x) = spawn_x_on_side(from_left, player, ZOMBIE_WIDTH, &w, &[]) {
            assert!(x + ZOMBIE_WIDTH <= hole.x || x >= hole.x + hole.w, "구멍 안에서 나오지 않는다: {x}");
        }
    }
}

#[test]
fn same_seed_gives_the_same_monsters_on_a_real_map() {
    let night = || {
        let w = World::long_map(7919);
        let mut c = Combat::new(5);
        let mut xs = Vec::new();
        for _ in 0..(40.0 / FRAME) as usize {
            xs.extend(appeared_xs(&c.update(FRAME, true, body_at(PLAYER_SPAWN_X), &w)));
        }
        xs
    };
    assert_eq!(night(), night());
    assert!(!night().is_empty());
}

/// 가만히 선 주인공에게 화면 왼쪽 바깥에서 나온 좀비가 걸어서 닿는지 본다. 상자에 막히면 false다.
fn zombie_from_the_left_reaches(player_x: f32, seed: u32) -> bool {
    let w = World::long_map(seed.wrapping_mul(7919));
    let player = body_at(player_x);
    let Some(mut x) = spawn_x_on_side(true, player, ZOMBIE_WIDTH, &w, &[]) else {
        return false;
    };
    if shortest_dx(player.center().x, x) > 0.0 {
        return false; // 왼쪽에 자리가 없어서 오른쪽에서 나왔다
    }
    for _ in 0..(SCREEN_WIDTH * 2.0 / 2.0) as usize {
        let body = Rect::new(x, GROUND_Y - ZOMBIE_HEIGHT, ZOMBIE_WIDTH, ZOMBIE_HEIGHT);
        if really_overlaps(body, player) {
            return true;
        }
        let next = walk_x(body, player.center().x, 2.0, &w);
        if next == x {
            return false;
        }
        x = next;
    }
    false
}

#[test]
fn no_safe_spot_at_the_old_left_end_without_boxes() {
    let w = World::long_map(1);
    let mut no_boxes = World { width: w.width, ground_y: w.ground_y, obstacles: w.obstacles.clone() };
    let boxes = w.boxes();
    no_boxes.obstacles.retain(|b| !boxes.contains(b));
    for player_x in [0.0, 30.0, PLAYER_SPAWN_X] {
        let mut c = night_combat();
        let player = body_at(player_x);
        let mut hurt_from_left = false;
        c.zombies.list.push(Zombie::new(no_boxes.wrap_x(player_x - SCREEN_WIDTH / 2.0 - ZOMBIE_WIDTH), GROUND_Y, false));
        for _ in 0..(10.0 / FRAME) as usize {
            let events = night_frame(&mut c, player, &no_boxes);
            if events.iter().any(|e| matches!(e, CombatEvent::PlayerHurt { .. })) {
                hurt_from_left = c.zombies.list.iter().any(|z| shortest_dx(player.center().x, z.rect().center().x) < 0.0);
                break;
            }
        }
        assert!(hurt_from_left, "x {player_x}: 왼쪽에서 온 좀비가 닿는다");
    }
}

#[test]
#[ignore]
fn measure_left_end_safe_zone() {
    for player_x in [0.0, 30.0, PLAYER_SPAWN_X, 300.0, 1500.0] {
        let reached = (0..MANY_SEEDS).filter(|seed| zombie_from_the_left_reaches(player_x, *seed)).count();
        println!("x={player_x}: 왼쪽에서 나온 좀비가 닿는 판 {reached}/{MANY_SEEDS}");
    }
}
