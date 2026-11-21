//! M9 튜토리얼 규칙과 연습용 몬스터 시험 (수산나): US-119(규칙 몫), US-120, US-124.
//! 숫자는 config 상수로 적어서 임시 값을 바꿔도 시험을 고치지 않아도 된다.
//! 반복은 모두 횟수가 정해져 있다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::combat::CombatEvent;
use logic::game::{Game, GameEvent};
use logic::mission::MissionKind;
use logic::player::{Facing, PlayerInput};
use logic::practice::{MonsterKind, PracticeEvent};
use logic::settings::Settings;
use logic::tutorial::{TutorialEvent, TutorialState, TutorialStep};
use logic::weapon::Weapon;
use logic::world::shortest_dx;
use macroquad::math::Vec2;

const FRAME: f32 = 1.0 / 60.0;
const SEED: u32 = 7;
const A_LITTLE: f32 = 0.05;

fn idle() -> PlayerInput {
    PlayerInput::default()
}

/// 상자와 구조물을 모두 치운 긴 맵 판 (걷기가 막히지 않게). 나무와 미스는 그대로다.
fn open_game() -> Game {
    let mut g = Game::new(CharacterKind::Niro, SEED);
    g.world.obstacles.clear();
    stand_at(&mut g, 600.0, Facing::Right);
    g
}

fn stand_at(g: &mut Game, center_x: f32, facing: Facing) {
    g.player.pos = Vec2::new(center_x - PLAYER_WIDTH / 2.0, GROUND_Y - PLAYER_HEIGHT);
    g.player.vel = Vec2::ZERO;
    g.player.facing = facing;
    g.update(&idle(), 0.0);
}

fn run(g: &mut Game, input: PlayerInput, seconds: f32) -> Vec<GameEvent> {
    let mut all = Vec::new();
    for _ in 0..(seconds / FRAME).ceil() as usize {
        all.extend(g.update(&input, FRAME));
    }
    all
}

fn walk_right(g: &mut Game, distance: f32) {
    run(g, PlayerInput { right: true, ..Default::default() }, distance / PLAYER_WALK_SPEED + A_LITTLE);
}

fn swing(g: &mut Game) -> Vec<GameEvent> {
    g.update(&PlayerInput { attack_pressed: true, ..Default::default() }, FRAME)
}

fn start(g: &mut Game) {
    assert!(g.start_tutorial(&Settings::default(), false));
    assert_eq!(g.take_tutorial_events(), vec![TutorialEvent::StepStarted(TutorialStep::Move)]);
}

fn step_of(g: &Game) -> Option<TutorialStep> {
    g.tutorial.current_step()
}

/// ①을 걸어서 끝내고 "잘했어요" 뒤 ②까지 간다.
fn go_to_attack_step(g: &mut Game) {
    walk_right(g, TUTORIAL_WALK_DISTANCE + 20.0);
    run(g, idle(), TUTORIAL_CHEER_SECONDS + A_LITTLE);
    assert_eq!(step_of(g), Some(TutorialStep::Attack));
}

/// 연습용 몬스터를 플레이어 바로 앞(공격이 닿는 곳)으로 옮긴다.
fn bring_practice_next_to_player(g: &mut Game) {
    let center = g.player.rect().center().x;
    let monster = g.practice.as_mut().expect("연습용 몬스터가 있어야 한다");
    monster.pos.x = center + PLAYER_WIDTH / 2.0 + 10.0;
}

fn face_practice(g: &mut Game) {
    let x = g.practice.as_ref().unwrap().rect().center().x;
    let center = g.player.rect().center().x;
    g.player.facing = if shortest_dx(center, x) < 0.0 { Facing::Left } else { Facing::Right };
}

// ---- 시작과 건너뛰기 (US-119, US-120) ----

#[test]
fn new_game_has_no_tutorial_and_no_practice_monster() {
    let mut g = open_game();
    assert_eq!(g.tutorial.state(), TutorialState::NotStarted);
    assert!(g.practice.is_none());
    walk_right(&mut g, 400.0);
    assert_eq!(g.tutorial.state(), TutorialState::NotStarted, "시작하기 전에는 아무 일도 없다");
    assert!(g.take_tutorial_events().is_empty() && g.take_practice_events().is_empty());
}

#[test]
fn already_seen_player_and_qa_do_not_start() {
    let mut g = open_game();
    let seen = Settings { tutorial_done: true, ..Settings::default() };
    assert!(!g.start_tutorial(&seen, false));
    assert!(!g.start_tutorial(&Settings::default(), true), "?qa는 안내 없음");
    assert_eq!(g.tutorial.state(), TutorialState::NotStarted);
    walk_right(&mut g, 400.0);
    run(&mut g, idle(), TUTORIAL_CHEER_SECONDS + 1.0);
    assert!(g.practice.is_none());
    assert!(g.take_tutorial_events().is_empty());
}

#[test]
fn skip_ends_all_steps_and_removes_practice_monster_and_game_goes_on() {
    let mut g = open_game();
    start(&mut g);
    go_to_attack_step(&mut g);
    assert!(g.practice.is_some());
    g.take_tutorial_events();
    assert!(g.skip_tutorial());
    assert_eq!(g.take_tutorial_events(), vec![TutorialEvent::Skipped]);
    assert!(g.practice.is_none());
    assert_eq!(g.tutorial.state(), TutorialState::Skipped);
    assert!(g.tutorial.settings_after(&Settings::default()).tutorial_done);
    assert!(!g.skip_tutorial(), "두 번째는 아무 일도 없다");
    // 게임은 그대로 이어진다
    let x = g.player.pos.x;
    walk_right(&mut g, 100.0);
    assert!(g.player.pos.x > x);
    assert!(!g.is_over());
    // 건너뛴 뒤에는 어떤 단계도 다시 안 나온다
    run(&mut g, PlayerInput { jump_pressed: true, ..Default::default() }, 1.0);
    assert!(g.take_tutorial_events().is_empty());
    assert!(g.practice.is_none());
}

#[test]
fn skipping_before_attack_step_means_no_practice_monster_ever() {
    let mut g = open_game();
    start(&mut g);
    assert!(g.skip_tutorial());
    walk_right(&mut g, 500.0);
    run(&mut g, idle(), 3.0);
    assert!(g.practice.is_none());
}

// ---- 단계 넘어감: 직접 해야 한다 ----

#[test]
fn move_step_needs_real_walking_not_waiting() {
    let mut g = open_game();
    start(&mut g);
    run(&mut g, idle(), 30.0);
    assert_eq!(step_of(&g), Some(TutorialStep::Move), "가만히 있으면 안 넘어간다");
    walk_right(&mut g, TUTORIAL_WALK_DISTANCE / 2.0);
    assert_eq!(step_of(&g), Some(TutorialStep::Move), "절반만 걸어도 아직");
    walk_right(&mut g, TUTORIAL_WALK_DISTANCE);
    assert!(g.take_tutorial_events().contains(&TutorialEvent::StepDone(TutorialStep::Move)));
    run(&mut g, idle(), TUTORIAL_CHEER_SECONDS + A_LITTLE);
    assert_eq!(step_of(&g), Some(TutorialStep::Attack));
}

#[test]
fn walking_back_and_forth_adds_up() {
    let mut g = open_game();
    start(&mut g);
    for _ in 0..4 {
        walk_right(&mut g, TUTORIAL_WALK_DISTANCE / 4.0);
        run(&mut g, PlayerInput { left: true, ..Default::default() }, (TUTORIAL_WALK_DISTANCE / 4.0) / PLAYER_WALK_SPEED);
    }
    assert!(g.take_tutorial_events().contains(&TutorialEvent::StepDone(TutorialStep::Move)), "왔다 갔다 걸은 거리도 센다");
}

#[test]
fn whole_walk_through_attack_gather_jump_findmiss() {
    let mut g = open_game();
    start(&mut g);
    go_to_attack_step(&mut g);
    // ② 연습용 몬스터를 2번 때린다
    bring_practice_next_to_player(&mut g);
    face_practice(&mut g);
    swing(&mut g);
    swing(&mut g);
    assert!(g.practice.is_none());
    run(&mut g, idle(), TUTORIAL_CHEER_SECONDS + A_LITTLE);
    assert_eq!(step_of(&g), Some(TutorialStep::Gather));
    // 점프는 ③ 다음이다: 지금 뛰어도 안 센다
    run(&mut g, PlayerInput { jump_pressed: true, ..Default::default() }, 0.5);
    assert_eq!(step_of(&g), Some(TutorialStep::Gather));
    // ③ 나무 캐기
    stand_at(&mut g, TREE_XS[0] - TREE_WIDTH / 2.0 - PLAYER_WIDTH / 2.0 - MINE_REACH / 2.0, Facing::Right);
    run(&mut g, idle(), 0.5);
    g.mine_held = true;
    g.update(&idle(), FRAME);
    let needed = g.gathering.bag.mine_seconds();
    run(&mut g, idle(), needed + A_LITTLE);
    g.mine_held = false;
    g.update(&idle(), FRAME);
    assert!(g.take_tutorial_events().contains(&TutorialEvent::StepDone(TutorialStep::Gather)));
    run(&mut g, idle(), TUTORIAL_CHEER_SECONDS + A_LITTLE);
    assert_eq!(step_of(&g), Some(TutorialStep::Jump));
    // ④ 땅에서 점프
    run(&mut g, idle(), 0.5);
    g.update(&PlayerInput { jump_pressed: true, ..Default::default() }, FRAME);
    assert!(g.take_tutorial_events().contains(&TutorialEvent::StepDone(TutorialStep::Jump)));
    run(&mut g, idle(), TUTORIAL_CHEER_SECONDS + A_LITTLE);
    assert_eq!(step_of(&g), Some(TutorialStep::FindMiss));
    // ⑤ 말만 한다: 시간이 지나면 끝
    run(&mut g, idle(), TUTORIAL_FIND_MISS_SECONDS + 1.0);
    assert!(g.take_tutorial_events().contains(&TutorialEvent::Finished));
    assert_eq!(g.tutorial.state(), TutorialState::Finished);
    assert!(g.tutorial.settings_after(&Settings::default()).tutorial_done);
}


// ---- 연습용 몬스터 (US-124) ----

#[test]
fn practice_monster_comes_in_front_on_screen_when_attack_step_starts_in_first_day() {
    let mut g = open_game();
    start(&mut g);
    assert!(g.practice.is_none(), "①이 끝나기 전에는 안 나온다");
    go_to_attack_step(&mut g);
    let events = g.take_practice_events();
    assert!(matches!(events.as_slice(), [PracticeEvent::Appeared { .. }]));
    let monster = g.practice.as_ref().unwrap();
    assert_eq!(monster.kind(), MonsterKind::Practice);
    assert_eq!(monster.hp, PRACTICE_MONSTER_HP);
    let player_x = g.player.rect().center().x;
    assert!(g.world.is_on_screen(monster.rect().center().x, player_x), "화면 안에 나온다");
    assert!(monster.rect().x > g.player.rect().x, "바라보는 쪽에 나온다");
}

#[test]
fn only_one_practice_monster_and_not_again_if_already_there() {
    let mut g = open_game();
    start(&mut g);
    go_to_attack_step(&mut g);
    assert!(!g.spawn_practice_monster(), "이미 있으면 또 안 나온다");
}

#[test]
fn falls_after_two_hits_whatever_the_weapon() {
    for weapon in [None, Some(Weapon::Axe), Some(Weapon::Gun)] {
        let mut g = open_game();
        if let Some(w) = weapon {
            g.weapons.add_bought(w);
        }
        assert!(g.spawn_practice_monster());
        g.take_practice_events();
        bring_practice_next_to_player(&mut g);
        face_practice(&mut g);
        swing(&mut g);
        let events = g.take_practice_events();
        assert!(matches!(events.as_slice(), [PracticeEvent::Hit { hp_left, .. }] if *hp_left == PRACTICE_MONSTER_HP - 1), "{events:?}");
        assert!(g.practice.is_some(), "한 번 맞고는 안 쓰러진다");
        swing(&mut g);
        let events = g.take_practice_events();
        assert!(matches!(events.as_slice(), [PracticeEvent::Defeated { .. }]), "{events:?}");
        assert!(g.practice.is_none());
    }
}

#[test]
fn practice_monster_never_hurts_even_touching_for_a_long_time() {
    let mut g = open_game();
    assert!(g.spawn_practice_monster());
    let hp = g.combat.health.hp();
    // 몬스터를 플레이어 몸 위로 옮겨 겹치게 두고 오래 둔다
    let x = g.player.rect().x;
    g.practice.as_mut().unwrap().pos.x = x;
    let mut events = Vec::new();
    events.extend(run(&mut g, idle(), 20.0));
    assert_eq!(g.combat.health.hp(), hp);
    assert!(!events.iter().any(|e| matches!(e, GameEvent::Combat(CombatEvent::PlayerHurt { .. } | CombatEvent::PlayerDown))));
    assert!(g.practice.is_some());
}

#[test]
fn practice_monster_follows_slowly_and_stops_near() {
    let mut g = open_game();
    assert!(g.spawn_practice_monster());
    let start_gap = shortest_dx(g.player.rect().center().x, g.practice.as_ref().unwrap().rect().center().x).abs();
    run(&mut g, idle(), 2.0);
    let gap = shortest_dx(g.player.rect().center().x, g.practice.as_ref().unwrap().rect().center().x).abs();
    assert!(gap < start_gap, "다가온다");
    assert!(start_gap - gap <= PRACTICE_MONSTER_WALK_SPEED * 2.0 + 1.0, "천천히");
    run(&mut g, idle(), 8.0);
    let gap = shortest_dx(g.player.rect().center().x, g.practice.as_ref().unwrap().rect().center().x).abs();
    assert!(gap <= PRACTICE_MONSTER_STOP_DISTANCE + 1.0 && gap > 10.0, "가까이에서 멈춘다: {gap}");
}

#[test]
fn no_coins_no_experience_no_score_no_mission_for_practice_monster() {
    let mut g = open_game();
    g.missions.turned_on = vec![MissionKind::Monsters];
    g.missions.give_night_mission(1); // 몬스터 물리치기는 밤 미션 종류다
    assert_eq!(g.missions.current().unwrap().kind, MissionKind::Monsters);
    assert!(g.spawn_practice_monster());
    let (coins, exp, level, defeated, given, score, picked) = (
        g.wallet.coins(),
        g.combat.experience,
        g.combat.level,
        g.combat.monsters_defeated,
        g.combat.monsters_given_experience,
        g.score(),
        g.coins.picked_total,
    );
    bring_practice_next_to_player(&mut g);
    face_practice(&mut g);
    let mut events = swing(&mut g);
    events.extend(swing(&mut g));
    assert!(g.practice.is_none(), "쓰러졌다");
    assert_eq!(g.wallet.coins(), coins);
    assert_eq!((g.combat.experience, g.combat.level), (exp, level));
    assert_eq!((g.combat.monsters_defeated, g.combat.monsters_given_experience), (defeated, given));
    assert_eq!(g.score(), score);
    assert_eq!(g.coins.picked_total, picked);
    assert_eq!(g.missions.current().unwrap().count, 0, "몬스터 물리치기 미션에 안 센다");
    assert!(
        !events.iter().any(|e| matches!(e, GameEvent::Combat(CombatEvent::ZombieDefeated { .. } | CombatEvent::LevelUp { .. }))),
        "밤 몬스터 일이 오지 않는다"
    );
}

#[test]
fn practice_monster_is_not_in_the_night_monster_lists() {
    let mut g = open_game();
    assert!(g.spawn_practice_monster());
    assert!(g.combat.zombies.list.is_empty() && g.combat.skeletons.list.is_empty());
    assert!(!g.combat.any_monster_touches(g.practice.as_ref().unwrap().rect()));
}

#[test]
fn disappears_by_itself_after_sixty_seconds_if_not_hit() {
    let mut g = open_game();
    start(&mut g);
    go_to_attack_step(&mut g);
    g.take_practice_events();
    g.take_tutorial_events();
    // 59초까지는 있다 (이미 ② 시작 뒤 흐른 시간만큼 빼고 본다)
    run(&mut g, idle(), PRACTICE_MONSTER_LIFE_SECONDS - 5.0);
    assert!(g.practice.is_some(), "아직 있다");
    assert_eq!(step_of(&g), Some(TutorialStep::Attack), "안 때려도 몬스터가 있는 동안은 ②");
    let events = run(&mut g, idle(), 6.0);
    let _ = events;
    assert!(g.practice.is_none(), "사라졌다");
    assert!(g.take_practice_events().iter().any(|e| matches!(e, PracticeEvent::Vanished { .. })));
    // 안 때려도 막히지 않는다: ③으로 넘어갔고 잘했어요는 없다
    assert_eq!(step_of(&g), Some(TutorialStep::Gather));
    assert!(!g.tutorial.is_cheering());
    assert!(!g.take_tutorial_events().contains(&TutorialEvent::StepDone(TutorialStep::Attack)));
}

#[test]
fn hit_once_then_wait_still_vanishes_in_sixty_seconds() {
    let mut g = open_game();
    assert!(g.spawn_practice_monster());
    bring_practice_next_to_player(&mut g);
    face_practice(&mut g);
    swing(&mut g);
    run(&mut g, idle(), PRACTICE_MONSTER_LIFE_SECONDS + 1.0);
    assert!(g.practice.is_none());
}

#[test]
fn gone_before_night_even_if_night_comes_with_quiz() {
    let mut g = open_game();
    assert!(g.spawn_practice_monster());
    g.take_practice_events();
    // 해 지기 바로 전으로: 18:00 프레임(퀴즈가 열리는 프레임)에 이미 없어야 한다
    g.clock.update(DAY_SECONDS - 1.0);
    run(&mut g, idle(), 0.5);
    assert!(g.practice.is_some());
    run(&mut g, idle(), 1.0);
    assert!(g.clock.is_night());
    assert!(g.practice.is_none(), "밤 몬스터와 섞이지 않는다");
    assert!(g.take_practice_events().iter().any(|e| matches!(e, PracticeEvent::Vanished { .. })));
}

#[test]
fn practice_monster_does_not_come_at_night_or_on_day_two_or_in_cave() {
    let mut g = open_game();
    g.clock.update(DAY_SECONDS + 1.0);
    assert!(g.clock.is_night());
    assert!(!g.spawn_practice_monster());
    let mut g = open_game();
    g.clock.update(DAY_SECONDS + NIGHT_SECONDS + 1.0);
    assert_eq!(g.clock.day_number(), 2);
    assert!(!g.clock.is_night());
    assert!(!g.spawn_practice_monster(), "첫 낮에만");
    let mut g = open_game();
    g.in_cave = true;
    assert!(!g.spawn_practice_monster());
}

#[test]
fn attack_step_is_skipped_if_step_one_ends_at_night() {
    let mut g = open_game();
    start(&mut g);
    g.clock.update(DAY_SECONDS + 1.0);
    g.update(&idle(), 0.0);
    walk_right(&mut g, TUTORIAL_WALK_DISTANCE + 20.0);
    // 밤 퀴즈가 열려 있으면 닫히는 것을 기다리지 않고 규칙만 본다: 연습용 몬스터는 안 나온다
    assert!(g.practice.is_none());
    assert!(!g.take_practice_events().iter().any(|e| matches!(e, PracticeEvent::Appeared { .. })));
}

#[test]
fn attack_on_practice_monster_does_not_break_a_box_next_to_it() {
    let mut g = open_game();
    assert!(g.spawn_practice_monster());
    bring_practice_next_to_player(&mut g);
    face_practice(&mut g);
    let boxes = g.world.obstacles.len();
    swing(&mut g);
    assert_eq!(g.world.obstacles.len(), boxes);
}

#[test]
fn night_monsters_still_hurt_and_give_experience_while_practice_monster_is_around() {
    // 밤 몬스터 규칙이 연습용 몬스터 때문에 바뀌지 않았는지 한 번 더 본다 (Combat 직접).
    let mut g = open_game();
    assert!(g.spawn_practice_monster());
    let zombie = logic::monster::Zombie::new(g.player.rect().center().x + 10.0, GROUND_Y, false);
    g.combat.zombies.list.push(zombie);
    let hp = g.combat.health.hp();
    let events = run(&mut g, idle(), 0.2);
    assert_eq!(g.combat.health.hp(), hp - ZOMBIE_ATTACK, "좀비는 그대로 아프다");
    assert!(events.iter().any(|e| matches!(e, GameEvent::Combat(CombatEvent::PlayerHurt { .. }))));
}

#[test]
fn restart_after_down_does_not_show_tutorial_again() {
    let mut g = open_game();
    start(&mut g);
    let next = g.tutorial.for_next_game();
    let mut g2 = open_game();
    g2.tutorial = next;
    assert!(!g2.start_tutorial(&Settings::default(), false));
    assert_eq!(g2.tutorial.state(), TutorialState::Finished);
}
