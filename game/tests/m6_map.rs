//! M6 긴 맵 테스트 (수산나): 장소, 카메라, 상자, 구멍, 나무 발판, 바위 계단.
//! 기준: stories/backlog.md M6 맨 위 표, US-050, US-051.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::game::Game;
use logic::monster::walk_x;
use logic::player::{Player, PlayerInput};
use logic::skeleton::{Projectile, ProjectileKind};
use logic::world::{really_overlaps, shortest_dx, World};
use macroquad::math::{Rect, Vec2};

const FRAME: f32 = 1.0 / 60.0;
const MANY_SEEDS: u32 = 500;

fn maps() -> impl Iterator<Item = (u32, World)> {
    (0..MANY_SEEDS).map(|seed| (seed, World::long_map(seed.wrapping_mul(7919))))
}

fn stand_at(x: f32, w: &World) -> Player {
    let mut p = Player::new(CharacterKind::Niro, Vec2::new(x, w.ground_y - PLAYER_HEIGHT));
    p.update(&PlayerInput::default(), FRAME, w);
    p
}

/// `going_right` 쪽으로 한 바퀴 걷고 막히면 점프한다. 다 돌면 걸린 초를 돌려준다.
fn run_one_lap(p: &mut Player, going_right: bool, w: &World, limit_seconds: f32) -> Option<f32> {
    let walk = PlayerInput { left: !going_right, right: going_right, ..Default::default() };
    let mut walked = 0.0;
    for frame in 0..(limit_seconds / FRAME) as usize {
        if walked >= w.width {
            return Some(frame as f32 * FRAME);
        }
        let before = p.pos.x;
        p.update(&walk, FRAME, w);
        if p.pos.x == before {
            p.update(&PlayerInput { jump_pressed: true, ..walk }, FRAME, w);
        }
        assert!((0.0..w.width).contains(&p.pos.x), "x는 맵 안으로 감긴다: {:?}", p.pos);
        let moved = shortest_dx(before, p.pos.x);
        walked += if going_right { moved } else { -moved };
    }
    None
}

/// 몸 가운데가 `x`에 올 때까지 걷고 막히면 점프한다. 도착하면 true.
fn walk_until(p: &mut Player, x: f32, w: &World) -> bool {
    for _ in 0..(40.0 / FRAME) as usize {
        let center = p.rect().center().x;
        if (center - x).abs() < 4.0 {
            return true;
        }
        let walk = PlayerInput { left: x < center, right: x > center, ..Default::default() };
        let before = p.pos.x;
        p.update(&walk, FRAME, w);
        if p.pos.x == before {
            p.update(&PlayerInput { jump_pressed: true, ..walk }, FRAME, w);
        }
    }
    false
}

/// 지금 선 곳에서 `top` 위로 뛰어 오른다. 아이처럼: 가장자리까지 걸어가 뛰고, 위에 오면 손을 뗀다.
fn hop_onto(p: &mut Player, top: Rect, w: &World) {
    let to_right = top.center().x > p.rect().center().x;
    let walk = PlayerInput { left: !to_right, right: to_right, ..Default::default() };
    p.update(&PlayerInput { jump_pressed: true, ..walk }, FRAME, w);
    for _ in 0..(1.5 / FRAME) as usize {
        let center = p.rect().center().x;
        let over_top = top.x + 8.0 < center && center < top.x + top.w - 8.0;
        let input = if over_top { PlayerInput::default() } else { walk };
        p.update(&input, FRAME, w);
    }
}

fn settle(p: &mut Player, w: &World) {
    for _ in 0..(1.5 / FRAME) as usize {
        p.update(&PlayerInput::default(), FRAME, w);
    }
}

fn is_standing_on(p: &Player, top: Rect) -> bool {
    p.on_ground && p.pos.y + p.size.y == top.y
}

/// 시작 자리에서 나무 꼭대기까지 올라간다.
fn climb_tree(p: &mut Player, w: &World) {
    let branches = w.tree_branches();
    assert!(walk_until(p, branches[0].x - 60.0, w), "나무까지 못 감, x {}", p.pos.x);
    settle(p, w);
    for (i, branch) in branches.iter().enumerate() {
        if i > 0 {
            // 앞 칸 오른쪽 끝에서 뛴다.
            let edge = branches[i - 1].x + branches[i - 1].w - 6.0;
            assert!(walk_until(p, edge, w));
        }
        hop_onto(p, *branch, w);
        settle(p, w);
        assert!(is_standing_on(p, *branch), "{i}번째 발판에 못 섬: {:?}", p.pos);
    }
}

/// 시작 자리에서 산꼭대기까지 올라간다.
fn climb_rock(p: &mut Player, w: &World) {
    let steps = w.rock_steps();
    assert!(walk_until(p, steps[0].x - 60.0, w), "바위까지 못 감, x {}", p.pos.x);
    settle(p, w);
    for step in &steps[..3] {
        hop_onto(p, *step, w);
        settle(p, w);
        assert!(is_standing_on(p, *step), "계단에 못 섬: {:?}", p.pos);
    }
}

// ---- 장소와 카메라 ----

#[test]
fn map_is_four_screens_with_places_in_order() {
    let w = World::long_map(1);
    assert_eq!(w.width, 3840.0);
    let names: Vec<&str> = [0.0, 959.0, 960.0, 1919.0, 1920.0, 2879.0, 2880.0, 3839.0, 3840.0, 3850.0, -10.0]
        .iter()
        .map(|x| w.place_name_at(*x))
        .collect();
    assert_eq!(names, ["해변", "해변", "화산", "화산", "산", "산", "집", "집", "해변", "해변", "집"], "이음매 너머는 다음 장소다");
}

#[test]
fn new_game_starts_on_the_beach_on_the_long_map() {
    let g = Game::new(CharacterKind::Niro, 1);
    assert_eq!(g.world.width, MAP_WIDTH);
    assert_eq!(g.player.pos.x, PLAYER_SPAWN_X);
    assert_eq!(g.place_name(), "해변");
    assert_eq!(g.camera_left(), PLAYER_SPAWN_X + PLAYER_WIDTH / 2.0 - SCREEN_WIDTH / 2.0, "시작할 때도 주인공이 가운데");
}

#[test]
fn camera_follows_the_player_and_never_stops_on_the_long_map() {
    let w = World::long_map(1);
    assert_eq!(w.camera_left(100.0), -380.0, "해변 왼쪽 너머도 보여서 멈추지 않는다");
    assert_eq!(w.camera_left(480.0), 0.0);
    assert_eq!(w.camera_left(1500.0), 1020.0, "주인공이 가운데");
    assert_eq!(w.camera_left(3360.0), 2880.0);
    assert_eq!(w.camera_left(3800.0), 3320.0, "집 오른쪽 너머도 보여서 멈추지 않는다");
}

#[test]
fn small_map_camera_never_moves() {
    let w = World::m1();
    assert_eq!(w.camera_left(0.0), 0.0);
    assert_eq!(w.camera_left(900.0), 0.0);
    assert!(w.hole().is_none() && w.tree_branches().is_empty() && w.rock_steps().is_empty());
    assert_eq!(w.boxes(), w.obstacles, "M1 맵의 장애물은 모두 상자다");
}

#[test]
fn player_position_and_place_change_as_they_walk_without_touching_clock_or_coins() {
    let mut g = Game::new(CharacterKind::Niro, 1);
    let boxes = g.world.boxes();
    g.world.obstacles.retain(|b| !boxes.contains(b));
    g.citizens.list.clear();
    g.player.pos.x = 950.0;
    let walk = PlayerInput { right: true, ..Default::default() };
    for _ in 0..10 {
        g.update(&walk, FRAME);
    }
    assert_eq!(g.place_name(), "화산");
    assert_eq!(g.clock.day_number(), 1);
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP);
}

// ---- 상자 ----

#[test]
fn every_place_has_boxes_and_never_more_than_ten() {
    for (seed, w) in maps() {
        for place in 0..PLACE_NAMES.len() {
            let count = w.boxes().iter().filter(|b| w.place_index_at(b.x) == place).count();
            assert!((1..=BOX_COUNT_MAX).contains(&count), "seed {seed}, {} 상자 {count}개", PLACE_NAMES[place]);
        }
    }
}

#[test]
fn citizens_have_running_room_on_both_sides() {
    for (seed, w) in maps() {
        for b in w.boxes() {
            for x in CITIZEN_XS {
                let clear = b.x + b.w <= x - CITIZEN_RUN_UP_ROOM || x + CITIZEN_RUN_UP_ROOM <= b.x;
                assert!(clear, "seed {seed}: {b:?}, 시민 {x}");
            }
        }
    }
}

#[test]
fn boxes_stay_in_the_box_area_of_each_place() {
    for (seed, w) in maps() {
        for b in w.boxes() {
            let left = w.place_index_at(b.x) as f32 * SCREEN_WIDTH;
            assert!(left + BOX_AREA_LEFT <= b.x && b.x + b.w <= left + BOX_AREA_RIGHT, "seed {seed}: {b:?}");
            assert_eq!(b.y + b.h, w.ground_y);
        }
    }
}

#[test]
fn same_seed_same_map_and_other_seeds_differ() {
    let list = |seed| World::long_map(seed).boxes().iter().map(|b| (b.x, b.w)).collect::<Vec<_>>();
    assert_eq!(list(42), list(42));
    assert_ne!(list(1), list(2));
}

#[test]
fn gap_between_boxes_is_zero_or_wide_enough() {
    for (seed, w) in maps() {
        let mut boxes = w.boxes();
        boxes.sort_by(|a, b| a.x.total_cmp(&b.x));
        for pair in boxes.windows(2) {
            let gap = pair[1].x - (pair[0].x + pair[0].w);
            assert!(gap == 0.0 || gap >= BOX_MIN_GAP, "seed {seed}: 틈 {gap}, {pair:?}");
        }
    }
}

#[test]
fn no_box_near_hide_spots_citizens_or_structures() {
    let w0 = World::long_map(0);
    let points = HIDE_SPOTS.iter().map(|spot| spot.1).chain(CITIZEN_XS).map(|x| (x, x));
    let solids: Vec<Rect> = w0.tree_branches().into_iter().chain(w0.rock_steps()).chain(w0.hole()).collect();
    let zones: Vec<(f32, f32)> = points.chain(solids.iter().map(|r| (r.x, r.x + r.w))).collect();
    for (seed, w) in maps() {
        for b in w.boxes() {
            for (left, right) in &zones {
                let clear = b.x + b.w <= left - NO_BOX_MARGIN || right + NO_BOX_MARGIN <= b.x;
                assert!(clear, "seed {seed}: {b:?}가 {left}~{right} 곁에 있다");
            }
        }
    }
}

#[test]
fn coins_sit_only_on_boxes_not_on_branches_or_steps() {
    for seed in 0..100 {
        let g = Game::new(CharacterKind::Niro, seed);
        let boxes = g.world.boxes();
        assert_eq!(g.coins.list.len(), boxes.len());
        assert_eq!(g.world.obstacles.len(), boxes.len() + TREE_BRANCHES.len() + ROCK_STEPS.len());
    }
}

// ---- 왕복과 낙하 ----

#[test]
fn player_laps_every_map_both_ways_within_30_seconds() {
    let mut slowest: f32 = 0.0;
    for (seed, w) in maps() {
        let mut p = stand_at(PLAYER_SPAWN_X, &w);
        let there = run_one_lap(&mut p, true, &w, 30.0);
        assert!(there.is_some(), "seed {seed}: 오른쪽으로 한 바퀴 못 돎, {:?}", p.pos);
        let back = run_one_lap(&mut p, false, &w, 30.0);
        assert!(back.is_some(), "seed {seed}: 왼쪽으로 한 바퀴 못 돎, {:?}", p.pos);
        slowest = slowest.max(there.unwrap_or(0.0)).max(back.unwrap_or(0.0));
    }
    println!("가장 오래 걸린 한 바퀴: {slowest:.1}초");
}

#[test]
fn player_crosses_the_seam_even_jumping_and_stays_on_the_map() {
    let w = World::long_map(1);
    for (x, input) in [
        (10.0, PlayerInput { left: true, jump_pressed: true, ..Default::default() }),
        (w.width - 60.0, PlayerInput { right: true, jump_pressed: true, ..Default::default() }),
    ] {
        let mut p = stand_at(x, &w);
        let mut crossed = false;
        for _ in 0..120 {
            let before = p.pos.x;
            p.update(&input, FRAME, &w);
            assert!((0.0..w.width).contains(&p.pos.x), "{:?}", p.pos);
            crossed |= (p.pos.x - before).abs() > w.width / 2.0;
        }
        assert!(crossed, "{x}에서 이음매를 넘는다");
    }
}

#[test]
fn player_never_hangs_in_the_air_wherever_they_land() {
    for (seed, w) in maps().take(6) {
        let mut x = 0.0;
        while x < w.width {
            let mut p = Player::new(CharacterKind::Ellie, Vec2::new(x, 100.0));
            for _ in 0..(2.0 / FRAME) as usize {
                p.update(&PlayerInput::default(), FRAME, &w);
            }
            assert!(p.on_ground, "seed {seed}: x {x}에서 떨어뜨리니 {:?}에 떠 있다", p.pos);
            assert!(!w.obstacles.iter().any(|b| really_overlaps(*b, p.rect())), "seed {seed}: x {x} 상자에 묻힘");
            x += 7.3;
        }
    }
}

// ---- 구멍 ----

#[test]
fn hole_matches_the_story_and_jump_can_clear_its_depth() {
    let hole = World::long_map(1).hole().expect("긴 맵에는 구멍이 있다");
    assert_eq!((hole.x, hole.w, hole.h), (1620.0, 160.0, 64.0));
    assert_eq!(hole.center().x, 1700.0);
    let jump_height = PLAYER_JUMP_SPEED * PLAYER_JUMP_SPEED / (2.0 * GRAVITY);
    assert!(HOLE_DEPTH + 8.0 < jump_height);
}

#[test]
fn player_walks_down_into_the_hole_and_jumps_back_out_on_both_sides() {
    let w = World::long_map(1);
    let hole = w.hole().unwrap();
    for (start_x, going_right) in [(hole.x - 100.0, true), (hole.x + hole.w + 52.0, false)] {
        let mut p = stand_at(start_x, &w);
        let walk = PlayerInput { left: !going_right, right: going_right, ..Default::default() };
        for _ in 0..90 {
            p.update(&walk, FRAME, &w);
        }
        assert!(p.on_ground);
        assert_eq!(p.pos.y + p.size.y, hole.y + hole.h, "구멍 바닥에 선다");
        assert!(hole.x <= p.pos.x && p.pos.x + p.size.x <= hole.x + hole.w, "옆벽을 뚫지 않는다");

        p.update(&PlayerInput { jump_pressed: true, ..walk }, FRAME, &w);
        for _ in 0..90 {
            p.update(&walk, FRAME, &w);
        }
        assert_eq!(p.pos.y + p.size.y, w.ground_y, "구멍을 나와 땅에 선다");
        assert!(if going_right { p.pos.x >= hole.x + hole.w - p.size.x } else { p.pos.x <= hole.x });
    }
}

#[test]
fn running_jump_clears_the_whole_hole() {
    let w = World::long_map(1);
    let hole = w.hole().unwrap();
    let mut p = stand_at(hole.x - 40.0, &w);
    let walk = PlayerInput { right: true, ..Default::default() };
    p.update(&PlayerInput { jump_pressed: true, ..walk }, FRAME, &w);
    for _ in 0..50 {
        p.update(&walk, FRAME, &w);
        assert!(p.pos.y + p.size.y <= w.ground_y, "구멍에 빠지지 않는다");
    }
    assert!(p.pos.x + p.size.x > hole.x + hole.w);
}

#[test]
fn standing_half_over_the_hole_edge_stays_on_the_ground() {
    let w = World::long_map(1);
    let hole = w.hole().unwrap();
    let mut p = stand_at(hole.x - 20.0, &w);
    settle(&mut p, &w);
    assert!(p.on_ground);
    assert_eq!(p.pos.y + p.size.y, w.ground_y);
}

#[test]
fn monsters_stop_at_the_hole_edge_from_both_sides() {
    let w = World::long_map(1);
    let hole = w.hole().unwrap();
    let body = |x| Rect::new(x, w.ground_y - ZOMBIE_HEIGHT, ZOMBIE_WIDTH, ZOMBIE_HEIGHT);
    let mut x = hole.x - ZOMBIE_WIDTH - 10.0;
    for _ in 0..30 {
        x = walk_x(body(x), hole.center().x, 2.0, &w);
    }
    assert_eq!(x + ZOMBIE_WIDTH, hole.x);
    let mut x = hole.x + hole.w + 10.0;
    for _ in 0..30 {
        x = walk_x(body(x), hole.center().x, 2.0, &w);
    }
    assert_eq!(x, hole.x + hole.w);
}

#[test]
fn arrows_fly_over_the_hole() {
    let w = World::long_map(1);
    let hole = w.hole().unwrap();
    let shot = Projectile::new(hole.center().x, 1.0, ProjectileKind::Arrow, 2, &w);
    assert!(!really_overlaps(shot.rect(), hole), "화살 높이는 땅 위다");
    assert!(!w.obstacles.iter().any(|b| really_overlaps(*b, shot.rect())));
}

// ---- 나무 발판과 바위 계단 ----

#[test]
fn branches_and_steps_rise_one_step_at_a_time_to_192() {
    let w = World::long_map(1);
    let heights = |list: Vec<Rect>| list.iter().map(|b| w.ground_y - b.y).collect::<Vec<f32>>();
    assert_eq!(heights(w.tree_branches()), [64.0, 128.0, 192.0]);
    assert_eq!(heights(w.rock_steps()), [64.0, 128.0, 192.0, 128.0, 64.0]);
    let jump_height = PLAYER_JUMP_SPEED * PLAYER_JUMP_SPEED / (2.0 * GRAVITY);
    assert!(CLIMB_STEP + 8.0 < jump_height);
}

#[test]
fn player_climbs_to_the_tree_top_on_every_map() {
    for (seed, w) in maps() {
        let mut p = stand_at(PLAYER_SPAWN_X, &w);
        climb_tree(&mut p, &w);
        let top = w.tree_branches()[2];
        assert!(is_standing_on(&p, top), "seed {seed}");
    }
}

#[test]
fn player_climbs_to_the_mountain_top_on_every_map() {
    for (seed, w) in maps() {
        let mut p = stand_at(PLAYER_SPAWN_X, &w);
        climb_rock(&mut p, &w);
        assert!(is_standing_on(&p, w.rock_steps()[2]), "seed {seed}");
    }
}

#[test]
fn player_reaches_every_hide_spot_on_every_map() {
    let touches_spot = |p: &Player, spot: (&str, f32, f32), w: &World| {
        let feet_y = w.ground_y - spot.2;
        let body = Rect::new(spot.1 - MISS_WIDTH / 2.0, feet_y - MISS_HEIGHT, MISS_WIDTH, MISS_HEIGHT);
        really_overlaps(p.rect(), body)
    };
    for (seed, w) in maps() {
        for spot in HIDE_SPOTS {
            let mut p = stand_at(PLAYER_SPAWN_X, &w);
            match spot.0 {
                "나무 꼭대기" => climb_tree(&mut p, &w),
                "산꼭대기" => climb_rock(&mut p, &w),
                _ => {
                    assert!(walk_until(&mut p, spot.1, &w), "seed {seed}: {}까지 못 감", spot.0);
                    settle(&mut p, &w);
                }
            }
            assert!(touches_spot(&p, spot, &w), "seed {seed}: {}에 안 닿음, {:?}", spot.0, p.pos);
        }
    }
}

#[test]
fn jumping_at_the_top_keeps_the_head_on_screen() {
    let w = World::long_map(1);
    let top = w.rock_steps()[2];
    let mut p = Player::new(CharacterKind::Niro, Vec2::new(top.x + 8.0, top.y - PLAYER_HEIGHT));
    settle(&mut p, &w);
    p.update(&PlayerInput { jump_pressed: true, ..Default::default() }, FRAME, &w);
    for _ in 0..60 {
        p.update(&PlayerInput::default(), FRAME, &w);
        assert!(p.pos.y > 0.0, "머리가 화면 위로 잘리면 안 된다: {}", p.pos.y);
    }
}

#[test]
fn player_can_get_down_from_the_tops_by_walking_off() {
    let w = World::long_map(1);
    for top in [w.tree_branches()[2], w.rock_steps()[2]] {
        let mut p = Player::new(CharacterKind::Niro, Vec2::new(top.x + 8.0, top.y - PLAYER_HEIGHT));
        settle(&mut p, &w);
        let walk = PlayerInput { right: true, ..Default::default() };
        for _ in 0..120 {
            p.update(&walk, FRAME, &w);
        }
        assert_eq!(p.pos.y + p.size.y, w.ground_y, "{top:?}에서 땅까지 내려온다");
    }
}
