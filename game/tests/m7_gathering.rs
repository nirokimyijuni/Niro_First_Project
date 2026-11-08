//! M7 제작과 동굴 시험 (수산나): 캐기, 나무, 광석, 동굴, 결과 이벤트, 제작대.
//! 기준: stories/backlog.md M7 맨 위 표, US-022, US-020, US-064, US-065, US-021.
//! 숫자는 config 상수로 적어서 임시 값을 바꿔도 시험을 고치지 않아도 된다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::crafting::Crafting;
use logic::game::{Game, GameEvent, GatherEvent, Usable};
use logic::gathering::{Bag, Material};
use logic::player::{Facing, PlayerInput};
use logic::world::{really_overlaps, shortest_dx, World};
use macroquad::math::{Rect, Vec2};

const FRAME: f32 = 1.0 / 60.0;
const SEED: u32 = 7;
const MANY_SEEDS: u32 = 300;
/// 프레임을 더한 시간은 f32 오차로 살짝 모자랄 수 있다.
const A_LITTLE: f32 = 0.05;

fn new_game() -> Game {
    Game::new(CharacterKind::Niro, SEED)
}

fn idle() -> PlayerInput {
    PlayerInput::default()
}

/// `seconds`초 동안 같은 입력으로 프레임을 돌린다.
fn run(g: &mut Game, input: PlayerInput, seconds: f32) {
    for _ in 0..(seconds / FRAME).ceil() as usize {
        g.update(&input, FRAME);
    }
}

/// 몸 가운데가 `center_x`이고 `facing`을 보며 땅에 서게 한다.
fn stand_at(g: &mut Game, center_x: f32, facing: Facing) {
    g.player.pos = Vec2::new(center_x - PLAYER_WIDTH / 2.0, GROUND_Y - PLAYER_HEIGHT);
    g.player.vel = Vec2::ZERO;
    g.player.facing = facing;
    g.update(&idle(), 0.0);
}

/// 첫 나무 바로 왼쪽에서 나무를 바라본다. 나무까지 닿는다.
fn stand_by_first_tree(g: &mut Game) {
    stand_at(g, TREE_XS[0] - TREE_WIDTH / 2.0 - PLAYER_WIDTH / 2.0 - MINE_REACH / 2.0, Facing::Right);
}

fn press(g: &mut Game) {
    g.mine_held = true;
    g.update(&idle(), FRAME);
}

fn release(g: &mut Game) {
    g.mine_held = false;
    g.update(&idle(), FRAME);
}

/// 한 번 눌렀다 뗀다 (입구·출구·제작대).
fn tap(g: &mut Game) {
    press(g);
    release(g);
}

fn go_into_cave(g: &mut Game) {
    stand_at(g, CAVE_ENTRANCE_X, Facing::Right);
    tap(g);
    assert!(g.in_cave);
}

fn stand_by_lump(g: &mut Game, index: usize) {
    stand_at(g, ORE_LUMP_XS[index] - ORE_LUMP_WIDTH / 2.0 - PLAYER_WIDTH / 2.0 - 1.0, Facing::Right);
}

/// 맨손으로 앞의 것을 끝까지 캐고 손을 뗀다.
fn mine_one(g: &mut Game) {
    press(g);
    run(g, idle(), g.gathering.bag.mine_seconds() + A_LITTLE);
    release(g);
}

/// 해가 지기 바로 전까지 시계를 돌린다.
fn skip_to_just_before_night(g: &mut Game) {
    g.clock.update(DAY_SECONDS - 1.0);
}

fn skip_to_night(g: &mut Game) {
    g.clock.update(DAY_SECONDS);
    g.update(&idle(), 0.0);
}

fn wood(g: &Game) -> u32 {
    g.gathering.bag.count(Material::Wood)
}

fn ores(g: &Game) -> u32 {
    [Material::Iron, Material::Copper, Material::Silver, Material::Gold].iter().map(|m| g.gathering.bag.count(*m)).sum()
}

// ---- US-022 캐기 입력과 닿는 거리 ----

#[test]
fn bare_hands_take_three_seconds_and_pickaxe_one() {
    for (has_pickaxe, seconds) in [(false, MINE_SECONDS_BARE_HAND), (true, MINE_SECONDS_PICKAXE)] {
        let mut g = new_game();
        g.gathering.bag.has_pickaxe = has_pickaxe;
        stand_by_first_tree(&mut g);
        press(&mut g);
        run(&mut g, idle(), seconds - 0.2);
        assert_eq!(wood(&g), 0, "아직 다 캐지 않았다");
        assert!(g.mining_progress().is_some_and(|p| p > 0.5 && p < 1.0), "표시가 찬다");
        run(&mut g, idle(), 0.2 + A_LITTLE);
        assert_eq!(wood(&g), 1);
        assert_eq!(g.mining_progress(), None);
    }
}

#[test]
fn fast_forward_does_not_speed_up_mining() {
    let mut g = new_game();
    stand_by_first_tree(&mut g);
    g.mine_held = true;
    for _ in 0..((MINE_SECONDS_BARE_HAND - 0.2) / FRAME) as usize {
        g.update_with_time_scale(&idle(), FRAME, DEV_FAST_FORWARD_SCALE);
    }
    assert_eq!(wood(&g), 0, "캐기는 실제 시간이다");
}

#[test]
fn mines_on_both_sides_of_the_body_but_not_out_of_reach() {
    // 긴 맵 나무 옆은 구멍·발판·상자가 있어 몸이 밀린다. 평평한 동굴에 덩이 하나만 남기고 잰다.
    let lump = 2;
    let edge = PLAYER_WIDTH / 2.0 + MINE_REACH;
    for facing in [Facing::Right, Facing::Left] {
        let mut g = new_game();
        go_into_cave(&mut g);
        for (i, other) in g.gathering.ore_lumps.iter_mut().enumerate() {
            if i != lump {
                other.rect.x += SCREEN_WIDTH * 2.0;
            }
        }
        let lump_left = g.gathering.ore_lumps[lump].rect.x;
        let lump_right = lump_left + ORE_LUMP_WIDTH;
        // 덩이가 오른쪽에 있을 때와 왼쪽에 있을 때, 어느 쪽을 봐도 같다 (등 뒤도 닿는다).
        for center_x in [lump_left - edge + 1.0, lump_right + edge - 1.0, lump_left - PLAYER_WIDTH / 2.0 - 1.0] {
            stand_at(&mut g, center_x, facing);
            assert_eq!(g.usable_in_reach(), Some(Usable::OreLump(lump)), "닿는 거리 {center_x} {facing:?}");
        }
        for center_x in [lump_left - edge - 1.0, lump_right + edge + 1.0] {
            stand_at(&mut g, center_x, facing);
            assert_eq!(g.usable_in_reach(), None, "닿지 않는 거리 {center_x} {facing:?}");
            press(&mut g);
            assert_eq!(g.mining_progress(), None);
            release(&mut g);
        }
        // 덩이가 왼쪽 끝에 닿는 자리: 오른쪽을 보면 등 뒤인데도 끝까지 캔다.
        stand_at(&mut g, lump_right + edge - 1.0, facing);
        mine_one(&mut g);
        assert_eq!(ores(&g), 1, "{facing:?}을 보며 끝까지 캔다");
    }
}

#[test]
fn turning_around_while_mining_keeps_mining() {
    let mut g = new_game();
    stand_by_first_tree(&mut g);
    press(&mut g);
    run(&mut g, idle(), 1.0);
    g.player.facing = Facing::Left;
    run(&mut g, idle(), MINE_SECONDS_BARE_HAND - 1.0 + A_LITTLE);
    assert_eq!(wood(&g), 1, "등을 돌려도 닿는 거리면 계속 캔다");
}

#[test]
fn letting_go_early_resets_and_gives_nothing() {
    let mut g = new_game();
    stand_by_first_tree(&mut g);
    press(&mut g);
    run(&mut g, idle(), MINE_SECONDS_BARE_HAND - 0.5);
    release(&mut g);
    assert_eq!(g.mining_progress(), None);
    press(&mut g);
    assert!(g.mining_progress().unwrap() < 0.1, "처음부터 다시");
    assert_eq!(wood(&g), 0);
}

#[test]
fn walking_out_of_reach_stops_mining() {
    let mut g = new_game();
    stand_by_first_tree(&mut g);
    press(&mut g);
    run(&mut g, PlayerInput { left: true, ..Default::default() }, 1.0);
    assert_eq!(g.mining_progress(), None);
    run(&mut g, idle(), MINE_SECONDS_BARE_HAND);
    assert_eq!(wood(&g), 0);
}

#[test]
fn getting_hurt_stops_mining() {
    let mut g = new_game();
    stand_by_first_tree(&mut g);
    press(&mut g);
    run(&mut g, idle(), 1.0);
    assert!(g.mining_progress().is_some());
    g.citizens.list[0].rect = g.player.rect(); // 시민이 펑 터져서 다친다
    run(&mut g, idle(), FRAME);
    assert!(g.combat.health.hp() < PLAYER_START_HP);
    assert_eq!(g.mining_progress(), None);
}

#[test]
fn nothing_nearby_does_nothing() {
    let mut g = new_game();
    assert!(!g.usable_in_reach().is_some());
    press(&mut g);
    run(&mut g, idle(), MINE_SECONDS_BARE_HAND + 1.0);
    assert_eq!(g.mining_progress(), None);
    assert!(g.take_gather_events().is_empty());
}

#[test]
fn holding_does_not_start_the_next_one_by_itself() {
    let mut g = new_game();
    go_into_cave(&mut g);
    stand_by_lump(&mut g, 0);
    press(&mut g);
    run(&mut g, idle(), MINE_SECONDS_BARE_HAND + A_LITTLE);
    assert_eq!(ores(&g), 1);

    // 꾹 누른 채 다음 덩이 앞으로 가도 시작하지 않는다.
    stand_by_lump(&mut g, 1);
    run(&mut g, idle(), MINE_SECONDS_BARE_HAND + 1.0);
    assert_eq!(ores(&g), 1);

    release(&mut g);
    mine_one(&mut g);
    assert_eq!(ores(&g), 2, "뗐다가 다시 누르면 캔다");
}

#[test]
fn cannot_mine_when_down_or_at_the_workbench() {
    let mut g = new_game();
    stand_by_first_tree(&mut g);
    g.crafting.open();
    press(&mut g);
    assert!(!g.usable_in_reach().is_some());
    run(&mut g, idle(), MINE_SECONDS_BARE_HAND + 1.0);
    assert_eq!(wood(&g), 0, "제작대 화면");
    g.crafting.close();
    release(&mut g);

    let body = g.player.rect();
    for _ in 0..PLAYER_START_HP {
        g.combat.citizen_blast(body.center(), body);
        g.combat.update(PLAYER_HURT_INVINCIBLE_SECONDS, false, body, &g.world);
    }
    assert!(g.combat.health.is_down());
    press(&mut g);
    run(&mut g, idle(), MINE_SECONDS_BARE_HAND + 1.0);
    assert_eq!(wood(&g), 0, "쓰러진 화면");
    assert_eq!(g.mining_progress(), None);
}

#[test]
fn mine_button_shows_only_near_something_to_use() {
    let mut g = new_game();
    stand_by_first_tree(&mut g);
    assert!(g.usable_in_reach().is_some(), "나무");
    stand_at(&mut g, CAVE_ENTRANCE_X, Facing::Left);
    assert!(g.usable_in_reach().is_some(), "입구");
    stand_at(&mut g, WORKBENCH_X, Facing::Left);
    assert!(g.usable_in_reach().is_some(), "제작대");
    go_into_cave(&mut g);
    assert!(g.usable_in_reach().is_some(), "출구");
    stand_by_lump(&mut g, 0);
    assert!(g.usable_in_reach().is_some(), "광석");
    // MINE_REACH 100이면 동굴(한 화면)에 빈자리가 없어서, 덩이를 화면 밖으로 치우고 출구 끝만 본다.
    for lump in &mut g.gathering.ore_lumps {
        lump.rect.x += SCREEN_WIDTH * 2.0;
    }
    let exit_right = CAVE_EXIT_X + CAVE_ENTRANCE_WIDTH / 2.0;
    stand_at(&mut g, exit_right + MINE_REACH + PLAYER_WIDTH / 2.0 - 1.0, Facing::Right);
    assert_eq!(g.usable_in_reach(), Some(Usable::CaveExit), "등 뒤 출구");
    stand_at(&mut g, exit_right + MINE_REACH + PLAYER_WIDTH / 2.0 + 1.0, Facing::Left);
    assert!(!g.usable_in_reach().is_some(), "동굴의 빈자리");
}

// ---- US-020 나무와 광석 ----

#[test]
fn four_trees_stand_on_the_long_map_only() {
    let g = new_game();
    assert_eq!(g.gathering.trees.len(), TREE_XS.len());
    assert!(g.gathering.trees.iter().all(|t| t.is_ready()));
    let practice = Game::on_world(CharacterKind::Niro, SEED, World::m1());
    assert!(practice.gathering.trees.is_empty(), "M1 연습 맵은 옛 규칙 그대로");
}

#[test]
fn no_box_ever_sits_on_a_tree_entrance_or_workbench() {
    for seed in 0..MANY_SEEDS {
        let g = Game::new(CharacterKind::Niro, seed.wrapping_mul(7919));
        let spots = g.gathering.trees.iter().map(|t| t.rect).chain(g.world.cave_entrance()).chain(g.world.workbench());
        for spot in spots.collect::<Vec<_>>() {
            assert!(!g.world.obstacles.iter().any(|b| really_overlaps(*b, spot)), "seed {seed}: {spot:?}");
        }
    }
}

#[test]
fn trees_entrance_and_workbench_do_not_block_walking() {
    for x in [TREE_XS[0], CAVE_ENTRANCE_X, WORKBENCH_X] {
        let mut g = new_game();
        stand_at(&mut g, x - 100.0, Facing::Right);
        run(&mut g, PlayerInput { right: true, ..Default::default() }, 1.0);
        assert!(g.player.rect().center().x > x, "{x}을 지나간다");
    }
}

#[test]
fn a_mined_tree_becomes_a_stump_then_grows_back() {
    let mut g = new_game();
    stand_by_first_tree(&mut g);
    mine_one(&mut g);
    assert_eq!(wood(&g), 1);
    assert!(!g.gathering.trees[0].is_ready(), "그루터기");
    assert!(!g.usable_in_reach().is_some());
    mine_one(&mut g);
    assert_eq!(wood(&g), 1, "그루터기는 못 캔다");

    run(&mut g, idle(), TREE_REGROW_SECONDS - MINE_SECONDS_BARE_HAND - 1.0);
    assert!(!g.gathering.trees[0].is_ready());
    run(&mut g, idle(), 1.0 + A_LITTLE);
    assert!(g.gathering.trees[0].is_ready(), "다시 자란다");
    mine_one(&mut g);
    assert_eq!(wood(&g), 2);
}

#[test]
fn fast_forward_does_not_speed_up_growing_back() {
    let mut g = new_game();
    stand_by_first_tree(&mut g);
    mine_one(&mut g);
    for _ in 0..((TREE_REGROW_SECONDS - MINE_SECONDS_BARE_HAND - 1.0) / FRAME) as usize {
        g.update_with_time_scale(&idle(), FRAME, DEV_FAST_FORWARD_SCALE);
    }
    assert!(!g.gathering.trees[0].is_ready());
}

#[test]
fn trees_can_be_mined_at_night() {
    let mut g = new_game();
    g.combat.skeleton_chance = 0.0;
    skip_to_night(&mut g);
    assert!(g.clock.is_night());
    stand_by_first_tree(&mut g);
    press(&mut g);
    g.gathering.bag.has_pickaxe = true; // 몬스터가 오기 전에 끝낸다
    run(&mut g, idle(), MINE_SECONDS_PICKAXE + A_LITTLE);
    assert_eq!(wood(&g), 1);
}

#[test]
fn ore_chances_add_up_to_one() {
    let sum = ORE_IRON_CHANCE + ORE_COPPER_CHANCE + ORE_SILVER_CHANCE + ORE_GOLD_CHANCE;
    assert!((sum - 1.0).abs() < 0.0001);
}

#[test]
fn iron_comes_most_and_gold_least() {
    const LUMPS_TO_MINE: usize = 2000;
    let mut g = new_game();
    go_into_cave(&mut g);
    g.gathering.bag.has_pickaxe = true;
    for _ in 0..LUMPS_TO_MINE {
        stand_by_lump(&mut g, 0);
        press(&mut g);
        g.update(&idle(), MINE_SECONDS_PICKAXE);
        g.update(&idle(), ORE_REGROW_SECONDS);
        g.clock = logic::clock::Clock::new(); // 밤이 오지 않게 한다
        release(&mut g);
    }
    let bag = &g.gathering.bag;
    assert_eq!(ores(&g) as usize, LUMPS_TO_MINE);
    let (iron, copper, silver, gold) =
        (bag.count(Material::Iron), bag.count(Material::Copper), bag.count(Material::Silver), bag.count(Material::Gold));
    assert!(iron > copper && copper > silver && silver > gold && gold > 0, "{iron} {copper} {silver} {gold}");
    let share = iron as f32 / LUMPS_TO_MINE as f32;
    assert!((share - ORE_IRON_CHANCE).abs() < 0.05, "철 비율 {share}");
}

#[test]
fn same_seed_gives_the_same_ores() {
    let mine_five = || {
        let mut g = new_game();
        go_into_cave(&mut g);
        for index in 0..5 {
            stand_by_lump(&mut g, index);
            mine_one(&mut g);
        }
        Material::ALL.map(|m| g.gathering.bag.count(m))
    };
    assert_eq!(mine_five(), mine_five());
}

#[test]
fn mined_lump_comes_back_even_when_outside() {
    let mut g = new_game();
    go_into_cave(&mut g);
    stand_by_lump(&mut g, 0);
    mine_one(&mut g);
    assert_eq!(ores(&g), 1);
    assert!(!g.gathering.ore_lumps[0].is_ready());
    stand_at(&mut g, CAVE_EXIT_X, Facing::Left);
    tap(&mut g);
    assert!(!g.in_cave);
    run(&mut g, idle(), ORE_REGROW_SECONDS);
    assert!(g.gathering.ore_lumps[0].is_ready());
}

#[test]
fn new_game_has_an_empty_bag() {
    let g = new_game();
    assert!(Material::ALL.iter().all(|m| g.gathering.bag.count(*m) == 0));
    assert!(!g.gathering.bag.has_pickaxe);
    assert!(!g.in_cave);
    assert_eq!(g.player.pos.x, PLAYER_SPAWN_X);
}

// ---- US-064 동굴 ----

#[test]
fn entering_and_leaving_the_cave_by_day() {
    let mut g = new_game();
    assert_eq!(g.world.cave_entrance().unwrap().center().x, CAVE_ENTRANCE_X);
    stand_at(&mut g, CAVE_ENTRANCE_X, Facing::Left);
    tap(&mut g);
    assert!(g.in_cave);
    assert_eq!(g.take_gather_events(), vec![GatherEvent::CaveEntered]);
    assert_eq!(g.place_name(), CAVE_NAME);
    assert_eq!(g.camera_left(), 0.0);
    assert_eq!(g.player.rect().center().x, CAVE_EXIT_X);
    assert_eq!(g.gathering.ore_lumps.len(), ORE_LUMP_XS.len());
    assert!(g.cave_world.obstacles.is_empty());

    tap(&mut g);
    assert!(!g.in_cave);
    assert_eq!(g.take_gather_events(), vec![GatherEvent::CaveLeft]);
    assert_eq!(g.player.rect().center().x, CAVE_ENTRANCE_X, "입구 앞");
    assert_eq!(g.place_name(), PLACE_NAMES[1]);
}

#[test]
fn one_long_press_goes_in_only_once() {
    let mut g = new_game();
    stand_at(&mut g, CAVE_ENTRANCE_X, Facing::Right);
    press(&mut g);
    run(&mut g, idle(), 1.0);
    assert!(g.in_cave, "출구 앞에 섰지만 다시 나가지 않는다");
}

#[test]
fn cave_is_walkable_and_has_walls_and_floor() {
    let mut g = new_game();
    go_into_cave(&mut g);
    run(&mut g, PlayerInput { left: true, ..Default::default() }, 2.0);
    assert_eq!(g.player.pos.x, 0.0, "왼쪽 끝");
    run(&mut g, PlayerInput { right: true, ..Default::default() }, (SCREEN_WIDTH / PLAYER_WALK_SPEED) + 1.0);
    assert_eq!(g.player.pos.x, SCREEN_WIDTH - PLAYER_WIDTH, "상자 없이 오른쪽 끝까지 간다");
    assert_eq!(g.player.pos.y + PLAYER_HEIGHT, GROUND_Y, "땅 위");
    g.update(&PlayerInput { jump_pressed: true, ..Default::default() }, FRAME);
    assert!(!g.player.on_ground, "점프");
    run(&mut g, idle(), 2.0);
    assert_eq!(g.player.pos.y + PLAYER_HEIGHT, GROUND_Y);
    let before = g.weapons.held();
    g.switch_weapon();
    assert_eq!(g.weapons.held(), before, "맨손뿐이면 그대로");
    g.update(&PlayerInput { attack_pressed: true, ..Default::default() }, FRAME);
    assert!(g.in_cave);
}

#[test]
fn map_things_do_not_touch_the_player_in_the_cave() {
    // 동굴 안 몸이 해변 상자 코인과 같은 자리(동굴 폭 안)에 있어도 코인을 줍지 않고 시민에도 닿지 않는다 (REVIEW-017 2번).
    let mut g = new_game();
    let coin = g.coins.list.iter().map(|c| c.rect)
        .find(|r| r.x > PLAYER_WIDTH && r.right() < WORLD_WIDTH - PLAYER_WIDTH)
        .expect("동굴 폭 안에 코인이 하나는 있어야 시험이 뜻이 있다");
    go_into_cave(&mut g);
    g.player.pos = Vec2::new(coin.center().x - PLAYER_WIDTH / 2.0, coin.y);
    g.player.vel = Vec2::ZERO;
    g.citizens.list[0].rect = g.player.rect();
    let events = g.update(&idle(), FRAME);
    assert!(g.in_cave);
    assert!(really_overlaps(g.player.rect(), coin), "주인공이 코인 자리에 서 있어야 한다");
    assert!(events.is_empty(), "{events:?}");
    assert_eq!(g.wallet.coins(), 0);
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP);
}

#[test]
fn time_keeps_going_in_the_cave() {
    let mut g = new_game();
    let box_ = g.world.boxes()[0];
    stand_at(&mut g, box_.center().x, Facing::Right);
    g.player.pos.y = box_.y - PLAYER_HEIGHT;
    run(&mut g, idle(), FRAME);
    assert_eq!(g.wallet.coins(), 1, "상자 위 코인을 줍는다");
    let miss_spot = g.miss.as_ref().unwrap().rect(&g.world);
    g.player.pos = Vec2::new(miss_spot.x, miss_spot.y);
    run(&mut g, idle(), FRAME);
    assert!(!g.miss.as_ref().unwrap().is_hiding(), "미스를 찾았다");

    go_into_cave(&mut g);
    assert!(!g.coins.list[0].is_visible());
    let seconds_before = g.clock.seconds_left_in_phase();
    run(&mut g, idle(), MISS_RETURN_SECONDS + 1.0);
    assert!(g.coins.list[0].is_visible(), "코인 시간");
    assert!(g.miss.as_ref().unwrap().is_hiding(), "미스 시간");
    assert!(g.weapons.dropped.is_empty(), "땅의 무기 시간");
    assert!(g.clock.seconds_left_in_phase() < seconds_before, "시계");
    assert_eq!(g.wallet.coins(), 1, "코인은 그대로");
}

#[test]
fn six_pm_pushes_the_player_out() {
    if CAVE_OPEN_AT_NIGHT {
        return;
    }
    let mut g = new_game();
    go_into_cave(&mut g);
    stand_by_lump(&mut g, 0);
    g.take_gather_events();
    press(&mut g);
    skip_to_just_before_night(&mut g);
    run(&mut g, idle(), 1.0 + A_LITTLE);
    assert!(g.clock.is_night());
    assert!(!g.in_cave);
    assert_eq!(g.player.rect().center().x, CAVE_ENTRANCE_X, "입구 앞");
    assert_eq!(g.mining_progress(), None, "캐던 것은 멈춘다");
    assert_eq!(g.take_gather_events(), vec![GatherEvent::PushedOutOfCave]);
    assert_eq!(CAVE_NIGHT_OUT_TEXT, "밤이 왔어요! 밖으로 나왔어요");
    // 임시-37: 입구 앞에 선 다음 밤 문제가 열린다. 그날 밤 문제는 하나다.
    assert!(g.math.is_open(), "밤 문제");
    let answer = g.math.question().unwrap().answer;
    g.math.type_and_submit(answer);
    run(&mut g, idle(), MATH_RIGHT_SHOW_SECONDS + 1.0);
    assert!(!g.math.is_open(), "밤 문제는 한 번만");
}

#[test]
fn fast_forward_day_ends_in_the_cave_after_thirty_seconds() {
    if CAVE_OPEN_AT_NIGHT {
        return;
    }
    let mut g = new_game();
    go_into_cave(&mut g);
    let seconds = DAY_SECONDS / DEV_FAST_FORWARD_SCALE;
    for _ in 0..((seconds - 1.0) / FRAME) as usize {
        g.update_with_time_scale(&idle(), FRAME, DEV_FAST_FORWARD_SCALE);
    }
    assert!(g.in_cave);
    for _ in 0..(2.0 / FRAME) as usize {
        g.update_with_time_scale(&idle(), FRAME, DEV_FAST_FORWARD_SCALE);
    }
    assert!(!g.in_cave);
}

#[test]
fn cave_is_closed_at_night_and_opens_at_six_am() {
    if CAVE_OPEN_AT_NIGHT {
        return;
    }
    let mut g = new_game();
    g.combat.skeleton_chance = 0.0;
    skip_to_night(&mut g);
    stand_at(&mut g, CAVE_ENTRANCE_X, Facing::Right);
    tap(&mut g);
    assert!(!g.in_cave);
    assert_eq!(g.take_gather_events(), vec![GatherEvent::CaveClosed]);
    assert_eq!(CAVE_CLOSED_TEXT, "동굴은 낮에 열려요");

    g.clock.update(NIGHT_SECONDS);
    assert!(!g.clock.is_night());
    stand_at(&mut g, CAVE_ENTRANCE_X, Facing::Right);
    tap(&mut g);
    assert!(g.in_cave, "아침에는 다시 열린다");
}

// ---- US-065 결과 이벤트 ----

#[test]
fn mining_tells_what_came_out_and_where() {
    let mut g = new_game();
    stand_by_first_tree(&mut g);
    mine_one(&mut g);
    let tree_center = g.gathering.trees[0].rect.center();
    assert_eq!(g.take_gather_events(), vec![GatherEvent::Mined { what: Material::Wood, at: tree_center }]);
    assert!(g.take_gather_events().is_empty(), "한 번만 받는다");

    go_into_cave(&mut g);
    g.take_gather_events();
    stand_by_lump(&mut g, 2);
    mine_one(&mut g);
    let events = g.take_gather_events();
    let lump_center = g.gathering.ore_lumps[2].rect.center();
    assert!(matches!(events[..], [GatherEvent::Mined { what, at }] if what != Material::Wood && at == lump_center), "{events:?}");
}

#[test]
fn every_material_has_its_popup_text() {
    let texts = Material::ALL.map(Material::mined_text);
    assert_eq!(texts, ["나무를 캤어요!", "철광석을 캤어요!", "동을 캤어요!", "은을 캤어요!", "금을 캤어요!"]);
}

// ---- US-021 제작대와 곡괭이 ----

fn bag_with(wood: u32, iron: u32) -> Bag {
    let mut bag = Bag::default();
    (0..wood).for_each(|_| bag.add(Material::Wood));
    (0..iron).for_each(|_| bag.add(Material::Iron));
    bag
}

fn material_of(mark: char) -> Option<Material> {
    match mark {
        CRAFT_WOOD_MARK => Some(Material::Wood),
        CRAFT_IRON_MARK => Some(Material::Iron),
        _ => None,
    }
}

/// `PICKAXE_SHAPE`대로 놓은 제작대.
fn crafting_with_shape(bag: &Bag) -> Crafting {
    let mut crafting = Crafting::default();
    for (row, marks) in PICKAXE_SHAPE.iter().enumerate() {
        for (column, mark) in marks.iter().enumerate() {
            if let Some(material) = material_of(*mark) {
                crafting.tap_cell(row, column, material, bag);
            }
        }
    }
    crafting
}

#[test]
fn pickaxe_shape_is_d5() {
    let count = |mark| PICKAXE_SHAPE.iter().flatten().filter(|m| **m == mark).count();
    assert_eq!((count(CRAFT_WOOD_MARK), count(CRAFT_IRON_MARK)), (4, 2));
}

#[test]
fn the_shape_makes_a_pickaxe_and_uses_up_materials() {
    let mut bag = bag_with(5, 3);
    let mut crafting = crafting_with_shape(&bag);
    assert_eq!(bag.count(Material::Wood), 5, "놓는 동안은 줄지 않는다");
    assert!(crafting.pickaxe_is_ready(&bag));
    assert!(crafting.take_pickaxe(&mut bag));
    assert!(bag.has_pickaxe);
    assert_eq!((bag.count(Material::Wood), bag.count(Material::Iron)), (1, 1));
    assert!(crafting.grid.iter().flatten().all(|cell| cell.is_none()), "칸이 빈다");
    assert_eq!(bag.mine_seconds(), MINE_SECONDS_PICKAXE);
}

#[test]
fn any_one_wrong_cell_makes_nothing() {
    let bag = bag_with(20, 20);
    for row in 0..CRAFT_GRID_HEIGHT {
        for column in 0..CRAFT_GRID_WIDTH {
            let mut crafting = crafting_with_shape(&bag);
            let swapped = match crafting.grid[row][column] {
                Some(Material::Wood) => Some(Material::Iron),
                Some(_) => Some(Material::Wood),
                None => Some(Material::Wood),
            };
            crafting.grid[row][column] = swapped;
            assert!(!crafting.pickaxe_is_ready(&bag), "{row}줄 {column}칸");
        }
    }
}

#[test]
fn same_materials_in_another_shape_make_nothing() {
    let mut bag = bag_with(4, 2);
    let mut crafting = Crafting::default();
    let cells = [(0, 0), (0, 1), (0, 2), (0, 3), (1, 0), (1, 1)];
    for (index, (row, column)) in cells.into_iter().enumerate() {
        crafting.tap_cell(row, column, if index < 4 { Material::Wood } else { Material::Iron }, &bag);
    }
    assert!(!crafting.pickaxe_is_ready(&bag));
    assert!(!crafting.take_pickaxe(&mut bag));
    assert_eq!(bag.count(Material::Wood), 4);
    assert!(!bag.has_pickaxe);
}

#[test]
fn cannot_place_more_than_you_have_and_tapping_again_takes_it_off() {
    let bag = bag_with(1, 0);
    let mut crafting = Crafting::default();
    crafting.tap_cell(0, 0, Material::Wood, &bag);
    crafting.tap_cell(0, 1, Material::Wood, &bag);
    crafting.tap_cell(0, 2, Material::Iron, &bag);
    assert_eq!(crafting.grid[0][..3], [Some(Material::Wood), None, None]);
    crafting.tap_cell(0, 0, Material::Iron, &bag);
    assert_eq!(crafting.grid[0][0], None, "놓인 칸을 누르면 빠진다");
    crafting.tap_cell(0, 1, Material::Wood, &bag);
    assert_eq!(crafting.grid[0][1], Some(Material::Wood), "빠진 만큼 다시 놓는다");
}

#[test]
fn only_one_pickaxe_per_game() {
    let mut bag = bag_with(8, 4);
    let mut crafting = crafting_with_shape(&bag);
    assert!(crafting.take_pickaxe(&mut bag));
    let mut again = crafting_with_shape(&bag);
    assert!(!again.pickaxe_is_ready(&bag));
    assert!(!again.take_pickaxe(&mut bag));
    assert_eq!(bag.count(Material::Wood), 4);
}

#[test]
fn closing_empties_the_grid_and_keeps_materials() {
    let bag = bag_with(4, 2);
    let mut crafting = crafting_with_shape(&bag);
    crafting.open();
    crafting.close();
    assert!(!crafting.is_open());
    assert!(crafting.grid.iter().flatten().all(|cell| cell.is_none()));
    assert_eq!(bag.count(Material::Wood), 4);
}

#[test]
fn workbench_opens_with_the_button_and_beats_the_trees_beside_it() {
    let mut g = new_game();
    assert_eq!(g.world.workbench().unwrap().center().x, WORKBENCH_X);
    for facing in [Facing::Left, Facing::Right] {
        stand_at(&mut g, WORKBENCH_X, facing);
        tap(&mut g);
        assert!(g.crafting.is_open());
        assert_eq!(g.take_gather_events(), vec![GatherEvent::CraftingOpened]);
        assert_eq!(g.mining_progress(), None);
        g.crafting.close();
    }
}

#[test]
fn trees_beside_the_workbench_can_still_be_mined_from_the_far_side() {
    let mut g = new_game();
    let last = TREE_XS.len() - 1;
    stand_at(&mut g, TREE_XS[2] - TREE_WIDTH / 2.0 - PLAYER_WIDTH / 2.0, Facing::Right);
    mine_one(&mut g);
    stand_at(&mut g, TREE_XS[last] + TREE_WIDTH / 2.0 + PLAYER_WIDTH / 2.0, Facing::Left);
    mine_one(&mut g);
    assert_eq!(wood(&g), 2);
    assert!(!g.crafting.is_open());
}

#[test]
fn open_workbench_pauses_the_game_and_resumes_where_it_stopped() {
    let mut g = new_game();
    stand_at(&mut g, WORKBENCH_X, Facing::Left);
    tap(&mut g);
    let seconds_left = g.clock.seconds_left_in_phase();
    let x = g.player.pos.x;
    run(&mut g, PlayerInput { right: true, ..Default::default() }, 1.0);
    assert_eq!(g.clock.seconds_left_in_phase() == seconds_left, CRAFTING_PAUSES_GAME, "시계");
    assert_eq!(g.player.pos.x, x, "화면을 여는 동안 주인공은 서 있다");

    g.crafting.close();
    g.update(&idle(), 1.0);
    assert!(g.clock.seconds_left_in_phase() < seconds_left, "닫으면 이어진다");
}

#[test]
fn held_button_does_not_reopen_the_workbench_after_closing() {
    let mut g = new_game();
    stand_at(&mut g, WORKBENCH_X, Facing::Left);
    press(&mut g);
    assert!(g.crafting.is_open());
    // main.rs처럼 창이 열린 동안에도 꾹 누른 E를 그대로 넘긴다 (REVIEW-017 1번: 예전엔 여기서 false였다)
    g.mine_held = true;
    run(&mut g, idle(), 1.0);
    g.crafting.close();
    assert!(g.mine_held, "닫는 순간에도 E는 눌린 채");
    run(&mut g, idle(), 1.0);
    assert!(!g.crafting.is_open(), "닫은 뒤 다음 프레임에 다시 열리지 않는다");
}

#[test]
fn whole_journey_from_trees_to_a_faster_pickaxe() {
    let mut g = new_game();
    g.gathering.bag.add(Material::Iron);
    g.gathering.bag.add(Material::Iron);
    (0..4).for_each(|_| g.gathering.bag.add(Material::Wood));
    stand_at(&mut g, WORKBENCH_X, Facing::Left);
    tap(&mut g);
    g.crafting = crafting_with_shape(&g.gathering.bag);
    g.crafting.open();
    assert!(g.crafting.take_pickaxe(&mut g.gathering.bag));
    g.crafting.close();

    stand_by_first_tree(&mut g);
    press(&mut g);
    run(&mut g, idle(), MINE_SECONDS_PICKAXE + A_LITTLE);
    assert_eq!(wood(&g), 1, "곡괭이는 나무도 빨리 캔다");
}

// ---- 2026-11-08 main에 옮기며 더한 시험: 고리 맵, 임시-36, 임시-37 ----

/// 임시-36: 이만큼 넘게 떨어진다 (픽셀).
const APART: f32 = 64.0;

/// `x`에서 구간 (left, right)까지 고리 거리. 구간 안이면 0이다.
fn ring_gap(x: f32, left: f32, right: f32) -> f32 {
    if shortest_dx(left, x) >= 0.0 && shortest_dx(x, right) >= 0.0 {
        return 0.0;
    }
    shortest_dx(x, left).abs().min(shortest_dx(x, right).abs())
}

#[test]
fn usable_things_keep_apart_from_each_other_and_from_structures() {
    let g = new_game();
    let things: Vec<f32> = TREE_XS.iter().copied().chain([CAVE_ENTRANCE_X, WORKBENCH_X]).collect();
    for (i, a) in things.iter().enumerate() {
        for b in &things[i + 1..] {
            assert!(shortest_dx(*a, *b).abs() > APART, "{a}와 {b}");
        }
        assert!(shortest_dx(*a, 0.0).abs() > APART, "{a}와 이음매");
        for spot in HIDE_SPOTS.iter().map(|s| s.1).chain(CITIZEN_XS) {
            assert!(shortest_dx(*a, spot).abs() > APART, "{a}와 {spot}");
        }
        let solids = g.world.tree_branches().into_iter().chain(g.world.rock_steps()).chain(g.world.hole());
        for solid in solids {
            assert!(ring_gap(*a, solid.x, solid.x + solid.w) > APART, "{a}와 {solid:?}");
        }
    }
}

#[test]
fn trees_are_where_niro_put_them() {
    assert_eq!(TREE_XS, [900.0, 1850.0, 2840.0, 3040.0]);
    let g = new_game();
    let xs: Vec<f32> = g.gathering.trees.iter().map(|t| t.rect.center().x).collect();
    assert_eq!(xs, TREE_XS);
}

#[test]
fn no_box_within_64_pixels_beside_trees_entrance_and_workbench() {
    for seed in 0..MANY_SEEDS {
        let g = Game::new(CharacterKind::Niro, seed.wrapping_mul(7919));
        let spots = g.world.trees().into_iter().chain(g.world.cave_entrance()).chain(g.world.workbench());
        for spot in spots.collect::<Vec<_>>() {
            let side = Rect { x: spot.x - APART, w: spot.w + APART * 2.0, ..spot };
            for b in g.world.boxes() {
                assert!(!really_overlaps(b, side), "seed {seed}: {spot:?} {b:?}");
            }
        }
    }
}

#[test]
fn only_the_nearest_thing_is_used() {
    let mut g = new_game();
    // 둘째 나무를 첫 나무 바로 뒤로 옮겨 둘 다 닿게 한다.
    let first = g.gathering.trees[0].rect;
    g.gathering.trees[1].rect = Rect { x: first.x + 20.0, ..first };
    stand_by_first_tree(&mut g);
    assert_eq!(g.usable_in_reach(), Some(Usable::Tree(0)));
    mine_one(&mut g);
    assert!(!g.gathering.trees[0].is_ready());
    assert!(g.gathering.trees[1].is_ready(), "둘째는 그대로");
    assert_eq!(wood(&g), 1);
    // 첫 나무가 그루터기면 다음으로 가까운 것을 쓴다.
    assert_eq!(g.usable_in_reach(), Some(Usable::Tree(1)));
}

/// 셋째 나무와 제작대 사이: 64픽셀 넘게 떨어져 있어도 MINE_REACH 100이면 둘 다 닿는다.
#[test]
fn between_a_tree_and_the_workbench_the_nearer_one_is_used_even_behind() {
    let (tree_x, bench_x) = (TREE_XS[2], WORKBENCH_X);
    assert!(bench_x - tree_x > APART, "임시-36 간격은 그대로");
    let middle = (tree_x + bench_x) / 2.0;
    let mut g = new_game();
    for facing in [Facing::Right, Facing::Left] {
        // 넷째 나무까지는 닿지 않는 자리라 둘만 닿는다. 가까운 쪽이 등 뒤여도 그쪽을 쓴다.
        stand_at(&mut g, middle - 5.0, facing);
        assert_eq!(g.usable_in_reach(), Some(Usable::Tree(2)), "나무가 가깝다 {facing:?}");
        stand_at(&mut g, middle + 5.0, facing);
        assert_eq!(g.usable_in_reach(), Some(Usable::Workbench), "제작대가 가깝다 {facing:?}");
    }
    // 거리가 같으면 바라보는 쪽을 쓴다.
    stand_at(&mut g, middle, Facing::Right);
    assert_eq!(g.usable_in_reach(), Some(Usable::Workbench));
    stand_at(&mut g, middle, Facing::Left);
    assert_eq!(g.usable_in_reach(), Some(Usable::Tree(2)));

    // 캐면 가까운 나무 하나만 줄고, 제작대는 열리지 않는다.
    stand_at(&mut g, middle - 5.0, Facing::Right);
    mine_one(&mut g);
    assert_eq!(wood(&g), 1);
    assert!(!g.gathering.trees[2].is_ready());
    assert!(g.gathering.trees.iter().enumerate().all(|(i, t)| i == 2 || t.is_ready()));
    assert!(!g.crafting.is_open());
    // 나무가 그루터기가 되면 등 뒤 제작대가 남는다.
    assert_eq!(g.usable_in_reach(), Some(Usable::Workbench));
}

#[test]
fn two_ore_lumps_behind_and_in_front_the_nearer_one_is_mined() {
    let mut g = new_game();
    go_into_cave(&mut g);
    // 덩이 0과 1 사이, 덩이 0 쪽으로 조금 치우쳐 서서 덩이 1을 바라본다.
    let middle = (ORE_LUMP_XS[0] + ORE_LUMP_XS[1]) / 2.0;
    stand_at(&mut g, middle - 5.0, Facing::Right);
    assert_eq!(g.usable_in_reach(), Some(Usable::OreLump(0)), "등 뒤 덩이 0이 더 가깝다");
    mine_one(&mut g);
    assert_eq!(ores(&g), 1);
    assert!(!g.gathering.ore_lumps[0].is_ready());
    assert!(g.gathering.ore_lumps[1].is_ready(), "덩이 1은 그대로");
    assert_eq!(g.usable_in_reach(), Some(Usable::OreLump(1)));
}

#[test]
fn button_text_follows_the_thing_to_use() {
    let mut g = new_game();
    stand_by_first_tree(&mut g);
    assert_eq!(g.usable_in_reach().map(Usable::button_text), Some("캐기"));
    stand_at(&mut g, CAVE_ENTRANCE_X, Facing::Right);
    assert_eq!(g.usable_in_reach().map(Usable::button_text), Some("들어가기"));
    stand_at(&mut g, WORKBENCH_X, Facing::Right);
    assert_eq!(g.usable_in_reach().map(Usable::button_text), Some("만들기"));
    go_into_cave(&mut g);
    assert_eq!(g.usable_in_reach().map(Usable::button_text), Some("나가기"));
    stand_by_lump(&mut g, 0);
    assert_eq!(g.usable_in_reach().map(Usable::button_text), Some("캐기"));
}

#[test]
fn a_tree_across_the_seam_can_be_mined() {
    let mut g = new_game();
    // 나무 하나를 이음매 바로 왼쪽(3,840 쪽)에 두고 이음매 오른쪽(0 쪽)에서 왼쪽을 본다.
    let tree = g.gathering.trees[0].rect;
    g.gathering.trees[0].rect = Rect { x: MAP_WIDTH - tree.w - 10.0, ..tree };
    stand_at(&mut g, PLAYER_WIDTH / 2.0 + 20.0, Facing::Left);
    assert_eq!(g.usable_in_reach(), Some(Usable::Tree(0)));
    mine_one(&mut g);
    assert_eq!(wood(&g), 1);
}

#[test]
fn mining_and_the_pickaxe_are_not_math_items() {
    let mut g = new_game();
    g.gathering.bag.has_pickaxe = true;
    for x in TREE_XS {
        // 나무 바로 위에 서야 제작대처럼 더 가까운 것이 없다 (MINE_REACH 100).
        stand_at(&mut g, x, Facing::Right);
        mine_one(&mut g);
    }
    go_into_cave(&mut g);
    for index in 0..ORE_LUMP_XS.len() {
        stand_by_lump(&mut g, index);
        mine_one(&mut g);
    }
    assert_eq!(wood(&g) + ores(&g), (TREE_XS.len() + ORE_LUMP_XS.len()) as u32);
    assert!(!g.math.is_open());
    assert_eq!(g.weapons.picked_up_count, 0, "아이템 수가 늘지 않는다");
}

#[test]
fn no_math_question_opens_inside_the_cave() {
    let mut g = new_game();
    go_into_cave(&mut g);
    g.math.add_waiting();
    run(&mut g, idle(), 1.0);
    assert!(!g.math.is_open(), "동굴 안");
    stand_at(&mut g, CAVE_EXIT_X, Facing::Right);
    tap(&mut g);
    assert!(!g.in_cave);
    run(&mut g, idle(), FRAME);
    assert!(g.math.is_open(), "나오면 기다리던 문제가 열린다");
}

#[test]
fn open_workbench_stops_six_pm_from_coming() {
    let mut g = new_game();
    g.combat.skeleton_chance = 0.0;
    skip_to_just_before_night(&mut g);
    stand_at(&mut g, WORKBENCH_X, Facing::Right);
    tap(&mut g);
    assert!(g.crafting.is_open());
    run(&mut g, idle(), 5.0);
    assert!(!g.clock.is_night(), "18:00이 오지 않는다");
    assert!(!g.math.is_open());

    g.crafting.close();
    let mut saw_night = false;
    for _ in 0..(2.0 / FRAME) as usize {
        saw_night |= g.update(&idle(), FRAME).contains(&GameEvent::NightStarted);
    }
    assert!(saw_night && g.clock.is_night());
    assert!(g.math.is_open(), "닫은 뒤 18:00에 밤 문제");
}

#[test]
fn cannot_mine_or_enter_while_a_math_question_is_open() {
    let mut g = new_game();
    stand_at(&mut g, CAVE_ENTRANCE_X, Facing::Right);
    g.math.add_waiting();
    g.math.open_next_waiting();
    assert!(g.math.is_open());
    assert_eq!(g.usable_in_reach(), None, "버튼이 안 보인다");
    tap(&mut g);
    assert!(!g.in_cave);

    let mut g = new_game();
    stand_by_first_tree(&mut g);
    press(&mut g);
    assert!(g.mining_progress().is_some());
    g.math.add_waiting();
    g.math.open_next_waiting();
    run(&mut g, idle(), MINE_SECONDS_BARE_HAND + 1.0);
    assert_eq!(g.mining_progress(), None);
    assert_eq!(wood(&g), 0);
}

#[test]
fn cannot_enter_the_cave_when_down() {
    let mut g = new_game();
    stand_at(&mut g, CAVE_ENTRANCE_X, Facing::Right);
    let body = g.player.rect();
    for _ in 0..PLAYER_START_HP {
        g.combat.citizen_blast(body.center(), body);
        g.combat.update(PLAYER_HURT_INVINCIBLE_SECONDS, false, body, &g.world);
    }
    assert!(g.is_over());
    assert_eq!(g.usable_in_reach(), None);
    tap(&mut g);
    assert!(!g.in_cave);
}
