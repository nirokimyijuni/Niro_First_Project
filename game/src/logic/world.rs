//! 캐릭터가 걸어 다니는 곳: 땅 + 상자(장애물).
//! 좌표: 왼쪽 위가 (0, 0), 아래로 갈수록 y가 커진다.

use crate::config::{GROUND_Y, WORLD_WIDTH};
use macroquad::math::Rect;

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
}
