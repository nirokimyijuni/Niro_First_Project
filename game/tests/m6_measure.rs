//! M6 측정 (수산나): 규칙을 바꾸지 않고 숫자만 잰다. 오래 걸려서 평소에는 건너뛴다.
//! 보려면 `cargo test --test m6_measure -- --ignored --nocapture`.
//! 1) 왼쪽 끝이 안전지대인지 (엘리 관찰)  2) 몬스터가 주인공까지 못 오는 판이 얼마나 되는지.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::combat::CombatEvent;
use logic::game::{Game, GameEvent};
use logic::monster::{pick_left_side, spawn_x_on_side, walk_x, SeededRandom};
use logic::player::PlayerInput;
use logic::skeleton::SkeletonStage;
use logic::world::{really_overlaps, shortest_dx, World};
use macroquad::math::Rect;

const FRAME: f32 = 1.0 / 60.0;
/// 측정 시험 모두가 `timeout 300` 안에 끝나는 판 수. 옛 기록은 500판이라 비율로 비교한다.
const SEEDS: u32 = 100;

/// `player_x`에 가만히 서서 밤을 `seconds`초 버틴 판 수.
fn games_without_a_scratch(world_of: impl Fn(u32) -> World, player_x: f32, seconds: f32, seeds: u32) -> u32 {
    let mut safe = 0;
    for seed in 0..seeds {
        let mut g = Game::on_world(CharacterKind::Niro, seed.wrapping_mul(7919), world_of(seed.wrapping_mul(7919)));
        g.citizens.list.clear();
        g.clock.update(DAY_SECONDS);
        g.player.pos.x = player_x;
        for _ in 0..(seconds / FRAME) as usize {
            g.update(&PlayerInput::default(), FRAME);
        }
        safe += u32::from(g.combat.health.hp() == PLAYER_START_HP);
    }
    safe
}

#[test]
#[ignore]
fn old_left_end_is_not_a_safe_zone() {
    for (name, seconds) in [("35초", 35.0)] {
        for x in [0.0, 48.0, 120.0] {
            let safe = games_without_a_scratch(World::long_map, x, seconds, SEEDS);
            println!("긴 맵 x={x}, {name}: 체력 그대로 {safe}/{SEEDS}");
        }
    }
    let left_end = games_without_a_scratch(World::long_map, 0.0, 35.0, SEEDS);
    // D61 전에는 왼쪽 끝이 훨씬 안전했다 (x=0 밤 끝까지 395/500). 벽 안쪽에서 나오면서 10% 아래로 내려왔다.
    assert!(left_end * 10 <= SEEDS, "왼쪽 끝 {left_end}/{SEEDS}");
}

/// 몬스터가 `from_x`에서 걸어서 주인공 몸에 닿으면 true. 몬스터 폭보다 짧게 걸어서 벽을 뚫지 않는다.
fn zombie_can_reach(from_x: f32, player: Rect, w: &World) -> bool {
    const LONGEST_MOVE: f32 = 24.0;
    let mut x = from_x;
    for _ in 0..(SCREEN_WIDTH * 2.0 / LONGEST_MOVE) as usize {
        let body = Rect::new(x, w.ground_y - ZOMBIE_HEIGHT, ZOMBIE_WIDTH, ZOMBIE_HEIGHT);
        if really_overlaps(body, player) {
            return true;
        }
        let next = walk_x(body, player.center().x, LONGEST_MOVE, w);
        if next == x {
            return false;
        }
        x = next;
    }
    false
}

/// (왼쪽에서 오는가, 오른쪽에서 오는가). 나오는 자리는 진짜 규칙으로 뽑는다.
fn reachable_sides(player: Rect, w: &World) -> (bool, bool) {
    let center = player.center().x;
    let spots: Vec<f32> = (0..24).filter_map(|seed| spawn_x_on_side(pick_left_side(&mut SeededRandom::new(seed)), player, ZOMBIE_WIDTH, w, &[])).collect();
    let left = spots.iter().filter(|x| shortest_dx(center, **x) < 0.0).any(|x| zombie_can_reach(*x, player, w));
    let right = spots.iter().filter(|x| shortest_dx(center, **x) > 0.0).any(|x| zombie_can_reach(*x, player, w));
    (left, right)
}

#[derive(Default, Clone, Copy)]
struct Tally {
    spots: u32,
    one_side: u32,
    no_side: u32,
}

impl Tally {
    fn add(&mut self, sides: (bool, bool)) {
        self.spots += 1;
        self.one_side += u32::from(sides.0 != sides.1);
        self.no_side += u32::from(!sides.0 && !sides.1);
    }
    fn line(&self) -> String {
        let percent = |n: u32| n as f32 * 100.0 / self.spots.max(1) as f32;
        format!("{}자리: 한쪽만 옴 {:.1}%, 양쪽 다 못 옴 {:.1}%", self.spots, percent(self.one_side), percent(self.no_side))
    }
}

#[test]
#[ignore]
fn how_often_monsters_cannot_reach_the_player_on_foot() {
    // 땅 위 자리를 맵 전체에서 고르게 잰다. 상자 위·구멍 안은 빼고 따로 본다.
    let mut all = Tally::default();
    let mut by_boxes_on_screen = [Tally::default(); 16];
    let mut by_place = [Tally::default(); PLACE_NAMES.len()];
    let mut start = Tally::default();
    let mut map_with_a_no_side_spot = 0;
    let mut nobody_appears = 0;
    for seed in 0..SEEDS {
        let w = World::long_map(seed.wrapping_mul(7919));
        let start_body = Rect::new(PLAYER_SPAWN_X, w.ground_y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT);
        start.add(reachable_sides(start_body, &w));
        let mut any_no_side = false;
        for x in (0..(MAP_WIDTH - PLAYER_WIDTH) as usize).step_by(96) {
            let body = Rect::new(x as f32, w.ground_y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT);
            let hole = w.hole().unwrap();
            let in_hole = body.x + body.w > hole.x && body.x < hole.x + hole.w;
            if in_hole || w.obstacles.iter().any(|b| really_overlaps(*b, body)) {
                continue;
            }
            let sides = reachable_sides(body, &w);
            all.add(sides);
            nobody_appears += u32::from(spawn_x_on_side(true, body, ZOMBIE_WIDTH, &w, &[]).is_none());
            any_no_side |= !sides.0 && !sides.1;
            by_place[w.place_index_at(x as f32)].add(sides);
            let left = w.camera_left(body.center().x);
            let on_screen = w.boxes().iter().filter(|b| left <= b.x && b.x < left + SCREEN_WIDTH).count();
            by_boxes_on_screen[on_screen.min(15)].add(sides);
        }
        map_with_a_no_side_spot += u32::from(any_no_side);
    }
    println!("긴 맵 땅 위 전체 {}", all.line());
    println!("양쪽 다 못 오는 자리가 하나라도 있는 판: {map_with_a_no_side_spot}/{SEEDS}");
    println!("몬스터가 아무도 안 나오는 땅: {nobody_appears}/{} ({:.1}%)", all.spots, nobody_appears as f32 * 100.0 / all.spots as f32);
    println!("시작 자리(x=120) {}", start.line());
    for (i, tally) in by_place.iter().enumerate() {
        println!("{} {}", PLACE_NAMES[i], tally.line());
    }
    for (count, tally) in by_boxes_on_screen.iter().enumerate().filter(|(_, t)| t.spots > 0) {
        println!("화면 안 상자 {count}개: {}", tally.line());
    }
    assert!(all.spots > 1000);
}

#[test]
#[ignore]
fn how_often_a_normal_skeleton_is_defeated_by_its_own_arrow() {
    // REVIEW-007 5번: 큰 상자에 붙은 스켈레톤이 튕긴 화살에 맞는 일이 긴 맵에서 얼마나 나오는지.
    let (mut appeared, mut self_defeated) = (0, 0);
    for seed in 0..SEEDS {
        let mut g = Game::new(CharacterKind::Niro, seed.wrapping_mul(7919));
        g.citizens.list.clear();
        g.combat.skeleton_chance = 1.0;
        g.clock.update(DAY_SECONDS);
        g.player.pos.x = 1100.0 + (seed % 16) as f32 * 120.0;
        for _ in 0..(40.0 / FRAME) as usize {
            for event in g.update(&PlayerInput::default(), FRAME) {
                match event {
                    GameEvent::Combat(CombatEvent::SkeletonAppeared { stage: SkeletonStage::Normal, .. }) => appeared += 1,
                    GameEvent::Combat(CombatEvent::SkeletonDefeated { .. }) => self_defeated += 1,
                    _ => {}
                }
            }
            if g.combat.health.is_down() {
                break;
            }
        }
    }
    println!("밤 40초 일반 스켈레톤 {appeared}마리 중 튕긴 자기 공격으로 쓰러짐 {self_defeated}마리");
    assert!(appeared > 0);
}

#[derive(Clone, Copy, PartialEq)]
enum Plan {
    StandStill,
    Walk,
    WalkAndAttack,
    /// 가장 가까운 몬스터 쪽을 보고 다가가 맨손으로 때린다.
    Fight,
}

#[derive(Default)]
struct NightResult {
    went_down: u32,
    down_seconds: f32,
    hp_left: u32,
    appeared: u32,
    hurt: u32,
}

/// 첫 밤 하나를 처음부터 끝까지. 빨리감기 기준 실제 시간(밤 / 10)만큼 몬스터 시간이 흐른다.
fn play_first_night(seed: u32, plan: Plan, result: &mut NightResult) {
    let mut g = Game::new(CharacterKind::Niro, seed.wrapping_mul(7919));
    g.clock.update(DAY_SECONDS);
    let mut going_right = true;
    let seconds = NIGHT_SECONDS / DEV_FAST_FORWARD_SCALE;
    let frames = (seconds / FRAME) as usize;
    for frame in 0..frames {
        let moving = plan == Plan::Walk || plan == Plan::WalkAndAttack;
        let at_end = g.player.pos.x <= 0.0 || g.player.pos.x + PLAYER_WIDTH >= g.world.width;
        if at_end {
            going_right = g.player.pos.x <= 0.0;
        }
        let mut input = PlayerInput {
            left: moving && !going_right,
            right: moving && going_right,
            jump_pressed: moving && frame % 20 == 0,
            attack_pressed: plan != Plan::StandStill && plan != Plan::Walk && frame % 15 == 0,
        };
        if plan == Plan::Fight {
            let me = g.player.rect().center().x;
            let zombies = g.combat.zombies.list.iter().map(|z| z.rect().center().x);
            let skeletons = g.combat.skeletons.list.iter().map(|s| s.rect().center().x);
            if let Some(target) = zombies.chain(skeletons).min_by(|a, b| (a - me).abs().total_cmp(&(b - me).abs())) {
                input.left = target < me;
                input.right = target > me;
            }
        }
        for event in g.update(&input, FRAME) {
            match event {
                GameEvent::Combat(CombatEvent::ZombieAppeared { .. } | CombatEvent::SkeletonAppeared { .. }) => result.appeared += 1,
                GameEvent::Combat(CombatEvent::PlayerHurt { .. }) => result.hurt += 1,
                _ => {}
            }
        }
        if g.combat.health.is_down() {
            result.went_down += 1;
            result.down_seconds += frame as f32 * FRAME;
            return;
        }
    }
    result.hp_left += g.combat.health.hp();
}

#[test]
#[ignore]
fn how_hard_the_first_night_is() {
    let seeds = SEEDS;
    for (name, plan) in [("가만히", Plan::StandStill), ("걷기·점프만", Plan::Walk), ("걷기·점프·공격", Plan::WalkAndAttack), ("맞서 싸우기", Plan::Fight)] {
        let mut r = NightResult::default();
        for seed in 0..seeds {
            play_first_night(seed, plan, &mut r);
        }
        let survived = seeds - r.went_down;
        println!(
            "첫 밤 {name}: 쓰러진 판 {}/{seeds} ({:.1}%), 쓰러진 시각 평균 {:.1}초, 버틴 판 남은 체력 평균 {:.1}, 나온 몬스터 평균 {:.1}, 맞은 횟수 평균 {:.1}",
            r.went_down,
            r.went_down as f32 * 100.0 / seeds as f32,
            r.down_seconds / r.went_down.max(1) as f32,
            r.hp_left as f32 / survived.max(1) as f32,
            r.appeared as f32 / seeds as f32,
            r.hurt as f32 / seeds as f32,
        );
    }
}
