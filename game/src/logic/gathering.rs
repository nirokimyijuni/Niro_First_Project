//! 캐기 (US-022, US-020): 맵의 나무와 동굴의 광석 덩이, 캔 재료를 담는 가방.
//! 무엇을 캘지(가장 가까운 하나)와 언제 멈추는지는 `Game`이 정하고, 여기는 시간과 재료를 맡는다.
//! 닿는지는 `is_reached`로 보아서 고리 맵 이음매 너머도 닿는다 (임시-21).

use super::combat::is_reached;
use super::monster::SeededRandom;
use super::player::Facing;
use super::world::World;
use crate::config::{
    COPPER_MINED_TEXT, COPPER_NAME, GOLD_MINED_TEXT, GOLD_NAME, IRON_MINED_TEXT, IRON_NAME, MINE_REACH, MINE_SECONDS_BARE_HAND,
    MINE_SECONDS_PICKAXE, ORE_COPPER_CHANCE, ORE_IRON_CHANCE, ORE_LUMP_HEIGHT, ORE_LUMP_WIDTH, ORE_LUMP_XS, ORE_REGROW_SECONDS,
    ORE_SILVER_CHANCE, SILVER_MINED_TEXT, SILVER_NAME, TREE_REGROW_SECONDS, WOOD_MINED_TEXT,
    WOOD_NAME,
};
use macroquad::math::{Rect, Vec2};

/// 캐서 얻는 재료. 철광석 1개는 곡괭이 재료인 철 블럭 1개로 친다 (니로 해석).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Material {
    Wood,
    Iron,
    Copper,
    Silver,
    Gold,
}

impl Material {
    /// 정보판에 보여 주는 순서.
    pub const ALL: [Material; 5] = [Material::Wood, Material::Iron, Material::Copper, Material::Silver, Material::Gold];

    pub fn name(self) -> &'static str {
        match self {
            Material::Wood => WOOD_NAME,
            Material::Iron => IRON_NAME,
            Material::Copper => COPPER_NAME,
            Material::Silver => SILVER_NAME,
            Material::Gold => GOLD_NAME,
        }
    }

    /// 다 캤을 때 팝업 문구 (US-065).
    pub fn mined_text(self) -> &'static str {
        match self {
            Material::Wood => WOOD_MINED_TEXT,
            Material::Iron => IRON_MINED_TEXT,
            Material::Copper => COPPER_MINED_TEXT,
            Material::Silver => SILVER_MINED_TEXT,
            Material::Gold => GOLD_MINED_TEXT,
        }
    }
}

/// `roll`(0 이상 1 미만)로 광석을 고른다 (임시-15). 남는 몫은 금이다.
fn ore_for_roll(roll: f32) -> Material {
    if roll < ORE_IRON_CHANCE {
        Material::Iron
    } else if roll < ORE_IRON_CHANCE + ORE_COPPER_CHANCE {
        Material::Copper
    } else if roll < ORE_IRON_CHANCE + ORE_COPPER_CHANCE + ORE_SILVER_CHANCE {
        Material::Silver
    } else {
        Material::Gold
    }
}

/// 가진 재료와 곡괭이. 새 게임을 하면 새로 만든다 (저장은 M9).
#[derive(Debug, Clone, Default)]
pub struct Bag {
    counts: [u32; Material::ALL.len()],
    pub has_pickaxe: bool,
}

impl Bag {
    pub fn count(&self, material: Material) -> u32 {
        self.counts[material as usize]
    }

    pub fn add(&mut self, material: Material) {
        self.counts[material as usize] += 1;
    }

    /// 재료 하나를 쓴다. 없으면 그대로다.
    pub fn take_one(&mut self, material: Material) {
        self.counts[material as usize] = self.counts[material as usize].saturating_sub(1);
    }

    /// 하나 캐는 데 걸리는 시간 (D5).
    pub fn mine_seconds(&self) -> f32 {
        if self.has_pickaxe {
            MINE_SECONDS_PICKAXE
        } else {
            MINE_SECONDS_BARE_HAND
        }
    }
}

/// 나무 한 그루나 광석 덩이 하나.
pub struct Mineable {
    /// 차지하는 네모 (왼쪽 위 기준). 캐릭터를 막지 않는다.
    pub rect: Rect,
    /// 다시 생길 때까지 남은 초. 0이면 캘 수 있다.
    seconds_until_back: f32,
}

impl Mineable {
    fn at(rect: Rect) -> Mineable {
        Mineable { rect, seconds_until_back: 0.0 }
    }

    /// 지금 캘 수 있으면 true. 아니면 나무는 그루터기로, 덩이는 안 보이게 그린다.
    pub fn is_ready(&self) -> bool {
        self.seconds_until_back <= 0.0
    }

    fn count_down(&mut self, dt: f32) {
        self.seconds_until_back = (self.seconds_until_back - dt.max(0.0)).max(0.0);
    }
}

struct MiningJob {
    /// 캐는 것의 번호 (`trees` 또는 `ore_lumps` 순서).
    index: usize,
    seconds: f32,
}

pub struct Gathering {
    pub trees: Vec<Mineable>,
    pub ore_lumps: Vec<Mineable>,
    pub bag: Bag,
    job: Option<MiningJob>,
    random: SeededRandom,
}

impl Gathering {
    /// 나무는 긴 맵에만 있다. `seed`가 같으면 나오는 광석 순서가 늘 같다.
    pub fn on_world(world: &World, cave: &World, seed: u32) -> Gathering {
        Gathering {
            trees: world.trees().into_iter().map(Mineable::at).collect(),
            ore_lumps: ORE_LUMP_XS.iter().map(|x| Mineable::at(cave.thing_on_ground(*x, ORE_LUMP_WIDTH, ORE_LUMP_HEIGHT))).collect(),
            bag: Bag::default(),
            job: None,
            // 맵·싸움·미스·무기와 같은 seed를 쓰므로 섞어야 서로 따라가지 않는다.
            random: SeededRandom::new(seed.wrapping_mul(61).wrapping_add(29)),
        }
    }

    /// 나무와 덩이의 시간을 흘린다. 동굴 밖에 있어도 덩이 시간은 간다.
    pub fn grow_back(&mut self, dt: f32) {
        self.trees.iter_mut().chain(&mut self.ore_lumps).for_each(|mineable| mineable.count_down(dt));
    }

    /// 동굴 안이면 광석 덩이, 밖이면 나무.
    pub fn list(&self, in_cave: bool) -> &[Mineable] {
        if in_cave {
            &self.ore_lumps
        } else {
            &self.trees
        }
    }

    pub fn start_mining(&mut self, index: usize) {
        self.job = Some(MiningJob { index, seconds: 0.0 });
    }

    pub fn stop_mining(&mut self) {
        self.job = None;
    }

    /// 캐는 중이면 0~1. 머리 위 표시에 쓴다. 캐지 않으면 None이다.
    pub fn progress(&self) -> Option<f32> {
        self.job.as_ref().map(|job| (job.seconds / self.bag.mine_seconds()).min(1.0))
    }

    /// `dt`초 캔다. 닿지 않게 되면 멈춘다. 다 캐면 재료를 가방에 넣고 무엇이 어디서 나왔는지 돌려준다.
    pub fn keep_mining(&mut self, dt: f32, in_cave: bool, body: Rect, facing: Facing) -> Option<(Material, Vec2)> {
        let needed = self.bag.mine_seconds();
        let job = self.job.as_mut()?;
        job.seconds += dt.max(0.0);
        let (index, is_done) = (job.index, job.seconds >= needed);
        let target = self.list(in_cave)[index].rect;
        if !is_reached(body, facing, MINE_REACH, target) {
            self.job = None;
            return None;
        }
        if !is_done {
            return None;
        }
        self.job = None;
        let material = if in_cave {
            self.ore_lumps[index].seconds_until_back = ORE_REGROW_SECONDS;
            ore_for_roll(self.random.next_unit())
        } else {
            self.trees[index].seconds_until_back = TREE_REGROW_SECONDS;
            Material::Wood
        };
        self.bag.add(material);
        Some((material, target.center()))
    }
}
