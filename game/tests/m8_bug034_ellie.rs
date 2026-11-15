//! BUG-034 엘리 시험: 18:00 낮 미션 실패 글을 보여 주는 2초 동안 주인공도 멈춰 있다.
//! (m8_missions의 bug034 시험은 시계·몬스터·퀴즈 차례를 본다. 여기서는 걷기·점프·공격 입력을 넣어 본다.)
//! 반복은 모두 횟수가 정해져 있다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::game::{Game, GameEvent};
use logic::player::PlayerInput;
use logic::world::World;

const FRAME: f32 = 1.0 / 60.0;

fn open_game() -> Game {
    let world = World { width: WORLD_WIDTH, ground_y: GROUND_Y, obstacles: vec![] };
    Game::on_world(CharacterKind::Niro, 3, world)
}

#[test]
fn bug034_player_stays_still_while_fail_text_shows() {
    let mut g = open_game();
    g.update(&PlayerInput::default(), g.clock.seconds_left_in_phase() - 1.0);
    g.combat.zombies.list.clear();
    g.combat.skeletons.clear();
    let mut reached = false;
    for _ in 0..(2.0 / FRAME) as usize {
        if g.update(&PlayerInput::default(), FRAME).contains(&GameEvent::NightStarted) {
            reached = true;
            break;
        }
    }
    assert!(reached, "2초 안에 18:00이 오지 않았다");
    assert!(g.is_waiting_to_open_quiz(), "낮 미션을 안 했으니 실패 글을 기다린다");

    let at = g.player.rect();
    let hp = g.combat.health.hp();
    let push = [
        PlayerInput { right: true, ..Default::default() },
        PlayerInput { left: true, jump_pressed: true, ..Default::default() },
        PlayerInput { attack_pressed: true, ..Default::default() },
    ];
    let frames = ((QUIZ_WAIT_AFTER_DAY_MISSION_FAIL_SECONDS - 0.1) / FRAME) as usize;
    for i in 0..frames {
        g.update(&push[i % push.len()], FRAME);
        assert!(g.is_waiting_to_open_quiz());
        assert_eq!(g.player.rect(), at, "실패 글 동안 주인공은 움직이지 않는다 ({i}프레임)");
    }
    assert_eq!(g.combat.health.hp(), hp);
    assert!(!g.math.is_open());
}
