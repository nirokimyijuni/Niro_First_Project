//! 싸움 규칙: 플레이어 체력(D2), 공격, 좀비에게 닿기, 아침 회복(D27).
//!
//! `Combat` 하나가 체력과 좀비를 함께 들고 있다. 결과는 `CombatEvent`로 돌려주고,
//! 화면(로건)이 그걸 보고 "쓰러졌어요", "펑!" 같은 표시와 소리를 낸다.

use super::monster::Zombies;
use super::player::Facing;
use super::world::World;
use crate::config::{
    PHYSICS_STEP_SECONDS, PLAYER_ATTACK_RANGE, PLAYER_HP_PER_LEVEL, PLAYER_HURT_INVINCIBLE_SECONDS,
    PLAYER_START_HP, ZOMBIE_ATTACK,
};
use macroquad::math::{Rect, Vec2};

/// 화면(로건)이 보고 표시·소리를 낼 일. `at`은 좀비 몸 가운데.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CombatEvent {
    /// 화면 끝에 좀비가 나타났다.
    ZombieAppeared { at: Vec2 },
    /// 공격이 맞았고 좀비는 아직 서 있다.
    ZombieHit { at: Vec2, hp_left: u32 },
    /// 공격으로 좀비가 쓰러졌다 ("쓰러졌어요"). M5에서 경험치를 준다.
    ZombieDefeated { at: Vec2 },
    /// 날아오른 좀비가 팔을 내려 펑 터졌다 ("펑!"). 경험치는 없다.
    ZombiePopped { at: Vec2 },
    /// 플레이어가 좀비에게 닿아 체력이 줄었다.
    PlayerHurt { amount: u32, hp_left: u32 },
    /// 체력이 0이 되어 쓰러졌다 (게임오버). 한 판에 한 번만 온다.
    PlayerDown,
    /// 아침이 와서 체력이 다시 찼다 (D27).
    HealthRefilled,
}

/// 플레이어 체력.
pub struct PlayerHealth {
    hp: u32,
    max_hp: u32,
    invincible_seconds_left: f32,
}

impl PlayerHealth {
    pub fn new() -> PlayerHealth {
        PlayerHealth { hp: PLAYER_START_HP, max_hp: PLAYER_START_HP, invincible_seconds_left: 0.0 }
    }

    pub fn hp(&self) -> u32 {
        self.hp
    }

    pub fn max_hp(&self) -> u32 {
        self.max_hp
    }

    pub fn is_down(&self) -> bool {
        self.hp == 0
    }

    /// 맞은 뒤 다시 안 맞는 시간이면 true. 이 동안 캐릭터가 깜빡인다.
    pub fn is_invincible(&self) -> bool {
        self.invincible_seconds_left > 0.0
    }

    /// `amount`만큼 맞는다. 정말로 맞았으면 true, 무적이거나 이미 쓰러졌으면 false.
    fn take_hit(&mut self, amount: u32) -> bool {
        if self.is_invincible() || self.is_down() {
            return false;
        }
        self.hp = self.hp.saturating_sub(amount);
        self.invincible_seconds_left = PLAYER_HURT_INVINCIBLE_SECONDS;
        true
    }
}

impl Default for PlayerHealth {
    fn default() -> PlayerHealth {
        PlayerHealth::new()
    }
}

/// 플레이어 앞쪽의 공격이 닿는 네모. 높이는 플레이어 키만큼.
pub fn attack_box(player_rect: Rect, facing: Facing) -> Rect {
    let x = match facing {
        Facing::Right => player_rect.x + player_rect.w,
        Facing::Left => player_rect.x - PLAYER_ATTACK_RANGE,
    };
    Rect::new(x, player_rect.y, PLAYER_ATTACK_RANGE, player_rect.h)
}

/// 몸이 겹쳐 있어도 등 뒤는 맞지 않게, 가운데가 바라보는 쪽에 있는지 본다.
fn is_in_front(player_rect: Rect, facing: Facing, target: Rect) -> bool {
    match facing {
        Facing::Right => target.center().x >= player_rect.center().x,
        Facing::Left => target.center().x <= player_rect.center().x,
    }
}

/// 한 판의 싸움 상태. 새 게임을 하면 새로 만든다.
pub struct Combat {
    pub health: PlayerHealth,
    pub zombies: Zombies,
    /// 플레이어 레벨이자 몬스터 레벨 (D3: 같이 오른다). 1부터.
    pub level: u32,
    was_night: bool,
}

impl Combat {
    /// `seed`가 같으면 좀비가 나오는 쪽과 날아오르는 좀비가 늘 똑같다.
    pub fn new(seed: u32) -> Combat {
        Combat { health: PlayerHealth::new(), zombies: Zombies::new(seed), level: 1, was_night: false }
    }

    /// 한 프레임 진행. 쓰러진 뒤에는 아무것도 움직이지 않는다 (US-043).
    pub fn update(&mut self, dt: f32, is_night: bool, player_rect: Rect, world: &World) -> Vec<CombatEvent> {
        let mut events = Vec::new();
        if self.health.is_down() {
            return events;
        }

        if self.was_night && !is_night {
            self.zombies.clear();
            self.health.hp = self.health.max_hp;
            events.push(CombatEvent::HealthRefilled);
        }
        self.was_night = is_night;

        // 시간을 잘게 나눠서 움직인다 → 프레임이 길어도 좀비가 상자를 뚫지 않는다.
        let mut time_left = dt.max(0.0);
        while time_left > 0.0 && !self.health.is_down() {
            let step = time_left.min(PHYSICS_STEP_SECONDS);
            self.health.invincible_seconds_left = (self.health.invincible_seconds_left - step).max(0.0);
            if is_night {
                self.zombies.step(step, player_rect, world, &mut events);
                self.get_hurt_by_touching_zombie(player_rect, &mut events);
            }
            time_left -= step;
        }
        events
    }

    /// 공격 버튼을 "새로" 눌렀을 때 한 번 부른다. 닿는 곳의 좀비가 모두 체력 1씩 줄어든다.
    pub fn player_attack(&mut self, player_rect: Rect, facing: Facing) -> Vec<CombatEvent> {
        let mut events = Vec::new();
        if self.health.is_down() {
            return events;
        }
        let reach = attack_box(player_rect, facing);
        self.zombies.list.retain_mut(|zombie| {
            let body = zombie.rect();
            if !reach.overlaps(&body) || !is_in_front(player_rect, facing, body) {
                return true;
            }
            zombie.take_hit();
            if zombie.hp == 0 {
                events.push(CombatEvent::ZombieDefeated { at: body.center() });
                return false;
            }
            events.push(CombatEvent::ZombieHit { at: body.center(), hp_left: zombie.hp });
            true
        });
        events
    }

    /// 레벨이 1 오른다: 최대 체력 +5 (D2), 몬스터도 같이 (D3).
    // 레벨업(M5)에서 쓸 함수라 아직 부르는 곳이 없다. M5를 붙이면 이 줄을 지운다.
    #[allow(dead_code)]
    pub fn level_up(&mut self) {
        self.level += 1;
        self.health.max_hp += PLAYER_HP_PER_LEVEL;
    }

    fn get_hurt_by_touching_zombie(&mut self, player_rect: Rect, events: &mut Vec<CombatEvent>) {
        if !self.zombies.any_walking_zombie_touches(player_rect) {
            return;
        }
        if self.health.take_hit(ZOMBIE_ATTACK) {
            events.push(CombatEvent::PlayerHurt { amount: ZOMBIE_ATTACK, hp_left: self.health.hp });
            if self.health.is_down() {
                events.push(CombatEvent::PlayerDown);
            }
        }
    }
}
