//! 쓰는 물건 안내 글 시험 (수산나): 부모님 결정 2026-11-15, US-091 가방.
//! `Game::why_cant_use`가 쓸 수 없는 이유를 알려 주고, 그 답이 실제로 썼을 때와 맞는지 본다.
//! 숫자는 config 상수로 적는다. 반복은 없다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::game::Game;
use logic::monster::Zombie;
use logic::player::{Facing, PlayerInput};
use logic::shop::{CantUse, Pocket, Potion};
use macroquad::math::Vec2;

/// 해변 모래밭 가운데쯤 (시작 자리 근처라 상자가 없다).
const BEACH_X: f32 = 150.0;

fn new_game() -> Game {
    Game::new(CharacterKind::Niro, 7)
}

fn stand_at(g: &mut Game, center_x: f32) {
    g.player.pos = Vec2::new(center_x - PLAYER_WIDTH / 2.0, GROUND_Y - PLAYER_HEIGHT);
    g.player.vel = Vec2::ZERO;
    g.player.facing = Facing::Right;
    g.update(&PlayerInput::default(), 0.0);
}

fn put_zombie(g: &mut Game, center_x: f32) {
    g.combat.zombies.list.push(Zombie::new(center_x - ZOMBIE_WIDTH / 2.0, GROUND_Y, false));
}

/// 네 가지 물건을 하나씩 가진 판.
fn game_with_one_of_each() -> Game {
    let mut g = new_game();
    stand_at(&mut g, BEACH_X);
    g.combat.zombies.list.clear();
    g.combat.skeletons.list.clear();
    g.shop.add_potion(Potion::Health);
    g.shop.add_potion(Potion::Poison);
    g.gathering.bag.syringe_guns = 1;
    g.gathering.bag.ender_fruits = 1;
    g
}

/// 실제로 써 본다 (view의 `use_pocket`과 같은 일).
fn use_it(g: &mut Game, pocket: Pocket) -> bool {
    match pocket {
        Pocket::Potion(potion) => g.use_potion(potion),
        Pocket::SyringeGun => g.fire_syringe_gun(),
        Pocket::EnderFruit => g.eat_ender_fruit(),
    }
}

#[test]
fn texts_come_from_config() {
    assert_eq!(CantUse::HealthFull.text(), USE_HEALTH_FULL_TEXT);
    assert_eq!(CantUse::NoMonsterNear.text(), USE_NO_MONSTER_NEAR_TEXT);
    assert_eq!(CantUse::NoMonsterOnScreen.text(), USE_NO_MONSTER_ON_SCREEN_TEXT);
    assert_eq!(CantUse::AlreadyEaten.text(), ENDER_FRUIT_ALREADY_EATEN_TEXT);
    assert_eq!(CantUse::NoneLeft.text(), USE_NONE_LEFT_TEXT);
    assert_eq!(USE_HEALTH_FULL_TEXT, "체력이 가득해요");
    assert_eq!(USE_NO_MONSTER_NEAR_TEXT, "가까이에 몬스터가 없어요");
    assert_eq!(USE_NO_MONSTER_ON_SCREEN_TEXT, "화면에 몬스터가 없어요");
}

#[test]
fn pocket_count_matches_the_bag_and_shop() {
    let g = game_with_one_of_each();
    for pocket in Pocket::ALL {
        assert_eq!(g.pocket_count(pocket), 1, "{pocket:?}");
    }
    assert_eq!(new_game().pocket_count(Pocket::SyringeGun), 0);
}

#[test]
fn nothing_left_says_none_left() {
    let g = new_game();
    for pocket in Pocket::ALL {
        assert_eq!(g.why_cant_use(pocket), Some(CantUse::NoneLeft), "{pocket:?}");
    }
}

#[test]
fn health_potion_full_then_hurt() {
    let mut g = game_with_one_of_each();
    let health = Pocket::Potion(Potion::Health);
    assert_eq!(g.why_cant_use(health), Some(CantUse::HealthFull));
    assert!(!use_it(&mut g, health));
    g.combat.health.lose_from_quiz(1);
    assert_eq!(g.why_cant_use(health), None);
    assert!(use_it(&mut g, health));
}

#[test]
fn poison_needs_a_monster_within_poison_range() {
    let mut g = game_with_one_of_each();
    let poison = Pocket::Potion(Potion::Poison);
    assert_eq!(g.why_cant_use(poison), Some(CantUse::NoMonsterNear));
    put_zombie(&mut g, BEACH_X + POISON_RANGE + 100.0); // 화면 안이지만 멀다
    assert_eq!(g.why_cant_use(poison), Some(CantUse::NoMonsterNear));
    assert!(!use_it(&mut g, poison));
    put_zombie(&mut g, BEACH_X + POISON_RANGE - 20.0);
    assert_eq!(g.why_cant_use(poison), None);
    assert!(use_it(&mut g, poison));
}

#[test]
fn syringe_gun_needs_a_monster_on_screen() {
    let mut g = game_with_one_of_each();
    assert_eq!(g.why_cant_use(Pocket::SyringeGun), Some(CantUse::NoMonsterOnScreen));
    put_zombie(&mut g, BEACH_X + SCREEN_WIDTH + 300.0); // 화면 밖
    assert_eq!(g.why_cant_use(Pocket::SyringeGun), Some(CantUse::NoMonsterOnScreen));
    assert!(!use_it(&mut g, Pocket::SyringeGun));
    // 화면 안이면 독약 거리 밖이어도 된다.
    put_zombie(&mut g, BEACH_X + POISON_RANGE + 100.0);
    assert_eq!(g.why_cant_use(Pocket::Potion(Potion::Poison)), Some(CantUse::NoMonsterNear));
    assert_eq!(g.why_cant_use(Pocket::SyringeGun), None);
    assert!(use_it(&mut g, Pocket::SyringeGun));
}

#[test]
fn in_the_cave_there_are_no_monsters_to_use_on() {
    let mut g = game_with_one_of_each();
    put_zombie(&mut g, BEACH_X + 50.0);
    g.in_cave = true;
    assert_eq!(g.why_cant_use(Pocket::Potion(Potion::Poison)), Some(CantUse::NoMonsterNear));
    assert_eq!(g.why_cant_use(Pocket::SyringeGun), Some(CantUse::NoMonsterOnScreen));
}

#[test]
fn ender_fruit_already_eaten() {
    let mut g = game_with_one_of_each();
    g.gathering.bag.ender_fruits = 2;
    assert_eq!(g.why_cant_use(Pocket::EnderFruit), None);
    assert!(use_it(&mut g, Pocket::EnderFruit));
    assert_eq!(g.why_cant_use(Pocket::EnderFruit), Some(CantUse::AlreadyEaten));
    assert!(!use_it(&mut g, Pocket::EnderFruit));
}

#[test]
fn open_window_or_game_over_gives_no_text() {
    let mut g = game_with_one_of_each();
    g.shop.open();
    for pocket in Pocket::ALL {
        assert_eq!(g.why_cant_use(pocket), None, "상점 {pocket:?}");
        assert!(!use_it(&mut g, pocket));
    }
    g.close_shop();
    g.crafting.open();
    assert_eq!(g.why_cant_use(Pocket::Potion(Potion::Health)), None, "제작대");
    g.crafting.close();
    let max = g.combat.health.max_hp();
    g.combat.health.lose_from_quiz(max);
    assert!(g.is_over());
    for pocket in Pocket::ALL {
        assert_eq!(g.why_cant_use(pocket), None, "판이 끝남 {pocket:?}");
    }
}

/// 이유가 None이면 쓰이고, Some이면 안 쓰인다. 여러 상황에서 한 번씩 맞춰 본다.
#[test]
fn reason_always_agrees_with_using() {
    let setups: [fn(&mut Game); 4] = [
        |_| {},
        |g| g.combat.health.lose_from_quiz(1),
        |g| put_zombie(g, BEACH_X + 60.0),
        |g| g.gathering.bag.has_ender_power = true,
    ];
    for (i, setup) in setups.iter().enumerate() {
        for pocket in Pocket::ALL {
            let mut g = game_with_one_of_each();
            setup(&mut g);
            let reason = g.why_cant_use(pocket);
            assert_eq!(use_it(&mut g, pocket), reason.is_none(), "상황 {i} {pocket:?} 이유 {reason:?}");
        }
    }
}

#[test]
fn pickaxe_how_and_bag_texts() {
    assert_eq!(PICKAXE_HOW_TEXT, "나무나 광석 가까이에서 E를 누르고 있으면 캐요");
    assert_eq!(DIAMOND_PICKAXE_HOW_TEXT, "더 빨리 캐요. 해변 바닷가에서 E를 누르면 쿵!");
    assert!(PICKAXE_HOW_TOUCH_TEXT.contains(MINE_BUTTON_TEXT));
    assert!(DIAMOND_PICKAXE_HOW_TOUCH_TEXT.contains(SLAM_BUTTON_TEXT));
    assert!(!PICKAXE_HOW_TOUCH_TEXT.contains('E') && !DIAMOND_PICKAXE_HOW_TOUCH_TEXT.contains('E'), "폰 글에는 E 키가 없다");
    assert_eq!(BAG_NAME_TEXT, "가방");
    assert!(KEY_HINT_SECONDS > 0.0 && BAG_FLASH_SECONDS > 0.0);
}
