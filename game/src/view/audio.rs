//! 소리 내기 (로건, US-112~117). 어떤 소리를 언제 얼마나 크게 낼지는 `logic::sound`가 정하고,
//! 여기서는 파일 받기, 재생, 음악 섞기, 소리 켜기/끄기 그림만 한다.
//! - 소리 파일은 "눌러서 시작"을 누른 뒤에 뒤에서 하나씩 받는다 (`Audio::start_loading`). 못 받으면 그 소리만 안 난다.
//! - 브라우저는 첫 누름 뒤에만 소리를 낸다. 그 "풀기"는 web/index.html이 한다 (아이폰은 손가락을 뗄 때).
//! - 이 파일의 "순수 함수·구조체"(`MusicFader`, `WindowSounds`, `SoundToast`, `volume_cell_for`)는 단위 시험이 있다.

use super::hud::draw_pill;
use super::screen::VIRTUAL_WIDTH;
use super::text::TextPainter;
use crate::config::*;
use crate::logic::sound::{loading_order, Sfx, Track};
use macroquad::audio::{load_sound_from_bytes, play_sound, set_sound_volume, stop_sound, PlaySoundParams, Sound};
use macroquad::experimental::coroutines::{start_coroutine, Coroutine};
use macroquad::file::load_file;
use macroquad::prelude::*;
use std::sync::{Arc, Mutex, MutexGuard};
use std::collections::HashMap;
use std::future::Future;
use std::task::{Context, Poll, Waker};

// ---------------------------------------------------------------- 음악 섞기 (순수)

#[derive(Debug, Clone, PartialEq)]
struct MusicLayer {
    path: String,
    /// 0~1 (이 곡의 섞기 정도. 실제 크기는 여기에 음악 크기를 곱한다)
    level: f32,
}

/// 한 프레임에 할 일: 새로 틀 곡, 멈출 곡, 지금 곡들의 섞기 정도.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct FadeStep {
    pub start: Vec<String>,
    pub stop: Vec<String>,
    pub levels: Vec<(String, f32)>,
}

/// 음악 섞기: 새 곡이 `fade_seconds` 동안 커지고 앞 곡은 그동안 작아진다. 새 곡이 아직 안 받아졌으면 지금 곡을 그대로 둔다.
#[derive(Debug, Default)]
pub struct MusicFader {
    target: Option<String>,
    layers: Vec<MusicLayer>,
}

impl MusicFader {
    pub fn new() -> MusicFader {
        MusicFader::default()
    }

    /// 틀고 싶은 곡 (None이면 음악을 멈춘다: 천천히 작아진다).
    pub fn set_target(&mut self, path: Option<String>) {
        self.target = path;
    }

    #[cfg(test)]
    pub fn is_idle(&self) -> bool {
        self.layers.is_empty()
    }

    pub fn step(&mut self, dt: f32, fade_seconds: f32, is_loaded: impl Fn(&str) -> bool) -> FadeStep {
        let dt = if dt.is_finite() && dt > 0.0 { dt } else { 0.0 };
        let rate = if fade_seconds > 0.0 { dt / fade_seconds } else { 1.0 };
        let mut out = FadeStep::default();
        let ready = self.target.clone().filter(|p| is_loaded(p));
        // 새 곡을 기다리는 중이면 지금 곡은 그대로 계속
        let waiting = self.target.is_some() && ready.is_none();
        if let Some(path) = &ready {
            if !self.layers.iter().any(|l| &l.path == path) {
                self.layers.push(MusicLayer { path: path.clone(), level: 0.0 });
                out.start.push(path.clone());
            }
        }
        for layer in &mut self.layers {
            if ready.as_ref() == Some(&layer.path) {
                layer.level = (layer.level + rate).min(1.0);
            } else if !waiting {
                layer.level = (layer.level - rate).max(0.0);
            }
        }
        let keep_path = ready;
        self.layers.retain(|l| {
            let keep = l.level > 0.0 || keep_path.as_ref() == Some(&l.path);
            if !keep {
                out.stop.push(l.path.clone());
            }
            keep
        });
        out.levels = self.layers.iter().map(|l| (l.path.clone(), l.level)).collect();
        out
    }
}

// ---------------------------------------------------------------- 창 열고 닫는 소리 (순수)

/// 소리가 나는 창 (멈춤 창은 효과음이 막혀 있어서 따로 다룬다).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WindowFlags {
    pub bag: bool,
    pub help: bool,
    pub craft: bool,
    pub shop: bool,
}

/// 창이 열리면 "창 열림", 닫히면 "창 닫힘". 물음표 안내는 열릴 때 버튼 소리.
#[derive(Debug, Default)]
pub struct WindowSounds {
    prev: WindowFlags,
}

impl WindowSounds {
    pub fn update(&mut self, now: WindowFlags) -> Vec<Sfx> {
        let before = self.prev;
        self.prev = now;
        let mut out = Vec::new();
        let mut edge = |was: bool, is: bool, open: Sfx| {
            if !was && is {
                out.push(open);
            } else if was && !is {
                out.push(Sfx::WindowClose);
            }
        };
        edge(before.bag, now.bag, Sfx::WindowOpen);
        edge(before.help, now.help, Sfx::ButtonClick);
        edge(before.craft, now.craft, Sfx::WindowOpen);
        edge(before.shop, now.shop, Sfx::WindowOpen);
        out
    }
}

// ---------------------------------------------------------------- "소리를 켰어요/껐어요" 글 (순수)

#[derive(Debug, Default)]
pub struct SoundToast {
    left: f32,
    on: bool,
}

impl SoundToast {
    pub fn show(&mut self, on: bool) {
        self.left = SOUND_TOAST_SECONDS;
        self.on = on;
    }

    pub fn update(&mut self, dt: f32) {
        if dt.is_finite() && dt > 0.0 {
            self.left = (self.left - dt).max(0.0);
        }
    }

    pub fn text(&self) -> Option<&'static str> {
        (self.left > 0.0).then_some(if self.on { SOUND_TOAST_ON_TEXT } else { SOUND_TOAST_OFF_TEXT })
    }
}

pub fn draw_toast(text: &TextPainter, toast: &SoundToast) {
    let Some(message) = toast.text() else { return };
    let size = 40.0;
    let w = text.width(message, size) + 60.0;
    draw_pill(SOUND_TOAST_CENTER_X - w / 2.0, SOUND_TOAST_Y, w, SOUND_TOAST_H, Color::new(0.0, 0.0, 0.0, 0.7));
    text.centered(message, SOUND_TOAST_CENTER_X, SOUND_TOAST_Y + SOUND_TOAST_H - 20.0, size, WHITE);
}

// ---------------------------------------------------------------- 크기 칸 (순수)

/// 지금 크기(%)와 가장 가까운 칸 번호 (`SOUND_VOLUME_STEPS`). 칸 사이 값(예: 70)은 가까운 쪽.
pub fn volume_cell_for(percent: u8) -> usize {
    let mut best = 0;
    for (i, step) in SOUND_VOLUME_STEPS.iter().enumerate() {
        if (*step as i32 - percent as i32).abs() < (SOUND_VOLUME_STEPS[best] as i32 - percent as i32).abs() {
            best = i;
        }
    }
    best
}

// ---------------------------------------------------------------- 스피커 버튼 (타이틀 구석)

/// 타이틀·캐릭터 고르기 오른쪽 위 스피커 버튼 (가상 좌표). 폰에서도 누르기 쉬운 크기.
pub const SPEAKER_BUTTON: Rect = Rect::new(VIRTUAL_WIDTH - 100.0, 14.0, 86.0, 86.0);

pub fn draw_speaker_button(on: bool) {
    let r = SPEAKER_BUTTON;
    draw_circle(r.center().x, r.center().y, r.w / 2.0, Color::new(0.0, 0.0, 0.0, 0.45));
    let c = r.center();
    // 스피커 몸통과 나팔
    draw_rectangle(c.x - 24.0, c.y - 10.0, 16.0, 20.0, WHITE);
    draw_triangle(vec2(c.x - 10.0, c.y - 10.0), vec2(c.x - 10.0, c.y + 10.0), vec2(c.x + 8.0, c.y + 24.0), WHITE);
    draw_triangle(vec2(c.x - 10.0, c.y - 10.0), vec2(c.x + 8.0, c.y - 24.0), vec2(c.x + 8.0, c.y + 24.0), WHITE);
    if on {
        draw_arc(c.x + 8.0, c.y, 12.0, 4.0);
        draw_arc(c.x + 8.0, c.y, 22.0, 4.0);
    } else {
        // 꺼지면 X
        draw_line(c.x + 14.0, c.y - 12.0, c.x + 34.0, c.y + 12.0, 6.0, Color::new(1.0, 0.3, 0.3, 1.0));
        draw_line(c.x + 14.0, c.y + 12.0, c.x + 34.0, c.y - 12.0, 6.0, Color::new(1.0, 0.3, 0.3, 1.0));
    }
}

/// 오른쪽으로 퍼지는 소리 물결 (반지름 `r`의 조각 호)
fn draw_arc(cx: f32, cy: f32, r: f32, thickness: f32) {
    let mut prev: Option<Vec2> = None;
    for i in 0..=8 {
        let a = -0.9 + 1.8 * i as f32 / 8.0;
        let p = vec2(cx + a.cos() * r, cy + a.sin() * r);
        if let Some(q) = prev {
            draw_line(q.x, q.y, p.x, p.y, thickness, WHITE);
        }
        prev = Some(p);
    }
}

/// M 키: 소리 켜기/끄기
pub fn sound_key_pressed() -> bool {
    is_key_pressed(KeyCode::M)
}

// ---------------------------------------------------------------- 소리 받기와 재생

type Bank = Arc<Mutex<HashMap<String, Sound>>>;
type Report = Arc<Mutex<LoadReport>>;

/// 잠금이 깨져도 패닉하지 않고 안의 값을 그대로 쓴다.
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

/// 지금 상태를 알려 주는 숫자 (콘솔 기록과 확인용).
#[derive(Debug, Default, Clone, Copy)]
struct LoadReport {
    loaded: usize,
    failed: usize,
    finished: bool,
}

pub struct Audio {
    bank: Bank,
    report: Report,
    loader: Option<Coroutine>,
    fader: MusicFader,
    /// 지금 재생 중인 곡과 마지막으로 넘긴 크기
    playing: HashMap<String, f32>,
    logged_finish: bool,
}

impl Audio {
    pub fn new() -> Audio {
        Audio {
            bank: Arc::new(Mutex::new(HashMap::new())),
            report: Arc::new(Mutex::new(LoadReport::default())),
            loader: None,
            fader: MusicFader::new(),
            playing: HashMap::new(),
            logged_finish: false,
        }
    }

    /// "눌러서 시작"을 누른 뒤에 한 번 부른다. 효과음 먼저, 음악은 타이틀 → 낮 → 밤 → 상점 순서로 하나씩 받는다.
    /// 두 번째부터는 아무 일도 안 한다.
    pub fn start_loading(&mut self) {
        if self.loader.is_some() {
            return;
        }
        let paths: Vec<String> = loading_order().into_iter().filter_map(|item| item.asset_path()).collect();
        let (bank, report) = (self.bank.clone(), self.report.clone());
        self.loader = Some(start_coroutine(load_all(paths, bank, report)));
    }

    /// 효과음 하나 재생 (`volume` 0~1). 안 받아졌거나 크기가 0이면 아무 일 없다.
    pub fn play_sfx(&self, sfx: Sfx, volume: f32) {
        if volume <= 0.0 {
            return;
        }
        let Some(path) = sfx.asset_path() else { return };
        if let Some(sound) = lock(&self.bank).get(&path) {
            play_sound(sound, PlaySoundParams { looped: false, volume: volume.clamp(0.0, 1.0) });
        }
    }

    /// 틀고 싶은 곡을 알린다 (None이면 멈춘다). 곡이 아직 안 받아졌으면 받아진 뒤에 시작된다.
    pub fn set_music(&mut self, track: Option<Track>) {
        self.fader.set_target(track.map(|t| t.asset_path()));
    }

    /// 매 프레임 부른다. `volume`은 음악 크기 (0~1, 끄면 0이라 곡은 계속 돌고 소리만 0).
    pub fn update_music(&mut self, dt: f32, volume: f32) {
        self.log_when_done();
        let bank = self.bank.clone();
        let step = self.fader.step(dt, MUSIC_FADE_SECONDS, |p| lock(&bank).contains_key(p));
        let bank = lock(&self.bank);
        for path in &step.start {
            if let Some(sound) = bank.get(path) {
                play_sound(sound, PlaySoundParams { looped: true, volume: 0.0 });
                self.playing.insert(path.clone(), 0.0);
            }
        }
        for (path, level) in &step.levels {
            let v = (level * volume).clamp(0.0, 1.0);
            let last = self.playing.get(path).copied().unwrap_or(-1.0);
            if (v - last).abs() > 0.002 {
                if let Some(sound) = bank.get(path) {
                    set_sound_volume(sound, v);
                    self.playing.insert(path.clone(), v);
                }
            }
        }
        for path in &step.stop {
            if let Some(sound) = bank.get(path) {
                stop_sound(sound);
            }
            self.playing.remove(path);
        }
    }

    fn log_when_done(&mut self) {
        let r = *lock(&self.report);
        if r.finished && !self.logged_finish {
            self.logged_finish = true;
            macroquad::logging::info!("소리 받기 끝: {}개 받음, {}개 못 받음", r.loaded, r.failed);
        }
    }
}

/// 제자리에서 한 번 훑어 보는 빈 깨우미 (macroquad는 프레임마다 직접 다시 부른다).
fn poll_once<F: Future>(fut: &mut std::pin::Pin<Box<F>>) -> Poll<F::Output> {
    let mut cx = Context::from_waker(Waker::noop());
    fut.as_mut().poll(&mut cx)
}

/// 한 일을 프레임마다 훑어 보며 기다린다. 시간(SOUND_LOAD_TIMEOUT_SECONDS)을 넘으면 포기하고 None (멈춰도 게임은 계속).
async fn wait_with_timeout<F: Future>(fut: F) -> Option<F::Output> {
    let mut fut = Box::pin(fut);
    let started = get_time();
    // 횟수 제한: 시간 제한을 넘으면 끝난다
    while get_time() - started < SOUND_LOAD_TIMEOUT_SECONDS {
        if let Poll::Ready(out) = poll_once(&mut fut) {
            return Some(out);
        }
        next_frame().await;
    }
    None
}

/// 파일을 순서대로 하나씩 받아서 풀어 둔다. 받기와 풀기 둘 다 시간 제한이 있어서, 하나가 멈춰도 다음으로 넘어간다 (패닉 없음).
async fn load_all(paths: Vec<String>, bank: Bank, report: Report) {
    for path in paths {
        let full = format!("assets/{path}");
        let sound = match wait_with_timeout(load_file(&full)).await {
            Some(Ok(data)) => match wait_with_timeout(load_sound_from_bytes(&data)).await {
                Some(Ok(sound)) => Some(sound),
                _ => None,
            },
            _ => None,
        };
        match sound {
            Some(sound) => {
                lock(&bank).insert(path.clone(), sound);
                lock(&report).loaded += 1;
            }
            None => lock(&report).failed += 1,
        }
    }
    lock(&report).finished = true;
}

#[cfg(test)]
mod tests {
    use super::*;

    const FRAME: f32 = 1.0 / 60.0;

    fn level_of(step: &FadeStep, path: &str) -> Option<f32> {
        step.levels.iter().find(|(p, _)| p == path).map(|(_, l)| *l)
    }

    #[test]
    fn first_track_starts_once_and_fades_in_over_two_seconds() {
        let mut f = MusicFader::new();
        f.set_target(Some("a".into()));
        let s = f.step(FRAME, MUSIC_FADE_SECONDS, |_| true);
        assert_eq!(s.start, vec!["a".to_string()]);
        let mut last = level_of(&s, "a").unwrap();
        assert!(last > 0.0 && last < 0.05);
        for _ in 0..119 {
            let s = f.step(FRAME, MUSIC_FADE_SECONDS, |_| true);
            assert!(s.start.is_empty(), "다시 시작하지 않는다");
            let now = level_of(&s, "a").unwrap();
            assert!(now >= last);
            last = now;
        }
        let s = f.step(FRAME, MUSIC_FADE_SECONDS, |_| true);
        assert_eq!(level_of(&s, "a"), Some(1.0));
    }

    #[test]
    fn crossfade_old_goes_down_new_goes_up_then_old_stops() {
        let mut f = MusicFader::new();
        f.set_target(Some("day".into()));
        for _ in 0..200 {
            f.step(FRAME, MUSIC_FADE_SECONDS, |_| true);
        }
        f.set_target(Some("night".into()));
        let s = f.step(1.0, MUSIC_FADE_SECONDS, |_| true);
        assert_eq!(s.start, vec!["night".to_string()]);
        assert!((level_of(&s, "day").unwrap() - 0.5).abs() < 1e-5, "1초 뒤 앞 곡 절반");
        assert!((level_of(&s, "night").unwrap() - 0.5).abs() < 1e-5, "새 곡 절반");
        let s = f.step(1.0, MUSIC_FADE_SECONDS, |_| true);
        assert_eq!(s.stop, vec!["day".to_string()]);
        assert_eq!(level_of(&s, "night"), Some(1.0));
        assert_eq!(level_of(&s, "day"), None);
    }

    #[test]
    fn waits_for_unloaded_track_and_keeps_old_one_playing() {
        let mut f = MusicFader::new();
        f.set_target(Some("title".into()));
        for _ in 0..200 {
            f.step(FRAME, MUSIC_FADE_SECONDS, |_| true);
        }
        f.set_target(Some("day".into()));
        let s = f.step(1.0, MUSIC_FADE_SECONDS, |p| p == "title");
        assert!(s.start.is_empty() && s.stop.is_empty());
        assert_eq!(level_of(&s, "title"), Some(1.0), "받는 동안 앞 곡은 그대로");
        let s = f.step(0.5, MUSIC_FADE_SECONDS, |_| true);
        assert_eq!(s.start, vec!["day".to_string()], "받아지면 그때 시작");
    }

    #[test]
    fn unloaded_first_track_is_silent_and_safe() {
        let mut f = MusicFader::new();
        f.set_target(Some("x".into()));
        let s = f.step(FRAME, MUSIC_FADE_SECONDS, |_| false);
        assert_eq!(s, FadeStep::default());
        assert!(f.is_idle());
    }

    #[test]
    fn stop_fades_out_and_bad_dt_is_safe() {
        let mut f = MusicFader::new();
        f.set_target(Some("a".into()));
        for _ in 0..200 {
            f.step(FRAME, MUSIC_FADE_SECONDS, |_| true);
        }
        f.set_target(None);
        let s = f.step(f32::NAN, MUSIC_FADE_SECONDS, |_| true);
        assert_eq!(level_of(&s, "a"), Some(1.0), "이상한 dt는 0");
        let s = f.step(1.0, MUSIC_FADE_SECONDS, |_| true);
        assert!((level_of(&s, "a").unwrap() - 0.5).abs() < 1e-5);
        let s = f.step(1.0, MUSIC_FADE_SECONDS, |_| true);
        assert_eq!(s.stop, vec!["a".to_string()]);
        assert!(f.is_idle());
    }

    #[test]
    fn going_back_to_a_fading_track_does_not_restart_it() {
        let mut f = MusicFader::new();
        f.set_target(Some("a".into()));
        for _ in 0..200 {
            f.step(FRAME, MUSIC_FADE_SECONDS, |_| true);
        }
        f.set_target(Some("b".into()));
        f.step(0.5, MUSIC_FADE_SECONDS, |_| true);
        f.set_target(Some("a".into()));
        let s = f.step(0.1, MUSIC_FADE_SECONDS, |_| true);
        assert!(!s.start.contains(&"a".to_string()), "a는 아직 틀어져 있어서 다시 시작하지 않는다");
    }

    #[test]
    fn windows_make_open_and_close_sounds() {
        let mut w = WindowSounds::default();
        assert!(w.update(WindowFlags::default()).is_empty());
        assert_eq!(w.update(WindowFlags { bag: true, ..Default::default() }), vec![Sfx::WindowOpen]);
        assert!(w.update(WindowFlags { bag: true, ..Default::default() }).is_empty(), "열려 있는 동안은 조용");
        assert_eq!(w.update(WindowFlags::default()), vec![Sfx::WindowClose]);
        assert_eq!(w.update(WindowFlags { help: true, ..Default::default() }), vec![Sfx::ButtonClick], "물음표는 버튼 소리");
        assert_eq!(w.update(WindowFlags::default()), vec![Sfx::WindowClose]);
        assert_eq!(w.update(WindowFlags { shop: true, ..Default::default() }), vec![Sfx::WindowOpen]);
    }

    #[test]
    fn toast_shows_for_one_second_with_right_text() {
        let mut t = SoundToast::default();
        assert_eq!(t.text(), None);
        t.show(false);
        assert_eq!(t.text(), Some("소리를 껐어요"));
        t.update(0.5);
        assert!(t.text().is_some());
        t.update(0.6);
        assert_eq!(t.text(), None);
        t.show(true);
        assert_eq!(t.text(), Some("소리를 켰어요"));
        t.update(f32::NAN);
        assert!(t.text().is_some());
    }

    #[test]
    fn toast_sits_low_on_the_right_inside_the_screen_beside_the_held_slot() {
        // 글 칸 너비는 넉넉히 셈: 한 글자 40 × 글자 수 + 양쪽 30
        for message in [SOUND_TOAST_ON_TEXT, SOUND_TOAST_OFF_TEXT] {
            let w = message.chars().count() as f32 * 40.0 + 60.0;
            let left = SOUND_TOAST_CENTER_X - w / 2.0;
            assert!(left >= super::super::hud::HELD_SLOT.right(), "손에 든 칸을 안 가린다");
            assert!(SOUND_TOAST_CENTER_X + w / 2.0 <= VIRTUAL_WIDTH);
        }
        assert!(SOUND_TOAST_Y > 400.0, "위쪽 정보판·제목과 멀다");
        assert!(SOUND_TOAST_Y + SOUND_TOAST_H <= super::super::screen::VIRTUAL_HEIGHT);
    }

    #[test]
    fn volume_cell_is_nearest_step() {
        assert_eq!(volume_cell_for(0), 0);
        assert_eq!(volume_cell_for(25), 1);
        assert_eq!(volume_cell_for(50), 2);
        assert_eq!(volume_cell_for(80), 3);
        assert_eq!(volume_cell_for(70), 3, "기본 음악 70은 크게 쪽");
        assert_eq!(volume_cell_for(100), 3);
        assert_eq!(volume_cell_for(10), 0);
        assert_eq!(volume_cell_for(15), 1);
    }

    #[test]
    fn speaker_button_is_big_and_inside_the_screen() {
        assert!(SPEAKER_BUTTON.x >= 0.0 && SPEAKER_BUTTON.right() <= VIRTUAL_WIDTH && SPEAKER_BUTTON.y >= 0.0);
        // 740×360 폰(0.667배)에서도 손가락으로 누르기 쉽게 55 화면 픽셀 넘게
        assert!(SPEAKER_BUTTON.h * 360.0 / 540.0 >= 55.0);
    }
}
