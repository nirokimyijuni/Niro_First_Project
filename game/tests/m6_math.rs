//! 미스의 수학 문제 시험 (US-074~076, D67, 부모님 답 2026-11-08) (수산나).
//! 문제 만들기는 시드 1,000개로, 나오는 때·멈춤·체력은 진짜 `Game`으로 본다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::combat::CombatEvent;
use logic::game::{Game, GameEvent};
use logic::math::{fill_story, with_particle, MathKind, MathQuestion, MathQuestionMaker, MathResult};
use logic::monster::Zombie;
use logic::player::PlayerInput;
use logic::weapon::{DroppedWeapon, Weapon};
use logic::world::World;
use macroquad::math::Rect;

const FRAME: f32 = 1.0 / 60.0;
const SEEDS: u32 = 1000;
const NONE: PlayerInput = PlayerInput { left: false, right: false, jump_pressed: false, attack_pressed: false };

/// 문장에서 수만 순서대로 꺼낸다 (이름에는 숫자가 없다).
fn numbers_in(text: &str) -> Vec<u32> {
    text.split(|c: char| !c.is_ascii_digit()).filter(|s| !s.is_empty()).map(|s| s.parse().unwrap()).collect()
}

fn first_questions() -> Vec<MathQuestion> {
    (0..SEEDS).map(|seed| MathQuestionMaker::new(seed).next().unwrap()).collect()
}

// ---- 문제 만들기 ----

/// 부모님 말(2026-11-08): 세 수 더하기는 각 수 1~20. 값을 바꾸면 여기서 먼저 알린다.
#[test]
fn add_three_numbers_are_1_to_20_and_both_ends_show_up() {
    assert_eq!(MATH_ADD_THREE_RANGE, (1, 20));
    let mut seen = Vec::new();
    for seed in 0..SEEDS {
        let mut maker = MathQuestionMaker::new(seed);
        for _ in 0..5 {
            let q = maker.next().unwrap();
            if q.kind == MathKind::AddThree {
                let n = numbers_in(&q.text);
                assert!(n.iter().all(|x| (1..=20).contains(x)), "{}", q.text);
                seen.extend(n);
            }
        }
    }
    assert!(seen.contains(&1) && seen.contains(&20), "1과 20이 한 번은 나와야 해요");
}

#[test]
fn every_kind_shows_up_and_none_is_rare() {
    let questions = first_questions();
    for kind in MathKind::ALL {
        let count = questions.iter().filter(|q| q.kind == kind).count();
        assert!(count * 100 >= SEEDS as usize * 15, "{kind:?} {count}번 (15% 이상)");
    }
}

#[test]
fn answers_are_right_and_in_range_for_1000_seeds() {
    for seed in 0..SEEDS {
        let mut maker = MathQuestionMaker::new(seed);
        for _ in 0..5 {
            let q = maker.next().unwrap();
            let n = numbers_in(&q.text);
            assert!(q.answer <= MATH_ANSWER_MAX, "{}", q.text);
            let in_range = |x: u32, (low, high): (u32, u32)| (low..=high).contains(&x);
            match q.kind {
                MathKind::Multiply => {
                    assert_eq!(n.len(), 2, "{}", q.text);
                    assert!(MATH_TIMES_TABLES.contains(&n[0]) && (1..=MATH_TIMES_MAX).contains(&n[1]), "{}", q.text);
                    assert_eq!(q.answer, n[0] * n[1], "{}", q.text);
                }
                MathKind::AddTwo => {
                    assert_eq!(n.len(), 2, "{}", q.text);
                    assert!(in_range(n[0], MATH_ADD_TWO_RANGE) && in_range(n[1], MATH_ADD_TWO_RANGE), "{}", q.text);
                    assert_eq!(q.answer, n[0] + n[1], "{}", q.text);
                }
                MathKind::AddThree => {
                    assert_eq!(n.len(), 3, "{}", q.text);
                    assert!(n.iter().all(|x| in_range(*x, MATH_ADD_THREE_RANGE)), "{}", q.text);
                    assert_eq!(q.answer, n[0] + n[1] + n[2], "{}", q.text);
                }
                MathKind::Subtract => {
                    assert_eq!(n.len(), 2, "{}", q.text);
                    assert!(in_range(n[0], MATH_SUBTRACT_RANGE), "{}", q.text);
                    assert!(n[1] >= MATH_SUBTRACT_TAKE_MIN && n[1] < n[0], "빼는 수 10 이상, 답 1 이상: {}", q.text);
                    assert_eq!(q.answer, n[0] - n[1], "{}", q.text);
                }
            }
        }
    }
}

#[test]
fn no_blank_left_in_any_sentence_for_1000_seeds() {
    for seed in 0..SEEDS {
        let mut maker = MathQuestionMaker::new(seed);
        for _ in 0..5 {
            let q = maker.next().unwrap();
            assert!(!q.text.contains('{') && !q.text.contains('}') && !q.text.contains('□'), "{}", q.text);
            assert!(MATH_STORY_NAMES.iter().any(|name| q.text.contains(name)), "이름이 나온다: {}", q.text);
            assert!(!q.text.contains('×') && !q.text.contains('=') , "식은 보여 주지 않는다: {}", q.text);
        }
    }
}

#[test]
fn particles_fit_every_name_in_every_story() {
    // (이름, 이/가, 은/는, 을/를)을 사람이 직접 적은 표. 계산으로 만든 값과 맞춰 본다.
    let expected = [
        ("니로", "니로가", "니로는", "니로를"),
        ("엘리", "엘리가", "엘리는", "엘리를"),
        ("로건", "로건이", "로건은", "로건을"),
        ("수산나", "수산나가", "수산나는", "수산나를"),
        ("헤일리", "헤일리가", "헤일리는", "헤일리를"),
        ("밥", "밥이", "밥은", "밥을"),
        ("케빈", "케빈이", "케빈은", "케빈을"),
        ("미스", "미스가", "미스는", "미스를"),
        ("카스테라", "카스테라가", "카스테라는", "카스테라를"),
    ];
    assert_eq!(expected.len(), MATH_STORY_NAMES.len());
    for (name, i_ga, eun_neun, eul_reul) in expected {
        assert!(MATH_STORY_NAMES.contains(&name));
        assert_eq!(with_particle(name, "이", "가"), i_ga);
        for kind in MathKind::ALL {
            for story in kind.stories() {
                let text = fill_story(story, name, [12, 3, 45]);
                assert!(!text.contains('{'), "{text}");
                for (mark, want) in [("{이름:이}", i_ga), ("{이름:은}", eun_neun), ("{이름:을}", eul_reul)] {
                    if story.contains(mark) {
                        assert!(text.contains(want), "{want}: {text}");
                    }
                }
            }
        }
    }
}

#[test]
fn every_story_has_its_numbers_and_a_name_slot() {
    for kind in MathKind::ALL {
        assert!(kind.stories().len() >= 3, "{kind:?}는 문장 틀이 3개 이상");
        let slots: &[&str] = if kind == MathKind::AddThree { &["{a}", "{b}", "{c}"] } else { &["{a}", "{b}"] };
        for story in kind.stories() {
            for slot in slots {
                assert_eq!(story.matches(slot).count(), 1, "{story}");
            }
            let a = story.find("{a}").unwrap();
            let b = story.find("{b}").unwrap();
            assert!(a < b, "수는 a, b, c 순서: {story}");
            assert!(story.contains("{이름:"), "{story}");
        }
    }
}

#[test]
fn same_sentence_never_comes_twice_in_a_row() {
    for seed in 0..50 {
        let mut maker = MathQuestionMaker::new(seed);
        let mut last = maker.next().unwrap();
        for _ in 0..100 {
            let q = maker.next().unwrap();
            assert!(!(q.kind == last.kind && q.story == last.story), "{} / {}", last.text, q.text);
            last = q;
        }
    }
}

#[test]
fn same_seed_makes_same_questions() {
    let mut a = MathQuestionMaker::new(42);
    let mut b = MathQuestionMaker::new(42);
    for _ in 0..30 {
        assert_eq!(a.next(), b.next());
    }
}

#[test]
fn a_kind_that_is_off_never_shows_up() {
    let on = [MathKind::Multiply, MathKind::AddTwo, MathKind::AddThree];
    for seed in 0..200 {
        let mut maker = MathQuestionMaker::with_kinds(seed, &on);
        for _ in 0..5 {
            assert_ne!(maker.next().unwrap().kind, MathKind::Subtract);
        }
    }
}

#[test]
fn math_dice_do_not_change_the_other_dice() {
    // 문제 주사위는 따로 굴린다: 문제를 내도 상자·미스 자리는 시드대로다.
    let plain = Game::new(CharacterKind::Niro, 9);
    let mut asked = Game::new(CharacterKind::Niro, 9);
    asked.math.add_waiting();
    asked.update(&NONE, FRAME);
    assert!(asked.math.is_open());
    assert_eq!(plain.world.obstacles, asked.world.obstacles);
}

// ---- 나오는 때 ----

/// 몬스터가 오지 않게 벽 사이에 넣은 게임 (game.rs `clock_events_come_through_game`과 같은 벽).
/// 큰 `dt` 한 번이면 몬스터도 그만큼 움직여서, 벽이 없으면 밤 문제 전에 쓰러진다.
fn quiet_game() -> Game {
    let wall = |x| Rect::new(x, 0.0, 10.0, GROUND_Y);
    let walled = World { width: WORLD_WIDTH, ground_y: GROUND_Y, obstacles: vec![wall(100.0), wall(180.0)] };
    let mut g = Game::on_world(CharacterKind::Niro, 7, walled);
    g.combat.skeleton_chance = 0.0;
    g
}

fn give_weapon(g: &mut Game, kind: Weapon) -> Vec<GameEvent> {
    let mut drop = DroppedWeapon::above(kind, 0.0, &g.world);
    drop.rect = g.player.rect();
    g.weapons.dropped.push(drop);
    g.update(&NONE, FRAME)
}

fn answer_right_and_close(g: &mut Game) {
    let answer = g.math.question().expect("문제가 열려 있다").answer;
    assert_eq!(g.math.type_and_submit(answer), Some(MathResult::Right));
    g.update(&NONE, MATH_RIGHT_SHOW_SECONDS);
    assert!(!g.math.is_open());
}

#[test]
fn every_second_item_opens_a_question() {
    // 이미 가진 무기를 또 주워도 1개로 센다 (US-075). 미스가 다시 나와 계속 떨어뜨리니 계속 나온다.
    let mut g = quiet_game();
    assert!(give_weapon(&mut g, Weapon::Sword).contains(&GameEvent::WeaponPicked { kind: Weapon::Sword }));
    assert!(!g.math.is_open(), "첫 번째 아이템은 문제가 없다");
    assert!(give_weapon(&mut g, Weapon::Sword).is_empty(), "가진 칼을 또 주우면 새 무기 글은 없다");
    assert!(g.math.is_open(), "두 번째 아이템에 문제가 나온다");
    answer_right_and_close(&mut g);
    give_weapon(&mut g, Weapon::Axe);
    assert!(!g.math.is_open(), "세 번째는 없다");
    assert_eq!(g.weapons.held(), Weapon::Axe, "아이템은 그대로 얻는다");
    give_weapon(&mut g, Weapon::Sword);
    assert!(g.math.is_open(), "네 번째에 또 나온다");
    assert_eq!(g.weapons.picked_up_count, 4);
}

#[test]
fn coins_do_not_open_a_question() {
    let mut g = quiet_game();
    g.player.pos = g.coins.list[0].rect.point();
    let events = g.update(&NONE, FRAME);
    assert!(events.iter().any(|e| matches!(e, GameEvent::CoinsPicked { .. })));
    assert!(!g.math.is_open());
}

#[test]
fn every_nightfall_opens_one_question_for_7_days() {
    let mut g = quiet_game();
    let mut asked = 0;
    for _ in 0..DAYS_TO_CLEAR * 2 {
        // 낮 → 밤, 밤 → 아침을 한 번에 넘긴다 (빨리감기로 건너뛰는 것과 같다).
        g.combat.zombies.list.clear();
        g.combat.skeletons.clear();
        let phase = if g.clock.is_night() { NIGHT_SECONDS } else { DAY_SECONDS };
        let events = g.update_with_time_scale(&NONE, phase / DEV_FAST_FORWARD_SCALE, DEV_FAST_FORWARD_SCALE);
        if events.contains(&GameEvent::NightStarted) {
            assert!(g.math.is_open());
            asked += 1;
            answer_right_and_close(&mut g);
        } else {
            assert!(!g.math.is_open(), "아침에는 나오지 않는다");
        }
    }
    assert_eq!(asked, DAYS_TO_CLEAR);
}

#[test]
fn item_and_night_at_once_open_one_after_another() {
    let mut g = quiet_game();
    give_weapon(&mut g, Weapon::Sword);
    let mut drop = DroppedWeapon::above(Weapon::Axe, 0.0, &g.world);
    drop.rect = g.player.rect();
    g.weapons.dropped.push(drop);
    let left = g.clock.seconds_left_in_phase();
    let events = g.update(&NONE, left + FRAME);
    assert!(events.contains(&GameEvent::NightStarted));
    assert!(g.math.is_open());
    let first = g.math.question().unwrap().clone();
    answer_right_and_close(&mut g);
    g.update(&NONE, FRAME);
    assert!(g.math.is_open(), "두 번째 문제가 이어서 열린다");
    assert_ne!(g.math.question().unwrap(), &first);
    answer_right_and_close(&mut g);
    g.update(&NONE, FRAME);
    assert!(!g.math.is_open(), "두 개뿐이다");
}

#[test]
fn no_question_after_going_down() {
    let mut g = quiet_game();
    let body = g.player.rect();
    for _ in 0..PLAYER_START_HP {
        g.combat.citizen_blast(body.center(), body);
        g.combat.update(PLAYER_HURT_INVINCIBLE_SECONDS, false, body, &g.world);
    }
    assert!(g.is_over());
    g.update(&NONE, DAY_SECONDS);
    assert!(!g.math.is_open());
}

// ---- 멈춤 ----

fn open_night_question(g: &mut Game) {
    g.update(&NONE, DAY_SECONDS);
    assert!(g.clock.is_night() && g.math.is_open());
}

#[test]
fn everything_waits_while_the_question_is_open() {
    let mut g = quiet_game();
    open_night_question(&mut g);
    let mut drop = DroppedWeapon::above(Weapon::Sword, g.world.width - 100.0, &g.world);
    drop.rect.y = g.world.ground_y - drop.rect.h; // 땅에 놓고 15초를 센다
    g.weapons.dropped.push(drop);
    let body = g.player.rect();
    g.combat.citizen_blast(body.center(), body);
    assert!(g.combat.health.is_invincible());
    let x = g.player.rect().center().x;
    g.combat.zombies.list.push(Zombie::new(x, g.world.ground_y, false));

    let clock = (g.clock.time_of_day_minutes(), g.clock.seconds_left_in_phase());
    let hp = g.combat.health.hp();
    let zombie = g.combat.zombies.list[0].rect();
    let weapon_left = g.weapons.dropped[0].seconds_left();
    let coin = g.coins.list[0].rect.point();
    g.player.pos = coin; // 코인에 닿아도 줍지 않는다

    for _ in 0..(70.0 / FRAME) as usize {
        assert!(g.update(&PlayerInput { right: true, jump_pressed: true, attack_pressed: true, ..NONE }, FRAME).is_empty());
        g.update_with_time_scale(&NONE, FRAME, DEV_FAST_FORWARD_SCALE);
    }
    assert!(g.math.is_open(), "1분 넘게 두어도 그대로다");
    assert_eq!(g.math.result(), None, "저절로 틀리지 않는다");
    assert_eq!((g.clock.time_of_day_minutes(), g.clock.seconds_left_in_phase()), clock);
    assert_eq!(g.combat.health.hp(), hp, "몬스터에 닿아 있어도 체력이 그대로");
    assert!(g.combat.health.is_invincible(), "무적 시간도 멈춘다");
    assert_eq!(g.combat.zombies.list.len(), 1, "새 몬스터가 나오지 않는다");
    assert_eq!(g.combat.zombies.list[0].rect(), zombie);
    assert_eq!(g.weapons.dropped[0].seconds_left(), weapon_left);
    assert_eq!(g.player.pos, coin, "걷기·점프도 멈춘다");
    assert_eq!(g.wallet.coins(), 0);
}

// ---- 답 넣기와 체력 ----

#[test]
fn right_answer_keeps_health_and_closes_after_2_seconds() {
    let mut g = quiet_game();
    open_night_question(&mut g);
    let (hp, score) = (g.combat.health.hp(), g.score());
    let answer = g.math.question().unwrap().answer;
    assert_eq!(g.math.type_and_submit(answer), Some(MathResult::Right));
    g.update(&NONE, MATH_RIGHT_SHOW_SECONDS - 0.1);
    assert!(g.math.is_open(), "칭찬 글 2초 동안은 열려 있다");
    g.update(&NONE, 0.2);
    assert!(!g.math.is_open());
    assert_eq!((g.combat.health.hp(), g.score()), (hp, score), "보상도 벌도 없다");
}

fn miss_three_times(g: &mut Game) {
    let answer = g.math.question().unwrap().answer;
    for _ in 0..MATH_TRIES {
        g.math.type_and_submit(answer + 1);
    }
    assert_eq!(g.math.result(), Some(MathResult::Wrong));
}

#[test]
fn one_or_two_misses_cost_nothing() {
    let mut g = quiet_game();
    open_night_question(&mut g);
    let answer = g.math.question().unwrap().answer;
    assert_eq!(g.math.type_and_submit(answer + 1), Some(MathResult::TryAgain));
    assert_eq!(g.math.tries_left_text(), "기회가 2번 남았어");
    g.update(&NONE, 30.0);
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP);
    assert!(g.math.is_open(), "다시 해 볼 시간 제한은 없다");
}

#[test]
fn three_misses_cost_2_hp_when_the_answer_closes() {
    let mut g = quiet_game();
    open_night_question(&mut g);
    miss_three_times(&mut g);
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP, "정답 글을 보여 주는 동안은 그대로");
    g.update(&NONE, MATH_WRONG_SHOW_SECONDS - 0.1);
    assert!(g.math.is_open());
    let events = g.update(&NONE, 0.2);
    assert!(!g.math.is_open());
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP - MATH_WRONG_HP_LOSS);
    assert!(events.contains(&GameEvent::Combat(CombatEvent::PlayerHurt { amount: MATH_WRONG_HP_LOSS, hp_left: PLAYER_START_HP - MATH_WRONG_HP_LOSS })));
    assert!(!g.combat.health.is_invincible(), "무적 시간은 생기지 않는다");
}

#[test]
fn three_misses_with_low_hp_go_down_like_any_fall() {
    let mut g = quiet_game();
    let body = g.player.rect();
    for _ in 0..(PLAYER_START_HP - 1) / CITIZEN_BLAST_DAMAGE {
        g.combat.citizen_blast(body.center(), body);
        g.combat.update(PLAYER_HURT_INVINCIBLE_SECONDS, false, body, &g.world);
    }
    assert_eq!(g.combat.health.hp(), 1);
    open_night_question(&mut g);
    miss_three_times(&mut g);
    let events = g.update(&NONE, MATH_WRONG_SHOW_SECONDS);
    assert_eq!(g.combat.health.hp(), 0, "부모님 답: 문제로도 쓰러질 수 있다");
    assert!(g.is_over() && !g.is_cleared());
    assert!(events.contains(&GameEvent::Combat(CombatEvent::PlayerDown)));
    assert!(g.update(&NONE, FRAME).is_empty(), "쓰러진 뒤에는 멈춘다");
}

#[test]
fn night_question_opens_before_any_monster_at_normal_speed() {
    // BUG-025: 18:00이 되는 그 프레임에 몬스터가 먼저 나오면 안 된다.
    let mut g = quiet_game();
    g.update(&NONE, DAY_SECONDS - 0.5);
    for _ in 0..(1.0 / FRAME) as usize {
        let events = g.update(&NONE, FRAME);
        if events.contains(&GameEvent::NightStarted) {
            assert!(g.math.is_open());
            assert_eq!(g.combat.zombies.list.len() + g.combat.skeletons.list.len(), 0);
            answer_right_and_close(&mut g);
            g.update(&NONE, FRAME);
            assert!(g.combat.zombies.list.len() + g.combat.skeletons.list.len() > 0, "문제를 닫으면 몬스터가 나온다");
            return;
        }
    }
    panic!("1초 안에 밤이 오지 않았다");
}
