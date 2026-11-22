//! US-135 시험 (수산나): 날이 지날수록 새로 나오는 몬스터의 체력이 세진다.
//! 1일째 3, 2일째부터 하루 +1, 7일째 9, 그 뒤로도 9. 반복은 모두 횟수가 정해져 있다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::boxes::BoxMonster;
use logic::combat::{Combat, CombatEvent};
use logic::monster::{monster_hp_on_day, Zombie};
use logic::player::Facing;
use logic::practice::PracticeMonster;
use logic::skeleton::{Skeleton, SkeletonStage};
use macroquad::math::Rect;

const FRAME: f32 = 1.0 / 60.0;
const SEED: u32 = 7;

/// 날 → 체력 표 (임시-166)
const TABLE: [(u32, u32); 7] = [(1, 3), (2, 4), (3, 5), (4, 6), (5, 7), (6, 8), (7, 9)];

fn player_rect() -> Rect {
    Rect::new(1000.0, GROUND_Y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT)
}

fn open_world() -> logic::world::World {
    logic::world::World { width: 3000.0, ground_y: GROUND_Y, obstacles: Vec::new() }
}

/// `weapon_attack`로 몇 방에 쓰러지는지 (한도 20방). 체력과 무기 공격력만 본다.
fn hits_to_fall(hp: u32, weapon_attack: u32) -> u32 {
    let mut left = hp;
    for hit in 1..=20 {
        left = left.saturating_sub(weapon_attack);
        if left == 0 {
            return hit;
        }
    }
    panic!("20방으로도 안 쓰러진다");
}

// ---- 순수 함수: 날 → 체력 ----

#[test]
fn hp_follows_the_table_for_zombie_and_skeleton() {
    for (day, hp) in TABLE {
        assert_eq!(monster_hp_on_day(ZOMBIE_HP, day), hp, "{day}일째 좀비");
        assert_eq!(monster_hp_on_day(SKELETON_HP, day), hp, "{day}일째 스켈레톤");
        assert_eq!(Zombie::new_on_day(day, 0.0, GROUND_Y, false).hp, hp, "{day}일째 Zombie::new_on_day");
        for stage in [SkeletonStage::Normal, SkeletonStage::Half, SkeletonStage::Full] {
            assert_eq!(Skeleton::new_on_day(day, 0.0, GROUND_Y, stage).hp, hp, "{day}일째 스켈레톤 {stage:?}");
        }
    }
}

#[test]
fn first_day_is_the_same_as_before() {
    assert_eq!(monster_hp_on_day(ZOMBIE_HP, 1), ZOMBIE_HP);
    assert_eq!(monster_hp_on_day(SKELETON_HP, 1), SKELETON_HP);
    assert_eq!(Zombie::new(0.0, GROUND_Y, false).hp, ZOMBIE_HP);
    assert_eq!(Skeleton::new(0.0, GROUND_Y, SkeletonStage::Normal).hp, SKELETON_HP);
    assert_eq!(monster_hp_on_day(ZOMBIE_HP, 0), ZOMBIE_HP, "0일째는 없지만 1일째로 본다");
}

#[test]
fn hp_stops_at_the_most_and_never_overflows() {
    for day in [8, 14, 100, 1000, u32::MAX - 1, u32::MAX] {
        assert_eq!(monster_hp_on_day(ZOMBIE_HP, day), MONSTER_HP_MOST, "{day}일째");
        assert_eq!(Zombie::new_on_day(day, 0.0, GROUND_Y, false).hp, MONSTER_HP_MOST, "{day}일째 좀비");
        assert_eq!(Skeleton::new_on_day(day, 0.0, GROUND_Y, SkeletonStage::Half).hp, MONSTER_HP_MOST, "{day}일째 스켈레톤");
    }
    assert_eq!(MONSTER_HP_MOST, 9);
}

#[test]
fn hp_never_goes_down_as_days_pass() {
    for day in 1..=30 {
        assert!(monster_hp_on_day(ZOMBIE_HP, day + 1) >= monster_hp_on_day(ZOMBIE_HP, day), "{day}일째");
    }
}

#[test]
fn constants_are_the_decided_values() {
    assert_eq!((MONSTER_HP_MORE_PER_DAY, MONSTER_HP_MOST), (1, 9));
    assert!(ZOMBIE_HP <= MONSTER_HP_MOST && SKELETON_HP <= MONSTER_HP_MOST);
}

// ---- 어디서 나오든 그날 체력 ----

#[test]
fn night_spawns_use_the_day_hp() {
    let world = open_world();
    for (day, hp) in TABLE {
        let mut c = Combat::new(SEED);
        c.day_number = day;
        c.health.set_armored(true);
        // 주인공은 아주 멀리 공중에 둔다 (몬스터가 닿지 않게)
        let far = Rect::new(1500.0, -10_000.0, PLAYER_WIDTH, PLAYER_HEIGHT);
        for _ in 0..(30.0 / FRAME) as usize {
            c.update(FRAME, true, far, &world);
        }
        let all = c.zombies.list.len() + c.skeletons.list.len();
        assert!(all > 0, "{day}일째 밤에 몬스터가 나온다");
        assert!(c.zombies.list.iter().all(|z| z.hp == hp), "{day}일째 좀비 체력");
        assert!(c.skeletons.list.iter().all(|s| s.hp == hp), "{day}일째 스켈레톤 체력");
    }
}

#[test]
fn night_spawns_include_both_kinds_with_the_day_hp() {
    // 두 종류를 다 보려고 스켈레톤 확률을 1과 0으로 나눠서 본다.
    let world = open_world();
    let far = Rect::new(1500.0, -10_000.0, PLAYER_WIDTH, PLAYER_HEIGHT);
    for chance in [0.0, 1.0] {
        let mut c = Combat::new(SEED);
        c.day_number = 5;
        c.skeleton_chance = chance;
        c.health.set_armored(true);
        // 5초: 첫 차례(5일째 3마리)만 나와서 좀비가 가득 차지 않는다.
        for _ in 0..(5.0 / FRAME) as usize {
            c.update(FRAME, true, far, &world);
        }
        if chance == 0.0 {
            assert!(!c.zombies.list.is_empty() && c.skeletons.list.is_empty());
            assert!(c.zombies.list.iter().all(|z| z.hp == 7));
        } else {
            assert!(c.zombies.list.is_empty() && !c.skeletons.list.is_empty());
            assert!(c.skeletons.list.iter().all(|s| s.hp == 7));
        }
    }
}

#[test]
fn box_monsters_use_the_day_hp() {
    for (day, hp) in TABLE {
        for wanted in [BoxMonster::Zombie, BoxMonster::Skeleton] {
            let mut c = Combat::new(SEED);
            c.day_number = day;
            let event = c.spawn_from_box(500.0, GROUND_Y, wanted);
            assert!(event.is_some(), "{day}일째 상자 몬스터가 나온다");
            let (zombie_hp, skeleton_hp) = (c.zombies.list.first().map(|z| z.hp), c.skeletons.list.first().map(|s| s.hp));
            match wanted {
                BoxMonster::Zombie => assert_eq!(zombie_hp, Some(hp), "{day}일째 상자 좀비"),
                BoxMonster::Skeleton => assert_eq!(skeleton_hp, Some(hp), "{day}일째 상자 스켈레톤"),
            }
        }
    }
}

#[test]
fn box_monsters_when_zombies_are_full_still_use_the_day_hp() {
    // 좀비가 차면 상자 좀비 대신 스켈레톤이 나온다. 그것도 그날 체력이다.
    let mut c = Combat::new(SEED);
    c.day_number = 3;
    for i in 0..20 {
        c.spawn_from_box(100.0 + i as f32 * 60.0, GROUND_Y, BoxMonster::Zombie);
    }
    assert!(!c.skeletons.list.is_empty());
    assert!(c.zombies.list.iter().all(|z| z.hp == 5));
    assert!(c.skeletons.list.iter().all(|s| s.hp == 5));
}

#[test]
fn box_waiting_monsters_keep_waiting() {
    let z = Zombie::waiting(7, 0.0, GROUND_Y, false, 2.0);
    assert!(z.is_waiting());
    assert_eq!(z.hp, 9);
    let s = Skeleton::waiting(7, 0.0, GROUND_Y, SkeletonStage::Normal, 2.0);
    assert!(s.is_waiting());
    assert_eq!(s.hp, 9);
}

// ---- 이미 나온 몬스터는 그대로 ----

#[test]
fn monsters_already_out_keep_their_hp_when_the_day_changes() {
    let mut c = Combat::new(SEED);
    c.day_number = 2;
    // 주인공(x 1500) 가까이에 둬서 너무 멀다고 잊히지 않게 한다.
    c.zombies.list.push(Zombie::new_on_day(c.day_number, 1300.0, GROUND_Y, false));
    c.skeletons.list.push(Skeleton::new_on_day(c.day_number, 1700.0, GROUND_Y, SkeletonStage::Normal));
    c.day_number = 7;
    assert_eq!(c.zombies.list[0].hp, 4);
    assert_eq!(c.skeletons.list[0].hp, 4);
    // 그 뒤 밤이 와도 (몬스터가 낮 동안 남아 있다가) 새로 나온 것만 9다
    let far = Rect::new(1500.0, -10_000.0, PLAYER_WIDTH, PLAYER_HEIGHT);
    c.health.set_armored(true);
    for _ in 0..(10.0 / FRAME) as usize {
        c.update(FRAME, true, far, &open_world());
    }
    assert_eq!(c.zombies.list.iter().filter(|z| z.hp == 4).count(), 1, "먼저 나온 좀비는 그대로 4");
    assert_eq!(c.skeletons.list.iter().filter(|s| s.hp == 4).count(), 1, "먼저 나온 스켈레톤은 그대로 4");
}

#[test]
fn a_hit_zombie_does_not_heal_when_the_day_changes() {
    let mut c = Combat::new(SEED);
    c.day_number = 7;
    let player = player_rect();
    c.zombies.list.push(Zombie::new_on_day(7, player.right() + 5.0, GROUND_Y, false));
    c.attack_with(player, Facing::Right, AXE_ATTACK, PLAYER_ATTACK_RANGE);
    assert_eq!(c.zombies.list[0].hp, 6);
    c.day_number = 8;
    assert_eq!(c.zombies.list[0].hp, 6);
}

// ---- 연습용 몬스터와 다른 값은 그대로 ----

#[test]
fn practice_monster_and_other_values_do_not_change() {
    assert_eq!(PRACTICE_MONSTER_HP, 2);
    assert_eq!(PracticeMonster::new(0.0, GROUND_Y).hp, 2);
    assert_eq!((PLAYER_START_HP, PLAYER_HP_PER_LEVEL), (7, 5));
    assert_eq!(ZOMBIE_ATTACK, 2);
    assert_eq!((SKELETON_NORMAL_ATTACK, SKELETON_HALF_ATTACK, SKELETON_FULL_ATTACK), (2, 3, 4));
    assert_eq!((SKELETON_HALF_MIN_LEVEL, SKELETON_FULL_MIN_LEVEL), (2, 3));
    assert_eq!(PLAYER_HURT_INVINCIBLE_SECONDS, 5.0);
    assert_eq!((HANDS_ATTACK, SWORD_ATTACK, AXE_ATTACK, GUN_ATTACK), (1, 2, 3, 4));
}

#[test]
fn skeleton_stage_follows_player_level_not_the_day() {
    for day in [1, 7, 100] {
        let mut c = Combat::new(SEED);
        c.day_number = day;
        for (level, stage) in [(1, SkeletonStage::Normal), (2, SkeletonStage::Half), (3, SkeletonStage::Full)] {
            c.level = level;
            c.zombies.list.clear();
            c.skeletons.list.clear();
            c.spawn_from_box(500.0, GROUND_Y, BoxMonster::Skeleton);
            assert_eq!(c.skeletons.list[0].stage, stage, "{day}일째 레벨 {level}");
        }
    }
}

#[test]
fn monster_attack_on_day_7_is_still_the_same() {
    // 7일째 좀비에게 닿아도 플레이어는 ZOMBIE_ATTACK만큼만 다친다.
    let mut c = Combat::new(SEED);
    c.day_number = 7;
    let player = player_rect();
    c.zombies.list.push(Zombie::new_on_day(7, player.x, GROUND_Y, false));
    let before = c.health.hp();
    for _ in 0..10 {
        c.update(FRAME, true, player, &open_world());
    }
    assert_eq!(before - c.health.hp(), ZOMBIE_ATTACK);
}

// ---- 몇 방에 쓰러지나 (순수 계산 + 실제 Combat) ----

#[test]
fn hits_to_fall_table() {
    // (날, 맨손, 칼, 도끼, 총)
    let expected = [
        (1, 3, 2, 1, 1),
        (2, 4, 2, 2, 1),
        (3, 5, 3, 2, 2),
        (4, 6, 3, 2, 2),
        (5, 7, 4, 3, 2),
        (6, 8, 4, 3, 2),
        (7, 9, 5, 3, 3),
        (8, 9, 5, 3, 3),
    ];
    for (day, hands, sword, axe, gun) in expected {
        let hp = monster_hp_on_day(ZOMBIE_HP, day);
        assert_eq!(hits_to_fall(hp, HANDS_ATTACK), hands, "{day}일째 맨손");
        assert_eq!(hits_to_fall(hp, SWORD_ATTACK), sword, "{day}일째 칼");
        assert_eq!(hits_to_fall(hp, AXE_ATTACK), axe, "{day}일째 도끼");
        assert_eq!(hits_to_fall(hp, GUN_ATTACK), gun, "{day}일째 총");
    }
}

/// 7일째 몬스터(좀비나 스켈레톤)를 `attack` 무기로 때려서 몇 방에 쓰러지는지 (한도 20방).
fn real_hits_to_fall_on_day_7(attack: u32, skeleton: bool) -> u32 {
    let mut c = Combat::new(SEED);
    c.day_number = 7;
    let player = player_rect();
    if skeleton {
        c.skeletons.list.push(Skeleton::new_on_day(7, player.right() + 5.0, GROUND_Y, SkeletonStage::Half));
    } else {
        c.zombies.list.push(Zombie::new_on_day(7, player.right() + 5.0, GROUND_Y, false));
    }
    for hit in 1..=20 {
        let events = c.attack_with(player, Facing::Right, attack, PLAYER_ATTACK_RANGE);
        let fell = events.iter().any(|e| matches!(e, CombatEvent::ZombieDefeated { .. } | CombatEvent::SkeletonDefeated { .. }));
        if fell {
            return hit;
        }
    }
    panic!("20방으로도 안 쓰러진다");
}

#[test]
fn day_7_monster_falls_in_3_hits_with_axe_and_gun() {
    for skeleton in [false, true] {
        assert_eq!(real_hits_to_fall_on_day_7(AXE_ATTACK, skeleton), 3, "도끼 skeleton={skeleton}");
        assert_eq!(real_hits_to_fall_on_day_7(GUN_ATTACK, skeleton), 3, "총 skeleton={skeleton}");
        assert_eq!(real_hits_to_fall_on_day_7(HANDS_ATTACK, skeleton), 9, "맨손 skeleton={skeleton}");
        assert_eq!(real_hits_to_fall_on_day_7(SWORD_ATTACK, skeleton), 5, "칼 skeleton={skeleton}");
    }
}

#[test]
fn defeating_a_day_7_monster_still_gives_experience_and_counts_for_missions() {
    let mut c = Combat::new(SEED);
    c.day_number = 7;
    let player = player_rect();
    c.zombies.list.push(Zombie::new_on_day(7, player.right() + 5.0, GROUND_Y, false));
    for _ in 0..3 {
        c.attack_with(player, Facing::Right, AXE_ATTACK, PLAYER_ATTACK_RANGE);
    }
    assert!(c.zombies.list.is_empty());
    assert_eq!((c.monsters_defeated, c.monsters_given_experience), (1, 1));
}

// ---- 한꺼번에 쓰러뜨리기와 독 ----

#[test]
fn sweep_and_blast_defeat_full_hp_monsters_at_once() {
    let mut c = Combat::new(SEED);
    c.day_number = 7;
    let player = player_rect();
    let x = player.x + 100.0;
    c.zombies.list.push(Zombie::new_on_day(7, x, GROUND_Y, false));
    c.skeletons.list.push(Skeleton::new_on_day(7, x + 60.0, GROUND_Y, SkeletonStage::Full));
    // 주사기총·쿵: 화면에 보이면 체력 9여도 한꺼번에
    let events = c.sweep_monsters_on_screen(player.x - 400.0);
    assert_eq!(events.len(), 2);
    assert!(c.zombies.list.is_empty() && c.skeletons.list.is_empty());

    // 시민 폭발: 가까우면 체력 9여도 한꺼번에
    c.zombies.list.push(Zombie::new_on_day(7, x, GROUND_Y, false));
    c.skeletons.list.push(Skeleton::new_on_day(7, x + 60.0, GROUND_Y, SkeletonStage::Full));
    let at = macroquad::math::Vec2::new(x + 40.0, GROUND_Y - 30.0);
    let far_player = Rect::new(10.0, GROUND_Y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT);
    c.citizen_blast(at, far_player);
    assert!(c.zombies.list.is_empty() && c.skeletons.list.is_empty());
}

#[test]
fn poison_still_hurts_one_per_second_on_a_full_hp_monster() {
    let mut c = Combat::new(SEED);
    c.day_number = 7;
    let player = player_rect();
    let x = player.x + 300.0;
    c.zombies.list.push(Zombie::new_on_day(7, x, GROUND_Y, false));
    c.skeletons.list.push(Skeleton::new_on_day(7, x + 100.0, GROUND_Y, SkeletonStage::Half));
    assert_eq!(c.spray_poison(macroquad::math::Vec2::new(x + 50.0, GROUND_Y - 30.0)), 2);
    c.health.set_armored(true);
    // 낮에 돌려서 새 몬스터가 안 나오게 한다. 1초 지날 때마다 1씩 준다.
    let world = open_world();
    let mut frames_run = 0usize;
    for second in 1..=3u32 {
        // 1초 눈금을 막 지난 0.2초 뒤에 본다 (실수 오차로 눈금 위에서 어긋나지 않게).
        let until = ((second as f32 + 0.2) / FRAME) as usize;
        for _ in frames_run..until {
            c.update(FRAME, false, player, &world);
        }
        frames_run = until;
        let expected = MONSTER_HP_MOST - POISON_DAMAGE * second;
        assert!(c.zombies.list.iter().all(|z| z.hp == expected), "{second}초 뒤 좀비 {:?}", c.zombies.list.iter().map(|z| z.hp).collect::<Vec<_>>());
        assert!(c.skeletons.list.iter().all(|s| s.hp == expected), "{second}초 뒤 스켈레톤");
    }
}
