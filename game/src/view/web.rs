//! 웹페이지(web/index.html)와 주고받기.
//! 브라우저에서는 index.html이 `loading_done`, `game_version_byte`, `best_days_load`, `best_days_save`,
//! `best_score_load`, `best_score_save`, 그리고 설정 저장 함수들(맨 아래)을 넣어 준다.
//! PC(네이티브)에서는 할 일이 없다.

use crate::config::*;

use crate::logic::settings::{settings_cleared, settings_from_text, settings_to_text, Settings};
#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    fn loading_done();
    /// 버전 글자(UTF-8)의 i번째 바이트. 끝을 지나거나 버전이 없으면 -1.
    fn game_version_byte(i: u32) -> i32;
    /// localStorage의 최고 기록. 없거나 못 읽으면 0.
    fn best_days_load() -> i32;
    fn best_days_save(days: u32);
    /// localStorage의 최고 점수. 없거나 못 읽으면 0.
    fn best_score_load() -> i32;
    fn best_score_save(score: u32);
    /// 주소에 `?qa`가 있으면 1, 아니면 0 (시험용 단축).
    fn qa_mode_on() -> i32;
}

/// 웹페이지의 "Now Loading..." 화면(도는 점 + 글자)을 치운다. 게임이 첫 화면을 그린 뒤에 한 번 부른다.
pub fn hide_loading_screen() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        loading_done();
    }
}

/// 저장된 최고 기록 (버틴 날, US-044). PC(네이티브)는 저장하지 않아서 늘 0.
pub fn load_best_days() -> u32 {
    #[cfg(target_arch = "wasm32")]
    {
        unsafe { best_days_load() }.max(0) as u32
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

/// 최고 기록을 브라우저에 저장한다. PC(네이티브)에서는 아무 일도 안 한다.
pub fn save_best_days(days: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        best_days_save(days);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = days;
}

/// 저장된 최고 점수 (US-069). PC(네이티브)는 저장하지 않아서 늘 0.
pub fn load_best_score() -> u32 {
    #[cfg(target_arch = "wasm32")]
    {
        unsafe { best_score_load() }.max(0) as u32
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

/// 최고 점수를 브라우저에 저장한다. PC(네이티브)에서는 아무 일도 안 한다.
pub fn save_best_score(score: u32) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        best_score_save(score);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = score;
}

/// 배포 버전 (US-031). `deploy/build.sh`가 `dist/index.html`에 넣는 `window.GAME_VERSION`.
/// 로컬 `cargo run`이나 버전 없이 연 페이지에서는 None (타이틀에 아무것도 안 그린다).
pub fn game_version() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        const MAX_LEN: u32 = 64;
        let mut bytes = Vec::new();
        for i in 0..MAX_LEN {
            let b = unsafe { game_version_byte(i) };
            if !(0..=255).contains(&b) {
                break;
            }
            bytes.push(b as u8);
        }
        String::from_utf8(bytes).ok().filter(|v| !v.is_empty())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

/// 시험용 단축을 켤지 (view::qa). 브라우저는 주소에 `?qa`가 있을 때만, PC 실행은 `cargo run -- qa`일 때만 true.
pub fn qa_mode() -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        unsafe { qa_mode_on() == 1 }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::env::args().skip(1).any(|a| a == "qa")
    }
}

// ---- 설정 저장 (US-125) ----
// 한계: 저장은 이 브라우저 안에만 있다. 다른 기기, 시크릿 창, 브라우저 데이터를 지우면 사라진다.
// 브라우저(index.html)는 `settings_load_byte`, `settings_save_begin`, `settings_save_byte`, `settings_save_end`,
// `settings_clear`를 넣어 준다. 글자는 바이트 하나씩 주고받는다 (game_version_byte와 같은 방식).

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// 저장 글자(UTF-8)의 i번째 바이트. 끝을 지나거나 없거나 못 읽으면 -1.
    fn settings_load_byte(i: u32) -> i32;
    fn settings_save_begin();
    fn settings_save_byte(b: u32);
    /// 모은 바이트를 새 키에 쓴다. 실패하면 조용히 넘어간다.
    fn settings_save_end();
    fn settings_clear();
}

/// 바이트들을 저장 글자로 바꾼다. 너무 길거나 UTF-8이 깨졌으면 빈 글자(= 기본값).
fn settings_text_from_bytes(bytes: &[u8]) -> String {
    match String::from_utf8(bytes.to_vec()) {
        Ok(t) if t.chars().count() <= SETTINGS_MAX_CHARS => t,
        _ => String::new(),
    }
}

/// 시작할 때 한 번 읽는다. `?qa`이거나 PC(네이티브)이거나 저장이 없거나 이상하면 기본값.
pub fn load_settings() -> Settings {
    #[cfg(target_arch = "wasm32")]
    {
        if qa_mode() {
            return Settings::default();
        }
        // 글자 수 제한(SETTINGS_MAX_CHARS)은 한 글자가 최대 4바이트라서 바이트는 4배까지만 받는다.
        let mut bytes = Vec::new();
        for i in 0..(SETTINGS_MAX_CHARS as u32 * 4) {
            let b = unsafe { settings_load_byte(i) };
            if !(0..=255).contains(&b) {
                break;
            }
            bytes.push(b as u8);
        }
        settings_from_text(&settings_text_from_bytes(&bytes))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Settings::default()
    }
}

/// 값이 바뀔 때만 부른다 (매 프레임 X). `?qa`와 PC에서는 아무 일도 안 한다.
#[allow(dead_code)] // 튜토리얼·소리 스토리(US-112, US-119~121)가 쓴다
pub fn save_settings(settings: &Settings) {
    #[cfg(target_arch = "wasm32")]
    {
        if qa_mode() {
            return;
        }
        let text = settings_to_text(settings);
        unsafe {
            settings_save_begin();
            for b in text.bytes() {
                settings_save_byte(b as u32);
            }
            settings_save_end();
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = settings;
}

/// 저장을 지우고 기본값을 돌려준다. 화면 버튼은 아직 없다 ([임시-139] 부모님 결정 대기).
#[allow(dead_code)]
pub fn clear_settings() -> Settings {
    #[cfg(target_arch = "wasm32")]
    if !qa_mode() {
        unsafe { settings_clear() };
    }
    settings_cleared()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytes_become_text_and_bad_ones_become_empty() {
        assert_eq!(settings_text_from_bytes(b"1;tutorial=1"), "1;tutorial=1");
        assert_eq!(settings_text_from_bytes(&[0xff, 0xfe]), "");
        assert_eq!(settings_text_from_bytes(&vec![b'a'; SETTINGS_MAX_CHARS + 1]), "");
    }

    #[test]
    fn native_has_no_storage() {
        let mut s = load_settings();
        assert_eq!(s, Settings::default());
        s.tutorial_done = true;
        save_settings(&s); // 아무 일도 없어야 한다
        assert_eq!(load_settings(), Settings::default());
        assert_eq!(clear_settings(), Settings::default());
    }
}
