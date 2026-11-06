//! 플레이어: 좌우로 걷고, 땅에 있을 때 점프하고, 땅과 상자에 부딪힌다.
//!
//! 좌표 기준: `pos`는 캐릭터 네모의 **왼쪽 위 모서리**다.
//! 왼쪽 위가 (0, 0)이고 아래로 갈수록 y가 커진다. 그래서 발 위치는 `pos.y + size.y`.

use super::character::CharacterKind;
use super::world::{really_overlaps, World};
use crate::config::{
    GRAVITY, MAX_FALL_SPEED, PHYSICS_STEP_SECONDS, PLAYER_EDGE_SLIDE_SPEED, PLAYER_HEIGHT,
    PLAYER_JUMP_SPEED, PLAYER_WALK_SPEED, PLAYER_WIDTH,
};
use macroquad::math::{Rect, Vec2};

/// 이번 프레임에 누른 버튼 (키보드든 터치든 로건이 채워 준다).
#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerInput {
    pub left: bool,
    pub right: bool,
    /// 이번 프레임에 "새로" 눌렀을 때만 true. 꾹 누르고 있는 동안은 false.
    pub jump_pressed: bool,
    /// 이번 프레임에 "새로" 눌렀을 때만 true. 꾹 눌러도 공격은 한 번 (US-040).
    pub attack_pressed: bool,
}

/// 어떤 동작 그림을 보여 줄지.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motion {
    /// 땅에서 가만히
    Idle,
    /// 땅에서 걷는 중
    Walk,
    /// 공중에 있음 (뛰어오르든 떨어지든)
    Jump,
}

/// 캐릭터가 바라보는 쪽.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Facing {
    Left,
    Right,
}

pub struct Player {
    pub kind: CharacterKind,
    /// 캐릭터 네모의 왼쪽 위 모서리.
    pub pos: Vec2,
    /// 속도 (초당 픽셀). y가 음수면 위로 올라가는 중.
    pub vel: Vec2,
    /// 캐릭터 크기 (가로, 세로).
    pub size: Vec2,
    /// 땅이나 상자 위에 서 있으면 true.
    pub on_ground: bool,
    pub facing: Facing,
    pub motion: Motion,
}

impl Player {
    /// `spawn`은 캐릭터 왼쪽 위 모서리 위치.
    pub fn new(kind: CharacterKind, spawn: Vec2) -> Player {
        Player {
            kind,
            pos: spawn,
            vel: Vec2::ZERO,
            size: Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT),
            on_ground: false,
            facing: Facing::Right,
            motion: Motion::Idle,
        }
    }

    /// 캐릭터가 차지하는 네모 (왼쪽 위 기준).
    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.size.x, self.size.y)
    }

    /// 한 프레임 진행. `dt`는 지난 프레임부터 흐른 시간(초).
    pub fn update(&mut self, input: &PlayerInput, dt: f32, world: &World) {
        // 1) 좌우: 둘 다 누르거나 둘 다 안 누르면 멈춘다.
        let walk_direction = match (input.left, input.right) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };
        self.vel.x = walk_direction * PLAYER_WALK_SPEED;
        if walk_direction < 0.0 {
            self.facing = Facing::Left;
        } else if walk_direction > 0.0 {
            self.facing = Facing::Right;
        }

        // 2) 점프: 땅(또는 상자) 위에 있을 때만.
        if input.jump_pressed && self.on_ground {
            self.vel.y = -PLAYER_JUMP_SPEED;
            self.on_ground = false;
        }

        // 3) 시간을 같은 크기로 잘게 나눠서 움직인다 → 프레임이 길어도 상자를 뚫지 않는다.
        // 같은 크기여야 아주 작은 나머지 조각이 안 생긴다. 그런 조각은 상자 위에서 공중으로 쳐진다 (REVIEW-004 8번).
        let dt = dt.max(0.0);
        let steps = (dt / PHYSICS_STEP_SECONDS).ceil();
        let step = dt / steps.max(1.0);
        for _ in 0..steps as u32 {
            self.move_one_step(step, world);
        }

        // 4) 보여 줄 동작 고르기.
        self.motion = if !self.on_ground {
            Motion::Jump
        } else if walk_direction != 0.0 {
            Motion::Walk
        } else {
            Motion::Idle
        };
    }

    /// 짧은 시간 `step` 동안 움직이기. 가로 먼저, 그다음 세로 (축마다 따로 부딪힘 처리).
    fn move_one_step(&mut self, step: f32, world: &World) {
        // 중력
        self.vel.y = (self.vel.y + GRAVITY * step).min(MAX_FALL_SPEED);

        // --- 가로 ---
        self.pos.x += self.vel.x * step;
        // 월드 양 끝
        self.pos.x = self.pos.x.clamp(0.0, world.width - self.size.x);
        // 상자 옆면
        for wall in &world.obstacles {
            if really_overlaps(self.rect(), *wall) {
                if self.vel.x > 0.0 {
                    self.pos.x = wall.x - self.size.x; // 상자 왼쪽 면에 막힘
                } else if self.vel.x < 0.0 {
                    self.pos.x = wall.x + wall.w; // 상자 오른쪽 면에 막힘
                }
            }
        }

        // --- 세로 ---
        self.pos.y += self.vel.y * step;
        self.on_ground = false;
        // 상자 윗면(올라섬) / 아랫면(머리 박음)
        for block in &world.obstacles {
            if really_overlaps(self.rect(), *block) {
                if self.vel.y > 0.0 {
                    self.pos.y = block.y - self.size.y;
                    self.vel.y = 0.0;
                    // 몸 가운데가 윗면 위(끝 포함)일 때만 선다. 밖이면 바깥쪽으로 미끄러져 떨어진다.
                    // 높이가 같게 붙은 상자는 이어진 바닥이라 옆 상자 위여도 선다.
                    // shortcut: 틈이 캐릭터보다 좁으면 못 빠져나간다, 지금은 World::with_random_boxes가 그런 틈을 안 만든다.
                    let center_x = self.pos.x + self.size.x / 2.0;
                    let is_over_a_top =
                        world.obstacles.iter().any(|b| b.y == block.y && b.x <= center_x && center_x <= b.x + b.w);
                    if is_over_a_top {
                        self.on_ground = true;
                    } else if center_x < block.x {
                        self.pos.x -= PLAYER_EDGE_SLIDE_SPEED * step;
                    } else {
                        self.pos.x += PLAYER_EDGE_SLIDE_SPEED * step;
                    }
                } else if self.vel.y < 0.0 {
                    self.pos.y = block.y + block.h;
                    self.vel.y = 0.0;
                }
            }
        }
        // 땅
        if self.pos.y + self.size.y >= world.ground_y {
            self.pos.y = world.ground_y - self.size.y;
            self.vel.y = 0.0;
            self.on_ground = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: f32 = 1.0 / 60.0;

    fn no_input() -> PlayerInput {
        PlayerInput::default()
    }
    fn hold_right() -> PlayerInput {
        PlayerInput { right: true, ..Default::default() }
    }
    fn hold_left() -> PlayerInput {
        PlayerInput { left: true, ..Default::default() }
    }
    fn press_jump() -> PlayerInput {
        PlayerInput { jump_pressed: true, ..Default::default() }
    }

    /// 상자 없는 평평한 땅.
    fn flat_world() -> World {
        World { width: 960.0, ground_y: 440.0, obstacles: vec![] }
    }

    /// 땅 위 x 위치에 서 있는 플레이어.
    fn player_standing_at(x: f32, world: &World) -> Player {
        let mut p = Player::new(CharacterKind::Niro, Vec2::new(x, world.ground_y - PLAYER_HEIGHT));
        p.update(&no_input(), FRAME, world);
        assert!(p.on_ground);
        p
    }

    fn run(p: &mut Player, input: PlayerInput, frames: usize, world: &World) {
        for _ in 0..frames {
            p.update(&input, FRAME, world);
        }
    }

    #[test]
    fn standing_still_is_idle() {
        let w = flat_world();
        let mut p = player_standing_at(100.0, &w);
        let start = p.pos;
        run(&mut p, no_input(), 30, &w);
        assert_eq!(p.motion, Motion::Idle);
        assert_eq!(p.pos, start);
    }

    #[test]
    fn holding_right_walks_right() {
        let w = flat_world();
        let mut p = player_standing_at(100.0, &w);
        run(&mut p, hold_right(), 10, &w);
        assert!(p.pos.x > 100.0);
        assert_eq!(p.motion, Motion::Walk);
        assert_eq!(p.facing, Facing::Right);
    }

    #[test]
    fn holding_left_faces_left() {
        let w = flat_world();
        let mut p = player_standing_at(300.0, &w);
        run(&mut p, hold_left(), 10, &w);
        assert!(p.pos.x < 300.0);
        assert_eq!(p.facing, Facing::Left);
    }

    #[test]
    fn cannot_jump_in_the_air() {
        let w = flat_world();
        let mut p = player_standing_at(100.0, &w);
        p.update(&press_jump(), FRAME, &w);
        run(&mut p, no_input(), 5, &w);
        assert!(!p.on_ground);
        assert_eq!(p.motion, Motion::Jump);
        let vel_before = p.vel.y;
        p.update(&press_jump(), FRAME, &w);
        // 두 번째 점프가 됐다면 위로 가는 속도가 다시 커졌을 것 (vel.y가 더 작아짐).
        assert!(p.vel.y > vel_before, "공중에서 또 점프하면 안 된다");
    }

    #[test]
    fn jump_comes_back_to_ground() {
        let w = flat_world();
        let mut p = player_standing_at(100.0, &w);
        let ground_pos_y = p.pos.y;
        p.update(&press_jump(), FRAME, &w);
        assert!(p.pos.y < ground_pos_y, "점프하면 위로 올라간다");
        run(&mut p, no_input(), 120, &w); // 2초면 충분
        assert!(p.on_ground);
        assert_eq!(p.pos.y, ground_pos_y);
        assert_eq!(p.motion, Motion::Idle);
    }

    #[test]
    fn cannot_walk_through_box_from_side() {
        let w = World::m1();
        let box_ = w.obstacles[0];
        let mut p = player_standing_at(box_.x - 150.0, &w);
        run(&mut p, hold_right(), 120, &w);
        assert_eq!(p.pos.x + p.size.x, box_.x, "상자 왼쪽 면에서 멈춘다");
        assert!(p.on_ground);
    }

    #[test]
    fn can_stand_on_box() {
        let w = World::m1();
        let box_ = w.obstacles[0];
        // 상자 바로 위 공중에서 떨어뜨린다.
        let mut p = Player::new(CharacterKind::Ellie, Vec2::new(box_.x, box_.y - 200.0));
        run(&mut p, no_input(), 120, &w);
        assert!(p.on_ground);
        assert_eq!(p.pos.y + p.size.y, box_.y, "발이 상자 윗면에 있다");
        // 상자 위에서 걸어도 옆면에 막힌 것으로 오해하지 않는다.
        let x_before = p.pos.x;
        run(&mut p, hold_right(), 5, &w);
        assert!(p.pos.x > x_before);
    }

    #[test]
    fn can_jump_onto_tallest_box() {
        let w = World::m1();
        let box_ = w.obstacles[1]; // 가장 높은 상자
        let mut p = player_standing_at(box_.x - 100.0, &w);
        p.update(&PlayerInput { right: true, jump_pressed: true, ..Default::default() }, FRAME, &w);
        // 상자 위쯤 오면 멈춘다.
        for _ in 0..120 {
            let over_box = p.pos.x > box_.x && p.pos.x + p.size.x < box_.x + box_.w;
            let input = if over_box { no_input() } else { hold_right() };
            p.update(&input, FRAME, &w);
        }
        assert!(p.on_ground);
        assert_eq!(p.pos.y + p.size.y, box_.y);
    }

    #[test]
    fn head_bump_stops_going_up() {
        // 머리 위에 천장 같은 상자.
        let mut w = flat_world();
        w.obstacles.push(Rect::new(80.0, 440.0 - PLAYER_HEIGHT - 30.0, 100.0, 20.0));
        let ceiling = w.obstacles[0];
        let mut p = player_standing_at(100.0, &w);
        p.update(&press_jump(), FRAME, &w);
        for _ in 0..10 {
            p.update(&no_input(), FRAME, &w);
            assert!(p.pos.y >= ceiling.y + ceiling.h, "상자 아랫면을 뚫고 올라가지 않는다");
        }
        run(&mut p, no_input(), 60, &w);
        assert!(p.on_ground);
    }

    #[test]
    fn cannot_leave_world_left_edge() {
        let w = flat_world();
        let mut p = player_standing_at(20.0, &w);
        run(&mut p, hold_left(), 60, &w);
        assert_eq!(p.pos.x, 0.0);
    }

    #[test]
    fn cannot_leave_world_right_edge() {
        let w = flat_world();
        let mut p = player_standing_at(w.width - 80.0, &w);
        run(&mut p, hold_right(), 60, &w);
        assert_eq!(p.pos.x, w.width - p.size.x);
    }

    #[test]
    fn uneven_frame_time_keeps_standing_on_box_review_004() {
        // 0.0167초는 1/120초로 나눈 나머지가 아주 작다. 그 조각 때문에 공중으로 쳐지면 안 된다.
        let w = World::m1();
        let box_ = w.obstacles[0];
        let mut p = Player::new(CharacterKind::Niro, Vec2::new(box_.x, box_.y - PLAYER_HEIGHT));
        p.update(&no_input(), FRAME, &w);
        for _ in 0..100 {
            p.update(&no_input(), 0.0167, &w);
            assert!(p.on_ground);
            assert_eq!(p.motion, Motion::Idle);
        }
    }

    #[test]
    fn big_dt_does_not_fall_through_ground() {
        let w = flat_world();
        let mut p = Player::new(CharacterKind::Logan, Vec2::new(100.0, 0.0));
        for _ in 0..20 {
            p.update(&no_input(), 0.1, &w);
            assert!(p.pos.y + p.size.y <= w.ground_y);
        }
        assert!(p.on_ground);
        assert_eq!(p.pos.y + p.size.y, w.ground_y);
    }

    #[test]
    fn big_dt_does_not_pass_through_box() {
        let w = World::m1();
        let box_ = w.obstacles[0];
        let mut p = player_standing_at(box_.x - 60.0, &w);
        p.update(&hold_right(), 0.5, &w); // 한 번에 110픽셀 갈 시간
        assert_eq!(p.pos.x + p.size.x, box_.x);
        // 위에서 큰 dt로 떨어져도 상자를 뚫지 않는다.
        let mut q = Player::new(CharacterKind::Niro, Vec2::new(box_.x, 0.0));
        q.update(&no_input(), 1.0, &w);
        assert_eq!(q.pos.y + q.size.y, box_.y);
    }
}
