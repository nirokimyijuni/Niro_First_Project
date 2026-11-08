//! 엘리 완료 조건 시험 (D70, US-022): 등 뒤도 100 안이면 캐고, 가까운 것 하나만 쓴다.
//! 나무 셋째(TREE_XS[2])와 제작대(WORKBENCH_X) 사이에서 잰다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::game::{Game, Usable};
use logic::player::{Facing, PlayerInput};
use macroquad::math::Vec2;

fn stand_at(g: &mut Game, center_x: f32, facing: Facing) {
    g.player.pos = Vec2::new(center_x - PLAYER_WIDTH / 2.0, GROUND_Y - PLAYER_HEIGHT);
    g.player.vel = Vec2::ZERO;
    g.player.facing = facing;
    g.update(&PlayerInput::default(), 0.0);
}

fn usable(center_x: f32, facing: Facing) -> Option<Usable> {
    let mut g = Game::new(CharacterKind::Niro, 7);
    stand_at(&mut g, center_x, facing);
    assert!((g.player.rect().center().x - center_x).abs() < 0.01, "선 자리가 밀렸다 {center_x}");
    g.usable_in_reach()
}

#[test]
fn tree_behind_reaches_at_99_not_101() {
    // 셋째 나무 왼쪽 끝에서 몸 오른쪽 끝까지 gap. 왼쪽을 보면 나무는 등 뒤.
    let tree_left = TREE_XS[2] - TREE_WIDTH / 2.0;
    for (gap, want) in [(99.0, Some(Usable::Tree(2))), (101.0, None)] {
        let center = tree_left - gap - PLAYER_WIDTH / 2.0;
        assert_eq!(usable(center, Facing::Left), want, "등 뒤 {gap}");
        assert_eq!(usable(center, Facing::Right), want, "앞 {gap}");
    }
}

#[test]
fn between_third_tree_and_workbench_uses_the_nearer_one() {
    let mid = (TREE_XS[2] + WORKBENCH_X) / 2.0;
    for facing in [Facing::Left, Facing::Right] {
        assert_eq!(usable(mid - 5.0, facing), Some(Usable::Tree(2)), "나무 쪽 {facing:?}");
        assert_eq!(usable(mid + 5.0, facing), Some(Usable::Workbench), "제작대 쪽 {facing:?}");
    }
    // 같은 거리면 바라보는 쪽.
    assert_eq!(usable(mid, Facing::Left), Some(Usable::Tree(2)));
    assert_eq!(usable(mid, Facing::Right), Some(Usable::Workbench));
}
