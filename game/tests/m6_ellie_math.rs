//! 엘리 완료 조건 시험: 수학 문제 (US-074~076). 스토리의 수치를 그대로 쓴다.
//! 반복은 모두 횟수가 정해져 있다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::game::{Game, GameEvent};
use logic::math::{MathKind, MathQuestion, MathQuestionMaker};
use logic::player::PlayerInput;
use logic::world::World;

const SEEDS: u32 = 1000;
const FRAME: f32 = 1.0 / 60.0;
const NONE: PlayerInput = PlayerInput { left: false, right: false, jump_pressed: false, attack_pressed: false };
const TEAM: [&str; 7] = ["니로", "엘리", "로건", "수산나", "헤일리", "밥", "케빈"];

fn numbers_in(text: &str) -> Vec<u32> {
    text.split(|c: char| !c.is_ascii_digit()).filter(|s| !s.is_empty()).map(|s| s.parse().unwrap()).collect()
}

/// 시드 1,000개 x 5문제
fn many() -> Vec<MathQuestion> {
    (0..SEEDS).flat_map(|s| { let mut m = MathQuestionMaker::new(s); (0..5).map(move |_| m.next().unwrap()).collect::<Vec<_>>() }).collect()
}

#[test]
fn us074_each_kind_at_least_15_percent() {
    let first: Vec<_> = (0..SEEDS).map(|s| MathQuestionMaker::new(s).next().unwrap()).collect();
    for kind in MathKind::ALL {
        let n = first.iter().filter(|q| q.kind == kind).count();
        assert!(n * 100 >= 15 * SEEDS as usize, "{kind:?} {n}/1000");
    }
}

#[test]
fn us074_subtract_follows_story_range() {
    // 표: 빼지는 수 100~999, 빼는 수 10 이상, 답 1 이상
    let bad: Vec<String> = many().into_iter().filter(|q| q.kind == MathKind::Subtract).filter(|q| {
        let n = numbers_in(&q.text);
        !(100..=999).contains(&n[0]) || n[1] < 10 || q.answer < 1
    }).map(|q| q.text).take(5).collect();
    assert!(bad.is_empty(), "표 범위를 벗어난 뺄셈: {bad:?}");
}

#[test]
fn us074_add_ranges_and_carry_borrow_exist() {
    let qs = many();
    let mut carry = false;
    let mut borrow = false;
    for q in &qs {
        let n = numbers_in(&q.text);
        assert!(q.answer <= 999, "{}", q.text);
        match q.kind {
            MathKind::AddTwo => {
                assert!(n.iter().all(|x| (100..=899).contains(x)) && q.answer <= 999, "{}", q.text);
                carry |= n[0] % 10 + n[1] % 10 >= 10;
            }
            MathKind::AddThree => assert!(n.iter().all(|x| (1..=99).contains(x)), "{}", q.text),
            MathKind::Multiply => assert!((2..=9).contains(&n[0]) && (1..=9).contains(&n[1]), "{}", q.text),
            MathKind::Subtract => borrow |= n[0] % 10 < n[1] % 10,
        }
    }
    assert!(carry && borrow, "받아올림 {carry} 받아내림 {borrow}");
}

#[test]
fn us074_sentences_ask_and_show_no_formula() {
    for q in many() {
        assert!(q.text.ends_with("까요?"), "묻는 말로 끝나야: {}", q.text);
        assert!(!q.text.contains(['+', '-', '=', '×', '{', '}']), "식이나 빈칸: {}", q.text);
        assert!(q.text.chars().count() <= 60, "{}자: {}", q.text.chars().count(), q.text);
        for scary in ["죽", "피", "때려", "무서"] {
            assert!(!q.text.contains(scary), "무서운 말 {scary}: {}", q.text);
        }
        assert!(!q.text.contains("시계") && !q.text.contains("시에"), "시계 문제 없음: {}", q.text);
    }
}

#[test]
fn us074_every_story_and_name_shows_up() {
    let qs = many();
    for kind in MathKind::ALL {
        assert!(kind.stories().len() >= 3, "{kind:?} 틀 {}개", kind.stories().len());
        for i in 0..kind.stories().len() {
            assert!(qs.iter().any(|q| q.kind == kind && q.story == i), "{kind:?} 틀 {i} 안 나옴");
        }
    }
    for name in TEAM.iter().chain(&["미스", "카스테라"]) {
        assert!(qs.iter().any(|q| q.text.contains(name)), "{name} 안 나옴");
    }
}

#[test]
fn us074_no_same_question_twice_in_a_row_and_seed_repeats() {
    let mut m = MathQuestionMaker::new(42);
    let mut again = MathQuestionMaker::new(42);
    let mut last: Option<MathQuestion> = None;
    for _ in 0..100 {
        let q = m.next().unwrap();
        assert_eq!(Some(&q), again.next().as_ref(), "같은 시드는 같은 문제");
        assert_ne!(last.as_ref().map(|l| &l.text), Some(&q.text));
        last = Some(q);
    }
}

/// 몬스터를 막는 벽 없는 보통 맵.
fn plain_game() -> Game {
    let world = World { width: WORLD_WIDTH, ground_y: GROUND_Y, obstacles: vec![] };
    Game::on_world(CharacterKind::Niro, 3, world)
}

#[test]
fn us075_night_question_comes_before_the_first_monster() {
    let mut g = plain_game();
    // 18:00 직전까지 가고, 그 뒤 한 프레임씩 (최대 10초)
    g.update(&NONE, DAY_SECONDS - 1.0);
    g.combat.zombies.list.clear();
    g.combat.skeletons.clear();
    for _ in 0..600 {
        let events = g.update(&NONE, FRAME);
        if events.contains(&GameEvent::NightStarted) {
            assert!(g.math.is_open(), "밤이 되면 문제가 열린다");
            let monsters = g.combat.zombies.list.len() + g.combat.skeletons.list.len();
            assert_eq!(monsters, 0, "문제보다 먼저 몬스터 {monsters}마리가 나왔다");
            return;
        }
    }
    panic!("10초 안에 밤이 오지 않았다");
}
