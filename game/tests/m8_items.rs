//! M8 새 조합 시험 (수산나): 새 광물, 제작 고르기, 자동차, 주사기총, 엔더열매, 다이아래더라이트 곡괭이.
//! 기준: stories/backlog.md US-078~US-083, 임시-15·39~48, docs/02-plan.md D74~D80.
//! 숫자는 config 상수로 적어서 임시 값을 바꿔도 시험을 고치지 않아도 된다. 반복은 모두 횟수가 정해져 있다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::combat::CombatEvent;
use logic::crafting::{Crafting, Recipe};
use logic::game::{Game, GameEvent, ItemEvent, SlamPower, Usable};
use logic::gathering::{ore_for_roll, Bag, Material};
use logic::monster::{SeededRandom, Zombie};
use logic::player::{Facing, Player, PlayerInput};
use logic::shop::{sell_price, ShopItem};
use logic::skeleton::{Skeleton, SkeletonStage};
use logic::weapon::Weapon;
use logic::world::{really_overlaps, World};
use macroquad::math::Vec2;

const FRAME: f32 = 1.0 / 60.0;
const SEED: u32 = 7;
/// 프레임을 더한 시간은 f32 오차로 살짝 모자랄 수 있다.
const A_LITTLE: f32 = 0.05;
/// 해변 모래밭 가운데쯤 (쿵 자리, 시작 자리 근처라 상자가 없다).
const BEACH_X: f32 = 150.0;

fn new_game() -> Game {
    Game::new(CharacterKind::Niro, SEED)
}

/// 상자·미스·시민·집 문이 없는 넓은 맵 (긴 맵이 아니다).
fn empty_game() -> Game {
    Game::on_world(CharacterKind::Niro, SEED, World { width: 3000.0, ground_y: GROUND_Y, obstacles: Vec::new() })
}

fn idle() -> PlayerInput {
    PlayerInput::default()
}

fn attack() -> PlayerInput {
    PlayerInput { attack_pressed: true, ..Default::default() }
}

fn right() -> PlayerInput {
    PlayerInput { right: true, ..Default::default() }
}

fn run(g: &mut Game, input: PlayerInput, seconds: f32) {
    for _ in 0..(seconds / FRAME).ceil() as usize {
        g.update(&input, FRAME);
    }
}

fn stand_at(g: &mut Game, center_x: f32, facing: Facing) {
    g.player.pos = Vec2::new(center_x - PLAYER_WIDTH / 2.0, GROUND_Y - PLAYER_HEIGHT);
    g.player.vel = Vec2::ZERO;
    g.player.facing = facing;
    g.update(&idle(), 0.0);
}

/// E 키(캐기 버튼)를 새로 눌렀다 뗀다.
fn tap_e(g: &mut Game) {
    g.mine_held = true;
    g.update(&idle(), FRAME);
    g.mine_held = false;
    g.update(&idle(), FRAME);
}

fn add(bag: &mut Bag, material: Material, count: u32) {
    for _ in 0..count {
        bag.add(material);
    }
}

/// 가방에 그 조합 재료를 딱 맞게 넣는다.
fn fill_bag_for(bag: &mut Bag, recipe: Recipe) {
    for (material, count) in recipe.needs() {
        add(bag, material, count);
    }
}

/// 판을 조합 모양대로 채운다 (재료 버튼을 고르고 칸을 누르는 것과 같다).
fn place_shape(crafting: &mut Crafting, recipe: Recipe, bag: &Bag) {
    for row in 0..CRAFT_GRID_HEIGHT {
        for column in 0..CRAFT_GRID_WIDTH {
            if let Some(material) = recipe.material_at(row, column) {
                crafting.tap_cell(row, column, material, bag);
            }
        }
    }
}

/// 제작대에서 `recipe`를 만든다. 재료는 이 함수가 넣는다.
fn craft(g: &mut Game, recipe: Recipe) -> Option<Recipe> {
    fill_bag_for(&mut g.gathering.bag, recipe);
    g.crafting.open();
    place_shape(&mut g.crafting, recipe, &g.gathering.bag);
    let made = g.crafting.take_result(&mut g.gathering.bag);
    g.crafting.close();
    made
}

/// 몸 가운데가 `center_x`인 좀비를 둔다.
fn put_zombie(g: &mut Game, center_x: f32) {
    g.combat.zombies.list.push(Zombie::new(center_x - ZOMBIE_WIDTH / 2.0, GROUND_Y, false));
}

fn put_skeleton(g: &mut Game, center_x: f32) {
    g.combat.skeletons.list.push(Skeleton::new(center_x - SKELETON_WIDTH / 2.0, GROUND_Y, SkeletonStage::Normal));
}

fn monster_count(g: &Game) -> usize {
    g.combat.zombies.list.len() + g.combat.skeletons.list.len()
}

fn defeated_shown(events: &[GameEvent]) -> usize {
    events
        .iter()
        .filter(|e| matches!(e, GameEvent::Combat(CombatEvent::ZombieDefeated { .. } | CombatEvent::SkeletonDefeated { .. })))
        .count()
}

/// 06:00가 올 때까지 게임 속 시간을 흘린다 (몬스터는 실제 시간이라 조금만 움직인다).
fn run_until_morning(g: &mut Game) -> Vec<ItemEvent> {
    let mut item_events = Vec::new();
    for _ in 0..2000 {
        let events = g.update_with_time_scale(&idle(), FRAME, 600.0);
        // 밤 수학 문제가 열리면 맞히고 닫는다.
        if let Some(q) = g.math.question() {
            let answer = q.answer;
            g.math.type_and_submit(answer);
            g.update(&idle(), MATH_RIGHT_SHOW_SECONDS + A_LITTLE);
        }
        item_events.extend(g.take_item_events());
        if events.iter().any(|e| matches!(e, GameEvent::DayStarted { .. })) {
            return item_events;
        }
    }
    panic!("아침이 오지 않았다");
}

// ---- US-079 새 광물 ----

#[test]
fn ore_ratios_over_100k_rolls_are_within_half_a_point() {
    const ROLLS: usize = 100_000;
    let mut random = SeededRandom::new(SEED);
    let mut bag = Bag::default();
    for _ in 0..ROLLS {
        bag.add(ore_for_roll(random.next_unit()));
    }
    let expected = [
        (Material::Iron, ORE_IRON_CHANCE),
        (Material::Copper, ORE_COPPER_CHANCE),
        (Material::Silver, ORE_SILVER_CHANCE),
        (Material::Ruby, ORE_RUBY_CHANCE),
        (Material::Gold, ORE_GOLD_CHANCE),
        (Material::Emerald, ORE_EMERALD_CHANCE),
        (Material::Diamond, ORE_DIAMOND_CHANCE),
    ];
    for (material, chance) in expected {
        let share = bag.count(material) as f32 / ROLLS as f32;
        assert!((share - chance).abs() <= 0.005, "{material:?} {share} (기대 {chance})");
    }
    let (ruby, emerald, diamond) = (bag.count(Material::Ruby), bag.count(Material::Emerald), bag.count(Material::Diamond));
    assert!(diamond > 0 && diamond < emerald && emerald < ruby, "다이아몬드가 가장 적고 에메랄드, 루비 순서");
    assert_eq!(bag.count(Material::Wood), 0, "광석에서 나무는 안 나온다");
}

#[test]
fn seven_chances_add_up_to_one() {
    let sum = ORE_IRON_CHANCE + ORE_COPPER_CHANCE + ORE_SILVER_CHANCE + ORE_RUBY_CHANCE + ORE_GOLD_CHANCE + ORE_EMERALD_CHANCE
        + ORE_DIAMOND_CHANCE;
    assert!((sum - 1.0).abs() < 0.0001);
}

#[test]
fn new_ores_have_names_texts_and_sparkle() {
    assert_eq!(Material::Diamond.mined_text(), "영차! 다이아몬드를 캤어요!", "아이 답: 영차!");
    assert_eq!(Material::Emerald.mined_text(), "에메랄드를 캤어요!");
    assert_eq!(Material::Ruby.mined_text(), "루비를 캤어요!");
    for m in [Material::Diamond, Material::Emerald, Material::Ruby, Material::Gold, Material::Silver] {
        assert!(m.is_precious(), "{m:?} 반짝인다");
    }
    assert!(!Material::Iron.is_precious() && !Material::Copper.is_precious());
}

#[test]
fn new_game_has_no_new_ores_and_they_are_not_sold() {
    let g = new_game();
    for m in [Material::Diamond, Material::Emerald, Material::Ruby] {
        assert_eq!(g.gathering.bag.count(m), 0);
        assert_eq!(sell_price(m), None, "임시-48 팔지 않는다");
    }
    let rows: Vec<Material> = g.sell_rows().iter().map(|r| r.material).collect();
    assert_eq!(rows, vec![Material::Gold, Material::Silver, Material::Copper]);
}

// ---- US-078 제작 고르기 ----

#[test]
fn every_recipe_is_made_from_its_shape() {
    for recipe in Recipe::ALL {
        let mut g = new_game();
        assert_eq!(craft(&mut g, recipe), Some(recipe), "{recipe:?}");
        let bag = &g.gathering.bag;
        for material in Material::ALL {
            assert_eq!(bag.count(material), 0, "{recipe:?}: {material:?}가 다 쓰였다");
        }
        assert!(g.crafting.grid.iter().flatten().all(Option::is_none), "판이 빈다");
    }
}

#[test]
fn result_shows_the_matching_recipe_even_if_another_is_chosen() {
    let mut g = new_game();
    fill_bag_for(&mut g.gathering.bag, Recipe::SyringeGun);
    g.crafting.open();
    g.crafting.choose(Recipe::Car);
    place_shape(&mut g.crafting, Recipe::SyringeGun, &g.gathering.bag);
    assert_eq!(g.crafting.ready_recipe(&g.gathering.bag), Some(Recipe::SyringeGun));
}

#[test]
fn one_wrong_cell_or_a_filled_fourth_column_makes_nothing() {
    let mut g = new_game();
    fill_bag_for(&mut g.gathering.bag, Recipe::Car);
    add(&mut g.gathering.bag, Material::Wood, 1);
    g.crafting.open();
    place_shape(&mut g.crafting, Recipe::Car, &g.gathering.bag);
    assert_eq!(g.crafting.ready_recipe(&g.gathering.bag), Some(Recipe::Car));
    // 4번째 칸에 나무를 하나 더 놓으면 안 나온다.
    g.crafting.tap_cell(0, 3, Material::Wood, &g.gathering.bag);
    assert_eq!(g.crafting.ready_recipe(&g.gathering.bag), None);
    g.crafting.tap_cell(0, 3, Material::Wood, &g.gathering.bag); // 다시 빼기
    // 철광석 한 칸을 빼면 안 나온다.
    g.crafting.tap_cell(2, 2, Material::Iron, &g.gathering.bag);
    assert_eq!(g.crafting.ready_recipe(&g.gathering.bag), None);
}

#[test]
fn pickaxe_is_chosen_first_and_the_last_choice_stays() {
    let mut g = new_game();
    g.crafting.open();
    assert_eq!(g.crafting.chosen(), Recipe::Pickaxe);
    assert_eq!(g.crafting.hint_at(0, 0), Some(Material::Iron));
    g.crafting.choose(Recipe::EnderFruit);
    g.crafting.close();
    g.crafting.open();
    assert_eq!(g.crafting.chosen(), Recipe::EnderFruit);
    assert_eq!(g.crafting.hint_at(1, 1), Some(Material::Copper), "흐린 칸이 고른 모양이다");
    assert_eq!(g.crafting.hint_at(1, 3), None, "4번째 칸은 빈다");
}

#[test]
fn a_material_with_zero_count_is_not_placed() {
    let mut g = new_game();
    g.crafting.open();
    g.crafting.tap_cell(0, 0, Material::Diamond, &g.gathering.bag);
    assert_eq!(g.crafting.grid[0][0], None);
}

#[test]
fn shortage_text_says_what_and_how_many() {
    let mut bag = Bag::default();
    add(&mut bag, Material::Copper, 7);
    assert_eq!(Recipe::EnderFruit.why_cant_make_text(&bag).as_deref(), Some("동이 2개 더 필요해요"));
    add(&mut bag, Material::Copper, 2);
    assert_eq!(Recipe::EnderFruit.why_cant_make_text(&bag), None);
    bag.has_syringe_gun = true;
    assert_eq!(Recipe::SyringeGun.why_cant_make_text(&bag).as_deref(), Some(CRAFT_ALREADY_HAVE_TEXT));
}

#[test]
fn old_pickaxe_crafting_still_works() {
    let mut g = new_game();
    fill_bag_for(&mut g.gathering.bag, Recipe::Pickaxe);
    g.crafting.open();
    place_shape(&mut g.crafting, Recipe::Pickaxe, &g.gathering.bag);
    assert_eq!(g.crafting.ready_recipe(&g.gathering.bag), Some(Recipe::Pickaxe));
    assert_eq!(g.crafting.take_result(&mut g.gathering.bag), Some(Recipe::Pickaxe));
    assert!(g.gathering.bag.has_pickaxe);
}

// ---- US-080 자동차 ----

#[test]
fn car_is_ridden_at_once_and_is_three_times_faster() {
    let mut g = empty_game();
    assert_eq!(craft(&mut g, Recipe::Car), Some(Recipe::Car));
    assert_eq!(Recipe::Car.crafted_text(), "자동차를 탔어요! 슝!");
    assert!(g.gathering.bag.riding_car);
    assert_eq!(g.walk_speed(), 660.0);
    stand_at(&mut g, 500.0, Facing::Right);
    let start = g.player.pos.x;
    run(&mut g, right(), 1.0);
    let moved = g.player.pos.x - start;
    assert!((moved - PLAYER_WALK_SPEED * CAR_SPEED_MULTIPLIER).abs() < 20.0, "1초에 {moved}");
}

#[test]
fn car_does_not_stack_with_skates() {
    let mut g = empty_game();
    g.wallet.add(SKATES_PRICE);
    g.buy(ShopItem::Skates).unwrap();
    assert_eq!(g.walk_speed(), 330.0);
    craft(&mut g, Recipe::Car);
    assert_eq!(g.walk_speed(), 660.0, "990이 아니다");
    g.gathering.bag.riding_car = false; // 자동차에서 내렸는데 스케이트는 남았다
    assert_eq!(g.walk_speed(), 330.0);
}

#[test]
fn car_is_left_at_the_first_six_o_clock() {
    let mut g = new_game();
    g.clock.update(DAY_SECONDS * (4.0 / 12.0)); // 1일째 10:00
    assert_eq!(g.clock.hour_minute(), (10, 0));
    craft(&mut g, Recipe::Car);
    let item_events = run_until_morning(&mut g);
    assert_eq!(g.clock.day_number(), 2);
    assert!(item_events.contains(&ItemEvent::CarOff));
    assert!(!g.gathering.bag.riding_car);
    assert_eq!(g.walk_speed(), PLAYER_WALK_SPEED);
    // 내린 뒤에는 다시 만들 수 있다.
    assert_eq!(craft(&mut g, Recipe::Car), Some(Recipe::Car));
}

#[test]
fn car_made_at_five_is_left_an_hour_later() {
    let mut g = new_game();
    g.clock.update(DAY_SECONDS + NIGHT_SECONDS * (11.0 / 12.0)); // 2일째 05:00
    assert_eq!(g.clock.hour_minute(), (5, 0));
    g.gathering.bag.riding_car = true;
    g.clock.update(NIGHT_SECONDS / 12.0 - 1.0); // 05:59쯤
    g.update(&idle(), 0.0);
    assert!(g.gathering.bag.riding_car);
    g.update_with_time_scale(&idle(), FRAME, 120.0); // 06:00을 넘긴다
    assert!(!g.gathering.bag.riding_car);
    assert_eq!(g.take_item_events(), vec![ItemEvent::CarOff]);
}

#[test]
fn riding_car_hides_car_from_the_result() {
    let mut g = new_game();
    g.gathering.bag.riding_car = true;
    fill_bag_for(&mut g.gathering.bag, Recipe::Car);
    g.crafting.open();
    place_shape(&mut g.crafting, Recipe::Car, &g.gathering.bag);
    assert_eq!(g.crafting.ready_recipe(&g.gathering.bag), None);
}

#[test]
fn fast_car_never_goes_into_boxes_on_500_maps() {
    const SEEDS: u32 = 500;
    const FRAMES: usize = 600; // 10초: 긴 맵을 한 바퀴 넘게 돈다
    for seed in 0..SEEDS {
        let world = World::long_map(seed);
        let mut player = Player::new(CharacterKind::Niro, Vec2::new(PLAYER_SPAWN_X, world.ground_y - PLAYER_HEIGHT));
        player.walk_speed = PLAYER_WALK_SPEED * CAR_SPEED_MULTIPLIER;
        for frame in 0..FRAMES {
            // 앞으로 달리며 가끔 점프한다.
            let input = PlayerInput { right: true, jump_pressed: frame % 45 == 0, ..Default::default() };
            player.update(&input, FRAME, &world);
            for wall in &world.obstacles {
                assert!(!really_overlaps(player.rect(), *wall), "seed {seed} frame {frame}: 상자 안에 들어갔다");
            }
        }
    }
}

#[test]
fn car_crosses_the_seam_without_a_break() {
    let mut g = new_game();
    g.gathering.bag.riding_car = true;
    stand_at(&mut g, MAP_WIDTH - 200.0, Facing::Right);
    let mut last_x = g.player.rect().center().x;
    for _ in 0..60 {
        g.update(&right(), FRAME);
        let x = g.player.rect().center().x;
        let step = logic::world::shortest_dx(last_x, x);
        assert!(step >= 0.0 && step <= 660.0 * FRAME + 1.0, "한 프레임에 {step}");
        last_x = x;
    }
    assert!(g.player.rect().center().x < MAP_WIDTH / 2.0, "이음매를 넘어 해변에 왔다");
}

#[test]
fn car_stops_at_the_end_of_the_cave() {
    let mut g = new_game();
    stand_at(&mut g, CAVE_ENTRANCE_X, Facing::Right);
    tap_e(&mut g);
    assert!(g.in_cave);
    g.gathering.bag.riding_car = true;
    run(&mut g, right(), 5.0);
    let body = g.player.rect();
    assert!(body.x + body.w <= g.cave_world.width + 0.01 && body.x >= 0.0, "동굴 안에 있다");
}

// ---- US-083 주사기총 ----

#[test]
fn switching_goes_through_weapons_only_even_with_syringe_gun_and_ender_fruit() {
    // 부모님 결정 2026-11-14: 주사기총·엔더열매는 바꾸기(W)에 없고 포션처럼 따로 쓴다.
    let mut g = new_game();
    g.weapons.add_bought(Weapon::Gun);
    g.weapons.hold(Weapon::Hands);
    g.gathering.bag.has_syringe_gun = true;
    g.gathering.bag.ender_fruits = 1;
    let mut seen = Vec::new();
    for _ in 0..4 {
        g.switch_weapon();
        seen.push(g.weapons.held());
    }
    assert_eq!(seen, vec![Weapon::Gun, Weapon::Hands, Weapon::Gun, Weapon::Hands]);
}

#[test]
fn syringe_gun_and_ender_fruit_alone_do_not_show_the_switch_button() {
    let mut g = new_game();
    g.gathering.bag.has_syringe_gun = true;
    g.gathering.bag.ender_fruits = 2;
    assert!(!g.can_switch(), "바꾸기는 무기만 센다");
}

#[test]
fn syringe_gun_clears_monsters_on_screen_once_for_500() {
    let mut g = new_game();
    stand_at(&mut g, BEACH_X, Facing::Right);
    craft(&mut g, Recipe::SyringeGun);
    g.weapons.add_bought(Weapon::Sword);
    put_zombie(&mut g, BEACH_X + 300.0);
    put_skeleton(&mut g, BEACH_X - 200.0); // 등 뒤도
    put_zombie(&mut g, MAP_WIDTH - 200.0); // 이음매 너머지만 화면 안
    put_zombie(&mut g, BEACH_X + 900.0); // 화면 밖
    let (score, experience, level) = (g.score(), g.combat.experience, g.combat.level);

    assert!(g.fire_syringe_gun());
    assert_eq!(monster_count(&g), 1, "화면 밖 몬스터는 그대로");
    assert_eq!(g.score(), score + SYRINGE_GUN_SCORE, "3마리여도 500 한 번");
    assert!(!g.gathering.bag.has_syringe_gun);
    assert_eq!(g.weapons.held(), Weapon::Sword, "든 무기는 그대로");
    assert!(matches!(g.take_item_events()[..], [ItemEvent::SyringeGunFired { monsters: 3, .. }]));
    // 쓰러졌어요는 다음 update가 돌려준다.
    let events = g.update(&idle(), FRAME);
    assert_eq!(defeated_shown(&events), 3, "쓰러졌어요 3번");
    assert_eq!((g.combat.experience, g.combat.level), (experience, level), "경험치 없음");
    assert_eq!(defeated_shown(&g.update(&idle(), FRAME)), 0, "한 번만 온다");
    assert!(!g.fire_syringe_gun(), "다 썼다");
}

#[test]
fn syringe_gun_with_no_monster_does_nothing() {
    let mut g = new_game();
    stand_at(&mut g, BEACH_X, Facing::Right);
    g.gathering.bag.has_syringe_gun = true;
    put_zombie(&mut g, BEACH_X + 900.0); // 화면 밖뿐
    let score = g.score();
    assert!(!g.fire_syringe_gun());
    assert!(g.gathering.bag.has_syringe_gun, "없어지지 않는다");
    assert_eq!(g.score(), score);
    assert!(g.take_item_events().is_empty());
}

#[test]
fn syringe_gun_is_not_used_while_a_window_is_open_or_the_game_is_over() {
    let mut g = new_game();
    stand_at(&mut g, BEACH_X, Facing::Right);
    g.gathering.bag.has_syringe_gun = true;
    put_zombie(&mut g, BEACH_X + 300.0);
    g.crafting.open();
    assert!(!g.fire_syringe_gun(), "제작대");
    g.crafting.close();
    g.shop.open();
    assert!(!g.fire_syringe_gun(), "상점");
    g.close_shop();
    g.math.add_waiting();
    g.math.open_next_waiting();
    assert!(g.math.is_open());
    assert!(!g.fire_syringe_gun(), "문제 창");
    let mut over = new_game();
    stand_at(&mut over, BEACH_X, Facing::Right);
    over.gathering.bag.has_syringe_gun = true;
    over.gathering.bag.ender_fruits = 1;
    put_zombie(&mut over, BEACH_X + 300.0);
    over.combat.health.lose_from_quiz(over.combat.health.max_hp());
    assert!(over.is_over());
    assert!(!over.fire_syringe_gun(), "판이 끝남");
    assert!(!over.eat_ender_fruit(), "판이 끝남");
    assert!(g.gathering.bag.has_syringe_gun);
    assert_eq!(monster_count(&g), 1);
    assert!(g.take_item_events().is_empty());
}

#[test]
fn attack_button_does_not_use_the_syringe_gun() {
    let mut g = new_game();
    stand_at(&mut g, BEACH_X, Facing::Right);
    g.gathering.bag.has_syringe_gun = true;
    put_zombie(&mut g, BEACH_X + 300.0);
    g.update(&attack(), FRAME);
    assert!(g.gathering.bag.has_syringe_gun);
    assert!(g.take_item_events().is_empty());
}

#[test]
fn syringe_gun_leaves_citizens_and_arrows_alone() {
    let mut g = new_game();
    let citizen_x = CITIZEN_XS[0];
    stand_at(&mut g, citizen_x - 200.0, Facing::Right);
    g.gathering.bag.has_syringe_gun = true;
    put_zombie(&mut g, citizen_x + 100.0);
    let citizens = g.citizens.list.len();
    assert!(g.fire_syringe_gun());
    g.update(&idle(), FRAME);
    assert_eq!(monster_count(&g), 0);
    assert_eq!(g.citizens.list.len(), citizens);
    assert!(g.citizens.list.iter().all(|c| c.rect.w > 0.0));
}

#[test]
fn only_one_syringe_gun_at_a_time() {
    let mut bag = Bag::default();
    bag.has_syringe_gun = true;
    fill_bag_for(&mut bag, Recipe::SyringeGun);
    let mut crafting = Crafting::default();
    place_shape(&mut crafting, Recipe::SyringeGun, &bag);
    assert_eq!(crafting.ready_recipe(&bag), None);
}

// ---- US-081 엔더열매 ----

#[test]
fn eating_ender_fruit_gives_power_and_cannot_be_doubled() {
    let mut g = new_game();
    craft(&mut g, Recipe::EnderFruit);
    craft(&mut g, Recipe::EnderFruit);
    assert_eq!(g.gathering.bag.ender_fruits, 2, "여러 개 가진다");
    assert!(g.eat_ender_fruit());
    assert!(g.gathering.bag.has_ender_power);
    assert_eq!(g.gathering.bag.ender_fruits, 1);
    assert!(matches!(g.take_item_events()[..], [ItemEvent::EnderFruitEaten { .. }]));
    assert!(!g.eat_ender_fruit());
    assert_eq!(g.gathering.bag.ender_fruits, 1, "벌써 먹었어요: 줄지 않는다");
    assert_eq!(g.take_item_events(), vec![ItemEvent::EnderFruitAlreadyEaten]);
}

#[test]
fn ender_fruit_is_not_eaten_without_one_or_while_a_window_is_open() {
    let mut g = new_game();
    assert!(!g.eat_ender_fruit(), "없다");
    assert!(g.take_item_events().is_empty());
    g.gathering.bag.ender_fruits = 1;
    g.crafting.open();
    assert!(!g.eat_ender_fruit(), "제작대");
    g.crafting.close();
    g.shop.open();
    assert!(!g.eat_ender_fruit(), "상점");
    g.close_shop();
    assert_eq!(g.gathering.bag.ender_fruits, 1);
    assert!(!g.gathering.bag.has_ender_power);
    assert!(g.take_item_events().is_empty());
    g.update(&attack(), FRAME);
    assert!(!g.gathering.bag.has_ender_power, "공격 버튼으로는 먹지 않는다");
}

#[test]
fn ender_power_lasts_through_morning_and_the_cave() {
    let mut g = new_game();
    g.gathering.bag.has_ender_power = true;
    run_until_morning(&mut g);
    assert!(g.gathering.bag.has_ender_power);
    stand_at(&mut g, CAVE_ENTRANCE_X, Facing::Right);
    tap_e(&mut g);
    assert!(g.in_cave);
    assert_eq!(g.slam_power_here(), None, "동굴에서는 쿵 하지 않는다");
    tap_e(&mut g); // 출구 앞이라 나간다
    assert!(!g.in_cave);
    assert!(g.gathering.bag.has_ender_power);
}

#[test]
fn ender_slam_clears_screen_and_gives_ten_diamonds_and_700() {
    let mut g = new_game();
    stand_at(&mut g, BEACH_X, Facing::Right);
    g.gathering.bag.has_ender_power = true;
    assert_eq!(g.slam_power_here(), Some(SlamPower::EnderFruit));
    put_zombie(&mut g, BEACH_X + 200.0);
    put_skeleton(&mut g, BEACH_X + 400.0);
    let (score, experience) = (g.score(), g.combat.experience);
    g.mine_held = true;
    let events = g.update(&idle(), FRAME);
    assert_eq!(defeated_shown(&events), 2);
    assert_eq!(monster_count(&g), 0);
    assert_eq!(g.gathering.bag.count(Material::Diamond), ENDER_FRUIT_SLAM_DIAMONDS);
    assert!(!g.gathering.bag.has_ender_power, "힘이 사라진다");
    assert_eq!(g.score(), score + ENDER_FRUIT_SLAM_SCORE);
    assert_eq!(g.combat.experience, experience);
    assert!(matches!(
        g.take_item_events()[..],
        [ItemEvent::Slammed { power: SlamPower::EnderFruit, monsters: 2, diamonds: 10, .. }]
    ));
}

#[test]
fn slam_without_monsters_gives_diamonds_but_no_score() {
    let mut g = new_game();
    stand_at(&mut g, BEACH_X, Facing::Right);
    g.gathering.bag.has_ender_power = true;
    let score = g.score();
    tap_e(&mut g);
    assert_eq!(g.gathering.bag.count(Material::Diamond), ENDER_FRUIT_SLAM_DIAMONDS);
    assert_eq!(g.score(), score, "몬스터가 없으면 점수 그대로 (부모 확인 필요)");
}

#[test]
fn no_slam_off_the_beach() {
    let mut g = new_game();
    g.gathering.bag.has_ender_power = true;
    for x in [SLAM_BEACH_LEFT - 10.0, SLAM_BEACH_RIGHT + 30.0, 1500.0] {
        stand_at(&mut g, x, Facing::Right);
        assert_eq!(g.slam_power_here(), None, "x={x}");
        tap_e(&mut g);
        assert!(g.gathering.bag.has_ender_power, "x={x}: 아무 일이 없다");
    }
}

#[test]
fn ender_power_goes_first_and_the_diamond_pickaxe_stays() {
    let mut g = new_game();
    stand_at(&mut g, BEACH_X, Facing::Right);
    g.gathering.bag.has_ender_power = true;
    g.gathering.bag.has_diamond_pickaxe = true;
    tap_e(&mut g);
    assert!(!g.gathering.bag.has_ender_power);
    assert!(g.gathering.bag.has_diamond_pickaxe);
    assert_eq!(g.slam_power_here(), Some(SlamPower::DiamondPickaxe));
}

#[test]
fn tree_near_the_beach_end_is_mined_not_slammed() {
    let mut g = new_game();
    g.gathering.bag.has_ender_power = true;
    // 셋째 나무(x=900)에 닿는 해변 끝: 쓸 것이 먼저라 쿵이 없다.
    stand_at(&mut g, SLAM_BEACH_RIGHT, Facing::Right);
    assert!(matches!(g.usable_in_reach(), Some(Usable::Tree(_))));
    assert_eq!(g.slam_power_here(), None);
}

// ---- US-082 다이아래더라이트 곡괭이 ----

#[test]
fn diamond_pickaxe_mines_in_half_a_second() {
    let mut bag = Bag::default();
    bag.has_diamond_pickaxe = true;
    assert_eq!(bag.mine_seconds(), MINE_SECONDS_DIAMOND_PICKAXE);
    bag.has_pickaxe = true;
    assert_eq!(bag.mine_seconds(), MINE_SECONDS_DIAMOND_PICKAXE, "빠른 쪽으로 캔다");

    // 빨리감기를 켜도 실제 0.5초다.
    let mut g = new_game();
    g.gathering.bag.has_diamond_pickaxe = true;
    stand_at(&mut g, TREE_XS[1] - TREE_WIDTH / 2.0 - PLAYER_WIDTH / 2.0 - 10.0, Facing::Right);
    g.mine_held = true;
    let mut seconds = 0.0;
    for _ in 0..120 {
        g.update_with_time_scale(&idle(), FRAME, DEV_FAST_FORWARD_SCALE);
        seconds += FRAME;
        if g.gathering.bag.count(Material::Wood) > 0 {
            break;
        }
    }
    assert_eq!(g.gathering.bag.count(Material::Wood), 1);
    assert!((seconds - MINE_SECONDS_DIAMOND_PICKAXE).abs() < 0.05, "{seconds}초");
}

#[test]
fn pickaxe_slam_gives_one_diamond_and_the_pickaxe_is_gone() {
    let mut g = new_game();
    stand_at(&mut g, BEACH_X, Facing::Right);
    craft(&mut g, Recipe::Pickaxe);
    assert_eq!(craft(&mut g, Recipe::DiamondPickaxe), Some(Recipe::DiamondPickaxe), "보통 곡괭이가 있어도 만든다");
    put_zombie(&mut g, BEACH_X + 100.0);
    let score = g.score();
    tap_e(&mut g);
    assert_eq!(monster_count(&g), 0);
    assert_eq!(g.gathering.bag.count(Material::Diamond), DIAMOND_PICKAXE_SLAM_DIAMONDS);
    assert!(!g.gathering.bag.has_diamond_pickaxe, "없어진다");
    assert!(g.gathering.bag.has_pickaxe, "보통 곡괭이는 남는다");
    assert_eq!(g.gathering.bag.mine_seconds(), MINE_SECONDS_PICKAXE);
    assert_eq!(g.score(), score + DIAMOND_PICKAXE_SLAM_SCORE);
    // 없어진 뒤 다시 만들 수 있다.
    assert_eq!(craft(&mut g, Recipe::DiamondPickaxe), Some(Recipe::DiamondPickaxe));
}

#[test]
fn pickaxe_slam_without_monsters_still_uses_it_up() {
    let mut g = new_game();
    stand_at(&mut g, BEACH_X, Facing::Right);
    g.gathering.bag.has_diamond_pickaxe = true;
    let score = g.score();
    tap_e(&mut g);
    assert!(!g.gathering.bag.has_diamond_pickaxe);
    assert_eq!(g.gathering.bag.count(Material::Diamond), 1);
    assert_eq!(g.score(), score);
}

#[test]
fn new_game_has_none_of_the_new_things() {
    let g = new_game();
    let bag = &g.gathering.bag;
    assert!(!bag.riding_car && !bag.has_syringe_gun && !bag.has_diamond_pickaxe && !bag.has_ender_power);
    assert_eq!(bag.ender_fruits, 0);
    assert_eq!(g.crafting.chosen(), Recipe::Pickaxe);
}
