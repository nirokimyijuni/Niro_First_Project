//! 상점 (US-062, US-070, US-071, US-072): 집에 들어가 코인으로 물건을 사고, 금·은·동을 판다.
//! 산 물건(포션 개수, 스케이트, 갑옷)도 여기 둔다. 코인·무기·가방과 함께 쓰는 일은 `Game::buy`, `Game::sell`이 한다.

use super::gathering::Material;
use crate::config::{
    COPPER_SELL_PRICE, DIAMOND_ARMOR_NAME, DIAMOND_ARMOR_PRICE, DIAMOND_ARMOR_SECONDS, GOLD_SELL_PRICE, HEALTH_POTION_NAME,
    HEALTH_POTION_PRICE, POISON_POTION_NAME, POISON_POTION_PRICE, RAINBOW_ARMOR_NAME, RAINBOW_ARMOR_PRICE, RAINBOW_ARMOR_SECONDS,
    SHOP_ALREADY_HAVE_TEXT, SHOP_NONE_TO_SELL_TEXT, SHOP_NOT_ENOUGH_COINS_TEXT, SILVER_SELL_PRICE, SKATES_NAME, SKATES_PRICE,
    SKATES_SPEED_MULTIPLIER,
};
use macroquad::math::Vec2;

/// 상점에서 파는 물건.
/// 칼·도끼·총은 팔지 않는다 (부모님 결정 2026-11-14): 무기는 미스·필드에서만 얻는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopItem {
    HealthPotion,
    PoisonPotion,
    Skates,
    RainbowArmor,
    DiamondArmor,
}

impl ShopItem {
    /// 상점 화면에 보여 주는 순서.
    pub const ALL: [ShopItem; 5] = [
        ShopItem::HealthPotion,
        ShopItem::PoisonPotion,
        ShopItem::Skates,
        ShopItem::RainbowArmor,
        ShopItem::DiamondArmor,
    ];

    /// 가격 (코인).
    pub fn price(self) -> u32 {
        match self {
            ShopItem::HealthPotion => HEALTH_POTION_PRICE,
            ShopItem::PoisonPotion => POISON_POTION_PRICE,
            ShopItem::Skates => SKATES_PRICE,
            ShopItem::RainbowArmor => RAINBOW_ARMOR_PRICE,
            ShopItem::DiamondArmor => DIAMOND_ARMOR_PRICE,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ShopItem::HealthPotion => HEALTH_POTION_NAME,
            ShopItem::PoisonPotion => POISON_POTION_NAME,
            ShopItem::Skates => SKATES_NAME,
            ShopItem::RainbowArmor => RAINBOW_ARMOR_NAME,
            ShopItem::DiamondArmor => DIAMOND_ARMOR_NAME,
        }
    }
}

/// 가방에 넣고 들고 다니다 쓰는 포션 (US-071).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Potion {
    Health,
    Poison,
}

/// 입으면 무적인 갑옷 (US-070).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Armor {
    Rainbow,
    Diamond,
}

impl Armor {
    /// 무적 시간 (게임 속 초, 임시-26).
    pub fn seconds(self) -> f32 {
        match self {
            Armor::Rainbow => RAINBOW_ARMOR_SECONDS,
            Armor::Diamond => DIAMOND_ARMOR_SECONDS,
        }
    }
}

/// 살 수 없는 이유.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CantBuy {
    NotEnoughCoins,
    AlreadyHave,
}

impl CantBuy {
    pub fn text(self) -> &'static str {
        match self {
            CantBuy::NotEnoughCoins => SHOP_NOT_ENOUGH_COINS_TEXT,
            CantBuy::AlreadyHave => SHOP_ALREADY_HAVE_TEXT,
        }
    }
}

/// 팔 수 없는 이유.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CantSell {
    /// 가진 것이 0개다. 철광석처럼 팔지 않는 것도 이것이다.
    NoneToSell,
}

impl CantSell {
    pub fn text(self) -> &'static str {
        match self {
            CantSell::NoneToSell => SHOP_NONE_TO_SELL_TEXT,
        }
    }
}

/// 상점 화면의 사기 줄 하나.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BuyRow {
    pub item: ShopItem,
    pub name: &'static str,
    pub price: u32,
    pub can_buy: bool,
    /// 살 수 없으면 그 이유 (`text()`로 문구).
    pub reason: Option<CantBuy>,
}

/// 상점 화면의 팔기 줄 하나.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SellRow {
    pub material: Material,
    pub name: &'static str,
    pub price: u32,
    /// 가방에 가진 개수.
    pub count: u32,
    pub can_sell: bool,
    pub reason: Option<CantSell>,
}

/// 파는 광물, 팔기 줄 순서 (임시-38). 철광석은 없다.
pub const SELLABLE: [Material; 3] = [Material::Gold, Material::Silver, Material::Copper];

/// 광물 파는 값. 팔지 않는 것은 None이다.
pub fn sell_price(material: Material) -> Option<u32> {
    match material {
        Material::Gold => Some(GOLD_SELL_PRICE),
        Material::Silver => Some(SILVER_SELL_PRICE),
        Material::Copper => Some(COPPER_SELL_PRICE),
        // 새 광물은 팔지 않는다 (임시-48, 아직 묻지 않았음).
        Material::Wood | Material::Iron | Material::Ruby | Material::Emerald | Material::Diamond => None,
    }
}

/// 상점에서 생긴 일과 산 물건을 쓴 일. `Game::take_shop_events`로 받는다 (GameEvent와 따로).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShopEvent {
    /// 집 문으로 상점에 들어갔다.
    Opened,
    /// 체력 포션을 썼다. `at`은 주인공 몸 가운데, `healed`는 찬 체력.
    HealthPotionUsed { at: Vec2, healed: u32 },
    /// 독약 포션을 뿌렸다 (초록 구름). `at`은 주인공 몸 가운데, `monsters`는 독에 걸린 몬스터 수.
    PoisonSprayed { at: Vec2, monsters: u32 },
    /// 06:00가 되어 스케이트가 벗겨졌다.
    SkatesOff,
    /// 갑옷 시간이 끝났다.
    ArmorOff,
}

/// 상점 화면과 산 물건. 새 게임을 하면 새로 만든다 (저장은 M9).
#[derive(Debug, Clone, Default)]
pub struct Shop {
    is_open: bool,
    health_potions: u32,
    poison_potions: u32,
    skates_on: bool,
    armor: Option<Armor>,
    armor_seconds_left: f32,
}

impl Shop {
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// 가방에 있는 포션 개수.
    pub fn potions(&self, potion: Potion) -> u32 {
        match potion {
            Potion::Health => self.health_potions,
            Potion::Poison => self.poison_potions,
        }
    }

    pub fn add_potion(&mut self, potion: Potion) {
        *self.potion_count(potion) += 1;
    }

    /// 포션 하나를 쓴다. 없으면 그대로다.
    pub fn take_potion(&mut self, potion: Potion) {
        let count = self.potion_count(potion);
        *count = count.saturating_sub(1);
    }

    fn potion_count(&mut self, potion: Potion) -> &mut u32 {
        match potion {
            Potion::Health => &mut self.health_potions,
            Potion::Poison => &mut self.poison_potions,
        }
    }

    pub fn has_skates(&self) -> bool {
        self.skates_on
    }

    pub fn put_on_skates(&mut self) {
        self.skates_on = true;
    }

    /// 06:00에 부른다. 신고 있었으면 벗고 true.
    pub fn take_off_skates(&mut self) -> bool {
        std::mem::replace(&mut self.skates_on, false)
    }

    /// 걷는 빠르기에 곱하는 수: 스케이트를 신으면 `SKATES_SPEED_MULTIPLIER`, 아니면 1.
    pub fn walk_speed_multiplier(&self) -> f32 {
        if self.skates_on {
            SKATES_SPEED_MULTIPLIER
        } else {
            1.0
        }
    }

    /// 입고 있는 갑옷.
    pub fn armor(&self) -> Option<Armor> {
        self.armor
    }

    /// 갑옷 남은 시간 (게임 속 초). 안 입었으면 0이다.
    pub fn armor_seconds_left(&self) -> f32 {
        self.armor_seconds_left
    }

    /// 갑옷을 입는다. 입고 있었으면 새 갑옷으로 바뀌고 시간도 새 갑옷의 시간으로 다시 시작한다.
    pub fn put_on_armor(&mut self, armor: Armor) {
        self.armor = Some(armor);
        self.armor_seconds_left = armor.seconds();
    }

    /// 갑옷 시간을 게임 속 `game_dt`초 흘린다. 이번에 끝났으면 true.
    pub fn wear_armor(&mut self, game_dt: f32) -> bool {
        if self.armor.is_none() {
            return false;
        }
        self.armor_seconds_left = (self.armor_seconds_left - game_dt.max(0.0)).max(0.0);
        if self.armor_seconds_left > 0.0 {
            return false;
        }
        self.armor = None;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prices_are_what_the_child_chose() {
        let prices: Vec<u32> = ShopItem::ALL.iter().map(|item| item.price()).collect();
        assert_eq!(prices, vec![100, 100, 200, 500, 1000], "칼·도끼·총은 팔지 않는다");
    }

    #[test]
    fn iron_and_wood_are_not_for_sale() {
        assert_eq!(sell_price(Material::Iron), None);
        assert_eq!(sell_price(Material::Wood), None);
        assert_eq!(SELLABLE.map(sell_price), [Some(1000), Some(500), Some(200)]);
    }

    #[test]
    fn armor_runs_out_and_new_armor_restarts_the_time() {
        let mut shop = Shop::default();
        shop.put_on_armor(Armor::Diamond);
        assert!(!shop.wear_armor(DIAMOND_ARMOR_SECONDS - 1.0));
        shop.put_on_armor(Armor::Rainbow);
        assert_eq!(shop.armor_seconds_left(), RAINBOW_ARMOR_SECONDS, "새 갑옷의 시간으로 다시 시작");
        assert!(!shop.wear_armor(RAINBOW_ARMOR_SECONDS - 0.5));
        assert!(shop.wear_armor(1.0), "끝난 순간 한 번 true");
        assert_eq!(shop.armor(), None);
        assert!(!shop.wear_armor(1.0));
    }
}
