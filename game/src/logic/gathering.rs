//! 캐기 (US-022, US-020): 맵의 나무와 동굴의 광석 덩이, 캔 재료를 담는 가방.
//! 무엇을 캘지(가장 가까운 하나)와 언제 멈추는지는 `Game`이 정하고, 여기는 시간과 재료를 맡는다.
//! 닿는지는 `is_reached`로 보아서 고리 맵 이음매 너머도 닿는다 (임시-21).

use super::combat::is_reached_on_either_side;
use super::monster::SeededRandom;
use super::world::World;
use crate::config::{
    COPPER_MINED_TEXT, COPPER_NAME, CRAFT_COPPER_MARK, CRAFT_DIAMOND_MARK, CRAFT_EMERALD_MARK, CRAFT_GOLD_MARK, CRAFT_IRON_MARK,
    CRAFT_RUBY_MARK, CRAFT_SILVER_MARK, CRAFT_WOOD_MARK, DIAMOND_MINED_TEXT, DIAMOND_NAME, EMERALD_MINED_TEXT, EMERALD_NAME,
    GOLD_MINED_TEXT, GOLD_NAME, IRON_MINED_TEXT, IRON_NAME, MINE_REACH, MINE_SECONDS_BARE_HAND, MINE_SECONDS_DIAMOND_PICKAXE,
    MINE_SECONDS_PICKAXE, ORE_COPPER_CHANCE, ORE_DIAMOND_CHANCE, ORE_EMERALD_CHANCE, ORE_GOLD_CHANCE, ORE_IRON_CHANCE, ORE_LUMP_HEIGHT, ORE_LUMP_WIDTH,
    ORE_LUMP_XS, ORE_REGROW_SECONDS, ORE_RUBY_CHANCE, ORE_SILVER_CHANCE, RUBY_MINED_TEXT, RUBY_NAME, SILVER_MINED_TEXT, SILVER_NAME,
    TREE_REGROW_SECONDS, WOOD_MINED_TEXT, WOOD_NAME,
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
    /// D75 새 광물 (US-079)
    Ruby,
    Emerald,
    Diamond,
}

impl Material {
    /// 정보판에 보여 주는 순서.
    pub const ALL: [Material; 8] = [
        Material::Wood,
        Material::Iron,
        Material::Copper,
        Material::Silver,
        Material::Gold,
        Material::Ruby,
        Material::Emerald,
        Material::Diamond,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Material::Wood => WOOD_NAME,
            Material::Iron => IRON_NAME,
            Material::Copper => COPPER_NAME,
            Material::Silver => SILVER_NAME,
            Material::Gold => GOLD_NAME,
            Material::Ruby => RUBY_NAME,
            Material::Emerald => EMERALD_NAME,
            Material::Diamond => DIAMOND_NAME,
        }
    }

    /// 다 캤을 때 팝업 문구 (US-065, US-079).
    pub fn mined_text(self) -> &'static str {
        match self {
            Material::Wood => WOOD_MINED_TEXT,
            Material::Iron => IRON_MINED_TEXT,
            Material::Copper => COPPER_MINED_TEXT,
            Material::Silver => SILVER_MINED_TEXT,
            Material::Gold => GOLD_MINED_TEXT,
            Material::Ruby => RUBY_MINED_TEXT,
            Material::Emerald => EMERALD_MINED_TEXT,
            Material::Diamond => DIAMOND_MINED_TEXT,
        }
    }

    /// 귀한 광물이면 true: 팝업에 반짝이는 표시가 더 나온다 (은, 금, 루비, 에메랄드, 다이아몬드).
    pub fn is_precious(self) -> bool {
        matches!(self, Material::Silver | Material::Gold | Material::Ruby | Material::Emerald | Material::Diamond)
    }

    /// 조합 모양(`config`의 `..._SHAPE`)에 쓰는 표시 글자.
    pub fn craft_mark(self) -> char {
        match self {
            Material::Wood => CRAFT_WOOD_MARK,
            Material::Iron => CRAFT_IRON_MARK,
            Material::Copper => CRAFT_COPPER_MARK,
            Material::Silver => CRAFT_SILVER_MARK,
            Material::Gold => CRAFT_GOLD_MARK,
            Material::Ruby => CRAFT_RUBY_MARK,
            Material::Emerald => CRAFT_EMERALD_MARK,
            Material::Diamond => CRAFT_DIAMOND_MARK,
        }
    }

    /// 조합 모양의 표시 글자가 가리키는 재료. 빈칸(' ')이나 모르는 글자는 None이다.
    pub fn from_craft_mark(mark: char) -> Option<Material> {
        Material::ALL.into_iter().find(|material| material.craft_mark() == mark)
    }
}

/// 광석이 나오는 차례와 확률 (임시-15, D75). 흔한 것부터 적었다.
const ORE_CHANCES: [(Material, f32); 7] = [
    (Material::Iron, ORE_IRON_CHANCE),
    (Material::Copper, ORE_COPPER_CHANCE),
    (Material::Silver, ORE_SILVER_CHANCE),
    (Material::Ruby, ORE_RUBY_CHANCE),
    (Material::Gold, ORE_GOLD_CHANCE),
    (Material::Emerald, ORE_EMERALD_CHANCE),
    (Material::Diamond, ORE_DIAMOND_CHANCE),
];

/// `roll`(0 이상 1 미만)로 광석을 고른다 (임시-15, D75). 확률을 다 더해도 남는 몫은 다이아몬드다.
pub fn ore_for_roll(roll: f32) -> Material {
    let mut sum = 0.0;
    for (material, chance) in ORE_CHANCES {
        sum += chance;
        if roll < sum {
            return material;
        }
    }
    Material::Diamond
}

/// 가진 재료, 곡괭이, 제작대로 만든 새 물건 (US-078~083). 새 게임을 하면 새로 만든다 (저장은 M9).
#[derive(Debug, Clone, Default)]
pub struct Bag {
    counts: [u32; Material::ALL.len()],
    pub has_pickaxe: bool,
    /// 다이아래더라이트 곡괭이 (US-082). 한 번에 하나, 쿵을 쓰면 없어진다.
    pub has_diamond_pickaxe: bool,
    /// 주사기총 (US-083). 한 번에 하나.
    pub has_syringe_gun: bool,
    /// 엔더열매 개수 (US-081). 여러 개 가질 수 있다.
    pub ender_fruits: u32,
    /// 엔더열매를 먹어서 생긴 힘. 쿵을 쓸 때까지 남는다 (임시-44).
    pub has_ender_power: bool,
    /// 자동차를 타고 있으면 true (US-080). 탄 뒤 처음 오는 06:00에 내린다.
    pub riding_car: bool,
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

    /// 하나 캐는 데 걸리는 시간 (D5, 임시-45): 가진 것 중 가장 빠른 것으로 캔다.
    pub fn mine_seconds(&self) -> f32 {
        if self.has_diamond_pickaxe {
            MINE_SECONDS_DIAMOND_PICKAXE
        } else if self.has_pickaxe {
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
    pub fn keep_mining(&mut self, dt: f32, in_cave: bool, body: Rect) -> Option<(Material, Vec2)> {
        let needed = self.bag.mine_seconds();
        let job = self.job.as_mut()?;
        job.seconds += dt.max(0.0);
        let (index, is_done) = (job.index, job.seconds >= needed);
        let target = self.list(in_cave)[index].rect;
        if !is_reached_on_either_side(body, MINE_REACH, target) {
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
