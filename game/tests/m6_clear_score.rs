//! 게임 클리어(US-068, D63)와 점수 시험 (수산나).
//! 빨리감기 경로는 `Game::update_with_time_scale`이다. `fast_frame`이 그것을 부른다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::combat::CombatEvent;
use logic::game::{Game, GameEvent};
use logic::monster::Zombie;
use logic::player::{Facing, PlayerInput};
use logic::skeleton::{Projectile, ProjectileKind, Skeleton, SkeletonStage};
use logic::world::World;
use macroquad::math::Vec2;

const FRAME: f32 = 1.0 / 60.0;

fn new_game() -> Game {
    let mut g = Game::new(CharacterKind::Niro, 3);
    g.combat.skeleton_chance = 0.0;
    g
}

fn idle() -> PlayerInput {
    PlayerInput::default()
}

/// 한밤이 끝나기 바로 전이 되도록 시계만 돌린다. 몬스터는 끼어들지 않는다.
fn skip_to_last_second_of_night(g: &mut Game, nights_done: u32) {
    let full_days = (DAY_SECONDS + NIGHT_SECONDS) * nights_done as f32;
    g.clock.update(full_days + DAY_SECONDS + NIGHT_SECONDS - 0.5);
}

fn hurt_player_until_down(g: &mut Game) {
    let body = g.player.rect();
    for _ in 0..PLAYER_START_HP {
        g.combat.citizen_blast(body.center(), body);
        g.combat.update(PLAYER_HURT_INVINCIBLE_SECONDS, false, body, &g.world);
    }
    assert!(g.combat.health.is_down());
}

fn run_to_morning(g: &mut Game) -> Vec<GameEvent> {
    let mut events = Vec::new();
    for _ in 0..(1.0 / FRAME) as usize {
        events.extend(g.update(&idle(), FRAME));
    }
    events
}

/// 빨리감기 한 프레임. 수학 문제로 멈추는 것까지 logic의 `Game::update_with_time_scale`이 맡는다
/// (main.rs도 이 함수를 부르도록 로건이 바꾼다).
fn fast_frame(g: &mut Game, input: &PlayerInput) -> Vec<GameEvent> {
    g.update_with_time_scale(input, FRAME, DEV_FAST_FORWARD_SCALE)
}

// ---- 클리어 ----

#[test]
fn clear_values_are_in_config() {
    assert_eq!(DAYS_TO_CLEAR, 7);
    assert!(GAME_CLEAR_TEXT.contains(&DAYS_TO_CLEAR.to_string()));
}

#[test]
fn six_nights_do_not_clear_but_the_seventh_does() {
    let mut g = new_game();
    skip_to_last_second_of_night(&mut g, DAYS_TO_CLEAR - 2);
    let events = run_to_morning(&mut g);
    assert_eq!(g.clock.nights_survived(), DAYS_TO_CLEAR - 1);
    assert!(!g.is_cleared() && !g.is_over());
    assert!(!events.contains(&GameEvent::GameCleared));

    let mut g = new_game();
    skip_to_last_second_of_night(&mut g, DAYS_TO_CLEAR - 1);
    assert!(!g.is_cleared(), "해가 뜨기 전에는 아직이다");
    let events = run_to_morning(&mut g);
    assert_eq!(g.clock.nights_survived(), DAYS_TO_CLEAR);
    assert!(g.is_cleared() && g.is_over());
    assert!(!g.combat.health.is_down(), "쓰러짐과 구별된다");
    assert_eq!(events.iter().filter(|e| **e == GameEvent::GameCleared).count(), 1, "한 번만 온다");
}

#[test]
fn a_cleared_game_stops() {
    let mut g = new_game();
    skip_to_last_second_of_night(&mut g, DAYS_TO_CLEAR - 1);
    run_to_morning(&mut g);
    assert!(g.is_cleared());
    let (x, seconds_left) = (g.player.pos.x, g.clock.seconds_left_in_phase());
    let events = g.update(&PlayerInput { right: true, ..Default::default() }, 1.0);
    assert!(events.is_empty());
    assert_eq!((g.player.pos.x, g.clock.seconds_left_in_phase()), (x, seconds_left));

    let mut after = Vec::new();
    g.update_combat(&PlayerInput { attack_pressed: true, ..Default::default() }, 1.0, &mut after);
    assert!(after.is_empty(), "빨리감기 경로도 멈춘다");
}

#[test]
fn going_down_on_the_seventh_night_is_not_a_clear() {
    let mut g = new_game();
    skip_to_last_second_of_night(&mut g, DAYS_TO_CLEAR - 1);
    hurt_player_until_down(&mut g);
    let events = run_to_morning(&mut g);
    assert!(events.is_empty());
    assert!(!g.is_cleared() && g.is_over());
    assert_eq!(g.clock.nights_survived(), DAYS_TO_CLEAR - 1, "쓰러지면 시계도 멈춘다");
}

#[test]
fn down_at_the_very_moment_of_six_am_comes_first() {
    // 시계는 이미 아침인데 체력이 0이다: 빨리감기 경로가 그 순간을 지난다.
    let mut g = new_game();
    skip_to_last_second_of_night(&mut g, DAYS_TO_CLEAR - 1);
    g.clock.update(1.0);
    assert_eq!(g.clock.nights_survived(), DAYS_TO_CLEAR);
    hurt_player_until_down(&mut g);
    let mut events = Vec::new();
    g.update_combat(&idle(), FRAME, &mut events);
    assert!(!g.is_cleared());
    assert!(!events.contains(&GameEvent::GameCleared));
}

#[test]
fn fast_forward_clears_after_about_630_seconds() {
    let mut g = Game::on_world(CharacterKind::Niro, 3, World::m1());
    // 맞아서 쓰러지지 않게 벽으로 막지 않고, 시간만 재려고 밤마다 몬스터를 치운다.
    let mut cleared_at = None;
    let limit = 700.0 / FRAME;
    for frame in 0..limit as usize {
        g.combat.zombies.list.clear();
        g.combat.skeletons.clear();
        // 밤마다 나오는 수학 문제는 바로 맞힌다. 칭찬 글 2초 동안은 게임이 멈춘다.
        if let Some(answer) = g.math.question().filter(|_| g.math.result().is_none()).map(|q| q.answer) {
            g.math.type_and_submit(answer);
        }
        if fast_frame(&mut g, &idle()).contains(&GameEvent::GameCleared) {
            cleared_at = Some(frame as f32 * FRAME);
            break;
        }
    }
    let expected = (DAY_SECONDS + NIGHT_SECONDS) * DAYS_TO_CLEAR as f32 / DEV_FAST_FORWARD_SCALE
        + DAYS_TO_CLEAR as f32 * MATH_RIGHT_SHOW_SECONDS;
    let seconds = cleared_at.expect("빨리감기로도 클리어된다");
    assert!((seconds - expected).abs() < 1.0, "{seconds}초");
}

#[test]
fn new_game_after_a_clear_starts_fresh() {
    let g = new_game();
    assert!(!g.is_cleared() && !g.is_over());
    assert_eq!(g.score(), 0);
    assert_eq!((g.clock.day_number(), g.clock.nights_survived(), g.wallet.coins()), (1, 0, 0));
}

// ---- 점수 ----

fn night_game() -> Game {
    let mut g = new_game();
    g.world.obstacles.clear(); // 상자가 화살과 몬스터를 막지 않게 한다
    g.clock.update(DAY_SECONDS);
    g.update(&idle(), 0.0);
    g.combat.zombies.list.clear();
    g.combat.skeletons.clear();
    g
}

fn attack() -> PlayerInput {
    PlayerInput { attack_pressed: true, ..Default::default() }
}

/// 주인공 바로 앞에 체력 1인 좀비를 세운다.
fn zombie_in_front(g: &mut Game) {
    let mut zombie = Zombie::new(g.player.rect().right() + 5.0, GROUND_Y, false);
    zombie.hp = 1;
    g.combat.zombies.list.push(zombie);
    g.player.facing = Facing::Right;
}

#[test]
fn score_weights_are_in_config() {
    assert_eq!((SCORE_PER_COIN, SCORE_PER_LEVEL_UP, SCORE_PER_MONSTER), (1, 1, 1));
}

#[test]
fn coins_count_even_when_the_wallet_is_full_and_after_spending() {
    let mut g = new_game();
    g.wallet.add(COIN_MAX);
    let coin = g.coins.list[0].rect;
    g.player.pos = Vec2::new(coin.x, coin.y);
    g.update(&idle(), 0.0);
    assert_eq!(g.wallet.coins(), COIN_MAX, "지갑은 꽉 찼다");
    assert_eq!(g.score(), 1, "그래도 얻은 코인이다");
    assert!(g.wallet.spend(100));
    assert_eq!(g.score(), 1, "써도 줄지 않는다");
}

#[test]
fn fast_forward_coins_count_too() {
    let mut g = new_game();
    let coin = g.coins.list[0].rect;
    g.player.pos = Vec2::new(coin.x, coin.y);
    fast_frame(&mut g, &idle());
    assert_eq!(g.score(), 1);
}

#[test]
fn level_ups_count_once_each() {
    let mut g = new_game();
    g.combat.level_up();
    g.combat.level_up();
    assert_eq!(g.score(), 2);
}

#[test]
fn a_defeat_by_attack_counts_and_a_hit_does_not() {
    let mut g = night_game();
    let mut zombie = Zombie::new(g.player.rect().right() + 5.0, GROUND_Y, false);
    zombie.hp = 2;
    g.combat.zombies.list.push(zombie);
    g.update(&attack(), FRAME);
    assert_eq!(g.combat.monsters_defeated, 0, "맞기만 한 것은 세지 않는다");
    g.update(&attack(), FRAME);
    assert_eq!(g.combat.monsters_defeated, 1);
    assert_eq!(g.score(), 1);
}

#[test]
fn defeats_in_the_fast_forward_path_count_too() {
    let mut g = night_game();
    zombie_in_front(&mut g);
    fast_frame(&mut g, &attack());
    assert_eq!(g.combat.monsters_defeated, 1);
}

#[test]
fn monsters_in_a_citizen_blast_do_not_count() {
    let mut g = night_game();
    let citizen = g.citizens.list[0].rect;
    g.player.pos.x = citizen.x - 300.0; // 펑은 안 닿고, 멀어져 사라지지도 않는 거리
    g.combat.zombies.list.push(Zombie::new(citizen.x, GROUND_Y, false));
    g.combat.skeletons.list.push(Skeleton::new(citizen.x + 10.0, GROUND_Y, SkeletonStage::Half));
    let events = g.update(&idle(), FRAME);
    assert!(events.iter().any(|e| matches!(e, GameEvent::CitizenPopped { .. })), "{events:?}");
    let fell = |e: &GameEvent| matches!(e, GameEvent::Combat(CombatEvent::ZombieDefeated { .. } | CombatEvent::SkeletonDefeated { .. }));
    assert!(events.iter().filter(|e| fell(e)).count() >= 2, "펑에 쓰러졌다: {events:?}");
    assert_eq!(g.combat.monsters_defeated, 0, "경험치를 주지 않으니 세지 않는다");
}

#[test]
fn attacking_a_citizen_does_not_count() {
    let mut g = new_game();
    let citizen = g.citizens.list[0].rect;
    g.player.pos = Vec2::new(citizen.x - PLAYER_WIDTH - 5.0, GROUND_Y - PLAYER_HEIGHT);
    g.player.facing = Facing::Right;
    let events = g.update(&attack(), FRAME);
    assert!(events.iter().any(|e| matches!(e, GameEvent::Combat(CombatEvent::CitizenDefeated { .. }))), "{events:?}");
    assert_eq!(g.score(), 0);
}

#[test]
fn touching_a_skeleton_counts() {
    let mut g = night_game();
    g.combat.skeletons.list.push(Skeleton::new(g.player.pos.x, GROUND_Y, SkeletonStage::Normal));
    g.update(&idle(), FRAME);
    assert_eq!(g.combat.monsters_defeated, 1);
}

#[test]
fn a_skeleton_defeated_by_a_bounced_shot_counts() {
    let mut g = night_game();
    let x = g.player.pos.x + 400.0;
    let mut skeleton = Skeleton::new(x, GROUND_Y, SkeletonStage::Normal);
    skeleton.hp = 1;
    g.combat.skeletons.list.push(skeleton);
    let mut shot = Projectile::new(x + 2.0, -1.0, ProjectileKind::Arrow, 2, &g.world);
    shot.bounced = true;
    g.combat.skeletons.projectiles.push(shot);
    g.update(&idle(), FRAME);
    assert!(g.combat.skeletons.list.is_empty());
    assert_eq!(g.combat.monsters_defeated, 1);
}

#[test]
fn a_zombie_that_pops_by_itself_does_not_count() {
    let mut g = night_game();
    g.combat.zombies.list.push(Zombie::new(g.player.pos.x + 300.0, GROUND_Y, true));
    let mut popped = false;
    for _ in 0..((ZOMBIE_FLOAT_START_SECONDS + ZOMBIE_FLOAT_SECONDS + 1.0) / FRAME) as usize {
        g.combat.skeletons.clear();
        g.combat.zombies.list.truncate(1);
        let events = g.update(&idle(), FRAME);
        popped |= events.iter().any(|e| matches!(e, GameEvent::Combat(CombatEvent::ZombiePopped { .. })));
    }
    assert!(popped);
    assert_eq!(g.combat.monsters_defeated, 0);
}

#[test]
fn monsters_cleared_by_morning_do_not_count() {
    let mut g = night_game();
    g.combat.zombies.list.push(Zombie::new(g.player.pos.x + 800.0, GROUND_Y, false));
    g.clock.update(NIGHT_SECONDS);
    g.update(&idle(), FRAME);
    assert!(g.combat.zombies.list.is_empty());
    assert_eq!(g.combat.monsters_defeated, 0);
}

#[test]
fn defeats_always_match_the_experience_given() {
    let mut g = night_game();
    for _ in 0..(EXPERIENCE_PER_LEVEL * 2 + 2) {
        zombie_in_front(&mut g);
        g.update(&attack(), FRAME);
        let from_levels = (g.combat.level - 1) * EXPERIENCE_PER_LEVEL;
        assert_eq!(g.combat.monsters_defeated * EXPERIENCE_PER_DEFEAT, from_levels + g.combat.experience);
    }
}

#[test]
fn five_zombies_and_three_coins_make_nine() {
    let mut g = night_game();
    for _ in 0..5 {
        zombie_in_front(&mut g);
        g.update(&attack(), FRAME);
    }
    g.coins.list.truncate(3);
    for coin in 0..3 {
        g.coins.list[coin].rect.x = 300.0 * (coin + 1) as f32; // 몸에 다른 코인이 같이 닿지 않게 벌린다
        let rect = g.coins.list[coin].rect;
        g.player.pos = Vec2::new(rect.x, rect.y);
        g.update(&idle(), 0.0);
    }
    assert_eq!((g.combat.monsters_defeated, g.combat.level, g.coins.picked_total), (5, 2, 3));
    assert_eq!(g.score(), 9);
}

#[test]
fn score_adds_all_three() {
    let mut g = night_game();
    zombie_in_front(&mut g);
    g.update(&attack(), FRAME);
    g.combat.level_up();
    g.coins.picked_total += 3;
    assert_eq!(g.score(), 3 * SCORE_PER_COIN + SCORE_PER_LEVEL_UP + SCORE_PER_MONSTER);
}
