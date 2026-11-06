//! 캐릭터 그리기 + 움직임(애니메이션).
//! 원화가 캐릭터마다 한 장뿐이라, 늘이기·통통 튀기·기울이기로 움직이는 느낌을 낸다.

use crate::logic::player::{Facing, Motion};
use crate::logic::skeleton::{Projectile, ProjectileKind, SkeletonStage};
use macroquad::prelude::*;
use std::f32::consts::TAU;

/// 게임 화면에서 안테나를 뺀 캐릭터 몸 높이 (가상 픽셀). 충돌 네모(64)보다 조금 크게.
pub const PLAY_SPRITE_HEIGHT: f32 = 84.0;
/// 좀비 그림 높이 (가상 픽셀). 몸 네모(48×64)보다 조금 크게. 가로는 원본 비율로 약 60.
pub const ZOMBIE_SPRITE_HEIGHT: f32 = 80.0;
/// 스켈레톤 그림 높이 (가상 픽셀). 몸 네모(48×64)에 좀비와 같은 키. 가로는 원본 비율로 약 52~54.
const SKELETON_SPRITE_HEIGHT: f32 = 80.0;
/// 완전체는 가늘어서 작아 보이므로 조금 크게 (가로 약 65).
const SKELETON_FULL_SPRITE_HEIGHT: f32 = 92.0;
/// 피할 때 뒤로 젖히는 각도 (라디안)
pub const DODGE_LEAN: f32 = 0.3;
/// 튕겨 온 화살·총알이 살짝 기울어 나는 각도 (라디안)
const BOUNCED_TILT: f32 = 0.2;
const ARROW_HEAD_LENGTH: f32 = 8.0;
const ARROW_SHAFT: Color = Color::new(0.98, 0.90, 0.68, 1.0);
const BULLET_BODY: Color = Color::new(1.0, 0.86, 0.30, 1.0);
const BOUNCED_SHOT: Color = Color::new(1.0, 0.50, 0.72, 1.0);
const SHOT_OUTLINE: Color = Color::new(0.13, 0.16, 0.36, 1.0);

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

pub fn skeleton_sprite_height(stage: SkeletonStage) -> f32 {
    match stage {
        SkeletonStage::Full => SKELETON_FULL_SPRITE_HEIGHT,
        _ => SKELETON_SPRITE_HEIGHT,
    }
}

/// 그림이 왼쪽을 보고 있는지. 일반·절반은 긴 팔이 왼쪽으로, 완전체는 총이 오른쪽에 있다.
pub fn skeleton_art_faces_left(stage: SkeletonStage) -> bool {
    stage != SkeletonStage::Full
}

/// 화살이나 총알 하나. 네모(`rect`) 크기대로, 날아가는 쪽이 앞이다. 튕긴 것은 분홍빛으로 살짝 기운다.
pub fn draw_projectile(shot: &Projectile) {
    let tilt = if shot.bounced { BOUNCED_TILT } else { 0.0 };
    let r = shot.rect();
    let forward = vec2(tilt.cos() * shot.direction, tilt.sin());
    let side = vec2(-forward.y, forward.x);
    let (tip, tail) = (r.center() + forward * r.w / 2.0, r.center() - forward * r.w / 2.0);
    let line = |a: Vec2, b: Vec2, thickness: f32, color: Color| draw_line(a.x, a.y, b.x, b.y, thickness, color);
    match shot.kind {
        ProjectileKind::Arrow => {
            let color = if shot.bounced { BOUNCED_SHOT } else { ARROW_SHAFT };
            let neck = tip - forward * ARROW_HEAD_LENGTH;
            let (wing, fin) = (side * r.h / 2.0, tail + forward * 5.0);
            line(tail, neck, 4.0, SHOT_OUTLINE);
            line(tail, neck, 2.0, color);
            draw_triangle(tip + forward * 1.5, neck + wing * 1.5, neck - wing * 1.5, SHOT_OUTLINE);
            draw_triangle(tip, neck + wing, neck - wing, color);
            line(fin, tail + wing, 2.0, color);
            line(fin, tail - wing, 2.0, color);
        }
        ProjectileKind::Bullet => {
            let color = if shot.bounced { BOUNCED_SHOT } else { BULLET_BODY };
            let front = tip - forward * r.h / 2.0;
            line(tail, front, r.h + 2.0, SHOT_OUTLINE);
            draw_circle(front.x, front.y, r.h / 2.0 + 1.0, SHOT_OUTLINE);
            line(tail, front, r.h, color);
            draw_circle(front.x, front.y, r.h / 2.0, color);
        }
    }
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
