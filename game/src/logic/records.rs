//! 최근 판 기록 (US-133). 판마다 버틴 날·점수·클리어 여부 세 가지만 (이름·날짜 없음).
//! 글자로 만들고 읽는 순수 함수만 있다. 브라우저·macroquad를 모른다.
//!
//!
//! 로건이 쓰는 순서:
//! 1. 판이 끝나는 순간 **한 번만** `settings.recent_runs.add(days, score, cleared)` (끝 화면이 다시 그려져도 또 부르지 않는다).
//!    같은 기록이어도 넣는다. "새 기록!" 판정은 따로(`view/gameover.rs`)라서 영향이 없다.
//! 2. 바뀐 `Settings`를 `settings_to_text`로 저장한다 (`?qa`에서는 읽지도 쓰지도 않는다).
//! 3. 표는 `settings.recent_runs.latest_first()` (최신이 위). 비었으면 `RECENT_RUNS_EMPTY_TEXT`.
//!    줄 글은 `RunRecord::line_text()`.

#![allow(dead_code)] // 로건이 부를 함수들이 화면에 이어지기 전까지 경고를 끈다

use crate::config::*;

/// 한 판의 기록.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunRecord {
    pub days: u32,
    pub score: u32,
    pub cleared: bool,
}

impl RunRecord {
    /// 표 한 줄 글. 예: `3일 120점`, 클리어하면 `7일 2400점 클리어`.
    pub fn line_text(&self) -> String {
        let mut line = format!("{}일 {}점", self.days, self.score);
        if self.cleared {
            line.push(' ');
            line.push_str(RECENT_RUN_CLEARED_TEXT);
        }
        line
    }
}

/// 최근 판들. 크기가 정해진 자리라서 `Copy`다. 오래된 판이 앞.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RecentRuns {
    runs: [RunRecord; RECENT_RUNS_MAX],
    len: usize,
}

impl RecentRuns {
    pub fn new() -> RecentRuns {
        RecentRuns::default()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// 새 판을 더한다. 가득 차 있으면 가장 오래된 판이 빠진다. 너무 큰 수는 상한으로 자른다.
    pub fn add(&mut self, days: u32, score: u32, cleared: bool) {
        let run = RunRecord { days: days.min(RECENT_RUN_DAYS_MAX), score: score.min(RECENT_RUN_SCORE_MAX), cleared };
        if self.len == RECENT_RUNS_MAX {
            self.runs.copy_within(1.., 0);
            self.runs[RECENT_RUNS_MAX - 1] = run;
        } else {
            self.runs[self.len] = run;
            self.len += 1;
        }
    }

    /// 오래된 것부터.
    pub fn oldest_first(&self) -> Vec<RunRecord> {
        self.runs[..self.len].to_vec()
    }

    /// 보여 줄 때: 최신이 위.
    pub fn latest_first(&self) -> Vec<RunRecord> {
        self.runs[..self.len].iter().rev().copied().collect()
    }

    /// 저장 글자의 값 부분. 예: `3.120.0,7.2400.1` (비면 빈 글자)
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        for (i, r) in self.runs[..self.len].iter().enumerate() {
            if i > 0 {
                text.push(RECENT_RUN_SEPARATOR);
            }
            text.push_str(&format!("{}{sep}{}{sep}{}", r.days, r.score, r.cleared as u8, sep = RECENT_RUN_FIELD_SEPARATOR));
        }
        text
    }

    /// 저장 글자의 값 부분을 읽는다. 앞의 5칸만 보고, 이상한 판은 그 판만 조용히 버린다. 패닉 없음.
    pub fn from_text(text: &str) -> RecentRuns {
        let mut runs = RecentRuns::new();
        for piece in text.trim().split(RECENT_RUN_SEPARATOR).take(RECENT_RUNS_MAX) {
            if let Some(r) = parse_run(piece.trim()) {
                runs.add(r.days, r.score, r.cleared);
            }
        }
        runs
    }
}

fn parse_run(piece: &str) -> Option<RunRecord> {
    let mut fields = piece.split(RECENT_RUN_FIELD_SEPARATOR);
    let days = parse_number(fields.next()?, RECENT_RUN_DAYS_MAX)?;
    let score = parse_number(fields.next()?, RECENT_RUN_SCORE_MAX)?;
    let cleared = match fields.next()? {
        "0" => false,
        "1" => true,
        _ => return None,
    };
    if fields.next().is_some() {
        return None; // 칸이 넷 이상
    }
    Some(RunRecord { days, score, cleared })
}

/// 숫자 글자만 (부호·소수·빈 글자·상한 넘음은 None).
fn parse_number(text: &str, max: u32) -> Option<u32> {
    if text.is_empty() || text.len() > 10 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let n: u32 = text.parse().ok()?;
    (n <= max).then_some(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filled(count: u32) -> RecentRuns {
        let mut r = RecentRuns::new();
        for i in 1..=count {
            r.add(i, i * 100, i % 2 == 0);
        }
        r
    }

    #[test]
    fn 비어_있다() {
        let r = RecentRuns::new();
        assert!(r.is_empty() && r.len() == 0 && r.latest_first().is_empty());
        assert_eq!(r.to_text(), "");
    }

    #[test]
    fn 다섯_판을_채운다() {
        let r = filled(5);
        assert_eq!(r.len(), RECENT_RUNS_MAX);
        assert_eq!(r.oldest_first()[0].days, 1);
    }

    #[test]
    fn 여섯째_판이_들어오면_첫_판이_빠진다() {
        let r = filled(6);
        assert_eq!(r.len(), RECENT_RUNS_MAX);
        let days: Vec<u32> = r.oldest_first().iter().map(|x| x.days).collect();
        assert_eq!(days, vec![2, 3, 4, 5, 6]);
    }

    #[test]
    fn 최신이_위로_보인다() {
        let days: Vec<u32> = filled(3).latest_first().iter().map(|x| x.days).collect();
        assert_eq!(days, vec![3, 2, 1]);
    }

    #[test]
    fn 같은_기록도_별개의_판으로_들어간다() {
        let mut r = RecentRuns::new();
        r.add(3, 100, false);
        r.add(3, 100, false);
        assert_eq!(r.len(), 2);
    }

    #[test]
    fn 너무_큰_수는_상한으로_자른다() {
        let mut r = RecentRuns::new();
        r.add(u32::MAX, u32::MAX, true);
        let run = r.latest_first()[0];
        assert_eq!((run.days, run.score), (RECENT_RUN_DAYS_MAX, RECENT_RUN_SCORE_MAX));
        assert_eq!(RecentRuns::from_text(&r.to_text()), r);
    }

    #[test]
    fn 줄_글() {
        assert_eq!(RunRecord { days: 3, score: 120, cleared: false }.line_text(), "3일 120점");
        assert_eq!(RunRecord { days: 7, score: 2400, cleared: true }.line_text(), "7일 2400점 클리어");
    }

    #[test]
    fn 글자로_왕복한다() {
        for n in 0..=7 {
            let r = filled(n);
            assert_eq!(RecentRuns::from_text(&r.to_text()), r, "{n}판");
        }
        assert_eq!(filled(2).to_text(), "1.100.0,2.200.1");
    }

    #[test]
    fn 이상한_판은_그_판만_버린다() {
        let bad = ["", "a.b.c", "-1.5.0", "1.-5.0", "1.5.2", "1.5", "1.5.0.0", "4294967295.1.0", "1.4294967295.0", "1.5.x", "+1.5.0", "1..0", "1.5.", "1.5.00", "1.5.1 1", "1,5,0", "한.글.1", "1.5.0;"];
        for b in bad {
            let text = format!("2.20.0,{b},3.30.1");
            let r = RecentRuns::from_text(&text);
            let days: Vec<u32> = r.oldest_first().iter().map(|x| x.days).collect();
            assert!(days.starts_with(&[2]), "{b}: {days:?}");
            assert!(r.len() <= RECENT_RUNS_MAX, "{b}");
        }
        assert_eq!(RecentRuns::from_text("zzz"), RecentRuns::new());
        assert_eq!(RecentRuns::from_text("\u{0}\u{ffff}"), RecentRuns::new());
        assert_eq!(RecentRuns::from_text(&"9".repeat(500)), RecentRuns::new());
    }

    #[test]
    fn 다섯_개_넘는_목록은_앞의_다섯만() {
        let r = RecentRuns::from_text("1.1.0,2.2.0,3.3.0,4.4.0,5.5.0,6.6.0,7.7.0");
        let days: Vec<u32> = r.oldest_first().iter().map(|x| x.days).collect();
        assert_eq!(days, vec![1, 2, 3, 4, 5]);
    }
}
