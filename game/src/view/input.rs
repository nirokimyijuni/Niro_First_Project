//! 키보드·마우스·터치 입력을 읽어서 logic이 쓰는 모양으로 바꾼다.
//! 터치는 macroquad가 마우스 클릭으로도 알려 주므로(기본 설정) 마우스와 같이 처리된다.
//! 모바일 가상 조이스틱은 M7에서 붙인다.

use super::screen::Letterbox;
use crate::logic::player::PlayerInput;
use macroquad::prelude::*;

/// 플레이 중: ←/→ 걷기, Space 또는 ↑ 점프.
pub fn player_input() -> PlayerInput {
    PlayerInput {
        left: is_key_down(KeyCode::Left),
        right: is_key_down(KeyCode::Right),
        jump_pressed: is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Up),
    }
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
    /// 마우스 클릭/화면 터치 위치 (가상 화면 좌표)
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

/// 개발용 빨리감기(D12) 켜기/끄기: F를 "새로 누른" 프레임에만 true (꾹 눌러도 한 번).
pub fn fast_forward_pressed() -> bool {
    is_key_pressed(KeyCode::F)
}
