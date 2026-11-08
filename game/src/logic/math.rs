//! 미스의 수학 문제 (D67, US-074~076). 상황 문제 만들기, 숫자 넣기, 기회 3번, 결과 글.
//! 언제 여는지와 체력을 줄이는 일은 `Game`이 한다. 화면(로건)은 `MathQuiz`의 값을 보고 그린다.

use super::monster::SeededRandom;
use crate::config::{
    MATH_ADD_THREE_ON, MATH_ADD_THREE_RANGE, MATH_ADD_THREE_STORIES, MATH_ADD_TWO_ON, MATH_ADD_TWO_RANGE,
    MATH_ADD_TWO_STORIES, MATH_ANSWER_DIGITS, MATH_MULTIPLY_ON, MATH_MULTIPLY_STORIES, MATH_RIGHT_SHOW_SECONDS,
    MATH_RIGHT_TEXT, MATH_SEED_MIX, MATH_STORY_NAMES, MATH_SUBTRACT_ON, MATH_SUBTRACT_RANGE, MATH_SUBTRACT_STORIES, MATH_SUBTRACT_TAKE_MIN, MATH_TIMES_MAX,
    MATH_TIMES_TABLES, MATH_TRIES, MATH_TRIES_LEFT_TEXT, MATH_TRY_AGAIN_TEXT, MATH_WRONG_SHOW_SECONDS, MATH_WRONG_TEXT,
};

/// 문제 종류 (부모님 답).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathKind {
    /// {a}개씩 {b}묶음 → a × b
    Multiply,
    /// 두 자리 수 두 개 더하기 → a + b
    AddTwo,
    /// 세 수 연속 더하기 → a + b + c
    AddThree,
    /// 빼기 → a - b (0 이상)
    Subtract,
}

impl MathKind {
    pub const ALL: [MathKind; 4] = [MathKind::Multiply, MathKind::AddTwo, MathKind::AddThree, MathKind::Subtract];

    /// config.rs에서 켜 둔 종류인지.
    pub fn is_on_in_config(self) -> bool {
        match self {
            MathKind::Multiply => MATH_MULTIPLY_ON,
            MathKind::AddTwo => MATH_ADD_TWO_ON,
            MathKind::AddThree => MATH_ADD_THREE_ON,
            MathKind::Subtract => MATH_SUBTRACT_ON,
        }
    }

    /// 이 종류의 상황 문장 틀 (config.rs).
    pub fn stories(self) -> &'static [&'static str] {
        match self {
            MathKind::Multiply => MATH_MULTIPLY_STORIES,
            MathKind::AddTwo => MATH_ADD_TWO_STORIES,
            MathKind::AddThree => MATH_ADD_THREE_STORIES,
            MathKind::Subtract => MATH_SUBTRACT_STORIES,
        }
    }
}

/// 문제 하나: 상황 문장과 정답.
#[derive(Debug, Clone, PartialEq)]
pub struct MathQuestion {
    pub kind: MathKind,
    /// 몇 번째 문장 틀인지 (`kind.stories()`의 자리)
    pub story: usize,
    pub text: String,
    pub answer: u32,
}

/// 같은 시드면 같은 문제를 같은 순서로 낸다. 바로 앞과 같은 문장 틀은 쓰지 않는다.
pub struct MathQuestionMaker {
    random: SeededRandom,
    kinds: Vec<MathKind>,
    last: Option<(MathKind, usize)>,
}

impl MathQuestionMaker {
    /// config.rs에서 켠 종류만 낸다. `game_seed`에 섞는 값을 더해 상자·무기 뽑기와 따로 굴린다.
    pub fn new(game_seed: u32) -> MathQuestionMaker {
        let on: Vec<MathKind> = MathKind::ALL.into_iter().filter(|k| k.is_on_in_config()).collect();
        MathQuestionMaker::with_kinds(game_seed, &on)
    }

    /// 고른 종류만 내는 문제 만들기. 시험이 종류를 끄고 켤 때 쓴다.
    pub fn with_kinds(game_seed: u32, kinds: &[MathKind]) -> MathQuestionMaker {
        MathQuestionMaker { random: SeededRandom::new(game_seed ^ MATH_SEED_MIX), kinds: kinds.to_vec(), last: None }
    }

    /// 다음 문제. 켠 종류가 하나도 없으면 None.
    pub fn next(&mut self) -> Option<MathQuestion> {
        if self.kinds.is_empty() {
            return None;
        }
        let index = self.pick(self.kinds.len());
        let kind = self.kinds[index];
        let story = self.pick_story(kind);
        let (numbers, answer) = match kind {
            MathKind::Multiply => {
                let a = MATH_TIMES_TABLES[self.pick(MATH_TIMES_TABLES.len())];
                let b = self.pick_between(1, MATH_TIMES_MAX);
                ([a, b, 0], a * b)
            }
            MathKind::AddTwo => {
                let a = self.pick_between(MATH_ADD_TWO_RANGE.0, MATH_ADD_TWO_RANGE.1);
                let b = self.pick_between(MATH_ADD_TWO_RANGE.0, MATH_ADD_TWO_RANGE.1);
                ([a, b, 0], a + b)
            }
            MathKind::AddThree => {
                let a = self.pick_between(MATH_ADD_THREE_RANGE.0, MATH_ADD_THREE_RANGE.1);
                let b = self.pick_between(MATH_ADD_THREE_RANGE.0, MATH_ADD_THREE_RANGE.1);
                let c = self.pick_between(MATH_ADD_THREE_RANGE.0, MATH_ADD_THREE_RANGE.1);
                ([a, b, c], a + b + c)
            }
            MathKind::Subtract => {
                let a = self.pick_between(MATH_SUBTRACT_RANGE.0, MATH_SUBTRACT_RANGE.1);
                let b = self.pick_between(MATH_SUBTRACT_TAKE_MIN, a - 1); // 답은 1 이상
                ([a, b, 0], a - b)
            }
        };
        let name = MATH_STORY_NAMES[self.pick(MATH_STORY_NAMES.len())];
        let text = fill_story(kind.stories()[story], name, numbers);
        self.last = Some((kind, story));
        Some(MathQuestion { kind, story, text, answer })
    }

    /// 문장 틀 하나. 바로 앞과 같은 종류면 바로 앞 틀은 빼고 고른다 (틀이 2개 이상일 때).
    fn pick_story(&mut self, kind: MathKind) -> usize {
        let count = kind.stories().len();
        match self.last {
            Some((last_kind, last_story)) if last_kind == kind && count > 1 => {
                let pick = self.pick(count - 1);
                if pick >= last_story {
                    pick + 1
                } else {
                    pick
                }
            }
            _ => self.pick(count),
        }
    }

    /// 0 ~ `count - 1` 중 하나.
    fn pick(&mut self, count: usize) -> usize {
        ((self.random.next_unit() * count as f32) as usize).min(count.saturating_sub(1))
    }

    /// `low` ~ `high` (둘 다 포함) 중 하나.
    fn pick_between(&mut self, low: u32, high: u32) -> u32 {
        low + self.pick((high - low + 1) as usize) as u32
    }
}

/// 확인을 누른 뒤의 결과.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathResult {
    /// 맞혔다. `MATH_RIGHT_SHOW_SECONDS` 뒤에 닫힌다.
    Right,
    /// 틀렸지만 기회가 남았다. 숫자를 다시 넣으면 사라진다. 시간 제한은 없다.
    TryAgain,
    /// 기회를 다 썼다. 정답을 `MATH_WRONG_SHOW_SECONDS` 보여 주고 닫힐 때 체력이 준다.
    Wrong,
}

/// 문제 창 하나의 상태. 열려 있는 동안(결과 글 시간 포함) 게임 속 시간이 멈춘다.
pub struct MathQuiz {
    maker: MathQuestionMaker,
    question: Option<MathQuestion>,
    typed: String,
    wrong_count: u32,
    result: Option<MathResult>,
    result_seconds_left: f32,
    /// 열기를 기다리는 문제 수.
    waiting: u32,
}

impl MathQuiz {
    pub fn new(game_seed: u32) -> MathQuiz {
        MathQuiz::with_maker(MathQuestionMaker::new(game_seed))
    }

    pub fn with_maker(maker: MathQuestionMaker) -> MathQuiz {
        MathQuiz {
            maker,
            question: None,
            typed: String::new(),
            wrong_count: 0,
            result: None,
            result_seconds_left: 0.0,
            waiting: 0,
        }
    }

    /// 지금 열려 있는 문제. 닫혀 있으면 None.
    pub fn question(&self) -> Option<&MathQuestion> {
        self.question.as_ref()
    }

    pub fn is_open(&self) -> bool {
        self.question.is_some()
    }

    /// 지금까지 넣은 숫자 (예: "12"). 비어 있을 수 있다.
    pub fn typed(&self) -> &str {
        &self.typed
    }

    /// 남은 기회 (처음 `MATH_TRIES`).
    pub fn tries_left(&self) -> u32 {
        MATH_TRIES - self.wrong_count
    }

    /// 남은 기회 글: "기회가 2번 남았어". 틀린 뒤 "다시 해 볼까?"와 함께 보여 준다.
    pub fn tries_left_text(&self) -> String {
        MATH_TRIES_LEFT_TEXT.replace('□', &self.tries_left().to_string())
    }

    /// 마지막으로 확인한 결과. 아직 확인 안 했거나, 다시 해 보려고 숫자를 넣기 시작했으면 None.
    pub fn result(&self) -> Option<MathResult> {
        self.result
    }

    /// 맞힘·정답 글이 남은 실제 초. 0이 되면 창이 닫힌다.
    pub fn result_seconds_left(&self) -> f32 {
        self.result_seconds_left
    }

    /// 결과 글: "넌 최고야!", "너무 아쉽다! 다시 해 볼까?", "너무 아쉽다! 정답은 362야".
    pub fn result_text(&self) -> Option<String> {
        let answer = self.question.as_ref()?.answer;
        Some(match self.result? {
            MathResult::Right => MATH_RIGHT_TEXT.to_string(),
            MathResult::TryAgain => MATH_TRY_AGAIN_TEXT.to_string(),
            MathResult::Wrong => MATH_WRONG_TEXT.replace("□이야", &format!("{answer}{}", iya_after(answer))),
        })
    }

    /// 숫자판에서 0~9를 눌렀다. `MATH_ANSWER_DIGITS`자리까지 넣는다. 맨 앞 0 뒤에 숫자를 넣으면 0을 바꾼다.
    pub fn press_digit(&mut self, digit: u32) {
        if !self.can_type() || digit > 9 {
            return;
        }
        self.result = None; // "다시 해 볼까?" 글은 다시 넣기 시작하면 사라진다
        if self.typed == "0" {
            self.typed.clear();
        }
        if self.typed.len() < MATH_ANSWER_DIGITS {
            self.typed.push(char::from(b'0' + digit as u8));
        }
    }

    /// 지우기: 마지막 숫자 하나를 지운다.
    pub fn erase(&mut self) {
        if self.can_type() {
            self.typed.pop();
        }
    }

    /// 확인: 넣은 수가 정답인지 본다. 숫자가 없거나 결과 글을 보여 주는 중이면 None.
    /// 틀린 횟수가 `MATH_TRIES`가 되면 `Wrong`이고, 체력은 정답 글이 닫힐 때 `Game`이 줄인다.
    pub fn submit(&mut self) -> Option<MathResult> {
        if !self.can_type() {
            return None;
        }
        let typed: u32 = self.typed.parse().ok()?;
        let answer = self.question.as_ref()?.answer;
        self.typed.clear();
        let result = if typed == answer {
            self.result_seconds_left = MATH_RIGHT_SHOW_SECONDS;
            MathResult::Right
        } else {
            self.wrong_count += 1;
            if self.wrong_count >= MATH_TRIES {
                self.result_seconds_left = MATH_WRONG_SHOW_SECONDS;
                MathResult::Wrong
            } else {
                MathResult::TryAgain
            }
        };
        self.result = Some(result);
        Some(result)
    }

    /// 시험용: 수를 숫자판으로 넣고 확인한다.
    #[cfg(test)]
    pub fn type_and_submit(&mut self, number: u32) -> Option<MathResult> {
        for digit in number.to_string().chars() {
            self.press_digit(digit.to_digit(10).unwrap_or(0));
        }
        self.submit()
    }

    /// 열려 있고 맞힘·정답 글을 보여 주는 중이 아니면 숫자를 넣을 수 있다.
    fn can_type(&self) -> bool {
        self.is_open() && !matches!(self.result, Some(MathResult::Right | MathResult::Wrong))
    }

    /// 문제 하나를 낼 차례로 넣는다. 같은 순간 두 가지(무기와 밤)가 겹치면 하나를 닫은 뒤에 다음 것이 열린다.
    pub(crate) fn add_waiting(&mut self) {
        self.waiting += 1;
    }

    /// 닫혀 있고 기다리는 문제가 있으면 연다.
    pub(crate) fn open_next_waiting(&mut self) {
        if self.is_open() || self.waiting == 0 {
            return;
        }
        self.waiting -= 1;
        self.question = self.maker.next();
        self.typed.clear();
        self.wrong_count = 0;
        self.result = None;
    }

    /// 맞힘·정답 글 시간을 실제 `dt`만큼 흘리고, 다 되면 창을 닫는다. 그 전에는 시간 제한이 없다.
    /// 돌려주는 값: 이번에 닫힌 문제의 결과 (`Right` 또는 `Wrong`).
    pub fn wait_after_answer(&mut self, dt: f32) -> Option<MathResult> {
        let result = self.result.filter(|r| *r != MathResult::TryAgain)?;
        self.result_seconds_left = (self.result_seconds_left - dt.max(0.0)).max(0.0);
        if self.result_seconds_left > 0.0 {
            return None;
        }
        self.question = None;
        self.result = None;
        Some(result)
    }
}

/// 문장 틀에 이름과 수를 넣는다. {이름:이}는 "로건이"/"니로가", {이름:은}은 "밥은"/"미스는", {이름:을}은 "케빈을"/"엘리를".
pub fn fill_story(story: &str, name: &str, numbers: [u32; 3]) -> String {
    story
        .replace("{이름:이}", &with_particle(name, "이", "가"))
        .replace("{이름:은}", &with_particle(name, "은", "는"))
        .replace("{이름:을}", &with_particle(name, "을", "를"))
        .replace("{a}", &numbers[0].to_string())
        .replace("{b}", &numbers[1].to_string())
        .replace("{c}", &numbers[2].to_string())
}

/// 이름 끝 글자에 받침이 있으면 `after_batchim`(이·은·을), 없으면 `no_batchim`(가·는·를)을 붙인다.
pub fn with_particle(name: &str, after_batchim: &str, no_batchim: &str) -> String {
    let has_batchim = name
        .chars()
        .last()
        .map(|c| ('가'..='힣').contains(&c) && !(c as u32 - '가' as u32).is_multiple_of(28))
        .unwrap_or(false);
    format!("{name}{}", if has_batchim { after_batchim } else { no_batchim })
}

/// 수 뒤에 붙는 말: 받침 없는 소리(이, 사, 오, 구)로 끝나면 "야", 아니면 "이야". 10, 20 …은 "십", 100은 "백".
fn iya_after(number: u32) -> &'static str {
    if matches!(number % 10, 2 | 4 | 5 | 9) {
        "야"
    } else {
        "이야"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_quiz(seed: u32) -> MathQuiz {
        let mut quiz = MathQuiz::new(seed);
        quiz.add_waiting();
        quiz.open_next_waiting();
        quiz
    }

    #[test]
    fn particle_follows_the_last_sound() {
        assert_eq!([2, 13, 10, 45, 100, 7, 362].map(iya_after), ["야", "이야", "이야", "야", "이야", "이야", "야"]);
    }

    #[test]
    fn no_kinds_means_no_question() {
        let mut quiz = MathQuiz::with_maker(MathQuestionMaker::with_kinds(1, &[]));
        quiz.add_waiting();
        quiz.open_next_waiting();
        assert!(!quiz.is_open());
    }

    #[test]
    fn typing_keeps_three_digits_and_erase_removes_last() {
        let mut quiz = open_quiz(5);
        for d in [0, 1, 2, 3, 4] {
            quiz.press_digit(d);
        }
        assert_eq!(quiz.typed(), "123", "맨 앞 0은 바뀌고 세 자리까지");
        quiz.erase();
        assert_eq!(quiz.typed(), "12");
        quiz.press_digit(10);
        assert_eq!(quiz.typed(), "12", "0~9만");
    }

    #[test]
    fn submit_with_nothing_typed_does_nothing() {
        let mut quiz = open_quiz(5);
        assert_eq!(quiz.submit(), None);
        assert_eq!(quiz.tries_left(), MATH_TRIES);
    }

    #[test]
    fn three_wrong_tries_show_the_answer() {
        let mut quiz = open_quiz(5);
        let answer = quiz.question().unwrap().answer;
        let wrong = if answer == 0 { 1 } else { answer - 1 };
        assert_eq!(quiz.type_and_submit(wrong), Some(MathResult::TryAgain));
        assert_eq!(quiz.result_text().unwrap(), "너무 아쉽다! 다시 해 볼까?");
        assert_eq!(quiz.type_and_submit(wrong), Some(MathResult::TryAgain));
        assert_eq!(quiz.tries_left(), 1);
        assert_eq!(quiz.tries_left_text(), "기회가 1번 남았어");
        assert_eq!(quiz.type_and_submit(wrong), Some(MathResult::Wrong));
        let text = quiz.result_text().unwrap();
        assert!(text.starts_with(&format!("너무 아쉽다! 정답은 {answer}")), "{text}");
        assert_eq!(quiz.type_and_submit(answer), None, "정답 글이 나온 뒤에는 넣을 수 없다");
        assert_eq!(quiz.wait_after_answer(MATH_WRONG_SHOW_SECONDS - 0.1), None);
        assert_eq!(quiz.wait_after_answer(0.2), Some(MathResult::Wrong));
        assert!(!quiz.is_open());
    }

    #[test]
    fn right_after_a_miss_is_still_right() {
        let mut quiz = open_quiz(9);
        let answer = quiz.question().unwrap().answer;
        assert_eq!(quiz.type_and_submit(answer + 1), Some(MathResult::TryAgain));
        assert_eq!(quiz.type_and_submit(answer), Some(MathResult::Right));
        assert_eq!(quiz.result_text().unwrap(), MATH_RIGHT_TEXT);
        assert_eq!(quiz.wait_after_answer(MATH_RIGHT_SHOW_SECONDS), Some(MathResult::Right));
    }
}
