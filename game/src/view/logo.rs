//! 니로 스튜디오 로고 (US-109, 부모님 결정 2026-11-21: 영상 파일 대신 게임 안에서 그린다).
//!
//! 빨간 화면에 1·2·3이 통통 → "Hey" → 위에 "Lucky Mis!", 아래에서 미스가 빼꼼 (눈 두 번 깜빡, 살짝 뿅),
//! 구석에 작게 "NIRO STUDIO". 소리는 없다.
//! - 게임을 열 때마다 한 번 (main이 장면 루프에 들어가기 전에 부른다). 다시 하기·"처음으로"는 장면 루프 안이라 안 나온다.
//! - 순서 (부모님 결정 2026-11-21): Now Loading(도는 점) → 다 받으면 "눌러서 시작" → 누르면 로고 → 타이틀.
//!   "눌러서 시작"은 나중에 로고에 아이 목소리를 넣기 위한 것 (브라우저는 처음 누르기 전에는 소리를 못 낸다).
//! - 글꼴만 있으면 그린다. 나머지 그림은 Now Loading 동안 받는다 (`play_logo`의 `poll`).
//! - 시작 누름은 로고를 넘기지 않는다. 로고는 클릭·터치·아무 키로 바로 넘긴다.
//!   칸을 바꾼 그 프레임의 누름은 다음 칸에 넘기지 않아서, 누른 것이 로고 넘기기나 타이틀 버튼이 되지 않는다.
//! - `?qa`에서는 "눌러서 시작"과 로고를 건너뛴다 (엘리 시험용).
//! 글자 크기는 고정하고 확대(`zoom`)만 바꾼다: 프레임마다 새 글꼴 크기를 만들면 끊긴다 (BUG-029).

use super::input::{any_key_pressed, fingers};
use super::screen::{Letterbox, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::text::TextPainter;
use super::touch::TouchControls;
use macroquad::prelude::*;

// ---- 시간 (초): config.rs의 "로고·시작 화면" 칸 (US-109 완료 조건) ----
use crate::config::{
    LOADING_SPINNER_TURN_SECONDS as SPINNER_TURN_SECONDS, LOGO_BLINKS as BLINKS, LOGO_BLINK_SECONDS as BLINK_SECONDS,
    LOGO_COUNT_END, LOGO_HEY_END, LOGO_HOP_SECONDS as HOP_SECONDS, LOGO_HOP_START as HOP_START, LOGO_MIS_RISE as MIS_RISE,
    LOGO_OUT_SECONDS as OUT_SECONDS, LOGO_POP_SECONDS as POP_SECONDS, LOGO_SECONDS, LOGO_STUDIO_FADE as STUDIO_FADE,
    TAP_TO_START_BLINK_SECONDS,
};
/// 뿅 뛰는 높이 (가상 픽셀)
const HOP_HEIGHT: f32 = 22.0;

// ---- 모양 (960×540 가상 화면) ----
/// 배경 빨강 #FF0000 (부모님 초안)
pub const LOGO_RED: Color = Color::new(1.0, 0.0, 0.0, 1.0);
const LETTER: Color = WHITE;
/// 글자 그림자 (진한 빨강)
const LETTER_SHADOW: Color = Color::new(0.45, 0.0, 0.0, 0.45);
const BIG_SIZE: f32 = 140.0;
const HEY_SIZE: f32 = 130.0;
const LUCKY_SIZE: f32 = 96.0;
const STUDIO_SIZE: f32 = 26.0;
const LOADING_SIZE: f32 = 40.0;
/// 받는 동안 글자 (부모님 결정 2026-11-21). web/index.html의 #loading과 같은 글자·모양.
const LOADING_TEXT: &str = "Now Loading...";
const LOADING_TEXT_CENTER: Vec2 = vec2(480.0, 320.0);
/// 빙글빙글 점 스피너: 점 8개가 원 위에서 1초에 한 바퀴 (index.html의 .spinner와 같다)
const SPINNER_CENTER: Vec2 = vec2(480.0, 225.0);
const SPINNER_RADIUS: f32 = 36.0;
const SPINNER_DOT: f32 = 7.0;
const SPINNER_DOTS: usize = 8;
/// 다 받은 뒤 "눌러서 시작" (부모님 결정 2026-11-21: 로고에 아이 목소리를 넣으려면 먼저 한 번 눌러야 한다)
const TAP_TEXT: &str = "눌러서 시작";
const TAP_SIZE: f32 = 72.0;
/// 깜빡일 때 가장 흐린 진하기 (안 보일 만큼 흐려지지는 않는다)
const TAP_MIN_ALPHA: f32 = 0.35;
/// 큰 글자의 가운데
const BIG_CENTER: Vec2 = vec2(480.0, 270.0);
const LUCKY_CENTER: Vec2 = vec2(480.0, 112.0);
/// "NIRO STUDIO" 가운데 (오른쪽 아래 구석, 미스 손과 안 겹친다)
const STUDIO_CENTER: Vec2 = vec2(850.0, 510.0);
/// 글자 가운데 → 바닥선 (Jua 글자 높이의 반쯤, 글자 크기 비율)
const BASELINE_DROP: f32 = 0.36;

/// 미스 모양: 헤일리 아이콘(art/icons/make_icons.py)의 원본 좌표 그대로. 몸 아래 선 가운데가 화면 아래 가운데에 온다.
const MIS_SCALE: f32 = 1.1;
const ICON_CENTER_X: f32 = 180.5;
const ICON_BOTTOM: f32 = 345.5;
const ICON_TBAR: (f32, f32, f32, f32, f32) = (84.0, 143.5, 274.5, 194.5, 9.0);
const ICON_TSTEM: (f32, f32, f32, f32) = (153.5, 190.0, 205.5, 280.0);
const ICON_BODY: (f32, f32, f32) = (180.5, 341.2, 110.0);
const ICON_EYES: [(f32, f32, f32, f32); 2] = [(140.0, 302.0, 9.6, 21.7), (220.8, 302.0, 9.6, 21.7)];
const ICON_HANDS: [(f32, f32, f32); 2] = [(62.0, 327.0, 19.4), (298.8, 327.0, 19.2)];
/// 몸·손 테두리 (아이콘의 회색). 빨간 바탕에서 보이게 아이콘보다 조금 굵게.
const MIS_LINE: Color = Color::new(88.0 / 255.0, 88.0 / 255.0, 88.0 / 255.0, 1.0);
const MIS_LINE_WIDTH: f32 = 2.5;

/// 통통 튀어나오기: 0 → 살짝 넘침(약 1.15) → 1.
fn pop(p: f32) -> f32 {
    const C1: f32 = 2.2;
    const C3: f32 = C1 + 1.0;
    let q = p.clamp(0.0, 1.0) - 1.0;
    (1.0 + C3 * q * q * q + C1 * q * q).max(0.0)
}

/// `[start, end)` 동안 나오는 글자의 (확대, 진하기). 처음에 통통, 끝에서 작아지며 사라진다.
fn slot(t: f32, start: f32, end: f32) -> Option<(f32, f32)> {
    if t < start || t >= end {
        return None;
    }
    let zoom = pop((t - start) / POP_SECONDS);
    let left = end - t;
    if left < OUT_SECONDS {
        let q = left / OUT_SECONDS;
        Some((zoom * q, q))
    } else {
        Some((zoom, 1.0))
    }
}

/// 한 순간의 로고 모습 (그리기와 시험이 같이 쓴다).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LogoFrame {
    /// 가운데 큰 글자 ("1"·"2"·"3"·"Hey"), 확대, 진하기
    pub big: Option<(&'static str, f32, f32)>,
    /// "Lucky Mis!" 확대 (0이면 아직 없음)
    pub lucky_zoom: f32,
    /// 미스가 올라온 정도 0(안 보임)~1(다 올라옴)
    pub mis_rise: f32,
    pub eyes_open: bool,
    /// 뿅 뛴 높이 (픽셀, 위쪽이 +)
    pub hop: f32,
    /// "NIRO STUDIO" 진하기
    pub studio_alpha: f32,
}

pub fn logo_frame(t: f32) -> LogoFrame {
    let count = LOGO_COUNT_END / 3.0;
    let numbers = ["1", "2", "3"];
    let big = (0..3)
        .find_map(|i| slot(t, count * i as f32, count * (i + 1) as f32).map(|(z, a)| (numbers[i], z, a)))
        .or_else(|| slot(t, LOGO_COUNT_END, LOGO_HEY_END).map(|(z, a)| ("Hey", z, a)));
    let lucky_zoom = if t >= LOGO_HEY_END { pop((t - LOGO_HEY_END) / POP_SECONDS) } else { 0.0 };
    let r = ((t - MIS_RISE.0) / (MIS_RISE.1 - MIS_RISE.0)).clamp(0.0, 1.0);
    let mis_rise = 1.0 - (1.0 - r).powi(3);
    let eyes_open = !BLINKS.iter().any(|b| t >= *b && t < b + BLINK_SECONDS);
    let hop_p = (t - HOP_START) / HOP_SECONDS;
    let hop = if (0.0..1.0).contains(&hop_p) { (hop_p * std::f32::consts::PI).sin() * HOP_HEIGHT } else { 0.0 };
    let studio_alpha = ((t - STUDIO_FADE.0) / (STUDIO_FADE.1 - STUDIO_FADE.0)).clamp(0.0, 1.0);
    LogoFrame { big, lucky_zoom, mis_rise, eyes_open, hop, studio_alpha }
}

/// 로고 시계. `skip`이면 처음부터 끝난 상태 (`?qa`).
#[derive(Debug, Clone, Copy)]
pub struct Logo {
    t: f32,
    done: bool,
}

impl Logo {
    pub fn new(skip: bool) -> Logo {
        Logo { t: 0.0, done: skip }
    }

    /// 한 프레임: 시간이 다 됐거나 무언가를 새로 눌렀으면 끝난다.
    pub fn update(&mut self, dt: f32, pressed: bool) {
        self.t += dt.max(0.0);
        self.done |= pressed || self.t >= LOGO_SECONDS;
    }

    pub fn finished(&self) -> bool {
        self.done
    }

    pub fn time(&self) -> f32 {
        self.t
    }
}

/// 게임을 열 때의 순서: Now Loading → 눌러서 시작 → 로고 → (타이틀).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntroStage {
    /// 그림을 받는 중 (누름은 무시)
    Loading,
    /// 다 받았다. 누르면 로고 (이 누름이 브라우저에서 소리를 켤 수 있게 해 준다)
    TapToStart,
    Logo,
    /// 끝. 다음 프레임 맨 앞에서 타이틀로 간다
    Done,
}

/// 시작 화면들의 시계. 그리기와 상관없어서 시험할 수 있다.
#[derive(Debug, Clone, Copy)]
pub struct Intro {
    stage: IntroStage,
    skip: bool,
    logo: Logo,
    /// 지금 칸에 들어온 뒤 지난 시간 (깜빡임용)
    stage_t: f32,
}

impl Intro {
    /// `skip`(`?qa`)이면 다 받자마자 끝난다: 눌러서 시작도 로고도 없다.
    pub fn new(skip: bool) -> Intro {
        Intro { stage: IntroStage::Loading, skip, logo: Logo::new(false), stage_t: 0.0 }
    }

    pub fn stage(&self) -> IntroStage {
        self.stage
    }

    pub fn logo(&self) -> &Logo {
        &self.logo
    }

    pub fn stage_time(&self) -> f32 {
        self.stage_t
    }

    /// 이번 프레임 **입력을 읽기 전에** 타이틀로 갈지.
    pub fn leave(&self) -> bool {
        self.stage == IntroStage::Done
    }

    /// 한 프레임. 칸을 바꾼 그 누름은 다음 칸에 넘기지 않는다:
    /// "눌러서 시작"을 누른 프레임에 로고가 시작되고, 로고는 다음 프레임의 새 누름부터 받는다.
    pub fn update(&mut self, dt: f32, pressed: bool, assets_ready: bool) {
        let dt = dt.max(0.0);
        self.stage_t += dt;
        let next = match self.stage {
            IntroStage::Loading if assets_ready => Some(if self.skip { IntroStage::Done } else { IntroStage::TapToStart }),
            IntroStage::TapToStart if pressed => Some(IntroStage::Logo),
            IntroStage::Logo => {
                self.logo.update(dt, pressed);
                self.logo.finished().then_some(IntroStage::Done)
            }
            _ => None,
        };
        if let Some(stage) = next {
            self.stage = stage;
            self.stage_t = 0.0;
        }
    }
}

/// "눌러서 시작" 진하기: 1에서 천천히 흐려졌다가 다시 1로 (한 번에 TAP_TO_START_BLINK_SECONDS).
fn tap_alpha(t: f32) -> f32 {
    let wave = (t.max(0.0) / TAP_TO_START_BLINK_SECONDS * std::f32::consts::TAU).cos(); // 1 → -1 → 1
    TAP_MIN_ALPHA + (1.0 - TAP_MIN_ALPHA) * (wave + 1.0) / 2.0
}

/// 미스 아이콘 좌표 → 가상 화면. `dy`만큼 아래로 (숨은 만큼, 뿅 뛰면 위로).
fn mis_point(x: f32, y: f32, dy: f32) -> Vec2 {
    vec2(VIRTUAL_WIDTH / 2.0 + (x - ICON_CENTER_X) * MIS_SCALE, VIRTUAL_HEIGHT + dy + (y - ICON_BOTTOM) * MIS_SCALE)
}

/// 미스 키 (아이콘 T 위 끝부터 몸 아래 선까지, 가상 픽셀)
fn mis_height() -> f32 {
    (ICON_BOTTOM - ICON_TBAR.1) * MIS_SCALE
}

/// 다 숨었을 때 아래로 내려간 거리 (테두리까지 안 보이게 조금 더)
fn mis_hidden_dy() -> f32 {
    mis_height() + 10.0
}

fn draw_rounded_rect(x: f32, y: f32, w: f32, h: f32, r: f32, color: Color) {
    let r = r.min(w / 2.0).min(h / 2.0);
    draw_rectangle(x + r, y, w - 2.0 * r, h, color);
    draw_rectangle(x, y + r, w, h - 2.0 * r, color);
    for (cx, cy) in [(x + r, y + r), (x + w - r, y + r), (x + r, y + h - r), (x + w - r, y + h - r)] {
        draw_circle(cx, cy, r, color);
    }
}

fn draw_mis(frame: &LogoFrame) {
    if frame.mis_rise <= 0.0 {
        return;
    }
    let dy = (1.0 - frame.mis_rise) * mis_hidden_dy() - frame.hop;
    let s = MIS_SCALE;
    // T 안테나 (기둥은 몸 뒤로 들어간다)
    let (a, b, c, e, r) = ICON_TBAR;
    let p = mis_point(a, b, dy);
    draw_rounded_rect(p.x, p.y, (c - a) * s, (e - b) * s, r * s, BLACK);
    let (a, b, c, e) = ICON_TSTEM;
    let p = mis_point(a, b, dy);
    draw_rectangle(p.x, p.y, (c - a) * s, (e - b) * s, BLACK);
    // 몸 (아래는 화면 끝에서 잘린다)
    let (bx, by, br) = ICON_BODY;
    let c0 = mis_point(bx, by, dy);
    draw_circle(c0.x, c0.y, br * s + MIS_LINE_WIDTH / 2.0, MIS_LINE);
    draw_circle(c0.x, c0.y, br * s - MIS_LINE_WIDTH / 2.0, WHITE);
    // 눈: 세로 타원. 깜빡이면 짧은 가로선
    for (ex, ey, rx, ry) in ICON_EYES {
        let c = mis_point(ex, ey, dy);
        if frame.eyes_open {
            draw_ellipse(c.x, c.y, rx * s, ry * s, 0.0, BLACK);
        } else {
            let w = rx * s * 2.6;
            draw_rounded_rect(c.x - w / 2.0, c.y + ry * s * 0.3, w, 5.0, 2.5, BLACK);
        }
    }
    // 아래 양손 동그라미
    for (hx, hy, hr) in ICON_HANDS {
        let c = mis_point(hx, hy, dy);
        draw_circle(c.x, c.y, hr * s + MIS_LINE_WIDTH / 2.0, MIS_LINE);
        draw_circle(c.x, c.y, hr * s - MIS_LINE_WIDTH / 2.0, WHITE);
    }
}

fn baseline(center_y: f32, size: f32, zoom: f32) -> f32 {
    center_y + size * zoom * BASELINE_DROP
}

/// 로고 한 장면 (가상 화면 좌표, 바탕은 이미 빨갛게 칠해져 있다).
pub fn draw_logo(text: &TextPainter, frame: &LogoFrame) {
    if let Some((word, zoom, alpha)) = frame.big {
        let size = if word == "Hey" { HEY_SIZE } else { BIG_SIZE };
        let shadow = Color { a: LETTER_SHADOW.a * alpha, ..LETTER_SHADOW };
        text.plain_zoomed(word, BIG_CENTER.x, baseline(BIG_CENTER.y, size, zoom), size, zoom, Color { a: alpha, ..LETTER }, shadow);
    }
    draw_mis(frame);
    if frame.lucky_zoom > 0.0 {
        let z = frame.lucky_zoom;
        text.plain_zoomed("Lucky Mis!", LUCKY_CENTER.x, baseline(LUCKY_CENTER.y, LUCKY_SIZE, z), LUCKY_SIZE, z, LETTER, LETTER_SHADOW);
    }
    if frame.studio_alpha > 0.0 {
        let a = frame.studio_alpha;
        let shadow = Color { a: LETTER_SHADOW.a * a, ..LETTER_SHADOW };
        text.plain_zoomed("NIRO STUDIO", STUDIO_CENTER.x, baseline(STUDIO_CENTER.y, STUDIO_SIZE, 1.0), STUDIO_SIZE, 1.0, Color { a, ..LETTER }, shadow);
    }
}

/// 스피너 점들의 (자리, 진하기). 한 칸씩 똑딱똑딱 시계 방향으로 돈다 (CSS `steps(8)`과 같다).
/// 가장 진한 점이 앞장서고, 뒤로 갈수록 흐려진다.
fn spinner_dots(t: f32) -> [(Vec2, f32); SPINNER_DOTS] {
    let n = SPINNER_DOTS as f32;
    let step = ((t.max(0.0) / SPINNER_TURN_SECONDS * n).floor() as usize) % SPINNER_DOTS;
    let mut dots = [(Vec2::ZERO, 0.0); SPINNER_DOTS];
    for (k, dot) in dots.iter_mut().enumerate() {
        let a = ((k + step) % SPINNER_DOTS) as f32 / n * std::f32::consts::TAU;
        let pos = SPINNER_CENTER + vec2(a.sin(), -a.cos()) * SPINNER_RADIUS;
        *dot = (pos, (k + 1) as f32 / n);
    }
    dots
}

/// 그림을 아직 다 못 받았을 때(맨 처음, 로고 앞): 도는 점 + "Now Loading...".
fn draw_loading(text: &TextPainter, t: f32) {
    for (pos, alpha) in spinner_dots(t) {
        draw_circle(pos.x, pos.y, SPINNER_DOT, Color { a: alpha, ..LETTER });
    }
    text.plain_zoomed(
        LOADING_TEXT,
        LOADING_TEXT_CENTER.x,
        baseline(LOADING_TEXT_CENTER.y, LOADING_SIZE, 1.0),
        LOADING_SIZE,
        1.0,
        LETTER,
        LETTER_SHADOW,
    );
}

/// 가운데 "눌러서 시작" (천천히 깜빡인다).
fn draw_tap_to_start(text: &TextPainter, t: f32) {
    let a = tap_alpha(t);
    let shadow = Color { a: LETTER_SHADOW.a * a, ..LETTER_SHADOW };
    text.plain_zoomed(TAP_TEXT, BIG_CENTER.x, baseline(BIG_CENTER.y, TAP_SIZE, 1.0), TAP_SIZE, 1.0, Color { a, ..LETTER }, shadow);
}

/// Now Loading → 눌러서 시작 → 로고를 보여 주고, 끝나면(또는 넘기면) `poll`이 돌려준 것(불러온 그림)을 돌려준다.
/// `touch`는 main과 같은 것을 써서, 누를 때 대고 있던 손가락이 다음 칸이나 타이틀에서 새 누름이 되지 않게 한다.
/// 목소리는 아직 없다 (부모님이 art/voice/에 넣으면 로고가 시작될 때 붙인다).
pub async fn play_logo<T>(font: &Font, touch: &mut TouchControls, skip: bool, mut poll: impl FnMut() -> Option<T>) -> T {
    let mut intro = Intro::new(skip);
    let mut ready: Option<T> = None;
    let mut first_frame = true;
    // 횟수 제한 없는 장면 루프지만, 끝나는 길(다 받음 → 누름/qa → 로고 시간)이 Intro 시험으로 확인돼 있다.
    loop {
        if ready.is_none() {
            ready = poll();
        }
        if intro.leave() {
            if let Some(assets) = ready.take() {
                return assets;
            }
        }
        let letterbox = Letterbox::current();
        let tap = touch.idle(&fingers(&letterbox));
        let pressed = tap.is_some() || is_mouse_button_pressed(MouseButton::Left) || any_key_pressed();
        intro.update(get_frame_time().clamp(0.0, crate::config::MAX_FRAME_SECONDS), pressed, ready.is_some());

        // 띠까지 빨강 (남는 곳도 빨간색)
        clear_background(LOGO_RED);
        letterbox.look_at(0.0);
        let text = TextPainter::new(font, letterbox.scale);
        match intro.stage() {
            IntroStage::Loading => draw_loading(&text, get_time() as f32),
            IntroStage::TapToStart => draw_tap_to_start(&text, intro.stage_time()),
            // 로고가 막 끝난 프레임에도 마지막 모습을 그대로 둔다 (빈 빨강으로 깜빡이지 않게)
            IntroStage::Logo => draw_logo(&text, &logo_frame(intro.logo().time())),
            IntroStage::Done if intro.logo().time() > 0.0 => draw_logo(&text, &logo_frame(intro.logo().time())),
            IntroStage::Done => {}
        }
        letterbox.end();
        if first_frame {
            // 첫 그림을 그렸으니 웹페이지의 "Now Loading..." 화면을 치운다.
            super::web::hide_loading_screen();
            first_frame = false;
        }
        next_frame().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::touch::Finger;

    const FRAME: f32 = 1.0 / 60.0;
    /// 넉넉히 잰 글자 한 칸 폭 (글자 크기 비율)
    const CHAR_WIDTH_MAX: f32 = 0.75;

    /// 넉넉히 잰 글자 네모 (가운데 x, 바닥선 y 기준): (왼, 위, 오른, 아래)
    fn text_box(chars: usize, center: Vec2, size: f32, zoom: f32) -> (f32, f32, f32, f32) {
        let shown = size * zoom;
        let w = chars as f32 * shown * CHAR_WIDTH_MAX;
        let base = baseline(center.y, size, zoom);
        (center.x - w / 2.0, base - shown, center.x + w / 2.0, base + shown * 0.25)
    }

    fn inside_screen(b: (f32, f32, f32, f32)) -> bool {
        b.0 >= 0.0 && b.1 >= 0.0 && b.2 <= VIRTUAL_WIDTH && b.3 <= VIRTUAL_HEIGHT
    }

    #[test]
    fn length_is_five_to_six_seconds() {
        assert!((5.0..=6.0).contains(&LOGO_SECONDS));
        assert!(LOGO_COUNT_END < LOGO_HEY_END && LOGO_HEY_END < LOGO_SECONDS);
        assert!(HOP_START + HOP_SECONDS <= LOGO_SECONDS, "뿅이 끝난 뒤에 끝난다");
    }

    #[test]
    fn one_two_three_then_hey_then_lucky_mis() {
        let word = |t: f32| logo_frame(t).big.map(|b| b.0);
        assert_eq!(word(0.4), Some("1"));
        assert_eq!(word(1.25), Some("2"));
        assert_eq!(word(2.1), Some("3"));
        assert_eq!(word(3.0), Some("Hey"));
        assert_eq!(word(3.7), None, "Hey가 사라진 뒤에는 큰 글자가 없다");
        assert_eq!(logo_frame(3.0).lucky_zoom, 0.0);
        assert!(logo_frame(4.0).lucky_zoom > 0.9);
        assert_eq!(logo_frame(3.0).mis_rise, 0.0, "Hey 때는 미스가 안 보인다");
        assert_eq!(logo_frame(4.5).mis_rise, 1.0, "다 올라왔다");
        assert_eq!(logo_frame(3.0).studio_alpha, 0.0);
        assert_eq!(logo_frame(5.0).studio_alpha, 1.0, "NIRO STUDIO가 보인다");
    }

    #[test]
    fn numbers_pop_a_little_bigger_then_settle_and_shrink_away() {
        let zooms: Vec<f32> = (0..50).map(|i| logo_frame(i as f32 * 0.01).big.unwrap().1).collect();
        let peak = zooms.iter().cloned().fold(0.0, f32::max);
        assert!(peak > 1.08 && peak < 1.3, "살짝만 커진다: {peak}");
        assert!((logo_frame(0.6).big.unwrap().1 - 1.0).abs() < 0.01, "튄 뒤에는 원래 크기");
        // 다음 숫자가 나오기 바로 전에 이전 숫자는 거의 안 보일 만큼 작다
        let (w, z, a) = logo_frame(LOGO_COUNT_END / 3.0 - 0.01).big.unwrap();
        assert_eq!(w, "1");
        assert!(z < 0.1 && a < 0.1);
    }

    #[test]
    fn eyes_blink_twice_then_mis_hops() {
        let mut closings = 0;
        let mut was_open = true;
        for i in 0..(LOGO_SECONDS / FRAME) as usize {
            let f = logo_frame(i as f32 * FRAME);
            if was_open && !f.eyes_open {
                closings += 1;
                assert!(f.mis_rise >= 1.0, "다 올라온 뒤에 깜빡인다");
            }
            was_open = f.eyes_open;
        }
        assert_eq!(closings, 2);
        assert!(logo_frame(HOP_START + HOP_SECONDS / 2.0).hop > HOP_HEIGHT * 0.9);
        assert_eq!(logo_frame(HOP_START - 0.05).hop, 0.0);
        assert_eq!(logo_frame(LOGO_SECONDS - 0.01).hop, 0.0, "끝날 때는 내려와 있다");
    }

    #[test]
    fn logo_ends_by_itself_after_its_length() {
        let mut logo = Logo::new(false);
        let mut frames = 0;
        // 횟수 제한: 10초어치 넘게 돌면 실패
        while !logo.finished() && frames < 600 {
            logo.update(FRAME, false);
            frames += 1;
        }
        assert!(logo.finished());
        let seconds = frames as f32 * FRAME;
        assert!((seconds - LOGO_SECONDS).abs() < 0.05, "{seconds}초");
    }

    #[test]
    fn any_press_skips_right_away_and_qa_skips_from_the_start() {
        let mut logo = Logo::new(false);
        logo.update(FRAME, false);
        assert!(!logo.finished());
        logo.update(FRAME, true);
        assert!(logo.finished(), "누르면 바로 끝");
        assert!(Logo::new(true).finished(), "?qa는 처음부터 건너뛴다");
    }

    #[test]
    fn the_skipping_press_does_not_reach_the_title() {
        // play_logo 순서: 이번 프레임 입력을 읽기 전에 leave → (안 떠나면) 입력 읽고 update → 그리기 → 다음 프레임
        let mut intro = started_logo();
        assert!(!intro.leave());
        intro.update(FRAME, true, true); // 이 프레임의 누름은 로고가 먹는다
        assert_eq!(intro.stage(), IntroStage::Done);
        // 다음 프레임 맨 앞에서 떠난다: 그 프레임 입력은 아직 아무도 안 읽었고, 지난 누름은 이미 지나갔다
        assert!(intro.leave());
    }

    /// 다 받고 "눌러서 시작"을 눌러 로고가 막 시작된 Intro
    fn started_logo() -> Intro {
        let mut intro = Intro::new(false);
        intro.update(FRAME, false, true);
        intro.update(FRAME, true, true);
        assert_eq!(intro.stage(), IntroStage::Logo);
        intro
    }

    #[test]
    fn loading_then_tap_to_start_then_logo() {
        let mut intro = Intro::new(false);
        // 받는 동안은 눌러도 그대로
        for _ in 0..10 {
            intro.update(FRAME, true, false);
            assert_eq!(intro.stage(), IntroStage::Loading);
        }
        // 다 받으면 "눌러서 시작" (받은 그 프레임의 누름으로 건너뛰지 않는다)
        intro.update(FRAME, true, true);
        assert_eq!(intro.stage(), IntroStage::TapToStart);
        // 누를 때까지 기다린다 (횟수 제한: 20초어치)
        for _ in 0..1200 {
            intro.update(FRAME, false, true);
        }
        assert_eq!(intro.stage(), IntroStage::TapToStart);
        assert!(!intro.leave());
        intro.update(FRAME, true, true);
        assert_eq!(intro.stage(), IntroStage::Logo);
    }

    #[test]
    fn the_start_press_does_not_skip_the_logo() {
        let mut intro = started_logo();
        assert_eq!(intro.logo().time(), 0.0, "누른 프레임에는 로고 시계가 안 간다");
        assert!(!intro.logo().finished(), "시작 누름이 로고 넘기기가 되지 않는다");
        // 손을 떼면 로고가 끝까지 나오고 (횟수 제한: 10초어치) 그다음 타이틀로 간다
        let mut frames = 0;
        while !intro.leave() && frames < 600 {
            intro.update(FRAME, false, true);
            frames += 1;
        }
        assert!(intro.leave());
        assert!(((frames as f32 * FRAME) - LOGO_SECONDS).abs() < 0.05);
    }

    #[test]
    fn qa_skips_tap_to_start_and_logo_but_still_waits_for_assets() {
        let mut intro = Intro::new(true);
        intro.update(FRAME, true, false);
        assert_eq!(intro.stage(), IntroStage::Loading);
        assert!(!intro.leave());
        intro.update(FRAME, false, true);
        assert!(intro.leave(), "?qa는 다 받자마자 타이틀");
        assert_eq!(intro.logo().time(), 0.0);
    }

    #[test]
    fn tap_to_start_blinks_slowly_but_never_disappears() {
        assert!((tap_alpha(0.0) - 1.0).abs() < 0.001, "처음에는 진하다");
        assert!((tap_alpha(TAP_TO_START_BLINK_SECONDS / 2.0) - TAP_MIN_ALPHA).abs() < 0.001);
        assert!((tap_alpha(TAP_TO_START_BLINK_SECONDS) - 1.0).abs() < 0.001);
        assert!(TAP_TO_START_BLINK_SECONDS >= 1.0, "천천히");
        for i in 0..200 {
            let a = tap_alpha(i as f32 * 0.02);
            assert!((TAP_MIN_ALPHA - 0.001..=1.001).contains(&a));
        }
        // 폰·PC 모두 가운데, 화면 안 (한글 5글자 + 띄어쓰기)
        assert!(inside_screen(text_box(TAP_TEXT.chars().count(), BIG_CENTER, TAP_SIZE, 1.0)));
        assert_eq!(BIG_CENTER, vec2(VIRTUAL_WIDTH / 2.0, VIRTUAL_HEIGHT / 2.0));
    }

    #[test]
    fn finger_held_from_the_logo_is_not_a_new_tap_on_the_title() {
        // main과 로고가 같은 TouchControls를 쓴다: 로고를 넘긴 손가락을 계속 대고 있어도 타이틀에서 누름이 아니다
        let mut touch = TouchControls::new();
        let down = Finger { id: 7, phase: TouchPhase::Started, pos: vec2(480.0, 270.0) };
        assert!(touch.idle(&[down]).is_some(), "로고에서는 누름 = 넘기기");
        let held = Finger { phase: TouchPhase::Stationary, ..down };
        assert!(touch.idle(&[held]).is_none(), "타이틀에서는 새 누름이 아니다");
        let fresh = Finger { id: 8, phase: TouchPhase::Started, pos: vec2(100.0, 100.0) };
        assert!(touch.idle(&[held, fresh]).is_some(), "새로 댄 손가락은 누름");
    }

    #[test]
    fn nothing_is_cut_off_on_any_screen() {
        // 가장 크게 튈 때(약 1.16배)도 960×540 안에 들어온다
        let peak = (0..100).map(|i| pop(i as f32 / 100.0)).fold(0.0, f32::max);
        assert!(inside_screen(text_box(1, BIG_CENTER, BIG_SIZE, peak)));
        assert!(inside_screen(text_box(3, BIG_CENTER, HEY_SIZE, peak)));
        assert!(inside_screen(text_box(10, LUCKY_CENTER, LUCKY_SIZE, peak)));
        assert!(inside_screen(text_box(11, STUDIO_CENTER, STUDIO_SIZE, 1.0)));
        assert!(inside_screen(text_box(LOADING_TEXT.chars().count(), LOADING_TEXT_CENTER, LOADING_SIZE, 1.0)));
        // 미스: 뿅 뛰어도 "Lucky Mis!" 밑이고, 손은 화면 안, "NIRO STUDIO"와 안 겹친다
        let lucky = text_box(10, LUCKY_CENTER, LUCKY_SIZE, peak);
        let mis_top = VIRTUAL_HEIGHT - mis_height() - HOP_HEIGHT;
        assert!(mis_top > lucky.3, "미스 {mis_top} / 글자 아래 {}", lucky.3);
        let (hx, _, hr) = ICON_HANDS[0];
        let left = mis_point(hx - hr, 0.0, 0.0).x;
        let (hx, _, hr) = ICON_HANDS[1];
        let right = mis_point(hx + hr, 0.0, 0.0).x;
        assert!(left > 0.0 && right < VIRTUAL_WIDTH);
        assert!(right + 10.0 < text_box(11, STUDIO_CENTER, STUDIO_SIZE, 1.0).0, "NIRO STUDIO와 손이 안 겹친다");
        // 폰·PC 창에서 가상 화면 전체가 창 안에 들어온다 (남는 곳은 빨강)
        for (w, h) in [(740.0, 360.0), (812.0, 375.0), (960.0, 540.0)] {
            let lb = Letterbox::for_window(w, h);
            assert!(lb.offset.x >= -0.01 && lb.offset.y >= -0.01, "{w}×{h}");
            assert!(lb.offset.x + VIRTUAL_WIDTH * lb.scale <= w + 0.01);
            assert!(lb.offset.y + VIRTUAL_HEIGHT * lb.scale <= h + 0.01);
        }
    }

    #[test]
    fn loading_spinner_turns_once_a_second_above_the_words() {
        let dots = spinner_dots(0.0);
        // 점은 원 위에 있고, 화면 안, 글자보다 위에 있다
        let words = text_box(LOADING_TEXT.chars().count(), LOADING_TEXT_CENTER, LOADING_SIZE, 1.0);
        for (pos, alpha) in dots {
            assert!((pos.distance(SPINNER_CENTER) - SPINNER_RADIUS).abs() < 0.01);
            assert!(pos.y + SPINNER_DOT < words.1, "점이 글자와 안 겹친다");
            assert!(pos.y - SPINNER_DOT > 0.0);
            assert!(alpha > 0.0 && alpha <= 1.0);
        }
        // 가장 진한 점(마지막)이 한 칸(1/8초) 뒤에 시계 방향으로 한 칸 간다
        let lead = |t: f32| spinner_dots(t)[SPINNER_DOTS - 1].0;
        let step = SPINNER_TURN_SECONDS / SPINNER_DOTS as f32;
        assert_eq!(lead(0.0), lead(step * 0.5), "한 칸 안에서는 그대로");
        let (a, b) = (lead(0.0) - SPINNER_CENTER, lead(step * 1.5) - SPINNER_CENTER);
        assert!(a.perp_dot(b) > 0.0, "화면(아래가 +y)에서 시계 방향");
        // 한 바퀴 돌면 제자리
        assert!(lead(0.01).distance(lead(SPINNER_TURN_SECONDS + 0.01)) < 0.01);
        // 글자는 Jua에 있는 영어와 점만 쓴다
        assert!(LOADING_TEXT.chars().all(|c| c.is_ascii_alphabetic() || c == ' ' || c == '.'));
    }

    #[test]
    fn hidden_mis_is_fully_below_the_screen() {
        let top = mis_point(0.0, ICON_TBAR.1, mis_hidden_dy()).y;
        assert!(top >= VIRTUAL_HEIGHT, "다 숨으면 T도 안 보인다: {top}");
    }
}
