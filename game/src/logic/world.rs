//! 캐릭터가 걸어 다니는 곳: 땅 + 상자(장애물).
//! 좌표: 왼쪽 위가 (0, 0), 아래로 갈수록 y가 커진다.

use super::monster::SeededRandom;
use crate::config::{
    BOX_AREA_LEFT, BOX_AREA_RIGHT, BOX_BIG_CHANCE, BOX_BIG_SIZE, BOX_COUNT_MAX, BOX_COUNT_MIN, BOX_GAP_CHANCE,
    BOX_MIN_GAP, BOX_SMALL_SIZE, GROUND_Y, WORLD_WIDTH,
};
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

    /// D36 새 게임 맵: 상자 수와 크기, 위치가 랜덤이다. `seed`가 같으면 늘 같은 배치.
    /// 왼쪽에서 오른쪽으로 놓는다. 상자 사이는 붙거나 `BOX_MIN_GAP` 이상 벌어진다.
    pub fn with_random_boxes(seed: u32) -> World {
        // Combat도 같은 seed를 쓴다. 뒤집어야 상자 배치와 몬스터 순서가 서로 따라가지 않는다.
        let mut random = SeededRandom::new(!seed);
        let (sizes, gap_before) = pick_box_sizes_and_gaps(&mut random);

        // 남는 폭을 맨 왼쪽과 틈마다 랜덤하게 나눈다. 마지막 몫은 맨 오른쪽에 남는다.
        let gap_count = gap_before.iter().filter(|has_gap| **has_gap).count();
        let boxes_width: f32 = sizes.iter().sum();
        let spare = BOX_AREA_RIGHT - BOX_AREA_LEFT - boxes_width - gap_count as f32 * BOX_MIN_GAP;
        let shares: Vec<f32> = (0..gap_count + 2).map(|_| 1.0 - random.next_unit()).collect();
        let spare_per_share = spare / shares.iter().sum::<f32>();
        let mut next_spare = shares.iter().map(|share| (share * spare_per_share).floor());

        let ground_y = GROUND_Y;
        let mut x = BOX_AREA_LEFT + next_spare.next().unwrap_or(0.0);
        let mut obstacles = Vec::new();
        for (size, has_gap) in sizes.into_iter().zip(gap_before) {
            if has_gap {
                x += BOX_MIN_GAP + next_spare.next().unwrap_or(0.0);
            }
            obstacles.push(Rect::new(x, ground_y - size, size, size));
            x += size;
        }
        World { width: WORLD_WIDTH, ground_y, obstacles }
    }
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
