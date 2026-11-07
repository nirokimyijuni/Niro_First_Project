//! M6 엘리 시험: 로건·수산나 시험에 없는 것을 처음부터 끝까지 게임으로 돌려 본다.
//! 재기는 시험은 `#[ignore]`다: `timeout 300 cargo test --test m6_ellie -- --ignored --nocapture`
//! 기준: stories/backlog.md US-045, US-050~054, US-057, 임시-9, 임시-10.

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
use logic::player::PlayerInput;
use logic::world::{shortest_dx, World};
use macroquad::math::{Rect, Vec2};

const FRAME: f32 = 1.0 / 60.0;

fn game(seed: u32) -> Game {
    Game::new(CharacterKind::Niro, seed)
}

fn stand(g: &mut Game, x: f32) {
    g.player.pos = Vec2::new(x, g.world.ground_y - PLAYER_HEIGHT);
    g.player.vel = Vec2::ZERO;
}

fn no_boxes(g: &mut Game) {
    let boxes = g.world.boxes();
    g.world.obstacles.retain(|b| !boxes.contains(b));
    g.coins.list.clear();
}

/// 달리다가 시민 왼쪽 `gap`픽셀 앞에서 뛴다. 시민에 닿으면 true.
fn jump_from(gap: f32) -> bool {
    let mut g = game(1);
    no_boxes(&mut g);
    g.citizens.walkers.clear();
    let left = g.citizens.list[0].rect.x;
    stand(&mut g, left - gap - PLAYER_WIDTH);
    g.update(&PlayerInput::default(), FRAME); // 처음 한 프레임은 땅에 서기 전이다
    let mut first = true;
    for _ in 0..120 {
        let input = PlayerInput { right: true, jump_pressed: first, ..Default::default() };
        first = false;
        if g.update(&input, FRAME).iter().any(|e| matches!(e, GameEvent::CitizenPopped { .. })) {
            return true;
        }
    }
    false
}

/// 폭발 시민을 뛰어넘을 수 있다 (점프 금지 규칙은 없다, D39).
#[test]
fn a_running_jump_can_clear_a_bomb_citizen() {
    assert!((0..200).any(|gap| !jump_from(gap as f32)));
}

/// 맵 폭만큼 달린다. 막히면 뛰고, `gap` 앞에 시민이 있으면 넘는다 (0이면 안 넘는다).
/// 돌려주는 값: (도착했는지, 초, 펑 수).
fn travel(g: &mut Game, to_right: bool, gap: f32, limit_seconds: f32) -> (bool, f32, u32) {
    let mut pops = 0;
    let mut walked = 0.0;
    for frame in 0..(limit_seconds / FRAME) as usize {
        let (px, pw) = (g.player.pos.x, g.player.size.x);
        // 상자 위에서는 더 일찍 뛴다: 그냥 걸어 내려오면 상자 옆 시민 위에 떨어진다.
        let raised = g.player.pos.y + PLAYER_HEIGHT < g.world.ground_y;
        let gap = if raised && gap > 0.0 { 110.0 } else { gap };
        let near_citizen = gap > 0.0
            && g.citizens.list.iter().any(|c| {
                let ahead = if to_right { c.rect.x - (px + pw) } else { px - (c.rect.x + c.rect.w) };
                c.is_standing() && (0.0..=gap).contains(&ahead)
            });
        let walk = PlayerInput { left: !to_right, right: to_right, ..Default::default() };
        let before = g.player.pos.x;
        let events = g.update(&PlayerInput { jump_pressed: near_citizen && g.player.on_ground, ..walk }, FRAME);
        pops += events.iter().filter(|e| matches!(e, GameEvent::CitizenPopped { .. })).count() as u32;
        if g.player.pos.x == before {
            g.update(&PlayerInput { jump_pressed: true, ..walk }, FRAME);
        }
        if g.combat.health.is_down() {
            return (false, frame as f32 * FRAME, pops);
        }
        let moved = shortest_dx(before, g.player.pos.x);
        walked += if to_right { moved } else { -moved };
        if walked >= MAP_WIDTH - PLAYER_WIDTH {
            return (true, frame as f32 * FRAME, pops);
        }
    }
    (false, limit_seconds, pops)
}

#[test]
#[ignore]
fn measure_jump_window_over_a_bomb_citizen() {
    let clear: Vec<u32> = (0..200).filter(|gap| !jump_from(*gap as f32)).collect();
    println!("시민을 넘는 뛰는 자리 (시민 앞 픽셀): {clear:?}");
    println!("넓이 {}px = {:.3}초 (걷는 속도 {PLAYER_WALK_SPEED})", clear.len(), clear.len() as f32 / PLAYER_WALK_SPEED);
}

#[test]
#[ignore]
fn measure_walking_straight_without_jumping_over_citizens() {
    let (mut down, mut reached, mut pops) = (0, 0, 0);
    let mut hp_left = [0u32; 8];
    for seed in 0..500 {
        let mut g = game(seed);
        let (ok, _, p) = travel(&mut g, true, 0.0, 60.0);
        down += g.combat.health.is_down() as u32;
        reached += ok as u32;
        pops += p;
        hp_left[g.combat.health.hp() as usize] += 1;
    }
    println!("낮 곧장 걷기 500판: 도착 {reached}, 쓰러짐 {down}, 펑 평균 {:.2}, 끝 체력 분포 {hp_left:?}", pops as f32 / 500.0);
}

#[test]
#[ignore]
fn measure_round_trip_day_and_night() {
    let gap = 38.0; // 뛰는 자리 가운데쯤 (measure_jump_window_over_a_bomb_citizen)
    for night in [false, true] {
        let (mut both, mut down, mut pops, mut hurt, mut secs) = (0, 0, 0, 0u32, 0.0);
        for seed in 0..500 {
            let mut g = game(seed);
            if night {
                g.clock.update(DAY_SECONDS);
            }
            let (a, ta, pa) = travel(&mut g, true, gap, 90.0);
            let (b, tb, pb) = if a { travel(&mut g, false, gap, 90.0) } else { (false, 0.0, 0) };
            both += (a && b) as u32;
            down += g.combat.health.is_down() as u32;
            pops += pa + pb;
            hurt += g.combat.health.max_hp() - g.combat.health.hp();
            secs += ta + tb;
        }
        println!(
            "{}: 왕복 {both}/500, 쓰러짐 {down}, 펑 평균 {:.2}, 잃은 체력 평균 {:.2}, 평균 {:.1}초",
            if night { "밤" } else { "낮" },
            pops as f32 / 500.0,
            hurt as f32 / 500.0,
            secs / 500.0
        );
    }
}

/// 진짜 시드는 밀리초 시각이라 이웃한 수다. 그런 시드에서도 숨는 곳이 고르게 나오는지.
#[test]
#[ignore]
fn measure_first_hide_spot_with_clock_like_seeds() {
    let mut count = [0u32; 6];
    let start = 1_800_000_000u32;
    for i in 0..6000 {
        let g = game(start.wrapping_add(i * 7));
        let x = g.miss.as_ref().unwrap().rect(&g.world).center().x;
        count[HIDE_SPOTS.iter().position(|s| s.1 == x).unwrap()] += 1;
    }
    println!("첫 숨는 곳 6000판: {count:?}");
}

/// US-057: 새 게임에 일반 시민은 자기 장소 가운데쯤에 있다.
#[test]
fn walkers_start_near_the_middle_of_their_place() {
    for seed in 0..500 {
        let g = game(seed);
        for (i, w) in g.citizens.walkers.iter().enumerate() {
            let middle = (i as f32 + 0.5) * SCREEN_WIDTH;
            assert!((w.rect.center().x - middle).abs() <= SCREEN_WIDTH / 4.0, "시드 {seed} 시민 {i}: {}", w.rect.center().x);
        }
    }
}

/// US-057: 시민은 걷다가 가끔 멈춘다.
#[test]
fn walkers_sometimes_stop() {
    let mut g = game(3);
    stand(&mut g, 40.0);
    let mut stopped = [false; 4];
    for _ in 0..(120.0 / FRAME) as usize {
        let before: Vec<f32> = g.citizens.walkers.iter().map(|w| w.rect.x).collect();
        let facing: Vec<_> = g.citizens.walkers.iter().map(|w| w.facing).collect();
        g.update(&PlayerInput::default(), FRAME);
        for (i, w) in g.citizens.walkers.iter().enumerate() {
            stopped[i] |= w.rect.x == before[i] && w.facing == facing[i];
        }
    }
    assert_eq!(stopped, [true; 4], "2분 동안 한 번은 서 있어야 한다");
}

/// US-045 D41: 다친 채로 진짜 공격으로 레벨 2가 되면 바로 12.
#[test]
fn level_up_by_real_attack_fills_health_at_once() {
    let mut g = game(5);
    no_boxes(&mut g);
    g.citizens.list.clear();
    g.clock.update(DAY_SECONDS);
    stand(&mut g, 600.0);
    let at = g.player.rect().center();
    g.combat.citizen_blast(at, g.player.rect());
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP - CITIZEN_BLAST_DAMAGE);
    g.combat.experience = EXPERIENCE_PER_LEVEL - 1;
    let mut zombie = Zombie::new(g.player.pos.x + 40.0, g.world.ground_y, false);
    zombie.hp = 1;
    g.combat.zombies.list = vec![zombie];
    g.update(&PlayerInput { attack_pressed: true, ..Default::default() }, FRAME);
    assert_eq!(g.combat.level, 2);
    assert_eq!((g.combat.health.hp(), g.combat.health.max_hp()), (12, 12));
}

#[test]
#[ignore]
fn measure_boxes_on_the_mountain() {
    use std::collections::BTreeMap;
    let mut count: BTreeMap<(i32, i32), u32> = BTreeMap::new();
    let mut near = [0u32; 4];
    for seed in 0..2000 {
        let w = World::long_map(seed);
        for b in w.boxes().iter().filter(|b| (1920.0..2880.0).contains(&b.x)) {
            *count.entry((b.x as i32, b.w as i32)).or_default() += 1;
        }
        for (i, x) in CITIZEN_XS.iter().enumerate() {
            let body = (x - CITIZEN_WIDTH / 2.0, x + CITIZEN_WIDTH / 2.0);
            if w.boxes().iter().any(|b| b.x < body.1 + NO_BOX_MARGIN && body.0 - NO_BOX_MARGIN < b.x + b.w) {
                near[i] += 1;
            }
        }
    }
    println!("산의 상자 (x, 폭): 2000판 {count:?}");
    println!("시민 몸 양옆 64픽셀 안에 상자가 있는 판 (해변, 화산, 산, 집): {near:?}");
}

/// 나무 발판에서 다음 발판으로 뛰는 자리 넓이. 오른쪽을 누른 채 뛰고, 다음 발판 위에 오면 손을 뗀다.
#[test]
#[ignore]
fn measure_tree_jump_windows() {
    use logic::player::Player;
    let w = World::long_map(1);
    let b = w.tree_branches();
    for i in 0..2 {
        let (from, to) = (b[i], b[i + 1]);
        let mut good = Vec::new();
        for center in (from.x as i32 - 20)..(from.x + from.w + 20.0) as i32 {
            let mut p = Player::new(CharacterKind::Niro, Vec2::new(center as f32 - PLAYER_WIDTH / 2.0, from.y - PLAYER_HEIGHT));
            p.update(&PlayerInput::default(), FRAME, &w);
            if !p.on_ground || p.pos.y + PLAYER_HEIGHT != from.y {
                continue;
            }
            let walk = PlayerInput { right: true, ..Default::default() };
            p.update(&PlayerInput { jump_pressed: true, ..walk }, FRAME, &w);
            for _ in 0..120 {
                let c = p.rect().center().x;
                let over = to.x + 8.0 < c && c < to.x + to.w - 8.0;
                p.update(&if over { PlayerInput::default() } else { walk }, FRAME, &w);
            }
            if p.on_ground && p.pos.y + PLAYER_HEIGHT == to.y {
                good.push(center);
            }
        }
        println!("{}칸 → {}칸: 뛰는 자리(몸 가운데) {:?}, 발판 {}~{}", i + 1, i + 2, (good.first(), good.last(), good.len()), from.x, from.x + from.w);
    }
}

/// 고치지 않은 빌드를 브라우저에서 볼 때 쓸 시드: 미스가 큰 돌 뒤에 있고 해변 상자가 적은 판.
#[test]
#[ignore]
fn measure_seed_for_browser() {
    let spot = |s: u32| {
        let g = game(s);
        g.miss.as_ref().unwrap().rect(&g.world).center().x
    };
    let big_rock_x = HIDE_SPOTS[0].1;
    let good: Vec<u32> = (1000..200_000)
        .filter(|s| spot(*s) == big_rock_x && spot(*s - 1) == big_rock_x && game(*s).world.boxes().iter().filter(|b| b.x < 800.0).count() <= 2)
        .take(3)
        .collect();
    for s in good {
        let boxes: Vec<(i32, i32)> = game(s).world.boxes().iter().filter(|b| b.x < 960.0).map(|b| (b.x as i32, b.w as i32)).collect();
        println!("시드 {s} 해변 상자 {boxes:?} 앞 시드 상자 {:?}", game(s - 1).world.boxes().iter().filter(|b| b.x < 960.0).map(|b| b.x as i32).collect::<Vec<_>>());
    }
}

/// 체력 7로 시민 네 명을 지나가면 몇 초에 어디서 쓰러지는지.
#[test]
#[ignore]
fn measure_where_walking_straight_goes_down() {
    let mut at = std::collections::BTreeMap::new();
    let mut secs = 0.0;
    for seed in 0..500 {
        let mut g = game(seed);
        let (_, t, _) = travel(&mut g, true, 0.0, 60.0);
        secs += t;
        *at.entry((g.player.pos.x / 100.0) as i32 * 100).or_insert(0) += 1;
    }
    println!("곧장 걷다 쓰러진 자리: {at:?}, 평균 {:.1}초", secs / 500.0);
}

/// 빨리감기처럼 한 프레임: 시계와 코인만 10배. main.rs `update_with_time_scale`과 같은 순서다.
fn fast_frame(g: &mut Game, input: &PlayerInput) -> Vec<GameEvent> {
    let mut events = Vec::new();
    if g.combat.health.is_down() {
        return events;
    }
    g.player.update(input, FRAME, &g.world);
    g.clock.update(FRAME * DEV_FAST_FORWARD_SCALE);
    g.update_combat(input, FRAME, &mut events);
    events
}

/// D44: 시작부터 빨리감기로 30초에 밤, 6초마다 2마리, 모두 8마리·좀비 5마리까지, 90초에 모두 사라진다.
/// 바위 꼭대기는 몬스터가 안 닿아서 체력을 건드리지 않고 잴 수 있다. 시민은 치운다 (펑으로 수가 줄지 않게).
#[test]
fn d44_monsters_arrive_two_each_turn_up_to_the_limit() {
    for seed in 0..50 {
        let mut g = game(seed);
        g.citizens.list.clear();
        g.player.pos = Vec2::new(2576.0, g.world.ground_y - CLIMB_STEP * 3.0 - PLAYER_HEIGHT);
        let count = |g: &Game| g.combat.zombies.list.len() + g.combat.skeletons.list.len();
        let mut rounds: Vec<(f32, u32)> = Vec::new();
        let (mut night_at, mut day_at, mut most, mut most_zombies) = (None, None, 0, 0);
        for frame in 0..(95.0 / FRAME) as usize {
            let t = frame as f32 * FRAME;
            let events = fast_frame(&mut g, &PlayerInput::default());
            let appeared = events
                .iter()
                .filter(|e| matches!(e, GameEvent::Combat(CombatEvent::ZombieAppeared { .. } | CombatEvent::SkeletonAppeared { .. })))
                .count() as u32;
            if appeared > 0 {
                rounds.push((t, appeared));
            }
            if g.clock.is_night() && night_at.is_none() {
                night_at = Some(t);
            }
            if night_at.is_some() && !g.clock.is_night() && day_at.is_none() {
                day_at = Some(t);
                assert_eq!(count(&g), 0, "시드 {seed} 아침에 남음");
            }
            most = most.max(count(&g));
            most_zombies = most_zombies.max(g.combat.zombies.list.len());
        }
        let night = night_at.unwrap();
        assert!((night - 30.0).abs() < 0.2, "시드 {seed} 밤 {night}");
        assert!((day_at.unwrap() - 90.0).abs() < 0.2, "시드 {seed} 아침 {day_at:?}");
        assert!(g.combat.health.hp() == g.combat.health.max_hp(), "시드 {seed}: 꼭대기에서 맞음");
        assert!(most <= MONSTER_MAX_ON_SCREEN && most_zombies <= ZOMBIE_MAX_ON_SCREEN, "시드 {seed}: {most}, {most_zombies}");
        for (i, (t, n)) in rounds.iter().take(MONSTER_MAX_ON_SCREEN / MONSTERS_PER_SPAWN).enumerate() {
            assert_eq!(*n, MONSTERS_PER_SPAWN as u32, "시드 {seed}: {rounds:?}");
            assert!((t - night - i as f32 * ZOMBIE_SPAWN_SECONDS).abs() < 0.2, "시드 {seed}: {rounds:?}");
        }
    }
}

/// 몬스터가 많은 밤을 체력 그대로 돌려 본다. 빨리감기 60초 밤.
#[test]
#[ignore]
fn measure_d44_night() {
    let seeds = 300;
    for (name, x, fight, gun) in [
        ("x=120 서 있기", 120.0, false, false),
        ("x=1100 서 있기", 1100.0, false, false),
        ("왼쪽 끝 서 있기", 0.0, false, false),
        ("x=120 맨손 싸우기", 120.0, true, false),
        ("x=1100 맨손 싸우기", 1100.0, true, false),
        ("x=2300 맨손 싸우기", 2300.0, true, false),
        ("x=120 총 싸우기", 120.0, true, true),
        ("x=1100 총 싸우기", 1100.0, true, true),
    ] {
        let (mut alive, mut down_at, mut level, mut hurt_free) = (0, 0.0, 0, 0);
        for seed in 0..seeds {
            let mut g = game(seed);
            g.clock.update(DAY_SECONDS);
            stand(&mut g, x);
            g.update(&PlayerInput::default(), FRAME);
            if gun {
                let gun_weapon = logic::weapon::DroppedWeapon::above(logic::weapon::Weapon::Gun, x, &g.world);
                g.weapons.dropped.push(gun_weapon);
                g.weapons.update(5.0, Rect::new(0.0, -100.0, MAP_WIDTH, 700.0));
            }
            let reach = if gun { GUN_RANGE } else { 75.0 };
            let mut t = 0.0;
            while t < 60.0 && !g.combat.health.is_down() {
                let center = g.player.rect().center().x;
                let bodies = g.combat.zombies.list.iter().map(|z| z.rect()).chain(g.combat.skeletons.list.iter().map(|s| s.rect()));
                let near = bodies.map(|b| b.center().x - center).min_by(|a, b| a.abs().total_cmp(&b.abs()));
                let input = match near {
                    Some(d) if fight && d.abs() < reach => PlayerInput { left: d < 0.0, right: d > 0.0, attack_pressed: (t * 60.0) as u32 % 20 == 0, ..Default::default() },
                    _ => PlayerInput::default(),
                };
                fast_frame(&mut g, &input);
                t += FRAME;
            }
            if g.combat.health.is_down() {
                down_at += t;
            } else {
                alive += 1;
            }
            hurt_free += (g.combat.level == 1 && g.combat.health.hp() == PLAYER_START_HP) as u32;
            level += g.combat.level;
        }
        println!("{name}: 밤 버팀 {alive}/{seeds}, 쓰러진 평균 {:.1}초, 안 다침 {hurt_free}, 평균 레벨 {:.2}", down_at / (seeds - alive).max(1) as f32, level as f32 / seeds as f32);
    }
}

/// D61: 옛 맵 왼쪽 끝(이음매)에 서 있어도 맵 끝 자리에서 나오지 않고, 양쪽 자리가 있으면 1마리씩 나온다.
/// 옛 D44 "반대쪽" 규칙은 M1 맵에만 있다.
#[test]
fn d61_standing_on_the_seam_gives_one_monster_on_each_open_side() {
    for seed in 0..200 {
        let mut g = game(seed);
        g.clock.update(DAY_SECONDS);
        stand(&mut g, 0.0);
        let events = g.update(&PlayerInput::default(), FRAME);
        let appeared: Vec<f32> = events
            .iter()
            .filter_map(|e| match e {
                GameEvent::Combat(CombatEvent::ZombieAppeared { at } | CombatEvent::SkeletonAppeared { at, .. }) => Some(at.x),
                _ => None,
            })
            .collect();
        assert!(appeared.len() <= MONSTERS_PER_SPAWN);
        let (body, center) = (g.player.rect(), g.player.rect().center().x);
        let open = |left: bool| {
            let spot = logic::monster::spawn_x_on_side(left, body, ZOMBIE_WIDTH, &g.world, &[]);
            spot.is_some_and(|x| (shortest_dx(center, x) < 0.0) == left)
        };
        if open(true) && open(false) {
            assert_eq!(appeared.len(), MONSTERS_PER_SPAWN, "시드 {seed}: {appeared:?}");
            assert!(shortest_dx(center, appeared[0]) * shortest_dx(center, appeared[1]) < 0.0, "시드 {seed}: {appeared:?}");
        }
        assert!(appeared.iter().all(|x| shortest_dx(center, *x).abs() >= MONSTER_SPAWN_MIN_DISTANCE), "시드 {seed}: {appeared:?}");
    }
}

/// 빨리감기 48.5초에 몇 마리인지. 날아오른 좀비가 스스로 터지면 8마리가 안 될 수 있다.
#[test]
#[ignore]
fn measure_count_at_48_seconds() {
    let mut count = [0u32; 9];
    for seed in 0..500 {
        let mut g = game(seed);
        g.citizens.list.clear();
        g.player.pos = Vec2::new(2576.0, g.world.ground_y - CLIMB_STEP * 3.0 - PLAYER_HEIGHT);
        for _ in 0..(48.5 / FRAME) as usize {
            fast_frame(&mut g, &PlayerInput::default());
        }
        count[g.combat.zombies.list.len() + g.combat.skeletons.list.len()] += 1;
    }
    println!("48.5초 마릿수 분포 0~8: {count:?}");
}

/// 빨리감기 60초 밤에 말풍선이 몇 번 나오는지와 D46·D47을 넣은 밤 난이도. 일반 시민 말풍선은 화면에서 잰다.
#[test]
#[ignore]
fn measure_d45_night_and_bubbles() {
    let seeds = 300;
    for (name, x, reach, gun) in [("x=120 서 있기", 120.0, 0.0, false), ("x=1100 서 있기", 1100.0, 0.0, false), ("x=120 맨손", 120.0, 75.0, false), ("x=1100 맨손", 1100.0, 75.0, false), ("x=120 총", 120.0, GUN_RANGE, true), ("x=1100 총", 1100.0, GUN_RANGE, true)] {
        let (mut alive, mut down_at, mut level, mut zombies, mut dodges, mut hurts) = (0, 0.0, 0, 0, 0, 0);
        for seed in 0..seeds {
            let mut g = game(seed);
            g.clock.update(DAY_SECONDS);
            stand(&mut g, x);
            g.update(&PlayerInput::default(), FRAME);
            if gun {
                for _ in 0..3 {
                    g.weapons.drop_one(x, &g.world);
                }
                g.weapons.update(5.0, Rect::new(0.0, -100.0, MAP_WIDTH, 700.0));
            }
            let mut t = 0.0;
            while t < 60.0 && !g.combat.health.is_down() {
                let center = g.player.rect().center().x;
                let bodies = g.combat.zombies.list.iter().map(|z| z.rect()).chain(g.combat.skeletons.list.iter().map(|s| s.rect()));
                let near = bodies.map(|b| b.center().x - center).min_by(|a, b| a.abs().total_cmp(&b.abs()));
                let input = match near {
                    Some(d) if d.abs() < reach => PlayerInput { left: d < 0.0, right: d > 0.0, attack_pressed: (t * 60.0) as u32 % 20 == 0, ..Default::default() },
                    _ => PlayerInput::default(),
                };
                for e in fast_frame(&mut g, &input) {
                    match e {
                        GameEvent::Combat(CombatEvent::ZombieAppeared { .. }) => zombies += 1,
                        GameEvent::Combat(CombatEvent::SkeletonDodged { .. }) => dodges += 1,
                        GameEvent::Combat(CombatEvent::PlayerHurt { .. }) => hurts += 1,
                        _ => {}
                    }
                }
                t += FRAME;
            }
            if g.combat.health.is_down() {
                down_at += t;
            } else {
                alive += 1;
            }
            level += g.combat.level;
        }
        let n = seeds as f32;
        println!(
            "{name}: 버팀 {alive}/{seeds}, 쓰러진 평균 {:.1}초, 평균 레벨 {:.2}, 맞은 횟수 {:.1}, 얍얍 {:.1}번, 만세 {:.1}번",
            down_at / (seeds - alive).max(1) as f32, level as f32 / n, hurts as f32 / n, zombies as f32 / n, dodges as f32 / n
        );
    }
}

/// 빨리감기 밤에 "만세"가 화면 안에서 몇 번 나오는지. 몬스터는 시작 자리 맨손 로봇이 싸운다.
#[test]
#[ignore]
fn measure_cheer_on_screen() {
    for x in [120.0, 1100.0, 2300.0, 3000.0] {
        let mut counts = Vec::new();
        let mut busiest_second = 0;
        for seed in 0..200 {
            let mut g = game(seed);
            g.clock.update(DAY_SECONDS);
            stand(&mut g, x);
            g.update(&PlayerInput::default(), FRAME);
            let (mut t, mut seen, mut in_second, mut second) = (0.0, 0, 0, 0);
            while t < 60.0 && !g.combat.health.is_down() {
                let left = g.camera_left();
                for e in fast_frame(&mut g, &PlayerInput::default()) {
                    if let GameEvent::Combat(CombatEvent::SkeletonDodged { at }) = e {
                        if (left..left + SCREEN_WIDTH).contains(&at.x) {
                            seen += 1;
                            if t as i32 != second {
                                second = t as i32;
                                in_second = 0;
                            }
                            in_second += 1;
                            busiest_second = busiest_second.max(in_second);
                        }
                    }
                }
                t += FRAME;
            }
            counts.push(seen);
        }
        counts.sort();
        println!("x={x}: 화면 안 만세 중앙값 {}, 90% {}, 최대 {}, 1초에 가장 많이 {busiest_second}", counts[100], counts[180], counts[199]);
    }
}

/// D62: 맵 자리마다 몬스터가 나올 수 있는지, 화면 안에서 얼마나 가까이 나오는지 잰다.
#[test]
#[ignore]
fn measure_d62_spawn_spots() {
    use logic::monster::spawn_x_on_side;
    use logic::world::shortest_dx;
    let (mut spots, mut none, mut on_screen, mut one_side) = (0u32, 0u32, 0u32, 0u32);
    let mut nearest = f32::MAX;
    for seed in 0..100 {
        let w = World::long_map(seed);
        let mut x = 0.0;
        while x < MAP_WIDTH {
            let body = macroquad::math::Rect::new(x, w.ground_y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT);
            x += 40.0;
            if w.monster_walls().any(|wall| logic::world::really_overlaps(wall, body)) {
                continue;
            }
            spots += 1;
            let found: Vec<f32> = [true, false].iter().filter_map(|left| spawn_x_on_side(*left, body, ZOMBIE_WIDTH, &w, &[])).collect();
            if found.is_empty() {
                none += 1;
                continue;
            }
            one_side += (found.len() == 2 && found[0] == found[1]) as u32;
            for f in found {
                let d = shortest_dx(body.center().x, f + ZOMBIE_WIDTH / 2.0).abs();
                nearest = nearest.min(d);
                on_screen += (d < SCREEN_WIDTH / 2.0) as u32;
            }
        }
    }
    println!("땅 자리 {spots}: 아무도 못 나옴 {none}, 한쪽만 {one_side}, 화면 안에서 나온 경우 {on_screen}, 가장 가까이 {nearest:.0}");
}

/// US-073: 찾은 뒤 옆으로 비켰다가 진짜 걷기·점프로 돌아와 15초 안에 줍는다. 숨는 곳마다 시드 500개.
#[test]
fn every_hide_spot_weapon_can_be_walked_to_and_picked_up() {
    let mut done = [0u32; 6];
    let mut slowest = [0.0f32; 6];
    let mut caught_falling = [0u32; 6];
    let mut seed = 0;
    while done.iter().any(|n| *n < 500) && seed < 12_000 {
        seed += 1;
        let mut g = game(seed);
        g.citizens.list.clear();
        let spot_rect = g.miss.as_ref().unwrap().rect(&g.world);
        let spot = HIDE_SPOTS.iter().position(|s| s.1 == spot_rect.center().x).unwrap();
        if done[spot] >= 500 {
            continue;
        }
        done[spot] += 1;
        g.player.pos = Vec2::new(spot_rect.x, spot_rect.y + spot_rect.h - PLAYER_HEIGHT);
        g.player.vel = Vec2::ZERO;
        g.update(&PlayerInput::default(), FRAME);
        assert!(!g.miss.as_ref().unwrap().is_hiding(), "시드 {seed}: 못 찾음");
        let (mut t, mut landed, mut step_off) = (0.0, None, false);
        while g.weapons.owned().is_empty() {
            let weapon = g.weapons.dropped.first().map(|w| (w.rect, w.is_falling()));
            let Some((rect, falling)) = weapon else { panic!("시드 {seed} {}: 무기가 {t:.1}초에 사라짐, 자리 {:?}", HIDE_SPOTS[spot].0, g.player.pos) };
            if !falling && landed.is_none() {
                landed = Some(t);
            }
            let away = t < 1.0;
            let dx = shortest_dx(g.player.rect().center().x, rect.center().x);
            let above = g.player.pos.y + PLAYER_HEIGHT < rect.y + rect.h - 40.0;
            step_off = (step_off || (above && dx.abs() < 40.0 && g.player.on_ground)) && above;
            let right = away || step_off || dx > 2.0;
            let left = !right && dx < -2.0;
            let before = g.player.pos.x;
            let walk = PlayerInput { left, right, ..Default::default() };
            let was_empty = g.weapons.owned().is_empty();
            g.update(&walk, FRAME);
            if g.player.pos.x == before && (left || right) {
                g.update(&PlayerInput { jump_pressed: true, ..walk }, FRAME);
            }
            t += FRAME;
            if was_empty && !g.weapons.owned().is_empty() && landed.is_none() {
                caught_falling[spot] += 1;
            }
            assert!(t < 25.0, "시드 {seed} {}: 못 주움 {:?} 무기 {rect:?}", HIDE_SPOTS[spot].0, g.player.pos);
        }
        slowest[spot] = slowest[spot].max(t);
    }
    assert_eq!(done, [500; 6]);
    println!("숨는 곳별 가장 늦게 주운 초 {slowest:?}, 떨어지는 중 주움 {caught_falling:?}");
}

/// 미스를 몇 번 찾아야 총이 나오는지 재기 (칼, 도끼 뒤 10%).
#[test]
#[ignore]
fn measure_finds_until_gun() {
    use logic::weapon::Weapons;
    let w = World::long_map(1);
    let far = Rect::new(-999.0, -999.0, 1.0, 1.0);
    let mut finds = Vec::new();
    for seed in 0..2000 {
        let mut weapons = Weapons::new(seed);
        let mut n = 0;
        while !weapons.owned().contains(&logic::weapon::Weapon::Gun) && n < 500 {
            weapons.drop_one(100.0, &w);
            n += 1;
            let body = weapons.dropped[0].rect;
            weapons.update(0.0, body);
            weapons.update(0.0, far);
        }
        finds.push(n);
    }
    finds.sort();
    println!("총까지 찾는 횟수 중앙값 {}, 90% {}, 최대 {}", finds[1000], finds[1800], finds[1999]);
}
