//! M1 "캐릭터가 움직여요" 규칙 테스트 (엘리).
//! 기준: stories/backlog.md US-004 ~ US-008 완료 조건.
//!
//! 게임이 bin 크레이트라서 `use miss_game::...`를 쓸 수 없다.
//! 그래서 logic/와 config.rs 소스를 이 테스트 크레이트에 그대로 붙여 와서 검사한다.
//! (logic 안의 단위 테스트도 같이 한 번 더 돌지만 괜찮다.)

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)] // 이 테스트에서 안 쓰는 logic 함수(체력·번쩍임 등)가 있다
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::player::{Facing, Motion, Player, PlayerInput};
use logic::world::World;
use macroquad::math::{Rect, Vec2};

const FRAME: f32 = 1.0 / 60.0;

fn input(left: bool, right: bool, jump: bool) -> PlayerInput {
    PlayerInput { left, right, jump_pressed: jump, attack_pressed: false }
}
const NONE: PlayerInput = PlayerInput { left: false, right: false, jump_pressed: false, attack_pressed: false };
const RIGHT: PlayerInput = PlayerInput { left: false, right: true, jump_pressed: false, attack_pressed: false };
const LEFT: PlayerInput = PlayerInput { left: true, right: false, jump_pressed: false, attack_pressed: false };

/// main.rs의 start_play와 같은 시작 위치.
fn spawn(kind: CharacterKind, world: &World) -> Player {
    Player::new(kind, Vec2::new(120.0, world.ground_y - PLAYER_HEIGHT))
}

fn run(p: &mut Player, inp: PlayerInput, frames: usize, w: &World) {
    for _ in 0..frames {
        p.update(&inp, FRAME, w);
    }
}

fn overlaps(a: Rect, b: Rect) -> bool {
    a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h
}

/// 캐릭터가 지켜야 할 것: 화면 안, 땅 위, 상자 속에 안 들어감.
fn assert_legal(p: &Player, w: &World, what: &str) {
    let r = p.rect();
    assert!(r.x >= 0.0 && r.x + r.w <= w.width, "{what}: 화면 밖으로 나감 x={}", r.x);
    assert!(r.y + r.h <= w.ground_y + 0.001, "{what}: 땅 아래로 빠짐 y={}", r.y);
    assert!(r.y >= 0.0, "{what}: 머리가 화면 위로 나감 y={}", r.y);
    for b in &w.obstacles {
        assert!(!overlaps(r, *b), "{what}: 상자 속에 끼임 player={r:?} box={b:?}");
    }
}

/// 지금 서 있는 곳에서 한쪽으로 걸으며 점프해서 `target_x`(왼쪽 위 x) 너머까지 가 본다.
fn jump_across(p: &mut Player, w: &World, go_right: bool, target_x: f32) -> bool {
    let dir = if go_right { RIGHT } else { LEFT };
    for _ in 0..600 {
        let jump = p.on_ground;
        p.update(&PlayerInput { jump_pressed: jump, ..dir }, FRAME, w);
        assert_legal(p, w, "상자 넘기");
        let passed = if go_right { p.pos.x >= target_x } else { p.pos.x <= target_x };
        if passed && p.on_ground {
            return true;
        }
    }
    false
}

// ---------- US-007 맵 ----------

#[test]
fn m1_map_has_ground_and_at_least_two_boxes_on_it() {
    let w = World::m1();
    assert!(w.obstacles.len() >= 2, "상자가 2개 이상");
    for b in &w.obstacles {
        assert_eq!(b.y + b.h, w.ground_y, "상자는 땅 위에 놓여 있다");
        assert!(b.x >= 0.0 && b.x + b.w <= w.width, "상자는 화면 안");
    }
    // 시작 위치가 상자와 겹치지 않는다.
    let p = spawn(CharacterKind::Niro, &w);
    for b in &w.obstacles {
        assert!(!overlaps(p.rect(), *b));
    }
}

#[test]
fn jump_is_high_enough_for_every_box_with_room_to_spare() {
    // 최고 높이 = v² / 2g. 8살 아이가 넘기 쉽도록 가장 높은 상자보다 20픽셀 이상 여유.
    let peak = PLAYER_JUMP_SPEED * PLAYER_JUMP_SPEED / (2.0 * GRAVITY);
    let tallest = World::m1().obstacles.iter().map(|b| b.h).fold(0.0, f32::max);
    assert!(peak >= tallest + 20.0, "점프 높이 {peak} / 가장 높은 상자 {tallest}");
}

// ---------- US-006 대기 / US-004 걷기 ----------

#[test]
fn spawn_lands_on_ground_and_idles() {
    let w = World::m1();
    for kind in CharacterKind::ALL {
        let mut p = spawn(kind, &w);
        run(&mut p, NONE, 60, &w);
        assert!(p.on_ground);
        assert_eq!(p.motion, Motion::Idle);
        assert_eq!(p.pos.y + p.size.y, w.ground_y);
        assert_eq!(p.kind, kind, "고른 캐릭터 그대로");
    }
}

#[test]
fn releasing_arrow_stops_and_keeps_facing() {
    let w = World::m1();
    let mut p = spawn(CharacterKind::Niro, &w);
    run(&mut p, RIGHT, 10, &w);
    assert_eq!(p.motion, Motion::Walk);
    run(&mut p, NONE, 1, &w);
    let x = p.pos.x;
    run(&mut p, NONE, 60, &w);
    assert_eq!(p.pos.x, x, "손을 떼면 멈춘다 (미끄러지지 않음)");
    assert_eq!(p.motion, Motion::Idle);
    assert_eq!(p.facing, Facing::Right);

    run(&mut p, LEFT, 10, &w);
    assert_eq!(p.facing, Facing::Left);
    run(&mut p, NONE, 60, &w);
    assert_eq!(p.facing, Facing::Left, "멈춘 뒤에도 왼쪽을 본다");
    assert_eq!(p.motion, Motion::Idle);
}

#[test]
fn both_arrows_together_do_not_jitter() {
    let w = World::m1();
    let mut p = spawn(CharacterKind::Ellie, &w);
    run(&mut p, NONE, 30, &w);
    run(&mut p, LEFT, 5, &w);
    let x = p.pos.x;
    for _ in 0..60 {
        p.update(&input(true, true, false), FRAME, &w);
        assert_eq!(p.pos.x, x, "둘 다 누르면 제자리");
        assert_eq!(p.facing, Facing::Left, "바라보는 쪽이 왔다 갔다 하지 않는다");
    }
}

// ---------- US-005 점프 ----------

#[test]
fn holding_jump_every_frame_only_jumps_when_on_ground() {
    // 입력 쪽이 실수로 "꾹 누름"을 매 프레임 jump_pressed=true로 줘도 하늘로 날아가지 않는다.
    let w = World::m1();
    let mut p = spawn(CharacterKind::Logan, &w);
    run(&mut p, NONE, 30, &w);
    let ground_top = p.pos.y;
    let mut highest = p.pos.y;
    for _ in 0..600 {
        p.update(&input(false, false, true), FRAME, &w);
        highest = highest.min(p.pos.y);
        assert_legal(&p, &w, "점프 꾹 누름");
    }
    let peak = PLAYER_JUMP_SPEED * PLAYER_JUMP_SPEED / (2.0 * GRAVITY);
    assert!(ground_top - highest <= peak + 1.0, "한 번 점프 높이보다 높이 올라가지 않는다");
}

#[test]
fn up_and_space_are_the_same_jump_in_logic() {
    // 로직에는 jump_pressed 하나뿐 → Space/↑ 차이는 입력(view) 쪽 책임. 높이가 정해진 값인지만 본다.
    let w = World::m1();
    let mut p = spawn(CharacterKind::Niro, &w);
    run(&mut p, NONE, 30, &w);
    let start = p.pos.y;
    p.update(&input(false, false, true), FRAME, &w);
    assert_eq!(p.motion, Motion::Jump);
    let mut top = start;
    for _ in 0..120 {
        p.update(&NONE, FRAME, &w);
        top = top.min(p.pos.y);
    }
    let peak = PLAYER_JUMP_SPEED * PLAYER_JUMP_SPEED / (2.0 * GRAVITY);
    assert!((start - top - peak).abs() < 8.0, "점프 높이 {} (기대 약 {peak})", start - top);
    assert!(p.on_ground);
    assert_eq!(p.motion, Motion::Idle, "내려오면 대기 동작");
}

#[test]
fn can_turn_around_in_the_air() {
    let w = World::m1();
    let mut p = spawn(CharacterKind::Ellie, &w);
    run(&mut p, NONE, 30, &w);
    p.update(&input(false, true, true), FRAME, &w);
    assert_eq!(p.facing, Facing::Right);
    run(&mut p, LEFT, 5, &w);
    assert!(!p.on_ground);
    assert_eq!(p.facing, Facing::Left, "공중에서 ← 누르면 왼쪽을 본다");
}

#[test]
fn head_never_leaves_screen_even_jumping_from_tallest_box() {
    let w = World::m1();
    let tall = w.obstacles.iter().copied().fold(w.obstacles[0], |a, b| if b.h > a.h { b } else { a });
    let mut p = Player::new(CharacterKind::Niro, Vec2::new(tall.x, tall.y - PLAYER_HEIGHT - 1.0));
    run(&mut p, NONE, 30, &w);
    assert!(p.on_ground);
    p.update(&input(false, false, true), FRAME, &w);
    for _ in 0..120 {
        p.update(&NONE, FRAME, &w);
        // 그림은 충돌 네모보다 크다(84 vs 64, 점프 때 14% 늘어남). 그림 머리 기준으로 본다.
        let sprite_top = p.pos.y + p.size.y - 84.0 * 1.14;
        assert!(sprite_top >= 0.0, "그림 머리가 화면 위로 잘림 (y={sprite_top})");
    }
}

// ---------- US-007 상자 ----------

#[test]
fn every_box_blocks_from_both_sides() {
    let w = World::m1();
    for (i, b) in w.obstacles.iter().enumerate() {
        // 왼쪽에서
        let mut p = Player::new(CharacterKind::Niro, Vec2::new(b.x - 100.0, w.ground_y - PLAYER_HEIGHT));
        run(&mut p, RIGHT, 120, &w);
        assert_eq!(p.pos.x + p.size.x, b.x, "상자 {i} 왼쪽 면에서 멈춤");
        assert!(p.on_ground);
        // 오른쪽에서
        let mut q = Player::new(CharacterKind::Niro, Vec2::new(b.x + b.w + 50.0, w.ground_y - PLAYER_HEIGHT));
        run(&mut q, LEFT, 120, &w);
        assert_eq!(q.pos.x, b.x + b.w, "상자 {i} 오른쪽 면에서 멈춤");
        assert!(q.on_ground);
    }
}

#[test]
fn can_jump_over_all_boxes_left_to_right_and_back() {
    let w = World::m1();
    let mut p = spawn(CharacterKind::Logan, &w);
    run(&mut p, NONE, 30, &w);
    for b in &w.obstacles {
        assert!(jump_across(&mut p, &w, true, b.x + b.w), "상자 x={}를 오른쪽으로 넘기", b.x);
    }
    for b in w.obstacles.iter().rev() {
        assert!(jump_across(&mut p, &w, false, b.x - PLAYER_WIDTH), "상자 x={}를 왼쪽으로 넘기", b.x);
    }
}

#[test]
fn stand_walk_jump_idle_on_every_box_then_walk_off() {
    let w = World::m1();
    for (i, b) in w.obstacles.iter().enumerate() {
        let mut p = Player::new(CharacterKind::Ellie, Vec2::new(b.x + (b.w - PLAYER_WIDTH) / 2.0, b.y - 150.0));
        run(&mut p, NONE, 90, &w);
        assert!(p.on_ground && p.pos.y + p.size.y == b.y, "상자 {i} 위에 선다");
        assert_eq!(p.motion, Motion::Idle);
        // 위에서 점프하고 다시 상자 위로
        p.update(&input(false, false, true), FRAME, &w);
        run(&mut p, NONE, 120, &w);
        assert!(p.on_ground && p.pos.y + p.size.y == b.y, "상자 {i} 위에서 점프 후 다시 상자 위");
        // 오른쪽 끝을 지나면 떨어진다
        run(&mut p, RIGHT, 60, &w);
        run(&mut p, NONE, 60, &w);
        assert_eq!(p.pos.y + p.size.y, w.ground_y, "상자 {i}에서 걸어 내려오면 땅에 닿는다");
    }
}

#[test]
fn jumping_while_pushing_into_box_never_gets_stuck_inside() {
    let w = World::m1();
    for b in &w.obstacles {
        for start_offset in [1.0_f32, 5.0, 20.0, 47.0] {
            let mut p = Player::new(CharacterKind::Niro, Vec2::new(b.x - PLAYER_WIDTH - start_offset, w.ground_y - PLAYER_HEIGHT));
            for f in 0..240 {
                // 박치기하듯 계속 밀면서 가끔 점프
                p.update(&input(false, true, f % 37 == 0), FRAME, &w);
                assert_legal(&p, &w, "상자에 밀면서 점프");
            }
        }
    }
}

// ---------- BUG-006 상자 끝에서 중심 잡기 ----------

/// 몸 가운데 x가 `center_x`가 되게 상자 조금 위에서 떨어뜨린다.
fn drop_with_center_at(center_x: f32, b: Rect) -> Player {
    Player::new(CharacterKind::Niro, Vec2::new(center_x - PLAYER_WIDTH / 2.0, b.y - PLAYER_HEIGHT - 40.0))
}

/// 2초 동안 안 끼는지 보며 자리 잡게 한 뒤, 3초 더 지나도 위치가 똑같은지(떨림 없음) 확인한다.
fn settle_and_assert_still(p: &mut Player, w: &World, what: &str) {
    for _ in 0..120 {
        p.update(&NONE, FRAME, w);
        assert_legal(p, w, what);
    }
    let settled = p.pos;
    for _ in 0..180 {
        p.update(&NONE, FRAME, w);
        assert_eq!(p.pos, settled, "{what}: 자리 잡은 뒤 떨림");
        assert!(p.on_ground, "{what}: 자리 잡은 뒤 공중");
    }
}

/// 상자 왼쪽 또는 오른쪽 땅에, 상자에 끼지 않고 내려왔는지.
fn assert_on_ground_beside(p: &Player, b: Rect, w: &World, left_side: bool) {
    assert_eq!(p.pos.y + p.size.y, w.ground_y, "땅까지 떨어진다");
    if left_side {
        assert!(p.pos.x + p.size.x <= b.x, "상자 왼쪽 바깥 x={}", p.pos.x);
    } else {
        assert!(p.pos.x >= b.x + b.w, "상자 오른쪽 바깥 x={}", p.pos.x);
    }
}

#[test]
fn center_over_box_stands_even_exactly_on_the_edge() {
    // 사용자 예시: 가운데가 상자 끝 바로 위면 한 발이 밖이어도 중심을 잡고 서 있다.
    let w = World::m1();
    for b in &w.obstacles {
        for center_x in [b.x, b.x + 1.0, b.x + b.w / 2.0, b.x + b.w - 1.0, b.x + b.w] {
            let mut p = drop_with_center_at(center_x, *b);
            settle_and_assert_still(&mut p, &w, "가운데가 상자 위");
            assert_eq!(p.pos.y + p.size.y, b.y, "가운데 {center_x}: 상자 위에 선다");
            assert_eq!(p.pos.x + PLAYER_WIDTH / 2.0, center_x, "가운데 {center_x}: 옆으로 안 밀린다");
        }
    }
}

#[test]
fn center_off_box_falls_to_ground_on_both_sides() {
    // 네모 끝만 걸린 경우부터 가운데가 끝을 살짝 벗어난 경우까지.
    let w = World::m1();
    for b in &w.obstacles {
        for off in [0.5_f32, 1.0, 10.0, PLAYER_WIDTH / 2.0 - 1.0] {
            let mut left = drop_with_center_at(b.x - off, *b);
            settle_and_assert_still(&mut left, &w, "왼쪽 끝 착지");
            assert_on_ground_beside(&left, *b, &w, true);

            let mut right = drop_with_center_at(b.x + b.w + off, *b);
            settle_and_assert_still(&mut right, &w, "오른쪽 끝 착지");
            assert_on_ground_beside(&right, *b, &w, false);
        }
    }
}

#[test]
fn walking_past_the_edge_then_stopping_still_falls() {
    let w = World::m1();
    for b in &w.obstacles {
        for (dir, left_side) in [(LEFT, true), (RIGHT, false)] {
            let mut p = drop_with_center_at(b.x + b.w / 2.0, *b);
            run(&mut p, NONE, 60, &w);
            // 가운데가 상자 밖으로 나가는 순간 손을 뗀다.
            for _ in 0..60 {
                let center_x = p.pos.x + PLAYER_WIDTH / 2.0;
                if center_x < b.x || center_x > b.x + b.w {
                    break;
                }
                p.update(&dir, FRAME, &w);
            }
            settle_and_assert_still(&mut p, &w, "걸어서 끝을 지남");
            assert_on_ground_beside(&p, *b, &w, left_side);
        }
    }
}

#[test]
fn pushing_back_toward_box_while_slipping_never_gets_stuck() {
    // 미끄러지는 동안 상자 쪽으로 계속 밀어도 공중에 멈추지 않고, 상자 위로 올라선다.
    let w = World::m1();
    for b in &w.obstacles {
        for (center_x, dir) in [(b.x - 5.0, RIGHT), (b.x + b.w + 5.0, LEFT)] {
            let mut p = drop_with_center_at(center_x, *b);
            // 발이 상자 윗면에 닿아 미끄러지기 시작할 때까지 기다린다.
            let mut frames = 0;
            while p.pos.y + p.size.y < b.y {
                p.update(&NONE, FRAME, &w);
                frames += 1;
                assert!(frames < 600, "10초 안에 상자 윗면까지 떨어져야 한다");
            }
            assert!(!p.on_ground, "가운데가 밖이면 서 있지 못한다");
            for _ in 0..10 {
                p.update(&dir, FRAME, &w);
                assert_legal(&p, &w, "미끄러지며 상자 쪽으로 밀기");
            }
            assert!(p.on_ground, "공중에 멈추지 않는다");
            assert_eq!(p.pos.y + p.size.y, b.y);
        }
    }
}

#[test]
fn big_dt_edge_landing_does_not_enter_box() {
    let w = World::m1();
    let b = w.obstacles[0];
    let mut p = drop_with_center_at(b.x - 10.0, b);
    for _ in 0..8 {
        p.update(&NONE, 0.25, &w);
        assert_legal(&p, &w, "느린 프레임 끝 착지");
    }
    assert_on_ground_beside(&p, b, &w, true);
}

// ---------- US-008 화면 밖 ----------

#[test]
fn jumping_at_edges_with_outward_arrow_stays_inside() {
    let w = World::m1();
    let mut p = spawn(CharacterKind::Niro, &w);
    run(&mut p, LEFT, 120, &w);
    assert_eq!(p.pos.x, 0.0);
    for f in 0..300 {
        p.update(&input(true, false, f % 20 == 0), FRAME, &w);
        assert_legal(&p, &w, "왼쪽 끝 점프");
    }
    // 오른쪽 끝까지 (상자는 점프로 넘는다)
    assert!(jump_across(&mut p, &w, true, w.width - PLAYER_WIDTH));
    for f in 0..300 {
        p.update(&input(false, true, f % 20 == 0), FRAME, &w);
        assert_legal(&p, &w, "오른쪽 끝 점프");
    }
    assert_eq!(p.pos.x, w.width - p.size.x);
}

// ---------- 느린 기기 / 숨은 탭 ----------

#[test]
fn slow_frames_still_respect_all_rules() {
    // 브라우저 탭이 느려서 한 프레임이 0.05~0.5초가 돼도 상자·땅·화면 끝을 뚫지 않는다.
    let w = World::m1();
    let mut p = spawn(CharacterKind::Logan, &w);
    let mut seed: u32 = 12345;
    let mut rnd = || {
        seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        (seed >> 16) & 0x7fff
    };
    for _ in 0..5000 {
        let r = rnd();
        let inp = input(r & 1 != 0, r & 2 != 0, r % 7 == 0);
        let dt = [FRAME, 0.05, 0.1, 0.25, 0.5][(r % 5) as usize];
        p.update(&inp, dt, &w);
        assert_legal(&p, &w, "랜덤 입력 + 느린 프레임");
    }
}

#[test]
fn slow_frames_can_still_jump_onto_a_box() {
    // 프레임이 0.25초씩이어도 상자 위로 올라설 수 있다 (가장 높은 상자).
    let w = World::m1();
    let tall = w.obstacles.iter().copied().fold(w.obstacles[0], |a, b| if b.h > a.h { b } else { a });
    let mut p = Player::new(CharacterKind::Niro, Vec2::new(tall.x - PLAYER_WIDTH, w.ground_y - PLAYER_HEIGHT));
    p.update(&NONE, 0.25, &w);
    p.update(&input(false, true, true), 0.25, &w);
    // 상자보다 높이 올라갈 때까지 옆면에 막혀서 첫 0.25초로는 가운데가 상자 위까지 못 간다 (BUG-006).
    p.update(&RIGHT, 0.25, &w);
    for _ in 0..8 {
        p.update(&NONE, 0.25, &w);
    }
    assert_legal(&p, &w, "느린 프레임 점프");
    assert!(p.on_ground);
    assert_eq!(p.pos.y + p.size.y, tall.y, "가운데가 상자 위까지 가면 올라선다");
}
