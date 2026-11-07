//! D36 랜덤 상자, 몬스터 양쪽, 둔해진 스켈레톤 테스트 (수산나).
//! 기준: docs/02-plan.md D36. 상자 틈 규칙은 BUG-006 미끄러짐이 좁은 틈에서 못 빠져나가는 문제 때문이다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::combat::{Combat, CombatEvent};
use logic::game::Game;
use logic::player::{Player, PlayerInput};
use logic::world::{really_overlaps, World};
use macroquad::math::{Rect, Vec2};

const FRAME: f32 = 1.0 / 60.0;
const MANY_SEEDS: u32 = 500;

fn maps() -> impl Iterator<Item = (u32, World)> {
    (0..MANY_SEEDS).map(|seed| (seed, World::with_random_boxes(seed.wrapping_mul(7919))))
}

fn box_list(w: &World) -> Vec<(f32, f32)> {
    w.obstacles.iter().map(|b| (b.x, b.w)).collect()
}

/// 왼쪽부터 이웃한 두 상자 사이 틈.
fn gaps(w: &World) -> Vec<f32> {
    w.obstacles.windows(2).map(|pair| pair[1].x - (pair[0].x + pair[0].w)).collect()
}

fn player_on_ground(x: f32, w: &World) -> Player {
    Player::new(CharacterKind::Niro, Vec2::new(x, w.ground_y - PLAYER_HEIGHT))
}

// ---- 상자 배치 ----

#[test]
fn box_count_is_3_to_10_and_every_count_appears() {
    let mut seen = [false; BOX_COUNT_MAX + 1];
    for (seed, w) in maps() {
        let count = w.obstacles.len();
        assert!((BOX_COUNT_MIN..=BOX_COUNT_MAX).contains(&count), "seed {seed}: {count}개");
        seen[count] = true;
    }
    assert_eq!((BOX_COUNT_MIN, BOX_COUNT_MAX), (3, 10), "D36");
    assert!(seen[BOX_COUNT_MIN..].iter().all(|s| *s), "{seen:?}");
}

#[test]
fn same_seed_same_boxes_and_other_seeds_differ() {
    assert_eq!(box_list(&World::with_random_boxes(42)), box_list(&World::with_random_boxes(42)));
    let mut layouts: Vec<String> = maps().map(|(_, w)| format!("{:?}", box_list(&w))).collect();
    layouts.sort();
    layouts.dedup();
    assert!(layouts.len() > MANY_SEEDS as usize * 9 / 10, "서로 다른 배치 {}개", layouts.len());
}

#[test]
fn neighbouring_seeds_also_differ() {
    // 판마다 시드는 시각이라 값이 가깝다.
    let different = (1000..1100)
        .filter(|&s| box_list(&World::with_random_boxes(s)) != box_list(&World::with_random_boxes(s + 1)))
        .count();
    assert!(different >= 95, "{different}");
}

#[test]
fn boxes_are_two_sizes_standing_on_the_ground_and_jumpable() {
    let highest_jump = PLAYER_JUMP_SPEED * PLAYER_JUMP_SPEED / (2.0 * GRAVITY);
    let (mut small, mut big) = (0, 0);
    for (seed, w) in maps() {
        for b in &w.obstacles {
            assert!(b.w == BOX_SMALL_SIZE || b.w == BOX_BIG_SIZE, "seed {seed}: {b:?}");
            assert_eq!(b.w, b.h);
            assert_eq!(b.y + b.h, w.ground_y);
            assert!(b.h < highest_jump, "seed {seed}: {b:?}");
            if b.w == BOX_BIG_SIZE {
                big += 1;
            } else {
                small += 1;
            }
        }
    }
    assert!(small > 500 && big > 500, "작은 상자 {small}개, 큰 상자 {big}개");
}

#[test]
fn start_spot_and_both_edges_stay_empty() {
    let start = Rect::new(PLAYER_SPAWN_X - PLAYER_WIDTH, 0.0, PLAYER_WIDTH * 3.0, GROUND_Y);
    // 몬스터가 나오는 자리보다 넓고, 끝과 상자 사이에도 플레이어가 끼지 않는다.
    assert!(BOX_MIN_GAP > ZOMBIE_WIDTH.max(SKELETON_WIDTH));
    let left_edge = Rect::new(0.0, 0.0, BOX_MIN_GAP, GROUND_Y);
    let right_edge = Rect::new(WORLD_WIDTH - BOX_MIN_GAP, 0.0, BOX_MIN_GAP, GROUND_Y);
    for (seed, w) in maps() {
        for b in &w.obstacles {
            for empty in [start, left_edge, right_edge] {
                assert!(!really_overlaps(*b, empty), "seed {seed}: {b:?}");
            }
        }
    }
}

#[test]
fn gap_between_boxes_is_zero_or_wide_enough_and_nothing_overlaps() {
    const { assert!(BOX_MIN_GAP > PLAYER_WIDTH) };
    let (mut touching, mut open) = (0, 0);
    for (seed, w) in maps() {
        for gap in gaps(&w) {
            assert!(gap == 0.0 || gap >= BOX_MIN_GAP, "seed {seed}: 틈 {gap}");
            if gap == 0.0 {
                touching += 1;
            } else {
                open += 1;
            }
        }
    }
    assert!(touching > 100 && open > 100, "붙음 {touching}, 벌어짐 {open}");
}

#[test]
fn ten_big_boxes_fit_in_the_box_area() {
    assert!(BOX_COUNT_MAX as f32 * BOX_BIG_SIZE <= BOX_AREA_RIGHT - BOX_AREA_LEFT);
}

#[test]
fn no_body_wide_pit_between_taller_boxes() {
    for (seed, w) in maps() {
        let b = &w.obstacles;
        for i in 2..b.len() {
            let touching = b[i - 2].x + b[i - 2].w == b[i - 1].x && b[i - 1].x + b[i - 1].w == b[i].x;
            let pit = b[i - 1].h < b[i - 2].h && b[i - 1].h < b[i].h;
            assert!(!(touching && pit), "seed {seed}: {b:?}");
        }
    }
}

#[test]
fn new_game_has_one_coin_per_box_and_different_games_have_different_boxes() {
    for seed in 0..100 {
        let g = Game::new(CharacterKind::Niro, seed);
        assert_eq!(g.coins.list.len(), g.world.boxes().len(), "seed {seed}");
        assert!(g.coins.list.iter().zip(&g.world.boxes()).all(|(c, b)| {
            c.rect.center().x == b.center().x && c.rect.y + c.rect.h < b.y
        }));
    }
    let a = Game::new(CharacterKind::Niro, 1);
    let b = Game::new(CharacterKind::Niro, 2);
    assert_ne!(box_list(&a.world), box_list(&b.world));
}

// ---- 진짜 움직임으로 다녀 보기 ----

/// `going_right` 쪽 끝까지 걸어간다. 막히면 점프한다. 끝에 닿으면 true.
fn walk_and_jump_to_the_end(p: &mut Player, going_right: bool, w: &World) -> bool {
    let end_x = if going_right { w.width - p.size.x } else { 0.0 };
    for _ in 0..(30.0 / FRAME) as usize {
        if p.pos.x == end_x {
            return true;
        }
        let before = p.pos.x;
        p.update(&PlayerInput { left: !going_right, right: going_right, ..Default::default() }, FRAME, w);
        if p.pos.x == before {
            p.update(&PlayerInput { left: !going_right, right: going_right, jump_pressed: true, ..Default::default() }, FRAME, w);
        }
    }
    false
}

#[test]
fn player_can_cross_every_map_both_ways_with_walking_and_jumping() {
    for (seed, w) in maps() {
        let mut p = player_on_ground(0.0, &w);
        assert!(walk_and_jump_to_the_end(&mut p, true, &w), "seed {seed}: 오른쪽으로 못 감, x {}", p.pos.x);
        assert!(walk_and_jump_to_the_end(&mut p, false, &w), "seed {seed}: 왼쪽으로 못 감, x {}", p.pos.x);
    }
}

#[test]
fn player_never_hangs_in_the_air_wherever_they_land() {
    // 좁은 틈이나 구덩이가 있으면 양쪽 상자 사이에서 미끄러지기만 하고 땅에 못 닿는다.
    for (seed, w) in maps().take(100) {
        let mut x = 0.0;
        while x <= w.width - PLAYER_WIDTH {
            let mut p = Player::new(CharacterKind::Ellie, Vec2::new(x, GROUND_Y - 200.0));
            for _ in 0..(1.5 / FRAME) as usize {
                p.update(&PlayerInput::default(), FRAME, &w);
            }
            assert!(p.on_ground, "seed {seed}: x {x}에서 떨어뜨리니 {:?}에 떠 있다", p.pos);
            x += 4.7;
        }
    }
}

#[test]
fn walking_over_touching_boxes_of_same_height_stays_on_the_ground() {
    let top = GROUND_Y - BOX_BIG_SIZE;
    let w = World {
        width: WORLD_WIDTH,
        ground_y: GROUND_Y,
        obstacles: vec![
            Rect::new(300.0, top, BOX_BIG_SIZE, BOX_BIG_SIZE),
            Rect::new(364.0, top, BOX_BIG_SIZE, BOX_BIG_SIZE),
            Rect::new(428.0, top, BOX_BIG_SIZE, BOX_BIG_SIZE),
        ],
    };
    for (start_x, input) in [
        (310.0, PlayerInput { right: true, ..Default::default() }),
        (440.0, PlayerInput { left: true, ..Default::default() }),
    ] {
        let mut p = Player::new(CharacterKind::Niro, Vec2::new(start_x, top - PLAYER_HEIGHT));
        p.update(&PlayerInput::default(), FRAME, &w);
        for _ in 0..30 {
            p.update(&input, FRAME, &w);
            assert!(p.on_ground, "x {}에서 떴다", p.pos.x);
        }
    }
}

// ---- 양쪽 끝에서 나오기 ----

#[test]
fn monsters_come_from_both_edges_wherever_the_player_is() {
    for player_x in [PLAYER_SPAWN_X, 480.0, WORLD_WIDTH - 200.0] {
        let player = Rect::new(player_x, GROUND_Y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT);
        let (mut lefts, mut rights) = (0, 0);
        for (seed, w) in maps().take(200) {
            let mut c = Combat::new(seed);
            let events = c.update(FRAME, true, player, &w);
            let at = events.iter().find_map(|e| match e {
                CombatEvent::ZombieAppeared { at } | CombatEvent::SkeletonAppeared { at, .. } => Some(*at),
                _ => None,
            });
            if at.expect("밤이 되면 첫 몬스터가 나온다").x < WORLD_WIDTH / 2.0 {
                lefts += 1;
            } else {
                rights += 1;
            }
        }
        assert!(lefts >= 60 && rights >= 60, "플레이어 x {player_x}: 왼쪽 {lefts}, 오른쪽 {rights}");
    }
}

// ---- 둔해진 스켈레톤 ----

/// 맵 하나에서 밤 1분: 스켈레톤이 튕긴 공격에 맞은 수, 피한 수, 얻은 경험치.
/// `level`로 무기를 고른다. 플레이어는 공격하지 않고 맞지 않게 하늘에 둔다.
fn bounce_results(seed: u32, w: &World, level: u32, player_x: f32) -> (u32, u32, u32) {
    let player = Rect::new(player_x, -1000.0, PLAYER_WIDTH, PLAYER_HEIGHT);
    let mut c = Combat::new(seed);
    c.skeleton_chance = 1.0;
    c.level = level;
    let (mut hits, mut dodges, mut defeats) = (0, 0, 0);
    for _ in 0..(60.0 / FRAME) as usize {
        for e in c.update(FRAME, true, player, w) {
            match e {
                CombatEvent::SkeletonHit { .. } => hits += 1,
                CombatEvent::SkeletonDefeated { .. } => {
                    hits += 1;
                    defeats += 1;
                }
                CombatEvent::SkeletonDodged { .. } => dodges += 1,
                _ => {}
            }
        }
    }
    (hits, dodges, defeats * EXPERIENCE_PER_DEFEAT)
}

#[test]
fn skeletons_really_get_hit_by_bounces_on_random_maps() {
    for (name, level, least_maps) in [("화살", 1, 50), ("총알", SKELETON_FULL_MIN_LEVEL, 80)] {
        for player_x in [PLAYER_SPAWN_X, 480.0, 800.0] {
            let (mut maps_with_hits, mut hits, mut dodges, mut experience) = (0, 0, 0, 0);
            for (seed, w) in maps().take(100) {
                let (h, d, e) = bounce_results(seed, &w, level, player_x);
                maps_with_hits += u32::from(h > 0);
                hits += h;
                dodges += d;
                experience += e;
            }
            println!("{name} 플레이어 x {player_x}: 맵 100개 중 {maps_with_hits}개에서 맞음, 맞음 {hits}, 피함 {dodges}, 경험치 {experience}");
            assert!(maps_with_hits >= least_maps, "{name} 플레이어 x {player_x}: {maps_with_hits}개");
            assert!(dodges > 0, "{name}: 피하는 일도 남아 있다");
            assert!(experience > 0, "{name}: 튕긴 공격으로 쓰러져도 경험치를 준다");
        }
    }
}
