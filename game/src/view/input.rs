//! 키보드·마우스·터치 입력을 읽어서 logic이 쓰는 모양으로 바꾼다.
//! 터치는 메뉴·플레이 모두 `fingers()` 한 길로 받는다 (main이 마우스 흉내를 끈다).
//! 메뉴: 진짜 마우스 클릭은 `menu_input`의 `tap`, 터치는 `TouchControls::idle`이 돌려주는 새 손가락 자리.
//! 플레이: 터치는 조이스틱·버튼(touch.rs)으로만 처리한다.

use super::math::MathKey;
use super::screen::Letterbox;
use super::shop::Pocket;
use super::touch::{Finger, TouchFrame};
use crate::logic::game::Usable;
use crate::logic::player::PlayerInput;
use crate::logic::shop::Potion;
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

/// ↑가 들어가기·나가기가 되는 곳인지 (부모님 결정 2026-11-15): 집 문, 동굴 입구, 동굴 출구에 닿아 있을 때.
/// 그때 ↑는 점프가 아니라 들어가기다. 다른 곳의 ↑는 지금처럼 점프다. E로 들어가기도 그대로다.
pub fn up_enters(usable: Option<Usable>) -> bool {
    matches!(usable, Some(Usable::HouseDoor | Usable::CaveEntrance | Usable::CaveExit))
}

/// 키 상태 → 플레이 입력. `up_enters`면 ↑는 점프가 아니다 (들어가기는 `mine_key_down`).
pub fn keys_to_input(left: bool, right: bool, space: bool, up: bool, q: bool, up_enters: bool) -> PlayerInput {
    PlayerInput { left, right, jump_pressed: space || (up && !up_enters), attack_pressed: q }
}

/// 플레이 중: ←/→ 걷기, Space 또는 ↑ 점프 (문·동굴 입구에서는 ↑가 들어가기), Q 공격.
pub fn player_input(up_enters: bool) -> PlayerInput {
    keys_to_input(
        is_key_down(KeyCode::Left),
        is_key_down(KeyCode::Right),
        is_key_pressed(KeyCode::Space),
        is_key_pressed(KeyCode::Up),
        is_key_pressed(KeyCode::Q),
        up_enters,
    )
}

/// 캐기·들어가기·만들기 (임시-16): E를 누르고 있는 동안 true. 새로 누름은 logic이 가린다.
/// 문·동굴 입구에 닿아 있으면 ↑도 들어가기지만, ↑는 **새로 누른 프레임에만** true다 (REVIEW-022 1번).
/// ↑를 꾹 누른 채 문에 닿거나, ↑를 누른 채 상점을 닫아도 저절로 들어가지 않는다.
pub fn keys_to_mine(e_down: bool, up_pressed: bool, up_enters: bool) -> bool {
    e_down || (up_enters && up_pressed)
}

/// 키보드로 본 캐기·들어가기 누름 (`keys_to_mine`).
pub fn mine_key_down(up_enters: bool) -> bool {
    keys_to_mine(is_key_down(KeyCode::E), is_key_pressed(KeyCode::Up), up_enters)
}

/// 무기 바꾸기 키 (부모님 결정 2026-11-15: W 대신 Tab). 화면에 쓰는 글자도 함께.
pub const SWITCH_KEY: (KeyCode, &str) = (KeyCode::Tab, "Tab");
/// 가방 열고 닫기 키 (부모님 결정 2026-11-15: Tab 대신 I).
pub const BAG_KEY: (KeyCode, &str) = (KeyCode::I, "I");

/// 튜토리얼 말풍선 건너뛰기 키 (US-120, 임시-128). Esc(멈춤 창)·Enter·Space(결정·점프)·Tab(무기 바꾸기)·I(가방)·
/// Q·E·R·F·Y·G·1~3(공격·캐기·물건)·←→↑(걷기·점프)와 겹치지 않는 N. 마우스 클릭은 버튼 위만 건너뛰기다.
pub const TUTORIAL_SKIP_KEY: (KeyCode, &str) = (KeyCode::N, "N");

/// 건너뛰기 키: N을 "새로 누른" 프레임에만 true.
pub fn tutorial_skip_key_pressed() -> bool {
    is_key_pressed(TUTORIAL_SKIP_KEY.0)
}

/// 무기 바꾸기: Tab을 "새로 누른" 프레임에만 true (꾹 눌러도 한 번).
pub fn switch_weapon_pressed() -> bool {
    is_key_pressed(SWITCH_KEY.0)
}

/// 가방: I를 "새로 누른" 프레임에만 true (꾹 눌러도 한 번 열리고 한 번 닫힌다).
pub fn bag_key_pressed() -> bool {
    is_key_pressed(BAG_KEY.0)
}

/// 바로 쓰는 물건 키 (부모님 결정 2026-11-15): 1 = 체력 포션, 2 = 독약, R = 주사기총 쏘기, 3 = 엔더열매 먹기.
/// F는 개발용 빨리감기라 쓰지 않는다. Shift+숫자는 시험용 단축(?qa)이라 물건을 쓰지 않는다. (키, 물건, 화면에 쓰는 글자)
pub const POCKET_KEYS: [(KeyCode, Pocket, &str); 4] = [
    (KeyCode::Key1, Pocket::Potion(Potion::Health), "1"),
    (KeyCode::Key2, Pocket::Potion(Potion::Poison), "2"),
    (KeyCode::R, Pocket::SyringeGun, "R"),
    (KeyCode::Key3, Pocket::EnderFruit, "3"),
];

/// Shift를 누르고 있는지 (Shift+숫자 = 시험용 단축). 한 프레임 안에 눌렀다 뗀 Shift도 센다.
pub fn shift_down() -> bool {
    [KeyCode::LeftShift, KeyCode::RightShift].into_iter().any(|k| is_key_down(k) || is_key_pressed(k))
}

/// 키 하나가 어느 물건인지.
pub fn pocket_for_key(key: KeyCode) -> Option<Pocket> {
    POCKET_KEYS.iter().find(|(k, ..)| *k == key).map(|(_, p, _)| *p)
}

/// 화면에 쓰는 키 글자 ("R", "T", "Y", "G").
pub fn pocket_key_label(pocket: Pocket) -> &'static str {
    POCKET_KEYS.iter().find(|(_, p, _)| *p == pocket).map_or("", |(.., label)| label)
}

/// 이번 프레임에 "새로" 누른 물건 키들 (꾹 눌러도 한 번). Shift를 누르고 있으면 없다 (시험용 단축).
pub fn pocket_keys_pressed() -> Vec<Pocket> {
    pockets_for_keys(get_keys_pressed().into_iter(), shift_down())
}

/// 누른 키들 → 쓸 물건들. `shift`면 아무것도 쓰지 않는다.
pub fn pockets_for_keys(keys: impl Iterator<Item = KeyCode>, shift: bool) -> Vec<Pocket> {
    if shift {
        return Vec::new();
    }
    keys.filter_map(pocket_for_key).collect()
}

/// 메뉴 화면(타이틀·캐릭터 고르기)에서 쓰는 입력. 모두 "이번 프레임에 새로 누름" 기준.
#[derive(Debug, Clone, Copy, Default)]
pub struct MenuInput {
    pub left: bool,
    pub right: bool,
    /// ↑/↓ (멈춤 창에서 칸 옮기기). 꾹 누르고 있던 키는 false
    pub up: bool,
    pub down: bool,
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
        up: is_key_pressed(KeyCode::Up),
        down: is_key_pressed(KeyCode::Down),
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

    #[test]
    fn up_enters_only_on_a_new_press() {
        // ↑를 새로 누른 프레임만 들어가기 (REVIEW-022 1번)
        assert!(keys_to_mine(false, true, true));
        assert!(!keys_to_mine(false, false, true), "↑를 누르고 있기만 하면 들어가지 않는다");
        assert!(!keys_to_mine(false, true, false), "문·동굴이 아닌 곳의 ↑는 점프다");
        // E는 누르고 있는 동안 계속 (캐기 꾹 누름)
        assert!(keys_to_mine(true, false, false));
        assert!(keys_to_mine(true, false, true));
    }

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
    fn keys_1_2_r_3_use_potions_syringe_gun_and_ender_fruit() {
        // 부모님 결정 2026-11-15: 1 체력 포션, 2 독약, R 주사기총, 3 엔더열매
        assert_eq!(pocket_for_key(KeyCode::Key1), Some(Pocket::Potion(Potion::Health)));
        assert_eq!(pocket_for_key(KeyCode::Key2), Some(Pocket::Potion(Potion::Poison)));
        assert_eq!(pocket_for_key(KeyCode::R), Some(Pocket::SyringeGun));
        assert_eq!(pocket_for_key(KeyCode::Key3), Some(Pocket::EnderFruit));
        for other in [KeyCode::Q, KeyCode::W, KeyCode::E, KeyCode::F, KeyCode::T, KeyCode::Y, KeyCode::G, KeyCode::Tab, KeyCode::I, KeyCode::Space, KeyCode::Escape] {
            assert_eq!(pocket_for_key(other), None, "{other:?}는 물건을 쓰지 않는다 (T·Y·G·W는 이제 아무 일 없음)");
        }
        let labels: Vec<&str> = Pocket::ALL.iter().map(|p| pocket_key_label(*p)).collect();
        assert_eq!(labels, ["1", "2", "R", "3"], "물건마다 키가 하나씩 있다");
        // Shift+숫자는 시험용 단축이라 포션을 쓰지 않는다
        assert_eq!(pockets_for_keys([KeyCode::Key1, KeyCode::R].into_iter(), false), vec![Pocket::Potion(Potion::Health), Pocket::SyringeGun]);
        assert!(pockets_for_keys([KeyCode::Key1].into_iter(), true).is_empty());
    }

    #[test]
    fn tab_switches_weapons_and_i_opens_the_bag() {
        assert_eq!(SWITCH_KEY.0, KeyCode::Tab);
        assert_eq!(BAG_KEY.0, KeyCode::I);
        assert_ne!(SWITCH_KEY.0, BAG_KEY.0);
        for (k, _, _) in POCKET_KEYS {
            assert!(k != SWITCH_KEY.0 && k != BAG_KEY.0 && k != KeyCode::Q && k != KeyCode::E && k != KeyCode::F);
        }
    }

    #[test]
    fn up_enters_doors_and_caves_but_jumps_elsewhere() {
        assert!(up_enters(Some(Usable::HouseDoor)) && up_enters(Some(Usable::CaveEntrance)) && up_enters(Some(Usable::CaveExit)));
        for other in [None, Some(Usable::Tree(0)), Some(Usable::OreLump(0)), Some(Usable::Workbench)] {
            assert!(!up_enters(other), "{other:?}: ↑는 점프");
        }
        let at_door = keys_to_input(false, false, false, true, false, true);
        assert!(!at_door.jump_pressed, "문 앞 ↑는 점프하지 않는다");
        assert!(keys_to_input(false, false, true, true, false, true).jump_pressed, "문 앞이어도 Space는 점프");
        assert!(keys_to_input(false, false, false, true, false, false).jump_pressed, "다른 곳의 ↑는 점프");
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
