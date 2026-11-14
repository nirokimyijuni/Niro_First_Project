//! M8 상자 부수기 시험 (수산나): US-089, D87, [임시-63]~[임시-69].
//! 숫자는 config 상수로 적어서 임시 값을 바꿔도 시험을 고치지 않아도 된다.
//! 확률은 정해진 시드로만 보고, 반복은 모두 횟수가 정해져 있다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::boxes::{BoxEvent, BoxMonster, Boxes, Surprise, SurpriseDice, SurpriseItem};
use logic::character::CharacterKind;
use logic::combat::{Combat, CombatEvent};
use logic::game::{Game, GameEvent};
use logic::monster::{SeededRandom, Zombie};
use logic::player::{Facing, PlayerInput};
use logic::shop::{Armor, Potion};
use logic::skeleton::{Skeleton, SkeletonStage};
use logic::weapon::{DroppedWeapon, Weapon};
use logic::world::{really_overlaps, World};
use macroquad::math::{Rect, Vec2};

const FRAME: f32 = 1.0 / 60.0;
const SEED: u32 = 7;
/// 상자 하나: 왼쪽이 400, 작은 상자.
const BOX_X: f32 = 400.0;

fn the_box() -> Rect {
    Rect::new(BOX_X, GROUND_Y - BOX_SMALL_SIZE, BOX_SMALL_SIZE, BOX_SMALL_SIZE)
}

/// 상자 하나만 있는 넓은 맵 (긴 맵이 아니라 미스·시민이 없다).
fn one_box_game(seed: u32) -> Game {
    let world = World { width: 3000.0, ground_y: GROUND_Y, obstacles: vec![the_box()] };
    let mut g = Game::on_world(CharacterKind::Niro, seed, world);
    stand_at(&mut g, BOX_X - PLAYER_WIDTH - 20.0, Facing::Right);
    g
}

fn stand_at(g: &mut Game, x: f32, facing: Facing) {
    g.player.pos = Vec2::new(x, GROUND_Y - PLAYER_HEIGHT);
    g.player.vel = Vec2::ZERO;
    g.player.facing = facing;
}

fn with_axe(mut g: Game) -> Game {
    g.weapons.add_bought(Weapon::Axe);
    g
}

/// 공격 버튼을 한 번 누른다. 그 프레임의 GameEvent와 상자 일을 돌려준다.
fn swing(g: &mut Game) -> (Vec<GameEvent>, Vec<BoxEvent>) {
    let events = g.update(&PlayerInput { attack_pressed: true, ..Default::default() }, FRAME);
    (events, g.take_box_events())
}

fn wait(g: &mut Game, seconds: f32) -> Vec<BoxEvent> {
    let mut box_events = Vec::new();
    for _ in 0..(seconds / FRAME).ceil() as usize {
        g.update(&PlayerInput::default(), FRAME);
        box_events.extend(g.take_box_events());
    }
    box_events
}

fn box_is_on_map(g: &Game) -> bool {
    g.world.obstacles.contains(&the_box())
}

// ---- 부수기 ----

#[test]
fn axe_breaks_the_box_on_the_third_hit_and_cracks_show_before() {
    let mut g = with_axe(one_box_game(SEED));
    let (_, first) = swing(&mut g);
    assert!(first.iter().any(|e| matches!(e, BoxEvent::Cracked { cracks: 1, .. })));
    assert_eq!(g.boxes.cracks_at(the_box()), 1, "금이 조금");
    let (_, second) = swing(&mut g);
    assert!(second.iter().any(|e| matches!(e, BoxEvent::Cracked { cracks: 2, .. })));
    assert_eq!(g.boxes.cracks_at(the_box()), 2, "금이 많이");
    assert!(box_is_on_map(&g));
    let (_, third) = swing(&mut g);
    assert!(third.iter().any(|e| matches!(e, BoxEvent::Broken { index: 0, .. })));
    assert!(!box_is_on_map(&g), "부서져 사라진다");
    assert!(!g.boxes.list[0].is_standing());
    assert_eq!(BOX_HITS_TO_BREAK, 3);
}

#[test]
fn hands_sword_and_gun_leave_the_box_alone_and_hint_comes_once() {
    let mut g = one_box_game(SEED);
    g.weapons.add_bought(Weapon::Sword);
    g.weapons.add_bought(Weapon::Gun);
    let mut hints = 0;
    for weapon in [Weapon::Hands, Weapon::Sword, Weapon::Gun] {
        g.weapons.hold(weapon);
        for _ in 0..5 {
            let (_, happened) = swing(&mut g);
            hints += happened.iter().filter(|e| matches!(e, BoxEvent::NeedAxe { .. })).count();
        }
        assert_eq!(g.boxes.cracks_at(the_box()), 0, "{weapon:?}");
        assert!(box_is_on_map(&g));
    }
    assert_eq!(hints, 1, "도끼 안내는 한 판에 한 번");
    assert_eq!(BOX_NEED_AXE_TEXT, "도끼로 부술 수 있어요");
}

#[test]
fn monster_in_reach_is_hit_instead_of_the_box() {
    let mut g = with_axe(one_box_game(SEED));
    let front = g.player.rect().x + PLAYER_WIDTH;
    g.combat.zombies.list.push(Zombie::new(front + 2.0, GROUND_Y, false));
    let (events, _) = swing(&mut g);
    assert!(events.iter().any(|e| matches!(e, GameEvent::Combat(CombatEvent::ZombieDefeated { .. }))));
    assert_eq!(g.boxes.cracks_at(the_box()), 0, "몬스터만 맞는다");
}

#[test]
fn one_swing_hits_only_the_nearest_box_in_front() {
    let near = Rect::new(400.0, GROUND_Y - 48.0, 20.0, 48.0);
    let far = Rect::new(425.0, GROUND_Y - 48.0, 20.0, 48.0);
    let behind = Rect::new(330.0, GROUND_Y - 48.0, 20.0, 48.0);
    let world = World { width: 3000.0, ground_y: GROUND_Y, obstacles: vec![behind, near, far] };
    let mut g = with_axe(Game::on_world(CharacterKind::Niro, SEED, world));
    stand_at(&mut g, 352.0, Facing::Right);
    swing(&mut g);
    assert_eq!(g.boxes.cracks_at(near), 1);
    assert_eq!(g.boxes.cracks_at(far), 0, "한 번에 하나만");
    assert_eq!(g.boxes.cracks_at(behind), 0, "등 뒤는 안 맞는다");
}

#[test]
fn broken_box_counts_as_nothing_for_walking_gun_and_monster_walls() {
    let mut g = with_axe(one_box_game(SEED));
    g.weapons.add_bought(Weapon::Gun);
    g.weapons.hold(Weapon::Gun);
    assert!(g.attack_reach() < GUN_RANGE, "서 있는 상자는 총 빛줄기를 막는다");
    g.weapons.hold(Weapon::Axe);
    for _ in 0..3 {
        swing(&mut g);
    }
    assert!(!g.world.monster_walls().any(|wall| wall == the_box()), "몬스터 벽이 아니다 (D48, D62)");
    g.weapons.hold(Weapon::Gun);
    assert_eq!(g.attack_reach(), GUN_RANGE, "총 빛줄기가 지나간다 (D66)");
    // 걸어서 지나간다.
    for _ in 0..120 {
        g.update(&PlayerInput { right: true, ..Default::default() }, FRAME);
    }
    assert!(g.player.rect().x > BOX_X + BOX_SMALL_SIZE, "상자 자리를 지나갔다");
    assert!(g.player.on_ground);
    // 좀비도 지나간다 (몬스터 걷기 규칙).
    let mut zombie = Zombie::new(BOX_X - 100.0, GROUND_Y, false);
    let target = BOX_X + 300.0;
    for _ in 0..600 {
        zombie.pos.x = logic::monster::walk_x(zombie.rect(), target, ZOMBIE_WALK_SPEED * FRAME, &g.world);
    }
    assert!(zombie.rect().x > BOX_X + BOX_SMALL_SIZE, "좀비가 상자 자리를 지나간다");
}

// ---- 위에 있던 것 ----

#[test]
fn player_on_the_box_falls_to_the_ground_with_same_hp() {
    let mut g = one_box_game(SEED);
    g.player.pos = Vec2::new(BOX_X, GROUND_Y - BOX_SMALL_SIZE - PLAYER_HEIGHT);
    wait(&mut g, 0.5);
    assert_eq!(g.player.rect().y + PLAYER_HEIGHT, GROUND_Y - BOX_SMALL_SIZE, "상자 위에 서 있다");
    let hp = g.combat.health.hp();
    let mut events = Vec::new();
    g.break_box(0, &mut events);
    wait(&mut g, 1.0);
    assert_eq!(g.player.rect().y + PLAYER_HEIGHT, GROUND_Y, "땅으로 떨어졌다");
    assert_eq!(g.combat.health.hp(), hp, "체력은 그대로");
}

#[test]
fn weapon_and_coin_on_the_box_fall_and_coin_does_not_come_back_while_box_is_gone() {
    let mut g = one_box_game(SEED);
    stand_at(&mut g, 1500.0, Facing::Right);
    g.weapons.dropped.push(DroppedWeapon::above(Weapon::Sword, BOX_X + BOX_SMALL_SIZE / 2.0, &g.world));
    wait(&mut g, 3.0);
    assert_eq!(g.weapons.dropped[0].rect.y + WEAPON_SIZE, GROUND_Y - BOX_SMALL_SIZE, "무기가 상자 위에 있다");
    let mut events = Vec::new();
    g.break_box(0, &mut events);
    wait(&mut g, 2.0);
    assert_eq!(g.weapons.dropped[0].rect.y + WEAPON_SIZE, GROUND_Y, "무기가 땅으로 떨어졌다");
    let coin = &g.coins.list[0];
    assert!(coin.is_visible());
    assert_eq!(coin.rect.y + COIN_SIZE + COIN_GAP_ABOVE_BOX, GROUND_Y, "코인이 땅으로 떨어졌다");
    // 땅 코인을 줍는다.
    let wallet = g.wallet.coins();
    stand_at(&mut g, BOX_X, Facing::Right);
    wait(&mut g, 0.1);
    assert_eq!(g.wallet.coins(), wallet + 1, "땅에 떨어진 코인을 주웠다");
    // 상자가 없는 동안 새로 생기지 않는다.
    stand_at(&mut g, 1500.0, Facing::Right);
    wait(&mut g, COIN_RESPAWN_SECONDS * 3.0);
    assert!(!g.coins.list[0].is_visible(), "상자가 없는 동안 코인이 새로 생기지 않는다");
    // 몬스터는 늘 땅에 있다.
    assert!(g.combat.zombies.list.iter().all(|z| z.rect().y + ZOMBIE_HEIGHT == GROUND_Y));
}

// ---- 깜짝 아이템과 몬스터 (주사위) ----

#[test]
fn surprise_is_30_item_15_monster_55_nothing_for_10000_breaks() {
    let mut dice = SurpriseDice::new(SEED);
    let (mut items, mut monsters, mut nothing) = (0, 0, 0);
    for _ in 0..10_000 {
        match dice.roll_surprise() {
            Surprise::Item => items += 1,
            Surprise::Monster => monsters += 1,
            Surprise::Nothing => nothing += 1,
        }
    }
    let near = |count: i32, chance: f32| ((count as f32 / 10_000.0) - chance).abs() <= 0.01;
    assert!(near(items, BOX_ITEM_CHANCE), "아이템 {items}");
    assert!(near(monsters, BOX_MONSTER_CHANCE), "몬스터 {monsters}");
    assert!(near(nothing, 1.0 - BOX_ITEM_CHANCE - BOX_MONSTER_CHANCE), "없음 {nothing}");
    assert_eq!((BOX_ITEM_CHANCE, BOX_MONSTER_CHANCE), (0.30, 0.15));
}

fn item_counts(have: &[SurpriseItem], rolls: usize) -> Vec<(SurpriseItem, usize)> {
    let mut dice = SurpriseDice::new(SEED);
    let mut counts: Vec<(SurpriseItem, usize)> = SurpriseItem::ALL.iter().map(|i| (*i, 0)).collect();
    for _ in 0..rolls {
        let item = dice.roll_item(have);
        counts.iter_mut().find(|(i, _)| *i == item).unwrap().1 += 1;
    }
    counts
}

#[test]
fn items_follow_the_shares_for_100000_rolls() {
    for (item, count) in item_counts(&[], 100_000) {
        let wanted = match item {
            SurpriseItem::Gun => 0.10,
            i if i.is_very_strong() => 0.01,
            _ => 0.86 / 7.0,
        };
        let got = count as f32 / 100_000.0;
        assert!((got - wanted).abs() <= 0.005, "{item:?} {got} (바라는 값 {wanted})");
    }
}

#[test]
fn only_ones_already_owned_never_come_and_the_rest_keep_their_ratio() {
    let have = [SurpriseItem::Sword, SurpriseItem::Gun, SurpriseItem::Skates];
    let left: f32 = SurpriseItem::ALL.iter().filter(|i| !have.contains(i)).map(|i| i.share()).sum();
    for (item, count) in item_counts(&have, 100_000) {
        let got = count as f32 / 100_000.0;
        if have.contains(&item) {
            assert_eq!(count, 0, "{item:?}은 이미 가졌다");
        } else {
            assert!((got - item.share() / left).abs() <= 0.005, "{item:?} {got}");
        }
    }
}

#[test]
fn potions_and_ender_fruit_still_come_when_owned_in_the_game() {
    // 게임에서 하나뿐인 것을 모두 가지면 (다이아 갑옷을 입은 채) 포션·무지개 갑옷·엔더열매만 나온다.
    let mut seen = Vec::new();
    for seed in 0..400 {
        let mut g = with_axe(one_box_game(seed));
        g.weapons.add_bought(Weapon::Sword);
        g.weapons.add_bought(Weapon::Gun);
        g.weapons.hold(Weapon::Axe);
        g.shop.put_on_skates();
        g.shop.add_potion(Potion::Health);
        g.gathering.bag.ender_fruits = 1;
        g.gathering.bag.riding_car = true;
        g.gathering.bag.has_diamond_pickaxe = true;
        g.gathering.bag.has_syringe_gun = true;
        g.shop.put_on_armor(Armor::Diamond);
        for _ in 0..3 {
            for event in swing(&mut g).1 {
                if let BoxEvent::ItemDropped { item, .. } = event {
                    seen.push(item);
                }
            }
        }
    }
    let only_ones = [
        SurpriseItem::Sword,
        SurpriseItem::Axe,
        SurpriseItem::Gun,
        SurpriseItem::Skates,
        SurpriseItem::Car,
        SurpriseItem::DiamondPickaxe,
        SurpriseItem::SyringeGun,
        SurpriseItem::DiamondArmor,
    ];
    assert!(!seen.is_empty());
    assert!(seen.iter().all(|i| !only_ones.contains(i)), "{seen:?}");
    assert!(seen.contains(&SurpriseItem::HealthPotion), "포션은 가지고 있어도 나온다");
    assert!(seen.contains(&SurpriseItem::RainbowArmor), "입지 않은 갑옷은 나온다");
}

#[test]
fn armor_you_are_wearing_does_not_come_from_a_box() {
    // [임시-65] 입고 있는 갑옷은 상자에서 안 나온다. 다른 갑옷은 나온다.
    for (worn, worn_item, other_item) in [
        (Armor::Rainbow, SurpriseItem::RainbowArmor, SurpriseItem::DiamondArmor),
        (Armor::Diamond, SurpriseItem::DiamondArmor, SurpriseItem::RainbowArmor),
    ] {
        let mut seen = Vec::new();
        for seed in 0..400 {
            let mut g = with_axe(one_box_game(seed));
            g.weapons.hold(Weapon::Axe);
            g.shop.put_on_armor(worn);
            for _ in 0..3 {
                for event in swing(&mut g).1 {
                    if let BoxEvent::ItemDropped { item, .. } = event {
                        seen.push(item);
                    }
                }
            }
        }
        assert!(!seen.contains(&worn_item), "{worn:?}를 입고 있으면 안 나온다: {seen:?}");
        assert!(!seen.is_empty());
        // 다이아 갑옷은 아주 센 것(1%)이라 400판에 안 나올 수 있다. 보통 몫인 무지개 갑옷만 꼭 나와야 한다.
        if !other_item.is_very_strong() {
            assert!(seen.contains(&other_item), "{worn:?}를 입어도 다른 갑옷은 나온다");
        }
    }
}

#[test]
fn item_and_monster_never_come_together_and_wow_only_for_very_strong() {
    let (mut items, mut strong, mut monsters) = (0, 0, 0);
    for seed in 0..3000 {
        let mut g = with_axe(one_box_game(seed));
        let mut happened = Vec::new();
        for _ in 0..3 {
            happened.extend(swing(&mut g).1);
        }
        let item = happened.iter().find_map(|e| match e {
            BoxEvent::ItemDropped { item, .. } => Some(*item),
            _ => None,
        });
        let monster = happened.iter().any(|e| matches!(e, BoxEvent::MonsterJumpedOut { .. }));
        let wow = happened.iter().any(|e| matches!(e, BoxEvent::Wow { .. }));
        assert!(!(item.is_some() && monster), "seed {seed}: 함께 나오지 않는다");
        assert_eq!(wow, item.is_some_and(|i| i.is_very_strong()), "seed {seed}: 와우는 아주 센 것만");
        items += item.is_some() as u32;
        strong += item.is_some_and(|i| i.is_very_strong()) as u32;
        monsters += monster as u32;
    }
    assert!(items > 0 && strong > 0 && monsters > 0, "아이템 {items}, 아주 센 것 {strong}, 몬스터 {monsters}");
    assert_eq!(BOX_WOW_TEXT, "와우!");
}

#[test]
fn breaking_boxes_adds_no_score_and_no_quiz_item() {
    let mut total_monsters = 0;
    for seed in 0..200 {
        let mut g = with_axe(one_box_game(seed));
        // 땅에 떨어진 상자 코인은 주우면 보통 코인처럼 센다. 그 몫은 빼고 본다.
        let score_without_coins = |g: &Game| g.score() - g.coins.picked_total * SCORE_PER_COIN;
        let (score, picked) = (score_without_coins(&g), g.weapons.picked_up_count);
        for _ in 0..3 {
            swing(&mut g);
        }
        // 나온 몬스터는 세고 치운다 (닿아서 쓰러지면 보통 몬스터처럼 점수가 되니까).
        total_monsters += g.combat.zombies.list.len() + g.combat.skeletons.list.len();
        g.combat.zombies.list.clear();
        g.combat.skeletons.list.clear();
        // 떨어진 아이템을 줍는다.
        stand_at(&mut g, BOX_X, Facing::Right);
        wait(&mut g, 2.0);
        assert_eq!(g.weapons.picked_up_count, picked, "seed {seed}: 수학 아이템으로 세지 않는다");
        assert_eq!(score_without_coins(&g), score, "seed {seed}: 점수가 늘지 않는다");
        assert!(!g.math.is_open());
    }
    assert!(total_monsters > 0);
}

// ---- 깜짝 아이템 줍기 ----

#[test]
fn picked_surprise_items_work_like_bought_or_crafted_ones() {
    for item in SurpriseItem::ALL {
        let mut g = one_box_game(SEED);
        g.world.obstacles.clear();
        stand_at(&mut g, BOX_X, Facing::Right);
        g.boxes.drop_item(item, the_box(), &g.world);
        let happened = wait(&mut g, 2.0);
        assert!(happened.contains(&BoxEvent::ItemPicked { item }), "{item:?}");
        assert!(item.picked_text().ends_with("얻었어요!"));
        let bag = &g.gathering.bag;
        let works = match item {
            SurpriseItem::Sword => g.weapons.held() == Weapon::Sword,
            SurpriseItem::Axe => g.weapons.held() == Weapon::Axe,
            SurpriseItem::Gun => g.weapons.held() == Weapon::Gun,
            SurpriseItem::HealthPotion => g.shop.potions(Potion::Health) == 1,
            SurpriseItem::PoisonPotion => g.shop.potions(Potion::Poison) == 1,
            SurpriseItem::Skates => g.shop.has_skates(),
            SurpriseItem::RainbowArmor => g.shop.armor() == Some(Armor::Rainbow),
            SurpriseItem::DiamondArmor => g.shop.armor() == Some(Armor::Diamond),
            SurpriseItem::SyringeGun => bag.has_syringe_gun,
            SurpriseItem::EnderFruit => bag.ender_fruits == 1,
            SurpriseItem::Car => bag.riding_car,
            SurpriseItem::DiamondPickaxe => bag.has_diamond_pickaxe,
        };
        assert!(works, "{item:?}");
    }
}

#[test]
fn surprise_item_lands_on_the_ground_and_goes_away_15_seconds_later() {
    let mut g = one_box_game(SEED);
    stand_at(&mut g, 1500.0, Facing::Right);
    let mut events = Vec::new();
    g.break_box(0, &mut events);
    g.boxes.items.clear();
    g.boxes.drop_item(SurpriseItem::HealthPotion, the_box(), &g.world);
    wait(&mut g, 1.0);
    assert_eq!(g.boxes.items[0].rect.y + BOX_ITEM_SIZE, GROUND_Y, "상자 자리 땅에 떨어졌다");
    wait(&mut g, BOX_ITEM_GROUND_SECONDS - 2.0);
    assert_eq!(g.boxes.items.len(), 1, "아직 있다");
    wait(&mut g, 2.0);
    assert!(g.boxes.items.is_empty(), "15초 뒤 사라졌다");
}

// ---- 상자에서 나온 몬스터 ----

#[test]
fn zombie_and_skeleton_are_about_half_each() {
    let mut dice = SurpriseDice::new(SEED);
    let skeletons = (0..10_000).filter(|_| dice.roll_monster() == BoxMonster::Skeleton).count();
    let share = skeletons as f32 / 10_000.0;
    assert!((share - 0.5).abs() <= 0.02, "스켈레톤 {share}");
    assert_eq!(BOX_SKELETON_CHANCE, 0.5);
}

#[test]
fn box_monster_waits_one_second_without_attacking_even_in_the_day() {
    let world = World { width: 3000.0, ground_y: GROUND_Y, obstacles: Vec::new() };
    let player = Rect::new(500.0, GROUND_Y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT);
    // 주인공과 겹친 좀비.
    let mut combat = Combat::new(SEED);
    combat.spawn_from_box(player.center().x, GROUND_Y, BoxMonster::Zombie).expect("좀비");
    let start = combat.zombies.list[0].pos;
    let mut events = Vec::new();
    for _ in 0..((BOX_MONSTER_WAIT_SECONDS - 0.1) / FRAME) as usize {
        events.extend(combat.update(FRAME, false, player, &world));
    }
    assert!(combat.zombies.list[0].is_waiting());
    assert_eq!(combat.zombies.list[0].pos, start, "가만히 있다");
    assert!(!events.iter().any(|e| matches!(e, CombatEvent::PlayerHurt { .. })), "공격하지 않는다");
    for _ in 0..(0.3 / FRAME) as usize {
        events.extend(combat.update(FRAME, false, player, &world));
    }
    assert!(events.iter().any(|e| matches!(e, CombatEvent::PlayerHurt { .. })), "1초 뒤 공격한다 (낮에도)");

    // 손이 닿는 절반 스켈레톤도 1초 동안은 때리지 않는다.
    let mut combat = Combat::new(SEED);
    combat.level = SKELETON_HALF_MIN_LEVEL;
    let near_x = player.x + player.w + SKELETON_HAND_RANGE / 2.0 + SKELETON_WIDTH / 2.0;
    combat.spawn_from_box(near_x, GROUND_Y, BoxMonster::Skeleton).expect("스켈레톤");
    assert_eq!(combat.skeletons.list[0].stage, SkeletonStage::Half, "지금 레벨의 단계");
    let mut events = Vec::new();
    for _ in 0..((BOX_MONSTER_WAIT_SECONDS - 0.1) / FRAME) as usize {
        events.extend(combat.update(FRAME, false, player, &world));
    }
    assert!(!events.iter().any(|e| matches!(e, CombatEvent::PlayerHurt { .. } | CombatEvent::ProjectileFired { .. })));
    for _ in 0..(0.5 / FRAME) as usize {
        events.extend(combat.update(FRAME, false, player, &world));
    }
    assert!(events.iter().any(|e| matches!(e, CombatEvent::PlayerHurt { .. })), "1초 뒤 손으로 때린다");
}

#[test]
fn monsters_stand_still_while_the_player_is_in_the_cave_and_move_again_after() {
    // 부모님 결정 2026-11-14 (REVIEW-021 2번): 동굴 안에 있는 동안 몬스터는 모두 멈춘다.
    let mut g = one_box_game(SEED);
    let x = g.player.rect().center().x + 400.0;
    g.combat.spawn_from_box(x, GROUND_Y, BoxMonster::Zombie).expect("좀비");
    g.in_cave = true;
    let start = g.combat.zombies.list[0].pos;
    wait(&mut g, BOX_MONSTER_WAIT_SECONDS + 3.0);
    assert!(g.in_cave);
    assert_eq!(g.combat.zombies.list.len(), 1);
    assert_eq!(g.combat.zombies.list[0].pos, start, "동굴에 있는 동안 가만히 있다");
    assert!(g.combat.zombies.list[0].is_waiting(), "기다리는 시간도 흐르지 않는다");

    g.in_cave = false;
    wait(&mut g, BOX_MONSTER_WAIT_SECONDS + 1.0);
    assert_ne!(g.combat.zombies.list[0].pos, start, "나오면 다시 움직인다");
}

#[test]
fn box_monster_counts_like_any_monster() {
    let player = Rect::new(500.0, GROUND_Y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT);
    let mut combat = Combat::new(SEED);
    combat.spawn_from_box(player.x + player.w + 30.0, GROUND_Y, BoxMonster::Zombie).unwrap();
    for _ in 0..ZOMBIE_HP {
        combat.attack_with(player, Facing::Right, 1, PLAYER_ATTACK_RANGE);
    }
    assert!(combat.zombies.list.is_empty(), "체력대로 맞으면 쓰러진다");
    assert_eq!(combat.monsters_defeated, 1);
    assert_eq!(combat.experience, EXPERIENCE_PER_DEFEAT);
}

#[test]
fn no_box_monster_when_full_and_skeleton_when_zombies_are_full() {
    let mut combat = Combat::new(SEED);
    for i in 0..ZOMBIE_MAX_ON_SCREEN {
        combat.zombies.list.push(Zombie::new(1000.0 + i as f32 * 60.0, GROUND_Y, false));
    }
    let came = combat.spawn_from_box(300.0, GROUND_Y, BoxMonster::Zombie);
    assert!(matches!(came, Some(CombatEvent::SkeletonAppeared { .. })), "좀비가 꽉 차면 스켈레톤");
    for _ in combat.zombies.list.len() + combat.skeletons.list.len()..MONSTER_MAX_ON_SCREEN {
        combat.skeletons.list.push(Skeleton::new(2000.0, GROUND_Y, SkeletonStage::Normal));
    }
    assert_eq!(combat.spawn_from_box(300.0, GROUND_Y, BoxMonster::Skeleton), None, "6마리면 나오지 않는다");
    assert_eq!(combat.zombies.list.len() + combat.skeletons.list.len(), MONSTER_MAX_ON_SCREEN);
}

// ---- 다시 생기기 ----

/// 밤 문제를 맞히고 아침 직전까지 보낸다. 갑옷으로 다치지 않게 한다.
fn go_to_just_before_morning(g: &mut Game) {
    g.update(&PlayerInput::default(), g.clock.seconds_left_in_phase() + 0.01);
    assert!(g.clock.is_night());
    let answer = g.math.question().expect("밤 문제").answer;
    g.math.type_and_submit(answer);
    g.update(&PlayerInput::default(), MATH_RIGHT_SHOW_SECONDS);
    assert!(!g.math.is_open());
    g.shop.put_on_armor(Armor::Diamond);
    g.update(&PlayerInput::default(), g.clock.seconds_left_in_phase() - 1.0);
    assert!(g.clock.is_night());
}

#[test]
fn broken_box_comes_back_new_at_the_next_6_oclock() {
    let mut g = with_axe(one_box_game(SEED));
    // 1일째 10:00쯤까지 기다린다 (낮 1초 = 2.4분).
    g.update(&PlayerInput::default(), 240.0 / 2.4 + 1.0);
    assert_eq!(g.clock.hour_minute().0, 10);
    for _ in 0..3 {
        swing(&mut g);
    }
    assert!(!box_is_on_map(&g));
    stand_at(&mut g, 1500.0, Facing::Right);
    go_to_just_before_morning(&mut g);
    assert!(!box_is_on_map(&g), "05:59에는 아직 없다");
    g.update(&PlayerInput::default(), 2.0);
    assert_eq!(g.clock.day_number(), 2);
    assert!(box_is_on_map(&g), "2일째 06:00에 같은 자리, 같은 크기로");
    assert!(g.boxes.list[0].is_standing());
    assert_eq!(g.boxes.list[0].cracks(), 0);
    assert!(!g.is_over());
}

#[test]
fn cracked_box_is_new_again_at_6_oclock() {
    let mut boxes = Boxes::on_world(&World::m1(), SEED);
    boxes.hit(0);
    boxes.hit(0);
    boxes.morning_came();
    assert_eq!(boxes.list[0].cracks(), 0);
    assert!(boxes.list[0].is_standing());
}

#[test]
fn box_waits_while_someone_stands_there_and_never_traps_anyone_for_500_seeds() {
    for seed in 0..500u32 {
        let mut g = one_box_game(seed);
        let mut random = SeededRandom::new(seed);
        let mut events = Vec::new();
        g.break_box(0, &mut events);
        g.combat.zombies.list.clear();
        g.combat.skeletons.list.clear();
        // 상자 자리 근처 아무 데나 선다. 몬스터도 하나 그 근처에 둔다.
        let x = BOX_X - PLAYER_WIDTH + random.next_unit() * (BOX_SMALL_SIZE + PLAYER_WIDTH * 2.0);
        stand_at(&mut g, x, Facing::Right);
        let zombie_x = BOX_X - ZOMBIE_WIDTH + random.next_unit() * (BOX_SMALL_SIZE + ZOMBIE_WIDTH * 2.0);
        if random.next_unit() < 0.5 {
            g.combat.zombies.list.push(Zombie::new(zombie_x, GROUND_Y, false));
        }
        let blocked = really_overlaps(g.player.rect(), the_box()) || g.combat.any_monster_touches(the_box());
        g.boxes.morning_came();
        g.update(&PlayerInput::default(), FRAME);
        let blocked_now = really_overlaps(g.player.rect(), the_box()) || g.combat.any_monster_touches(the_box());
        if blocked && blocked_now {
            assert!(!box_is_on_map(&g), "seed {seed}: 누가 있으면 기다린다");
        }
        // 이리저리 걷다가 멀리 간다. 한 번도 상자에 끼지 않는다.
        let mut came_back = box_is_on_map(&g);
        for step in 0..240 {
            let input = if step < 120 && random.next_unit() < 0.5 {
                PlayerInput { left: true, ..Default::default() }
            } else {
                PlayerInput { right: true, ..Default::default() }
            };
            g.combat.zombies.list.clear();
            g.update(&input, FRAME);
            let stuck = g.world.obstacles.iter().any(|o| really_overlaps(*o, g.player.rect()));
            assert!(!stuck, "seed {seed}: 상자에 끼었다");
            came_back |= box_is_on_map(&g);
        }
        assert!(came_back, "seed {seed}: 비키면 바로 생긴다");
    }
}

#[test]
fn new_game_has_all_boxes_new() {
    let g = Game::new(CharacterKind::Ellie, SEED);
    assert_eq!(g.boxes.list.len(), g.world.boxes().len());
    assert!(g.boxes.list.iter().all(|b| b.is_standing() && b.cracks() == 0));
}
