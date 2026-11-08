//! 키보드·마우스·터치 입력을 읽어서 logic이 쓰는 모양으로 바꾼다.
//! 터치는 메뉴·플레이 모두 `fingers()` 한 길로 받는다 (main이 마우스 흉내를 끈다).
//! 메뉴: 진짜 마우스 클릭은 `menu_input`의 `tap`, 터치는 `TouchControls::idle`이 돌려주는 새 손가락 자리.
//! 플레이: 터치는 조이스틱·버튼(touch.rs)으로만 처리한다.

use super::math::MathKey;
use super::screen::Letterbox;
use super::touch::{Finger, TouchFrame};
use crate::logic::player::PlayerInput;
use macroquad::prelude::*;

/// 키보드 입력에 터치 입력을 더한다 (둘 중 하나라도 누르면 누른 것).
pub fn merge_touch(keys: PlayerInput, touch: &TouchFrame) -> PlayerInput {
    PlayerInput {
        left: keys.left || touch.left,
        right: keys.right || touch.right,
        jump_pressed: keys.jump_pressed || touch.jump_pressed,
        attack_pressed: keys.attack_pressed || touch.attack_pressed,
    }
}

/// 이번 프레임 화면 위 손가락들 (가상 화면 좌표).
pub fn fingers(letterbox: &Letterbox) -> Vec<Finger> {
    let dpi = screen_dpi_scale();
    touches()
        .into_iter()
        .map(|t| Finger { id: t.id, phase: t.phase, pos: touch_to_virtual(t.position, dpi, letterbox) })
        .collect()
}

/// 터치 위치 → 가상 화면 좌표.
/// `high_dpi`에서 `touches()`는 실제 화면 점(물리 픽셀) 단위라서, `screen_width()`·`mouse_position()`과
/// 같은 단위(논리 픽셀)가 되게 dpi 배율로 나눈 뒤 `Letterbox`에 넣는다 (REVIEW-003 1번).
pub fn touch_to_virtual(physical: Vec2, dpi: f32, letterbox: &Letterbox) -> Vec2 {
    letterbox.to_virtual(physical / dpi.max(0.01))
}

/// 이번 프레임에 키보드 키를 하나라도 새로 눌렀는지 (터치 모드 끄기용).
pub fn any_key_pressed() -> bool {
    !get_keys_pressed().is_empty()
}

/// 플레이 중: ←/→ 걷기, Space 또는 ↑ 점프, Q 공격.
pub fn player_input() -> PlayerInput {
    PlayerInput {
        left: is_key_down(KeyCode::Left),
        right: is_key_down(KeyCode::Right),
        jump_pressed: is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Up),
        attack_pressed: is_key_pressed(KeyCode::Q),
    }
}

/// 무기 바꾸기: W를 "새로 누른" 프레임에만 true (꾹 눌러도 한 번).
pub fn switch_weapon_pressed() -> bool {
    is_key_pressed(KeyCode::W)
}

/// 메뉴 화면(타이틀·캐릭터 고르기)에서 쓰는 입력. 모두 "이번 프레임에 새로 누름" 기준.
#[derive(Debug, Clone, Copy, Default)]
pub struct MenuInput {
    pub left: bool,
    pub right: bool,
    /// Enter 또는 Space
    pub confirm: bool,
    /// Esc
    pub back: bool,
    /// 마우스 클릭 또는 (main이 넣는) 새 손가락 위치 (가상 화면 좌표)
    pub tap: Option<Vec2>,
}

pub fn menu_input(letterbox: &Letterbox) -> MenuInput {
    let tap = if is_mouse_button_pressed(MouseButton::Left) {
        Some(letterbox.to_virtual(mouse_position().into()))
    } else {
        None
    };
    MenuInput {
        left: is_key_pressed(KeyCode::Left),
        right: is_key_pressed(KeyCode::Right),
        confirm: is_key_pressed(KeyCode::Enter)
            || is_key_pressed(KeyCode::KpEnter)
            || is_key_pressed(KeyCode::Space),
        back: is_key_pressed(KeyCode::Escape),
        tap,
    }
}

/// 문제 창에서 지우기·확인 키 (Backspace, Enter). 숫자는 키가 아니라 글자 줄에서 받는다 (`math_keys`).
pub fn math_key(key: KeyCode) -> Option<MathKey> {
    match key {
        KeyCode::Backspace => Some(MathKey::Erase),
        KeyCode::Enter | KeyCode::KpEnter => Some(MathKey::Submit),
        _ => None,
    }
}

/// 이번 프레임의 숫자판 버튼들. 숫자(윗줄·키패드)는 글자 차례대로 (한 프레임에 "4", "7"이 같이 와도 47),
/// 지우기·확인은 그 뒤에. 글자 줄은 창이 닫혀 있어도 늘 다 꺼내 버린다:
/// macroquad 0.4.16은 이 줄을 프레임마다 비우지 않아서, 놀다 누른 숫자가 다음 문제 답 칸에 들어간다 (REVIEW-016 3번).
pub fn math_keys(chars: impl Iterator<Item = char>, keys: impl Iterator<Item = KeyCode>, quiz_open: bool) -> Vec<MathKey> {
    let digits: Vec<MathKey> = chars.filter_map(|c| c.to_digit(10)).map(MathKey::Digit).collect();
    if !quiz_open {
        return Vec::new();
    }
    digits.into_iter().chain(keys.filter_map(math_key)).collect()
}

/// 매 프레임 한 번 부른다 (글자 줄 비우기 때문에). 창이 닫혀 있으면 빈 목록.
pub fn math_keys_pressed(quiz_open: bool) -> Vec<MathKey> {
    math_keys(std::iter::from_fn(get_char_pressed), get_keys_pressed().into_iter(), quiz_open)
}

/// 문제 창이 닫힌 뒤 걷기 막기: 창이 열려 있던 동안 누르던 ←/→는 한 번 뗄 때까지 걷기가 되지 않는다.
/// `locked`는 창이 열린 동안 true로 둔다. 돌려주는 값: (이번 입력, 다음 프레임의 막기).
pub fn hold_walk_until_released(locked: bool, input: PlayerInput) -> (PlayerInput, bool) {
    if !locked || !(input.left || input.right) {
        return (input, false);
    }
    (PlayerInput { left: false, right: false, ..input }, true)
}

/// 개발용 빨리감기(D12) 켜기/끄기: F를 "새로 누른" 프레임에만 true (꾹 눌러도 한 번).
pub fn fast_forward_pressed() -> bool {
    is_key_pressed(KeyCode::F)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::touch::{button_at, ButtonKind, PLAY_BUTTONS};

    /// 아이폰 12 Pro 가로: 창 844×390 (논리 픽셀), 배율 3.
    fn iphone() -> Letterbox {
        Letterbox::for_window(844.0, 390.0)
    }

    /// 가상 좌표 → 그 자리를 누르면 들어오는 터치 위치 (물리 픽셀).
    fn physical_at(lb: &Letterbox, virtual_point: Vec2, dpi: f32) -> Vec2 {
        (lb.offset + virtual_point * lb.scale) * dpi
    }

    #[test]
    fn number_keys_backspace_and_enter_work_in_the_quiz() {
        let typed = math_keys("4a7".chars(), [KeyCode::Backspace, KeyCode::Q].into_iter(), true);
        assert_eq!(typed, [MathKey::Digit(4), MathKey::Digit(7), MathKey::Erase], "숫자는 차례대로, 글자 a와 Q는 버린다");
        assert_eq!(math_key(KeyCode::Key7), None, "숫자는 글자 줄에서만 받는다");
        assert_eq!(math_key(KeyCode::Backspace), Some(MathKey::Erase));
        assert_eq!(math_key(KeyCode::Enter), Some(MathKey::Submit));
        assert_eq!(math_key(KeyCode::KpEnter), Some(MathKey::Submit));
        for other in [KeyCode::Q, KeyCode::Space, KeyCode::Left, KeyCode::W, KeyCode::F, KeyCode::Escape] {
            assert_eq!(math_key(other), None);
        }
    }

    #[test]
    fn numbers_typed_while_playing_are_thrown_away() {
        let mut queue: std::collections::VecDeque<char> = "53".chars().collect();
        let keys = math_keys(std::iter::from_fn(|| queue.pop_front()), [KeyCode::Enter].into_iter(), false);
        assert!(keys.is_empty(), "창이 닫혀 있으면 아무것도 넣지 않는다");
        assert!(queue.is_empty(), "줄은 비워서 다음 문제 답 칸에 53이 들어가지 않는다");
    }

    #[test]
    fn walking_waits_until_the_arrow_is_let_go_after_the_quiz() {
        let held = PlayerInput { right: true, jump_pressed: true, ..Default::default() };
        let (input, locked) = hold_walk_until_released(true, held);
        assert!(!input.right && input.jump_pressed && locked, "누르던 →는 막고, 새로 누른 점프는 둔다");
        let (_, locked) = hold_walk_until_released(locked, PlayerInput::default());
        assert!(!locked, "떼면 풀린다");
        let (input, _) = hold_walk_until_released(locked, held);
        assert!(input.right, "다시 누르면 걷는다");
    }

    #[test]
    fn attack_button_works_at_dpi_3() {
        let lb = iphone();
        let attack = PLAY_BUTTONS[0].center;
        let p = touch_to_virtual(physical_at(&lb, attack, 3.0), 3.0, &lb);
        assert!(p.distance(attack) < 0.01);
        assert_eq!(button_at(&PLAY_BUTTONS, p), Some(ButtonKind::Attack));
    }

    #[test]
    fn same_spot_gives_same_result_at_dpi_1_and_3() {
        let lb = iphone();
        for v in [vec2(100.0, 400.0), vec2(480.0, 270.0), PLAY_BUTTONS[0].center] {
            let a = touch_to_virtual(physical_at(&lb, v, 1.0), 1.0, &lb);
            let b = touch_to_virtual(physical_at(&lb, v, 3.0), 3.0, &lb);
            assert!(a.distance(b) < 0.01, "{a} vs {b}");
        }
    }
}
