//! M9 새 기록 소리(US-132)와 최근 5판 기록(US-133) 규칙 시험 (수산나).
//! 반복은 모두 횟수가 정해져 있다.

#[path = "../src/config.rs"]
mod config;
#[allow(dead_code)]
#[path = "../src/logic/mod.rs"]
mod logic;

use config::*;
use logic::records::{RecentRuns, RunRecord};
use logic::settings::{settings_from_text, settings_to_text, Settings};
use logic::sound::{sfx_for_end_screen, Sfx, SfxMixer, SoundSettings};

fn five_biggest() -> RecentRuns {
    let mut r = RecentRuns::new();
    for _ in 0..RECENT_RUNS_MAX {
        r.add(u32::MAX, u32::MAX, true); // 상한으로 잘려 가장 긴 글자가 된다
    }
    r
}

// ---- US-132 ----

#[test]
fn 새_기록_소리_여섯_경우() {
    assert_eq!(sfx_for_end_screen(true, false), Some(Sfx::NewRecord));
    assert_eq!(sfx_for_end_screen(false, true), Some(Sfx::NewRecord));
    assert_eq!(sfx_for_end_screen(true, true), Some(Sfx::NewRecord));
    assert_eq!(sfx_for_end_screen(false, false), None);
}

#[test]
fn 소리가_꺼져_있으면_믹서가_버린다_켜면_난다() {
    let mut off = SoundSettings::default();
    off.toggle();
    let mut m = SfxMixer::new();
    m.request_opt(sfx_for_end_screen(true, true));
    assert!(m.take_playable(0.016, &off, false).is_empty());
    m.request_opt(sfx_for_end_screen(true, true));
    assert_eq!(m.take_playable(0.016, &SoundSettings::default(), false), vec![Sfx::NewRecord]);
}

#[test]
fn 같은_프레임에_두_번_부탁해도_한_번만_난다() {
    let mut m = SfxMixer::new();
    m.request_opt(sfx_for_end_screen(true, false));
    m.request_opt(sfx_for_end_screen(false, true));
    assert_eq!(m.take_playable(0.016, &SoundSettings::default(), false), vec![Sfx::NewRecord]);
}

#[test]
fn 새_기록_소리는_중요하고_파일이_있다() {
    assert!(Sfx::NewRecord.is_important());
    assert!(Sfx::ALL.contains(&Sfx::NewRecord));
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets").join(SOUNDS_SFX_DIR).join(NEW_RECORD_FILE);
    assert!(path.is_file(), "새 기록 소리 파일이 없다: {path:?}");
}

// ---- US-133 ----

#[test]
fn 다섯_판_채움_여섯째에_첫_판_빠짐() {
    let mut r = RecentRuns::new();
    for i in 1..=5u32 {
        r.add(i, i * 10, false);
    }
    assert_eq!(r.len(), 5);
    r.add(6, 60, true);
    assert_eq!(r.len(), RECENT_RUNS_MAX);
    let days: Vec<u32> = r.oldest_first().iter().map(|x| x.days).collect();
    assert_eq!(days, vec![2, 3, 4, 5, 6]);
    assert_eq!(r.latest_first()[0], RunRecord { days: 6, score: 60, cleared: true });
}

#[test]
fn 같은_기록도_목록에_들어간다() {
    let mut r = RecentRuns::new();
    r.add(4, 500, false);
    r.add(4, 500, false);
    assert_eq!(r.len(), 2);
}

#[test]
fn 설정_글자로_왕복한다() {
    let mut s = Settings { tutorial_done: true, sound_on: false, music_percent: 30, ..Settings::default() };
    for i in 0..7u32 {
        s.recent_runs.add(i + 1, i * 333, i % 3 == 0);
        assert_eq!(settings_from_text(&settings_to_text(&s)), s, "{i}판째");
    }
    assert!(settings_to_text(&s).contains("runs="));
}

#[test]
fn 저장_글자_모양_예시() {
    let mut s = Settings::default();
    s.recent_runs.add(3, 120, false);
    s.recent_runs.add(7, 2400, true);
    let text = settings_to_text(&s);
    assert!(text.ends_with(";runs=3.120.0,7.2400.1"), "{text}");
}

#[test]
fn 옛_저장_글자는_빈_목록으로_정상() {
    let old = settings_from_text("1;tutorial=1;tip=0;sound=1;music=70;effects=80");
    assert!(old.recent_runs.is_empty());
    assert!(old.tutorial_done && old.sound_on);
    assert_eq!((old.music_percent, old.effects_percent), (70, 80));
    // 기록이 없는 설정은 옛 글자처럼 runs 없이 저장된다
    assert!(!settings_to_text(&old).contains("runs"));
}

#[test]
fn 이상한_값은_그_판만_버리고_나머지_설정은_산다() {
    let bad = ["-1.5.0", "1.-5.0", "4294967295.5.0", "1.4294967295.0", "1.5.2", "1.5", "abc", "1.5.0.0", "", "1.5.x", "99999999999999999999.1.0"];
    for b in bad {
        let s = settings_from_text(&format!("1;tutorial=1;music=40;runs=2.20.0,{b},3.30.1"));
        assert!(s.tutorial_done && s.music_percent == 40, "{b}");
        let days: Vec<u32> = s.recent_runs.oldest_first().iter().map(|x| x.days).collect();
        assert!(days.starts_with(&[2]), "{b}: {days:?}");
        assert!(days.len() <= RECENT_RUNS_MAX, "{b}");
    }
    assert!(settings_from_text("1;runs=\u{0}\u{ffff}한글").recent_runs.is_empty());
    assert!(settings_from_text("1;runs=").recent_runs.is_empty());
    assert!(settings_from_text("1;runs").recent_runs.is_empty());
}

#[test]
fn 같은_이름이_또_나오면_첫_것만() {
    let s = settings_from_text("1;runs=1.1.0;runs=9.9.1");
    assert_eq!(s.recent_runs.oldest_first(), vec![RunRecord { days: 1, score: 1, cleared: false }]);
}

#[test]
fn 다섯_개_넘는_목록은_앞의_다섯만() {
    let s = settings_from_text("1;runs=1.1.0,2.2.0,3.3.0,4.4.0,5.5.0,6.6.0,7.7.0,8.8.0");
    let days: Vec<u32> = s.recent_runs.oldest_first().iter().map(|x| x.days).collect();
    assert_eq!(days, vec![1, 2, 3, 4, 5]);
}

#[test]
fn 가장_큰_다섯_판과_다른_설정이_글자_상한_안에_들어간다() {
    let s = Settings {
        tutorial_done: true,
        window_tips_seen: logic::window_tips::WindowTipsSeen::from_bits(logic::window_tips::WindowTipsSeen::ALL_BITS).unwrap(),
        music_percent: 100,
        effects_percent: 100,
        recent_runs: five_biggest(),
        ..Settings::default()
    };
    let text = settings_to_text(&s);
    assert!(text.chars().count() <= SETTINGS_MAX_CHARS, "{} 글자: {text}", text.chars().count());
    assert!(text.is_ascii());
    assert_eq!(settings_from_text(&text), s, "가장 큰 값도 다시 읽으면 같다");
}

#[test]
fn 목록_빈_글과_개인_정보_칸() {
    assert_eq!(RECENT_RUNS_EMPTY_TEXT, "아직 없어요");
    let mut s = Settings::default();
    s.recent_runs.add(2, 30, false);
    let text = settings_to_text(&s);
    for part in text.split(SETTINGS_ITEM_SEPARATOR).skip(1) {
        let key = part.split(SETTINGS_VALUE_SEPARATOR).next().unwrap();
        assert!(["tutorial", "tips", "sound", "music", "effects", "runs"].contains(&key), "{key}");
    }
}

#[test]
fn 소리와_튜토리얼_설정은_기록과_따로_산다() {
    let mut s = Settings::default();
    s.recent_runs.add(1, 1, false);
    let mut sound = SoundSettings::from_settings(&s);
    sound.set_music_percent(10);
    let after = sound.settings_after(&s);
    assert_eq!(after.recent_runs, s.recent_runs);
}
