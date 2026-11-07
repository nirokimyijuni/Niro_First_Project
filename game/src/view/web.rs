//! 웹페이지(web/index.html)와 주고받기.
//! 브라우저에서는 index.html이 `loading_done`, `game_version_byte`, `best_days_load`, `best_days_save`,
//! `best_score_load`, `best_score_save` 함수를 넣어 준다.
//! PC(네이티브)에서는 할 일이 없다.

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
}

/// "불러오는 중..." 글자를 치운다. 게임이 첫 화면을 그린 뒤에 한 번 부른다.
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
