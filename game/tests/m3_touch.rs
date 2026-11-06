//! M7 앞당기기 폰 터치 조작(US-032~US-034) 판정 테스트 (엘리).
//! 기준: stories/backlog.md US-032~US-034, docs/02-plan.md D16, qa/reviews/REVIEW-003.
//! `view/touch.rs`는 bin 쪽이라 그대로 못 부른다 → 그 파일만 #[path]로 가져오고,
//! 그림 쪽 이웃(hud·screen·text)은 아래에 아무 일도 안 하는 작은 대역으로 둔다. (본체 코드는 안 바꿈)
//! 좌표는 모두 가상 화면(960×540).

#[allow(dead_code)]
mod view {
    pub mod screen {
        pub const VIRTUAL_WIDTH: f32 = 960.0;
        pub const VIRTUAL_HEIGHT: f32 = 540.0;
    }
    pub mod hud {
        use macroquad::prelude::Color;
        pub fn draw_pill(_x: f32, _y: f32, _w: f32, _h: f32, _color: Color) {}
    }
    pub mod text {
        use macroquad::prelude::Color;
        pub struct TextPainter;
        impl TextPainter {
            pub fn centered(&self, _t: &str, _x: f32, _y: f32, _size: f32, _c: Color) {}
        }
    }
    #[path = "../../src/view/touch.rs"]
    pub mod touch;
}

use macroquad::prelude::*;
use view::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use view::touch::*;

fn f(id: u64, phase: TouchPhase, x: f32, y: f32) -> Finger {
    Finger { id, phase, pos: vec2(x, y) }
}

fn jump_center() -> Vec2 {
    PLAY_BUTTONS[0].center
}

/// 같은 자리에 엄지를 대고 `frames` 프레임 동안 가만히 있을 때, 한 번이라도 걸었는지.
fn walks_while_resting(x: f32, y: f32, frames: usize) -> bool {
    let mut t = TouchControls::new();
    let mut walked = false;
    let first = t.update(&[f(1, TouchPhase::Started, x, y)], &PLAY_BUTTONS, true);
    walked |= first.left || first.right;
    for _ in 0..frames {
        let o = t.update(&[f(1, TouchPhase::Stationary, x, y)], &PLAY_BUTTONS, true);
        walked |= o.left || o.right;
    }
    walked
}

#[test]
fn us032_resting_thumb_in_middle_of_left_half_does_not_walk() {
    // 왼쪽 절반 한가운데, 위, 아래에 대고 가만히 → 안 걷는다
    for (x, y) in [(240.0, 270.0), (240.0, 120.0), (300.0, 420.0), (470.0, 270.0)] {
        assert!(!walks_while_resting(x, y, 30), "({x}, {y})에 가만히 대었는데 걸음");
    }
}

#[test]
fn us032_small_wiggle_inside_deadzone_does_not_walk() {
    let mut t = TouchControls::new();
    t.update(&[f(1, TouchPhase::Started, 240.0, 300.0)], &PLAY_BUTTONS, true);
    let dead = JOYSTICK_RADIUS * JOYSTICK_DEADZONE;
    for i in 0..60 {
        let dx = if i % 2 == 0 { dead * 0.9 } else { -dead * 0.9 };
        let o = t.update(&[f(1, TouchPhase::Moved, 240.0 + dx, 300.0 + (i % 3) as f32 * 4.0)], &PLAY_BUTTONS, true);
        assert!(!o.left && !o.right, "데드존 안 떨림({dx})에 걸음");
    }
}

#[test]
fn us032_push_far_or_little_gives_same_direction_flag() {
    // 키보드처럼 true/false만 나가므로 빠르기가 같다 (조금 밀기 vs 화면 끝까지 밀기)
    for push in [JOYSTICK_RADIUS * 0.4, JOYSTICK_RADIUS, JOYSTICK_RADIUS * 5.0] {
        let mut t = TouchControls::new();
        t.update(&[f(1, TouchPhase::Started, 240.0, 300.0)], &PLAY_BUTTONS, true);
        let o = t.update(&[f(1, TouchPhase::Moved, 240.0 + push, 300.0)], &PLAY_BUTTONS, true);
        assert!(o.right && !o.left, "오른쪽 {push} 밀기");
        let o = t.update(&[f(1, TouchPhase::Moved, 240.0 - push, 300.0)], &PLAY_BUTTONS, true);
        assert!(o.left && !o.right, "왼쪽 {push} 밀기");
    }
}

#[test]
fn us032_push_up_or_down_never_walks_or_jumps() {
    let mut t = TouchControls::new();
    t.update(&[f(1, TouchPhase::Started, 240.0, 300.0)], &PLAY_BUTTONS, true);
    for y in [0.0, 100.0, 200.0, 400.0, 540.0] {
        let o = t.update(&[f(1, TouchPhase::Moved, 240.0, y)], &PLAY_BUTTONS, true);
        assert_eq!(o, TouchFrame::default(), "위·아래({y}) 밀기");
    }
}

#[test]
fn us032_joystick_keeps_working_in_right_half_even_over_jump_button() {
    let mut t = TouchControls::new();
    t.update(&[f(1, TouchPhase::Started, 300.0, 400.0)], &PLAY_BUTTONS, true);
    let j = jump_center();
    let o = t.update(&[f(1, TouchPhase::Moved, j.x, j.y)], &PLAY_BUTTONS, true);
    assert!(o.right && !o.jump_pressed, "조이스틱 손가락이 점프 버튼 위로 가도 점프가 아님");
    let o = t.update(&[f(1, TouchPhase::Ended, j.x, j.y)], &PLAY_BUTTONS, true);
    assert!(!o.right && t.joystick.is_none(), "떼면 멈추고 조이스틱이 사라짐");
}

#[test]
fn us033_two_fingers_walk_and_jump_and_lifting_one_keeps_other() {
    let mut t = TouchControls::new();
    let j = jump_center();
    t.update(&[f(1, TouchPhase::Started, 200.0, 400.0)], &PLAY_BUTTONS, true);
    t.update(&[f(1, TouchPhase::Moved, 300.0, 400.0)], &PLAY_BUTTONS, true);
    let o = t.update(
        &[f(1, TouchPhase::Stationary, 300.0, 400.0), f(2, TouchPhase::Started, j.x, j.y)],
        &PLAY_BUTTONS,
        true,
    );
    assert!(o.right && o.jump_pressed, "걷기 + 점프 같은 프레임");
    // 점프 손가락만 뗌 → 계속 걷기
    let o = t.update(
        &[f(1, TouchPhase::Stationary, 300.0, 400.0), f(2, TouchPhase::Ended, j.x, j.y)],
        &PLAY_BUTTONS,
        true,
    );
    assert!(o.right && !o.jump_pressed);
    // 다시 점프를 누른 채 조이스틱만 뗌 → 점프 버튼은 계속 눌림
    t.update(
        &[f(1, TouchPhase::Stationary, 300.0, 400.0), f(3, TouchPhase::Started, j.x, j.y)],
        &PLAY_BUTTONS,
        true,
    );
    let o = t.update(
        &[f(1, TouchPhase::Ended, 300.0, 400.0), f(3, TouchPhase::Stationary, j.x, j.y)],
        &PLAY_BUTTONS,
        true,
    );
    assert!(!o.right && !o.left && !o.jump_pressed);
    assert!(t.is_held(ButtonKind::Jump), "조이스틱을 떼도 점프 버튼은 눌린 채");
}

#[test]
fn us033_jump_button_is_big_and_away_from_edges() {
    let b = PLAY_BUTTONS[0];
    assert!(b.radius * 2.0 >= 110.0, "엄지로 누르기 쉬운 크기");
    assert!(b.center.x + b.radius <= VIRTUAL_WIDTH - 10.0);
    assert!(b.center.y + b.radius <= VIRTUAL_HEIGHT - 10.0);
    assert!(b.center.x > VIRTUAL_WIDTH / 2.0, "오른쪽");
    assert!(b.center.y > VIRTUAL_HEIGHT / 2.0, "아래쪽");
}

#[test]
fn us034_back_button_only_on_first_press_not_when_joystick_passes() {
    let mut t = TouchControls::new();
    let back = BACK_BUTTON.center();
    t.update(&[f(1, TouchPhase::Started, 300.0, 300.0)], &PLAY_BUTTONS, true);
    let o = t.update(&[f(1, TouchPhase::Moved, back.x, back.y)], &PLAY_BUTTONS, true);
    assert!(!o.back_pressed, "조이스틱 엄지가 고르기 버튼 위로 지나가도 안 돌아감");
    t.update(&[f(1, TouchPhase::Ended, back.x, back.y)], &PLAY_BUTTONS, true);
    let o = t.update(&[f(2, TouchPhase::Started, back.x, back.y)], &PLAY_BUTTONS, true);
    assert!(o.back_pressed && t.joystick.is_none(), "처음 누른 곳이 버튼이면 돌아가기, 조이스틱은 안 생김");
}

/// D25: 오른쪽 절반은 어디든 새 손가락이면 점프. 왼쪽 절반과 "고르기" 버튼은 그대로.
#[test]
fn d25_anywhere_on_right_half_jumps_once() {
    for (x, y) in [(VIRTUAL_WIDTH / 2.0, 270.0), (600.0, 60.0), (700.0, 300.0), (950.0, 530.0)] {
        let mut t = TouchControls::new();
        let o = t.update(&[f(1, TouchPhase::Started, x, y)], &PLAY_BUTTONS, true);
        assert!(o.jump_pressed && t.joystick.is_none(), "({x}, {y})");
        for _ in 0..10 {
            let o = t.update(&[f(1, TouchPhase::Stationary, x, y)], &PLAY_BUTTONS, true);
            assert!(!o.jump_pressed, "꾹 눌러도 다시 점프하지 않는다");
        }
    }
    let mut t = TouchControls::new();
    let o = t.update(&[f(1, TouchPhase::Started, VIRTUAL_WIDTH / 2.0 - 1.0, 270.0)], &PLAY_BUTTONS, true);
    assert!(!o.jump_pressed && t.joystick.is_some(), "왼쪽 절반은 조이스틱");
}

/// BUG-005: 왼쪽 끝 가까이(받침 반지름 - 데드존 안쪽, 가상 x < 약 64)에 엄지를 **가만히** 대기만 하면
/// 받침이 화면 안으로 당겨져도 걷지 않는다.
#[test]
fn bug005_resting_thumb_near_left_edge_does_not_walk() {
    for (x, y) in [(-50.0, 520.0), (0.0, 300.0), (30.0, 300.0), (60.0, 450.0)] {
        assert!(!walks_while_resting(x, y, 30), "({x}, {y})에 가만히 대었는데 걸음");
    }
}
