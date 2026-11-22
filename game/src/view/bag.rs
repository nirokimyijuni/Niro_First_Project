//! 가방 (US-091, D93, 임시-73~77, 로건): 가진 것을 모아 보는 창. 버튼 그림은 갈색 장난감 상자 (아이 확정).
//! PC는 I 키(부모님 결정 2026-11-15)나 오른쪽 위 작은 장난감 상자를 마우스로, 폰은 오른쪽 위 구석 장난감 상자 버튼으로 연다.
//! 열린 동안 게임은 멈춘다: main이 그동안 logic을 부르지 않는다 (상점·멈춤 창과 같은 방법).
//! 창 안은 늘 같은 자리의 세 줄: 무기 / 쓰는 것 / 재료. 가진 것은 밝게 개수와 함께, 없는 것은 흐리게.
//! 쓰는 것·무기 칸을 누르면 쓰거나 손에 들고 창이 바로 닫힌다 (부모님 확정). 쓸 수 있는지는 logic이 정한다.
//! 쓸 수 없으면 창은 그대로이고, logic이 알려 준 까닭(`Game::why_cant_use`)을 창 안에 보인다. 곡괭이 칸은 쓰는 법을 보인다 (D97).

use super::assets::{ItemPicture, M6Art};
use super::gather::{draw_material_icon, draw_pickaxe, recipe_line};
use super::hud::draw_pill;
use super::input::{pocket_key_label, BAG_KEY, POCKET_KEYS, SWITCH_KEY};
use super::items::draw_item_picture;
use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::shop::{draw_pocket, Pocket};
use super::sprites::fit_size;
use super::text::TextPainter;
use super::touch::draw_star;
use super::upgrade::{self, UpgradeView};
use crate::config::{
    BAG_FLASH_SECONDS, BAG_NAME_TEXT, DIAMOND_PICKAXE_HOW_TEXT, DIAMOND_PICKAXE_HOW_TOUCH_TEXT, KEY_HINT_SECONDS, PICKAXE_HOW_TEXT, PICKAXE_HOW_TOUCH_TEXT,
};
use crate::logic::game::Game;
use crate::logic::gathering::Material;
use crate::logic::weapon::Weapon;
use macroquad::prelude::*;

// ---- 가방 버튼 (장난감 상자) ----
/// 폰: 화면 오른쪽 위 구석. 740×360에서도 48 화면 픽셀이 넘는다.
const BUTTON_TOUCH: Rect = Rect::new(872.0, 8.0, 80.0, 76.0);
/// PC: 작은 상자 + 아래 키 글자. 마우스로 눌러도 열린다.
const BUTTON_PC: Rect = Rect::new(894.0, 6.0, 60.0, 72.0);
/// 아래 키 안내가 끝에 흐려지는 초. 보이는 초는 config `KEY_HINT_SECONDS` (임시-76), 반짝이는 초는 `BAG_FLASH_SECONDS` (임시-77)
const HINT_FADE_SECONDS: f32 = 1.0;
/// 아래 키 안내 자리 (왼쪽 아래, 무기 칸과 안 겹친다)
const HINT_LEFT: f32 = 16.0;
const HINT_BASELINE: f32 = 526.0;
const HINT_SIZE: f32 = 26.0;

/// 장난감 상자 색 (아이 확정: 갈색)
pub const TOY_BOX_BROWN: Color = Color::new(0.62, 0.38, 0.18, 1.0);
const TOY_BOX_DARK: Color = Color::new(0.40, 0.23, 0.10, 1.0);
const TOY_BOX_LIGHT: Color = Color::new(0.78, 0.53, 0.29, 1.0);
const LATCH: Color = Color::new(1.0, 0.82, 0.25, 1.0);
const FLASH_COLOR: Color = Color::new(1.0, 0.93, 0.45, 1.0);

// ---- 창 ----
const DIM: Color = Color::new(0.0, 0.0, 0.0, 0.45);
pub const PANEL: Rect = Rect::new(24.0, 12.0, 912.0, 516.0);
const PANEL_FILL: Color = Color::new(0.98, 0.93, 0.83, 0.98);
const TITLE_BASELINE: f32 = 70.0;
const TITLE_SIZE: f32 = 46.0;
pub const CLOSE: Rect = Rect::new(PANEL.x + PANEL.w - 16.0 - 170.0, 22.0, 170.0, 76.0);
const CLOSE_FILL: Color = Color::new(0.30, 0.62, 0.90, 1.0);
const ROW_LABEL_X: f32 = 44.0;
const ROW_LABEL_SIZE: f32 = 30.0;
const CELLS_LEFT: f32 = 168.0;
pub const CELL: f32 = 80.0;
const CELL_GAP: f32 = 8.0;
const ROW_TOPS: [f32; 3] = [108.0, 198.0, 288.0];
const CELL_FILL: Color = Color::new(1.0, 1.0, 1.0, 0.92);
const CELL_LINE: Color = Color::new(0.62, 0.40, 0.20, 1.0);
/// 없는 것을 흐리게 덮는 색
const FADE: Color = Color::new(0.93, 0.90, 0.85, 0.72);
const HELD_RING: Color = Color::new(1.0, 0.62, 0.15, 1.0);
const KEY_FILL: Color = Color::new(1.0, 0.62, 0.15, 1.0);
const COUNT_SIZE: f32 = 26.0;
const KEY_SIZE: f32 = 20.0;
const ICON: f32 = 50.0;
const RECIPE_BASELINE: f32 = 396.0;
const RECIPE_SIZE: f32 = 26.0;
const MESSAGE_BASELINE: f32 = 432.0;
const MESSAGE_SIZE: f32 = 32.0;
const MESSAGE_COLOR: Color = Color::new(1.0, 0.62, 0.22, 1.0);
/// 키 안내: 판 아래 어두운 띠 위에 흰 글씨, 두 칸 × 세 줄 (BUG-033: 크림색 위 흰 글씨는 안 읽혔다)
const GUIDE_BAND: Rect = Rect::new(PANEL.x + 12.0, 438.0, PANEL.w - 24.0, 84.0);
const GUIDE_BAND_FILL: Color = Color::new(0.30, 0.19, 0.10, 0.95);
const GUIDE_BASELINES: [f32; 3] = [462.0, 487.0, 512.0];
const GUIDE_SIZE: f32 = 24.0;
/// 두 칸 각각의 왼쪽 끝과 칸 너비
const GUIDE_COLUMN_X: [f32; 2] = [GUIDE_BAND.x + 20.0, GUIDE_BAND.x + GUIDE_BAND.w / 2.0 + 10.0];
const GUIDE_COLUMN_W: f32 = GUIDE_BAND.w / 2.0 - 30.0;

/// 가방 칸 하나에 들어가는 것.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BagSlot {
    Weapon(Weapon),
    Pickaxe,
    DiamondPickaxe,
    Pocket(Pocket),
    Material(Material),
}

/// 세 줄의 이름과 칸 (늘 같은 자리, 임시-73).
pub const ROW_NAMES: [&str; 3] = ["무기", "쓰는 것", "재료"];

pub fn rows() -> [Vec<BagSlot>; 3] {
    [
        vec![BagSlot::Weapon(Weapon::Sword), BagSlot::Weapon(Weapon::Axe), BagSlot::Weapon(Weapon::Gun), BagSlot::Pickaxe, BagSlot::DiamondPickaxe],
        Pocket::ALL.into_iter().map(BagSlot::Pocket).collect(),
        Material::ALL.into_iter().map(BagSlot::Material).collect(),
    ]
}

/// `row`줄 `col`번째 칸.
pub fn slot_rect(row: usize, col: usize) -> Rect {
    Rect::new(CELLS_LEFT + col as f32 * (CELL + CELL_GAP), ROW_TOPS[row], CELL, CELL)
}

/// 가진 개수 (무기·곡괭이는 있으면 1).
pub fn slot_count(game: &Game, slot: BagSlot) -> u32 {
    let bag = &game.gathering.bag;
    match slot {
        BagSlot::Weapon(w) => (w != Weapon::Hands && game.weapons.has(w)) as u32,
        BagSlot::Pickaxe => bag.has_pickaxe as u32,
        BagSlot::DiamondPickaxe => bag.has_diamond_pickaxe as u32,
        BagSlot::Pocket(p) => p.count(game),
        BagSlot::Material(m) => bag.count(m),
    }
}

/// 모든 칸의 개수 (새로 얻었는지 보기용).
pub fn all_counts(game: &Game) -> Vec<u32> {
    rows().iter().flatten().map(|s| slot_count(game, *s)).collect()
}

/// 가방 창에서 누른 곳.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BagTap {
    Close,
    Slot(BagSlot),
}

/// 누른 곳(가상 좌표)이 무엇인지. 빈 곳은 None (창 밖을 눌러도 닫히지 않는다).
pub fn bag_tap_at(point: Vec2) -> Option<BagTap> {
    if CLOSE.contains(point) {
        return Some(BagTap::Close);
    }
    rows().iter().enumerate().find_map(|(r, slots)| {
        slots.iter().enumerate().find(|(c, _)| slot_rect(r, *c).contains(point)).map(|(_, s)| BagTap::Slot(*s))
    })
}

/// 가방 버튼 자리 (화면 좌표). 폰은 크게, PC는 작게.
pub fn bag_button_rect(touch_mode: bool) -> Rect {
    if touch_mode {
        BUTTON_TOUCH
    } else {
        BUTTON_PC
    }
}

/// 물건 하나를 쓴다 (1·2·R·3 키, 폰 정보판 그림, 가방 칸이 모두 이 길). 쓸 수 있는지는 logic이 정한다. 쓰였으면 true.
pub fn use_pocket(game: &mut Game, pocket: Pocket) -> bool {
    match pocket {
        Pocket::Potion(potion) => game.use_potion(potion),
        Pocket::SyringeGun => game.fire_syringe_gun(),
        Pocket::EnderFruit => game.eat_ender_fruit(),
    }
}

/// 쓸 수 있으면 쓰고 None, 쓸 수 없으면 쓰지 않고 안내 글 (`Game::why_cant_use`, 부모님 결정 2026-11-15).
/// 가방은 이 글을 창 안에, 키·폰 정보판 그림은 짧은 알림으로 보인다.
pub fn use_pocket_or_why(game: &mut Game, pocket: Pocket) -> Option<&'static str> {
    if let Some(reason) = game.why_cant_use(pocket) {
        return Some(reason.text());
    }
    use_pocket(game, pocket);
    None
}

/// 가방에서 곡괭이 칸을 누르면 보이는 쓰는 법 (D97). 폰은 E 대신 버튼 이름으로.
pub fn pickaxe_how_text(diamond: bool, touch_mode: bool) -> &'static str {
    match (diamond, touch_mode) {
        (false, false) => PICKAXE_HOW_TEXT,
        (false, true) => PICKAXE_HOW_TOUCH_TEXT,
        (true, false) => DIAMOND_PICKAXE_HOW_TEXT,
        (true, true) => DIAMOND_PICKAXE_HOW_TOUCH_TEXT,
    }
}

/// PC 아래 키 안내 한 줄 (처음 8초): "I: 가방  M: 소리  Esc: 멈춤". 폰에는 이 줄이 안 나온다 (키가 없다).
pub fn short_key_hint() -> String {
    format!("{}: {BAG_NAME_TEXT}  M: 소리  Esc: 멈춤", BAG_KEY.1)
}

/// 가방 아래 전체 키 안내 (PC): 세 줄 × [왼쪽 칸, 오른쪽 칸]. 화살표 글자와 가운뎃점은 Jua 글꼴에 없어서
/// (네모로 보인다) 말과 쉼표로 쓴다. 위 화살표는 점프이고, 문·동굴 앞에서는 들어가기다.
pub fn full_key_guide() -> [[String; 2]; 3] {
    let pockets: Vec<&str> = POCKET_KEYS.iter().map(|(.., label)| *label).collect();
    [
        ["화살표: 걷기  Q: 공격".to_string(), "Space, 위 화살표: 점프".to_string()],
        [format!("{}: 무기 바꾸기  {}: {BAG_NAME_TEXT}", SWITCH_KEY.1, BAG_KEY.1), "E, 위 화살표: 캐기, 들어가기".to_string()],
        [format!("{}: 쓰기  Esc: 멈춤", pockets.join(", ")), "M: 소리 켜기, 끄기".to_string()],
    ]
}

/// 화면 쪽 가방 상태: 열림, 창 안 글, 장난감 상자 반짝임, 아래 키 안내 시간. 새 게임마다 새로 만든다.
#[derive(Debug, Default)]
pub struct BagView {
    pub open: bool,
    message: Option<&'static str>,
    flash: f32,
    seen: Option<Vec<u32>>,
    hint_age: f32,
}

impl BagView {
    pub fn new() -> BagView {
        BagView::default()
    }

    pub fn open_bag(&mut self) {
        self.open = true;
        self.message = None;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.message = None;
    }

    /// 창 안에 보이는 글 (없으면 None)
    pub fn message(&self) -> Option<&'static str> {
        self.message
    }

    /// 칸 개수를 지켜보다가 하나라도 늘면 장난감 상자가 반짝인다 (임시-77). 처음 본 개수로는 반짝이지 않는다.
    pub fn watch(&mut self, counts: Vec<u32>) {
        if let Some(old) = &self.seen {
            if counts.iter().zip(old).any(|(now, before)| now > before) {
                self.flash = BAG_FLASH_SECONDS;
            }
        }
        self.seen = Some(counts);
    }

    /// `hint_running`: 아래 키 안내 시간이 흐르는지 (멈춤·가방·퀴즈·상점·제작대 창이 열린 동안은 false).
    pub fn update(&mut self, dt: f32, hint_running: bool) {
        self.flash = (self.flash - dt).max(0.0);
        if hint_running {
            self.hint_age += dt;
        }
    }

    /// 반짝임 세기 (1 막 시작 ~ 0 끝)
    pub fn flash(&self) -> f32 {
        self.flash / BAG_FLASH_SECONDS
    }

    /// 아래 키 안내의 진하기: 처음엔 1, 끝 1초 동안 흐려지고 8초가 지나면 0.
    pub fn hint_alpha(&self) -> f32 {
        ((KEY_HINT_SECONDS - self.hint_age) / HINT_FADE_SECONDS).clamp(0.0, 1.0)
    }

    /// 누른 칸을 logic에 넘긴다. 돌려주는 값: 창이 닫혔으면 true.
    /// 무기는 손에 들고 닫는다. 쓰는 것은 쓰이면 닫고, 쓸 수 없으면 창 그대로 logic이 알려 준 까닭을 창 안에 보인다.
    /// 곡괭이 칸(손에 드는 무기가 아니다)은 창 그대로 쓰는 법을 보인다 (D97). 없는 무기·곡괭이·재료는 아무 일 없다.
    pub fn apply(&mut self, game: &mut Game, tap: BagTap, touch_mode: bool) -> bool {
        // 다른 칸을 누르면 지난 글은 지운다
        self.message = None;
        match tap {
            BagTap::Close => {
                self.close();
                true
            }
            BagTap::Slot(BagSlot::Pocket(p)) => match use_pocket_or_why(game, p) {
                Some(why) => {
                    self.message = Some(why);
                    false
                }
                None => {
                    self.close();
                    true
                }
            },
            BagTap::Slot(slot) if slot_count(game, slot) == 0 => false,
            BagTap::Slot(BagSlot::Weapon(w)) => {
                game.weapons.hold(w);
                self.close();
                true
            }
            BagTap::Slot(BagSlot::Pickaxe) => {
                self.message = Some(pickaxe_how_text(false, touch_mode));
                false
            }
            BagTap::Slot(BagSlot::DiamondPickaxe) => {
                self.message = Some(pickaxe_how_text(true, touch_mode));
                false
            }
            BagTap::Slot(BagSlot::Material(_)) => false,
        }
    }
}

/// 장난감 상자 그림 (가방 버튼, 가방 창 제목). `c` 가운데, `size` 폭. `flash`(0~1)면 둘레가 반짝인다.
pub fn draw_toy_box(c: Vec2, size: f32, flash: f32, t: f32) {
    if flash > 0.0 {
        let p = 0.5 + 0.5 * (t * 12.0).sin();
        draw_circle(c.x, c.y, size * (0.62 + 0.08 * p), Color { a: 0.45 * flash, ..FLASH_COLOR });
        for i in 0..4 {
            let a = t * 3.0 + i as f32 * std::f32::consts::FRAC_PI_2;
            draw_star(c + vec2(a.cos(), a.sin()) * size * 0.62, size * 0.12 * (0.6 + 0.4 * p), a, Color { a: flash, ..FLASH_COLOR });
        }
    }
    let w = size * 0.86;
    let body = Rect::new(c.x - w / 2.0, c.y - size * 0.18, w, size * 0.5);
    let lid = Rect::new(c.x - w * 0.54, c.y - size * 0.36, w * 1.08, size * 0.2);
    draw_rectangle(body.x, body.y, body.w, body.h, TOY_BOX_BROWN);
    // 나무 판자 줄
    draw_line(body.x, body.y + body.h * 0.5, body.right(), body.y + body.h * 0.5, 2.0, TOY_BOX_DARK);
    draw_rectangle_lines(body.x, body.y, body.w, body.h, 2.5, TOY_BOX_DARK);
    draw_rectangle(lid.x, lid.y, lid.w, lid.h, TOY_BOX_LIGHT);
    draw_rectangle_lines(lid.x, lid.y, lid.w, lid.h, 2.5, TOY_BOX_DARK);
    // 가운데 걸쇠
    draw_rectangle(c.x - size * 0.07, lid.bottom() - 2.0, size * 0.14, size * 0.14, LATCH);
    draw_rectangle_lines(c.x - size * 0.07, lid.bottom() - 2.0, size * 0.14, size * 0.14, 1.5, TOY_BOX_DARK);
}

/// 오른쪽 위 가방 버튼. 폰은 크고 둥근 판, PC는 작은 상자 + 아래 키 글자.
pub fn draw_bag_button(text: &TextPainter, view: &BagView, touch_mode: bool) {
    let t = get_time() as f32;
    let r = bag_button_rect(touch_mode);
    draw_rectangle(r.x, r.y, r.w, r.h, super::hud::PANEL);
    if touch_mode {
        draw_toy_box(r.center(), r.w * 0.78, view.flash(), t);
    } else {
        draw_toy_box(vec2(r.center().x, r.y + 28.0), r.w * 0.72, view.flash(), t);
        text.centered(BAG_KEY.1, r.center().x, r.bottom() - 6.0, 22.0, WHITE);
    }
}

/// PC 처음 8초의 아래 키 안내 한 줄 ("I: 가방  Esc: 멈춤"). 다 흐려졌으면 그리지 않는다.
pub fn draw_short_hint(text: &TextPainter, view: &BagView) {
    let a = view.hint_alpha();
    if a > 0.0 {
        text.left(&short_key_hint(), HINT_LEFT, HINT_BASELINE, HINT_SIZE, Color { a, ..WHITE });
    }
}

/// 칸 하나의 그림.
fn draw_slot_icon(art: &M6Art, slot: BagSlot, c: Vec2) {
    match slot {
        BagSlot::Weapon(w) => {
            if let Some(texture) = art.weapon(w) {
                let size = fit_size(texture, ICON);
                draw_texture_ex(texture, c.x - size.x / 2.0, c.y - size.y / 2.0, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
            }
        }
        BagSlot::Pickaxe => draw_pickaxe(c, ICON, WHITE),
        BagSlot::DiamondPickaxe => draw_item_picture(art, ItemPicture::DiamondPickaxe, c, ICON * 1.05),
        BagSlot::Pocket(p) => draw_pocket(p, art, c, ICON * 0.9),
        BagSlot::Material(m) => draw_material_icon(m, c, ICON * 0.8),
    }
}

/// 무기 줄 오른쪽 빈 곳 (칸 5개 끝 ~ 판 안쪽 끝). 숫자 안내와 총 범위 글이 들어간다 (US-130).
pub const NOTES: Rect = Rect::new(CELLS_LEFT + 5.0 * (CELL + CELL_GAP) + 8.0, ROW_TOPS[0], PANEL.x + PANEL.w - 20.0 - (CELLS_LEFT + 5.0 * (CELL + CELL_GAP) + 8.0), CELL);
/// 안내 글과 총 범위 글의 바닥선
const NOTE_BASELINE: f32 = ROW_TOPS[0] + 30.0;
const RANGE_BASELINE: f32 = ROW_TOPS[0] + 68.0;

/// 무기 줄 오른쪽 글: 숫자 안내(무기를 가졌을 때), 총을 가졌으면 총 범위 (끝에 닿으면 "최대"). 칸 밖으로 안 나가게 줄인다.
fn draw_weapon_notes(text: &TextPainter, game: &Game) {
    let owned = game.weapons.owned();
    if owned.is_empty() {
        return;
    }
    let note = crate::config::WEAPON_NUMBERS_HELP_TEXT;
    let size = upgrade::fit_text_size(text, note, upgrade::NOTE_SIZE, NOTES.w);
    text.left(note, NOTES.x, NOTE_BASELINE, size, WHITE);
    if owned.contains(&Weapon::Gun) {
        let line = upgrade::gun_range_line(game.weapons.levels().range(Weapon::Gun));
        let size = upgrade::fit_text_size(text, &line, upgrade::RANGE_SIZE, NOTES.w);
        text.left(&line, NOTES.x, RANGE_BASELINE, size, MESSAGE_COLOR);
    }
}

/// 가방 창 전체. 화면 좌표. `show_recipe`: 제작대가 있는 맵이면 곡괭이 만드는 법 한 줄.
pub fn draw_bag_window(text: &TextPainter, game: &Game, art: &M6Art, view: &BagView, upgrade: &UpgradeView, touch_mode: bool, show_recipe: bool) {
    let t = get_time() as f32;
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, DIM);
    draw_rectangle(PANEL.x, PANEL.y, PANEL.w, PANEL.h, PANEL_FILL);
    draw_rectangle_lines(PANEL.x, PANEL.y, PANEL.w, PANEL.h, 8.0, TOY_BOX_BROWN);

    // 제목: 장난감 상자 + "가방", 그 옆에 지금 있는 곳
    draw_toy_box(vec2(PANEL.x + 46.0, 56.0), 58.0, 0.0, t);
    text.left(BAG_NAME_TEXT, PANEL.x + 86.0, TITLE_BASELINE, TITLE_SIZE, TOY_BOX_LIGHT);
    text.left(game.place_name(), PANEL.x + 196.0, TITLE_BASELINE - 4.0, 28.0, WHITE);

    // 닫기
    draw_pill(CLOSE.x, CLOSE.y, CLOSE.w, CLOSE.h, CLOSE_FILL);
    let close_y = if touch_mode { CLOSE.center().y + 16.0 } else { CLOSE.center().y + 6.0 };
    text.centered("닫기", CLOSE.center().x, close_y, 44.0, WHITE);
    if !touch_mode {
        text.centered("Esc", CLOSE.center().x, CLOSE.bottom() - 8.0, 22.0, WHITE);
    }

    let held = game.weapons.held();
    let levels = game.weapons.levels();
    draw_weapon_notes(text, game);
    for (r, slots) in rows().iter().enumerate() {
        text.left(ROW_NAMES[r], ROW_LABEL_X, ROW_TOPS[r] + CELL / 2.0 + 10.0, ROW_LABEL_SIZE, WHITE);
        for (c, slot) in slots.iter().enumerate() {
            let cell = slot_rect(r, c);
            let n = slot_count(game, *slot);
            draw_rectangle(cell.x, cell.y, cell.w, cell.h, CELL_FILL);
            draw_rectangle_lines(cell.x, cell.y, cell.w, cell.h, 3.0, CELL_LINE);
            draw_slot_icon(art, *slot, cell.center() - vec2(0.0, 4.0));
            if n == 0 {
                draw_rectangle(cell.x + 2.0, cell.y + 2.0, cell.w - 4.0, cell.h - 4.0, FADE);
            } else if matches!(slot, BagSlot::Pocket(_) | BagSlot::Material(_)) {
                let label = n.to_string();
                text.left(&label, cell.right() - 6.0 - text.width(&label, COUNT_SIZE), cell.bottom() - 6.0, COUNT_SIZE, WHITE);
            }
            if let (BagSlot::Weapon(w), true) = (slot, n > 0) {
                // 강화 (US-130): 공격력 숫자, 강화했으면 "+n", 방금 강화했으면 반짝
                upgrade::draw_cell_flash(cell, upgrade.flash(*w), t);
                upgrade::draw_cell_numbers(text, cell, *w, levels.attack(*w), levels.level(*w));
            }
            if *slot == BagSlot::Weapon(held) {
                draw_rectangle_lines(cell.x - 3.0, cell.y - 3.0, cell.w + 6.0, cell.h + 6.0, 5.0, HELD_RING);
            }
            if let (BagSlot::Pocket(p), false) = (slot, touch_mode) {
                // 쓰는 것 칸 옆 PC 키 글자 (1, 2, R, 3)
                let key = vec2(cell.x + 12.0, cell.y + 12.0);
                draw_circle(key.x, key.y, KEY_SIZE * 0.6, KEY_FILL);
                text.centered(pocket_key_label(*p), key.x, key.y + KEY_SIZE * 0.36, KEY_SIZE, WHITE);
            }
        }
    }

    if show_recipe && !game.gathering.bag.has_pickaxe {
        text.left(&recipe_line(&game.gathering.bag), ROW_LABEL_X, RECIPE_BASELINE, RECIPE_SIZE, WHITE);
    }
    if let Some(message) = view.message() {
        // 글이 판보다 길면 조금 줄인다 (곡괭이 쓰는 법은 길다)
        let size = (MESSAGE_SIZE * (PANEL.w - 40.0) / text.width(message, MESSAGE_SIZE).max(1.0)).min(MESSAGE_SIZE);
        text.centered(message, PANEL.center().x, MESSAGE_BASELINE, size, MESSAGE_COLOR);
    }
    if !touch_mode {
        draw_rectangle(GUIDE_BAND.x, GUIDE_BAND.y, GUIDE_BAND.w, GUIDE_BAND.h, GUIDE_BAND_FILL);
        for (row, baseline) in full_key_guide().iter().zip(GUIDE_BASELINES) {
            for (line, x) in row.iter().zip(GUIDE_COLUMN_X) {
                // 혹시 칸보다 길면 조금 줄인다 (지금 글은 다 들어간다)
                let size = (GUIDE_SIZE * GUIDE_COLUMN_W / text.width(line, GUIDE_SIZE).max(1.0)).min(GUIDE_SIZE);
                text.left(line, x, baseline, size, WHITE);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::character::CharacterKind;
    use crate::config::{ENDER_FRUIT_ALREADY_EATEN_TEXT, USE_HEALTH_FULL_TEXT, USE_NONE_LEFT_TEXT, USE_NO_MONSTER_ON_SCREEN_TEXT};
    use crate::logic::shop::{Potion, ShopItem};
    use crate::logic::world::World;
    use crate::view::touch::{button_touches_rect, play_buttons_with_mine, BACK_BUTTON};

    fn game() -> Game {
        Game::on_world(CharacterKind::Niro, 7, World::m1())
    }

    const SCREENS: [(f32, f32); 3] = [(960.0, 540.0), (812.0, 375.0), (740.0, 360.0)];

    fn scale(w: f32, h: f32) -> f32 {
        f32::min(w / VIRTUAL_WIDTH, h / VIRTUAL_HEIGHT)
    }

    #[test]
    fn rows_always_have_the_same_slots() {
        let r = rows();
        assert_eq!(r[0].len(), 5, "칼·도끼·총·곡괭이·다이아래더라이트 곡괭이");
        assert_eq!(r[1], Pocket::ALL.map(BagSlot::Pocket).to_vec());
        assert_eq!(r[2].len(), 8, "나무, 철광석, 동, 은, 금, 루비, 에메랄드, 다이아몬드");
        assert_eq!(ROW_NAMES, ["무기", "쓰는 것", "재료"]);
    }

    #[test]
    fn window_cells_are_big_inside_and_apart() {
        let mut all: Vec<Rect> = vec![CLOSE];
        for (r, slots) in rows().iter().enumerate() {
            for c in 0..slots.len() {
                all.push(slot_rect(r, c));
            }
        }
        for (i, a) in all.iter().enumerate() {
            assert!(PANEL.contains(a.point()) && PANEL.contains(vec2(a.right(), a.bottom())), "{a:?} 판 밖");
            for (w, h) in SCREENS {
                assert!(a.w.min(a.h) * scale(w, h) >= 48.0, "{a:?} {w}×{h}에서 작다");
            }
            for b in &all[i + 1..] {
                assert!(!a.overlaps(b), "{a:?} {b:?} 겹친다");
            }
        }
        // 칸 아래 글 줄들은 마지막 칸 줄 아래, 판 안
        let last = ROW_TOPS[2] + CELL;
        assert!(RECIPE_BASELINE - RECIPE_SIZE > last && MESSAGE_BASELINE - MESSAGE_SIZE > RECIPE_BASELINE);
        assert!(GUIDE_BAND.y > MESSAGE_BASELINE + 4.0 && GUIDE_BAND.bottom() < PANEL.bottom() - 4.0, "키 안내 띠는 글 아래, 판 안");
        assert!(GUIDE_BASELINES[0] - GUIDE_SIZE >= GUIDE_BAND.y && GUIDE_BASELINES[2] + 6.0 <= GUIDE_BAND.bottom(), "키 안내 글은 띠 안");
        assert!(ROW_TOPS[0] > CLOSE.bottom(), "닫기와 첫 줄이 안 겹친다");
        // 줄 이름이 첫 칸 왼쪽에 들어간다 ("쓰는 것" 30 크기 ≈ 3글자)
        const { assert!(CELLS_LEFT - ROW_LABEL_X >= ROW_LABEL_SIZE * 3.6) };
    }

    #[test]
    fn weapon_notes_fit_beside_the_weapon_cells() {
        let last = slot_rect(0, 4);
        assert!(NOTES.x >= last.right() && NOTES.right() <= PANEL.right() - 8.0, "칸 오른쪽 빈 곳, 판 안");
        assert!(NOTES.y > CLOSE.bottom() && NOTES.bottom() <= slot_rect(1, 0).y, "닫기와 다음 줄 사이");
        // 가장 긴 글을 글꼴 없이 넉넉히 센 폭 (한글 글자 크기만큼): 안 맞으면 실제로는 줄여 그린다
        let est = |s: &str, size: f32| s.chars().map(|c| if c.is_ascii() { size * 0.62 } else { size }).sum::<f32>();
        assert!(NOTE_BASELINE - upgrade::NOTE_SIZE >= NOTES.y && RANGE_BASELINE <= NOTES.bottom() && NOTE_BASELINE < RANGE_BASELINE - upgrade::RANGE_SIZE + 6.0);
        let range = upgrade::gun_range_line(crate::config::GUN_RANGE_MAX);
        assert!(est(&range, upgrade::RANGE_SIZE) <= NOTES.w + 40.0, "총 범위 글 폭 {}", est(&range, upgrade::RANGE_SIZE));
        for size in [upgrade::NOTE_SIZE, upgrade::RANGE_SIZE, upgrade::ATTACK_SIZE, upgrade::LEVEL_SIZE] {
            assert!(size * scale(740.0, 360.0) >= 16.0, "{size}");
        }
    }

    #[test]
    fn small_text_is_readable_on_a_small_phone() {
        for size in [COUNT_SIZE, RECIPE_SIZE, MESSAGE_SIZE, ROW_LABEL_SIZE, GUIDE_SIZE] {
            assert!(size * scale(740.0, 360.0) >= 16.0, "{size}");
        }
    }

    #[test]
    fn taps_find_the_right_slot() {
        assert_eq!(bag_tap_at(CLOSE.center()), Some(BagTap::Close));
        assert_eq!(bag_tap_at(slot_rect(0, 0).center()), Some(BagTap::Slot(BagSlot::Weapon(Weapon::Sword))));
        assert_eq!(bag_tap_at(slot_rect(1, 3).center()), Some(BagTap::Slot(BagSlot::Pocket(Pocket::EnderFruit))));
        assert_eq!(bag_tap_at(slot_rect(2, 7).center()), Some(BagTap::Slot(BagSlot::Material(Material::Diamond))));
        assert_eq!(bag_tap_at(vec2(5.0, 5.0)), None);
    }

    #[test]
    fn phone_bag_button_is_big_in_the_corner_and_clear_of_buttons() {
        let b = bag_button_rect(true);
        for (w, h) in SCREENS {
            assert!(b.w.min(b.h) * scale(w, h) >= 48.0);
        }
        assert!(b.right() <= VIRTUAL_WIDTH && b.y >= 0.0 && b.y < 20.0 && b.right() > VIRTUAL_WIDTH - 20.0, "오른쪽 위 구석");
        assert!(!b.overlaps(&BACK_BUTTON));
        assert!(!b.overlaps(&crate::view::qa::QA_BADGE));
        for btn in play_buttons_with_mine(true, true) {
            assert!(!button_touches_rect(&btn, b), "{:?}", btn.kind);
        }
        let pc = bag_button_rect(false);
        assert!(pc.w < b.w && pc.right() <= VIRTUAL_WIDTH, "PC는 작게");
    }

    #[test]
    fn toy_box_is_brown() {
        const { assert!(TOY_BOX_BROWN.r > TOY_BOX_BROWN.g && TOY_BOX_BROWN.g > TOY_BOX_BROWN.b && TOY_BOX_BROWN.r < 0.8, "아이 확정: 갈색") };
    }

    #[test]
    fn weapon_slot_holds_that_weapon_and_closes() {
        let mut g = game();
        let mut v = BagView::new();
        v.open_bag();
        assert!(!v.apply(&mut g, BagTap::Slot(BagSlot::Weapon(Weapon::Axe)), false), "없는 무기는 아무 일 없다");
        assert!(v.open && g.weapons.held() == Weapon::Hands);
        g.weapons.add_bought(Weapon::Sword);
        g.weapons.add_bought(Weapon::Axe);
        assert!(v.apply(&mut g, BagTap::Slot(BagSlot::Weapon(Weapon::Sword)), false));
        assert_eq!(g.weapons.held(), Weapon::Sword);
        assert!(!v.open, "들고 바로 닫힌다");
        // 곡괭이 칸은 손에 드는 무기가 아니라서 아무 일 없다
        g.gathering.bag.has_pickaxe = true;
        v.open_bag();
        assert!(!v.apply(&mut g, BagTap::Slot(BagSlot::Pickaxe), false) && v.open);
    }

    #[test]
    fn pickaxe_slots_show_how_to_use_and_keep_the_window() {
        let mut g = game();
        let mut v = BagView::new();
        v.open_bag();
        assert!(!v.apply(&mut g, BagTap::Slot(BagSlot::Pickaxe), false));
        assert_eq!(v.message(), None, "없는 곡괭이는 아무 일 없다");
        g.gathering.bag.has_pickaxe = true;
        g.gathering.bag.has_diamond_pickaxe = true;
        assert!(!v.apply(&mut g, BagTap::Slot(BagSlot::Pickaxe), false) && v.open);
        assert_eq!(v.message(), Some(PICKAXE_HOW_TEXT));
        assert!(!v.apply(&mut g, BagTap::Slot(BagSlot::DiamondPickaxe), false) && v.open);
        assert_eq!(v.message(), Some(DIAMOND_PICKAXE_HOW_TEXT));
        // 폰은 E 대신 버튼 이름
        assert!(!v.apply(&mut g, BagTap::Slot(BagSlot::Pickaxe), true));
        assert_eq!(v.message(), Some(PICKAXE_HOW_TOUCH_TEXT));
        assert!(!v.apply(&mut g, BagTap::Slot(BagSlot::DiamondPickaxe), true));
        assert_eq!(v.message(), Some(DIAMOND_PICKAXE_HOW_TOUCH_TEXT));
        assert!(!PICKAXE_HOW_TOUCH_TEXT.contains('E') && !DIAMOND_PICKAXE_HOW_TOUCH_TEXT.contains('E'));
    }

    #[test]
    fn key_or_phone_use_says_why_it_cannot() {
        let mut g = game();
        assert_eq!(use_pocket_or_why(&mut g, Pocket::Potion(Potion::Health)), Some(USE_NONE_LEFT_TEXT));
        g.wallet.add(100);
        g.buy(ShopItem::HealthPotion).unwrap();
        assert_eq!(use_pocket_or_why(&mut g, Pocket::Potion(Potion::Health)), Some(USE_HEALTH_FULL_TEXT));
        assert_eq!(g.shop.potions(Potion::Health), 1, "안 쓰인다");
        g.combat.health.lose_from_quiz(1);
        assert_eq!(use_pocket_or_why(&mut g, Pocket::Potion(Potion::Health)), None);
        assert_eq!(g.shop.potions(Potion::Health), 0, "쓰였다");
        // 엔더열매: 벌써 먹었으면 까닭만, logic 이벤트는 없다 (같은 글이 두 번 나오지 않게)
        g.gathering.bag.ender_fruits = 1;
        g.gathering.bag.has_ender_power = true;
        g.take_item_events();
        assert_eq!(use_pocket_or_why(&mut g, Pocket::EnderFruit), Some(ENDER_FRUIT_ALREADY_EATEN_TEXT));
        assert!(g.take_item_events().is_empty());
    }

    #[test]
    fn long_messages_fit_inside_the_window() {
        // 그릴 때 판 폭에 맞춰 줄이지만, 너무 작아지지 않게: 줄인 크기도 740x360 폰에서 16픽셀 넘게
        let est = |s: &str| s.chars().map(|c| if c.is_ascii() { 0.5 } else { 1.0 }).sum::<f32>() * MESSAGE_SIZE;
        for m in [PICKAXE_HOW_TEXT, PICKAXE_HOW_TOUCH_TEXT, DIAMOND_PICKAXE_HOW_TEXT, DIAMOND_PICKAXE_HOW_TOUCH_TEXT] {
            let size = (MESSAGE_SIZE * (PANEL.w - 40.0) / est(m)).min(MESSAGE_SIZE);
            assert!(size * scale(740.0, 360.0) >= 16.0, "{m}: {size}");
        }
    }

    #[test]
    fn potion_slot_uses_it_and_closes_like_the_key() {
        let mut g = game();
        g.wallet.add(100);
        g.buy(ShopItem::HealthPotion).unwrap();
        let mut v = BagView::new();
        v.open_bag();
        assert!(!v.apply(&mut g, BagTap::Slot(BagSlot::Pocket(Pocket::Potion(Potion::Health))), false), "체력이 가득이면 안 쓰이고 창 그대로");
        assert!(v.open && g.shop.potions(Potion::Health) == 1);
        assert_eq!(v.message(), Some(USE_HEALTH_FULL_TEXT), "까닭을 창 안에 보인다");
        g.combat.health.lose_from_quiz(1);
        assert!(v.apply(&mut g, BagTap::Slot(BagSlot::Pocket(Pocket::Potion(Potion::Health))), false));
        assert!(!v.open && g.shop.potions(Potion::Health) == 0, "하나 줄고 닫힌다");
        assert_eq!(g.take_shop_events().len(), 1, "1 키와 같은 결과");
    }

    #[test]
    fn syringe_gun_without_monsters_keeps_the_bag_open() {
        let mut g = game();
        g.gathering.bag.syringe_guns = 1;
        g.combat.zombies.list.clear();
        g.combat.skeletons.list.clear();
        let mut v = BagView::new();
        v.open_bag();
        assert!(!v.apply(&mut g, BagTap::Slot(BagSlot::Pocket(Pocket::SyringeGun)), false));
        assert!(v.open && g.gathering.bag.syringe_guns == 1);
        assert_eq!(v.message(), Some(USE_NO_MONSTER_ON_SCREEN_TEXT));
    }

    #[test]
    fn ender_fruit_eaten_twice_says_so_inside_the_bag() {
        let mut g = game();
        g.gathering.bag.ender_fruits = 2;
        let mut v = BagView::new();
        v.open_bag();
        assert!(v.apply(&mut g, BagTap::Slot(BagSlot::Pocket(Pocket::EnderFruit)), false));
        assert!(!v.open && g.gathering.bag.ender_fruits == 1);
        g.take_item_events();
        v.open_bag();
        assert!(!v.apply(&mut g, BagTap::Slot(BagSlot::Pocket(Pocket::EnderFruit)), false));
        assert!(v.open, "창 그대로");
        assert_eq!(v.message(), Some(ENDER_FRUIT_ALREADY_EATEN_TEXT));
        assert_eq!(g.gathering.bag.ender_fruits, 1, "줄지 않는다");
        assert!(g.take_item_events().is_empty(), "닫은 뒤 큰 글로 또 나오지 않는다");
        v.close();
        assert_eq!(v.message(), None);
    }

    #[test]
    fn empty_slots_do_nothing_and_close_closes() {
        let mut g = game();
        let mut v = BagView::new();
        v.open_bag();
        for slots in rows() {
            for s in slots {
                assert!(!v.apply(&mut g, BagTap::Slot(s), false), "{s:?}");
                // 없는 쓰는 것은 "남은 게 없어요", 나머지 빈 칸은 글 없음
                let want = matches!(s, BagSlot::Pocket(_)).then_some(USE_NONE_LEFT_TEXT);
                assert_eq!(v.message(), want, "{s:?}");
            }
        }
        assert!(v.open);
        assert!(v.apply(&mut g, BagTap::Close, false) && !v.open);
    }

    #[test]
    fn toy_box_flashes_two_seconds_when_something_new_comes() {
        let mut g = game();
        let mut v = BagView::new();
        v.watch(all_counts(&g));
        assert_eq!(v.flash(), 0.0, "처음 본 것으로는 반짝이지 않는다");
        g.gathering.bag.add(Material::Wood);
        v.watch(all_counts(&g));
        assert_eq!(v.flash(), 1.0);
        v.update(1.0, true);
        assert!(v.flash() > 0.0);
        v.update(1.01, true);
        assert_eq!(v.flash(), 0.0, "2초 뒤 멈춘다");
        g.combat.health.lose_from_quiz(1);
        v.watch(all_counts(&g));
        assert_eq!(v.flash(), 0.0, "줄거나 그대로면 안 반짝인다");
    }

    #[test]
    fn key_hint_shows_eight_seconds_and_waits_while_windows_are_open() {
        let mut v = BagView::new();
        assert_eq!(v.hint_alpha(), 1.0);
        for _ in 0..600 {
            v.update(1.0 / 60.0, false);
        }
        assert_eq!(v.hint_alpha(), 1.0, "멈춤·가방·퀴즈 창이 열린 동안은 줄지 않는다");
        for _ in 0..(6.5 * 60.0) as usize {
            v.update(1.0 / 60.0, true);
        }
        assert_eq!(v.hint_alpha(), 1.0);
        for _ in 0..60 {
            v.update(1.0 / 60.0, true);
        }
        assert!(v.hint_alpha() > 0.0 && v.hint_alpha() < 1.0, "흐려진다");
        for _ in 0..60 {
            v.update(1.0 / 60.0, true);
        }
        assert_eq!(v.hint_alpha(), 0.0, "8초 뒤 사라진다");
        assert_eq!(BagView::new().hint_alpha(), 1.0, "새 게임에서 다시 나온다");
    }

    #[test]
    fn key_texts_follow_the_new_keys() {
        assert_eq!(short_key_hint(), "I: 가방  M: 소리  Esc: 멈춤");
        let lines: Vec<String> = full_key_guide().into_iter().flatten().collect();
        let all = lines.join("  ");
        for keys in ["Q: 공격", "Tab: 무기 바꾸기", "캐기, 들어가기", "Space, 위 화살표: 점프", "1, 2, R, 3: 쓰기", "I: 가방", "Esc: 멈춤", "M: 소리"] {
            assert!(all.contains(keys), "{keys} 없음: {all}");
        }
        // 빨리감기(F)는 숨은 기능이라 안내에 없다 (부모님 결정 2026-11-15)
        assert!(!all.contains("빨리감기") && !all.contains("F:"), "{all}");
        assert!(!short_key_hint().contains("빨리감기"));
        assert!(short_key_hint().contains("M: 소리"), "BUG-041: 소리 키 안내");
        for line in lines.into_iter().chain([short_key_hint()]) {
            assert!(!line.chars().any(|c| "←↑→↓×÷…①②③④□○「」—·".contains(c)), "Jua에 없는 글자: {line}");
        }
    }

    /// BUG-033: 키 안내 한 칸이 줄이지 않은 크기로 칸에 들어간다 (넉넉히 셈: 한글 한 글자 = 크기, 나머지 = 크기 × 0.6).
    #[test]
    fn key_guide_fits_its_column_without_shrinking() {
        for line in full_key_guide().into_iter().flatten() {
            let w: f32 = line.chars().map(|c| if c as u32 >= 0xAC00 { GUIDE_SIZE } else { GUIDE_SIZE * 0.6 }).sum();
            assert!(w <= GUIDE_COLUMN_W, "{line}: {w} > {GUIDE_COLUMN_W}");
        }
        assert!(GUIDE_COLUMN_X[0] + GUIDE_COLUMN_W < GUIDE_COLUMN_X[1], "두 칸이 겹치지 않는다");
        assert!(GUIDE_COLUMN_X[1] + GUIDE_COLUMN_W <= GUIDE_BAND.right(), "오른쪽 칸이 띠 안");
    }
}
