//! US-127~129 시험 (수산나): 같은 무기를 또 주우면 강해진다 (D115).
//! 칼·도끼 공격력 +1, 총 공격력 +1과 범위 +36(상한 480). 반복은 모두 횟수가 정해져 있다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::combat::Combat;
use logic::game::{Game, GameEvent};
use logic::monster::{monster_hp_on_day, Zombie};
use logic::player::{Facing, PlayerInput};
use logic::sound::{sfx_for_weapon_event, Sfx};
use logic::weapon::{capped_for_display, is_over_display_cap, DroppedWeapon, Weapon, WeaponEvent, WeaponLevels, Weapons};
use logic::world::World;
use macroquad::math::{Rect, Vec2};

const FRAME: f32 = 1.0 / 60.0;
const BLADES_AND_GUN: [Weapon; 3] = [Weapon::Sword, Weapon::Axe, Weapon::Gun];

fn none() -> PlayerInput {
    PlayerInput::default()
}

fn quiet_game(seed: u32) -> Game {
    let mut g = Game::new(CharacterKind::Niro, seed);
    let boxes = g.world.boxes();
    g.world.obstacles.retain(|b| !boxes.contains(b));
    g.coins.list.clear();
    g.citizens.list.clear();
    g
}

/// 아이템 2개째마다 문제가 열려 게임이 멈추므로 (US-075) 맞히고 닫는다.
fn close_quiz_if_open(g: &mut Game) {
    if !g.math.is_open() {
        return;
    }
    if let Some(q) = g.math.question() {
        let answer = q.answer;
        g.math.type_and_submit(answer);
    } else if let Some(e) = g.math.english_question() {
        let slot = e.answer;
        g.math.pick_choice(slot);
    }
    g.update(&none(), MATH_RIGHT_SHOW_SECONDS);
    assert!(!g.math.is_open(), "문제를 닫았다");
}

/// 무기 하나를 하늘에서 떨어뜨리고 플레이어가 바로 줍게 한다 (한 프레임).
fn pick_up(g: &mut Game, kind: Weapon) {
    close_quiz_if_open(g);
    g.weapons.dropped.push(DroppedWeapon::above(kind, 300.0, &g.world));
    g.player.pos = g.weapons.dropped[0].rect.point();
    g.player.vel = Vec2::ZERO;
    g.update(&none(), FRAME);
    assert!(g.weapons.dropped.is_empty(), "주운 무기는 땅에서 사라진다 {kind:?} {:?}", g.player.pos);
}

fn levels_after(kind: Weapon, times: u32) -> WeaponLevels {
    let mut levels = WeaponLevels::default();
    for _ in 0..times {
        levels.upgrade(kind);
    }
    levels
}

// ---- 순수 규칙: 단계 → 공격력·범위 ----

#[test]
fn constants_are_the_decided_values() {
    assert_eq!((WEAPON_ATTACK_PER_UPGRADE, GUN_RANGE_PER_UPGRADE, GUN_RANGE_MAX), (1, 36.0, 480.0));
    assert_eq!(GUN_RANGE_MAX, SCREEN_WIDTH / 2.0);
    assert_eq!(GUN_RANGE + GUN_RANGE_PER_UPGRADE * 5.0, GUN_RANGE_MAX, "5번 강화하면 상한");
    assert_eq!(WEAPON_LEVEL_DISPLAY_CAP, 99);
}

#[test]
fn without_upgrades_the_numbers_are_the_old_ones() {
    let levels = WeaponLevels::default();
    for kind in Weapon::ORDER {
        assert_eq!(levels.level(kind), 0);
        assert_eq!(levels.attack(kind), kind.attack(), "{kind:?}");
        assert_eq!(levels.range(kind), kind.range(), "{kind:?}");
    }
    assert_eq!((levels.attack(Weapon::Sword), levels.attack(Weapon::Axe), levels.attack(Weapon::Gun)), (2, 3, 4));
    assert_eq!(levels.range(Weapon::Gun), 300.0);
}

#[test]
fn blades_gain_one_attack_each_and_keep_their_range() {
    for (kind, base) in [(Weapon::Sword, 2), (Weapon::Axe, 3)] {
        for n in 0..=10 {
            let levels = levels_after(kind, n);
            assert_eq!(levels.level(kind), n);
            assert_eq!(levels.attack(kind), base + n, "{kind:?} +{n}");
            assert_eq!(levels.range(kind), PLAYER_ATTACK_RANGE, "닿는 거리는 그대로");
        }
    }
}

#[test]
fn each_weapon_counts_on_its_own() {
    let mut levels = WeaponLevels::default();
    levels.upgrade(Weapon::Sword);
    levels.upgrade(Weapon::Sword);
    levels.upgrade(Weapon::Gun);
    assert_eq!((levels.level(Weapon::Sword), levels.level(Weapon::Axe), levels.level(Weapon::Gun)), (2, 0, 1));
    assert_eq!((levels.attack(Weapon::Sword), levels.attack(Weapon::Axe), levels.attack(Weapon::Gun)), (4, 3, 5));
}

#[test]
fn gun_range_goes_300_336_to_480_and_stops_but_attack_keeps_rising() {
    let expected = [300.0, 336.0, 372.0, 408.0, 444.0, 480.0];
    for (n, range) in expected.into_iter().enumerate() {
        let levels = levels_after(Weapon::Gun, n as u32);
        assert_eq!(levels.range(Weapon::Gun), range, "총 +{n}");
        assert_eq!(levels.attack(Weapon::Gun), 4 + n as u32);
    }
    for n in [6, 7, 10, 100, 1000] {
        let levels = levels_after(Weapon::Gun, n);
        assert_eq!(levels.range(Weapon::Gun), 480.0, "총 +{n}: 범위는 480에서 멈춘다");
        assert_eq!(levels.attack(Weapon::Gun), 4 + n, "총 +{n}: 공격력은 계속 오른다");
    }
}

#[test]
fn hands_cannot_be_upgraded() {
    let mut levels = WeaponLevels::default();
    assert_eq!(levels.upgrade(Weapon::Hands), None);
    assert_eq!(levels.level(Weapon::Hands), 0);
    assert_eq!(levels.attack(Weapon::Hands), HANDS_ATTACK);
    assert_eq!(levels.range(Weapon::Hands), PLAYER_ATTACK_RANGE);
    assert_eq!(levels, WeaponLevels::default());
}

#[test]
fn huge_levels_do_not_overflow() {
    for near_max in [u32::MAX - 1, u32::MAX] {
        let mut levels = WeaponLevels::with_counts(near_max, near_max, near_max);
        for kind in BLADES_AND_GUN {
            assert_eq!(levels.upgrade(kind), Some(u32::MAX), "{kind:?}: 더 올라도 넘치지 않는다");
            assert_eq!(levels.upgrade(kind), Some(u32::MAX));
            assert_eq!(levels.attack(kind), u32::MAX, "{kind:?}: 공격력도 넘치지 않는다");
        }
        assert_eq!(levels.range(Weapon::Gun), GUN_RANGE_MAX);
        assert_eq!(levels.range(Weapon::Sword), PLAYER_ATTACK_RANGE);
    }
    assert_eq!(capped_for_display(u32::MAX), WEAPON_LEVEL_DISPLAY_CAP);
    assert!(is_over_display_cap(u32::MAX));
}

#[test]
fn display_cap_for_the_screen() {
    for (level, shown, over) in [(0, 0, false), (1, 1, false), (98, 98, false), (99, 99, false), (100, 99, true), (5000, 99, true)] {
        assert_eq!(capped_for_display(level), shown, "{level}");
        assert_eq!(is_over_display_cap(level), over, "{level}");
    }
}

#[test]
fn sound_for_an_upgrade_is_the_short_bright_one() {
    assert_eq!(sfx_for_weapon_event(&WeaponEvent::Upgraded { weapon: Weapon::Gun, level: 3 }), Some(Sfx::LevelUp));
}

// ---- 줍기: 처음 vs 또 ----

#[test]
fn first_pick_up_is_level_zero_and_second_upgrades() {
    let mut g = quiet_game(4);
    pick_up(&mut g, Weapon::Sword);
    assert_eq!(g.weapons.levels().level(Weapon::Sword), 0, "처음은 0");
    assert_eq!(g.weapons.held(), Weapon::Sword);
    assert!(g.take_weapon_events().is_empty(), "처음 줍기에는 강화 소식이 없다");
    pick_up(&mut g, Weapon::Sword);
    assert_eq!(g.weapons.levels().level(Weapon::Sword), 1);
    assert_eq!(g.take_weapon_events(), vec![WeaponEvent::Upgraded { weapon: Weapon::Sword, level: 1 }]);
    assert!(g.take_weapon_events().is_empty(), "한 번 꺼내면 비어 있다");
    assert_eq!(g.weapons.held_attack(), 3);
    assert_eq!(g.weapons.owned(), [Weapon::Sword], "종류는 그대로 하나");
}

#[test]
fn first_pick_up_still_sends_weapon_picked_and_upgrade_does_not() {
    let mut g = quiet_game(4);
    g.weapons.dropped.push(DroppedWeapon::above(Weapon::Axe, 300.0, &g.world));
    g.player.pos = g.weapons.dropped[0].rect.point();
    g.player.vel = Vec2::ZERO;
    let first = g.update(&none(), FRAME);
    assert!(first.contains(&GameEvent::WeaponPicked { kind: Weapon::Axe }));
    g.weapons.dropped.push(DroppedWeapon::above(Weapon::Axe, 300.0, &g.world));
    g.player.pos = g.weapons.dropped[0].rect.point();
    let second = g.update(&none(), FRAME);
    assert!(!second.iter().any(|e| matches!(e, GameEvent::WeaponPicked { .. })));
    assert_eq!(g.weapons.levels().level(Weapon::Axe), 1);
}

#[test]
fn upgrading_a_weapon_in_the_bag_does_not_change_the_one_in_hand() {
    let mut g = quiet_game(4);
    pick_up(&mut g, Weapon::Sword);
    pick_up(&mut g, Weapon::Axe);
    assert_eq!(g.weapons.held(), Weapon::Axe);
    pick_up(&mut g, Weapon::Sword);
    assert_eq!(g.weapons.held(), Weapon::Axe, "손의 무기는 그대로");
    assert_eq!(g.weapons.levels().level(Weapon::Sword), 1);
    assert_eq!(g.weapons.levels().level(Weapon::Axe), 0);
}

#[test]
fn levels_stay_with_each_weapon_when_switching_with_tab() {
    let mut g = quiet_game(4);
    for kind in [Weapon::Sword, Weapon::Axe, Weapon::Gun, Weapon::Sword, Weapon::Gun, Weapon::Gun] {
        pick_up(&mut g, kind);
    }
    let seen = g.take_weapon_events();
    assert_eq!(seen.len(), 3, "또 주운 3번만 소식이 온다");
    let mut attacks = Vec::new();
    for _ in 0..8 {
        attacks.push((g.weapons.held(), g.weapons.held_attack()));
        g.switch_weapon();
    }
    for (held, attack) in attacks {
        let expected = match held {
            Weapon::Hands => 1,
            Weapon::Sword => 3,
            Weapon::Axe => 3,
            Weapon::Gun => 6,
        };
        assert_eq!(attack, expected, "{held:?}");
    }
    assert_eq!(g.weapons.levels().level(Weapon::Gun), 2);
    assert_eq!(g.weapons.held_range(), if g.weapons.held() == Weapon::Gun { 372.0 } else { g.weapons.held().range() });
}

#[test]
fn a_new_game_starts_over_at_zero() {
    let mut g = quiet_game(4);
    for kind in [Weapon::Gun, Weapon::Gun, Weapon::Gun, Weapon::Sword, Weapon::Sword] {
        pick_up(&mut g, kind);
    }
    assert!(g.weapons.levels().level(Weapon::Gun) > 0);
    let fresh = quiet_game(4);
    assert_eq!(fresh.weapons.levels(), WeaponLevels::default());
    assert!(fresh.weapons.owned().is_empty());
    let mut weapons = Weapons::new(1);
    assert_eq!(weapons.levels(), WeaponLevels::default());
    assert!(weapons.take_events().is_empty());
}

#[test]
fn picked_up_count_still_counts_each_pick_up_once() {
    let mut g = quiet_game(4);
    pick_up(&mut g, Weapon::Gun);
    pick_up(&mut g, Weapon::Gun);
    pick_up(&mut g, Weapon::Gun);
    assert_eq!(g.weapons.picked_up_count, 3, "강화해도 아이템 수는 줍는 만큼 (US-075)");
}

// ---- 하늘에서 떨어지는 무기 ----

#[test]
fn weapons_you_have_drop_again_so_they_can_be_upgraded() {
    let world = World::long_map(1);
    let far = Rect::new(-999.0, -999.0, 1.0, 1.0);
    let mut seen = [0u32; 3];
    for seed in 0..200 {
        let mut weapons = Weapons::new(seed);
        // 3종을 모두 가진다
        for kind in BLADES_AND_GUN {
            weapons.add_bought(kind);
        }
        weapons.drop_one(100.0, &world);
        let kind = weapons.dropped[0].kind;
        weapons.update(0.0, far);
        let slot = BLADES_AND_GUN.iter().position(|k| *k == kind).expect("맨손은 떨어지지 않는다");
        seen[slot] += 1;
    }
    assert!(seen.iter().all(|n| *n > 30), "3종 모두 자주 나온다 {seen:?}");
}

#[test]
fn owned_gun_share_is_about_a_third_and_never_while_a_gun_lies_on_the_ground() {
    let world = World::long_map(1);
    let mut guns = 0;
    for seed in 0..3000 {
        let mut weapons = Weapons::new(seed);
        for kind in BLADES_AND_GUN {
            weapons.add_bought(kind);
        }
        weapons.drop_one(100.0, &world);
        guns += u32::from(weapons.dropped[0].kind == Weapon::Gun);
        weapons.drop_one(300.0, &world);
        assert!(weapons.dropped[0].kind != Weapon::Gun || weapons.dropped[1].kind != Weapon::Gun, "seed {seed}: 땅에 총이 있으면 총이 또 안 떨어진다");
    }
    let share = guns as f32 / 3000.0;
    assert!((0.30..0.37).contains(&share), "총 {share}");
}

// ---- 전투: 몇 방에 쓰러지나 ----

/// 좀비 하나를 옆에 세우고 `attack`·`range`로 몇 방에 쓰러지는지 (한도 20방).
fn hits_on_day(day: u32, attack: u32, range: f32) -> u32 {
    let mut c = Combat::new(7);
    let player = Rect::new(1000.0, GROUND_Y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT);
    c.zombies.list.push(Zombie::new_on_day(day, player.x + PLAYER_WIDTH + 5.0, GROUND_Y, false));
    for hit in 1..=20 {
        c.attack_with(player, Facing::Right, attack, range);
        if c.zombies.list.is_empty() {
            return hit;
        }
    }
    panic!("20방으로도 안 쓰러진다");
}

#[test]
fn hits_table_by_day_weapon_and_upgrades() {
    // (날, 체력) x (칼, 도끼, 총) x 강화 (0, 2, 5번) → 방 수 = 올림(체력 / 공격력)
    for day in [1, 3, 5, 7] {
        let hp = monster_hp_on_day(ZOMBIE_HP, day);
        for kind in BLADES_AND_GUN {
            for n in [0, 2, 5] {
                let levels = levels_after(kind, n);
                let attack = levels.attack(kind);
                let hits = hits_on_day(day, attack, levels.range(kind));
                assert_eq!(hits, hp.div_ceil(attack), "{day}일째(체력 {hp}) {kind:?} +{n} (공격력 {attack})");
            }
        }
    }
}

#[test]
fn some_decided_examples() {
    // 7일째 체력 9
    let axe2 = levels_after(Weapon::Axe, 2);
    assert_eq!(axe2.attack(Weapon::Axe), 5);
    assert_eq!(hits_on_day(7, 5, axe2.range(Weapon::Axe)), 2);
    let gun5 = levels_after(Weapon::Gun, 5);
    assert_eq!((gun5.attack(Weapon::Gun), gun5.range(Weapon::Gun)), (9, 480.0));
    assert_eq!(hits_on_day(7, 9, gun5.range(Weapon::Gun)), 1);
    // 기본값: 1일째 체력 3, 칼 2방, 도끼 1방, 총 1방, 맨손 3방
    assert_eq!(hits_on_day(1, WeaponLevels::default().attack(Weapon::Sword), PLAYER_ATTACK_RANGE), 2);
    assert_eq!(hits_on_day(1, WeaponLevels::default().attack(Weapon::Axe), PLAYER_ATTACK_RANGE), 1);
    assert_eq!(hits_on_day(1, WeaponLevels::default().attack(Weapon::Hands), PLAYER_ATTACK_RANGE), 3);
}

#[test]
fn gun_range_is_what_decides_who_is_hit() {
    // 총 +0은 300, 총 +5는 480까지 닿는다. 400 떨어진 좀비는 +0에 안 맞고 +5에 맞는다.
    let player = Rect::new(1000.0, GROUND_Y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT);
    let far_zombie_x = player.x + PLAYER_WIDTH + 400.0;
    for (n, should_hit) in [(0, false), (5, true), (50, true)] {
        let levels = levels_after(Weapon::Gun, n);
        let mut c = Combat::new(7);
        c.zombies.list.push(Zombie::new_on_day(1, far_zombie_x, GROUND_Y, false));
        let events = c.attack_with(player, Facing::Right, levels.attack(Weapon::Gun), levels.range(Weapon::Gun));
        assert_eq!(!events.is_empty(), should_hit, "총 +{n}");
    }
}

#[test]
fn attack_in_game_uses_the_upgraded_attack_and_range() {
    let mut g = quiet_game(4);
    for kind in [Weapon::Gun, Weapon::Gun, Weapon::Gun] {
        pick_up(&mut g, kind);
    }
    close_quiz_if_open(&mut g);
    g.player.pos = Vec2::new(300.0, g.world.ground_y - PLAYER_HEIGHT);
    g.player.vel = Vec2::ZERO;
    g.player.facing = Facing::Right;
    assert_eq!(g.weapons.held(), Weapon::Gun);
    assert!(g.attack_reach() <= 372.0, "총 +2의 범위 372를 넘지 않는다 (벽에 막히면 더 짧다)");
    // 몸 앞 100픽셀에 좀비 (체력 3일 때 공격력 6이면 한 방)
    let x = g.player.rect().x + PLAYER_WIDTH + 100.0;
    g.combat.zombies.list.push(Zombie::new_on_day(1, x, g.world.ground_y, false));
    let attack = PlayerInput { attack_pressed: true, ..Default::default() };
    let events = g.update(&attack, FRAME);
    assert!(events.iter().any(|e| matches!(e, GameEvent::Combat(logic::combat::CombatEvent::ZombieDefeated { .. }))), "{events:?}");
}

/// 총을 `times`번 더 주워 (처음 한 번 + 강화) 든 채, 몸을 `x`에 세운 게임.
fn gun_game_at(x: f32, upgrades: u32, facing: Facing) -> Game {
    let mut g = quiet_game(9);
    g.world.obstacles.clear(); // 바위·벽이 빛줄기를 막지 않게 한다
    for _ in 0..=upgrades {
        pick_up(&mut g, Weapon::Gun);
    }
    close_quiz_if_open(&mut g);
    g.player.pos = Vec2::new(x, g.world.ground_y - PLAYER_HEIGHT);
    g.player.vel = Vec2::ZERO;
    g.player.facing = facing;
    g
}

/// 몸 가운데에서 `edge_from_center`만큼 떨어진 곳에 좀비 가까운 쪽 끝이 오게 두고 총을 쏴 맞는지 본다.
fn gun_hits_zombie_edge_at(g: &Game, edge_from_center: f32) -> bool {
    let body = g.player.rect();
    let center = body.center().x;
    let x = match g.player.facing {
        Facing::Right => center + edge_from_center,
        Facing::Left => center - edge_from_center - ZOMBIE_WIDTH,
    };
    let mut c = Combat::new(7);
    c.zombies.list.push(Zombie::new_on_day(1, x, g.world.ground_y, false));
    let attack = g.weapons.held_attack();
    !c.attack_with(body, g.player.facing, attack, g.attack_reach()).is_empty()
}

#[test]
fn gun_reach_is_measured_from_body_center_up_to_screen_edge() {
    // 총 +5: 범위 480은 몸 가운데에서 화면 끝까지. 빛줄기는 몸 앞 끝에서 456이다.
    for facing in [Facing::Right, Facing::Left] {
        let g = gun_game_at(2000.0, 5, facing);
        assert_eq!(g.weapons.held_range(), 480.0);
        assert_eq!(g.attack_reach(), GUN_RANGE_MAX - PLAYER_WIDTH / 2.0, "{facing:?}");
        assert!(gun_hits_zombie_edge_at(&g, 479.0), "{facing:?}: 화면 끝 안쪽은 맞는다");
        assert!(!gun_hits_zombie_edge_at(&g, 481.0), "{facing:?}: 화면 밖은 안 맞는다");
        assert!(!gun_hits_zombie_edge_at(&g, 500.0), "{facing:?}: 가운데에서 500 밖");
    }
}

#[test]
fn gun_reach_for_upgrades_zero_one_two_stays_as_before() {
    // 강화 0·1·2는 300·336·372 그대로, 몸 앞 끝에서 잰다.
    for (n, range) in [(0, 300.0), (1, 336.0), (2, 372.0)] {
        let g = gun_game_at(2000.0, n, Facing::Right);
        assert_eq!(g.attack_reach(), range, "총 +{n}");
        let front_gap = range - 1.0;
        assert!(gun_hits_zombie_edge_at(&g, PLAYER_WIDTH / 2.0 + front_gap), "총 +{n}: 안쪽");
        assert!(!gun_hits_zombie_edge_at(&g, PLAYER_WIDTH / 2.0 + range + 1.0), "총 +{n}: 바깥");
    }
}

#[test]
fn gun_reach_never_passes_screen_edge_even_with_huge_upgrades() {
    let g = gun_game_at(2000.0, 12, Facing::Right);
    assert_eq!(g.attack_reach(), GUN_REACH_MAX_FROM_FRONT);
    assert!(PLAYER_WIDTH / 2.0 + g.attack_reach() <= SCREEN_WIDTH / 2.0);
}
