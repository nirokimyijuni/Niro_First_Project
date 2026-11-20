//! 잠깐 퀴즈 영어 낱말 문제 시험 (US-087·088, D82·D106) (수산나).
//! 기준: stories/backlog.md [임시-56]~[임시-62]. 수치·낱말은 config 상수로 적어서 임시 값을 바꿔도 시험은 그대로다.
//! 뽑기는 정해진 시드로만 보고, 반복은 모두 횟수가 정해져 있다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::character::CharacterKind;
use logic::combat::CombatEvent;
use logic::english::{EnglishKind, EnglishQuestion, EnglishQuestionMaker};
use logic::game::{Game, GameEvent};
use logic::math::{MathQuestionMaker, MathQuiz, MathResult, QuizSubject};
use logic::player::PlayerInput;
use logic::shop::Armor;
use logic::weapon::{DroppedWeapon, Weapon};
use logic::world::World;

const FRAME: f32 = 1.0 / 60.0;
const SEEDS: u32 = 1000;
const NONE: PlayerInput = PlayerInput { left: false, right: false, jump_pressed: false, attack_pressed: false };

/// 시드마다 처음 낸 영어 문제 1,000개.
fn first_questions() -> Vec<EnglishQuestion> {
    (0..SEEDS).map(|seed| EnglishQuestionMaker::new(seed).next().expect("문제가 나온다")).collect()
}

/// 시드 몇 개로 문제를 이어서 많이 낸다 (시드 50개 × 20문제 = 1,000문제).
fn many_questions() -> Vec<EnglishQuestion> {
    let mut all = Vec::new();
    for seed in 0..50 {
        let mut maker = EnglishQuestionMaker::new(seed);
        for _ in 0..20 {
            all.push(maker.next().expect("문제가 나온다"));
        }
    }
    all
}

fn are_confusing(a: &str, b: &str) -> bool {
    ENGLISH_CONFUSING_GROUPS.iter().any(|group| group.contains(&a) && group.contains(&b))
}

// ---- 수학과 영어 고르기 [임시-56] ----

#[test]
fn english_is_about_half_of_the_first_quizzes_for_1000_seeds() {
    let mut english = 0;
    for seed in 0..SEEDS {
        let mut g = Game::on_world(CharacterKind::Niro, seed, World::m1());
        g.math.add_waiting();
        g.math.open_next_waiting();
        match g.math.subject() {
            Some(QuizSubject::English) => english += 1,
            Some(QuizSubject::Math) => {}
            None => panic!("seed {seed}: 문제가 열려야 한다"),
        }
    }
    assert_eq!(ENGLISH_QUIZ_PERCENT, 50, "부모님 확정: 반반");
    assert!((450..=550).contains(&english), "영어 {english}개 / 1000");
}

fn subjects_with_percent(percent: u32) -> Vec<QuizSubject> {
    let mut quiz = MathQuiz::with_subjects(MathQuestionMaker::new(3), EnglishQuestionMaker::new(3), 3, percent);
    let mut subjects = Vec::new();
    for _ in 0..200 {
        quiz.add_waiting();
        quiz.open_next_waiting();
        subjects.push(quiz.subject().expect("열린다"));
        quiz.answer_right();
        quiz.wait_after_answer(MATH_RIGHT_SHOW_SECONDS);
        assert!(!quiz.is_open());
    }
    subjects
}

#[test]
fn percent_0_is_only_math_and_100_is_only_english() {
    assert!(subjects_with_percent(0).iter().all(|s| *s == QuizSubject::Math));
    assert!(subjects_with_percent(100).iter().all(|s| *s == QuizSubject::English));
}

#[test]
fn same_seed_same_order_and_math_sentences_do_not_change() {
    let run = |seed: u32| {
        let mut g = Game::on_world(CharacterKind::Niro, seed, World::m1());
        let mut seen = Vec::new();
        for _ in 0..30 {
            g.math.add_waiting();
            g.math.open_next_waiting();
            seen.push((g.math.subject(), g.math.question().cloned(), g.math.english_question().cloned()));
            g.math.answer_right();
            g.math.wait_after_answer(MATH_RIGHT_SHOW_SECONDS);
        }
        seen
    };
    for seed in [0, 7, 42] {
        let seen = run(seed);
        assert_eq!(seen, run(seed), "seed {seed}");
        assert!(seen.iter().any(|(s, ..)| *s == Some(QuizSubject::Math)));
        assert!(seen.iter().any(|(s, ..)| *s == Some(QuizSubject::English)));
        // 영어 문제가 끼어도 수학 문제는 영어 없는 문제 만들기와 같은 차례다.
        let mut maker = MathQuestionMaker::new(seed);
        for (_, math, _) in seen.iter().filter(|(s, ..)| *s == Some(QuizSubject::Math)) {
            assert_eq!(*math, maker.next(), "seed {seed}");
        }
    }
}

// ---- 문제 만들기 [임시-57], [임시-59], [임시-62] ----

#[test]
fn both_kinds_show_up_more_than_40_percent() {
    let questions = first_questions();
    let picture_to_word = questions.iter().filter(|q| q.kind == EnglishKind::PictureToWord).count();
    let word_to_picture = questions.len() - picture_to_word;
    assert!(picture_to_word > 400 && word_to_picture > 400, "(가) {picture_to_word}, (나) {word_to_picture}");
}

#[test]
fn four_different_choices_with_exactly_one_answer() {
    for (i, q) in first_questions().iter().chain(many_questions().iter()).enumerate() {
        assert_eq!(q.choices.len(), 4);
        assert!(q.answer < 4);
        for a in 0..4 {
            for b in (a + 1)..4 {
                assert_ne!(q.choices[a].word, q.choices[b].word, "문제 {i}");
            }
        }
        let answer = q.answer_word().word;
        assert_eq!(q.choices.iter().filter(|c| c.word == answer).count(), 1, "문제 {i}");
    }
}

#[test]
fn answer_slot_is_spread_over_1_to_4() {
    let mut count = [0; 4];
    for q in first_questions() {
        count[q.answer] += 1;
    }
    assert!(count.iter().all(|&c| c >= 200), "1~4번 자리 {count:?} / 1000");
}

#[test]
fn same_answer_word_never_comes_twice_in_a_row() {
    let mut maker = EnglishQuestionMaker::new(11);
    let mut last = maker.next().unwrap().answer_word().word;
    for i in 0..100 {
        let now = maker.next().unwrap().answer_word().word;
        assert_ne!(now, last, "{i}번째");
        last = now;
    }
}

#[test]
fn all_48_words_are_answers_in_1000_seeds_and_only_first_letter_is_big() {
    assert_eq!(ENGLISH_WORDS.len(), 48);
    let answers: Vec<&str> = first_questions().iter().map(|q| q.answer_word().word).collect();
    for (word, _) in ENGLISH_WORDS {
        assert!(answers.contains(word), "{word}가 정답으로 안 나왔다");
        let mut letters = word.chars();
        assert!(letters.next().is_some_and(|c| c.is_ascii_uppercase()) && letters.all(|c| c.is_ascii_lowercase()), "{word}");
    }
    assert!(ENGLISH_WORDS.iter().any(|(w, _)| *w == "Acorn") && ENGLISH_WORDS.iter().any(|(w, _)| *w == "Kite"));
    // Pole은 뺐다 (D106): 정답에도 보기에도 안 나온다.
    assert!(first_questions().iter().all(|q| q.choices.iter().all(|c| c.word != "Pole")), "Pole이 나왔다");
}

#[test]
fn confusing_words_never_share_one_question() {
    // D106 묶음이 config에 다 있다.
    let groups: Vec<Vec<&str>> = ENGLISH_CONFUSING_GROUPS.iter().map(|g| g.to_vec()).collect();
    for expected in [
        vec!["Pine", "Tree", "Maple"],
        vec!["Feet", "Teeth", "Cheek"],
        vec!["Hole", "Mole"],
        vec!["Apron", "Bib", "Vest", "Jacket"],
        vec!["Baby", "Doll", "King"],
        vec!["Fire", "Smoke", "Sky"],
        vec!["Hose", "Rope", "Mop"],
        vec!["Watch", "Phone"],
        vec!["Acorn", "Seed"],
        vec!["Cone", "Tent", "Pine"],
        vec!["Insect", "Beehive"],
        vec!["Ice", "Candy"],
    ] {
        assert!(groups.contains(&expected), "{expected:?} 묶음이 없다");
    }
    let mut pine_answers = 0;
    for q in first_questions().iter().chain(many_questions().iter()) {
        for a in 0..4 {
            for b in (a + 1)..4 {
                let (x, y) = (q.choices[a].word, q.choices[b].word);
                assert!(!are_confusing(x, y), "{x}와 {y}가 한 문제에 나왔다: {q:?}");
            }
        }
        if q.answer_word().word == "Pine" {
            pine_answers += 1;
            for other in ["Tree", "Maple", "Cone", "Tent"] {
                assert!(q.choices.iter().all(|c| c.word != other), "정답이 Pine이면 {other}는 안 나온다");
            }
        }
    }
    assert!(pine_answers > 0, "Pine이 정답인 문제도 보았다");
}

// 낱말 한 줄을 더하거나 빼면 문제도 따라 바뀐다 [임시-59].
static WITH_ONE_MORE: &[(&str, &str)] = &[
    ("Fan", "en-fan.png"),
    ("Hen", "en-hen.png"),
    ("Net", "en-net.png"),
    ("Kite", "en-kite.png"),
    ("Lamp", "en-lamp.png"),
    ("Zebra", "en-zebra.png"),
];

#[test]
fn adding_or_removing_a_word_line_changes_the_questions() {
    // 맨 앞 줄(Acorn)을 뺀 표
    let without_acorn = &ENGLISH_WORDS[1..];
    assert_eq!(without_acorn.len(), 47);
    for seed in 0..200 {
        let mut maker = EnglishQuestionMaker::with_words(seed, without_acorn, ENGLISH_CONFUSING_GROUPS);
        for _ in 0..5 {
            let q = maker.next().unwrap();
            assert!(q.choices.iter().all(|c| c.word != "Acorn"), "뺀 낱말은 안 나온다");
        }
    }
    // 새 줄(Zebra)을 더한 표
    let mut zebra_answers = 0;
    for seed in 0..200 {
        let q = EnglishQuestionMaker::with_words(seed, WITH_ONE_MORE, &[]).next().unwrap();
        if q.answer_word().word == "Zebra" {
            zebra_answers += 1;
            assert_eq!(q.answer_word().picture, "en-zebra.png");
        }
    }
    assert!(zebra_answers > 0, "더한 낱말이 나온다");
}

#[test]
fn pictures_are_named_after_the_word_and_tree_uses_the_game_tree() {
    for (word, picture) in ENGLISH_WORDS {
        if *word == "Tree" {
            assert_eq!(*picture, ENGLISH_GAME_TREE_PICTURE);
        } else {
            assert_eq!(*picture, format!("en-{}.png", word.to_lowercase()));
        }
    }
}

// ---- 창, 기회, 체력 [임시-58] ----

/// 영어만 내는 퀴즈 창을 단 게임. 미션 실패와 해골은 섞지 않는다.
fn english_game(seed: u32) -> Game {
    let mut g = Game::on_world(CharacterKind::Niro, seed, World::m1());
    g.combat.skeleton_chance = 0.0;
    g.missions.turn_off_all();
    g.math = MathQuiz::with_subjects(MathQuestionMaker::new(seed), EnglishQuestionMaker::new(seed), seed, 100);
    g
}

fn open_night_quiz(g: &mut Game) -> EnglishQuestion {
    g.update(&NONE, DAY_SECONDS);
    assert!(g.clock.is_night());
    assert_eq!(g.math.subject(), Some(QuizSubject::English));
    assert_eq!(g.math.tries_left(), ENGLISH_TRIES);
    g.math.english_question().cloned().unwrap()
}

fn a_wrong_slot(q: &EnglishQuestion, not: Option<usize>) -> usize {
    (0..4).find(|&s| s != q.answer && Some(s) != not).unwrap()
}

#[test]
fn texts_for_the_quiz_window() {
    assert_eq!(EnglishKind::PictureToWord.ask_text(), "이 그림은 영어로 뭘까?");
    assert_eq!(EnglishKind::WordToPicture.ask_text(), "이 낱말에 맞는 그림은?");
    assert_eq!(MATH_TITLE_TEXT, "잠깐 퀴즈!");
    assert_eq!(MATH_MISS_ASK_TEXT, "나랑 같이 풀어 볼래?");
}

#[test]
fn first_miss_keeps_health_and_the_same_question_and_greys_out_the_choice() {
    let mut g = english_game(7);
    let q = open_night_quiz(&mut g);
    let wrong = a_wrong_slot(&q, None);
    assert_eq!(g.math.pick_choice(wrong), Some(MathResult::TryAgain));
    assert_eq!(g.math.result_text().unwrap(), "너무 아쉽다! 다시 해 볼까?");
    assert_eq!(g.math.tries_left_text(), "기회가 1번 남았어");
    assert!(g.math.is_crossed_out(wrong));
    assert!((0..4).filter(|&s| s != wrong).all(|s| !g.math.is_crossed_out(s)));
    assert_eq!(g.math.pick_choice(wrong), None, "흐려진 보기는 다시 골라도 아무 일이 없다");
    assert_eq!(g.math.tries_left(), 1);
    g.update(&NONE, 30.0);
    assert!(g.math.is_open(), "다시 해 볼 시간 제한은 없다");
    assert_eq!(g.math.english_question(), Some(&q), "같은 문제");
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP);
}

#[test]
fn number_keys_do_nothing_on_an_english_question() {
    let mut g = english_game(7);
    open_night_quiz(&mut g);
    g.math.press_digit(1);
    assert_eq!(g.math.typed(), "");
    assert_eq!(g.math.submit(), None);
    assert_eq!(g.math.pick_choice(4), None, "보기는 1~4번(0~3)뿐");
    assert_eq!(g.math.tries_left(), ENGLISH_TRIES);
}

#[test]
fn right_on_first_or_second_try_changes_nothing() {
    for miss_first in [false, true] {
        let mut g = english_game(9);
        let q = open_night_quiz(&mut g);
        let before = (g.combat.health.hp(), g.wallet.coins(), g.score());
        if miss_first {
            assert_eq!(g.math.pick_choice(a_wrong_slot(&q, None)), Some(MathResult::TryAgain));
        }
        assert_eq!(g.math.pick_choice(q.answer), Some(MathResult::Right));
        assert_eq!(g.math.result_text().unwrap(), "넌 최고야!");
        assert_eq!(g.math.pick_choice(q.answer), None, "칭찬 글 동안은 고를 수 없다");
        g.update(&NONE, MATH_RIGHT_SHOW_SECONDS - 0.1);
        assert!(g.math.is_open(), "칭찬 글 2초 동안은 열려 있다");
        g.update(&NONE, 0.2);
        assert!(!g.math.is_open());
        assert_eq!((g.combat.health.hp(), g.wallet.coins(), g.score()), before, "보상도 벌도 없다");
    }
}

fn miss_twice(g: &mut Game, q: &EnglishQuestion) {
    let first = a_wrong_slot(q, None);
    assert_eq!(g.math.pick_choice(first), Some(MathResult::TryAgain));
    assert_eq!(g.math.pick_choice(a_wrong_slot(q, Some(first))), Some(MathResult::Wrong));
}

#[test]
fn two_misses_show_the_word_then_cost_2_hp_even_with_armor() {
    let mut g = english_game(7);
    g.shop.put_on_armor(Armor::Diamond);
    g.combat.health.set_armored(true);
    let q = open_night_quiz(&mut g);
    miss_twice(&mut g, &q);
    let word = q.answer_word().word;
    assert_eq!(g.math.result_text().unwrap(), format!("너무 아쉽다! 정답은 {word} 야"));
    assert_eq!(g.math.pick_choice(q.answer), None, "정답 글이 나온 뒤에는 고를 수 없다");
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP, "정답 글을 보여 주는 동안은 그대로");
    g.update(&NONE, MATH_WRONG_SHOW_SECONDS - 0.1);
    assert!(g.math.is_open(), "정답 글 3초");
    let events = g.update(&NONE, 0.2);
    assert!(!g.math.is_open());
    assert_eq!(ENGLISH_WRONG_HP_LOSS, 2);
    assert_eq!(g.combat.health.hp(), PLAYER_START_HP - ENGLISH_WRONG_HP_LOSS, "갑옷이어도 준다");
    assert!(events.contains(&GameEvent::Combat(CombatEvent::PlayerHurt {
        amount: ENGLISH_WRONG_HP_LOSS,
        hp_left: PLAYER_START_HP - ENGLISH_WRONG_HP_LOSS
    })));
}

#[test]
fn two_misses_with_2_hp_go_down() {
    let mut g = english_game(7);
    let q = open_night_quiz(&mut g);
    g.combat.health.lose_from_quiz(PLAYER_START_HP - 2);
    assert_eq!(g.combat.health.hp(), 2);
    miss_twice(&mut g, &q);
    let events = g.update(&NONE, MATH_WRONG_SHOW_SECONDS);
    assert_eq!(g.combat.health.hp(), 0);
    assert!(g.is_over());
    assert!(events.contains(&GameEvent::Combat(CombatEvent::PlayerDown)));
}

#[test]
fn everything_waits_while_the_english_question_is_open() {
    let mut g = english_game(7);
    open_night_quiz(&mut g);
    let clock = (g.clock.time_of_day_minutes(), g.clock.seconds_left_in_phase());
    let pos = g.player.pos;
    for _ in 0..(70.0 / FRAME) as usize {
        g.update(&PlayerInput { right: true, jump_pressed: true, attack_pressed: true, ..NONE }, FRAME);
    }
    assert!(g.math.is_open() && g.math.result().is_none(), "저절로 틀리거나 닫히지 않는다");
    assert_eq!((g.clock.time_of_day_minutes(), g.clock.seconds_left_in_phase()), clock);
    assert_eq!(g.player.pos, pos);
    assert!(g.combat.zombies.list.is_empty(), "몬스터가 나오지 않는다");
}

#[test]
fn second_weapon_still_gives_the_weapon_and_opens_english() {
    let mut g = english_game(7);
    let give = |g: &mut Game, kind: Weapon| {
        let mut drop = DroppedWeapon::above(kind, 0.0, &g.world);
        drop.rect = g.player.rect();
        g.weapons.dropped.push(drop);
        g.update(&NONE, FRAME);
    };
    give(&mut g, Weapon::Sword);
    assert!(!g.math.is_open());
    give(&mut g, Weapon::Axe);
    assert_eq!(g.math.subject(), Some(QuizSubject::English));
    assert_eq!(g.weapons.held(), Weapon::Axe, "무기는 그대로 얻는다");
}
