//! 미스의 영어 낱말 문제 (D82·D106, US-087·088). 문제 만들기만 한다.
//! 창 열기, 고르기, 기회, 결과 글은 수학과 같은 창(`MathQuiz`)이 한다. 그림은 다루지 않고 이름만 넘긴다.

use super::monster::SeededRandom;
use crate::config::{
    ENGLISH_CHOICE_COUNT, ENGLISH_CONFUSING_GROUPS, ENGLISH_PICTURE_TO_WORD_ASK_TEXT, ENGLISH_SEED_MIX,
    ENGLISH_WORDS, ENGLISH_WORD_TO_PICTURE_ASK_TEXT,
};

/// 문제 종류 ([임시-57]). 둘은 똑같은 확률로 나온다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnglishKind {
    /// (가) 그림 하나를 보고 영어 낱말 4개 중 고르기. "이 그림은 영어로 뭘까?"
    PictureToWord,
    /// (나) 영어 낱말 하나를 보고 그림 4개 중 고르기. "이 낱말에 맞는 그림은?"
    WordToPicture,
}

impl EnglishKind {
    /// 묻는 글 (config.rs).
    pub fn ask_text(self) -> &'static str {
        match self {
            EnglishKind::PictureToWord => ENGLISH_PICTURE_TO_WORD_ASK_TEXT,
            EnglishKind::WordToPicture => ENGLISH_WORD_TO_PICTURE_ASK_TEXT,
        }
    }
}

/// 낱말 하나와 그 그림 이름 (config.rs `ENGLISH_WORDS`의 한 줄).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnglishWord {
    /// 앞 글자만 대문자 (예: "Kite")
    pub word: &'static str,
    /// 그림 파일 이름 (예: "en-kite.png"). Tree는 `ENGLISH_GAME_TREE_PICTURE`.
    pub picture: &'static str,
}

/// 문제 하나: 보기 4개와 정답 자리.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnglishQuestion {
    pub kind: EnglishKind,
    /// 2×2 칸에 놓을 보기. 0이 1번(왼쪽 위), 3이 4번(오른쪽 아래).
    pub choices: [EnglishWord; ENGLISH_CHOICE_COUNT],
    /// 정답 보기의 자리 (0~3)
    pub answer: usize,
}

impl EnglishQuestion {
    /// 정답 낱말. (가)는 이 그림을 위에, (나)는 이 낱말을 위에 보여 준다.
    pub fn answer_word(&self) -> EnglishWord {
        self.choices[self.answer]
    }
}

/// 같은 시드면 같은 문제를 같은 순서로 낸다. 바로 앞 문제의 정답 낱말은 다시 정답으로 내지 않는다.
pub struct EnglishQuestionMaker {
    random: SeededRandom,
    words: &'static [(&'static str, &'static str)],
    confusing_groups: &'static [&'static [&'static str]],
    last_answer: Option<&'static str>,
}

impl EnglishQuestionMaker {
    /// config.rs의 낱말 표와 헷갈리는 묶음으로 낸다. 다른 뽑기와 따로 굴린다.
    pub fn new(game_seed: u32) -> EnglishQuestionMaker {
        EnglishQuestionMaker::with_words(game_seed, ENGLISH_WORDS, ENGLISH_CONFUSING_GROUPS)
    }

    /// 고른 낱말 표로 낸다. 시험이 낱말을 더하고 뺄 때 쓴다.
    pub fn with_words(
        game_seed: u32,
        words: &'static [(&'static str, &'static str)],
        confusing_groups: &'static [&'static [&'static str]],
    ) -> EnglishQuestionMaker {
        EnglishQuestionMaker {
            random: SeededRandom::new(game_seed ^ ENGLISH_SEED_MIX),
            words,
            confusing_groups,
            last_answer: None,
        }
    }

    /// 다음 문제. 보기 4개를 채울 낱말이 모자라면 None (그때는 수학 문제를 낸다).
    pub fn next(&mut self) -> Option<EnglishQuestion> {
        let kind = if self.pick(2) == 0 { EnglishKind::PictureToWord } else { EnglishKind::WordToPicture };

        // 정답: 바로 앞 문제의 정답 낱말은 뺀다.
        let last = self.last_answer;
        let can_be_answer: Vec<usize> = (0..self.words.len()).filter(|&i| Some(self.words[i].0) != last).collect();
        if can_be_answer.is_empty() {
            return None;
        }
        let answer_index = can_be_answer[self.pick(can_be_answer.len())];

        // 틀린 답 3개: 이미 고른 낱말과 같거나 헷갈리는 낱말은 뺀다.
        let mut picked = vec![answer_index];
        for _ in 1..ENGLISH_CHOICE_COUNT {
            let can_pick: Vec<usize> = (0..self.words.len())
                .filter(|&i| picked.iter().all(|&p| p != i && !self.are_confusing(self.words[p].0, self.words[i].0)))
                .collect();
            if can_pick.is_empty() {
                return None;
            }
            picked.push(can_pick[self.pick(can_pick.len())]);
        }

        // 4개의 자리를 섞는다: 정답 자리를 고르고, 남은 자리에 틀린 답을 뽑은 차례대로 놓는다.
        let answer = self.pick(ENGLISH_CHOICE_COUNT);
        let mut wrong = picked[1..].iter();
        let choices: [EnglishWord; ENGLISH_CHOICE_COUNT] = std::array::from_fn(|slot| {
            let index = if slot == answer { answer_index } else { *wrong.next().unwrap_or(&answer_index) };
            EnglishWord { word: self.words[index].0, picture: self.words[index].1 }
        });
        self.last_answer = Some(self.words[answer_index].0);
        Some(EnglishQuestion { kind, choices, answer })
    }

    /// 두 낱말이 같은 헷갈리는 묶음에 들어 있는지.
    fn are_confusing(&self, a: &str, b: &str) -> bool {
        self.confusing_groups.iter().any(|group| group.contains(&a) && group.contains(&b))
    }

    /// 0 ~ `count - 1` 중 하나.
    fn pick(&mut self, count: usize) -> usize {
        ((self.random.next_unit() * count as f32) as usize).min(count.saturating_sub(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ENGLISH_GAME_TREE_PICTURE;

    #[test]
    fn word_list_is_48_words_with_only_first_letter_big() {
        assert_eq!(ENGLISH_WORDS.len(), 48);
        for (word, _) in ENGLISH_WORDS {
            let mut letters = word.chars();
            assert!(letters.next().is_some_and(|c| c.is_ascii_uppercase()), "{word}");
            assert!(letters.all(|c| c.is_ascii_lowercase()), "{word}");
        }
    }

    #[test]
    fn words_are_all_different_and_pictures_follow_the_name_rule() {
        for (i, (word, picture)) in ENGLISH_WORDS.iter().enumerate() {
            assert!(ENGLISH_WORDS[..i].iter().all(|(w, _)| w != word), "{word}가 두 번 있어요");
            if *word == "Tree" {
                assert_eq!(*picture, ENGLISH_GAME_TREE_PICTURE);
            } else {
                assert_eq!(*picture, format!("en-{}.png", word.to_lowercase()));
            }
        }
    }

    #[test]
    fn confusing_groups_use_only_words_in_the_list() {
        for group in ENGLISH_CONFUSING_GROUPS {
            for word in *group {
                assert!(ENGLISH_WORDS.iter().any(|(w, _)| w == word), "묶음의 {word}가 낱말 표에 없어요");
            }
        }
    }

    #[test]
    fn too_few_words_means_no_question() {
        static THREE: &[(&str, &str)] = &[("Fan", "en-fan.png"), ("Hen", "en-hen.png"), ("Net", "en-net.png")];
        assert_eq!(EnglishQuestionMaker::with_words(1, THREE, &[]).next(), None);
    }
}
