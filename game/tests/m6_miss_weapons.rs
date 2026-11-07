//! M6 미스, 무기, 폭발형 시민 테스트 (수산나).
//! 기준: stories/backlog.md US-051~054, 임시-5~9.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::combat::CombatEvent;
use logic::game::{Game, GameEvent};
use logic::monster::{Zombie, ZombieState};
use logic::player::{Facing, PlayerInput};
use logic::skeleton::{Projectile, ProjectileKind, Skeleton, SkeletonStage};
use logic::weapon::{DroppedWeapon, Weapon};
use logic::world::World;
use macroquad::math::{Rect, Vec2};

const FRAME: f32 = 1.0 / 60.0;

fn none() -> PlayerInput {
    PlayerInput::default()
}

fn new_game(seed: u32) -> Game {
    Game::new(CharacterKind::Niro, seed)
}

/// 상자·시민이 없는 긴 맵의 게임. 미스를 찾을 때 다른 일이 끼어들지 않는다.
fn quiet_game(seed: u32) -> Game {
    let mut g = new_game(seed);
    let boxes = g.world.boxes();
    g.world.obstacles.retain(|b| !boxes.contains(b));
    g.coins.list.clear();
    g.citizens.list.clear();
    g
}

fn run(g: &mut Game, input: &PlayerInput, seconds: f32) -> Vec<GameEvent> {
    let mut events = Vec::new();
    for _ in 0..(seconds / FRAME) as usize {
        events.extend(g.update(input, FRAME));
    }
    events
}

fn stand(g: &mut Game, x: f32) {
    g.player.pos = Vec2::new(x, g.world.ground_y - PLAYER_HEIGHT);
    g.player.vel = Vec2::ZERO;
}

fn miss_rect(g: &Game) -> Rect {
    g.miss.as_ref().expect("긴 맵에는 미스가 있다").rect(&g.world)
}

/// 플레이어를 미스 몸 위에 놓고 한 프레임 돌려서 찾는다.
fn find_miss(g: &mut Game) -> Vec<GameEvent> {
    let r = miss_rect(g);
    g.player.pos = Vec2::new(r.x, r.y + r.h - PLAYER_HEIGHT);
    g.player.vel = Vec2::ZERO;
    g.update(&none(), FRAME)
}

/// 다음 06:00까지 시계를 돌린다.
fn go_to_next_morning(g: &mut Game) {
    stand(g, 40.0);
    let day = g.clock.day_number();
    g.clock.update(DAY_SECONDS + NIGHT_SECONDS);
    assert_eq!(g.clock.day_number(), day + 1);
    g.update(&none(), FRAME);
}

// ---- US-051 미스 ----

#[test]
fn all_six_hide_spots_appear_across_new_games() {
    let mut seen = [false; HIDE_SPOTS.len()];
    for seed in 0..500 {
        let g = new_game(seed);
        let x = miss_rect(&g).center().x;
        let index = HIDE_SPOTS.iter().position(|spot| spot.1 == x).unwrap();
        seen[index] = true;
    }
    assert!(seen.iter().all(|s| *s), "{seen:?}");
}

#[test]
fn m1_practice_map_has_no_miss_and_no_citizens() {
    let g = Game::on_world(CharacterKind::Niro, 1, World::m1());
    assert!(g.miss.is_none());
    assert!(g.citizens.list.is_empty());
}

#[test]
fn miss_stands_on_the_surface_of_every_spot() {
    let g = new_game(1);
    let hole_floor = g.world.ground_y + HOLE_DEPTH;
    let top = g.world.ground_y - CLIMB_STEP * 3.0;
    let expected = [g.world.ground_y, g.world.ground_y, hole_floor, top, top, g.world.ground_y];
    for (index, (name, x, _)) in HIDE_SPOTS.iter().enumerate() {
        // 그 곳에 숨는 시드를 찾아 진짜 Miss로 본다.
        let mut g = quiet_game(1);
        for seed in 0..500 {
            let m = logic::miss::Miss::new(seed);
            if m.rect(&g.world).center().x == *x {
                g.miss = Some(m);
                break;
            }
        }
        let r = miss_rect(&g);
        assert_eq!(r.center().x, *x);
        assert_eq!(r.y + r.h, expected[index], "{name}");
        let body_under = Rect::new(r.x, r.y, r.w, r.h);
        assert_eq!(g.world.surface_y_under(body_under), expected[index], "{name} 발 밑이 실제 윗면이다");
    }
}

#[test]
fn touching_miss_finds_her_day_or_night_and_she_disappears() {
    for night in [false, true] {
        let mut g = quiet_game(3);
        g.combat.skeleton_chance = 0.0;
        if night {
            g.clock.update(DAY_SECONDS);
        }
        let at = miss_rect(&g).center();
        let events = find_miss(&mut g);
        assert!(events.contains(&GameEvent::MissFound { at }), "{events:?}");
        assert!(!g.miss.as_ref().unwrap().is_hiding());
        let again = run(&mut g, &none(), 0.5);
        assert!(!again.iter().any(|e| matches!(e, GameEvent::MissFound { .. })), "한 번만 찾는다");
    }
}

#[test]
fn found_miss_hides_somewhere_else_after_20_seconds_and_can_be_found_again() {
    for seed in 0..200 {
        let mut g = quiet_game(seed);
        let first = miss_rect(&g);
        find_miss(&mut g);
        stand(&mut g, 40.0);
        g.update(&none(), MISS_RETURN_SECONDS + 0.1);
        assert!(g.miss.as_ref().unwrap().is_hiding(), "seed {seed}");
        assert_ne!(miss_rect(&g), first, "seed {seed}: 조금 전과 다른 곳");
        let events = find_miss(&mut g);
        assert!(events.iter().any(|e| matches!(e, GameEvent::MissFound { .. })));
    }
}

#[test]
fn found_miss_comes_back_after_20_real_seconds_not_by_the_clock() {
    let mut g = quiet_game(5);
    find_miss(&mut g);
    stand(&mut g, 40.0);
    g.clock.update(DAY_SECONDS + NIGHT_SECONDS);
    g.update(&none(), FRAME);
    assert!(!g.miss.as_ref().unwrap().is_hiding(), "아침이 와도 시간이 다 차기 전에는 안 나온다");
    run(&mut g, &none(), MISS_RETURN_SECONDS - 1.0);
    assert!(!g.miss.as_ref().unwrap().is_hiding());
    run(&mut g, &none(), 1.5);
    assert!(g.miss.as_ref().unwrap().is_hiding());
}

#[test]
fn unfound_miss_stays_in_place_through_time_and_mornings() {
    let mut g = quiet_game(5);
    let before = miss_rect(&g);
    stand(&mut g, 40.0);
    run(&mut g, &none(), MISS_RETURN_SECONDS * 2.0);
    for _ in 0..3 {
        go_to_next_morning(&mut g);
    }
    assert!(g.miss.as_ref().unwrap().is_hiding());
    assert_eq!(miss_rect(&g), before);
}

#[test]
fn miss_does_not_block_the_player_and_monsters_do_nothing_to_her() {
    let mut g = quiet_game(9);
    let r = miss_rect(&g);
    g.combat.zombies.list.push(Zombie::new(r.x, g.world.ground_y, false));
    stand(&mut g, r.x - 300.0);
    g.update(&none(), FRAME);
    assert_eq!(g.combat.zombies.list.len(), 1);
    assert!(g.miss.as_ref().unwrap().is_hiding());
    assert_eq!(miss_rect(&g), r);
}

#[test]
fn new_game_forgets_found_miss_and_weapons() {
    let mut g = quiet_game(2);
    find_miss(&mut g);
    let fresh = new_game(2);
    assert!(fresh.miss.as_ref().unwrap().is_hiding());
    assert!(fresh.weapons.dropped.is_empty());
    assert_eq!(fresh.weapons.held(), Weapon::Hands);
}

// ---- US-052 쏟아지는 무기 ----

#[test]
fn finding_miss_drops_one_weapon_at_her_spot() {
    for seed in 0..100 {
        let mut g = quiet_game(seed);
        let x = miss_rect(&g).center().x;
        find_miss(&mut g);
        assert_eq!(g.weapons.dropped.len(), 1, "seed {seed}");
        let dropped = &g.weapons.dropped[0];
        assert_ne!(dropped.kind, Weapon::Hands);
        assert_eq!(dropped.rect.center().x, x);
        assert!(dropped.is_falling() && dropped.rect.y < 0.0, "화면 위쪽에서 시작");
    }
}

/// 시드가 같은 무기 꾸러미에서 줍기를 되풀이한 무기 순서.
fn weapons_in_drop_order(seed: u32, drops: usize, w: &World) -> Vec<Weapon> {
    let mut weapons = logic::weapon::Weapons::new(seed);
    let mut order = Vec::new();
    for _ in 0..drops {
        weapons.drop_one(100.0, w);
        order.push(weapons.dropped[0].kind);
        // 플레이어가 떨어지는 무기를 바로 줍는다.
        weapons.update(0.0, weapons.dropped[0].rect);
    }
    order
}

#[test]
fn drops_follow_the_seed_and_blades_come_new_ones_first_and_gun_only_once() {
    let w = World::long_map(1);
    let mut first_kinds = Vec::new();
    for seed in 0..100 {
        let order = weapons_in_drop_order(seed, 12, &w);
        assert_eq!(order, weapons_in_drop_order(seed, 12, &w), "시드가 같으면 같다");
        let mut owned = Vec::new();
        for kind in &order {
            assert_ne!(*kind, Weapon::Hands);
            if *kind == Weapon::Gun {
                assert!(!owned.contains(&Weapon::Gun), "seed {seed}: 총을 가졌는데 또 나옴 {order:?}");
            } else if owned.contains(kind) {
                let other = if *kind == Weapon::Sword { Weapon::Axe } else { Weapon::Sword };
                assert!(owned.contains(&other), "seed {seed}: 없는 칼·도끼가 먼저다 {order:?}");
            }
            owned.push(*kind);
        }
        first_kinds.push(order[0]);
    }
    for kind in [Weapon::Sword, Weapon::Axe, Weapon::Gun] {
        assert!(first_kinds.contains(&kind), "첫 무기로 {kind:?}도 나온다");
    }
}

/// 시드마다 `owned`를 가진 뒤 한 번 떨어뜨려서 총이 나온 비율.
fn gun_share_with(owned: &[Weapon], w: &World) -> f32 {
    const SEEDS: u32 = 5000;
    let far = Rect::new(-999.0, -999.0, 1.0, 1.0);
    let mut guns = 0;
    for seed in 0..SEEDS {
        let mut weapons = logic::weapon::Weapons::new(seed);
        for kind in owned {
            let drop = DroppedWeapon::above(*kind, 100.0, w);
            let body = drop.rect;
            weapons.dropped.push(drop);
            weapons.update(0.0, body);
        }
        weapons.update(0.0, far);
        weapons.drop_one(100.0, w);
        guns += u32::from(weapons.dropped[0].kind == Weapon::Gun);
    }
    guns as f32 / SEEDS as f32
}

#[test]
fn gun_drops_about_10_percent_even_when_only_the_gun_is_missing() {
    let w = World::long_map(1);
    assert_eq!(GUN_DROP_CHANCE, 0.1);
    for owned in [&[][..], &[Weapon::Sword], &[Weapon::Sword, Weapon::Axe]] {
        let share = gun_share_with(owned, &w);
        assert!((0.085..0.115).contains(&share), "{owned:?}: 총 {share}");
    }
    assert_eq!(gun_share_with(&[Weapon::Gun], &w), 0.0, "이미 있으면 안 나온다");
}

#[test]
fn blade_lying_on_the_ground_is_not_dropped_twice() {
    let w = World::long_map(1);
    for seed in 0..200 {
        let mut weapons = logic::weapon::Weapons::new(seed);
        weapons.drop_one(100.0, &w);
        weapons.drop_one(200.0, &w);
        let (first, second) = (weapons.dropped[0].kind, weapons.dropped[1].kind);
        assert!(first != second || first == Weapon::Gun, "seed {seed}: {first:?}");
    }
}

#[test]
fn weapon_falls_at_300_and_lands_on_the_surface_on_every_spot_and_map() {
    for seed in 0..500 {
        let w = World::long_map(seed);
        for (_, x, _) in HIDE_SPOTS {
            let mut weapons = logic::weapon::Weapons::new(seed);
            weapons.drop_one(x, &w);
            let far = Rect::new(-999.0, -999.0, 1.0, 1.0);
            weapons.update(1.0, far);
            let one_second_down = -WEAPON_SIZE + WEAPON_FALL_SPEED;
            assert!(weapons.dropped.iter().all(|d| d.rect.y == one_second_down || !d.is_falling()), "1초에 300픽셀");
            weapons.update(3.0, far);
            for d in &weapons.dropped {
                assert!(!d.is_falling());
                assert_eq!(d.rect.y + d.rect.h, w.surface_y_under(d.rect), "seed {seed}: {:?}", d.rect);
                assert!(d.rect.x >= 0.0 && d.rect.x + d.rect.w <= w.width);
                assert!(!w.obstacles.iter().any(|b| logic::world::really_overlaps(*b, d.rect)), "seed {seed}");
            }
        }
    }
}

#[test]
fn fast_forward_does_not_speed_up_falling_weapons() {
    // 무기는 update_combat의 dt(진짜 시간)만 받는다. 시계만 돌려도 그대로다.
    let mut g = quiet_game(4);
    find_miss(&mut g);
    stand(&mut g, 40.0);
    let y = g.weapons.dropped[0].rect.y;
    g.clock.update(30.0);
    assert_eq!(g.weapons.dropped[0].rect.y, y);
}

#[test]
fn touching_a_weapon_picks_it_up_and_holds_it_even_in_the_air() {
    let mut g = quiet_game(4);
    drop_weapon(&mut g, Weapon::Sword, 300.0);
    drop_weapon(&mut g, Weapon::Gun, 500.0);
    let sword = g.weapons.dropped[0].rect;
    g.player.pos = sword.point();
    g.player.vel = Vec2::ZERO;
    let events = g.update(&none(), FRAME);
    assert!(events.contains(&GameEvent::WeaponPicked { kind: Weapon::Sword }), "{events:?}");
    assert_eq!(g.weapons.held(), Weapon::Sword);
    assert_eq!(g.weapons.owned(), [Weapon::Sword]);
    assert_eq!(g.weapons.dropped.len(), 1);
    assert_eq!(Weapon::Sword.picked_text(), "칼을 주웠어요!");
    assert_eq!(Weapon::Axe.picked_text(), "도끼를 주웠어요!");
    assert_eq!(Weapon::Gun.picked_text(), "총을 주웠어요!");
}

#[test]
fn weapon_does_not_block_and_monsters_leave_it_alone() {
    let mut g = quiet_game(4);
    find_miss(&mut g);
    stand(&mut g, 40.0);
    run(&mut g, &none(), 3.0);
    let x = g.weapons.dropped[0].rect.x;
    g.combat.zombies.list.push(Zombie::new(x, g.world.ground_y, false));
    stand(&mut g, x - 300.0);
    g.update(&none(), FRAME);
    assert_eq!(g.combat.zombies.list.len(), 1);
    assert_eq!(g.weapons.dropped.len(), 1);
}

#[test]
fn picking_up_a_weapon_you_have_only_removes_it() {
    let mut g = quiet_game(4);
    give(&mut g, &[Weapon::Axe]);
    assert_eq!(g.weapons.held(), Weapon::Axe);
    g.weapons.switch();
    assert_eq!(g.weapons.held(), Weapon::Hands);
    drop_weapon(&mut g, Weapon::Axe, 300.0);
    g.player.pos = g.weapons.dropped[0].rect.point();
    let events = g.update(&none(), FRAME);
    assert!(g.weapons.dropped.is_empty(), "땅에서 사라진다");
    assert!(!events.iter().any(|e| matches!(e, GameEvent::WeaponPicked { .. })));
    assert_eq!(g.weapons.owned(), [Weapon::Axe], "하나뿐이다");
    assert_eq!(g.weapons.held(), Weapon::Hands);
}

#[test]
fn unpicked_weapon_vanishes_15_seconds_after_landing() {
    let mut g = quiet_game(4);
    find_miss(&mut g);
    stand(&mut g, 40.0);
    let falling_seconds = (GROUND_Y + WEAPON_SIZE) / WEAPON_FALL_SPEED;
    run(&mut g, &none(), 0.5);
    assert_eq!(g.weapons.dropped[0].seconds_left(), WEAPON_GROUND_SECONDS, "떨어지는 동안은 줄지 않는다");
    run(&mut g, &none(), falling_seconds + 0.5);
    assert!(!g.weapons.dropped[0].is_falling());
    let left = g.weapons.dropped[0].seconds_left();
    assert!(left < WEAPON_GROUND_SECONDS);
    run(&mut g, &none(), left - 0.5);
    assert_eq!(g.weapons.dropped.len(), 1, "시간이 남으면 그대로 있다");
    let events = run(&mut g, &none(), 1.0);
    assert!(g.weapons.dropped.is_empty());
    assert!(!events.iter().any(|e| matches!(e, GameEvent::WeaponPicked { .. })), "주운 것이 아니다");
    assert!(g.weapons.owned().is_empty());
}

#[test]
fn clock_alone_does_not_shorten_a_weapons_time() {
    let mut g = quiet_game(4);
    drop_weapon(&mut g, Weapon::Sword, 300.0);
    stand(&mut g, 40.0);
    run(&mut g, &none(), 3.0);
    let left = g.weapons.dropped[0].seconds_left();
    g.clock.update(WEAPON_GROUND_SECONDS * 3.0);
    assert_eq!(g.weapons.dropped[0].seconds_left(), left);
}

#[test]
fn vanished_weapon_can_drop_again_and_owned_weapons_stay() {
    let w = World::long_map(1);
    let mut weapons = logic::weapon::Weapons::new(1);
    let far = Rect::new(-999.0, -999.0, 1.0, 1.0);
    weapons.drop_one(100.0, &w);
    weapons.drop_one(200.0, &w);
    let first = weapons.dropped[0].rect;
    let kept = weapons.dropped[0].kind;
    let vanishing = weapons.dropped[1].kind;
    weapons.update(0.0, first);
    assert_eq!(weapons.held(), kept);
    weapons.update(3.0, far);
    weapons.update(WEAPON_GROUND_SECONDS + 1.0, far);
    assert!(weapons.dropped.is_empty());
    assert_eq!((weapons.owned(), weapons.held()), (vec![kept], kept), "가진 무기와 손의 무기는 그대로다");

    let mut seen = Vec::new();
    for _ in 0..200 {
        weapons.drop_one(100.0, &w);
        seen.push(weapons.dropped[0].kind);
        weapons.dropped.clear();
    }
    assert!(seen.contains(&vanishing), "사라진 무기는 다시 나올 수 있다");
    assert!(!seen.contains(&kept), "새 무기가 있으면 가진 무기는 안 나온다");
}

// ---- US-053 무기로 공격 ----

fn give(g: &mut Game, kinds: &[Weapon]) {
    for kind in kinds {
        drop_weapon(g, *kind, 300.0);
        g.player.pos = g.weapons.dropped[0].rect.point();
        g.player.vel = Vec2::ZERO;
        g.update(&none(), FRAME);
    }
    stand(g, 300.0);
    g.player.facing = Facing::Right;
}

/// 미스를 찾지 않고 무기 하나를 하늘에서 떨어뜨린다.
fn drop_weapon(g: &mut Game, kind: Weapon, x: f32) {
    g.weapons.dropped.push(DroppedWeapon::above(kind, x, &g.world));
}

fn attack_input() -> PlayerInput {
    PlayerInput { attack_pressed: true, ..Default::default() }
}

/// 맞는 쪽 몬스터가 쓰러질 때까지 공격을 몇 번 눌렀는지.
fn hits_to_defeat(kind: Option<Weapon>, skeleton: Option<SkeletonStage>) -> usize {
    let mut g = quiet_game(4);
    g.combat.skeleton_chance = 0.0;
    if let Some(kind) = kind {
        give(&mut g, &[kind]);
    }
    stand(&mut g, 300.0);
    let x = g.player.pos.x + PLAYER_WIDTH + 10.0;
    match skeleton {
        Some(stage) => g.combat.skeletons.list.push(Skeleton::new(x, g.world.ground_y, stage)),
        None => g.combat.zombies.list.push(Zombie::new(x, g.world.ground_y, false)),
    }
    for hits in 1..=5 {
        g.update(&attack_input(), 0.0);
        if g.combat.zombies.list.is_empty() && g.combat.skeletons.list.is_empty() {
            return hits;
        }
    }
    panic!("5번 안에 안 쓰러짐");
}

#[test]
fn every_weapon_takes_the_right_number_of_hits_on_every_monster() {
    let table = [(None, 3), (Some(Weapon::Sword), 2), (Some(Weapon::Axe), 1), (Some(Weapon::Gun), 1)];
    for (weapon, hits) in table {
        assert_eq!(hits_to_defeat(weapon, None), hits, "좀비, {weapon:?}");
        for stage in [SkeletonStage::Normal, SkeletonStage::Half, SkeletonStage::Full] {
            assert_eq!(hits_to_defeat(weapon, Some(stage)), hits, "{stage:?}, {weapon:?}");
        }
    }
}

#[test]
fn sword_goes_3_to_1_to_down_and_gives_one_experience() {
    let mut g = quiet_game(4);
    give(&mut g, &[Weapon::Sword]);
    let x = g.player.pos.x + PLAYER_WIDTH + 10.0;
    g.combat.zombies.list.push(Zombie::new(x, g.world.ground_y, false));
    let first = g.update(&attack_input(), 0.0);
    assert!(first.iter().any(|e| matches!(e, GameEvent::Combat(CombatEvent::ZombieHit { hp_left: 1, .. }))));
    assert!(g.combat.zombies.list[0].is_flashing());
    let second = g.update(&attack_input(), 0.0);
    assert!(second.iter().any(|e| matches!(e, GameEvent::Combat(CombatEvent::ZombieDefeated { .. }))));
    assert_eq!(g.combat.experience, EXPERIENCE_PER_DEFEAT);
}

#[test]
fn gun_reaches_300_in_front_only_and_sword_axe_only_60() {
    let distance_cases = [
        (Weapon::Gun, GUN_RANGE - 1.0, true),
        (Weapon::Gun, GUN_RANGE + 1.0, false),
        (Weapon::Sword, PLAYER_ATTACK_RANGE - 1.0, true),
        (Weapon::Sword, PLAYER_ATTACK_RANGE + 1.0, false),
        (Weapon::Axe, PLAYER_ATTACK_RANGE - 1.0, true),
        (Weapon::Axe, PLAYER_ATTACK_RANGE + 1.0, false),
    ];
    for (kind, distance, should_hit) in distance_cases {
        let mut g = quiet_game(4);
        give(&mut g, &[kind]);
        let x = g.player.pos.x + PLAYER_WIDTH + distance;
        g.combat.zombies.list.push(Zombie::new(x, g.world.ground_y, false));
        let hit = g.update(&attack_input(), 0.0).iter().any(|e| matches!(e, GameEvent::Combat(_)));
        assert_eq!(hit, should_hit, "{kind:?} {distance}");
    }

    let mut g = quiet_game(4);
    give(&mut g, &[Weapon::Gun]);
    let behind = g.player.pos.x - 100.0;
    g.combat.zombies.list.push(Zombie::new(behind, g.world.ground_y, false));
    assert!(g.update(&attack_input(), 0.0).is_empty(), "등 뒤는 맞지 않는다");
}

#[test]
fn switch_cycles_hands_sword_axe_gun_skipping_missing() {
    let mut g = quiet_game(4);
    g.switch_weapon();
    assert_eq!(g.weapons.held(), Weapon::Hands, "무기가 없으면 맨손 그대로");
    assert!(g.weapons.owned().is_empty());

    give(&mut g, &[Weapon::Gun, Weapon::Sword]);
    assert_eq!(g.weapons.held(), Weapon::Sword, "주운 무기를 바로 든다");
    assert_eq!(g.weapons.owned(), [Weapon::Sword, Weapon::Gun]);
    let order: Vec<Weapon> = (0..4)
        .map(|_| {
            g.switch_weapon();
            g.weapons.held()
        })
        .collect();
    assert_eq!(order, [Weapon::Gun, Weapon::Hands, Weapon::Sword, Weapon::Gun], "도끼는 건너뛴다");
}

#[test]
fn weapons_stay_after_morning() {
    let mut g = quiet_game(4);
    give(&mut g, &[Weapon::Axe]);
    go_to_next_morning(&mut g);
    assert_eq!(g.weapons.held(), Weapon::Axe);
}

// ---- US-054 폭발형 시민 ----

fn citizen_game() -> Game {
    let mut g = new_game(4);
    let boxes = g.world.boxes();
    g.world.obstacles.retain(|b| !boxes.contains(b));
    g.coins.list.clear();
    g.miss = None;
    g
}

fn popped(events: &[GameEvent]) -> usize {
    events.iter().filter(|e| matches!(e, GameEvent::CitizenPopped { .. })).count()
}

fn step_onto_citizen(g: &mut Game, index: usize) -> Vec<GameEvent> {
    g.player.pos.x = g.citizens.list[index].rect.center().x - PLAYER_WIDTH / 2.0;
    g.player.pos.y = g.world.ground_y - PLAYER_HEIGHT;
    g.update(&none(), FRAME)
}

#[test]
fn two_citizens_stand_in_the_volcano_and_the_house() {
    let g = citizen_game();
    let xs: Vec<f32> = g.citizens.list.iter().map(|c| c.rect.center().x).collect();
    assert_eq!(xs, [1500.0, 3200.0]);
    let places: Vec<&str> = xs.iter().map(|x| g.world.place_name_at(*x)).collect();
    assert_eq!(places, ["화산", "집"]);
    for c in &g.citizens.list {
        assert!(c.is_standing());
        assert_eq!(c.rect.y + c.rect.h, g.world.ground_y);
    }
}

#[test]
fn walking_into_a_citizen_pops_and_hurts_by_two() {
    let mut g = citizen_game();
    let events = step_onto_citizen(&mut g, 0);
    assert_eq!(popped(&events), 1, "{events:?}");
    assert!(events.contains(&GameEvent::Combat(CombatEvent::PlayerHurt { amount: 2, hp_left: PLAYER_START_HP - 2 })));
    assert!(!g.citizens.list[0].is_standing());
    assert!(g.combat.health.is_invincible(), "맞은 뒤 1초 규칙이 같다");
}

#[test]
fn landing_on_a_citizen_from_a_jump_pops_too() {
    let mut g = citizen_game();
    let body = g.citizens.list[0].rect;
    g.player.pos = Vec2::new(body.x, body.y - PLAYER_HEIGHT - 60.0);
    let events = run(&mut g, &none(), 1.0);
    assert_eq!(popped(&events), 1);
}

#[test]
fn monster_touch_pops_and_nearby_player_is_hurt_but_far_player_is_not() {
    for (distance, hurt) in [(CITIZEN_BLAST_RANGE - 1.0, true), (CITIZEN_BLAST_RANGE + 1.0, false)] {
        let mut g = citizen_game();
        let at = g.citizens.list[1].rect.center();
        stand(&mut g, at.x + distance - PLAYER_WIDTH / 2.0);
        // 플레이어 몸 가운데와 시민 몸 가운데의 높이 차이만큼 거리가 더 벌어지지 않게 높이를 맞춘다.
        g.player.pos.y = at.y - PLAYER_HEIGHT / 2.0;
        g.combat.zombies.list.push(Zombie::new(at.x - ZOMBIE_WIDTH / 2.0, g.world.ground_y, false));
        let happened = g.combat.citizen_blast(at, g.player.rect());
        assert_eq!(g.combat.health.hp() < PLAYER_START_HP, hurt, "거리 {distance}, {happened:?}");
    }
    for monster_is_zombie in [true, false] {
        let mut g = citizen_game();
        let x = g.citizens.list[1].rect.center().x - ZOMBIE_WIDTH / 2.0;
        stand(&mut g, x - 400.0);
        if monster_is_zombie {
            g.combat.zombies.list.push(Zombie::new(x, g.world.ground_y, false));
        } else {
            g.combat.skeletons.list.push(Skeleton::new(x, g.world.ground_y, SkeletonStage::Normal));
        }
        let events = g.update(&none(), FRAME);
        assert_eq!(popped(&events), 1, "{events:?}");
        assert_eq!(g.combat.health.hp(), PLAYER_START_HP);
    }
}

#[test]
fn floating_zombie_pops_only_when_its_body_touches() {
    let mut g = citizen_game();
    let body = g.citizens.list[1].rect;
    stand(&mut g, body.x - 400.0);
    let mut zombie = Zombie::new(body.x, g.world.ground_y, true);
    zombie.state = ZombieState::Floating;
    zombie.pos.y = body.y - ZOMBIE_HEIGHT - 20.0;
    g.combat.zombies.list = vec![zombie];
    assert_eq!(popped(&g.update(&none(), FRAME)), 0, "위로 떠 있으면 안 닿는다");
    g.combat.zombies.list[0].pos.y = body.y;
    assert_eq!(popped(&g.update(&none(), FRAME)), 1);
}

#[test]
fn arrows_do_not_touch_citizens() {
    let mut g = citizen_game();
    let body = g.citizens.list[0].rect;
    stand(&mut g, body.x - PLAYER_WIDTH - 20.0);
    let world = World { width: MAP_WIDTH, ground_y: GROUND_Y, obstacles: vec![] };
    g.combat.skeletons.projectiles.push(Projectile::new(body.x, 1.0, ProjectileKind::Arrow, 2, &world));
    g.clock.update(DAY_SECONDS);
    g.update(&none(), FRAME);
    assert!(g.citizens.list[0].is_standing());
}

/// 시민 앞 `distance`에서 바라보고 때린다.
fn attack_citizen_from(g: &mut Game, distance: f32) -> Vec<GameEvent> {
    let body = g.citizens.list[0].rect;
    stand(g, body.x - PLAYER_WIDTH - distance);
    g.player.facing = Facing::Right;
    g.update(&attack_input(), 0.0)
}

#[test]
fn attack_knocks_a_citizen_down_quietly_with_every_weapon() {
    for (weapon, distance) in [
        (None, PLAYER_ATTACK_RANGE - 1.0),
        (Some(Weapon::Sword), SWORD_RANGE - 1.0),
        (Some(Weapon::Axe), AXE_RANGE - 1.0),
        (Some(Weapon::Gun), GUN_RANGE - 1.0),
    ] {
        let mut g = citizen_game();
        if let Some(kind) = weapon {
            give(&mut g, &[kind]);
        }
        let at = g.citizens.list[0].rect.center();
        let events = attack_citizen_from(&mut g, distance);
        assert_eq!(events, [GameEvent::Combat(CombatEvent::CitizenDefeated { at })], "{weapon:?}");
        assert!(!g.citizens.list[0].is_standing());
        assert!(g.citizens.list[1].is_standing());
        assert_eq!(g.combat.health.hp(), PLAYER_START_HP, "주인공은 안 다친다");
        assert_eq!(g.combat.experience, 0, "경험치 없음");
    }
}

#[test]
fn attack_blast_does_not_defeat_monsters_next_to_the_citizen() {
    let mut g = citizen_game();
    let body = g.citizens.list[0].rect;
    g.combat.zombies.list.push(Zombie::new(body.x + 60.0, g.world.ground_y, false));
    attack_citizen_from(&mut g, 10.0);
    assert_eq!(g.combat.zombies.list.len(), 1, "터지지 않으니 곁의 몬스터도 그대로다");
}

#[test]
fn attack_does_not_reach_citizens_too_far_or_behind() {
    let mut g = citizen_game();
    attack_citizen_from(&mut g, PLAYER_ATTACK_RANGE + 1.0);
    assert!(g.citizens.list[0].is_standing(), "멀면 안 닿는다");

    g.player.facing = Facing::Left;
    let body = g.citizens.list[0].rect;
    stand(&mut g, body.x - PLAYER_WIDTH - 10.0);
    g.update(&attack_input(), 0.0);
    assert!(g.citizens.list[0].is_standing(), "등 뒤는 안 맞는다");
}

#[test]
fn citizen_knocked_down_by_attack_comes_back_after_30_seconds() {
    let mut g = citizen_game();
    attack_citizen_from(&mut g, 10.0);
    stand(&mut g, 40.0);
    run(&mut g, &none(), CITIZEN_RESPAWN_SECONDS - 0.5);
    assert!(!g.citizens.list[0].is_standing());
    run(&mut g, &none(), 1.0);
    assert!(g.citizens.list[0].is_standing());
}

#[test]
fn blast_defeats_near_monsters_of_any_hp_without_experience_and_leaves_far_ones() {
    let mut g = citizen_game();
    let at = g.citizens.list[1].rect.center();
    stand(&mut g, at.x - 400.0);
    let ground = g.world.ground_y;
    g.combat.zombies.list.push(Zombie::new(at.x - ZOMBIE_WIDTH / 2.0, ground, false));
    g.combat.skeletons.list.push(Skeleton::new(at.x + 60.0, ground, SkeletonStage::Full));
    let mut floating = Zombie::new(at.x - 90.0, ground, true);
    floating.state = ZombieState::Floating;
    g.combat.zombies.list.push(floating);
    g.combat.zombies.list.push(Zombie::new(at.x + CITIZEN_BLAST_RANGE + 40.0, ground, false));

    let events = g.update(&none(), FRAME);
    let down = |e: &&GameEvent| {
        matches!(e, GameEvent::Combat(CombatEvent::ZombieDefeated { .. } | CombatEvent::SkeletonDefeated { .. }))
    };
    assert_eq!(events.iter().filter(down).count(), 3, "{events:?}");
    assert_eq!((g.combat.zombies.list.len(), g.combat.skeletons.list.len()), (1, 0));
    assert_eq!((g.combat.experience, g.combat.level), (0, 1), "경험치 없음");
}

#[test]
fn blast_can_bring_the_player_down() {
    let mut g = citizen_game();
    wear_down_to_one_blast_left(&mut g);
    run(&mut g, &none(), PLAYER_HURT_INVINCIBLE_SECONDS + 0.1);
    let events = step_onto_citizen(&mut g, 0);
    assert_eq!(popped(&events), 1);
    assert!(g.combat.health.is_down(), "체력 7이 2씩 네 번이면 쓰러진다");
    assert!(events.contains(&GameEvent::Combat(CombatEvent::PlayerDown)));
}

/// 펑을 세 번 맞혀서 한 번만 더 맞으면 쓰러지게 한다.
fn wear_down_to_one_blast_left(g: &mut Game) {
    stand(g, 40.0);
    for _ in 0..3 {
        run(g, &none(), PLAYER_HURT_INVINCIBLE_SECONDS + 0.1);
        let body = g.player.rect();
        g.combat.citizen_blast(body.center(), body);
    }
    assert_eq!(g.combat.health.hp(), 1);
}

#[test]
fn blast_shares_the_5_second_safety_with_every_other_hit() {
    let mut g = citizen_game();
    stand(&mut g, 40.0);
    let body = g.player.rect();
    g.combat.citizen_blast(body.center(), body);
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP - 2);
    run(&mut g, &none(), PLAYER_HURT_INVINCIBLE_SECONDS - 0.5);
    g.combat.citizen_blast(body.center(), body);
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP - 2, "5초 안에는 안 다친다");
    run(&mut g, &none(), 1.0);
    g.combat.citizen_blast(body.center(), body);
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP - 4);
}

#[test]
fn citizen_comes_back_after_30_real_seconds_in_the_same_spot() {
    let mut g = citizen_game();
    let body = g.citizens.list[0].rect;
    step_onto_citizen(&mut g, 0);
    stand(&mut g, 40.0);
    run(&mut g, &none(), CITIZEN_RESPAWN_SECONDS - 0.5);
    assert!(!g.citizens.list[0].is_standing());
    run(&mut g, &none(), 1.0);
    assert!(g.citizens.list[0].is_standing());
    assert_eq!(g.citizens.list[0].rect, body);
}

#[test]
fn clock_alone_does_not_bring_citizen_back() {
    // 빨리감기는 시계·코인만 빠르게 한다. 시계를 돌려도 시민의 30초는 그대로다.
    let mut g = citizen_game();
    step_onto_citizen(&mut g, 0);
    stand(&mut g, 40.0);
    g.clock.update(CITIZEN_RESPAWN_SECONDS * 2.0);
    g.update(&none(), FRAME);
    assert!(!g.citizens.list[0].is_standing());
}

#[test]
fn citizen_waits_until_the_spot_is_clear_and_does_not_pop_on_return() {
    let mut g = citizen_game();
    step_onto_citizen(&mut g, 0);
    let events = run(&mut g, &none(), CITIZEN_RESPAWN_SECONDS + 2.0);
    assert!(!g.citizens.list[0].is_standing(), "플레이어가 서 있으면 기다린다");
    assert_eq!(popped(&events), 0);
    stand(&mut g, 40.0);
    let events = run(&mut g, &none(), 0.2);
    assert!(g.citizens.list[0].is_standing());
    assert_eq!(popped(&events), 0, "생기자마자 터지지 않는다");
}

#[test]
fn down_screen_stops_the_citizen_timer() {
    let mut g = citizen_game();
    step_onto_citizen(&mut g, 0);
    stand(&mut g, 40.0);
    g.clock.update(DAY_SECONDS);
    while_player_goes_down(&mut g);
    run(&mut g, &none(), CITIZEN_RESPAWN_SECONDS + 5.0);
    assert!(!g.citizens.list[0].is_standing());
}

fn while_player_goes_down(g: &mut Game) {
    // 체력 7은 네 번 맞아야 하고, 맞을 때마다 5초씩 안 맞는다.
    for _ in 0..(PLAYER_HURT_INVINCIBLE_SECONDS * 5.0 / FRAME) as usize {
        if g.combat.health.is_down() {
            return;
        }
        g.combat.zombies.list = vec![Zombie::new(g.player.pos.x, g.world.ground_y, false)];
        g.update(&none(), FRAME);
    }
    panic!("쓰러지지 않음");
}

// ---- 아이가 정한 값 ----

#[test]
fn gun_is_strongest_then_axe_then_sword() {
    assert!(Weapon::Gun.attack() > Weapon::Axe.attack());
    assert!(Weapon::Axe.attack() > Weapon::Sword.attack());
    assert_eq!(Weapon::Gun.range(), 300.0);
    assert_eq!(MISS_FOUND_TEXT, "아이템을 줄게!");
}

// ---- 일반 시민 ----

fn walker_snapshot(g: &Game) -> Vec<(Rect, Facing)> {
    g.citizens.walkers.iter().map(|w| (w.rect, w.facing)).collect()
}

#[test]
fn one_walking_citizen_per_place_only_on_the_long_map() {
    for seed in 0..500 {
        let g = new_game(seed);
        assert_eq!(g.citizens.walkers.len(), PLACE_NAMES.len());
        for (place, walker) in g.citizens.walkers.iter().enumerate() {
            assert_eq!(g.world.place_index_at(walker.rect.center().x), place, "seed {seed}");
            assert_eq!(walker.rect.y + walker.rect.h, g.world.ground_y);
        }
    }
    let m1 = Game::on_world(CharacterKind::Niro, 1, World::m1());
    assert!(m1.citizens.walkers.is_empty());
}

#[test]
fn same_seed_same_walk_and_other_seeds_differ() {
    let mut a = new_game(11);
    let mut b = new_game(11);
    for _ in 0..600 {
        a.update(&none(), FRAME);
        b.update(&none(), FRAME);
    }
    assert_eq!(walker_snapshot(&a), walker_snapshot(&b));
    let walks: Vec<Vec<(Rect, Facing)>> = (0..8)
        .map(|seed| {
            let mut g = new_game(seed);
            run(&mut g, &none(), 5.0);
            walker_snapshot(&g)
        })
        .collect();
    assert!(walks.iter().any(|walk| *walk != walks[0]), "시드가 다르면 걸음도 다르다");
}

#[test]
fn walking_citizens_move_but_never_enter_walls_or_leave_their_place() {
    for seed in 0..40 {
        let mut g = new_game(seed);
        let start = walker_snapshot(&g);
        // 프레임 시간이 가장 길어도 뚫지 않는지 함께 본다.
        for dt in [FRAME, MAX_FRAME_SECONDS] {
            for _ in 0..(60.0 / dt) as usize {
                let world = &g.world;
                g.citizens.walk(dt, world);
                for (place, walker) in g.citizens.walkers.iter().enumerate() {
                    let r = walker.rect;
                    assert!(!g.world.monster_walls().any(|wall| logic::world::really_overlaps(wall, r)), "seed {seed}: {r:?}");
                    assert!(place as f32 * SCREEN_WIDTH <= r.x, "seed {seed}");
                    assert!(r.x + r.w <= (place + 1) as f32 * SCREEN_WIDTH, "seed {seed}");
                }
            }
        }
        assert_ne!(walker_snapshot(&g), start, "seed {seed}: 움직인다");
    }
}

#[test]
fn walking_citizen_turns_around_at_a_wall() {
    let mut g = quiet_game(1);
    let facing_before = g.citizens.walkers[3].facing;
    let mut turned = false;
    // 장소 하나를 다 걸어도 시간이 남는다: 960 / 40 = 24초.
    for _ in 0..(30.0 / FRAME) as usize {
        let world = &g.world;
        g.citizens.walk(FRAME, world);
        turned |= g.citizens.walkers[3].facing != facing_before;
    }
    assert!(turned);
}

#[test]
fn walking_citizen_touches_nobody_and_nothing_happens() {
    let mut g = citizen_game();
    g.combat.skeleton_chance = 0.0;
    g.citizens.list.clear();
    let body = g.citizens.walkers[0].rect;
    stand(&mut g, body.x);
    g.player.facing = Facing::Right;
    let x = g.player.pos.x;
    g.combat.zombies.list.push(Zombie::new(body.x + 10.0, g.world.ground_y, false));
    let zombie_x = g.combat.zombies.list[0].pos.x;
    let events = g.update(&none(), 0.0);
    assert!(events.is_empty(), "닿아도 아무 일이 없다: {events:?}");
    assert_eq!((g.player.pos.x, g.combat.zombies.list[0].pos.x), (x, zombie_x));
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP);

    g.combat.zombies.list.clear();
    let before = g.citizens.walkers.len();
    assert!(g.update(&attack_input(), 0.0).is_empty(), "공격에 맞지 않는다");
    assert_eq!(g.citizens.walkers.len(), before);
}

#[test]
fn walking_citizens_stop_on_the_down_screen() {
    let mut g = citizen_game();
    g.clock.update(DAY_SECONDS);
    stand(&mut g, 400.0);
    while_player_goes_down(&mut g);
    let frozen = walker_snapshot(&g);
    run(&mut g, &none(), 5.0);
    assert_eq!(walker_snapshot(&g), frozen);
}

#[test]
fn walking_citizens_start_at_the_free_spot_nearest_the_middle_of_their_place() {
    let mut started_in_the_middle = 0;
    for seed in 0..500 {
        let g = new_game(seed);
        for (place, walker) in g.citizens.walkers.iter().enumerate() {
            let r = walker.rect;
            let is_blocked = |x: f32| {
                let body = Rect { x, ..r };
                g.world.monster_walls().any(|wall| logic::world::really_overlaps(wall, body))
            };
            assert!(!is_blocked(r.x), "seed {seed}: {r:?}");
            let middle_x = place as f32 * SCREEN_WIDTH + (SCREEN_WIDTH - r.w) / 2.0;
            let distance = (r.x - middle_x).abs();
            started_in_the_middle += u32::from(distance == 0.0);
            // 찾는 칸이 8픽셀이라 그만큼은 더 멀 수 있다.
            let mut closer = 0.0;
            while closer + 8.0 < distance {
                assert!(is_blocked(middle_x - closer) && is_blocked(middle_x + closer), "seed {seed}: {r:?}");
                closer += 8.0;
            }
        }
    }
    assert!(started_in_the_middle > 1000, "가운데에 선 시민 {started_in_the_middle}/2000");
}

#[test]
fn walking_citizens_take_breaks_and_stand_still_during_them() {
    let mut g = quiet_game(3);
    g.citizens.list.clear();
    stand(&mut g, 40.0);
    let (mut walking_frames, mut pausing_frames) = (0, 0);
    for _ in 0..(40.0 / FRAME) as usize {
        let before: Vec<Rect> = g.citizens.walkers.iter().map(|w| w.rect).collect();
        g.update(&none(), FRAME);
        for (walker, old) in g.citizens.walkers.iter().zip(&before) {
            if walker.is_walking() {
                walking_frames += 1;
            } else {
                pausing_frames += 1;
                assert_eq!(walker.rect, *old, "멈춘 동안은 움직이지 않는다");
            }
        }
    }
    let share = pausing_frames as f32 / (walking_frames + pausing_frames) as f32;
    let expected = WALKING_CITIZEN_PAUSE_SECONDS / (WALKING_CITIZEN_PAUSE_SECONDS + WALKING_CITIZEN_WALK_SECONDS);
    assert!((share - expected).abs() < 0.1, "멈춘 비율 {share}");
}

#[test]
fn walking_citizens_are_not_walking_on_the_down_screen() {
    // 몬스터에게 쓰러질 때.
    let mut g = citizen_game();
    g.clock.update(DAY_SECONDS);
    stand(&mut g, 400.0);
    while_player_goes_down(&mut g);
    assert!(g.citizens.walkers.iter().all(|w| !w.is_walking()));

    // 펑으로 쓰러지는 바로 그 프레임에도.
    let mut g = citizen_game();
    wear_down_to_one_blast_left(&mut g);
    run(&mut g, &none(), PLAYER_HURT_INVINCIBLE_SECONDS + 0.1);
    step_onto_citizen(&mut g, 0);
    assert!(g.combat.health.is_down());
    assert!(g.citizens.walkers.iter().all(|w| !w.is_walking()));
}

// ---- BUG-019 달려서 시민 넘기 ----

/// 몸 앞쪽과 시민 몸 사이가 이만큼이 되면 뛴다. 평평한 땅에서 넘을 수 있는 폭은 약 29~46픽셀이다.
const TAKE_OFF_GAP: f32 = 40.0;

/// 지금 자리에서 방향키를 누른 채 달려서, 시민 앞에서 한 번 뛴다. 터지지 않고 넘으면 true.
fn run_and_jump_over(g: &mut Game, index: usize, going_right: bool) -> bool {
    let body = g.citizens.list[index].rect;
    let walk = PlayerInput { left: !going_right, right: going_right, ..Default::default() };
    for _ in 0..(3.0 / FRAME) as usize {
        let me = g.player.rect();
        let gap = if going_right { body.x - (me.x + me.w) } else { me.x - (body.x + body.w) };
        if gap < -(body.w + me.w) {
            return true;
        }
        let jump = g.player.on_ground && gap <= TAKE_OFF_GAP && gap > 0.0;
        let events = g.update(&PlayerInput { jump_pressed: jump, ..walk }, FRAME);
        if popped(&events) > 0 {
            return false;
        }
    }
    false
}

/// 낮의 새 게임. 시민 곁 상자는 진짜 맵 그대로다.
fn game_near_citizen(seed: u32, x: f32) -> Game {
    let mut g = new_game(seed);
    g.miss = None;
    stand(&mut g, x);
    g.update(&none(), FRAME);
    g
}

#[test]
fn holding_right_and_jumping_clears_every_citizen_on_many_maps() {
    for seed in 0..200 {
        for index in 0..CITIZEN_XS.len() {
            let body = new_game(seed).citizens.list[index].rect;
            let mut g = game_near_citizen(seed, body.x - PLAYER_WIDTH - 60.0);
            assert!(run_and_jump_over(&mut g, index, true), "seed {seed}: {index}번 시민을 못 넘음, {:?}", g.player.pos);
        }
    }
}

#[test]
fn holding_left_and_jumping_clears_every_citizen_on_many_maps() {
    for seed in 0..200 {
        for index in 0..CITIZEN_XS.len() {
            let probe = new_game(seed);
            let body = probe.citizens.list[index].rect;
            let mut g = game_near_citizen(seed, body.x + body.w + 52.0);
            assert!(run_and_jump_over(&mut g, index, false), "seed {seed}: {index}번 시민을 못 넘음, {:?}", g.player.pos);
        }
    }
}

#[test]
fn walking_off_a_box_there_is_room_to_jump_the_citizen() {
    let mut checked = 0;
    for seed in 0..200 {
        for index in 0..CITIZEN_XS.len() {
            let g = new_game(seed);
            let body = g.citizens.list[index].rect;
            let boxes = g.world.boxes();
            let Some(nearest) = boxes.iter().filter(|b| b.x < body.x && body.x - b.x < 400.0).max_by(|a, b| a.x.total_cmp(&b.x)) else {
                continue;
            };
            let mut g = game_near_citizen(seed, 0.0);
            g.player.pos = Vec2::new(nearest.x + nearest.w - PLAYER_WIDTH / 2.0 - 2.0, nearest.y - PLAYER_HEIGHT);
            g.update(&none(), FRAME);
            assert!(run_and_jump_over(&mut g, index, true), "seed {seed}: {index}번, 상자 {nearest:?}");
            checked += 1;
        }
    }
    assert!(checked > 100, "상자 뒤 시민 {checked}곳");
}

#[test]
fn jumping_a_box_then_the_citizen_works_from_most_take_off_spots() {
    // BUG-019 재현: 상자 앞 어디서 뛰어도 시민 위로 떨어지던 자리.
    let (mut cleared, mut tries) = (0, 0);
    for seed in 0..40 {
        for index in 0..CITIZEN_XS.len() {
            let g = new_game(seed);
            let body = g.citizens.list[index].rect;
            let boxes = g.world.boxes();
            let Some(nearest) = boxes.iter().filter(|b| b.x < body.x && body.x - b.x < 400.0).max_by(|a, b| a.x.total_cmp(&b.x)).copied() else {
                continue;
            };
            for before_box in (0..160).step_by(16) {
                let mut g = game_near_citizen(seed, nearest.x - PLAYER_WIDTH - before_box as f32 - 2.0);
                let walk = PlayerInput { right: true, ..Default::default() };
                g.update(&PlayerInput { jump_pressed: true, ..walk }, FRAME);
                // 상자에 막히거나 올라서면 다시 뛰지 않고 달린다.
                tries += 1;
                cleared += usize::from(run_and_jump_over(&mut g, index, true));
            }
        }
    }
    println!("상자 앞에서 뛰고 시민도 넘은 판 {cleared}/{tries}");
    assert!(tries > 100);
    assert!(cleared * 2 > tries, "{cleared}/{tries}");
}

// ---- 아이가 정한 문구 ----

#[test]
fn texts_for_the_screen() {
    assert_eq!(ZOMBIE_APPEAR_TEXT, "얍얍 공격하라");
    assert_eq!(SKELETON_DODGE_TEXT, "만세");
    assert_eq!(CITIZEN_HELP_TEXT, "도움이 필요해요");
}

// ---- REVIEW-011 ----

#[test]
fn returning_miss_never_lands_on_the_player() {
    let ground_spots: Vec<f32> = HIDE_SPOTS.iter().filter(|spot| spot.2 == 0.0).map(|spot| spot.1).collect();
    for seed in 0..300 {
        let mut g = quiet_game(seed);
        let found_x = miss_rect(&g).center().x;
        find_miss(&mut g);
        let wait_x = ground_spots.iter().copied().find(|x| *x != found_x).unwrap();
        stand(&mut g, wait_x - PLAYER_WIDTH / 2.0);
        let events = g.update(&none(), MISS_RETURN_SECONDS + 0.1);
        assert!(!events.iter().any(|e| matches!(e, GameEvent::MissFound { .. })), "seed {seed}: 보이기 전에 찾아짐");
        assert!(g.miss.as_ref().unwrap().is_hiding());
        assert_ne!(miss_rect(&g).center().x, wait_x, "seed {seed}");
        assert_ne!(miss_rect(&g).center().x, found_x, "seed {seed}");
    }
}

#[test]
fn miss_still_comes_back_when_every_spot_is_near_the_player() {
    let w = World::long_map(1);
    let everywhere = Rect::new(-100.0, -100.0, MAP_WIDTH + 200.0, 1000.0);
    let mut miss = logic::miss::Miss::new(1);
    let before = miss.rect(&w);
    miss.get_found();
    miss.wait_to_come_back(MISS_RETURN_SECONDS + 0.1, everywhere, &w);
    assert!(miss.is_hiding());
    assert_ne!(miss.rect(&w), before);
}

#[test]
fn hitting_the_empty_citizen_spot_does_not_restart_the_30_seconds() {
    let mut g = citizen_game();
    attack_citizen_from(&mut g, 10.0);
    assert!(!g.citizens.list[0].is_standing());
    run(&mut g, &none(), 10.0);
    assert!(attack_citizen_from(&mut g, 10.0).is_empty(), "쓰러진 시민은 또 안 맞는다");
    run(&mut g, &none(), CITIZEN_RESPAWN_SECONDS - 10.5);
    assert!(!g.citizens.list[0].is_standing());
    run(&mut g, &none(), 1.0);
    assert!(g.citizens.list[0].is_standing(), "처음 쓰러진 때부터 30초에 돌아온다");
}
