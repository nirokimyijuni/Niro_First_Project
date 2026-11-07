//! M6 긴 맵의 몬스터 테스트 (수산나): 화면 바깥에서 나오기, 멀어지면 사라지기, 화면 안에서만 쏘기.
//! 기준: stories/backlog.md 임시-1, 임시-3, 임시-4, US-050. 엘리 관찰(왼쪽 끝 35초).

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::combat::{Combat, CombatEvent};
use logic::game::Game;
use logic::monster::{pick_left_side, spawn_x_on_side, SeededRandom, Zombie};
use logic::player::PlayerInput;
use logic::skeleton::{Projectile, ProjectileKind, Skeleton, SkeletonStage};
use logic::world::{really_overlaps, World};
use macroquad::math::Rect;

const FRAME: f32 = 1.0 / 60.0;

/// 구조물만 있는 긴 맵. 상자가 끼어들지 않게 한다.
fn map_without_boxes() -> World {
    let mut w = World::long_map(1);
    let boxes = w.boxes();
    w.obstacles.retain(|b| !boxes.contains(b));
    w
}

fn body_at(x: f32, w: &World) -> Rect {
    Rect::new(x, w.ground_y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT)
}

fn spawn_xs(player: Rect, w: &World, tries: u32) -> Vec<f32> {
    (0..tries).map(|seed| spawn_x_on_side(pick_left_side(&mut SeededRandom::new(seed)), player, ZOMBIE_WIDTH, w, &[])).collect()
}

// ---- 임시-1 나오는 자리 ----

#[test]
fn monsters_appear_just_outside_the_screen_on_both_sides_anywhere_in_the_middle() {
    let w = map_without_boxes();
    for player_x in [700.0, 1100.0, 2300.0, 3000.0] {
        let player = body_at(player_x, &w);
        let left = w.camera_left(player.center().x);
        let xs = spawn_xs(player, &w, 40);
        assert!(xs.iter().any(|x| *x + ZOMBIE_WIDTH <= left), "왼쪽 바깥: {xs:?}");
        assert!(xs.iter().any(|x| *x >= left + SCREEN_WIDTH), "오른쪽 바깥: {xs:?}");
        assert!(xs.iter().all(|x| *x + ZOMBIE_WIDTH <= left || *x >= left + SCREEN_WIDTH), "몸이 안 보인다: {xs:?}");
    }
}

#[test]
fn half_and_half_like_before() {
    let w = map_without_boxes();
    let player = body_at(1200.0, &w);
    let left = w.camera_left(player.center().x);
    let xs = spawn_xs(player, &w, 1000);
    let on_left = xs.iter().filter(|x| **x < left).count();
    assert!((400..600).contains(&on_left), "왼쪽 {on_left}/1000");
}

#[test]
fn at_map_end_the_outer_side_uses_the_map_end_spot() {
    let w = map_without_boxes();
    let near_start = body_at(PLAYER_SPAWN_X, &w);
    let xs = spawn_xs(near_start, &w, 40);
    assert!(xs.contains(&0.0), "왼쪽 바깥이 맵 밖이면 맵 끝: {xs:?}");
    assert!(xs.contains(&SCREEN_WIDTH), "오른쪽은 화면 바로 바깥: {xs:?}");

    let near_end = body_at(MAP_WIDTH - 200.0, &w);
    let xs = spawn_xs(near_end, &w, 40);
    assert!(xs.contains(&(MAP_WIDTH - ZOMBIE_WIDTH)), "{xs:?}");
    assert!(xs.contains(&(MAP_WIDTH - SCREEN_WIDTH - ZOMBIE_WIDTH)), "{xs:?}");
}

#[test]
fn player_on_the_map_end_spot_sends_monsters_to_the_other_side() {
    let w = map_without_boxes();
    let xs = spawn_xs(body_at(0.0, &w), &w, 40);
    assert!(xs.iter().all(|x| *x == SCREEN_WIDTH), "{xs:?}");
}

#[test]
fn spawn_spot_on_a_box_or_hole_moves_farther_from_the_screen() {
    // 화면 왼쪽 바로 바깥이 구멍이 되는 자리: 카메라 왼쪽이 구멍 오른쪽 끝 + 10.
    let w = map_without_boxes();
    let hole = w.hole().unwrap();
    let camera = hole.x + hole.w + 10.0;
    let player = body_at(camera + SCREEN_WIDTH / 2.0 - PLAYER_WIDTH / 2.0, &w);
    let left_side: Vec<f32> = spawn_xs(player, &w, 40).into_iter().filter(|x| *x < camera).collect();
    assert!(!left_side.is_empty());
    assert!(left_side.iter().all(|x| *x + ZOMBIE_WIDTH == hole.x), "구멍 왼쪽 빈 땅: {left_side:?}");

    // 화면 오른쪽 바로 바깥에 상자 두 개가 이어져 있다.
    let mut w = map_without_boxes();
    let player = body_at(100.0 + SCREEN_WIDTH / 2.0, &w);
    let right_x = w.camera_left(player.center().x) + SCREEN_WIDTH;
    let top = w.ground_y - BOX_BIG_SIZE;
    w.obstacles.push(Rect::new(right_x - 10.0, top, BOX_BIG_SIZE, BOX_BIG_SIZE));
    w.obstacles.push(Rect::new(right_x + BOX_BIG_SIZE - 10.0, top, BOX_BIG_SIZE, BOX_BIG_SIZE));
    let xs = spawn_xs(player, &w, 40);
    assert!(xs.contains(&(right_x + BOX_BIG_SIZE * 2.0 - 10.0)), "{xs:?}");
}

#[test]
fn monsters_never_start_inside_a_wall_on_any_map() {
    for seed in 0..200 {
        let w = World::long_map(seed);
        for player_x in (0..3800).step_by(190) {
            let player = body_at(player_x as f32, &w);
            for x in spawn_xs(player, &w, 8) {
                let body = Rect::new(x, w.ground_y - ZOMBIE_HEIGHT, ZOMBIE_WIDTH, ZOMBIE_HEIGHT);
                assert!(!w.monster_walls().any(|wall| really_overlaps(wall, body)), "seed {seed}: {body:?}");
                assert!(x >= 0.0 && x + ZOMBIE_WIDTH <= w.width);
            }
        }
    }
}

#[test]
fn first_monster_appears_right_at_18_even_near_the_house() {
    let w = map_without_boxes();
    let player = body_at(3400.0, &w);
    let mut c = Combat::new(7);
    let events = c.update(FRAME, true, player, &w);
    let appeared = events.iter().any(|e| matches!(e, CombatEvent::ZombieAppeared { .. } | CombatEvent::SkeletonAppeared { .. }));
    assert!(appeared, "{events:?}");
}

// ---- 부모님 피드백: 한 번에 양쪽에서 ----

fn appeared_xs(events: &[CombatEvent]) -> Vec<f32> {
    events
        .iter()
        .filter_map(|e| match e {
            CombatEvent::ZombieAppeared { at } | CombatEvent::SkeletonAppeared { at, .. } => Some(at.x),
            _ => None,
        })
        .collect()
}

#[test]
fn each_turn_brings_one_monster_from_each_side() {
    let w = map_without_boxes();
    let player = body_at(1200.0, &w);
    for seed in 0..50 {
        let xs = appeared_xs(&Combat::new(seed).update(FRAME, true, player, &w));
        assert_eq!(xs.len(), MONSTERS_PER_SPAWN, "seed {seed}");
        assert!(xs.iter().any(|x| *x < player.x) && xs.iter().any(|x| *x > player.x), "seed {seed}: {xs:?}");
    }
}

#[test]
fn player_on_the_end_spot_gets_both_from_the_other_side_without_overlap() {
    let long = map_without_boxes();
    let small = World { width: WORLD_WIDTH, ground_y: GROUND_Y, obstacles: vec![] };
    for w in [&long, &small] {
        for (player_x, end) in [(0.0, "왼쪽"), (w.width - PLAYER_WIDTH, "오른쪽")] {
            for seed in 0..30 {
                let xs = appeared_xs(&Combat::new(seed).update(FRAME, true, body_at(player_x, w), w));
                assert_eq!(xs.len(), 2);
                assert!((xs[0] - xs[1]).abs() >= ZOMBIE_WIDTH, "{end} 끝, seed {seed}: {xs:?}");
                assert!(xs.iter().all(|x| (x - player_x - PLAYER_WIDTH / 2.0).abs() > PLAYER_WIDTH), "{xs:?}");
            }
        }
    }
}

#[test]
fn only_as_many_as_fit_under_the_limit_come() {
    let w = map_without_boxes();
    let player = body_at(1200.0, &w);
    let mut c = Combat::new(7);
    c.skeleton_chance = 0.0;
    for _ in 1..ZOMBIE_MAX_ON_SCREEN {
        c.zombies.list.push(Zombie::new(1000.0, w.ground_y, false));
    }
    for _ in ZOMBIE_MAX_ON_SCREEN..MONSTER_MAX_ON_SCREEN {
        c.skeletons.list.push(Skeleton::new(1400.0, w.ground_y, SkeletonStage::Half));
    }
    assert_eq!(appeared_xs(&c.update(FRAME, true, player, &w)).len(), 1, "자리가 하나면 하나만 나온다");
    assert_eq!(c.zombies.list.len() + c.skeletons.list.len(), MONSTER_MAX_ON_SCREEN);
}

// ---- 임시-3 멀어진 몬스터 ----

#[test]
fn far_monsters_vanish_without_experience_and_new_ones_can_come() {
    let w = map_without_boxes();
    let mut c = Combat::new(7);
    c.skeleton_chance = 0.0;
    for _ in 0..ZOMBIE_MAX_ON_SCREEN {
        c.zombies.list.push(Zombie::new(100.0, w.ground_y, false));
    }
    for _ in ZOMBIE_MAX_ON_SCREEN..MONSTER_MAX_ON_SCREEN {
        c.skeletons.list.push(Skeleton::new(130.0, w.ground_y, SkeletonStage::Normal));
    }
    assert_eq!(c.zombies.list.len() + c.skeletons.list.len(), MONSTER_MAX_ON_SCREEN);

    let events = c.update(FRAME, true, body_at(100.0 + FORGET_DISTANCE + 200.0, &w), &w);
    assert!(c.skeletons.list.is_empty());
    assert!(c.zombies.list.iter().all(|z| z.pos.x > 200.0), "옛 몬스터는 없다");
    assert_eq!((c.experience, c.level), (0, 1), "경험치 없음");
    assert!(!events.iter().any(|e| matches!(e, CombatEvent::ZombieDefeated { .. } | CombatEvent::SkeletonDefeated { .. })));

    // 빈자리가 생겼으니 다음 차례에 나온다.
    let far_player = body_at(1500.0, &w);
    let mut new_monster = !c.zombies.list.is_empty();
    for _ in 0..(ZOMBIE_SPAWN_SECONDS / FRAME) as usize + 2 {
        new_monster |= c.update(FRAME, true, far_player, &w).iter().any(|e| matches!(e, CombatEvent::ZombieAppeared { .. }));
    }
    assert!(new_monster);
}

#[test]
fn monster_just_inside_the_distance_stays() {
    let w = map_without_boxes();
    let mut c = Combat::new(7);
    let player = body_at(1300.0, &w);
    let x = player.center().x - FORGET_DISTANCE + 2.0 - ZOMBIE_WIDTH / 2.0;
    c.zombies.list.push(Zombie::new(x, w.ground_y, false));
    c.update(FRAME, false, player, &w);
    assert_eq!(c.zombies.list.len(), 1);
}

// ---- 임시-4 화면 안에서만 쏘기 ----

/// `seconds`초 동안 쏜 화살·총알 수.
fn count_shots(c: &mut Combat, seconds: f32, player: Rect, w: &World) -> usize {
    let mut shots = 0;
    for _ in 0..(seconds / FRAME) as usize {
        let events = c.update(FRAME, true, player, w);
        shots += events.iter().filter(|e| matches!(e, CombatEvent::ProjectileFired { .. })).count();
    }
    shots
}

#[test]
fn skeleton_off_screen_does_not_shoot_then_shoots_when_it_comes_in() {
    let w = map_without_boxes();
    let mut c = Combat::new(7);
    c.skeleton_chance = 0.0;
    let player = body_at(1000.0, &w);
    let screen_left = w.camera_left(player.center().x);
    // 화면 왼쪽 400픽셀 바깥. 걸어도 시험 시간 안에는 들어오지 못한다.
    let start_x = screen_left - 400.0;
    c.skeletons.list.push(Skeleton::new(start_x, w.ground_y, SkeletonStage::Normal));
    let shots = count_shots(&mut c, ARROW_INTERVAL_SECONDS * 1.5, player, &w);
    assert_eq!(shots, 0, "화면 밖에서는 쏘지 않는다");

    // 화면 안으로 옮기면 곧바로 쏘고, 그 뒤로는 쏘는 간격을 지킨다 (한꺼번에 몰아 쏘지 않는다).
    c.skeletons.list[0].pos.x = screen_left + 200.0;
    let events = c.update(FRAME, true, player, &w);
    assert!(events.iter().any(|e| matches!(e, CombatEvent::ProjectileFired { .. })));
    let shots = count_shots(&mut c, ARROW_INTERVAL_SECONDS * 0.8, player, &w);
    assert_eq!(shots, 0);
}

#[test]
fn shots_vanish_at_the_map_end_and_far_from_the_player() {
    let w = map_without_boxes();
    let mut c = Combat::new(7);
    let player = body_at(100.0, &w);
    let far = player.center().x + FORGET_DISTANCE + 10.0;
    c.skeletons.projectiles.push(Projectile::new(far, 1.0, ProjectileKind::Arrow, 2, &w));
    c.skeletons.projectiles.push(Projectile::new(player.center().x + 500.0, 1.0, ProjectileKind::Arrow, 2, &w));
    c.update(FRAME, true, player, &w);
    assert_eq!(c.skeletons.projectiles.len(), 1, "멀리 간 것만 사라진다");

    c.skeletons.projectiles.clear();
    c.skeletons.projectiles.push(Projectile::new(w.width - 5.0, 1.0, ProjectileKind::Bullet, 4, &w));
    for _ in 0..10 {
        c.update(FRAME, true, body_at(w.width - 300.0, &w), &w);
    }
    assert!(c.skeletons.projectiles.iter().all(|shot| shot.pos.x < w.width), "맵 끝에서 사라진다");
}

#[test]
fn morning_clears_monsters_and_shots_even_off_screen() {
    let w = map_without_boxes();
    let mut c = Combat::new(7);
    let player = body_at(1500.0, &w);
    c.update(FRAME, true, player, &w);
    c.zombies.list.push(Zombie::new(400.0, w.ground_y, false));
    c.skeletons.list.push(Skeleton::new(2600.0, w.ground_y, SkeletonStage::Normal));
    c.skeletons.projectiles.push(Projectile::new(2500.0, 1.0, ProjectileKind::Arrow, 2, &w));
    c.update(FRAME, false, player, &w);
    assert!(c.zombies.list.is_empty() && c.skeletons.list.is_empty() && c.skeletons.projectiles.is_empty());
}

// ---- 엘리 관찰: 왼쪽 끝에서 스켈레톤 옆에 있어도 안 맞음 ----

/// 스켈레톤이 플레이어와 같은 자리에 섰을 때 쏜 공격에 맞는지.
fn hurt_by_skeleton_on_same_spot(x: f32, stage: SkeletonStage, w: &World) -> bool {
    let mut c = Combat::new(7);
    c.skeletons.list.push(Skeleton::new(x, w.ground_y, stage));
    let player = body_at(x, w);
    for _ in 0..(ARROW_INTERVAL_SECONDS * 2.0 / FRAME) as usize {
        // 나올 몬스터가 끼어들지 않게 치운다.
        c.zombies.list.clear();
        c.skeletons.list.truncate(1);
        c.update(FRAME, true, player, w);
    }
    c.health.hp() < PLAYER_START_HP
}

#[test]
fn skeleton_on_the_same_spot_hits_at_both_map_ends_and_in_the_middle() {
    let flat = World { width: WORLD_WIDTH, ground_y: GROUND_Y, obstacles: vec![] };
    for x in [0.0, 400.0, WORLD_WIDTH - SKELETON_WIDTH] {
        for stage in [SkeletonStage::Normal, SkeletonStage::Full] {
            assert!(hurt_by_skeleton_on_same_spot(x, stage, &flat), "x {x}, {stage:?}");
        }
    }
}

#[test]
fn skeleton_slightly_off_the_player_also_hits() {
    // 몸 앞에서 나가면 이 자리의 화살은 첫 걸음에 플레이어를 지나쳐 버린다.
    let flat = World { width: WORLD_WIDTH, ground_y: GROUND_Y, obstacles: vec![] };
    let mut c = Combat::new(7);
    c.skeletons.list.push(Skeleton::new(100.0, GROUND_Y, SkeletonStage::Normal));
    let player = Rect::new(101.0, GROUND_Y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT);
    for _ in 0..(ARROW_INTERVAL_SECONDS * 2.0 / FRAME) as usize {
        c.zombies.list.clear();
        c.skeletons.list.truncate(1);
        c.update(FRAME, true, player, &flat);
    }
    assert!(c.health.hp() < PLAYER_START_HP);
}

#[test]
fn walking_zombie_on_the_left_end_hurts() {
    let w = map_without_boxes();
    let mut g = Game::on_world(CharacterKind::Niro, 7, w);
    g.combat.skeleton_chance = 0.0;
    g.clock.update(DAY_SECONDS);
    g.player.pos.x = 0.0;
    g.combat.zombies.list.push(Zombie::new(0.0, GROUND_Y, false));
    for _ in 0..30 {
        g.update(&PlayerInput::default(), FRAME);
    }
    assert!(g.combat.health.hp() < PLAYER_START_HP, "왼쪽 끝이라고 좀비에게 안전하지 않다");
}
