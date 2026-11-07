//! 좀비 카스테라 (D29): 밤에 화면 끝에서 나타나 플레이어 쪽으로 걸어온다.
//!
//! - 3마리 중 1마리쯤은 나타나고 조금 뒤 둥실 떠오르고, 팔을 내리는 순간 펑 터진다 (US-042).
//! - 맞고 쓰러지는 일과 플레이어 체력은 `combat.rs`가 맡는다.

use super::combat::CombatEvent;
use super::world::{nearest_copy, really_overlaps, shortest_dx, World};
use crate::config::{
    MONSTER_SPAWN_MIN_DISTANCE, SCREEN_WIDTH, SKELETON_HEIGHT, ZOMBIE_FLOAT_RISE_SPEED, ZOMBIE_FLOAT_SECONDS, ZOMBIE_FLOAT_START_SECONDS,
    ZOMBIE_HEIGHT, ZOMBIE_HIT_FLASH_SECONDS, ZOMBIE_HP, ZOMBIE_SPAWN_LEFT_CHANCE, ZOMBIE_WALK_SPEED, ZOMBIE_WIDTH,
};
use macroquad::math::{Rect, Vec2};

/// 좀비가 지금 무엇을 하는지. 화면은 이것을 보고 그림을 고른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZombieState {
    /// 땅에서 플레이어 쪽으로 걷는 중
    Walking,
    /// 팔을 위로 들고 떠오르는 중. 이때는 플레이어를 공격하지 않는다.
    Floating,
}

pub struct Zombie {
    /// 좀비 네모의 왼쪽 위 모서리.
    pub pos: Vec2,
    pub hp: u32,
    pub state: ZombieState,
    /// 날아오르는 좀비인지. 나타날 때 정해진다.
    will_float: bool,
    /// 지금 상태가 시작된 뒤 흐른 초.
    seconds_in_state: f32,
    hit_flash_seconds_left: f32,
}

impl Zombie {
    /// 땅 위 `x`에 서 있는 좀비.
    pub fn new(x: f32, ground_y: f32, will_float: bool) -> Zombie {
        Zombie {
            pos: Vec2::new(x, ground_y - ZOMBIE_HEIGHT),
            hp: ZOMBIE_HP,
            state: ZombieState::Walking,
            will_float,
            seconds_in_state: 0.0,
            hit_flash_seconds_left: 0.0,
        }
    }

    /// 좀비가 차지하는 네모 (왼쪽 위 기준).
    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, ZOMBIE_WIDTH, ZOMBIE_HEIGHT)
    }

    /// 막 맞아서 반짝여야 하면 true.
    pub fn is_flashing(&self) -> bool {
        self.hit_flash_seconds_left > 0.0
    }

    /// 공격을 맞아 체력이 `attack`만큼 준다.
    pub fn take_hit(&mut self, attack: u32) {
        self.hp = self.hp.saturating_sub(attack);
        self.hit_flash_seconds_left = ZOMBIE_HIT_FLASH_SECONDS;
    }

    /// 짧은 시간 `step` 동안 움직이기.
    fn step(&mut self, step: f32, target_x: f32, world: &World) {
        self.seconds_in_state += step;
        self.hit_flash_seconds_left = (self.hit_flash_seconds_left - step).max(0.0);

        match self.state {
            ZombieState::Walking if self.will_float && self.seconds_in_state >= ZOMBIE_FLOAT_START_SECONDS => {
                self.state = ZombieState::Floating;
                self.seconds_in_state = 0.0;
            }
            ZombieState::Walking => {
                self.pos.x = walk_x(self.rect(), target_x, ZOMBIE_WALK_SPEED * step, world);
            }
            ZombieState::Floating => self.pos.y -= ZOMBIE_FLOAT_RISE_SPEED * step,
        }
    }

    fn arms_came_down(&self) -> bool {
        self.state == ZombieState::Floating && self.seconds_in_state >= ZOMBIE_FLOAT_SECONDS
    }
}

/// 땅 위의 몬스터 `body`가 `target_x` 쪽으로 걸은 뒤의 x. 고리의 가까운 쪽으로 걷는다 (임시-21).
/// 상자와 구멍은 넘지 못한다 (US-039, D35, US-051).
pub fn walk_x(body: Rect, target_x: f32, longest_move: f32, world: &World) -> f32 {
    let move_x = shortest_dx(body.center().x, target_x).clamp(-longest_move, longest_move);
    let mut moved = Rect::new(body.x + move_x, body.y, body.w, body.h);
    for wall in world.monster_walls() {
        if really_overlaps(moved, wall) {
            let wall = nearest_copy(wall, moved.center().x);
            moved.x = if move_x > 0.0 { wall.x - body.w } else { wall.x + wall.w };
        }
    }
    world.wrap_x(moved.x)
}

/// 이번 차례 첫 몬스터가 왼쪽에서 나오면 true.
pub fn pick_left_side(random: &mut SeededRandom) -> bool {
    random.next_unit() < ZOMBIE_SPAWN_LEFT_CHANCE
}

/// 몬스터가 나올 x. `from_left` 쪽에 자리가 없으면 반대쪽이고, 양쪽 다 없으면 None이라 이번에는 안 나온다.
/// 긴 맵은 `spot_on_long_map`이 정한다. M1 연습 맵은 보이는 화면의 끝 자리에서 나오고,
/// 주인공이 그 자리에 있으면 겹치지 않게 반대쪽이다 (D36).
/// `width`는 나올 몬스터 폭. `taken`은 이번 차례에 먼저 나온 몬스터들의 x.
pub fn spawn_x_on_side(from_left: bool, player_rect: Rect, width: f32, world: &World, taken: &[f32]) -> Option<f32> {
    if world.is_long_map() {
        let spot = |left| spot_on_long_map(left, player_rect, width, world, taken);
        return spot(from_left).or_else(|| spot(!from_left));
    }
    let screen_left = world.camera_left(player_rect.center().x);
    let left_x = free_ground_x(world.keep_on_map(screen_left - width, width), -1.0, width, world);
    let right_x = free_ground_x(world.keep_on_map(screen_left + SCREEN_WIDTH, width), 1.0, width, world);
    let (wanted_x, other_x) = if from_left { (left_x, right_x) } else { (right_x, left_x) };
    // 점프 중이어도 내려오면 겹치니까 가로만 본다.
    let player_is_there = player_rect.x < wanted_x + width && wanted_x < player_rect.x + player_rect.w;
    let x = if player_is_there { other_x } else { wanted_x };
    let away = if x == left_x { -1.0 } else { 1.0 };
    Some(step_aside_from_taken(x, away, width, world, taken))
}

/// 긴 맵에서 `from_left` 쪽에 나올 x. 보이는 화면 바로 바깥이지만 (임시-1), 그 사이에 몬스터가 못 넘는 벽이
/// 있으면 가장 가까운 벽의 주인공 쪽에 붙어서 나온다 (D48을 지키면서 상자 뒤에 숨지 못하게 한다).
/// 몸 가운데끼리 `MONSTER_SPAWN_MIN_DISTANCE`보다 가까우면 None이다.
/// 주인공이 밟고 있는 벽(상자 위, 구멍 위·안)과 거기 이어진 벽은 사이의 벽으로 치지 않고 그 바깥부터 본다.
fn spot_on_long_map(from_left: bool, player_rect: Rect, width: f32, world: &World, taken: &[f32]) -> Option<f32> {
    let height = ZOMBIE_HEIGHT.max(SKELETON_HEIGHT);
    let center = player_rect.center().x;
    let walls: Vec<Rect> = world
        .monster_walls()
        .filter(|wall| wall.y < world.ground_y && world.ground_y - height < wall.y + wall.h)
        .map(|wall| nearest_copy(wall, center))
        .collect();

    let (mut left, mut right) = (player_rect.x, player_rect.x + player_rect.w);
    if walls.iter().any(|wall| wall.x < right && left < wall.x + wall.w) {
        for _ in 0..walls.len() {
            for wall in &walls {
                if wall.x <= right && left <= wall.x + wall.w {
                    left = left.min(wall.x);
                    right = right.max(wall.x + wall.w);
                }
            }
        }
    }

    let screen_left = world.camera_left(center);
    let mut x = if from_left {
        let wall_edge = walls.iter().map(|wall| wall.x + wall.w).filter(|edge| *edge <= left).fold(f32::MIN, f32::max);
        (screen_left - width).max(wall_edge)
    } else {
        let wall_edge = walls.iter().map(|wall| wall.x).filter(|edge| *edge >= right).fold(f32::MAX, f32::min);
        (screen_left + SCREEN_WIDTH).min(wall_edge - width)
    };
    // 먼저 나온 몬스터가 있으면 주인공 쪽으로 비킨다. 바깥쪽은 벽일 수 있다.
    for _ in 0..taken.len() {
        if taken.iter().any(|other| shortest_dx(x, *other).abs() < width) {
            x += if from_left { width } else { -width };
        }
    }
    let fits = if from_left { x + width <= left } else { x >= right };
    let distance = shortest_dx(center, x + width / 2.0).abs();
    (fits && distance >= MONSTER_SPAWN_MIN_DISTANCE).then(|| world.wrap_x(x))
}

/// 먼저 나온 몬스터와 겹치지 않을 때까지 `away` 쪽으로 비킨다. M1 맵 끝이라 못 가면 반대쪽으로.
fn step_aside_from_taken(mut x: f32, away: f32, width: f32, world: &World, taken: &[f32]) -> f32 {
    for _ in 0..taken.len() {
        if !taken.iter().any(|other| shortest_dx(x, *other).abs() < width) {
            break;
        }
        let farther = free_ground_x(x + away * width, away, width, world);
        x = if farther == x { free_ground_x(x - away * width, -away, width, world) } else { farther };
    }
    x
}

/// `x`가 상자나 구멍에 걸리면 `away`(-1 왼쪽, 1 오른쪽) 쪽의 가장 가까운 빈 땅 (임시-1).
fn free_ground_x(mut x: f32, away: f32, width: f32, world: &World) -> f32 {
    let height = ZOMBIE_HEIGHT.max(SKELETON_HEIGHT);
    for _ in 0..=world.obstacles.len() + 1 {
        let body = Rect::new(x, world.ground_y - height, width, height);
        let Some(wall) = world.monster_walls().find(|wall| really_overlaps(body, *wall)) else {
            break;
        };
        let wall = nearest_copy(wall, body.center().x);
        x = if away < 0.0 { wall.x - width } else { wall.x + wall.w };
    }
    world.keep_on_map(x, width)
}

/// 시드를 넣으면 늘 같은 순서로 나오는 작은 난수. 테스트에서 결과를 고정할 수 있다.
/// 비슷한 시드(1, 2, 3 …)에서도 결과가 고르게 섞이도록 murmur3의 섞기 단계를 쓴다.
pub struct SeededRandom {
    state: u32,
}

impl SeededRandom {
    pub fn new(seed: u32) -> SeededRandom {
        SeededRandom { state: seed }
    }

    /// 0 이상 1 미만의 수.
    pub fn next_unit(&mut self) -> f32 {
        self.state = self.state.wrapping_add(0x9E37_79B9);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 16)).wrapping_mul(0x85EB_CA6B);
        mixed = (mixed ^ (mixed >> 13)).wrapping_mul(0xC2B2_AE35);
        mixed ^= mixed >> 16;
        (mixed >> 8) as f32 / (1u32 << 24) as f32
    }
}

/// 맵에 있는 좀비 전부. 언제 나올지는 스켈레톤과 함께 `Combat`이 정한다.
pub struct Zombies {
    pub list: Vec<Zombie>,
}

impl Zombies {
    pub fn new() -> Zombies {
        Zombies { list: Vec::new() }
    }

    /// 밤에 짧은 시간 `step` 동안: 움직이고, 팔을 내린 좀비를 터뜨린다.
    pub fn step(&mut self, step: f32, player_rect: Rect, world: &World, events: &mut Vec<CombatEvent>) {
        let player_x = player_rect.center().x;
        for zombie in &mut self.list {
            zombie.step(step, player_x, world);
        }
        self.list.retain(|zombie| {
            let popped = zombie.arms_came_down();
            if popped {
                events.push(CombatEvent::ZombiePopped { at: zombie.rect().center() });
            }
            !popped
        });
    }

    /// 걷고 있는 좀비가 플레이어 몸에 닿았는지. 떠 있는 좀비는 공격하지 않는다.
    /// 선으로만 닿은 건 빼야 상자 위 발이 좀비 머리에 맞지 않는다 (REVIEW-006 2번).
    pub fn any_walking_zombie_touches(&self, player_rect: Rect) -> bool {
        self.list.iter().any(|z| z.state == ZombieState::Walking && really_overlaps(z.rect(), player_rect))
    }
}
