//! 캐릭터가 걸어 다니는 곳: 땅 + 상자(장애물).
//! 좌표: 왼쪽 위가 (0, 0), 아래로 갈수록 y가 커진다.

use super::monster::SeededRandom;
use crate::config::{
    BOX_AREA_LEFT, BOX_AREA_RIGHT, BOX_BIG_CHANCE, BOX_BIG_SIZE, BOX_COUNT_MAX, BOX_COUNT_MIN, BOX_GAP_CHANCE,
    BOX_MIN_GAP, BOX_SMALL_SIZE, CITIZEN_RUN_UP_ROOM, CITIZEN_XS, GROUND_Y, HIDE_SPOTS, HOLE_DEPTH, HOLE_LEFT, HOLE_WIDTH, MAP_WIDTH,
    NO_BOX_MARGIN, PLACE_NAMES, ROCK_STEPS, SCREEN_WIDTH, TREE_BRANCHES,
};
#[cfg(test)]
use crate::config::WORLD_WIDTH;
use macroquad::math::Rect;

/// 두 네모가 정말로 겹치는지. 모서리끼리 딱 닿기만 한 건 겹친 게 아니다
/// (`Rect::overlaps`는 닿기만 해도 겹침이다). 상자 위에 서서 옆으로 걸어도 "옆면에 막힘"으로
/// 오해하지 않고, 상자 옆에 붙은 몬스터가 건너뛰지 않고, 상자 위 발이 몬스터 머리에 맞지 않는다.
pub fn really_overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

pub struct World {
    /// 월드 가로 길이. 캐릭터는 0 ~ width 안에서만 움직인다.
    pub width: f32,
    /// 땅 윗면의 y. 이보다 아래로는 떨어지지 않는다.
    pub ground_y: f32,
    /// 상자들. Rect의 (x, y)는 상자의 왼쪽 위 모서리.
    pub obstacles: Vec<Rect>,
}

impl World {
    /// M1 연습 맵: 화면 폭만 한 땅과 점프로 넘을 상자 3개.
    #[cfg(test)]
    pub fn m1() -> World {
        let ground_y = GROUND_Y;
        World {
            width: WORLD_WIDTH,
            ground_y,
            obstacles: vec![
                // 작은 상자
                Rect::new(300.0, ground_y - 48.0, 48.0, 48.0),
                // 조금 큰 상자 (점프 최고 높이 약 97픽셀이라 넘을 수 있다)
                Rect::new(560.0, ground_y - 64.0, 64.0, 64.0),
                // 작은 상자
                Rect::new(780.0, ground_y - 48.0, 48.0, 48.0),
            ],
        }
    }

    /// D36 한 화면짜리 랜덤 맵: 상자 수와 크기, 위치가 랜덤이다. `seed`가 같으면 늘 같은 배치.
    #[cfg(test)]
    #[allow(dead_code)] // game/tests의 D36 시험만 쓴다
    pub fn with_random_boxes(seed: u32) -> World {
        // Combat도 같은 seed를 쓴다. 뒤집어야 상자 배치와 몬스터 순서가 서로 따라가지 않는다.
        let mut random = SeededRandom::new(!seed);
        World { width: WORLD_WIDTH, ground_y: GROUND_Y, obstacles: random_boxes(&mut random, 0.0) }
    }

    /// US-050 새 게임의 긴 맵: 장소마다 상자를 따로 뽑고 (임시-2), 숨는 곳·시민·구조물 곁의 상자는 뺀다 (임시-5).
    pub fn long_map(seed: u32) -> World {
        let mut random = SeededRandom::new(!seed);
        let mut world = World { width: MAP_WIDTH, ground_y: GROUND_Y, obstacles: Vec::new() };
        let zones = world.no_box_zones();
        for place in 0..PLACE_NAMES.len() {
            let left = place as f32 * SCREEN_WIDTH;
            let boxes = keep_boxes_out_of_zones(random_boxes(&mut random, left), &zones);
            if boxes.is_empty() {
                world.obstacles.extend(small_box_in_free_spot(left, &zones));
            }
            world.obstacles.extend(boxes);
        }
        let structures = [world.tree_branches(), world.rock_steps()].concat();
        world.obstacles.extend(structures);
        world
    }

    /// 구멍·구조물·미스·시민은 긴 맵에만 있다. M1 연습 맵은 옛 규칙 그대로다.
    pub fn is_long_map(&self) -> bool {
        self.width >= MAP_WIDTH
    }

    /// US-051 화산의 구멍. 땅 윗면 아래의 네모라서 y는 땅이고 h는 깊이다.
    /// shortcut: 구멍이 하나뿐이라 config에서 바로 계산한다, 맵마다 달라지면 World 필드로 옮긴다.
    pub fn hole(&self) -> Option<Rect> {
        self.is_long_map().then(|| Rect::new(HOLE_LEFT, self.ground_y, HOLE_WIDTH, HOLE_DEPTH))
    }

    /// US-051 나무 발판. `obstacles`에 들어 있어서 상자처럼 서고 부딪힌다.
    pub fn tree_branches(&self) -> Vec<Rect> {
        self.blocks(&TREE_BRANCHES)
    }

    /// US-051 바위 계단. `obstacles`에 들어 있어서 상자처럼 서고 부딪힌다.
    pub fn rock_steps(&self) -> Vec<Rect> {
        self.blocks(&ROCK_STEPS)
    }

    fn blocks(&self, list: &[(f32, f32, f32, f32)]) -> Vec<Rect> {
        if !self.is_long_map() {
            return Vec::new();
        }
        list.iter().map(|&(x, top, width, thickness)| Rect::new(x, self.ground_y - top, width, thickness)).collect()
    }

    /// 나무 발판과 바위 계단을 뺀 진짜 상자들. 코인은 이 위에만 놓인다.
    pub fn boxes(&self) -> Vec<Rect> {
        let (branches, steps) = (self.tree_branches(), self.rock_steps());
        self.obstacles.iter().copied().filter(|b| !branches.contains(b) && !steps.contains(b)).collect()
    }

    /// 땅을 걷는 몬스터를 막는 벽: 상자·구조물과 구멍 (US-051: 구멍 앞에서 막힌다).
    pub fn monster_walls(&self) -> impl Iterator<Item = Rect> + '_ {
        let hole_wall = self.hole().map(|hole| Rect::new(hole.x, 0.0, hole.w, self.ground_y));
        self.obstacles.iter().copied().chain(hole_wall)
    }

    /// `body`가 떨어지면 닿는 땅의 y. 몸이 구멍 안에 다 들어와야 구멍 바닥이다 (걸치면 땅에 선다).
    pub fn floor_y_under(&self, body: Rect) -> f32 {
        match self.hole() {
            Some(hole) if hole.x <= body.x && body.x + body.w <= hole.x + hole.w => hole.y + hole.h,
            _ => self.ground_y,
        }
    }

    /// `body` 아래에서 가장 높은 윗면의 y: 땅, 구멍 바닥, 상자, 구조물 중 하나.
    pub fn surface_y_under(&self, body: Rect) -> f32 {
        self.obstacles
            .iter()
            .filter(|b| b.x < body.x + body.w && body.x < b.x + b.w)
            .fold(self.floor_y_under(body), |top, b| top.min(b.y))
    }

    /// 카메라 왼쪽 끝 x. 주인공을 가운데에 두고, 맵 끝에서는 멈춘다 (US-050).
    /// 로건은 화면에 그릴 때 월드 x에서 이 값을 뺀다.
    pub fn camera_left(&self, player_center_x: f32) -> f32 {
        (player_center_x - SCREEN_WIDTH / 2.0).clamp(0.0, (self.width - SCREEN_WIDTH).max(0.0))
    }

    /// 월드 `x`가 주인공이 보는 화면 안에 있는지.
    pub fn is_on_screen(&self, x: f32, player_center_x: f32) -> bool {
        let left = self.camera_left(player_center_x);
        left <= x && x <= left + SCREEN_WIDTH
    }

    /// 월드 `x`가 있는 장소 번호 (`PLACE_NAMES` 순서).
    pub fn place_index_at(&self, x: f32) -> usize {
        ((x / SCREEN_WIDTH) as usize).min(PLACE_NAMES.len() - 1)
    }

    /// 월드 `x`가 있는 장소 이름.
    pub fn place_name_at(&self, x: f32) -> &'static str {
        PLACE_NAMES[self.place_index_at(x)]
    }

    /// 상자를 놓지 않는 구간들 (왼쪽 x, 오른쪽 x).
    fn no_box_zones(&self) -> Vec<(f32, f32)> {
        let spots = HIDE_SPOTS.iter().map(|spot| (spot.1, spot.1));
        let solids = self.tree_branches().into_iter().chain(self.rock_steps()).chain(self.hole());
        let citizens = CITIZEN_XS.iter().map(|x| (x - CITIZEN_RUN_UP_ROOM, x + CITIZEN_RUN_UP_ROOM));
        spots
            .chain(solids.map(|r| (r.x, r.x + r.w)))
            .map(|(left, right)| (left - NO_BOX_MARGIN, right + NO_BOX_MARGIN))
            .chain(citizens)
            .collect()
    }
}

/// 장소 왼쪽 끝이 `left`인 곳에 놓을 랜덤 상자들. 왼쪽에서 오른쪽으로 놓는다.
/// 상자 사이는 붙거나 `BOX_MIN_GAP` 이상 벌어진다.
fn random_boxes(random: &mut SeededRandom, left: f32) -> Vec<Rect> {
    let (sizes, gap_before) = pick_box_sizes_and_gaps(random);

    // 남는 폭을 맨 왼쪽과 틈마다 랜덤하게 나눈다. 마지막 몫은 맨 오른쪽에 남는다.
    let gap_count = gap_before.iter().filter(|has_gap| **has_gap).count();
    let boxes_width: f32 = sizes.iter().sum();
    let spare = BOX_AREA_RIGHT - BOX_AREA_LEFT - boxes_width - gap_count as f32 * BOX_MIN_GAP;
    let shares: Vec<f32> = (0..gap_count + 2).map(|_| 1.0 - random.next_unit()).collect();
    let spare_per_share = spare / shares.iter().sum::<f32>();
    let mut next_spare = shares.iter().map(|share| (share * spare_per_share).floor());

    let mut x = left + BOX_AREA_LEFT + next_spare.next().unwrap_or(0.0);
    let mut boxes = Vec::new();
    for (size, has_gap) in sizes.into_iter().zip(gap_before) {
        if has_gap {
            x += BOX_MIN_GAP + next_spare.next().unwrap_or(0.0);
        }
        boxes.push(Rect::new(x, GROUND_Y - size, size, size));
        x += size;
    }
    boxes
}

fn is_in_a_zone(left: f32, right: f32, zones: &[(f32, f32)]) -> bool {
    zones.iter().any(|&(zone_left, zone_right)| left < zone_right && zone_left < right)
}

/// 구간에 걸린 상자를 뺀다. 빼서 생긴 틈이 몸이 끼는 폭이면 다음 상자도 뺀다 (US-051, BUG-006).
fn keep_boxes_out_of_zones(boxes: Vec<Rect>, zones: &[(f32, f32)]) -> Vec<Rect> {
    let mut kept: Vec<Rect> = Vec::new();
    for b in boxes {
        let gap = kept.last().map_or(BOX_MIN_GAP, |last| b.x - (last.x + last.w));
        let narrow_gap = gap > 0.0 && gap < BOX_MIN_GAP;
        if !narrow_gap && !is_in_a_zone(b.x, b.x + b.w, zones) {
            kept.push(b);
        }
    }
    kept
}

/// 상자를 다 뺀 장소에도 자리가 있으면 하나는 남긴다 (US-050). 왼쪽부터 빈 자리를 찾는다.
fn small_box_in_free_spot(left: f32, zones: &[(f32, f32)]) -> Option<Rect> {
    const SEARCH_STEP: usize = 4;
    let area_left = left + BOX_AREA_LEFT;
    let search_width = BOX_AREA_RIGHT - BOX_AREA_LEFT - BOX_SMALL_SIZE;
    (0..search_width as usize)
        .step_by(SEARCH_STEP)
        .map(|offset| area_left + offset as f32)
        .find(|&x| !is_in_a_zone(x, x + BOX_SMALL_SIZE, zones))
        .map(|x| Rect::new(x, GROUND_Y - BOX_SMALL_SIZE, BOX_SMALL_SIZE, BOX_SMALL_SIZE))
}

/// 상자마다 크기와 "앞에 틈이 있는지"를 고른다. 첫 상자 앞에는 틈이 없다.
fn pick_box_sizes_and_gaps(random: &mut SeededRandom) -> (Vec<f32>, Vec<bool>) {
    let count_choices = (BOX_COUNT_MAX - BOX_COUNT_MIN + 1) as f32;
    let count = BOX_COUNT_MIN + (random.next_unit() * count_choices) as usize;
    let mut sizes: Vec<f32> = (0..count)
        .map(|_| if random.next_unit() < BOX_BIG_CHANCE { BOX_BIG_SIZE } else { BOX_SMALL_SIZE })
        .collect();

    let boxes_width: f32 = sizes.iter().sum();
    let mut gaps_that_fit = ((BOX_AREA_RIGHT - BOX_AREA_LEFT - boxes_width) / BOX_MIN_GAP) as usize;
    let mut gap_before = vec![false; count];
    for has_gap in gap_before.iter_mut().skip(1) {
        if gaps_that_fit > 0 && random.next_unit() < BOX_GAP_CHANCE {
            *has_gap = true;
            gaps_that_fit -= 1;
        }
    }

    // 큰 상자 둘 사이에 붙은 작은 상자 하나는 몸 폭만 한 구덩이라서 끼인다. 뒤 상자를 작게 줄인다.
    for i in 2..count {
        let touching = !gap_before[i - 1] && !gap_before[i];
        if touching && sizes[i - 2] == BOX_BIG_SIZE && sizes[i - 1] == BOX_SMALL_SIZE && sizes[i] == BOX_BIG_SIZE {
            sizes[i] = BOX_SMALL_SIZE;
        }
    }
    (sizes, gap_before)
}
