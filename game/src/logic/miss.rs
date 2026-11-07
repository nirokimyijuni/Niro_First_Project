//! 미스 (US-051): `HIDE_SPOTS` 중 한 곳에 숨어 있고, 찾으면 사라졌다가 잠시 뒤 다른 곳에 숨는다.
//! 못 찾은 미스는 그 자리에서 기다린다 (임시-6). 몬스터와는 아무 일이 없다.

use super::monster::SeededRandom;
use super::world::{really_overlaps, World};
use crate::config::{HIDE_SPOTS, MISS_HEIGHT, MISS_RETURN_SECONDS, MISS_WIDTH};
use macroquad::math::Rect;

pub struct Miss {
    /// 지금 숨는 곳 번호 (`HIDE_SPOTS` 순서).
    spot: usize,
    /// 찾은 뒤 다시 숨을 때까지 남은 초. 숨어 있으면 None이다.
    seconds_until_back: Option<f32>,
    random: SeededRandom,
}

impl Miss {
    /// `seed`가 같으면 숨는 순서가 늘 같다.
    pub fn new(seed: u32) -> Miss {
        // 맵·싸움과 같은 seed를 쓰므로 섞어야 서로 따라가지 않는다.
        let mut random = SeededRandom::new(seed.wrapping_mul(31).wrapping_add(17));
        let spot = (random.next_unit() * HIDE_SPOTS.len() as f32) as usize;
        Miss { spot, seconds_until_back: None, random }
    }

    /// 아직 찾지 않아서 맵에 있으면 true.
    pub fn is_hiding(&self) -> bool {
        self.seconds_until_back.is_none()
    }

    /// 미스 몸 네모 (왼쪽 위 기준).
    pub fn rect(&self, world: &World) -> Rect {
        spot_rect(self.spot, world)
    }

    /// 찾았다. `MISS_RETURN_SECONDS` 뒤에 다른 곳에 다시 숨는다.
    pub fn get_found(&mut self) {
        self.seconds_until_back = Some(MISS_RETURN_SECONDS);
    }

    /// `dt`초 지낸다. 찾은 뒤 시간이 다 되면 조금 전과 다른 곳에 숨는다.
    /// 주인공 몸에 닿는 곳은 뽑지 않는다: 보이지도 않고 바로 찾아지면 재미없다. 모두 닿으면 그대로 뽑는다.
    pub fn wait_to_come_back(&mut self, dt: f32, player_rect: Rect, world: &World) {
        let Some(seconds_left) = self.seconds_until_back else {
            return;
        };
        let seconds_left = seconds_left - dt.max(0.0);
        if seconds_left > 0.0 {
            self.seconds_until_back = Some(seconds_left);
            return;
        }
        let others: Vec<usize> = (0..HIDE_SPOTS.len()).filter(|spot| *spot != self.spot).collect();
        let away_from_player: Vec<usize> =
            others.iter().copied().filter(|spot| !really_overlaps(spot_rect(*spot, world), player_rect)).collect();
        let choices = if away_from_player.is_empty() { others } else { away_from_player };
        self.spot = choices[(self.random.next_unit() * choices.len() as f32) as usize];
        self.seconds_until_back = None;
    }
}

/// `spot`번 숨는 곳에 선 미스 몸 네모 (왼쪽 위 기준).
fn spot_rect(spot: usize, world: &World) -> Rect {
    let (_, center_x, feet_height) = HIDE_SPOTS[spot];
    Rect::new(center_x - MISS_WIDTH / 2.0, world.ground_y - feet_height - MISS_HEIGHT, MISS_WIDTH, MISS_HEIGHT)
}
