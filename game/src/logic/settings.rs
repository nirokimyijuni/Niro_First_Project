//! 설정 저장 (US-125). 글자로 만들고 읽는 순수 함수만 있다. 브라우저·macroquad를 모른다.
//!
//! 글자 모양 (손으로 쓴 간단한 모양, 새 의존성 없음):
//! `1;tutorial=1;tip=0;sound=1;music=70;effects=80`
//! - 맨 앞이 버전 번호. 모르는 버전이면 통째로 새 게임 값.
//! - 한 항목만 이상하면 그 항목만 기본값. 모르는 항목은 무시, 빠진 항목은 기본값.
//! - 이름·주소·나이 같은 개인 정보는 없다 ("봤다" 표시와 소리 값뿐).

use crate::config::*;

/// 저장하는 설정 값들.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Settings {
    /// 튜토리얼 말풍선을 다 봤거나 건너뛰었다
    pub tutorial_done: bool,
    /// 창 한 줄 안내를 봤다
    pub window_tip_seen: bool,
    /// 소리 켜기(true)/끄기(false). 소리 기능은 US-112에서, 여기는 값 자리만
    pub sound_on: bool,
    /// 음악 크기 0~100
    pub music_percent: u8,
    /// 효과음 크기 0~100
    pub effects_percent: u8,
}

impl Default for Settings {
    /// 새 게임 값.
    fn default() -> Self {
        Settings {
            tutorial_done: false,
            window_tip_seen: false,
            sound_on: SETTINGS_DEFAULT_SOUND_ON,
            music_percent: SETTINGS_DEFAULT_MUSIC_PERCENT,
            effects_percent: SETTINGS_DEFAULT_EFFECTS_PERCENT,
        }
    }
}

/// 설정을 저장 글자로 만든다.
pub fn settings_to_text(s: &Settings) -> String {
    let items: [(&str, u32); 5] = [
        (SETTINGS_KEY_TUTORIAL_DONE, s.tutorial_done as u32),
        (SETTINGS_KEY_WINDOW_TIP_SEEN, s.window_tip_seen as u32),
        (SETTINGS_KEY_SOUND_ON, s.sound_on as u32),
        (SETTINGS_KEY_MUSIC_PERCENT, s.music_percent as u32),
        (SETTINGS_KEY_EFFECTS_PERCENT, s.effects_percent as u32),
    ];
    let mut text = SETTINGS_VERSION.to_string();
    for (key, value) in items {
        text.push(SETTINGS_ITEM_SEPARATOR);
        text.push_str(key);
        text.push(SETTINGS_VALUE_SEPARATOR);
        text.push_str(&value.to_string());
    }
    text
}

/// 저장 글자를 읽는다. 이상하면 조용히 기본값(새 게임). 절대 패닉하지 않는다.
pub fn settings_from_text(text: &str) -> Settings {
    let mut s = Settings::default();
    let text = text.trim();
    if text.is_empty() || text.chars().count() > SETTINGS_MAX_CHARS {
        return s;
    }
    let mut parts = text.split(SETTINGS_ITEM_SEPARATOR);
    // 맨 앞은 버전 번호. 우리가 아는 버전이 아니면 새 게임 값.
    let version = parts.next().and_then(|p| p.trim().parse::<u32>().ok());
    if version != Some(SETTINGS_VERSION) {
        return s;
    }
    // 같은 이름이 또 나와도 첫 번째만 쓴다. 항목 수는 글자 길이로 이미 제한된다.
    let mut seen: Vec<&str> = Vec::new();
    for part in parts {
        let Some((key, value)) = part.split_once(SETTINGS_VALUE_SEPARATOR) else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);
        match key {
            SETTINGS_KEY_TUTORIAL_DONE => {
                if let Some(b) = parse_flag(value) {
                    s.tutorial_done = b;
                }
            }
            SETTINGS_KEY_WINDOW_TIP_SEEN => {
                if let Some(b) = parse_flag(value) {
                    s.window_tip_seen = b;
                }
            }
            SETTINGS_KEY_SOUND_ON => {
                if let Some(b) = parse_flag(value) {
                    s.sound_on = b;
                }
            }
            SETTINGS_KEY_MUSIC_PERCENT => {
                if let Some(p) = parse_percent(value) {
                    s.music_percent = p;
                }
            }
            SETTINGS_KEY_EFFECTS_PERCENT => {
                if let Some(p) = parse_percent(value) {
                    s.effects_percent = p;
                }
            }
            _ => {} // 모르는 항목(나중 버전이 만든 것)은 무시
        }
    }
    s
}

/// 저장을 지운 뒤의 값 (기본값으로 되돌림). 화면이 이 값을 쓰고 저장한다.
pub fn settings_cleared() -> Settings {
    Settings::default()
}

fn parse_flag(value: &str) -> Option<bool> {
    match value {
        "0" => Some(false),
        "1" => Some(true),
        _ => None,
    }
}

fn parse_percent(value: &str) -> Option<u8> {
    let n: u8 = value.parse().ok()?; // 음수·소수·너무 큰 수는 None
    (n <= SETTINGS_VOLUME_MAX_PERCENT).then_some(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn changed() -> Settings {
        Settings {
            tutorial_done: true,
            window_tip_seen: true,
            sound_on: false,
            music_percent: 30,
            effects_percent: 100,
        }
    }

    #[test]
    fn 만들고_다시_읽으면_같다() {
        for s in [Settings::default(), changed()] {
            assert_eq!(settings_from_text(&settings_to_text(&s)), s);
        }
    }

    #[test]
    fn 글자_맨_앞은_버전_번호다() {
        let t = settings_to_text(&changed());
        assert!(t.starts_with(&format!("{}{}", SETTINGS_VERSION, SETTINGS_ITEM_SEPARATOR)));
    }

    #[test]
    fn 빈_값은_기본값() {
        assert_eq!(settings_from_text(""), Settings::default());
        assert_eq!(settings_from_text("   "), Settings::default());
    }

    #[test]
    fn 깨진_글은_기본값이고_패닉하지_않는다() {
        for t in ["asdf", ";;;", "=", "1;=;=", "\u{0}\u{ffff}한글", "1;tutorial", "{\"a\":1}", "-1;tutorial=1"] {
            assert_eq!(settings_from_text(t), Settings::default(), "{t}");
        }
    }

    #[test]
    fn 모르는_버전은_통째로_기본값() {
        for v in ["0", "2", "99", "4294967295", "99999999999"] {
            let t = format!("{v};tutorial=1;sound=0;music=10");
            assert_eq!(settings_from_text(&t), Settings::default(), "{t}");
        }
    }

    #[test]
    fn 범위_밖_숫자는_그_항목만_기본값() {
        let s = settings_from_text("1;tutorial=1;sound=0;music=101;effects=-5");
        assert!(s.tutorial_done && !s.sound_on); // 멀쩡한 항목은 살아 있다
        assert_eq!(s.music_percent, SETTINGS_DEFAULT_MUSIC_PERCENT);
        assert_eq!(s.effects_percent, SETTINGS_DEFAULT_EFFECTS_PERCENT);
        for bad in ["1;music=0.5", "1;music=abc", "1;music=256", "1;music=", "1;sound=2", "1;tutorial=yes"] {
            assert_eq!(settings_from_text(bad), Settings::default(), "{bad}");
        }
    }

    #[test]
    fn 너무_긴_글은_기본값() {
        let t = format!("1;tutorial=1;{}", "x".repeat(SETTINGS_MAX_CHARS + 1));
        assert_eq!(settings_from_text(&t), Settings::default());
    }

    #[test]
    fn 버전1_글자는_모르는_항목과_빠진_항목이_있어도_읽힌다() {
        // 나중 버전이 항목을 더해도 옛 항목은 그대로 읽힌다. 빠진 항목은 기본값.
        let s = settings_from_text("1;tutorial=1;newthing=7;music=40;future=;noequal");
        assert!(s.tutorial_done);
        assert_eq!(s.music_percent, 40);
        assert_eq!(s.effects_percent, SETTINGS_DEFAULT_EFFECTS_PERCENT);
        assert!(!s.window_tip_seen);
        assert_eq!(settings_from_text("1"), Settings::default());
    }

    #[test]
    fn 같은_이름이_또_나오면_첫_번째만() {
        assert_eq!(settings_from_text("1;music=10;music=90").music_percent, 10);
    }

    #[test]
    fn 지우면_기본값으로_돌아간다() {
        assert_eq!(settings_cleared(), Settings::default());
        assert_eq!(settings_from_text(&settings_to_text(&settings_cleared())), Settings::default());
    }

    #[test]
    fn 저장_글자에_개인_정보_항목이_없다() {
        let t = settings_to_text(&changed());
        let allowed = [
            SETTINGS_KEY_TUTORIAL_DONE,
            SETTINGS_KEY_WINDOW_TIP_SEEN,
            SETTINGS_KEY_SOUND_ON,
            SETTINGS_KEY_MUSIC_PERCENT,
            SETTINGS_KEY_EFFECTS_PERCENT,
        ];
        for part in t.split(SETTINGS_ITEM_SEPARATOR).skip(1) {
            let key = part.split(SETTINGS_VALUE_SEPARATOR).next().unwrap();
            assert!(allowed.contains(&key), "{key}");
        }
        assert!(t.is_ascii() && t.chars().count() <= SETTINGS_MAX_CHARS);
    }
}
