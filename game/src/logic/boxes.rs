//! 상자 부수기 (D87, US-089): 도끼로 상자를 3번 치면 부서지고, 가끔 깜짝 아이템이나 몬스터가 나온다.
//! 부서진 상자는 다음 06:00에 같은 자리로 돌아온다. 그 자리에 누가 서 있으면 비킬 때까지 기다린다.
//!
//! 여기는 상자의 체력, 주사위, 땅에 떨어진 깜짝 아이템을 맡는다.
//! 맵에서 상자를 빼고 넣는 일, 코인·무기·몬스터와 함께 하는 일은 `Game`이 한다.

use super::combat::attack_box_with_range;
use super::monster::SeededRandom;
use super::player::Facing;
use super::world::{really_overlaps, shortest_dx, World};
use crate::config::{
    BOX_GOT_AXE_TEXT, BOX_GOT_CAR_TEXT, BOX_GOT_DIAMOND_ARMOR_TEXT, BOX_GOT_DIAMOND_PICKAXE_TEXT, BOX_GOT_ENDER_FRUIT_TEXT,
    BOX_GOT_GUN_TEXT, BOX_GOT_HEALTH_POTION_TEXT, BOX_GOT_POISON_POTION_TEXT, BOX_GOT_RAINBOW_ARMOR_TEXT, BOX_GOT_SKATES_TEXT,
    BOX_GOT_SWORD_TEXT, BOX_GOT_SYRINGE_GUN_TEXT, BOX_HITS_TO_BREAK, BOX_ITEM_CHANCE, BOX_ITEM_FALL_SPEED, BOX_ITEM_GROUND_SECONDS,
    BOX_ITEM_GUN_SHARE, BOX_ITEM_SIZE, BOX_ITEM_VERY_STRONG_SHARE, BOX_MONSTER_CHANCE, BOX_SKELETON_CHANCE,
};
use macroquad::math::{Rect, Vec2};

/// 상자에서 나올 수 있는 깜짝 아이템 12가지 ("모든 아이템 중 하나", 아이).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurpriseItem {
    Sword,
    Axe,
    Gun,
    HealthPotion,
    PoisonPotion,
    Skates,
    RainbowArmor,
    DiamondArmor,
    SyringeGun,
    EnderFruit,
    Car,
    DiamondPickaxe,
}

impl SurpriseItem {
    pub const ALL: [SurpriseItem; 12] = [
        SurpriseItem::Sword,
        SurpriseItem::Axe,
        SurpriseItem::Gun,
        SurpriseItem::HealthPotion,
        SurpriseItem::PoisonPotion,
        SurpriseItem::Skates,
        SurpriseItem::RainbowArmor,
        SurpriseItem::DiamondArmor,
        SurpriseItem::SyringeGun,
        SurpriseItem::EnderFruit,
        SurpriseItem::Car,
        SurpriseItem::DiamondPickaxe,
    ];

    /// 아주 센 것이면 true. 나오면 캐릭터가 "와우!" 한다.
    pub fn is_very_strong(self) -> bool {
        matches!(self, SurpriseItem::DiamondArmor | SurpriseItem::DiamondPickaxe | SurpriseItem::EnderFruit | SurpriseItem::Car)
    }

    /// 아무것도 갖지 않았을 때 이 아이템이 뽑힐 몫 (12개를 다 더하면 1).
    /// 총과 아주 센 것은 config 값이고, 나머지 보통 아이템이 남은 몫을 똑같이 나눈다.
    pub fn share(self) -> f32 {
        let very_strong_count = SurpriseItem::ALL.iter().filter(|i| i.is_very_strong()).count() as f32;
        let normal_count = SurpriseItem::ALL.len() as f32 - very_strong_count - 1.0;
        if self == SurpriseItem::Gun {
            BOX_ITEM_GUN_SHARE
        } else if self.is_very_strong() {
            BOX_ITEM_VERY_STRONG_SHARE
        } else {
            (1.0 - BOX_ITEM_GUN_SHARE - BOX_ITEM_VERY_STRONG_SHARE * very_strong_count) / normal_count
        }
    }

    /// 주웠을 때 문구 ("○○을 얻었어요!").
    pub fn picked_text(self) -> &'static str {
        match self {
            SurpriseItem::Sword => BOX_GOT_SWORD_TEXT,
            SurpriseItem::Axe => BOX_GOT_AXE_TEXT,
            SurpriseItem::Gun => BOX_GOT_GUN_TEXT,
            SurpriseItem::HealthPotion => BOX_GOT_HEALTH_POTION_TEXT,
            SurpriseItem::PoisonPotion => BOX_GOT_POISON_POTION_TEXT,
            SurpriseItem::Skates => BOX_GOT_SKATES_TEXT,
            SurpriseItem::RainbowArmor => BOX_GOT_RAINBOW_ARMOR_TEXT,
            SurpriseItem::DiamondArmor => BOX_GOT_DIAMOND_ARMOR_TEXT,
            SurpriseItem::SyringeGun => BOX_GOT_SYRINGE_GUN_TEXT,
            SurpriseItem::EnderFruit => BOX_GOT_ENDER_FRUIT_TEXT,
            SurpriseItem::Car => BOX_GOT_CAR_TEXT,
            SurpriseItem::DiamondPickaxe => BOX_GOT_DIAMOND_PICKAXE_TEXT,
        }
    }
}

/// 상자가 부서질 때 나오는 것. 아이템과 몬스터가 함께 나오는 일은 없다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Surprise {
    Item,
    Monster,
    Nothing,
}

/// 상자에서 나오는 몬스터.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxMonster {
    Zombie,
    Skeleton,
}

/// 상자 주사위. 시드가 같으면 늘 같은 순서로 나온다.
pub struct SurpriseDice {
    random: SeededRandom,
}

impl SurpriseDice {
    pub fn new(seed: u32) -> SurpriseDice {
        // 맵·싸움·무기와 같은 seed를 쓰므로 섞어야 서로 따라가지 않는다.
        SurpriseDice { random: SeededRandom::new(seed.wrapping_mul(71).wrapping_add(0xB0C5)) }
    }

    /// 아이템 `BOX_ITEM_CHANCE`, 몬스터 `BOX_MONSTER_CHANCE`, 나머지는 아무것도 없음.
    pub fn roll_surprise(&mut self) -> Surprise {
        let roll = self.random.next_unit();
        if roll < BOX_ITEM_CHANCE {
            Surprise::Item
        } else if roll < BOX_ITEM_CHANCE + BOX_MONSTER_CHANCE {
            Surprise::Monster
        } else {
            Surprise::Nothing
        }
    }

    /// 깜짝 아이템 하나. `already_have`에 든 것(이미 가진 하나뿐인 것)은 빼고, 남은 것끼리 원래 비율대로 뽑는다.
    pub fn roll_item(&mut self, already_have: &[SurpriseItem]) -> SurpriseItem {
        let choices: Vec<SurpriseItem> = SurpriseItem::ALL.into_iter().filter(|i| !already_have.contains(i)).collect();
        let total: f32 = choices.iter().map(|i| i.share()).sum();
        let roll = self.random.next_unit() * total;
        let mut sum = 0.0;
        for item in &choices {
            sum += item.share();
            if roll < sum {
                return *item;
            }
        }
        // 더하기 반올림으로 끝까지 온 경우. 포션은 늘 후보라서 비지 않는다.
        choices.last().copied().unwrap_or(SurpriseItem::HealthPotion)
    }

    /// 좀비나 스켈레톤 (스켈레톤 `BOX_SKELETON_CHANCE`).
    pub fn roll_monster(&mut self) -> BoxMonster {
        if self.random.next_unit() < BOX_SKELETON_CHANCE {
            BoxMonster::Skeleton
        } else {
            BoxMonster::Zombie
        }
    }
}

/// 상자가 지금 어떤지.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoxState {
    Standing,
    /// 부서져서 없다. 다음 06:00을 기다린다.
    Broken,
    /// 06:00가 지났다. 자리가 비면 바로 돌아온다.
    WaitingForSpace,
}

/// 부술 수 있는 나무 상자 하나 (D36). 나무 발판과 바위 계단은 부서지지 않는다.
pub struct BreakableBox {
    /// 상자 자리와 크기. 돌아올 때도 같다.
    pub rect: Rect,
    hits_taken: u32,
    state: BoxState,
}

impl BreakableBox {
    /// 맵에 서 있으면 true. 부서진 동안은 모든 규칙에서 없는 것으로 친다.
    pub fn is_standing(&self) -> bool {
        self.state == BoxState::Standing
    }

    /// 금 간 정도: 0이면 새것, 1이면 금이 조금, 2면 금이 많이.
    pub fn cracks(&self) -> u32 {
        self.hits_taken
    }
}

/// 상자에서 나와 떨어지는 중이거나 땅에 놓인 깜짝 아이템.
pub struct DroppedItem {
    pub item: SurpriseItem,
    /// 아이템 네모 (왼쪽 위 기준).
    pub rect: Rect,
    landing_y: f32,
    seconds_left: f32,
}

impl DroppedItem {
    /// 부서진 상자 `box_rect` 가운데에서 떨어지기 시작한다. 상자는 이미 맵에서 빠져 있어야 한다.
    fn from_box(item: SurpriseItem, box_rect: Rect, world: &World) -> DroppedItem {
        let center = box_rect.center();
        let rect = Rect::new(center.x - BOX_ITEM_SIZE / 2.0, center.y - BOX_ITEM_SIZE / 2.0, BOX_ITEM_SIZE, BOX_ITEM_SIZE);
        let mut dropped = DroppedItem { item, rect, landing_y: rect.y, seconds_left: BOX_ITEM_GROUND_SECONDS };
        dropped.land_again(world);
        dropped
    }

    pub fn is_falling(&self) -> bool {
        self.rect.y < self.landing_y
    }

    /// 사라질 때까지 남은 초. 땅에 닿은 때부터 줄어든다.
    pub fn seconds_left(&self) -> f32 {
        self.seconds_left
    }

    /// 맵이 바뀌었을 때(상자가 부서지거나 돌아올 때) 멈출 곳을 다시 정한다.
    /// 밑의 상자가 없어졌으면 다시 떨어지고, 상자가 돌아오면 그 위로 올라간다.
    pub fn land_again(&mut self, world: &World) {
        self.landing_y = world.weapon_floor_y_under(self.rect) - self.rect.h;
        self.rect.y = self.rect.y.min(self.landing_y);
    }

    fn update(&mut self, dt: f32) {
        if self.is_falling() {
            self.rect.y = (self.rect.y + BOX_ITEM_FALL_SPEED * dt).min(self.landing_y);
        } else {
            self.seconds_left = (self.seconds_left - dt).max(0.0);
        }
    }
}

/// 상자에서 생긴 일. `Game::take_box_events`로 받는다. 부서질 때 글과 소리는 없다 (아이 답).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoxEvent {
    /// 도끼에 맞았고 아직 서 있다. `cracks`는 1(금이 조금) 또는 2(금이 많이). `at`은 상자 가운데.
    Cracked { index: usize, at: Vec2, cracks: u32 },
    /// 부서져 사라졌다. 나뭇조각이 흩어지는 모습만 (번쩍임·글·소리 없음).
    Broken { index: usize, at: Vec2 },
    /// 도끼가 아닌 것으로 상자를 쳤다 (`BOX_NEED_AXE_TEXT`). 한 판에 한 번만 온다.
    NeedAxe { at: Vec2 },
    /// 깜짝 아이템이 상자 자리에서 떨어지기 시작했다. `at`은 아이템 가운데.
    ItemDropped { item: SurpriseItem, at: Vec2 },
    /// 아주 센 것이 나왔다 (`BOX_WOW_TEXT`). `at`은 주인공 몸 가운데. 머리 위에 잠깐 보여 준다.
    Wow { at: Vec2 },
    /// 상자에서 몬스터가 나왔다 (`BOX_MONSTER_TEXT`). `at`은 몬스터 몸 가운데.
    /// 몬스터가 가만히 있는 동안(`Zombie::is_waiting`, `Skeleton::is_waiting`) 머리 위에 보여 줄 수 있다.
    MonsterJumpedOut { monster: BoxMonster, at: Vec2 },
    /// 깜짝 아이템을 주웠다 (`SurpriseItem::picked_text`).
    ItemPicked { item: SurpriseItem },
    /// 06:00 뒤 상자가 제자리로 돌아왔다.
    CameBack { index: usize },
}

/// 맵의 부술 수 있는 상자 전부와 상자에서 나온 깜짝 아이템. 새 게임을 하면 새로 만든다.
pub struct Boxes {
    /// 순서는 `World::boxes()`, `Coins::list`와 같다.
    pub list: Vec<BreakableBox>,
    pub items: Vec<DroppedItem>,
    pub dice: SurpriseDice,
    told_need_axe: bool,
}

impl Boxes {
    /// 맵의 진짜 상자(나무 발판·바위 계단 빼고)가 모두 새것으로 선다.
    pub fn on_world(world: &World, seed: u32) -> Boxes {
        let list = world.boxes().into_iter().map(|rect| BreakableBox { rect, hits_taken: 0, state: BoxState::Standing }).collect();
        Boxes { list, items: Vec::new(), dice: SurpriseDice::new(seed), told_need_axe: false }
    }

    /// 상자 `rect`가 금 간 정도 (0~2). 화면이 `world.obstacles`를 그릴 때 쓴다. 부술 수 없는 것은 0이다.
    pub fn cracks_at(&self, rect: Rect) -> u32 {
        self.list.iter().find(|b| b.is_standing() && b.rect == rect).map_or(0, |b| b.cracks())
    }

    /// 지금 공격하면 맞는 상자: 바라보는 쪽 `reach` 안에서 몸 높이에 걸린, 서 있는 상자 중 가장 가까운 하나.
    /// 발밑 상자는 맞지 않는다.
    pub fn box_in_reach(&self, body: Rect, facing: Facing, reach: f32) -> Option<usize> {
        let swing = attack_box_with_range(body, facing, reach);
        let dx = |rect: Rect| shortest_dx(body.center().x, rect.center().x);
        let in_front = |rect: Rect| match facing {
            Facing::Right => dx(rect) >= 0.0,
            Facing::Left => dx(rect) <= 0.0,
        };
        self.list
            .iter()
            .enumerate()
            .filter(|(_, b)| b.is_standing() && really_overlaps(swing, b.rect) && in_front(b.rect))
            .min_by(|(_, a), (_, b)| dx(a.rect).abs().total_cmp(&dx(b.rect).abs()))
            .map(|(index, _)| index)
    }

    /// 상자 `index`를 한 번 친다. 이번에 부서졌으면 true (맵에서 빼는 일은 `Game`이 한다).
    pub fn hit(&mut self, index: usize) -> bool {
        let the_box = &mut self.list[index];
        the_box.hits_taken += 1;
        if the_box.hits_taken < BOX_HITS_TO_BREAK {
            return false;
        }
        self.mark_broken(index);
        true
    }

    /// 상자 `index`가 부서진 것으로 둔다 (금은 없어진다). 이미 부서졌으면 그대로다.
    pub fn mark_broken(&mut self, index: usize) {
        self.list[index].hits_taken = 0;
        self.list[index].state = BoxState::Broken;
    }

    /// "도끼로 부술 수 있어요"를 보여 줄 차례면 true. 한 판에 한 번만 true다.
    pub fn take_need_axe_hint(&mut self) -> bool {
        !std::mem::replace(&mut self.told_need_axe, true)
    }

    /// 06:00: 부서진 상자는 돌아올 준비를 하고, 금 간 상자는 새것이 된다.
    pub fn morning_came(&mut self) {
        for the_box in &mut self.list {
            the_box.hits_taken = 0;
            if the_box.state == BoxState::Broken {
                the_box.state = BoxState::WaitingForSpace;
            }
        }
    }

    /// 06:00 뒤 자리가 비기를 기다리는 상자들의 번호.
    pub fn waiting_for_space(&self) -> Vec<usize> {
        (0..self.list.len()).filter(|i| self.list[*i].state == BoxState::WaitingForSpace).collect()
    }

    /// 상자 `index`가 다시 선다 (맵에 넣는 일은 `Game`이 한다).
    pub fn stand_again(&mut self, index: usize) {
        self.list[index].state = BoxState::Standing;
        self.list[index].hits_taken = 0;
    }

    /// 부서진 상자 자리에 깜짝 아이템을 떨어뜨린다. 상자는 이미 `world`에서 빠져 있어야 한다.
    pub fn drop_item(&mut self, item: SurpriseItem, box_rect: Rect, world: &World) -> Vec2 {
        let dropped = DroppedItem::from_box(item, box_rect, world);
        let at = dropped.rect.center();
        self.items.push(dropped);
        at
    }

    /// 아이템을 떨어뜨리고, 몸에 닿은 것을 줍고, 시간이 다 된 것을 없앤다. 돌려주는 값: 주운 아이템들.
    pub fn update_items(&mut self, dt: f32, body: Rect) -> Vec<SurpriseItem> {
        let mut picked = Vec::new();
        for dropped in &mut self.items {
            dropped.update(dt.max(0.0));
        }
        self.items.retain(|dropped| {
            let touched = really_overlaps(dropped.rect, body);
            if touched {
                picked.push(dropped.item);
            }
            !touched && dropped.seconds_left > 0.0
        });
        picked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shares_add_up_to_one() {
        let total: f32 = SurpriseItem::ALL.iter().map(|i| i.share()).sum();
        assert!((total - 1.0).abs() < 1e-5, "{total}");
        assert!((SurpriseItem::Sword.share() - 0.86 / 7.0).abs() < 1e-5);
    }

    #[test]
    fn three_hits_break_and_cracks_show() {
        let world = World::m1();
        let mut boxes = Boxes::on_world(&world, 1);
        assert!(!boxes.hit(0));
        assert_eq!(boxes.list[0].cracks(), 1);
        assert!(!boxes.hit(0));
        assert_eq!(boxes.cracks_at(boxes.list[0].rect), 2);
        assert!(boxes.hit(0));
        assert!(!boxes.list[0].is_standing());
    }

    #[test]
    fn need_axe_hint_only_once() {
        let mut boxes = Boxes::on_world(&World::m1(), 1);
        assert!(boxes.take_need_axe_hint());
        assert!(!boxes.take_need_axe_hint());
    }
}
