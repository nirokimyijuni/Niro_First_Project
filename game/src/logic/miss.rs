//! 미스 (US-051): `HIDE_SPOTS` 중 한 곳에 숨어 있고, 찾으면 사라졌다가 다음 아침에 다른 곳에 숨는다.
//! 못 찾은 미스는 그 자리에서 기다린다 (임시-6). 몬스터와는 아무 일이 없다.

use super::monster::SeededRandom;
use super::world::World;
use crate::config::{HIDE_SPOTS, MISS_HEIGHT, MISS_WIDTH};
use macroquad::math::Rect;

pub struct Miss {
    /// 지금 숨는 곳 번호 (`HIDE_SPOTS` 순서).
    spot: usize,
    /// 찾은 날 번호. 숨어 있으면 None이다.
    found_on_day: Option<u32>,
    random: SeededRandom,
}

impl Miss {
    /// `seed`가 같으면 숨는 순서가 늘 같다.
    pub fn new(seed: u32) -> Miss {
        // 맵·싸움과 같은 seed를 쓰므로 섞어야 서로 따라가지 않는다.
        let mut random = SeededRandom::new(seed.wrapping_mul(31).wrapping_add(17));
        let spot = (random.next_unit() * HIDE_SPOTS.len() as f32) as usize;
        Miss { spot, found_on_day: None, random }
    }

    /// 아직 찾지 않아서 맵에 있으면 true.
    pub fn is_hiding(&self) -> bool {
        self.found_on_day.is_none()
    }

    /// 미스 몸 네모 (왼쪽 위 기준).
    pub fn rect(&self, world: &World) -> Rect {
        let (_, center_x, feet_height) = HIDE_SPOTS[self.spot];
        Rect::new(center_x - MISS_WIDTH / 2.0, world.ground_y - feet_height - MISS_HEIGHT, MISS_WIDTH, MISS_HEIGHT)
    }

    /// `day_number`날에 찾았다.
    pub fn get_found(&mut self, day_number: u32) {
        self.found_on_day = Some(day_number);
    }

    /// 찾은 날 다음 아침이 오면 조금 전과 다른 곳에 숨는다. `day_number`는 지금 날.
    pub fn move_on_new_morning(&mut self, day_number: u32) {
        if self.found_on_day.is_some_and(|found_day| day_number > found_day) {
            let other_spots = HIDE_SPOTS.len() - 1;
            let jump = 1 + (self.random.next_unit() * other_spots as f32) as usize;
            self.spot = (self.spot + jump) % HIDE_SPOTS.len();
            self.found_on_day = None;
        }
    }
}
