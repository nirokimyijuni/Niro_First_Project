//! 무기 (US-052, US-053): 하늘에서 떨어지고, 닿으면 줍고, 바꿔 들 수 있다.
//! 무기를 든 공격은 `Combat::attack_with`가 맡는다.

use super::monster::SeededRandom;
use super::world::{really_overlaps, World};
use crate::config::{
    AXE_ATTACK, AXE_PICKED_TEXT, AXE_RANGE, GUN_ATTACK, GUN_DROP_CHANCE, GUN_PICKED_TEXT, GUN_RANGE, HANDS_ATTACK, PLAYER_ATTACK_RANGE,
    SWORD_ATTACK, SWORD_PICKED_TEXT, SWORD_RANGE, WEAPON_FALL_SPEED, WEAPON_GROUND_SECONDS, WEAPON_SIZE,
};
use macroquad::math::Rect;

/// 손에 드는 것. 화면은 이것을 보고 그림을 고른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weapon {
    Hands,
    Sword,
    Axe,
    Gun,
}

impl Weapon {
    /// 바꾸기 순서 (임시-8).
    pub const ORDER: [Weapon; 4] = [Weapon::Hands, Weapon::Sword, Weapon::Axe, Weapon::Gun];

    pub fn attack(self) -> u32 {
        match self {
            Weapon::Hands => HANDS_ATTACK,
            Weapon::Sword => SWORD_ATTACK,
            Weapon::Axe => AXE_ATTACK,
            Weapon::Gun => GUN_ATTACK,
        }
    }

    /// 앞쪽으로 닿는 거리 (픽셀).
    pub fn range(self) -> f32 {
        match self {
            Weapon::Hands => PLAYER_ATTACK_RANGE,
            Weapon::Sword => SWORD_RANGE,
            Weapon::Axe => AXE_RANGE,
            Weapon::Gun => GUN_RANGE,
        }
    }

    /// 주웠을 때 문구. 맨손은 줍는 일이 없다.
    pub fn picked_text(self) -> &'static str {
        match self {
            Weapon::Hands => "",
            Weapon::Sword => SWORD_PICKED_TEXT,
            Weapon::Axe => AXE_PICKED_TEXT,
            Weapon::Gun => GUN_PICKED_TEXT,
        }
    }
}

/// 떨어지는 중이거나 놓여 있는 무기 하나.
pub struct DroppedWeapon {
    pub kind: Weapon,
    /// 무기 네모 (왼쪽 위 기준).
    pub rect: Rect,
    /// 멈출 때의 `rect.y`.
    landing_y: f32,
    seconds_left: f32,
}

impl DroppedWeapon {
    /// 화면 바로 위에서 가운데가 `center_x`인 자리로 떨어지기 시작한다. 맵 밖 자리면 맵 안으로 옮긴다.
    pub fn above(kind: Weapon, center_x: f32, world: &World) -> DroppedWeapon {
        let x = (center_x - WEAPON_SIZE / 2.0).clamp(0.0, world.width - WEAPON_SIZE);
        let rect = Rect::new(x, -WEAPON_SIZE, WEAPON_SIZE, WEAPON_SIZE);
        let landing_y = world.surface_y_under(rect) - WEAPON_SIZE;
        DroppedWeapon { kind, rect, landing_y, seconds_left: WEAPON_GROUND_SECONDS }
    }

    pub fn is_falling(&self) -> bool {
        self.rect.y < self.landing_y
    }

    /// 사라질 때까지 남은 초. 땅에 닿은 때부터 줄어든다. 화면은 끝이 가까우면 깜빡이게 할 수 있다.
    pub fn seconds_left(&self) -> f32 {
        self.seconds_left
    }

    /// `dt`초 지낸다: 떨어지는 중이면 떨어지고, 땅에 있으면 남은 시간이 준다.
    fn update(&mut self, dt: f32) {
        if self.is_falling() {
            self.rect.y = (self.rect.y + WEAPON_FALL_SPEED * dt).min(self.landing_y);
        } else {
            self.seconds_left = (self.seconds_left - dt).max(0.0);
        }
    }
}

/// 맵의 무기와 플레이어가 가진 무기. 새 게임을 하면 새로 만든다 (US-053).
pub struct Weapons {
    pub dropped: Vec<DroppedWeapon>,
    /// 주운 무기. 맨손은 넣지 않는다.
    owned: Vec<Weapon>,
    held: Weapon,
    random: SeededRandom,
}

impl Weapons {
    /// `seed`가 같으면 떨어지는 무기 순서가 늘 같다.
    pub fn new(seed: u32) -> Weapons {
        // 맵·싸움·미스와 같은 seed를 쓰므로 섞어야 서로 따라가지 않는다.
        let random = SeededRandom::new(seed.wrapping_mul(41).wrapping_add(53));
        Weapons { dropped: Vec::new(), owned: Vec::new(), held: Weapon::Hands, random }
    }

    /// 지금 손에 든 것.
    pub fn held(&self) -> Weapon {
        self.held
    }

    /// 가진 무기를 바꾸기 순서로. 비었으면 바꾸기 버튼을 숨긴다.
    pub fn owned(&self) -> Vec<Weapon> {
        Weapon::ORDER.into_iter().filter(|w| self.owned.contains(w)).collect()
    }

    /// 무기 하나가 `center_x`에 떨어진다. `GUN_DROP_CHANCE`로 총이고 (이미 가졌으면 안 나온다),
    /// 아니면 칼·도끼 중 아직 없는 것(땅에 놓인 것도 뺀다)이고, 그런 것이 없으면 둘 중 아무거나다.
    pub fn drop_one(&mut self, center_x: f32, world: &World) {
        let wants_gun = self.random.next_unit() < GUN_DROP_CHANCE;
        let blades = [Weapon::Sword, Weapon::Axe];
        let is_new = |w: &&Weapon| !self.owned.contains(w) && !self.dropped.iter().any(|d| d.kind == **w);
        let new_blades: Vec<Weapon> = blades.iter().filter(is_new).copied().collect();
        let choices = if new_blades.is_empty() { blades.to_vec() } else { new_blades };
        let blade = choices[(self.random.next_unit() * choices.len() as f32) as usize];
        let kind = if wants_gun && !self.owned.contains(&Weapon::Gun) { Weapon::Gun } else { blade };
        self.dropped.push(DroppedWeapon::above(kind, center_x, world));
    }

    /// 떨어뜨리고, 플레이어 몸에 닿은 무기를 줍고, 시간이 다 된 무기를 없앤다. 돌려주는 값: 새로 얻은 무기들.
    /// 이미 가진 무기는 땅에서 사라지기만 하고 손의 무기도 그대로다 (US-052).
    pub fn update(&mut self, dt: f32, player_rect: Rect) -> Vec<Weapon> {
        let mut picked = Vec::new();
        for weapon in &mut self.dropped {
            weapon.update(dt.max(0.0));
        }
        let (owned, held) = (&mut self.owned, &mut self.held);
        self.dropped.retain(|weapon| {
            let touched = really_overlaps(weapon.rect, player_rect);
            if touched && !owned.contains(&weapon.kind) {
                owned.push(weapon.kind);
                *held = weapon.kind;
                picked.push(weapon.kind);
            }
            !touched && weapon.seconds_left > 0.0
        });
        picked
    }

    /// 맨손 → 칼 → 도끼 → 총 → 맨손 순서로 다음 것을 든다. 없는 무기는 건너뛴다 (임시-8).
    pub fn switch(&mut self) {
        let now = Weapon::ORDER.iter().position(|w| *w == self.held).unwrap_or(0);
        let mut next_in_order = (1..Weapon::ORDER.len()).map(|step| Weapon::ORDER[(now + step) % Weapon::ORDER.len()]);
        self.held = next_in_order.find(|w| *w == Weapon::Hands || self.owned.contains(w)).unwrap_or(self.held);
    }
}
