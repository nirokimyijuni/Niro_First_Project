//! 무기 강화가 보인다 (US-130, D115, 로건): "Lv n" 글자, 반짝이는 별, 가방·손에 든 칸의 숫자, 총 범위 글.
//! 값은 logic이 준다 (`WeaponLevels`, `WeaponEvent::Upgraded`). 여기서는 읽어서 보여 주기만 하고 단계나 공격력을 계산하지 않는다.
//! 큰 알림 글("칼이 더 세졌어요!")은 `Effects::on_weapon_event`가 기존 큰 알림 쌓기에 넣는다.

use super::assets::M6Art;
use super::hud::HELD_SLOT;
use super::sky::lerp;
use super::sprites::fit_size;
use super::text::TextPainter;
use super::touch::draw_star;
use crate::config::{
    AXE_UPGRADED_TEXT, GUN_RANGE_LABEL_TEXT, GUN_RANGE_MAX, GUN_RANGE_MAX_TEXT, GUN_UPGRADED_TEXT, SWORD_UPGRADED_TEXT, WEAPON_ATTACK_DISPLAY_CAP,
    WEAPON_UPGRADE_COLOR, WEAPON_UPGRADE_FLASH_SECONDS, WEAPON_UPGRADE_MAX_FLOATS, WEAPON_UPGRADE_RISE, WEAPON_UPGRADE_SECONDS, WEAPON_UPGRADE_STARS,
};
use crate::logic::weapon::{capped_for_display, is_over_display_cap, Weapon};
use macroquad::prelude::*;
use std::f32::consts::TAU;

/// 올라가는 "Lv n" 글자와 무기 그림 크기
const FLOAT_SIZE: f32 = 44.0;
const FLOAT_ICON: f32 = 40.0;
const FLOAT_GAP: f32 = 8.0;
/// 손에 든 칸 위로 "Lv n"이 시작하는 높이 (칸 위 끝에서 위로)
const FLOAT_START_ABOVE: f32 = 6.0;
/// 별이 퍼지는 가까운·먼 거리와 별 크기
const STAR_NEAR: f32 = 22.0;
const STAR_FAR: f32 = 76.0;
const STAR_SIZE: f32 = 10.0;
/// 가방 무기 칸의 숫자: 오른쪽 아래 큰 숫자(공격력), 오른쪽 위 작은 "Lv n"
pub const ATTACK_SIZE: f32 = 26.0;
pub const LEVEL_SIZE: f32 = 24.0;
const CELL_PAD: f32 = 5.0;
/// 손에 든 칸 오른쪽 위 "Lv n"
const HELD_LEVEL_SIZE: f32 = 24.0;
const HELD_LEVEL_TOP: f32 = 2.0;
/// 가방 무기 줄 오른쪽 글 크기
pub const NOTE_SIZE: f32 = 26.0;
pub const RANGE_SIZE: f32 = 28.0;

fn gold(a: f32) -> Color {
    let [r, g, b] = WEAPON_UPGRADE_COLOR;
    Color::new(r, g, b, a)
}

/// 화면에 보이는 레벨: 처음 가진 무기가 1, 강화할 때마다 1씩 (표시 레벨 = 강화 횟수 + 1).
pub fn display_level(count: u32) -> u32 {
    count.saturating_add(1)
}

/// 레벨 글: 칼·도끼·총은 "Lv1", "Lv2"... 99까지 숫자, 100부터 "Lv99+". 맨손은 강화가 없어서 없음.
pub fn level_label(weapon: Weapon, count: u32) -> Option<String> {
    slot(weapon)?;
    let shown = display_level(count);
    let n = capped_for_display(shown);
    Some(if is_over_display_cap(shown) { format!("Lv{n}+") } else { format!("Lv{n}") })
}

/// 큰 알림 글 뒤에 붙는 레벨 ("칼이 더 세졌어요! Lv3"). 같은 글은 한 번만 만들어 둔다 (큰 글은 `&'static str`이라서; 무기 3 x 레벨 100가지가 끝이다).
pub fn upgrade_message(weapon: Weapon, count: u32) -> &'static str {
    use std::cell::RefCell;
    use std::collections::HashMap;
    thread_local! {
        static MADE: RefCell<HashMap<String, &'static str>> = RefCell::new(HashMap::new());
    }
    let words = format!("{} {}", upgrade_text(weapon), level_label(weapon, count).unwrap_or_default());
    MADE.with_borrow_mut(|made| *made.entry(words.clone()).or_insert_with(|| Box::leak(words.into_boxed_str())))
}

/// 공격력 글: `WEAPON_ATTACK_DISPLAY_CAP`을 넘으면 "999+".
pub fn attack_label(attack: u32) -> String {
    if attack > WEAPON_ATTACK_DISPLAY_CAP {
        format!("{WEAPON_ATTACK_DISPLAY_CAP}+")
    } else {
        attack.to_string()
    }
}

/// 총 범위 한 줄: "사거리 408", 끝에 닿으면 "사거리 480 (최대)".
pub fn gun_range_line(range: f32) -> String {
    let base = format!("{GUN_RANGE_LABEL_TEXT} {}", range.round() as u32);
    if range >= GUN_RANGE_MAX {
        format!("{base} ({GUN_RANGE_MAX_TEXT})")
    } else {
        base
    }
}

/// 강화한 무기에 맞는 큰 알림 글. 맨손은 강화가 없어서 빈 글.
pub fn upgrade_text(weapon: Weapon) -> &'static str {
    match weapon {
        Weapon::Hands => "",
        Weapon::Sword => SWORD_UPGRADED_TEXT,
        Weapon::Axe => AXE_UPGRADED_TEXT,
        Weapon::Gun => GUN_UPGRADED_TEXT,
    }
}

fn slot(weapon: Weapon) -> Option<usize> {
    match weapon {
        Weapon::Hands => None,
        Weapon::Sword => Some(0),
        Weapon::Axe => Some(1),
        Weapon::Gun => Some(2),
    }
}

/// 글이 `max_width`보다 길면 줄인 글자 크기.
pub fn fit_text_size(text: &TextPainter, s: &str, size: f32, max_width: f32) -> f32 {
    (size * max_width / text.width(s, size).max(1.0)).min(size)
}

/// 한 번의 강화 효과: 올라가는 "Lv n"과 무기 그림.
struct Float {
    weapon: Weapon,
    label: String,
    age: f32,
}

/// 강화 효과 상태 (화면 쪽). 새 게임마다 `Effects`와 함께 새로 만든다.
#[derive(Default)]
pub struct UpgradeView {
    floats: Vec<Float>,
    /// 칼·도끼·총 칸이 반짝이는 남은 초
    flash: [f32; 3],
}

impl UpgradeView {
    /// 강화 소식 하나를 받는다: "Lv n"이 올라가고 별이 퍼지고, 그 무기 칸이 반짝인다. 한 소식당 한 번이다.
    pub fn on_upgraded(&mut self, weapon: Weapon, level: u32) {
        let (Some(label), Some(i)) = (level_label(weapon, level), slot(weapon)) else { return };
        let label = format!("{label}!");
        self.flash[i] = WEAPON_UPGRADE_FLASH_SECONDS;
        self.floats.push(Float { weapon, label, age: 0.0 });
        if self.floats.len() > WEAPON_UPGRADE_MAX_FLOATS {
            self.floats.remove(0);
        }
    }

    pub fn update(&mut self, dt: f32) {
        for f in &mut self.floats {
            f.age += dt;
        }
        self.floats.retain(|f| f.age < WEAPON_UPGRADE_SECONDS);
        for left in &mut self.flash {
            *left = (*left - dt).max(0.0);
        }
    }

    /// 그 무기 칸 반짝임 세기 (1 막 시작 ~ 0 끝).
    pub fn flash(&self, weapon: Weapon) -> f32 {
        slot(weapon).map_or(0.0, |i| self.flash[i] / WEAPON_UPGRADE_FLASH_SECONDS)
    }

    /// 떠 있는 "Lv n" 수 (시험용).
    #[cfg(test)]
    pub fn floating(&self) -> usize {
        self.floats.len()
    }

    /// 손에 든 칸 위로 올라가는 "Lv n"(무기 그림과 함께)과 퍼지는 별. 화면 좌표.
    pub fn draw(&self, text: &TextPainter, art: &M6Art) {
        let c = HELD_SLOT.center();
        for f in &self.floats {
            let t = f.age / WEAPON_UPGRADE_SECONDS;
            let fade = ((1.0 - t) * 3.0).min(1.0);
            // 처음에 빠르게 올라가다 천천히 멈춘다
            let rise = WEAPON_UPGRADE_RISE * (1.0 - (1.0 - t) * (1.0 - t));
            let baseline = HELD_SLOT.y - FLOAT_START_ABOVE - rise;
            let icon = art.weapon(f.weapon).map(|tex| (tex, fit_size(tex, FLOAT_ICON)));
            let word_w = text.width(&f.label, FLOAT_SIZE);
            let total = word_w + icon.as_ref().map_or(0.0, |(_, s)| s.x + FLOAT_GAP);
            let mut x = c.x - total / 2.0;
            if let Some((tex, size)) = icon {
                draw_texture_ex(tex, x, baseline - FLOAT_SIZE * 0.4 - size.y / 2.0, Color::new(1.0, 1.0, 1.0, fade), DrawTextureParams { dest_size: Some(size), ..Default::default() });
                x += size.x + FLOAT_GAP;
            }
            text.left(&f.label, x, baseline, FLOAT_SIZE, gold(fade));
            for i in 0..WEAPON_UPGRADE_STARS {
                let a = i as f32 / WEAPON_UPGRADE_STARS as f32 * TAU + 0.5;
                // 칸이 화면 아래 끝이라 별은 위쪽 반원으로만 퍼진다
                let p = c + vec2(a.cos(), -a.sin().abs() * 0.8) * lerp(STAR_NEAR, STAR_FAR, 1.0 - (1.0 - t) * (1.0 - t));
                draw_star(p, STAR_SIZE * (1.0 - t * 0.6), t * 5.0 + i as f32, gold(1.0 - t));
            }
        }
    }
}

/// 손에 든 칸 오른쪽 위의 작은 "Lv n". 맨손이면 그리지 않는다. 글이 칸(64)보다 넓으면 줄인다.
pub fn draw_held_level(text: &TextPainter, weapon: Weapon, count: u32) {
    let Some(label) = level_label(weapon, count) else { return };
    let size = fit_text_size(text, &label, HELD_LEVEL_SIZE, HELD_SLOT.w - 4.0);
    text.left(&label, HELD_SLOT.right() - 2.0 - text.width(&label, size), HELD_SLOT.y + HELD_LEVEL_TOP + size, size, gold(1.0));
}

/// 손에 든 칸이 반짝일 때 둘레 후광 (`flash` 1~0).
pub fn draw_held_flash(flash: f32, t: f32) {
    if flash > 0.0 {
        let c = HELD_SLOT.center();
        let p = 0.5 + 0.5 * (t * 12.0).sin();
        draw_circle(c.x, c.y, HELD_SLOT.w * (0.58 + 0.06 * p), gold(0.45 * flash));
    }
}

/// 가방 무기 칸 하나의 위치: 큰 공격력 숫자 오른쪽 아래, 작은 "Lv n" 오른쪽 위. (글 왼쪽 x, 바닥선 y)
pub fn cell_text_spot(cell: Rect, width: f32, size: f32, top: bool) -> (f32, f32) {
    let x = cell.right() - CELL_PAD - width;
    let baseline = if top { cell.y + CELL_PAD + size } else { cell.bottom() - CELL_PAD };
    (x, baseline)
}

/// 가방 무기 칸 안 숫자: 공격력(오른쪽 아래), 강화 횟수 "Lv n"(오른쪽 위, 0이면 없음). 칸 폭 안에 들어가게 줄인다.
pub fn draw_cell_numbers(text: &TextPainter, cell: Rect, weapon: Weapon, attack: u32, count: u32) {
    let max_w = cell.w - CELL_PAD * 2.0;
    let a = attack_label(attack);
    let a_size = fit_text_size(text, &a, ATTACK_SIZE, max_w);
    let (x, y) = cell_text_spot(cell, text.width(&a, a_size), a_size, false);
    text.left(&a, x, y, a_size, WHITE);
    if let Some(p) = level_label(weapon, count) {
        let p_size = fit_text_size(text, &p, LEVEL_SIZE, max_w);
        let (x, y) = cell_text_spot(cell, text.width(&p, p_size), p_size, true);
        text.left(&p, x, y, p_size, gold(1.0));
    }
}

/// 칸이 반짝일 때: 노란 테두리와 칸 모서리 별 (`flash` 1~0).
pub fn draw_cell_flash(cell: Rect, flash: f32, t: f32) {
    if flash > 0.0 {
        let p = 0.5 + 0.5 * (t * 12.0).sin();
        draw_rectangle(cell.x, cell.y, cell.w, cell.h, gold(0.35 * flash * (0.6 + 0.4 * p)));
        draw_rectangle_lines(cell.x - 2.0, cell.y - 2.0, cell.w + 4.0, cell.h + 4.0, 5.0, gold(flash));
        for (i, corner) in [cell.point(), vec2(cell.right(), cell.y), vec2(cell.x, cell.bottom()), vec2(cell.right(), cell.bottom())].into_iter().enumerate() {
            draw_star(corner, 9.0 * (0.6 + 0.4 * p), t * 3.0 + i as f32, gold(flash));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::bag::{slot_rect, CELL};
    use crate::view::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};

    /// 글꼴 없이 센 넉넉한 글 폭: 한글은 글자 크기만큼, 숫자·기호는 0.62배.
    fn guess_width(s: &str, size: f32) -> f32 {
        s.chars().map(|c| if c.is_ascii() { size * 0.62 } else { size }).sum()
    }

    #[test]
    fn level_is_count_plus_one_and_caps_at_99_plus() {
        let l = |c| level_label(Weapon::Sword, c);
        assert_eq!(l(0).as_deref(), Some("Lv1"), "처음 가진 무기도 Lv1");
        assert_eq!(l(1).as_deref(), Some("Lv2"));
        assert_eq!(l(2).as_deref(), Some("Lv3"));
        assert_eq!(l(98).as_deref(), Some("Lv99"), "횟수 98 = Lv99");
        assert_eq!(l(99).as_deref(), Some("Lv99+"), "횟수 99는 Lv100이 아니라 Lv99+");
        assert_eq!(l(u32::MAX).as_deref(), Some("Lv99+"));
        assert_eq!(level_label(Weapon::Gun, 0).as_deref(), Some("Lv1"));
        assert_eq!(level_label(Weapon::Axe, 1).as_deref(), Some("Lv2"));
        assert_eq!(level_label(Weapon::Hands, 0), None, "맨손은 레벨 표시 없음");
        assert!(!is_over_display_cap(display_level(98)) && is_over_display_cap(display_level(99)));
    }

    #[test]
    fn big_message_has_words_and_level() {
        assert_eq!(upgrade_message(Weapon::Sword, 2), "칼이 더 세졌어요! Lv3");
        assert_eq!(upgrade_message(Weapon::Sword, 2).as_ptr(), upgrade_message(Weapon::Sword, 2).as_ptr(), "같은 글은 한 번만 만든다");
        assert!(upgrade_message(Weapon::Gun, 1).ends_with("Lv2"));
    }

    #[test]
    fn attack_and_range_labels() {
        assert_eq!(attack_label(5), "5");
        assert_eq!(attack_label(999), "999");
        assert_eq!(attack_label(1000), "999+");
        assert_eq!(attack_label(u32::MAX), "999+");
        assert_eq!(gun_range_line(300.0), "사거리 300");
        assert_eq!(gun_range_line(408.0), "사거리 408");
        assert_eq!(gun_range_line(GUN_RANGE_MAX), format!("사거리 {} (최대)", GUN_RANGE_MAX.round() as u32), "끝에 닿으면 최대");
    }

    #[test]
    fn every_weapon_has_a_gentle_upgrade_text() {
        for w in [Weapon::Sword, Weapon::Axe, Weapon::Gun] {
            assert!(upgrade_text(w).ends_with('!') && !upgrade_text(w).is_empty());
        }
        assert_eq!(upgrade_text(Weapon::Hands), "");
    }

    #[test]
    fn an_upgrade_shows_once_then_goes_away() {
        let mut v = UpgradeView::default();
        assert_eq!(v.floating(), 0);
        assert_eq!(v.flash(Weapon::Gun), 0.0);
        v.on_upgraded(Weapon::Gun, 2);
        assert_eq!(v.floating(), 1, "소식 하나에 한 번");
        assert_eq!(v.flash(Weapon::Gun), 1.0);
        assert_eq!(v.flash(Weapon::Sword), 0.0, "다른 무기 칸은 안 반짝인다");
        v.update(WEAPON_UPGRADE_SECONDS / 2.0);
        assert_eq!(v.floating(), 1);
        assert!(v.flash(Weapon::Gun) > 0.0 && v.flash(Weapon::Gun) < 1.0);
        v.update(WEAPON_UPGRADE_SECONDS);
        assert_eq!(v.floating(), 0, "시간이 지나면 사라진다");
        v.update(WEAPON_UPGRADE_FLASH_SECONDS);
        assert_eq!(v.flash(Weapon::Gun), 0.0);
    }

    #[test]
    fn hands_and_level_zero_make_no_effect_and_floats_do_not_pile_up() {
        let mut v = UpgradeView::default();
        v.on_upgraded(Weapon::Hands, 1);
        assert_eq!(v.floating(), 0);
        for level in 1..=10 {
            v.on_upgraded(Weapon::Sword, level);
        }
        assert_eq!(v.floating(), WEAPON_UPGRADE_MAX_FLOATS);
    }

    #[test]
    fn cell_numbers_fit_inside_the_cell_and_do_not_overlap() {
        for (s, size) in [("999+", ATTACK_SIZE), ("Lv99", LEVEL_SIZE)] {
            assert!(guess_width(s, size) <= CELL - CELL_PAD * 2.0, "{s}가 칸보다 넓다");
        }
        // "Lv99+"는 칸에 맞춰 줄여 그린다: 줄어도 20 아래로는 안 내려간다
        assert!(LEVEL_SIZE * (CELL - CELL_PAD * 2.0) / guess_width("Lv99+", LEVEL_SIZE) >= 20.0);
        let cell = slot_rect(0, 0);
        let (ax, ay) = cell_text_spot(cell, guess_width("999+", ATTACK_SIZE), ATTACK_SIZE, false);
        let (px, py) = cell_text_spot(cell, guess_width("Lv99", LEVEL_SIZE), LEVEL_SIZE, true);
        assert!(ax >= cell.x && px >= cell.x && ay <= cell.bottom() && py - LEVEL_SIZE >= cell.y, "글이 칸 안");
        assert!(py < ay - ATTACK_SIZE, "Lv 글과 공격력 글이 위아래로 겹치지 않는다");
    }

    #[test]
    fn held_plus_fits_the_held_slot_and_float_stays_on_screen() {
        assert!(guess_width("Lv99", HELD_LEVEL_SIZE) <= HELD_SLOT.w - 4.0);
        assert!(HELD_LEVEL_SIZE * (HELD_SLOT.w - 4.0) / guess_width("Lv99+", HELD_LEVEL_SIZE) >= 18.0);
        // 올라가는 "Lv n": 가장 높이 올라가도, 가장 넓어도 화면 안
        let top = HELD_SLOT.y - FLOAT_START_ABOVE - WEAPON_UPGRADE_RISE - FLOAT_SIZE;
        assert!(top >= 0.0 && HELD_SLOT.y <= VIRTUAL_HEIGHT);
        let width = FLOAT_ICON * 1.5 + FLOAT_GAP + guess_width("Lv99+!", FLOAT_SIZE);
        assert!(width / 2.0 <= VIRTUAL_WIDTH / 2.0 - 10.0);
        // 별도 화면 안
        let c = HELD_SLOT.center();
        assert!(c.y - STAR_FAR * 0.8 - STAR_SIZE >= 0.0 && c.y + STAR_SIZE <= VIRTUAL_HEIGHT && c.x - STAR_FAR - STAR_SIZE >= 0.0);
    }

    #[test]
    fn effect_colors_and_times_come_from_config() {
        assert!(WEAPON_UPGRADE_SECONDS > 0.5 && WEAPON_UPGRADE_FLASH_SECONDS > 0.0);
        assert!(WEAPON_UPGRADE_COLOR.iter().all(|c| (0.0..=1.0).contains(c)));
    }
}
