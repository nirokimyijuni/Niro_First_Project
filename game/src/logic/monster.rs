//! 좀비 카스테라 (D29): 밤에 화면 끝에서 나타나 플레이어 쪽으로 걸어온다.
//!
//! - 3마리 중 1마리쯤은 나타나고 조금 뒤 둥실 떠오르고, 팔을 내리는 순간 펑 터진다 (US-042).
//! - 맞고 쓰러지는 일과 플레이어 체력은 `combat.rs`가 맡는다.

use super::combat::CombatEvent;
use super::world::{really_overlaps, World};
use crate::config::{
    SCREEN_WIDTH, SKELETON_HEIGHT, ZOMBIE_FLOAT_RISE_SPEED, ZOMBIE_FLOAT_SECONDS, ZOMBIE_FLOAT_START_SECONDS,
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

/// 땅 위의 몬스터 `body`가 `target_x` 쪽으로 걸은 뒤의 x. 상자와 구멍은 넘지 못한다 (US-039, D35, US-051).
pub fn walk_x(body: Rect, target_x: f32, longest_move: f32, world: &World) -> f32 {
    let move_x = (target_x - body.center().x).clamp(-longest_move, longest_move);
    let mut moved = Rect::new(body.x + move_x, body.y, body.w, body.h);
    for wall in world.monster_walls() {
        if really_overlaps(moved, wall) {
            moved.x = if move_x > 0.0 { wall.x - body.w } else { wall.x + wall.w };
        }
    }
    moved.x
}

/// 이번 차례 첫 몬스터가 왼쪽에서 나오면 true.
pub fn pick_left_side(random: &mut SeededRandom) -> bool {
    random.next_unit() < ZOMBIE_SPAWN_LEFT_CHANCE
}

/// 보이는 화면의 `from_left` 쪽 바로 바깥의 x (임시-1). 그쪽이 맵 밖이면 맵 끝 자리다 (D36).
/// 주인공이 그 자리에 있으면 겹치지 않게 반대쪽이다. 점프 중이어도 내려오면 겹치니까 가로만 본다.
/// `width`는 나올 몬스터 폭.
/// `taken`은 이번 차례에 먼저 나온 몬스터들의 x. 겹치면 화면에서 더 먼 쪽으로 비켜 선다.
pub fn spawn_x_on_side(from_left: bool, player_rect: Rect, width: f32, world: &World, taken: &[f32]) -> f32 {
    let screen_left = world.camera_left(player_rect.center().x);
    let left_x = free_ground_x((screen_left - width).max(0.0), -1.0, width, world);
    let right_x = free_ground_x((screen_left + SCREEN_WIDTH).min(world.width - width), 1.0, width, world);
    let (wanted_x, other_x) = if from_left { (left_x, right_x) } else { (right_x, left_x) };
    let player_is_there = player_rect.x < wanted_x + width && wanted_x < player_rect.x + player_rect.w;
    let x = if player_is_there { other_x } else { wanted_x };
    let away = if x == left_x { -1.0 } else { 1.0 };
    step_aside_from_taken(x, away, width, world, taken)
}

/// 먼저 나온 몬스터와 겹치지 않을 때까지 `away` 쪽으로 비킨다. 맵 끝이라 못 가면 반대쪽으로.
fn step_aside_from_taken(mut x: f32, away: f32, width: f32, world: &World, taken: &[f32]) -> f32 {
    for _ in 0..taken.len() {
        if !taken.iter().any(|other| (x - other).abs() < width) {
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
        x = if away < 0.0 { wall.x - width } else { wall.x + wall.w };
    }
    x.clamp(0.0, world.width - width)
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
