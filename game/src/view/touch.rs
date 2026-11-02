//! 폰 터치 조작 (D16): 왼쪽 떠다니는 조이스틱 + 오른쪽 큰 버튼들 + 왼쪽 위 "고르기" 버튼.
//! 위쪽은 판정(손가락 추적·데드존·버튼 영역)만 하는 작은 함수들이라 단위 테스트를 한다.
//! 아래쪽 `draw_*`는 판정 결과를 보여 주기만 한다. 좌표는 모두 가상 화면(960×540) 기준.
//! 게임 규칙은 없다: 결과는 키보드와 같은 `PlayerInput` 모양(왼쪽/오른쪽/점프 새로 누름)이다.

use super::hud::draw_pill;
use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::text::TextPainter;
use macroquad::prelude::*;

// ---- 크기·위치 (8살 엄지 기준으로 크게) ----
/// 조이스틱 받침 반지름 (지름 170)
pub const JOYSTICK_RADIUS: f32 = 85.0;
/// 손잡이 반지름
const KNOB_RADIUS: f32 = 40.0;
/// 받침 반지름의 이 비율보다 덜 밀면 멈춤 (데드존)
pub const JOYSTICK_DEADZONE: f32 = 0.25;
/// 이 x보다 왼쪽에 손가락을 대면 조이스틱 (가상 화면 왼쪽 절반)
pub const JOYSTICK_AREA_RIGHT: f32 = VIRTUAL_WIDTH / 2.0;

/// 오른쪽 버튼 반지름 (지름 136)
const BUTTON_RADIUS: f32 = 68.0;
/// 화면 오른쪽·아래 끝에서 버튼까지 떨어진 거리
const BUTTON_EDGE_GAP: f32 = 22.0;
/// 손가락이 버튼 그림보다 조금 밖을 눌러도 눌린 것으로 친다 (아이 손가락은 빗나가기 쉽다)
const BUTTON_TOUCH_SLOP: f32 = 12.0;

/// "◀ 고르기" 버튼: 정보판(위쪽 줄, y 10~66) 바로 아래 왼쪽
pub const BACK_BUTTON: Rect = Rect::new(12.0, 78.0, 156.0, 60.0);

// ---- 색 ----
const BASE_FILL: Color = Color::new(1.0, 1.0, 1.0, 0.16);
const BASE_LINE: Color = Color::new(1.0, 1.0, 1.0, 0.55);
const KNOB_FILL: Color = Color::new(1.0, 1.0, 1.0, 0.60);
const KNOB_LINE: Color = Color::new(0.13, 0.16, 0.36, 0.55);
const ARROW_HINT: Color = Color::new(1.0, 1.0, 1.0, 0.55);
const BUTTON_FILL: Color = Color::new(0.10, 0.13, 0.32, 0.42);
const BUTTON_FILL_HELD: Color = Color::new(1.0, 0.70, 0.10, 0.60);
const BUTTON_LINE: Color = Color::new(1.0, 1.0, 1.0, 0.75);
const BUTTON_ICON: Color = Color::new(1.0, 1.0, 1.0, 0.95);
const BACK_FILL: Color = Color::new(0.10, 0.13, 0.32, 0.55);
const LINE_WIDTH: f32 = 5.0;

/// 오른쪽 버튼 종류. M4에 공격, M6에 캐기를 여기와 `PLAY_BUTTONS`에 더하면 된다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonKind {
    Jump,
}

impl ButtonKind {
    fn label(self) -> &'static str {
        match self {
            ButtonKind::Jump => "점프",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TouchButton {
    pub kind: ButtonKind,
    pub center: Vec2,
    pub radius: f32,
}

/// 플레이 화면 오른쪽 버튼 목록. 지금은 점프 하나 (오른쪽 아래 구석).
/// 버튼이 늘면 점프 왼쪽·위로 하나씩 놓는다.
pub const PLAY_BUTTONS: [TouchButton; 1] = [TouchButton {
    kind: ButtonKind::Jump,
    center: Vec2::new(
        VIRTUAL_WIDTH - BUTTON_EDGE_GAP - BUTTON_RADIUS,
        VIRTUAL_HEIGHT - BUTTON_EDGE_GAP - BUTTON_RADIUS,
    ),
    radius: BUTTON_RADIUS,
}];

/// 누른 곳(가상 좌표)이 어느 버튼인지. 겹치면 앞쪽 버튼.
pub fn button_at(buttons: &[TouchButton], point: Vec2) -> Option<ButtonKind> {
    buttons
        .iter()
        .find(|b| b.center.distance(point) <= b.radius + BUTTON_TOUCH_SLOP)
        .map(|b| b.kind)
}

/// 받침 가운데: 손가락을 댄 자리를 받침이 화면(가상 960×540) 안에 다 보이게 당긴다.
/// 손잡이는 `knob_offset`으로 손가락 쪽으로 밀리므로 방향은 그대로 맞는다 (REVIEW-003 3번).
pub fn joystick_base(finger: Vec2) -> Vec2 {
    let r = JOYSTICK_RADIUS;
    finger.clamp(vec2(r, r), vec2(VIRTUAL_WIDTH - r, VIRTUAL_HEIGHT - r))
}

/// 손잡이 자리: 받침 가운데에서 손가락 쪽으로, 받침 반지름까지만.
pub fn knob_offset(base: Vec2, finger: Vec2, radius: f32) -> Vec2 {
    (finger - base).clamp_length_max(radius)
}

/// 손잡이 가로 위치 → (왼쪽, 오른쪽). 세로는 쓰지 않는다. 데드존 안이면 둘 다 false.
pub fn joystick_direction(knob_x: f32, radius: f32, deadzone: f32) -> (bool, bool) {
    let dead = radius * deadzone;
    (knob_x < -dead, knob_x > dead)
}

/// "가로로 돌려 주세요" 안내를 띄울지: 터치 모드이고 화면이 세로로 길 때만.
/// PC(키보드만)는 창을 좁고 길게 만들어도 안 띄운다 (US-035, US-036).
pub fn show_rotate_hint(touch_mode: bool, width: f32, height: f32) -> bool {
    touch_mode && height > width
}

/// 터치 화면 모드 켜기/끄기: 터치가 들어오면 켜고, 키보드를 누르면 끈다.
/// 키보드만 쓰는 PC에서는 계속 꺼져 있어서 조이스틱·버튼이 안 보인다.
pub fn next_touch_mode(current: bool, touched: bool, key_pressed: bool) -> bool {
    if touched {
        true
    } else if key_pressed {
        false
    } else {
        current
    }
}

/// 손가락 하나 (가상 화면 좌표).
#[derive(Debug, Clone, Copy)]
pub struct Finger {
    pub id: u64,
    pub phase: TouchPhase,
    pub pos: Vec2,
}

impl Finger {
    fn lifted(&self) -> bool {
        matches!(self.phase, TouchPhase::Ended | TouchPhase::Cancelled)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Joystick {
    /// 조이스틱을 잡은 손가락
    pub finger: u64,
    /// 받침 가운데 (처음 댄 자리. 화면 끝이면 받침이 다 보이게 안쪽으로)
    pub base: Vec2,
    /// 받침 가운데에서 손잡이까지 (반지름 안)
    pub knob: Vec2,
}

/// 이번 프레임 터치 결과. 키보드 입력과 같은 규칙: 점프·돌아가기는 "새로 누른" 프레임에만 true.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TouchFrame {
    pub left: bool,
    pub right: bool,
    pub jump_pressed: bool,
    pub back_pressed: bool,
}

/// 손가락들을 기억하며 조이스틱·버튼을 판정한다.
#[derive(Debug, Default)]
pub struct TouchControls {
    pub joystick: Option<Joystick>,
    /// 버튼을 누르고 있는 손가락들 (버튼 눌림 그림용)
    held: Vec<(u64, ButtonKind)>,
    /// 지난 프레임에 화면에 있던 손가락 id. 여기 없는 id가 "새 손가락"이다.
    known: Vec<u64>,
}

impl TouchControls {
    pub fn new() -> TouchControls {
        TouchControls::default()
    }

    pub fn is_held(&self, kind: ButtonKind) -> bool {
        self.held.iter().any(|(_, k)| *k == kind)
    }

    /// 플레이 화면이 아닐 때 (메뉴, 세로 안내): 조이스틱·버튼은 놓고, 화면에 있는 손가락만 기억한다.
    /// 그래서 메뉴에서 누른 채로 플레이가 시작되어도 그 손가락이 점프나 조이스틱이 되지 않는다.
    pub fn idle(&mut self, fingers: &[Finger]) {
        self.joystick = None;
        self.held.clear();
        self.known = fingers.iter().filter(|f| !f.lifted()).map(|f| f.id).collect();
    }

    /// 플레이 화면에서 한 프레임.
    pub fn update(&mut self, fingers: &[Finger], buttons: &[TouchButton], show_back: bool) -> TouchFrame {
        let mut out = TouchFrame::default();
        for f in fingers {
            // 같은 번호가 이번 프레임에 다시 "대기 시작"이면, 예전 손가락은 떼어졌고 새 손가락이다
            // (안드로이드는 뗀 번호를 바로 다시 쓰고, macroquad는 한 프레임 안의 뗌을 덮어쓸 수 있다).
            if f.phase == TouchPhase::Started && self.known.contains(&f.id) {
                self.known.retain(|id| *id != f.id);
                self.held.retain(|(id, _)| *id != f.id);
                if self.joystick.is_some_and(|j| j.finger == f.id) {
                    self.joystick = None;
                }
            }
            // 새 손가락: 댄 자리로 할 일을 정한다. 한 프레임 안에 대고 뗀 짧은 톡도 여기서 잡힌다
            // (macroquad는 같은 프레임의 시작·끝을 하나로 합쳐 Ended만 알려 줄 수 있다).
            if !self.known.contains(&f.id) {
                if show_back && BACK_BUTTON.contains(f.pos) {
                    out.back_pressed = true;
                } else if let Some(kind) = button_at(buttons, f.pos) {
                    match kind {
                        ButtonKind::Jump => out.jump_pressed = true,
                    }
                    if !f.lifted() {
                        self.held.push((f.id, kind));
                    }
                } else if f.pos.x < JOYSTICK_AREA_RIGHT && self.joystick.is_none() && !f.lifted() {
                    self.joystick = Some(Joystick { finger: f.id, base: joystick_base(f.pos), knob: Vec2::ZERO });
                }
            }
            if let Some(j) = self.joystick.as_mut() {
                if j.finger == f.id {
                    j.knob = knob_offset(j.base, f.pos, JOYSTICK_RADIUS);
                }
            }
        }

        // 뗀 손가락, 그리고 소리 없이 사라진 손가락을 놓는다.
        let present: Vec<u64> = fingers.iter().filter(|f| !f.lifted()).map(|f| f.id).collect();
        self.held.retain(|(id, _)| present.contains(id));
        if self.joystick.is_some_and(|j| !present.contains(&j.finger)) {
            self.joystick = None;
        }
        self.known = present;

        if let Some(j) = self.joystick {
            (out.left, out.right) = joystick_direction(j.knob.x, JOYSTICK_RADIUS, JOYSTICK_DEADZONE);
        }
        out
    }
}

// ---- 그리기 (판정 결과를 보여 주기만) ----

/// 위·왼쪽·오른쪽을 가리키는 세모. `dir`은 가리키는 방향 (단위 벡터).
fn draw_arrow_head(tip: Vec2, dir: Vec2, size: f32, color: Color) {
    let side = vec2(-dir.y, dir.x) * size * 0.6;
    let back = tip - dir * size;
    draw_triangle(tip, back + side, back - side, color);
}

fn draw_joystick(j: &Joystick) {
    let b = j.base;
    draw_circle(b.x, b.y, JOYSTICK_RADIUS, BASE_FILL);
    draw_circle_lines(b.x, b.y, JOYSTICK_RADIUS, LINE_WIDTH, BASE_LINE);
    // 좌우로 미는 것이라는 표시
    let r = JOYSTICK_RADIUS - 10.0;
    draw_arrow_head(b + vec2(-r, 0.0), vec2(-1.0, 0.0), 18.0, ARROW_HINT);
    draw_arrow_head(b + vec2(r, 0.0), vec2(1.0, 0.0), 18.0, ARROW_HINT);
    let k = b + j.knob;
    draw_circle(k.x, k.y, KNOB_RADIUS, KNOB_FILL);
    draw_circle_lines(k.x, k.y, KNOB_RADIUS, 3.0, KNOB_LINE);
}

fn draw_button(text: &TextPainter, b: &TouchButton, held: bool) {
    // 누르고 있으면 살짝 작아지고 주황빛
    let r = if held { b.radius * 0.94 } else { b.radius };
    let c = b.center;
    draw_circle(c.x, c.y, r, if held { BUTTON_FILL_HELD } else { BUTTON_FILL });
    draw_circle_lines(c.x, c.y, r, LINE_WIDTH, BUTTON_LINE);
    match b.kind {
        ButtonKind::Jump => draw_arrow_head(c + vec2(0.0, -r * 0.62), vec2(0.0, -1.0), r * 0.42, BUTTON_ICON),
    }
    text.centered(b.kind.label(), c.x, c.y + r * 0.42, 32.0, BUTTON_ICON);
}

fn draw_back_button(text: &TextPainter) {
    let r = BACK_BUTTON;
    draw_pill(r.x, r.y, r.w, r.h, BACK_FILL);
    let mid_y = r.y + r.h / 2.0;
    draw_arrow_head(vec2(r.x + 18.0, mid_y), vec2(-1.0, 0.0), 22.0, BUTTON_ICON);
    text.centered("고르기", r.x + r.w / 2.0 + 16.0, mid_y + 11.0, 32.0, BUTTON_ICON);
}

/// 플레이 화면 위에 조이스틱·버튼·"고르기" 버튼 (터치 모드일 때만 부른다).
pub fn draw_touch_controls(text: &TextPainter, controls: &TouchControls, buttons: &[TouchButton]) {
    if let Some(j) = &controls.joystick {
        draw_joystick(j);
    }
    for b in buttons {
        draw_button(text, b, controls.is_held(b.kind));
    }
    draw_back_button(text);
}

/// 폰 모양 (둥근 테두리 + 화면 + 아래 동그라미). `size`는 (가로, 세로).
fn draw_phone(center: Vec2, size: Vec2, line: f32) {
    let tl = center - size / 2.0;
    draw_rectangle(tl.x, tl.y, size.x, size.y, Color::new(1.0, 1.0, 1.0, 0.15));
    draw_rectangle_lines(tl.x, tl.y, size.x, size.y, line, WHITE);
    let dot = if size.y > size.x { center + vec2(0.0, size.y * 0.38) } else { center + vec2(size.x * 0.38, 0.0) };
    draw_circle(dot.x, dot.y, line * 0.8, WHITE);
}

/// "가로로 돌려 주세요" 안내. 창 좌표(기본 카메라)로 화면 전체에 그린다.
/// `text`는 창 1픽셀 = 크기 1로 만든 붓 (TextPainter::new(font, dpi)).
pub fn draw_rotate_hint(text: &TextPainter, width: f32, height: f32) {
    draw_rectangle(0.0, 0.0, width, height, Color::new(0.10, 0.13, 0.32, 0.88));
    let s = width.min(height) / 400.0;
    let c = vec2(width / 2.0, height * 0.42);
    let line = (4.0 * s).max(2.0);
    // 세로 폰 → 둥근 화살표 → 가로 폰
    let gap = 95.0 * s;
    draw_phone(c - vec2(gap, 0.0), vec2(56.0, 96.0) * s, line);
    draw_phone(c + vec2(gap, 0.0), vec2(96.0, 56.0) * s, line);
    let arc_c = c - vec2(0.0, 10.0 * s);
    let arc_r = 48.0 * s;
    const STEPS: usize = 16;
    let (a0, a1) = (std::f32::consts::PI * 1.15, std::f32::consts::PI * 1.85);
    let point = |a: f32| arc_c + vec2(a.cos(), a.sin()) * arc_r;
    for i in 0..STEPS {
        let p = point(a0 + (a1 - a0) * i as f32 / STEPS as f32);
        let q = point(a0 + (a1 - a0) * (i + 1) as f32 / STEPS as f32);
        draw_line(p.x, p.y, q.x, q.y, line * 1.5, WHITE);
    }
    let end = point(a1);
    let dir = vec2(-a1.sin(), a1.cos()); // 원을 따라 도는 방향
    draw_arrow_head(end + dir * 10.0 * s, dir, 22.0 * s, WHITE);

    let size = (width * 0.1).clamp(28.0, 72.0);
    text.centered("가로로 돌려 주세요", width / 2.0, c.y + 100.0 * s + size, size, WHITE);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finger(id: u64, phase: TouchPhase, x: f32, y: f32) -> Finger {
        Finger { id, phase, pos: vec2(x, y) }
    }

    const JUMP: Vec2 = PLAY_BUTTONS[0].center;

    #[test]
    fn buttons_are_big_and_on_screen() {
        for b in PLAY_BUTTONS {
            assert!(b.radius * 2.0 >= 110.0, "8살 엄지 기준 지름 110 이상");
            assert!(b.center.x - b.radius >= JOYSTICK_AREA_RIGHT, "조이스틱 자리(왼쪽 절반)와 안 겹친다");
            assert!(b.center.x + b.radius <= VIRTUAL_WIDTH && b.center.y + b.radius <= VIRTUAL_HEIGHT);
        }
        const { assert!(JOYSTICK_RADIUS * 2.0 >= 110.0) };
        // 고르기 버튼은 정보판(y 10~66) 아래
        const { assert!(BACK_BUTTON.y >= 66.0) };
    }

    #[test]
    fn button_hit_test() {
        assert_eq!(button_at(&PLAY_BUTTONS, JUMP), Some(ButtonKind::Jump));
        assert_eq!(button_at(&PLAY_BUTTONS, JUMP + vec2(0.0, -PLAY_BUTTONS[0].radius)), Some(ButtonKind::Jump));
        assert_eq!(button_at(&PLAY_BUTTONS, JUMP + vec2(-PLAY_BUTTONS[0].radius - 30.0, 0.0)), None);
        assert_eq!(button_at(&PLAY_BUTTONS, vec2(100.0, 400.0)), None);
    }

    #[test]
    fn knob_stays_inside_base() {
        let base = vec2(200.0, 400.0);
        assert_eq!(knob_offset(base, vec2(230.0, 400.0), 85.0), vec2(30.0, 0.0));
        let far = knob_offset(base, vec2(500.0, 400.0), 85.0);
        assert!((far.length() - 85.0).abs() < 0.001 && far.x > 0.0);
    }

    #[test]
    fn deadzone_and_direction() {
        let r = 80.0; // 데드존 20
        assert_eq!(joystick_direction(0.0, r, 0.25), (false, false));
        assert_eq!(joystick_direction(19.0, r, 0.25), (false, false));
        assert_eq!(joystick_direction(-19.0, r, 0.25), (false, false));
        assert_eq!(joystick_direction(21.0, r, 0.25), (false, true));
        assert_eq!(joystick_direction(-21.0, r, 0.25), (true, false));
    }

    #[test]
    fn touch_mode_follows_last_input() {
        assert!(!next_touch_mode(false, false, false), "키보드만 쓰는 PC는 꺼진 채");
        assert!(next_touch_mode(false, true, false));
        assert!(next_touch_mode(true, false, false), "손을 떼도 켜진 채");
        assert!(!next_touch_mode(true, false, true), "키보드를 누르면 숨긴다");
    }

    #[test]
    fn rotate_hint_only_for_touch_portrait() {
        assert!(show_rotate_hint(true, 390.0, 844.0));
        assert!(!show_rotate_hint(true, 844.0, 390.0));
        assert!(!show_rotate_hint(true, 500.0, 500.0));
        assert!(!show_rotate_hint(false, 390.0, 844.0), "PC 창을 좁고 길게 해도 안 나온다");
    }

    #[test]
    fn joystick_appears_where_finger_lands_and_walks() {
        let mut t = TouchControls::new();
        let f = t.update(&[finger(1, TouchPhase::Started, 200.0, 400.0)], &PLAY_BUTTONS, true);
        assert_eq!(f, TouchFrame::default(), "대자마자는 멈춤");
        assert_eq!(t.joystick.unwrap().base, vec2(200.0, 400.0));

        let f = t.update(&[finger(1, TouchPhase::Moved, 260.0, 380.0)], &PLAY_BUTTONS, true);
        assert!(f.right && !f.left);
        let f = t.update(&[finger(1, TouchPhase::Moved, 150.0, 430.0)], &PLAY_BUTTONS, true);
        assert!(f.left && !f.right);
        let f = t.update(&[finger(1, TouchPhase::Stationary, 205.0, 300.0)], &PLAY_BUTTONS, true);
        assert!(!f.left && !f.right, "위아래로만 밀면 데드존 → 멈춤");

        let f = t.update(&[finger(1, TouchPhase::Ended, 300.0, 400.0)], &PLAY_BUTTONS, true);
        assert_eq!(f, TouchFrame::default(), "손을 떼면 멈춤");
        assert!(t.joystick.is_none());
        t.update(&[], &PLAY_BUTTONS, true);
        assert!(t.joystick.is_none());
    }

    #[test]
    fn right_half_touch_is_not_joystick() {
        let mut t = TouchControls::new();
        t.update(&[finger(1, TouchPhase::Started, 600.0, 300.0)], &PLAY_BUTTONS, true);
        assert!(t.joystick.is_none());
    }

    #[test]
    fn jump_fires_once_while_held() {
        let mut t = TouchControls::new();
        let f = t.update(&[finger(7, TouchPhase::Started, JUMP.x, JUMP.y)], &PLAY_BUTTONS, true);
        assert!(f.jump_pressed);
        assert!(t.is_held(ButtonKind::Jump));
        for _ in 0..30 {
            let f = t.update(&[finger(7, TouchPhase::Stationary, JUMP.x, JUMP.y)], &PLAY_BUTTONS, true);
            assert!(!f.jump_pressed, "꾹 눌러도 다시 점프하지 않는다");
        }
        t.update(&[finger(7, TouchPhase::Ended, JUMP.x, JUMP.y)], &PLAY_BUTTONS, true);
        assert!(!t.is_held(ButtonKind::Jump));
        let f = t.update(&[finger(8, TouchPhase::Started, JUMP.x, JUMP.y)], &PLAY_BUTTONS, true);
        assert!(f.jump_pressed, "떼고 다시 누르면 또 점프");
    }

    #[test]
    fn quick_tap_within_one_frame_still_jumps() {
        let mut t = TouchControls::new();
        let f = t.update(&[finger(3, TouchPhase::Ended, JUMP.x, JUMP.y)], &PLAY_BUTTONS, true);
        assert!(f.jump_pressed);
        assert!(!t.is_held(ButtonKind::Jump));
    }

    #[test]
    fn joystick_and_jump_together_do_not_mix() {
        let mut t = TouchControls::new();
        t.update(&[finger(1, TouchPhase::Started, 200.0, 400.0)], &PLAY_BUTTONS, true);
        t.update(&[finger(1, TouchPhase::Moved, 270.0, 400.0)], &PLAY_BUTTONS, true);
        // 걷는 중에 다른 손가락으로 점프
        let f = t.update(
            &[finger(1, TouchPhase::Stationary, 270.0, 400.0), finger(2, TouchPhase::Started, JUMP.x, JUMP.y)],
            &PLAY_BUTTONS,
            true,
        );
        assert!(f.right && f.jump_pressed);
        assert_eq!(t.joystick.unwrap().finger, 1, "점프 손가락이 조이스틱을 뺏지 않는다");
        // 점프 손가락이 왼쪽으로 미끄러져 와도 조이스틱은 1번 손가락 그대로
        let f = t.update(
            &[finger(1, TouchPhase::Stationary, 270.0, 400.0), finger(2, TouchPhase::Moved, 100.0, 400.0)],
            &PLAY_BUTTONS,
            true,
        );
        assert!(f.right && !f.left && !f.jump_pressed);
        // 왼쪽에 두 번째 손가락을 대도 조이스틱은 하나
        let f = t.update(
            &[finger(1, TouchPhase::Stationary, 270.0, 400.0), finger(3, TouchPhase::Started, 50.0, 300.0)],
            &PLAY_BUTTONS,
            true,
        );
        assert!(f.right);
        assert_eq!(t.joystick.unwrap().base, vec2(200.0, 400.0));
        // 조이스틱 손가락이 사라지면(id가 목록에서 빠짐) 멈춘다
        let f = t.update(&[finger(3, TouchPhase::Stationary, 50.0, 300.0)], &PLAY_BUTTONS, true);
        assert!(!f.left && !f.right);
        assert!(t.joystick.is_none());
    }

    #[test]
    fn back_button_only_when_shown() {
        let p = BACK_BUTTON.center();
        let mut t = TouchControls::new();
        let f = t.update(&[finger(1, TouchPhase::Started, p.x, p.y)], &PLAY_BUTTONS, true);
        assert!(f.back_pressed);
        assert!(t.joystick.is_none(), "고르기 버튼을 누른 손가락은 조이스틱이 아니다");
        let mut t = TouchControls::new();
        let f = t.update(&[finger(1, TouchPhase::Started, p.x, p.y)], &PLAY_BUTTONS, false);
        assert!(!f.back_pressed);
    }

    #[test]
    fn finger_held_from_menu_is_ignored_in_play() {
        // 메뉴에서 점프 버튼 자리를 누른 채 플레이가 시작돼도 점프하지 않는다.
        let mut t = TouchControls::new();
        t.idle(&[finger(5, TouchPhase::Started, JUMP.x, JUMP.y), finger(6, TouchPhase::Started, 200.0, 300.0)]);
        let f = t.update(
            &[finger(5, TouchPhase::Stationary, JUMP.x, JUMP.y), finger(6, TouchPhase::Moved, 300.0, 300.0)],
            &PLAY_BUTTONS,
            true,
        );
        assert_eq!(f, TouchFrame::default());
        assert!(t.joystick.is_none());
    }

    #[test]
    fn joystick_base_stays_on_screen_near_edges() {
        let mut t = TouchControls::new();
        t.update(&[finger(1, TouchPhase::Started, -50.0, 520.0)], &PLAY_BUTTONS, true);
        let j = t.joystick.unwrap();
        let r = JOYSTICK_RADIUS;
        assert!(j.base.x - r >= 0.0 && j.base.y + r <= VIRTUAL_HEIGHT, "받침이 화면 안: {}", j.base);
        // 손가락을 오른쪽으로 밀면 손잡이가 따라가고 걷는다 (손가락-손잡이 관계 유지)
        let f = t.update(&[finger(1, TouchPhase::Moved, 200.0, 520.0)], &PLAY_BUTTONS, true);
        assert!(f.right && !f.left);
        assert_eq!(t.joystick.unwrap().knob, knob_offset(j.base, vec2(200.0, 520.0), r));
        // 화면 가운데 쪽에 대면 그 자리 그대로
        assert_eq!(joystick_base(vec2(200.0, 300.0)), vec2(200.0, 300.0));
    }

    #[test]
    fn reused_finger_id_started_again_is_a_new_finger() {
        // 조이스틱 엄지(id 1)를 떼고 같은 프레임 안에 점프를 누르면 id 1이 Started로만 보인다.
        let mut t = TouchControls::new();
        t.update(&[finger(1, TouchPhase::Started, 200.0, 400.0)], &PLAY_BUTTONS, true);
        assert!(t.joystick.is_some());
        let f = t.update(&[finger(1, TouchPhase::Started, JUMP.x, JUMP.y)], &PLAY_BUTTONS, true);
        assert!(f.jump_pressed, "점프가 된다");
        assert!(!f.left && !f.right, "조이스틱으로 걷지 않는다");
        assert!(t.joystick.is_none());
        assert!(t.is_held(ButtonKind::Jump));
    }
}
