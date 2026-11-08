//! 제작대 (US-021): 4×3 칸에 재료를 놓아 곡괭이를 만든다.
//! 칸에 놓는 동안 가방은 줄지 않고, 곡괭이를 꺼낼 때 놓인 만큼 줄어든다.

use super::gathering::{Bag, Material};
use crate::config::{CRAFT_GRID_HEIGHT, CRAFT_GRID_WIDTH, CRAFT_IRON_MARK, CRAFT_WOOD_MARK, PICKAXE_SHAPE};

#[derive(Default)]
pub struct Crafting {
    is_open: bool,
    /// `grid[줄][칸]`. 왼쪽 위가 [0][0]이다.
    pub grid: [[Option<Material>; CRAFT_GRID_WIDTH]; CRAFT_GRID_HEIGHT],
}

fn mark_of(cell: Option<Material>) -> char {
    match cell {
        Some(Material::Wood) => CRAFT_WOOD_MARK,
        Some(Material::Iron) => CRAFT_IRON_MARK,
        Some(_) => '?',
        None => ' ',
    }
}

impl Crafting {
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// 닫는다. 칸은 비고 재료는 그대로다.
    pub fn close(&mut self) {
        *self = Crafting::default();
    }

    fn placed(&self, material: Material) -> u32 {
        self.grid.iter().flatten().filter(|cell| **cell == Some(material)).count() as u32
    }

    /// 칸을 눌렀다: 놓인 칸은 빠지고, 빈 칸에는 가방에 남은 만큼만 `chosen`이 놓인다.
    pub fn tap_cell(&mut self, row: usize, column: usize, chosen: Material, bag: &Bag) {
        let can_place = self.placed(chosen) < bag.count(chosen);
        let cell = &mut self.grid[row][column];
        if cell.is_some() {
            *cell = None;
        } else if can_place {
            *cell = Some(chosen);
        }
    }

    /// 결과 칸에 곡괭이가 나오는지. 이미 가졌으면 안 나온다 (임시-19).
    pub fn pickaxe_is_ready(&self, bag: &Bag) -> bool {
        let same_shape = self.grid.iter().flatten().map(|cell| mark_of(*cell)).eq(PICKAXE_SHAPE.iter().flatten().copied());
        same_shape && !bag.has_pickaxe
    }

    /// 결과 칸을 눌렀다: 곡괭이를 얻고 놓인 재료가 줄고 칸이 빈다. 나온 곡괭이가 없으면 false다.
    pub fn take_pickaxe(&mut self, bag: &mut Bag) -> bool {
        if !self.pickaxe_is_ready(bag) {
            return false;
        }
        self.grid.iter().flatten().flatten().for_each(|material| bag.take_one(*material));
        bag.has_pickaxe = true;
        self.grid = Default::default();
        true
    }
}
