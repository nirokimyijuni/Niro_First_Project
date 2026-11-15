//! M8 상점 시험 (수산나): 집 문, 멈춤, 사기, 스케이트, 갑옷, 포션, 독약, 광물 팔기.
//! 기준: stories/backlog.md M8 맨 위 표, US-062, US-070, US-071, US-072, docs/02-plan.md D65·D71.
//! 숫자는 config 상수로 적어서 임시 값을 바꿔도 시험을 고치지 않아도 된다. 반복은 모두 횟수가 정해져 있다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::combat::CombatEvent;
use logic::game::{Game, GameEvent, Usable};
use logic::gathering::Material;
use logic::monster::Zombie;
use logic::player::{Facing, Player, PlayerInput};
use logic::shop::{Armor, CantBuy, CantSell, Potion, ShopEvent, ShopItem};
use logic::weapon::{Weapon, Weapons};
use logic::world::{really_overlaps, World};
use macroquad::math::Vec2;

const FRAME: f32 = 1.0 / 60.0;
const SEED: u32 = 7;
const MANY_SEEDS: u32 = 300;
/// 프레임을 더한 시간은 f32 오차로 살짝 모자랄 수 있다.
const A_LITTLE: f32 = 0.05;

fn new_game() -> Game {
    Game::new(CharacterKind::Niro, SEED)
}

/// 상자·미스·시민·집 문이 없는 넓은 맵. 포션·갑옷·스케이트 시험에 쓴다.
fn empty_game() -> Game {
    Game::on_world(CharacterKind::Niro, SEED, World { width: 3000.0, ground_y: GROUND_Y, obstacles: Vec::new() })
}

fn idle() -> PlayerInput {
    PlayerInput::default()
}

fn attack() -> PlayerInput {
    PlayerInput { attack_pressed: true, ..Default::default() }
}

/// `seconds`초(실제) 동안 같은 입력으로 프레임을 돌린다.
fn run(g: &mut Game, input: PlayerInput, seconds: f32) {
    for _ in 0..(seconds / FRAME).ceil() as usize {
        g.update(&input, FRAME);
    }
}

/// 빨리감기 `scale`로 실제 `seconds`초를 돌린다.
fn run_fast(g: &mut Game, seconds: f32, scale: f32) {
    for _ in 0..(seconds / FRAME).ceil() as usize {
        g.update_with_time_scale(&idle(), FRAME, scale);
    }
}

/// 게임 속 시간을 `game_seconds`초쯤 흘린다. 실제 시간은 아주 조금만 흘러 몬스터가 거의 움직이지 않는다.
/// 수학 문제가 열리면 맞히고 닫는다. 한 걸음은 게임 속 1초이고 걸음 수는 정해져 있다.
fn pass_game_time(g: &mut Game, game_seconds: f32) {
    let real_step = 0.001;
    for _ in 0..game_seconds.round() as usize {
        answer_quiz_if_open(g);
        g.update_with_time_scale(&idle(), real_step, 1.0 / real_step);
    }
    answer_quiz_if_open(g);
}

fn answer_quiz_if_open(g: &mut Game) {
    if let Some(answer) = g.math.question().map(|q| q.answer) {
        g.math.type_and_submit(answer);
        g.update(&idle(), MATH_RIGHT_SHOW_SECONDS + A_LITTLE);
        assert!(!g.math.is_open());
    }
}

/// 몸 가운데가 `center_x`이고 `facing`을 보며 땅에 서게 한다.
fn stand_at(g: &mut Game, center_x: f32, facing: Facing) {
    g.player.pos = Vec2::new(center_x - PLAYER_WIDTH / 2.0, GROUND_Y - PLAYER_HEIGHT);
    g.player.vel = Vec2::ZERO;
    g.player.facing = facing;
    g.update(&idle(), 0.0);
}

/// E 키(캐기 버튼)를 한 번 눌렀다 뗀다.
fn tap_e(g: &mut Game) {
    g.mine_held = true;
    g.update(&idle(), FRAME);
    g.mine_held = false;
    g.update(&idle(), FRAME);
}

fn enter_shop(g: &mut Game) {
    stand_at(g, HOUSE_DOOR_X, Facing::Right);
    tap_e(g);
    assert!(g.shop.is_open());
}

fn give_coins(g: &mut Game, amount: u32) {
    g.wallet.add(amount);
}

/// 몸 가운데가 주인공 몸 가운데에서 오른쪽으로 `distance`인 좀비 (같은 땅이라 키 가운데도 같다).
fn zombie_at_distance(g: &Game, distance: f32) -> Zombie {
    let center_x = g.player.rect().center().x + distance;
    Zombie::new(center_x - ZOMBIE_WIDTH / 2.0, GROUND_Y, false)
}

/// 밤으로 건너뛴다 (밤 문제는 열리지 않는다).
fn skip_to_night(g: &mut Game) {
    g.clock.update(DAY_SECONDS);
    g.update(&idle(), 0.0);
}

// ---- US-062 집에 들어가기 ----

#[test]
fn door_is_the_thing_to_use_in_front_of_the_house() {
    let mut g = new_game();
    stand_at(&mut g, HOUSE_DOOR_X, Facing::Right);
    assert_eq!(g.usable_in_reach(), Some(Usable::HouseDoor));
    assert_eq!(Usable::HouseDoor.button_text(), "들어가기");
    assert_eq!(HOUSE_DOOR_BUTTON_TEXT, "들어가기");
}

#[test]
fn e_opens_the_shop_once_even_when_held() {
    let mut g = new_game();
    stand_at(&mut g, HOUSE_DOOR_X, Facing::Left);
    g.mine_held = true;
    g.update(&idle(), FRAME);
    assert!(g.shop.is_open());
    assert_eq!(g.take_shop_events(), vec![ShopEvent::Opened]);
    g.close_shop();
    // 꾹 누르고 있으면 다시 열리지 않는다.
    run(&mut g, idle(), 1.0);
    assert!(!g.shop.is_open());
    assert!(g.take_shop_events().is_empty());
}

#[test]
fn away_from_the_door_e_does_nothing() {
    let mut g = new_game();
    for x in [HOUSE_DOOR_X - HOUSE_DOOR_WIDTH / 2.0 - PLAYER_WIDTH / 2.0 - MINE_REACH - 10.0, HOUSE_DOOR_X + 200.0] {
        stand_at(&mut g, x, Facing::Right);
        assert_ne!(g.usable_in_reach(), Some(Usable::HouseDoor));
        tap_e(&mut g);
        assert!(!g.shop.is_open());
    }
}

/// 부모님 결정 2026-11-15: 문 바로 앞(문 폭 + 좌우 `HOUSE_DOOR_REACH`)에서만 들어간다. 바라보는 쪽은 상관없다.
#[test]
fn door_opens_only_right_in_front_of_it() {
    let door_edge = HOUSE_DOOR_WIDTH / 2.0 + HOUSE_DOOR_REACH;
    for x in [HOUSE_DOOR_X - door_edge + 1.0, HOUSE_DOOR_X, HOUSE_DOOR_X + door_edge - 1.0] {
        for facing in [Facing::Left, Facing::Right] {
            let mut g = new_game();
            stand_at(&mut g, x, facing);
            assert_eq!(g.usable_in_reach(), Some(Usable::HouseDoor), "문 앞 x={x}");
            tap_e(&mut g);
            assert!(g.shop.is_open(), "문 앞 x={x}");
        }
    }
}

/// 집 벽 앞(문 밖, 예전엔 `MINE_REACH` 때문에 들어가던 자리)은 안 들어가고, ↑는 점프다.
#[test]
fn house_wall_beside_the_door_does_not_open_and_up_jumps() {
    let door_edge = HOUSE_DOOR_WIDTH / 2.0 + HOUSE_DOOR_REACH;
    for x in [HOUSE_DOOR_X - door_edge - 2.0, HOUSE_DOOR_X + door_edge + 2.0, HOUSE_DOOR_X + door_edge + MINE_REACH] {
        let mut g = new_game();
        stand_at(&mut g, x, Facing::Right);
        assert_ne!(g.usable_in_reach(), Some(Usable::HouseDoor), "벽 앞 x={x}");
        tap_e(&mut g);
        assert!(!g.shop.is_open(), "벽 앞 x={x}");
        // 화면은 문이 닿지 않으면 ↑를 점프로 보낸다 (view::input::up_enters).
        let ground_y = g.player.pos.y;
        g.update(&PlayerInput { jump_pressed: true, ..Default::default() }, FRAME);
        run(&mut g, idle(), 0.1);
        assert!(g.player.pos.y < ground_y, "↑는 점프 x={x}");
        assert!(!g.shop.is_open());
    }
}

#[test]
fn door_is_only_on_the_long_map() {
    assert!(World::long_map(SEED).house_door().is_some());
    assert!(World::m1().house_door().is_none());
}

#[test]
fn no_box_near_the_door_on_any_seed() {
    for seed in 0..MANY_SEEDS {
        let w = World::long_map(seed);
        let door = w.house_door().unwrap();
        for b in w.boxes() {
            let gap = (b.x - (door.x + door.w)).max(door.x - (b.x + b.w));
            assert!(gap >= NO_BOX_MARGIN, "시드 {seed}: 상자 {b:?}가 문 {door:?}에 가깝다");
        }
    }
}

/// 부모님 결정 2026-11-15: 동굴 입구(화산) 둘레 `BOX_CLEAR_AROUND_CAVE`, 집 문(집) 둘레 `BOX_CLEAR_AROUND_DOOR` 안에는 상자가 없다.
/// 상자 사이 틈 규칙(붙음 또는 `BOX_MIN_GAP` 이상)은 그대로이고, 집에는 상자가 하나 이상 남는다 (화산은 0개 허용).
/// 부서진 상자는 같은 네모로 돌아오므로 배치만 보면 06:00에도 그 둘레는 비어 있다.
#[test]
fn no_box_around_cave_entrance_and_house_door_on_any_seed() {
    const { assert!(BOX_CLEAR_AROUND_CAVE >= PLAYER_WIDTH * 2.0 + MINE_REACH) };
    const { assert!(BOX_CLEAR_AROUND_DOOR >= PLAYER_WIDTH * 2.0 + HOUSE_DOOR_REACH) };
    for seed in 0..MANY_SEEDS {
        let w = World::long_map(seed);
        let cave = w.cave_entrance().unwrap();
        let door = w.house_door().unwrap();
        assert_eq!(w.place_name_at(cave.center().x), "화산");
        assert_eq!(w.place_name_at(door.center().x), "집");
        let mut boxes = w.boxes();
        for b in &boxes {
            for (spot, clear) in [(cave, BOX_CLEAR_AROUND_CAVE), (door, BOX_CLEAR_AROUND_DOOR)] {
                let gap = (b.x - (spot.x + spot.w)).max(spot.x - (b.x + b.w));
                assert!(gap >= clear, "시드 {seed}: 상자 {b:?}가 {spot:?}에 가깝다 (틈 {gap})");
            }
        }
        // 화산은 0개여도 된다 (부모님 결정 2026-11-15). 집에는 하나 이상 남는다.
        let house = w.place_index_at(door.center().x);
        assert!(boxes.iter().any(|b| w.place_index_at(b.center().x) == house), "시드 {seed}: 집에 상자가 없다");
        boxes.sort_by(|a, b| a.x.total_cmp(&b.x));
        for pair in boxes.windows(2) {
            let gap = pair[1].x - (pair[0].x + pair[0].w);
            assert!(gap == 0.0 || gap >= BOX_MIN_GAP, "시드 {seed}: 틈 {gap}, {pair:?}");
        }
    }
}

#[test]
fn door_is_far_from_the_other_things_to_use() {
    let w = World::long_map(SEED);
    let door = w.house_door().unwrap();
    let others = w.trees().into_iter().chain(w.workbench()).chain(w.cave_entrance());
    for other in others {
        let gap = (other.x - (door.x + door.w)).max(door.x - (other.x + other.w));
        assert!(gap > NO_BOX_MARGIN, "{other:?}");
    }
    for x in CITIZEN_XS {
        assert!((x - HOUSE_DOOR_X).abs() > NO_BOX_MARGIN);
    }
    assert!(MAP_WIDTH - (door.x + door.w) > NO_BOX_MARGIN, "이음매");
}

#[test]
fn shop_opens_at_night_too() {
    let mut g = new_game();
    skip_to_night(&mut g);
    assert!(g.clock.is_night());
    enter_shop(&mut g);
}

#[test]
fn everything_stops_while_the_shop_is_open_even_fast_forward() {
    let mut g = new_game();
    skip_to_night(&mut g);
    enter_shop(&mut g);
    give_coins(&mut g, RAINBOW_ARMOR_PRICE);
    g.buy(ShopItem::RainbowArmor).unwrap();
    g.combat.zombies.list.clear();
    g.combat.skeletons.clear();
    g.combat.zombies.list.push(zombie_at_distance(&g, 300.0));
    let clock_before = g.clock.seconds_left_in_phase();
    let zombie_before = g.combat.zombies.list[0].pos;
    let armor_before = g.shop.armor_seconds_left();
    let player_before = g.player.pos;
    run_fast(&mut g, 5.0, DEV_FAST_FORWARD_SCALE);
    run(&mut g, PlayerInput { right: true, ..Default::default() }, 1.0);
    assert_eq!(g.clock.seconds_left_in_phase(), clock_before);
    assert_eq!(g.combat.zombies.list.len(), 1, "새 몬스터도 안 나온다");
    assert_eq!(g.combat.zombies.list[0].pos, zombie_before);
    assert_eq!(g.shop.armor_seconds_left(), armor_before);
    assert_eq!(g.player.pos, player_before, "걷지 않는다");
    assert_eq!(g.usable_in_reach(), None);

    // 닫으면 멈춘 때부터 이어진다. 문 앞 그대로다.
    g.close_shop();
    g.update(&idle(), FRAME);
    assert_eq!(g.player.pos, player_before);
    assert!(g.clock.seconds_left_in_phase() < clock_before);
    assert!(g.shop.armor_seconds_left() < armor_before);
}

// ---- US-062 사기 ----

#[test]
fn prices_are_the_ones_the_child_chose() {
    let g = new_game();
    let rows: Vec<(&str, u32)> = g.buy_rows().iter().map(|r| (r.name, r.price)).collect();
    assert_eq!(
        rows,
        vec![
            // 칼·도끼·총은 팔지 않는다 (부모님 결정 2026-11-14).
            ("체력 포션", 100),
            ("독약 포션", 100),
            ("스케이트", 200),
            ("무지개 갑옷", 500),
            ("다이아몬드 갑옷", 1000)
        ]
    );
}

#[test]
fn not_enough_coins_buys_nothing() {
    let mut g = new_game();
    give_coins(&mut g, SKATES_PRICE - 1);
    assert_eq!(g.buy(ShopItem::Skates), Err(CantBuy::NotEnoughCoins));
    assert_eq!(CantBuy::NotEnoughCoins.text(), "코인이 모자라요");
    assert_eq!(g.wallet.coins(), SKATES_PRICE - 1);
    assert!(!g.shop.has_skates());
    let skates_row = g.buy_rows().into_iter().find(|r| r.item == ShopItem::Skates).unwrap();
    assert!(!skates_row.can_buy);
    assert_eq!(skates_row.reason, Some(CantBuy::NotEnoughCoins));
}

#[test]
fn buying_spends_coins_but_not_score() {
    let mut g = new_game();
    give_coins(&mut g, 100);
    g.coins.picked_total = 100;
    let score = g.score();
    g.buy(ShopItem::HealthPotion).unwrap();
    assert_eq!(g.wallet.coins(), 100 - HEALTH_POTION_PRICE);
    assert_eq!(g.score(), score);
}

#[test]
fn shop_sells_no_weapons() {
    // 부모님 결정 2026-11-14: 칼·도끼·총은 미스·필드에서만 얻는다.
    let mut g = new_game();
    give_coins(&mut g, COIN_MAX);
    for item in ShopItem::ALL {
        let _ = g.buy(item);
    }
    assert!(g.weapons.owned().is_empty(), "무엇을 사도 무기는 생기지 않는다");
    assert_eq!(g.weapons.held(), Weapon::Hands);
    assert_eq!(g.buy_rows().len(), 5);
    assert_eq!(CantBuy::AlreadyHave.text(), "가지고 있어요");
}

#[test]
fn owned_weapons_are_skipped_by_miss() {
    let w = World::long_map(SEED);
    let mut weapons = Weapons::new(SEED);
    weapons.add_bought(Weapon::Sword);
    weapons.drop_one(500.0, &w);
    assert_eq!(weapons.dropped[0].kind, Weapon::Axe, "칼이 있으니 도끼부터");
}

#[test]
fn owned_gun_never_comes_from_miss() {
    let w = World::long_map(SEED);
    for seed in 0..MANY_SEEDS {
        let mut weapons = Weapons::new(seed);
        weapons.add_bought(Weapon::Gun);
        for _ in 0..20 {
            weapons.drop_one(500.0, &w);
        }
        assert!(weapons.dropped.iter().all(|d| d.kind != Weapon::Gun), "시드 {seed}");
    }
}

#[test]
fn shop_items_do_not_count_as_math_items() {
    let mut g = new_game();
    give_coins(&mut g, COIN_MAX);
    for item in ShopItem::ALL {
        let _ = g.buy(item);
    }
    assert_eq!(g.weapons.picked_up_count, 0);
    run(&mut g, idle(), 1.0);
    assert!(!g.math.is_open());
}

#[test]
fn new_game_has_no_coins_or_bought_things() {
    let g = new_game();
    assert_eq!(g.wallet.coins(), 0);
    assert!(!g.shop.has_skates());
    assert_eq!(g.shop.armor(), None);
    assert_eq!(g.shop.potions(Potion::Health) + g.shop.potions(Potion::Poison), 0);
    assert!(!g.shop.is_open());
}

// ---- US-070 스케이트 ----

/// 오른쪽으로 1초 걸은 거리.
fn walk_one_second(g: &mut Game) -> f32 {
    stand_at(g, 500.0, Facing::Right);
    let start = g.player.pos.x;
    run(g, PlayerInput { right: true, ..Default::default() }, 1.0);
    g.player.pos.x - start
}

#[test]
fn skates_make_walking_one_and_a_half_times_faster() {
    let mut g = empty_game();
    let normal = walk_one_second(&mut g);
    give_coins(&mut g, SKATES_PRICE);
    g.buy(ShopItem::Skates).unwrap();
    assert!(g.shop.has_skates());
    let skating = walk_one_second(&mut g);
    assert!((normal - PLAYER_WALK_SPEED).abs() < 5.0, "{normal}");
    assert!((skating - PLAYER_WALK_SPEED * SKATES_SPEED_MULTIPLIER).abs() < 5.0, "{skating}");
    assert_eq!(PLAYER_WALK_SPEED * SKATES_SPEED_MULTIPLIER, 330.0);
}

#[test]
fn skates_do_not_change_jump_height() {
    let mut g = empty_game();
    stand_at(&mut g, 500.0, Facing::Right);
    run(&mut g, idle(), 0.1);
    assert!(g.player.on_ground);
    let jump =PlayerInput { jump_pressed: true, ..Default::default() };
    let highest = |g: &mut Game| {
        g.update(&jump, FRAME);
        let mut top = g.player.pos.y;
        for _ in 0..120 {
            g.update(&idle(), FRAME);
            top = top.min(g.player.pos.y);
        }
        top
    };
    let normal = highest(&mut g);
    give_coins(&mut g, SKATES_PRICE);
    g.buy(ShopItem::Skates).unwrap();
    assert_eq!(highest(&mut g), normal);
}

#[test]
fn skating_never_gets_stuck_in_a_box() {
    // 시드 500개: 스케이트 빠르기로 뛰며 고리 맵을 한 바퀴 넘게 돈다. 상자 안에 끼지 않고, 상자에 막히기도 한다.
    let mut blocked_somewhere = false;
    for seed in 0..500 {
        let w = World::long_map(seed);
        for right in [true, false] {
            let mut p = Player::new(CharacterKind::Logan, Vec2::new(PLAYER_SPAWN_X, GROUND_Y - PLAYER_HEIGHT));
            p.walk_speed = PLAYER_WALK_SPEED * SKATES_SPEED_MULTIPLIER;
            for frame in 0..900 {
                let input = PlayerInput { right, left: !right, jump_pressed: frame % 50 == 0, ..Default::default() };
                let before = p.pos.x;
                p.update(&input, FRAME, &w);
                for b in w.boxes() {
                    assert!(!really_overlaps(p.rect(), b), "시드 {seed}: {:?}가 상자 {b:?}에 끼었다", p.rect());
                }
                blocked_somewhere |= p.on_ground && (p.pos.x - before).abs() < 0.01;
            }
        }
    }
    assert!(blocked_somewhere, "상자에 막히는 일도 있다");
}

#[test]
fn skates_stay_on_until_the_next_six_am() {
    // 1일째 10:00에 사면 2일째 06:00에 벗겨진다.
    let mut g = empty_game();
    g.clock.update(4.0 * 60.0 / 2.4 + 0.1);
    assert_eq!(g.clock.hour_minute().0, 10);
    give_coins(&mut g, SKATES_PRICE);
    g.buy(ShopItem::Skates).unwrap();
    assert_eq!(g.buy(ShopItem::Skates), Err(CantBuy::AlreadyHave));
    let until_six = g.clock.seconds_left_in_phase() + NIGHT_SECONDS;
    pass_game_time(&mut g, until_six - 2.0);
    assert_eq!(g.clock.day_number(), 1);
    assert!(g.shop.has_skates(), "아직 05:59쯤");
    pass_game_time(&mut g, 3.0);
    assert_eq!(g.clock.day_number(), 2);
    assert!(!g.shop.has_skates(), "06:00에 벗겨진다");
    assert!(g.take_shop_events().contains(&ShopEvent::SkatesOff));
    assert_eq!(g.player.walk_speed, PLAYER_WALK_SPEED);
    // 벗겨지면 다시 살 수 있다.
    give_coins(&mut g, SKATES_PRICE);
    assert_eq!(g.buy(ShopItem::Skates), Ok(()));
}

#[test]
fn skates_bought_at_five_am_come_off_an_hour_later() {
    let mut g = empty_game();
    g.clock.update(DAY_SECONDS + 11.0 * 60.0 / 1.2);
    assert_eq!(g.clock.hour_minute(), (5, 0));
    g.update(&idle(), 0.0);
    give_coins(&mut g, SKATES_PRICE);
    g.buy(ShopItem::Skates).unwrap();
    let one_hour = 60.0 / 1.2;
    pass_game_time(&mut g, one_hour - 2.0);
    assert!(g.shop.has_skates());
    pass_game_time(&mut g, 3.0);
    assert!(!g.shop.has_skates());
}

// ---- US-070 갑옷 ----

#[test]
fn armor_times_are_twelve_minutes_and_seventy_two_seconds() {
    assert_eq!(DIAMOND_ARMOR_SECONDS, 720.0);
    assert_eq!(RAINBOW_ARMOR_SECONDS, 72.0);
    assert_eq!(Armor::Diamond.seconds(), Armor::Rainbow.seconds() * 10.0);
}

#[test]
fn armor_is_put_on_right_away_and_runs_out_in_game_time() {
    let mut g = empty_game();
    give_coins(&mut g, RAINBOW_ARMOR_PRICE);
    g.buy(ShopItem::RainbowArmor).unwrap();
    assert_eq!(g.shop.armor(), Some(Armor::Rainbow));
    // 빨리감기 ×10이면 실제 약 7.2초.
    run_fast(&mut g, RAINBOW_ARMOR_SECONDS / DEV_FAST_FORWARD_SCALE - 0.2, DEV_FAST_FORWARD_SCALE);
    assert_eq!(g.shop.armor(), Some(Armor::Rainbow));
    run_fast(&mut g, 0.2 + A_LITTLE, DEV_FAST_FORWARD_SCALE);
    assert_eq!(g.shop.armor(), None);
    assert!(g.take_shop_events().contains(&ShopEvent::ArmorOff));
    // 갑옷이 끝나면 다시 맞는다.
    let hp = g.combat.health.hp();
    let (at, player) = (g.player.rect().center(), g.player.rect());
    g.combat.citizen_blast(at, player);
    assert!(g.combat.health.hp() < hp);
}

#[test]
fn turning_fast_forward_off_and_on_changes_how_fast_armor_runs_out() {
    let mut g = empty_game();
    give_coins(&mut g, RAINBOW_ARMOR_PRICE);
    g.buy(ShopItem::RainbowArmor).unwrap();
    run_fast(&mut g, 3.0, DEV_FAST_FORWARD_SCALE);
    let after_fast = g.shop.armor_seconds_left();
    assert!((after_fast - (RAINBOW_ARMOR_SECONDS - 3.0 * DEV_FAST_FORWARD_SCALE)).abs() < 0.5, "{after_fast}");
    run_fast(&mut g, 2.0, 1.0);
    assert!((g.shop.armor_seconds_left() - (after_fast - 2.0)).abs() < 0.1);
}

#[test]
fn buying_armor_again_restarts_with_the_new_armor() {
    let mut g = empty_game();
    give_coins(&mut g, DIAMOND_ARMOR_PRICE + RAINBOW_ARMOR_PRICE);
    g.buy(ShopItem::DiamondArmor).unwrap();
    run(&mut g, idle(), 1.0);
    g.buy(ShopItem::RainbowArmor).unwrap();
    assert_eq!(g.shop.armor(), Some(Armor::Rainbow));
    assert_eq!(g.shop.armor_seconds_left(), RAINBOW_ARMOR_SECONDS);
}

#[test]
fn armor_time_stops_in_quiz_and_when_down() {
    let mut g = empty_game();
    give_coins(&mut g, DIAMOND_ARMOR_PRICE);
    g.buy(ShopItem::DiamondArmor).unwrap();
    g.math.add_waiting();
    g.math.open_next_waiting();
    assert!(g.math.is_open());
    run_fast(&mut g, 3.0, DEV_FAST_FORWARD_SCALE);
    assert_eq!(g.shop.armor_seconds_left(), DIAMOND_ARMOR_SECONDS, "문제 창");
    answer_quiz_if_open(&mut g);

    let before = g.shop.armor_seconds_left();
    g.combat.health.lose_from_quiz(1000);
    assert!(g.is_over());
    run_fast(&mut g, 3.0, DEV_FAST_FORWARD_SCALE);
    assert_eq!(g.shop.armor_seconds_left(), before, "쓰러진 화면");
}

#[test]
fn armor_blocks_zombies_and_pops() {
    for armored in [false, true] {
        let mut g = empty_game();
        skip_to_night(&mut g);
        if armored {
            give_coins(&mut g, DIAMOND_ARMOR_PRICE);
            g.buy(ShopItem::DiamondArmor).unwrap();
        }
        let hp = g.combat.health.hp();
        g.combat.zombies.list.push(zombie_at_distance(&g, 0.0));
        assert!(really_overlaps(g.combat.zombies.list[0].rect(), g.player.rect()));
        run(&mut g, idle(), 0.5);
        let at = g.player.rect().center();
        let player = g.player.rect();
        g.combat.citizen_blast(at, player);
        if armored {
            assert_eq!(g.combat.health.hp(), hp, "갑옷이면 줄지 않는다");
            assert!(!g.combat.health.is_invincible(), "맞은 뒤 깜빡임과 구별된다");
        } else {
            assert!(g.combat.health.hp() < hp);
        }
    }
}

// ---- US-071 포션과 독약 포션 ----

#[test]
fn switching_goes_through_weapons_only() {
    // 부모님 결정: 포션은 바꾸기(W)에 없고 다른 키로 바로 먹는다.
    let mut g = empty_game();
    give_coins(&mut g, HEALTH_POTION_PRICE + POISON_POTION_PRICE);
    g.buy(ShopItem::HealthPotion).unwrap();
    g.buy(ShopItem::PoisonPotion).unwrap();
    g.weapons.add_bought(Weapon::Sword);
    assert!(g.can_switch());
    assert_eq!(g.weapons.held(), Weapon::Sword);
    let mut seen = Vec::new();
    for _ in 0..4 {
        g.switch_weapon();
        seen.push(g.weapons.held());
    }
    assert_eq!(seen, vec![Weapon::Hands, Weapon::Sword, Weapon::Hands, Weapon::Sword]);
}

#[test]
fn potions_alone_do_not_show_the_switch_button() {
    let mut g = empty_game();
    give_coins(&mut g, HEALTH_POTION_PRICE + POISON_POTION_PRICE);
    g.buy(ShopItem::HealthPotion).unwrap();
    g.buy(ShopItem::PoisonPotion).unwrap();
    assert!(!g.can_switch(), "바꾸기는 무기 기준");
}

#[test]
fn holding_potions_does_not_stop_the_sword_attack() {
    // REVIEW-019 1번이 다시 생기지 않게: 포션을 가진 채(체력 가득) 칼을 주워도 Q는 칼 공격이다.
    let mut g = empty_game();
    give_coins(&mut g, HEALTH_POTION_PRICE);
    g.buy(ShopItem::HealthPotion).unwrap();
    g.weapons.add_bought(Weapon::Sword);
    assert_eq!(g.weapons.held(), Weapon::Sword);
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP);
    stand_at(&mut g, 1000.0, Facing::Right);
    g.combat.zombies.list.push(zombie_at_distance(&g, ZOMBIE_WIDTH / 2.0 + PLAYER_WIDTH / 2.0));
    let events = g.update(&attack(), FRAME);
    assert!(
        events.iter().any(|e| matches!(e, GameEvent::Combat(CombatEvent::ZombieHit { .. } | CombatEvent::ZombieDefeated { .. }))),
        "{events:?}"
    );
    assert_eq!(g.shop.potions(Potion::Health), 1, "공격으로 포션이 쓰이지 않는다");
    assert!(g.take_shop_events().is_empty());
}

#[test]
fn health_potion_heals_five_without_going_over_max() {
    let mut g = empty_game();
    give_coins(&mut g, HEALTH_POTION_PRICE * 2);
    g.buy(ShopItem::HealthPotion).unwrap();
    g.buy(ShopItem::HealthPotion).unwrap();

    // 가득이면 아무 일도 없고 줄지 않는다.
    assert!(!g.use_potion(Potion::Health));
    assert_eq!(g.shop.potions(Potion::Health), 2);
    assert!(g.take_shop_events().is_empty());

    g.combat.health.lose_from_quiz(6);
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP - 6);
    assert!(g.use_potion(Potion::Health));
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP - 6 + HEALTH_POTION_HEAL);
    assert_eq!(g.shop.potions(Potion::Health), 1);
    let events = g.take_shop_events();
    assert!(matches!(events[..], [ShopEvent::HealthPotionUsed { healed: 5, .. }]), "{events:?}");

    assert!(g.use_potion(Potion::Health));
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP, "최대를 넘지 않는다");
    assert_eq!(g.shop.potions(Potion::Health), 0);

    // 0개면 아무 일도 없다.
    g.combat.health.lose_from_quiz(1);
    assert!(!g.use_potion(Potion::Health));
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP - 1);
}

#[test]
fn potions_are_not_used_while_a_window_is_open_or_after_the_game() {
    let mut g = new_game();
    give_coins(&mut g, HEALTH_POTION_PRICE);
    g.buy(ShopItem::HealthPotion).unwrap();
    g.combat.health.lose_from_quiz(1);

    enter_shop(&mut g);
    assert!(!g.use_potion(Potion::Health), "상점 안");
    g.close_shop();

    g.math.add_waiting();
    g.math.open_next_waiting();
    assert!(g.math.is_open());
    assert!(!g.use_potion(Potion::Health), "문제 창");
    answer_quiz_if_open(&mut g);

    g.combat.health.lose_from_quiz(1000);
    assert!(g.is_over());
    assert!(!g.use_potion(Potion::Health), "쓰러진 뒤");
    assert_eq!(g.shop.potions(Potion::Health), 1);
}

#[test]
fn poison_hurts_monsters_within_200_once_a_second() {
    let mut g = empty_game();
    give_coins(&mut g, POISON_POTION_PRICE);
    g.buy(ShopItem::PoisonPotion).unwrap();
    stand_at(&mut g, 1000.0, Facing::Right);
    g.combat.zombies.list.push(zombie_at_distance(&g, POISON_RANGE));
    g.combat.zombies.list.push(zombie_at_distance(&g, -(POISON_RANGE + 1.0)));
    assert!(g.use_potion(Potion::Poison));
    assert_eq!(g.shop.potions(Potion::Poison), 0);
    assert!(matches!(g.take_shop_events()[..], [ShopEvent::PoisonSprayed { monsters: 1, .. }]));
    assert!(g.combat.zombies.list[0].poison.is_on());
    assert!(!g.combat.zombies.list[1].poison.is_on());

    run(&mut g, idle(), POISON_TICK_SECONDS * 2.0 + A_LITTLE);
    assert_eq!(g.combat.zombies.list[0].hp, ZOMBIE_HP - 2);
    let defeated_before = g.combat.monsters_defeated;
    let mut events = Vec::new();
    for _ in 0..((POISON_TICK_SECONDS + A_LITTLE) / FRAME).ceil() as usize {
        events.extend(g.update(&idle(), FRAME));
    }
    // 체력 3인 몬스터는 3초 뒤 쓰러지고 경험치 1을 준다.
    assert_eq!(g.combat.zombies.list.len(), 1);
    assert_eq!(g.combat.zombies.list[0].hp, ZOMBIE_HP, "201픽셀 밖은 그대로");
    assert!(events.iter().any(|e| matches!(e, GameEvent::Combat(CombatEvent::ZombieDefeated { .. }))));
    assert_eq!(g.combat.monsters_defeated, defeated_before + 1);
    assert_eq!(g.combat.experience, EXPERIENCE_PER_DEFEAT);
}

#[test]
fn poison_with_no_monster_nearby_is_not_used_up() {
    let mut g = empty_game();
    give_coins(&mut g, POISON_POTION_PRICE);
    g.buy(ShopItem::PoisonPotion).unwrap();
    g.combat.zombies.list.push(zombie_at_distance(&g, POISON_RANGE + 1.0));
    assert!(!g.use_potion(Potion::Poison));
    assert_eq!(g.shop.potions(Potion::Poison), 1);
    assert!(g.take_shop_events().is_empty());
}

#[test]
fn poison_does_not_speed_up_with_fast_forward() {
    let mut g = empty_game();
    give_coins(&mut g, POISON_POTION_PRICE);
    g.buy(ShopItem::PoisonPotion).unwrap();
    g.combat.zombies.list.push(zombie_at_distance(&g, 100.0));
    assert!(g.use_potion(Potion::Poison));
    run_fast(&mut g, POISON_TICK_SECONDS + A_LITTLE, DEV_FAST_FORWARD_SCALE);
    assert_eq!(g.combat.zombies.list[0].hp, ZOMBIE_HP - POISON_DAMAGE);
}

#[test]
fn poison_goes_away_with_the_monsters_at_six_am() {
    let mut g = empty_game();
    g.clock.update(DAY_SECONDS + NIGHT_SECONDS - 0.5);
    g.update(&idle(), 0.0);
    g.combat.zombies.list.clear();
    g.combat.skeletons.list.clear();
    give_coins(&mut g, POISON_POTION_PRICE);
    g.buy(ShopItem::PoisonPotion).unwrap();
    g.combat.zombies.list.push(zombie_at_distance(&g, 150.0));
    assert!(g.use_potion(Potion::Poison));
    run(&mut g, idle(), 3.0);
    assert!(!g.clock.is_night());
    assert!(g.combat.zombies.list.is_empty());
    assert_eq!(g.combat.monsters_defeated, 0, "사라진 몬스터는 경험치가 없다");
}

// ---- US-072 광물 팔기 ----

#[test]
fn gold_silver_copper_sell_for_their_prices() {
    let mut g = new_game();
    for m in [Material::Gold, Material::Silver, Material::Copper] {
        g.gathering.bag.add(m);
    }
    assert_eq!(g.sell(Material::Gold), Ok(GOLD_SELL_PRICE));
    assert_eq!(g.sell(Material::Silver), Ok(SILVER_SELL_PRICE));
    assert_eq!(g.sell(Material::Copper), Ok(COPPER_SELL_PRICE));
    assert_eq!(g.wallet.coins(), 1000 + 500 + 200);
    assert_eq!((GOLD_SELL_PRICE, SILVER_SELL_PRICE, COPPER_SELL_PRICE), (1000, 500, 200));
    assert_eq!(g.gathering.bag.count(Material::Gold), 0);
    assert_eq!(g.coins.picked_total, 0, "판 코인은 획득 코인으로 세지 않는다");
    assert_eq!(g.score(), 0);
}

#[test]
fn nothing_to_sell_sells_nothing() {
    let mut g = new_game();
    assert_eq!(g.sell(Material::Gold), Err(CantSell::NoneToSell));
    assert_eq!(g.wallet.coins(), 0);
}

#[test]
fn iron_is_not_for_sale() {
    let mut g = new_game();
    g.gathering.bag.add(Material::Iron);
    assert_eq!(g.sell(Material::Iron), Err(CantSell::NoneToSell));
    assert_eq!(g.gathering.bag.count(Material::Iron), 1);
    let rows: Vec<Material> = g.sell_rows().iter().map(|r| r.material).collect();
    assert_eq!(rows, vec![Material::Gold, Material::Silver, Material::Copper]);
}

#[test]
fn selling_over_the_limit_sells_and_stops_at_the_limit() {
    // 부모님 결정: 막지 않는다. 4,500에서 금을 팔면 5,000 (US-072).
    let mut g = new_game();
    give_coins(&mut g, COIN_MAX - GOLD_SELL_PRICE / 2);
    g.gathering.bag.add(Material::Gold);
    let row = g.sell_rows()[0];
    assert!(row.can_sell);
    assert_eq!(row.reason, None);
    assert_eq!(g.sell(Material::Gold), Ok(GOLD_SELL_PRICE / 2), "실제로 들어간 코인");
    assert_eq!(g.wallet.coins(), COIN_MAX);
    assert_eq!(g.gathering.bag.count(Material::Gold), 0);

    // 이미 5,000이어도 판다. 코인은 그대로 5,000.
    g.gathering.bag.add(Material::Copper);
    assert_eq!(g.sell(Material::Copper), Ok(0));
    assert_eq!(g.wallet.coins(), COIN_MAX);
}

#[test]
fn selling_up_to_exactly_the_limit_is_fine() {
    let mut g = new_game();
    give_coins(&mut g, COIN_MAX - GOLD_SELL_PRICE);
    g.gathering.bag.add(Material::Gold);
    assert_eq!(g.sell(Material::Gold), Ok(GOLD_SELL_PRICE));
    assert_eq!(g.wallet.coins(), COIN_MAX);
}
