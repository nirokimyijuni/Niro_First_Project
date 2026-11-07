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
use logic::weapon::Weapon;
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
fn found_miss_moves_to_a_different_spot_next_morning_and_can_be_found_again() {
    for seed in 0..200 {
        let mut g = quiet_game(seed);
        let first = miss_rect(&g);
        find_miss(&mut g);
        go_to_next_morning(&mut g);
        assert!(g.miss.as_ref().unwrap().is_hiding(), "seed {seed}");
        assert_ne!(miss_rect(&g), first, "seed {seed}: 조금 전과 다른 곳");
        let events = find_miss(&mut g);
        assert!(events.iter().any(|e| matches!(e, GameEvent::MissFound { .. })));
    }
}

#[test]
fn found_miss_stays_gone_until_morning_even_after_night_comes() {
    let mut g = quiet_game(5);
    find_miss(&mut g);
    stand(&mut g, 40.0);
    g.clock.update(DAY_SECONDS);
    g.update(&none(), FRAME);
    assert!(g.clock.is_night());
    assert!(!g.miss.as_ref().unwrap().is_hiding());
}

#[test]
fn unfound_miss_stays_in_place_after_morning() {
    let mut g = quiet_game(5);
    let before = miss_rect(&g);
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
fn finding_miss_drops_sword_axe_gun_around_her() {
    let mut g = quiet_game(4);
    let x = miss_rect(&g).center().x;
    find_miss(&mut g);
    stand(&mut g, 40.0);
    let list: Vec<(Weapon, f32)> = g.weapons.dropped.iter().map(|w| (w.kind, w.rect.center().x)).collect();
    let expect_x = |dx: f32| (x + dx).clamp(WEAPON_SIZE / 2.0, MAP_WIDTH - WEAPON_SIZE / 2.0);
    assert_eq!(
        list,
        [
            (Weapon::Sword, expect_x(-WEAPON_DROP_SPREAD)),
            (Weapon::Axe, expect_x(0.0)),
            (Weapon::Gun, expect_x(WEAPON_DROP_SPREAD)),
        ]
    );
    assert!(g.weapons.dropped.iter().all(|w| w.is_falling() && w.rect.y < 0.0), "화면 위쪽에서 시작");
}

#[test]
fn weapon_falls_at_300_and_lands_on_the_surface_on_every_spot_and_map() {
    for seed in 0..500 {
        let w = World::long_map(seed);
        for (_, x, _) in HIDE_SPOTS {
            let mut weapons = logic::weapon::Weapons::new();
            weapons.drop_around(x, &w);
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
    find_miss(&mut g);
    let sword = g.weapons.dropped[0].rect;
    assert_eq!(g.weapons.dropped[0].kind, Weapon::Sword);
    g.player.pos = sword.point();
    g.player.vel = Vec2::ZERO;
    let events = g.update(&none(), FRAME);
    assert!(events.contains(&GameEvent::WeaponPicked { kind: Weapon::Sword }), "{events:?}");
    assert_eq!(g.weapons.held(), Weapon::Sword);
    assert_eq!(g.weapons.owned(), [Weapon::Sword]);
    assert_eq!(g.weapons.dropped.len(), 2);
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
    assert_eq!(g.weapons.dropped.len(), 3);
}

#[test]
fn picking_up_a_weapon_you_have_only_removes_it() {
    let mut g = quiet_game(4);
    find_miss(&mut g);
    let axe = g.weapons.dropped[1].rect;
    g.player.pos = axe.point();
    g.update(&none(), FRAME);
    assert_eq!(g.weapons.held(), Weapon::Axe);
    go_to_next_morning(&mut g);
    find_miss(&mut g);
    g.weapons.switch();
    assert_eq!(g.weapons.held(), Weapon::Hands);
    let axe = g.weapons.dropped.iter().rfind(|w| w.kind == Weapon::Axe).map(|w| w.rect).unwrap();
    let left = g.weapons.dropped.len();
    g.player.pos = axe.point();
    let events = g.update(&none(), FRAME);
    assert_eq!(g.weapons.dropped.len(), left - 1, "땅에서 사라진다");
    assert!(!events.iter().any(|e| matches!(e, GameEvent::WeaponPicked { .. })));
    assert_eq!(g.weapons.owned(), [Weapon::Axe], "하나뿐이다");
    assert_eq!(g.weapons.held(), Weapon::Hands);
}

#[test]
fn unpicked_weapons_stay_through_the_night() {
    let mut g = quiet_game(4);
    find_miss(&mut g);
    stand(&mut g, 40.0);
    run(&mut g, &none(), 3.0);
    go_to_next_morning(&mut g);
    assert_eq!(g.weapons.dropped.len(), 3);
}

// ---- US-053 무기로 공격 ----

fn give(g: &mut Game, kinds: &[Weapon]) {
    find_miss(g);
    for kind in kinds {
        let rect = g.weapons.dropped.iter().find(|d| d.kind == *kind).expect("떨어진 무기").rect;
        g.player.pos = rect.point();
        g.player.vel = Vec2::ZERO;
        g.update(&none(), FRAME);
    }
    g.weapons.dropped.clear();
    stand(g, 300.0);
    g.player.facing = Facing::Right;
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
fn one_citizen_per_place_standing_on_the_ground() {
    let g = citizen_game();
    let xs: Vec<f32> = g.citizens.list.iter().map(|c| c.rect.center().x).collect();
    assert_eq!(xs, [480.0, 1500.0, 2400.0, 3200.0]);
    for (i, c) in g.citizens.list.iter().enumerate() {
        assert!(c.is_standing());
        assert_eq!(c.rect.y + c.rect.h, g.world.ground_y);
        assert_eq!(g.world.place_index_at(c.rect.x), i);
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
    let body = g.citizens.list[2].rect;
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
fn arrows_and_attacks_do_not_pop_citizens() {
    let mut g = citizen_game();
    let body = g.citizens.list[0].rect;
    stand(&mut g, body.x - PLAYER_WIDTH - 20.0);
    g.update(&attack_input(), 0.0);
    assert!(g.citizens.list[0].is_standing());
    let world = World { width: MAP_WIDTH, ground_y: GROUND_Y, obstacles: vec![] };
    g.combat.skeletons.projectiles.push(Projectile::new(body.x, 1.0, ProjectileKind::Arrow, 2, &world));
    g.clock.update(DAY_SECONDS);
    g.update(&none(), FRAME);
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
    g.combat.health = logic::combat::PlayerHealth::new();
    let mut last = Vec::new();
    for round in 0..4 {
        for _ in 0..(PLAYER_HURT_INVINCIBLE_SECONDS / FRAME) as usize + 2 {
            g.update(&none(), FRAME);
        }
        last = step_onto_citizen(&mut g, round);
        assert_eq!(popped(&last), 1);
    }
    assert!(g.combat.health.is_down(), "체력 7이 2씩 네 번이면 쓰러진다");
    assert!(last.contains(&GameEvent::Combat(CombatEvent::PlayerDown)));
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
    for _ in 0..600 {
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
fn walking_citizens_start_near_the_middle_of_their_place() {
    for seed in 0..500 {
        let g = new_game(seed);
        for (place, walker) in g.citizens.walkers.iter().enumerate() {
            let middle = (place as f32 + 0.5) * SCREEN_WIDTH;
            assert!((walker.rect.center().x - middle).abs() <= 160.0, "seed {seed}: {:?}", walker.rect);
            assert!(!g.world.monster_walls().any(|wall| logic::world::really_overlaps(wall, walker.rect)));
        }
    }
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
    for round in 0..4 {
        run(&mut g, &none(), PLAYER_HURT_INVINCIBLE_SECONDS + 0.1);
        step_onto_citizen(&mut g, round);
    }
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
            // 산 시민 오른쪽은 바위 첫 칸이다. 그 위에서 뛴다.
            let step = probe.world.rock_steps()[0];
            let on_rock = step.x < body.x + 100.0 && body.x < step.x;
            let mut g = game_near_citizen(seed, if on_rock { step.x } else { body.x + body.w + 52.0 });
            if on_rock {
                g.player.pos = Vec2::new(step.x + 8.0, step.y - PLAYER_HEIGHT);
                g.update(&none(), FRAME);
            }
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
