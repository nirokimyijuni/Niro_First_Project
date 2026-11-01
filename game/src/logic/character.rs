//! 고를 수 있는 캐릭터 3명.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterKind {
    Niro,
    Ellie,
    Logan,
}

impl CharacterKind {
    /// 캐릭터 선택 화면에 보여 줄 순서.
    pub const ALL: [CharacterKind; 3] = [CharacterKind::Niro, CharacterKind::Ellie, CharacterKind::Logan];

    /// 영어 이름 (한글 폰트가 들어오기 전까지 화면에 쓴다).
    pub fn name(self) -> &'static str {
        match self {
            CharacterKind::Niro => "Niro",
            CharacterKind::Ellie => "Ellie",
            CharacterKind::Logan => "Logan",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_characters_with_names() {
        let names: Vec<&str> = CharacterKind::ALL.iter().map(|c| c.name()).collect();
        assert_eq!(names, ["Niro", "Ellie", "Logan"]);
    }
}
