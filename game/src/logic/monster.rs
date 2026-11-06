//! 좀비 카스테라 (D29): 밤에 화면 끝에서 나타나 플레이어 쪽으로 걸어온다.
//!
//! - 3마리 중 1마리쯤은 나타나고 조금 뒤 둥실 떠오르고, 팔을 내리는 순간 펑 터진다 (US-042).
//! - 맞고 쓰러지는 일과 플레이어 체력은 `combat.rs`가 맡는다.

use super::combat::CombatEvent;
use super::world::World;
use crate::config::{
    ZOMBIE_FLOAT_CHANCE, ZOMBIE_FLOAT_RISE_SPEED, ZOMBIE_FLOAT_SECONDS, ZOMBIE_FLOAT_START_SECONDS,
    ZOMBIE_HEIGHT, ZOMBIE_HIT_FLASH_SECONDS, ZOMBIE_HP, ZOMBIE_MAX_ON_SCREEN, ZOMBIE_SPAWN_LEFT_CHANCE,
    ZOMBIE_SPAWN_MIN_DISTANCE, ZOMBIE_SPAWN_SECONDS, ZOMBIE_WALK_SPEED, ZOMBIE_WIDTH,
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

    /// 공격을 한 번 맞는다.
    pub fn take_hit(&mut self) {
        self.hp = self.hp.saturating_sub(1);
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
            ZombieState::Walking => self.walk_toward(target_x, step, world),
            ZombieState::Floating => self.pos.y -= ZOMBIE_FLOAT_RISE_SPEED * step,
        }
    }

    /// 땅 위를 걷기만 하므로 땅 아래로 빠질 일이 없다. 상자는 넘지 못한다 (US-039).
    fn walk_toward(&mut self, target_x: f32, step: f32, world: &World) {
        let longest_move = ZOMBIE_WALK_SPEED * step;
        let move_x = (target_x - self.rect().center().x).clamp(-longest_move, longest_move);
        self.pos.x += move_x;
        for wall in &world.obstacles {
            if really_overlaps(self.rect(), *wall) {
                self.pos.x = if move_x > 0.0 { wall.x - ZOMBIE_WIDTH } else { wall.x + wall.w };
            }
        }
    }

    fn arms_came_down(&self) -> bool {
        self.state == ZombieState::Floating && self.seconds_in_state >= ZOMBIE_FLOAT_SECONDS
    }
}

/// 모서리끼리 딱 닿기만 한 건 겹친 게 아니다. `Rect::overlaps`는 닿기만 해도 겹침이라,
/// 상자 옆에 붙은 좀비가 상자 반대편으로 건너뛰는 일이 생겨서 따로 쓴다.
fn really_overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// 시드를 넣으면 늘 같은 순서로 나오는 작은 난수. 테스트에서 결과를 고정할 수 있다.
/// 비슷한 시드(1, 2, 3 …)에서도 결과가 고르게 섞이도록 murmur3의 섞기 단계를 쓴다.
struct SeededRandom {
    state: u32,
}

impl SeededRandom {
    /// 0 이상 1 미만의 수.
    fn next_unit(&mut self) -> f32 {
        self.state = self.state.wrapping_add(0x9E37_79B9);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 16)).wrapping_mul(0x85EB_CA6B);
        mixed = (mixed ^ (mixed >> 13)).wrapping_mul(0xC2B2_AE35);
        mixed ^= mixed >> 16;
        (mixed >> 8) as f32 / (1u32 << 24) as f32
    }
}

/// 맵에 있는 좀비 전부와, 다음 좀비가 나올 때까지의 시간.
pub struct Zombies {
    pub list: Vec<Zombie>,
    seconds_to_next_spawn: f32,
    random: SeededRandom,
}

impl Zombies {
    /// 좀비가 없는 상태. 밤이 되면 첫 좀비가 바로 나온다.
    pub fn new(seed: u32) -> Zombies {
        Zombies { list: Vec::new(), seconds_to_next_spawn: 0.0, random: SeededRandom { state: seed } }
    }

    /// 아침: 좀비가 모두 사라지고, 다음 밤에 첫 좀비가 바로 나올 준비를 한다.
    pub fn clear(&mut self) {
        self.list.clear();
        self.seconds_to_next_spawn = 0.0;
    }

    /// 밤에 짧은 시간 `step` 동안: 새 좀비를 내보내고, 움직이고, 팔을 내린 좀비를 터뜨린다.
    pub fn step(&mut self, step: f32, player_rect: Rect, world: &World, events: &mut Vec<CombatEvent>) {
        let player_x = player_rect.center().x;
        self.spawn_when_ready(step, player_x, world, events);
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
    pub fn any_walking_zombie_touches(&self, player_rect: Rect) -> bool {
        self.list.iter().any(|z| z.state == ZombieState::Walking && z.rect().overlaps(&player_rect))
    }

    fn spawn_when_ready(&mut self, step: f32, player_x: f32, world: &World, events: &mut Vec<CombatEvent>) {
        if self.list.len() >= ZOMBIE_MAX_ON_SCREEN {
            return;
        }
        self.seconds_to_next_spawn -= step;
        if self.seconds_to_next_spawn <= 0.0 {
            self.seconds_to_next_spawn += ZOMBIE_SPAWN_SECONDS;
            let x = self.pick_spawn_x(player_x, world);
            let will_float = self.random.next_unit() < ZOMBIE_FLOAT_CHANCE;
            let zombie = Zombie::new(x, world.ground_y, will_float);
            events.push(CombatEvent::ZombieAppeared { at: zombie.rect().center() });
            self.list.push(zombie);
        }
    }

    /// 왼쪽 끝이나 오른쪽 끝. 플레이어 바로 옆이면 반대쪽 끝으로.
    fn pick_spawn_x(&mut self, player_x: f32, world: &World) -> f32 {
        let wants_left = self.random.next_unit() < ZOMBIE_SPAWN_LEFT_CHANCE;
        let left_is_safe = player_x >= ZOMBIE_SPAWN_MIN_DISTANCE;
        let right_is_safe = world.width - player_x >= ZOMBIE_SPAWN_MIN_DISTANCE;
        if (wants_left && left_is_safe) || !right_is_safe {
            0.0
        } else {
            world.width - ZOMBIE_WIDTH
        }
    }
}
