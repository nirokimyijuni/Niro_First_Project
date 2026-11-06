//! 캐릭터 그리기 + 움직임(애니메이션).
//! 원화가 캐릭터마다 한 장뿐이라, 늘이기·통통 튀기·기울이기로 움직이는 느낌을 낸다.

use crate::logic::player::{Facing, Motion};
use macroquad::prelude::*;
use std::f32::consts::TAU;

/// 게임 화면에서 안테나를 뺀 캐릭터 몸 높이 (가상 픽셀). 충돌 네모(64)보다 조금 크게.
pub const PLAY_SPRITE_HEIGHT: f32 = 84.0;
/// 좀비 그림 높이 (가상 픽셀). 몸 네모(48×64)보다 조금 크게. 가로는 원본 비율로 약 60.
pub const ZOMBIE_SPRITE_HEIGHT: f32 = 80.0;

/// 대기: 숨쉬기 빠르기(초당 번)와 세기
const IDLE_BREATHS_PER_SECOND: f32 = 0.8;
pub(super) const IDLE_STRETCH: f32 = 0.03;
/// 걷기: 초당 걸음 수, 통통 튀는 높이(그림 높이에 대한 비율), 기울기(라디안)
const WALK_STEPS_PER_SECOND: f32 = 4.4;
const WALK_BOUNCE: f32 = 0.07;
const WALK_TILT: f32 = 0.09;
/// 점프: 올라갈 때 / 내려올 때 위로 늘어나는 정도
const JUMP_STRETCH_UP: f32 = 0.14;
const JUMP_STRETCH_DOWN: f32 = 0.05;
/// 떠오른 좀비: 초당 살랑거리는 번, 기울기(라디안), 위아래 폭(그림 높이에 대한 비율)
const FLOAT_SWAYS_PER_SECOND: f32 = 0.7;
const FLOAT_TILT: f32 = 0.16;
const FLOAT_BOB: f32 = 0.05;

/// 한 순간의 모양.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    pub scale_x: f32,
    pub scale_y: f32,
    /// 발이 땅에서 뜨는 높이 (그림 높이에 대한 비율)
    pub lift: f32,
    /// 발을 중심으로 기울이는 각도 (라디안)
    pub tilt: f32,
}

/// `time`: 초 단위로 계속 흐르는 시간. `rising`: 점프 중 올라가는 중인지.
pub fn pose(motion: Motion, time: f32, rising: bool) -> Pose {
    match motion {
        Motion::Idle => {
            let s = (time * TAU * IDLE_BREATHS_PER_SECOND).sin();
            Pose { scale_x: 1.0 - IDLE_STRETCH * 0.5 * s, scale_y: 1.0 + IDLE_STRETCH * s, lift: 0.0, tilt: 0.0 }
        }
        Motion::Walk => {
            // 한 걸음(반 바퀴)마다 한 번 튀고, 왼발/오른발에 따라 좌우로 기운다.
            let phase = time * TAU * WALK_STEPS_PER_SECOND / 2.0;
            Pose {
                scale_x: 1.0,
                scale_y: 1.0,
                lift: phase.sin().abs() * WALK_BOUNCE,
                tilt: phase.sin() * WALK_TILT,
            }
        }
        Motion::Jump => {
            let stretch = if rising { JUMP_STRETCH_UP } else { JUMP_STRETCH_DOWN };
            Pose { scale_x: 1.0 - stretch * 0.6, scale_y: 1.0 + stretch, lift: 0.0, tilt: 0.0 }
        }
    }
}

/// 떠오른 좀비의 살랑살랑 (`time`이 0이면 가만히).
pub fn float_pose(time: f32) -> Pose {
    let s = (time * TAU * FLOAT_SWAYS_PER_SECOND).sin();
    Pose { scale_x: 1.0, scale_y: 1.0, lift: s * FLOAT_BOB, tilt: s * FLOAT_TILT }
}

/// 그림이 발 가운데에서 옆으로 가장 멀리 뻗는 거리 (숨쉬기로 넓어짐 + 걸을 때 기울기 포함).
/// 화면 끝에서 그림이 잘리지 않게 할 때 쓴다 (BUG-001).
pub fn max_half_width(texture: &Texture2D, height: f32) -> f32 {
    let width = height * texture.width() / texture.height();
    width / 2.0 * (1.0 + IDLE_STRETCH) + height * WALK_TILT.sin()
}

/// `feet`: 발 가운데 위치. `height`: 기본 그림 높이 (원화 비율대로 가로가 정해진다).
/// `tint`: 그림에 곱할 색. 보통 WHITE(그대로), 밤에는 살짝 푸르스름하게.
pub fn draw_character(texture: &Texture2D, feet: Vec2, height: f32, facing: Facing, pose: Pose, tint: Color) {
    let width = height * texture.width() / texture.height();
    let size = vec2(width * pose.scale_x, height * pose.scale_y);
    let foot = vec2(feet.x, feet.y - pose.lift * height);
    draw_texture_ex(
        texture,
        foot.x - size.x / 2.0,
        foot.y - size.y,
        tint,
        DrawTextureParams {
            dest_size: Some(size),
            rotation: pose.tilt,
            pivot: Some(foot),
            flip_x: facing == Facing::Left,
            ..Default::default()
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_breathes_but_stays_on_ground() {
        let a = pose(Motion::Idle, 0.3, false);
        let b = pose(Motion::Idle, 0.9, false);
        assert_ne!(a.scale_y, b.scale_y);
        assert_eq!(a.lift, 0.0);
        assert!((a.scale_y - 1.0).abs() <= IDLE_STRETCH);
    }

    #[test]
    fn walk_bounces_and_tilts_both_ways() {
        let left = pose(Motion::Walk, 0.25 / WALK_STEPS_PER_SECOND * 2.0, false);
        let right = pose(Motion::Walk, 0.75 / WALK_STEPS_PER_SECOND * 2.0, false);
        assert!(left.lift > 0.0 && right.lift > 0.0);
        assert!(left.tilt > 0.0 && right.tilt < 0.0);
    }

    #[test]
    fn jump_is_taller() {
        assert!(pose(Motion::Jump, 0.0, true).scale_y > 1.1);
        assert!(pose(Motion::Jump, 0.0, false).scale_y > 1.0);
    }
}
