//! 제작대 (US-021, US-078): 4×3 칸에 재료를 놓아 곡괭이, 자동차, 엔더열매, 다이아래더라이트 곡괭이, 주사기총을 만든다.
//! 칸에 놓는 동안 가방은 줄지 않고, 결과를 꺼낼 때 놓인 만큼 줄어든다.
//! 위쪽 그림 버튼으로 "만들 것"을 고르면 그 모양이 흐린 칸으로 보인다 (임시-39).
//! 결과 칸은 고른 것과 상관없이 판과 칸마다 똑같은 조합을 보여 준다.

use super::gathering::{Bag, Material};
use crate::config::{
    CAR_NAME, CAR_ON_TEXT, CAR_SHAPE, CRAFT_ALREADY_HAVE_TEXT, CRAFT_GRID_HEIGHT, CRAFT_GRID_WIDTH, CRAFT_NEED_MORE_TEXT,
    DIAMOND_PICKAXE_CRAFTED_TEXT, DIAMOND_PICKAXE_NAME, DIAMOND_PICKAXE_SHAPE, ENDER_FRUIT_CRAFTED_TEXT, ENDER_FRUIT_NAME,
    ENDER_FRUIT_SHAPE, PICKAXE_CRAFTED_TEXT, PICKAXE_NAME, PICKAXE_SHAPE, SYRINGE_GUN_CRAFTED_TEXT, SYRINGE_GUN_NAME,
    SYRINGE_GUN_SHAPE,
};

/// 조합 모양: `[줄][칸]`의 표시 글자. ' '는 빈칸이다.
pub type Shape = [[char; CRAFT_GRID_WIDTH]; CRAFT_GRID_HEIGHT];

/// 제작대에서 만들 수 있는 것 (D74).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Recipe {
    #[default]
    Pickaxe,
    Car,
    EnderFruit,
    DiamondPickaxe,
    SyringeGun,
}

impl Recipe {
    /// 제작대 위쪽 그림 버튼 순서 (임시-39).
    pub const ALL: [Recipe; 5] = [Recipe::Pickaxe, Recipe::Car, Recipe::EnderFruit, Recipe::DiamondPickaxe, Recipe::SyringeGun];

    pub fn name(self) -> &'static str {
        match self {
            Recipe::Pickaxe => PICKAXE_NAME,
            Recipe::Car => CAR_NAME,
            Recipe::EnderFruit => ENDER_FRUIT_NAME,
            Recipe::DiamondPickaxe => DIAMOND_PICKAXE_NAME,
            Recipe::SyringeGun => SYRINGE_GUN_NAME,
        }
    }

    /// 조합 모양 (`config.rs`).
    pub fn shape(self) -> &'static Shape {
        match self {
            Recipe::Pickaxe => &PICKAXE_SHAPE,
            Recipe::Car => &CAR_SHAPE,
            Recipe::EnderFruit => &ENDER_FRUIT_SHAPE,
            Recipe::DiamondPickaxe => &DIAMOND_PICKAXE_SHAPE,
            Recipe::SyringeGun => &SYRINGE_GUN_SHAPE,
        }
    }

    /// 꺼냈을 때 나오는 문구. 자동차는 바로 타서 "자동차를 탔어요! 슝!"이다.
    pub fn crafted_text(self) -> &'static str {
        match self {
            Recipe::Pickaxe => PICKAXE_CRAFTED_TEXT,
            Recipe::Car => CAR_ON_TEXT,
            Recipe::EnderFruit => ENDER_FRUIT_CRAFTED_TEXT,
            Recipe::DiamondPickaxe => DIAMOND_PICKAXE_CRAFTED_TEXT,
            Recipe::SyringeGun => SYRINGE_GUN_CRAFTED_TEXT,
        }
    }

    /// 칸 하나에 놓아야 하는 재료. 빈칸이면 None이다. 흐린 칸 힌트에 쓴다.
    pub fn material_at(self, row: usize, column: usize) -> Option<Material> {
        Material::from_craft_mark(self.shape()[row][column])
    }

    /// 한 개 만드는 데 드는 (재료, 개수). `Material::ALL` 순서이고 0개인 재료는 없다.
    pub fn needs(self) -> Vec<(Material, u32)> {
        Material::ALL
            .into_iter()
            .map(|material| {
                let count = self.shape().iter().flatten().filter(|mark| **mark == material.craft_mark()).count() as u32;
                (material, count)
            })
            .filter(|(_, count)| *count > 0)
            .collect()
    }

    /// 이미 가지고 있어서 더 만들 수 없으면 true (임시-19, 임시-41, 임시-45, 임시-46). 엔더열매와 주사기총은 여러 개 가진다.
    pub fn already_have(self, bag: &Bag) -> bool {
        match self {
            Recipe::Pickaxe => bag.has_pickaxe,
            Recipe::Car => bag.riding_car,
            Recipe::EnderFruit => false,
            Recipe::DiamondPickaxe => bag.has_diamond_pickaxe,
            Recipe::SyringeGun => false,
        }
    }

    /// 가방에 모자란 재료 (재료, 더 필요한 개수). 다 있으면 비어 있다.
    pub fn missing(self, bag: &Bag) -> Vec<(Material, u32)> {
        self.needs()
            .into_iter()
            .filter(|(material, need)| bag.count(*material) < *need)
            .map(|(material, need)| (material, need - bag.count(material)))
            .collect()
    }

    /// 지금 이것을 못 만드는 까닭 문구: "벌써 가지고 있어요" 또는 "에메랄드가 2개 더 필요해요". 만들 수 있으면 None이다.
    pub fn why_cant_make_text(self, bag: &Bag) -> Option<String> {
        if self.already_have(bag) {
            return Some(CRAFT_ALREADY_HAVE_TEXT.to_string());
        }
        let missing = self.missing(bag);
        if missing.is_empty() {
            return None;
        }
        let parts: Vec<String> =
            missing.iter().map(|(material, more)| format!("{}{} {}개", material.name(), subject_particle(material.name()), more)).collect();
        Some(CRAFT_NEED_MORE_TEXT.replace("{모자란 것}", &parts.join(", ")))
    }

    /// 가방에 넣는다 (자동차는 바로 탄다).
    fn give_to(self, bag: &mut Bag) {
        match self {
            Recipe::Pickaxe => bag.has_pickaxe = true,
            Recipe::Car => bag.riding_car = true,
            Recipe::EnderFruit => bag.ender_fruits += 1,
            Recipe::DiamondPickaxe => bag.has_diamond_pickaxe = true,
            Recipe::SyringeGun => bag.syringe_guns += 1,
        }
    }
}

/// 이름 뒤에 붙는 "이"/"가": 마지막 글자에 받침이 있으면 "이" (철광석이, 동이), 없으면 "가" (나무가, 루비가).
fn subject_particle(name: &str) -> &'static str {
    let has_final_consonant = name.chars().last().is_some_and(|c| ('가'..='힣').contains(&c) && (c as u32 - '가' as u32) % 28 != 0);
    if has_final_consonant {
        "이"
    } else {
        "가"
    }
}

#[derive(Default)]
pub struct Crafting {
    is_open: bool,
    /// `grid[줄][칸]`. 왼쪽 위가 [0][0]이다.
    pub grid: [[Option<Material>; CRAFT_GRID_WIDTH]; CRAFT_GRID_HEIGHT],
    /// 위쪽 그림 버튼에서 고른 만들 것 (임시-39). 처음은 곡괭이, 닫았다 열어도 그대로다.
    chosen: Recipe,
}

impl Crafting {
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// 닫는다. 칸은 비고 재료는 그대로다. 고른 만들 것은 남는다 (임시-39).
    pub fn close(&mut self) {
        self.is_open = false;
        self.grid = Default::default();
    }

    /// 지금 골라진 만들 것. 테두리를 그리고 흐린 칸을 고를 때 쓴다.
    pub fn chosen(&self) -> Recipe {
        self.chosen
    }

    /// 그림 버튼을 눌렀다. 판에 놓인 재료는 그대로다.
    pub fn choose(&mut self, recipe: Recipe) {
        self.chosen = recipe;
    }

    /// 칸 하나의 흐린 칸 힌트: 골라진 것의 모양에서 이 칸 재료 (빈칸이면 None).
    pub fn hint_at(&self, row: usize, column: usize) -> Option<Material> {
        self.chosen.material_at(row, column)
    }

    fn placed(&self, material: Material) -> u32 {
        self.grid.iter().flatten().filter(|cell| **cell == Some(material)).count() as u32
    }

    /// 칸을 눌렀다: 놓인 칸은 빠지고, 빈 칸에는 가방에 남은 만큼만 `chosen`이 놓인다.
    /// 가진 수가 0인 재료는 놓이지 않는다 (임시-40).
    pub fn tap_cell(&mut self, row: usize, column: usize, chosen: Material, bag: &Bag) {
        let can_place = self.placed(chosen) < bag.count(chosen);
        let cell = &mut self.grid[row][column];
        if cell.is_some() {
            *cell = None;
        } else if can_place {
            *cell = Some(chosen);
        }
    }

    /// 판이 칸마다 똑같은 조합. 하나라도 다르거나 4번째 칸에 무엇이 있으면 None이다.
    pub fn matching_recipe(&self) -> Option<Recipe> {
        let marks = || self.grid.iter().flatten().map(|cell| cell.map_or(' ', Material::craft_mark));
        Recipe::ALL.into_iter().find(|recipe| marks().eq(recipe.shape().iter().flatten().copied()))
    }

    /// 결과 칸에 나오는 것: 판이 맞고, 이미 가진 것이 아니면 (고른 버튼과 달라도 나온다).
    pub fn ready_recipe(&self, bag: &Bag) -> Option<Recipe> {
        self.matching_recipe().filter(|recipe| !recipe.already_have(bag))
    }

    /// 결과 칸을 눌렀다: 그것을 얻고 놓인 재료가 줄고 판이 빈다. 나온 것이 없으면 None이다.
    /// 문구는 `Recipe::crafted_text`. 만든 것은 수학 문제 "아이템"으로 세지 않는다 (D74).
    pub fn take_result(&mut self, bag: &mut Bag) -> Option<Recipe> {
        let recipe = self.ready_recipe(bag)?;
        self.grid.iter().flatten().flatten().for_each(|material| bag.take_one(*material));
        recipe.give_to(bag);
        self.grid = Default::default();
        Some(recipe)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn particle_follows_the_last_letter() {
        assert_eq!(subject_particle("에메랄드"), "가");
        assert_eq!(subject_particle("철광석"), "이");
        assert_eq!(subject_particle("동"), "이");
        assert_eq!(subject_particle("나무"), "가");
    }

    #[test]
    fn recipe_needs_match_the_plan() {
        let n = |r: Recipe| r.needs();
        assert_eq!(n(Recipe::Car), vec![(Material::Wood, 3), (Material::Iron, 4)]);
        assert_eq!(n(Recipe::EnderFruit), vec![(Material::Copper, 9)]);
        assert_eq!(n(Recipe::DiamondPickaxe), vec![(Material::Wood, 3), (Material::Iron, 2), (Material::Diamond, 1)]);
        assert_eq!(
            n(Recipe::SyringeGun),
            vec![(Material::Wood, 2), (Material::Silver, 3), (Material::Ruby, 1), (Material::Emerald, 3)]
        );
    }

    #[test]
    fn fourth_column_is_empty_in_every_recipe() {
        for recipe in Recipe::ALL {
            assert!(recipe.shape().iter().all(|line| line[CRAFT_GRID_WIDTH - 1] == ' '), "{recipe:?}");
        }
    }

    #[test]
    fn need_more_text_names_what_is_missing() {
        let mut bag = Bag::default();
        for m in [Material::Silver, Material::Silver, Material::Silver, Material::Wood, Material::Wood, Material::Emerald, Material::Ruby] {
            bag.add(m);
        }
        assert_eq!(Recipe::SyringeGun.why_cant_make_text(&bag).as_deref(), Some("에메랄드가 2개 더 필요해요"));
    }
}
