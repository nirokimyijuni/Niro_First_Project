//! 무기 (US-052, US-053): 하늘에서 떨어지고, 닿으면 줍고, 바꿔 들 수 있다.
//! 무기를 든 공격은 `Combat::attack_with`가 맡는다.

use super::monster::SeededRandom;
use super::world::{really_overlaps, World};
use crate::config::{
    AXE_ATTACK, AXE_PICKED_TEXT, AXE_RANGE, GUN_ATTACK, GUN_DROP_CHANCE, GUN_PICKED_TEXT, GUN_RANGE, GUN_RANGE_MAX, GUN_RANGE_PER_UPGRADE,
    GUN_UPGRADE_DROP_CHANCE, HANDS_ATTACK, PLAYER_ATTACK_RANGE, SWORD_ATTACK, SWORD_PICKED_TEXT, SWORD_RANGE, WEAPON_ATTACK_PER_UPGRADE,
    WEAPON_FALL_SPEED, WEAPON_GROUND_SECONDS, WEAPON_LEVEL_DISPLAY_CAP, WEAPON_SIZE,
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

    /// 강화하지 않은 처음 공격력. 강화한 값은 `WeaponLevels::attack`.
    pub fn attack(self) -> u32 {
        match self {
            Weapon::Hands => HANDS_ATTACK,
            Weapon::Sword => SWORD_ATTACK,
            Weapon::Axe => AXE_ATTACK,
            Weapon::Gun => GUN_ATTACK,
        }
    }

    /// 강화하지 않은 처음 닿는 거리 (픽셀). 강화한 값은 `WeaponLevels::range`.
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

/// 무기마다 "강화 횟수" (D115, US-127~129). 처음 주운 무기는 0이고 이미 가진 무기를 또 주울 때마다 1씩 는다.
/// 칼·도끼·총만 강화되고 맨손은 늘 0이다. 한 판 안에서만 기억한다 (새 판은 `default`로 0).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WeaponLevels {
    sword: u32,
    axe: u32,
    gun: u32,
}

impl WeaponLevels {
    /// 칼·도끼·총의 횟수를 정해서 만든다 (시험에서 아주 큰 횟수를 볼 때 쓴다).
    #[allow(dead_code)] // 로건이 화면에 연결하기 전까지 시험에서만 쓴다
    pub fn with_counts(sword: u32, axe: u32, gun: u32) -> WeaponLevels {
        WeaponLevels { sword, axe, gun }
    }

    /// 강화 횟수. 맨손은 0.
    pub fn level(&self, kind: Weapon) -> u32 {
        match kind {
            Weapon::Hands => 0,
            Weapon::Sword => self.sword,
            Weapon::Axe => self.axe,
            Weapon::Gun => self.gun,
        }
    }

    /// 강화 횟수를 1 올리고 새 횟수를 돌려준다. 맨손이면 아무 일 없이 None. 아주 커도 넘치지 않는다.
    pub fn upgrade(&mut self, kind: Weapon) -> Option<u32> {
        let slot = match kind {
            Weapon::Hands => return None,
            Weapon::Sword => &mut self.sword,
            Weapon::Axe => &mut self.axe,
            Weapon::Gun => &mut self.gun,
        };
        *slot = slot.saturating_add(1);
        Some(*slot)
    }

    /// 강화한 공격력: 처음 값 + 횟수 x `WEAPON_ATTACK_PER_UPGRADE`. 맨손은 그대로다. 총도 범위가 멈춘 뒤에 계속 오른다.
    pub fn attack(&self, kind: Weapon) -> u32 {
        match kind {
            Weapon::Hands => kind.attack(),
            _ => kind.attack().saturating_add(self.level(kind).saturating_mul(WEAPON_ATTACK_PER_UPGRADE)),
        }
    }

    /// 강화한 닿는 거리 (픽셀). 총만 늘고 `GUN_RANGE_MAX`에서 멈춘다. 나머지는 그대로다.
    pub fn range(&self, kind: Weapon) -> f32 {
        match kind {
            Weapon::Gun => (GUN_RANGE + GUN_RANGE_PER_UPGRADE * self.gun as f32).min(GUN_RANGE_MAX),
            _ => kind.range(),
        }
    }
}

/// 화면에 보여 줄 강화 숫자: `WEAPON_LEVEL_DISPLAY_CAP`보다 크면 그 값으로 줄인다.
pub fn capped_for_display(level: u32) -> u32 {
    level.min(WEAPON_LEVEL_DISPLAY_CAP)
}

/// 강화 숫자가 표시 상한을 넘어 "99+"처럼 보여야 하면 true.
pub fn is_over_display_cap(level: u32) -> bool {
    level > WEAPON_LEVEL_DISPLAY_CAP
}

/// 무기를 강화했다는 소식. 화면은 "+n"과 반짝이를 그린다 (`Game::take_weapon_events`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponEvent {
    /// `weapon`을 또 주워서 강화 횟수가 `level`(1 이상)이 되었다. 손의 무기는 그대로다.
    Upgraded { weapon: Weapon, level: u32 },
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
    /// 화면 바로 위에서 가운데가 `center_x`인 자리로 떨어지기 시작한다. 맵 밖 자리면 맵 안으로 옮긴다 (긴 맵은 감는다).
    pub fn above(kind: Weapon, center_x: f32, world: &World) -> DroppedWeapon {
        let x = world.keep_on_map(center_x - WEAPON_SIZE / 2.0, WEAPON_SIZE);
        let rect = Rect::new(x, -WEAPON_SIZE, WEAPON_SIZE, WEAPON_SIZE);
        let landing_y = world.weapon_floor_y_under(rect) - WEAPON_SIZE;
        DroppedWeapon { kind, rect, landing_y, seconds_left: WEAPON_GROUND_SECONDS }
    }

    pub fn is_falling(&self) -> bool {
        self.rect.y < self.landing_y
    }

    /// 맵이 바뀌었을 때(상자가 부서지거나 돌아올 때, D87) 멈출 곳을 다시 정한다.
    /// 밑의 상자가 없어졌으면 다시 떨어지고, 상자가 돌아오면 그 위로 올라간다.
    pub fn land_again(&mut self, world: &World) {
        self.landing_y = world.weapon_floor_y_under(self.rect) - WEAPON_SIZE;
        self.rect.y = self.rect.y.min(self.landing_y);
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
    /// 이번 판에 땅에서 주운 무기 수. 이미 가진 무기를 또 주워도 1개로 센다 (US-075 아이템 세기).
    pub picked_up_count: u32,
    levels: WeaponLevels,
    /// 아직 `take_events`로 내보내지 않은 강화 소식.
    events: Vec<WeaponEvent>,
}

impl Weapons {
    /// `seed`가 같으면 떨어지는 무기 순서가 늘 같다.
    pub fn new(seed: u32) -> Weapons {
        // 맵·싸움·미스와 같은 seed를 쓰므로 섞어야 서로 따라가지 않는다.
        let random = SeededRandom::new(seed.wrapping_mul(41).wrapping_add(53));
        Weapons { dropped: Vec::new(), owned: Vec::new(), held: Weapon::Hands,
            random,
            picked_up_count: 0,
            levels: WeaponLevels::default(),
            events: Vec::new(),
        }
    }

    /// 무기마다 강화 횟수.
    pub fn levels(&self) -> WeaponLevels {
        self.levels
    }

    /// 손에 든 무기의 강화한 공격력.
    pub fn held_attack(&self) -> u32 {
        self.levels.attack(self.held)
    }

    /// 손에 든 무기의 강화한 닿는 거리 (벽에 막히기 전).
    pub fn held_range(&self) -> f32 {
        self.levels.range(self.held)
    }

    /// 쌓인 강화 소식을 꺼내 간다.
    pub fn take_events(&mut self) -> Vec<WeaponEvent> {
        std::mem::take(&mut self.events)
    }

    /// 지금 손에 든 것.
    pub fn held(&self) -> Weapon {
        self.held
    }

    /// 가진 무기를 바꾸기 순서로. 비었으면 바꾸기 버튼을 숨긴다.
    pub fn owned(&self) -> Vec<Weapon> {
        Weapon::ORDER.into_iter().filter(|w| self.owned.contains(w)).collect()
    }

    /// 가진 무기면 true. 맨손은 늘 가졌다.
    pub fn has(&self, kind: Weapon) -> bool {
        kind == Weapon::Hands || self.owned.contains(&kind)
    }

    /// 무기를 바로 가진 무기로 넣고 손에 든다. 수학 문제 "아이템"으로는 세지 않는다.
    /// 상점은 무기를 팔지 않는다 (부모님 결정 2026-11-14). 시험에서 무기를 줄 때 쓴다.
    pub fn add_bought(&mut self, kind: Weapon) {
        if !self.has(kind) {
            self.owned.push(kind);
        }
        self.held = kind;
    }

    /// 가진 무기(또는 맨손)를 손에 든다. 없는 무기면 그대로다.
    pub fn hold(&mut self, kind: Weapon) {
        if self.has(kind) {
            self.held = kind;
        }
    }

    /// 가졌거나 땅에 놓여 있으면 true. 그런 무기는 "없는 무기"로 치지 않는다.
    fn is_around(&self, kind: Weapon) -> bool {
        self.owned.contains(&kind) || self.dropped.iter().any(|d| d.kind == kind)
    }

    /// 무기 하나가 `center_x`에 떨어진다: 칼이 없으면 칼, 도끼가 없으면 도끼다.
    /// 둘 다 있으면 총이 땅에 없을 때 총이 나올 수 있다: 총이 없으면 `GUN_DROP_CHANCE`, 이미 가졌으면
    /// `GUN_UPGRADE_DROP_CHANCE`(강화 재료)다. 총이 아니면 칼·도끼 중 아무거나다 (이미 가진 무기도 나온다, D115).
    pub fn drop_one(&mut self, center_x: f32, world: &World) {
        let gun_on_ground = self.dropped.iter().any(|d| d.kind == Weapon::Gun);
        let gun_chance = if self.owned.contains(&Weapon::Gun) { GUN_UPGRADE_DROP_CHANCE } else { GUN_DROP_CHANCE };
        let kind = if !self.is_around(Weapon::Sword) {
            Weapon::Sword
        } else if !self.is_around(Weapon::Axe) {
            Weapon::Axe
        } else if !gun_on_ground && self.random.next_unit() < gun_chance {
            Weapon::Gun
        } else if self.random.next_unit() < 0.5 {
            Weapon::Sword
        } else {
            Weapon::Axe
        };
        self.dropped.push(DroppedWeapon::above(kind, center_x, world));
    }

    /// 떨어뜨리고, 플레이어 몸에 닿은 무기를 줍고, 시간이 다 된 무기를 없앤다. 돌려주는 값: 새로 얻은 무기들.
    /// 이미 가진 무기는 땅에서 사라지고 그 무기가 강화된다 (`WeaponEvent::Upgraded`, `take_events`). 손의 무기는 그대로다 (D115).
    pub fn update(&mut self, dt: f32, player_rect: Rect) -> Vec<Weapon> {
        let mut picked = Vec::new();
        for weapon in &mut self.dropped {
            weapon.update(dt.max(0.0));
        }
        let (owned, held, count) = (&mut self.owned, &mut self.held, &mut self.picked_up_count);
        let (levels, events) = (&mut self.levels, &mut self.events);
        self.dropped.retain(|weapon| {
            let touched = really_overlaps(weapon.rect, player_rect);
            if touched {
                *count += 1;
            }
            if touched && !owned.contains(&weapon.kind) {
                owned.push(weapon.kind);
                *held = weapon.kind;
                picked.push(weapon.kind);
            } else if touched {
                if let Some(level) = levels.upgrade(weapon.kind) {
                    events.push(WeaponEvent::Upgraded { weapon: weapon.kind, level });
                }
            }
            !touched && weapon.seconds_left > 0.0
        });
        picked
    }
}
