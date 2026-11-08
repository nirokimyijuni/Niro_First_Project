//! M7 캐기·동굴 화면 (로건): 나무·그루터기·광석 덩이·동굴 입구와 출구·동굴 안·제작대·곡괭이의 임시 그림,
//! 머리 위 캐기 표시, 캔 것 팝업, 재료 판. 그림은 모두 코드로 그린 임시 그림이다 ([임시-20]).
//! 무엇이 어디 있고 언제 다시 생기는지는 logic(gathering, game, world)이 정하고, 여기는 보여 주기만 한다.

use super::hud::draw_pill;
use super::screen::{Letterbox, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::sky::night;
use super::text::TextPainter;
use super::touch::{draw_pickaxe_shape, draw_star, glow_pulse};
use crate::logic::game::{Game, Usable};
use crate::config::{CRAFT_IRON_MARK, CRAFT_WOOD_MARK, PICKAXE_SHAPE};
use crate::logic::gathering::{Bag, Material};
use crate::logic::world::nearest_copy;
use macroquad::prelude::*;

// ---- 색 (밝고 무섭지 않게) ----
const LEAF: Color = Color::new(0.35, 0.72, 0.30, 1.0);
const LEAF_LIGHT: Color = Color::new(0.50, 0.84, 0.38, 1.0);
const TRUNK: Color = Color::new(0.58, 0.38, 0.20, 1.0);
const TRUNK_TOP: Color = Color::new(0.88, 0.70, 0.45, 1.0);
const LINE: Color = Color::new(0.30, 0.18, 0.08, 1.0);
const STONE: Color = Color::new(0.62, 0.56, 0.54, 1.0);
const STONE_DARK: Color = Color::new(0.42, 0.37, 0.36, 1.0);
const CAVE_INSIDE: Color = Color::new(0.30, 0.22, 0.26, 1.0);
const LANTERN: Color = Color::new(1.0, 0.86, 0.35, 1.0);
const LANTERN_GLOW: Color = Color::new(1.0, 0.86, 0.35, 0.25);
const OUTSIDE_LIGHT: Color = Color::new(0.70, 0.88, 1.0, 1.0);
const CAVE_WALL: Color = Color::new(0.56, 0.46, 0.55, 1.0);
const CAVE_WALL_LIGHT: Color = Color::new(0.64, 0.54, 0.62, 1.0);
const CAVE_FLOOR: Color = Color::new(0.62, 0.48, 0.36, 1.0);
const CAVE_FLOOR_TOP: Color = Color::new(0.74, 0.60, 0.46, 1.0);
const CRYSTALS: [Color; 3] = [
    Color::new(0.55, 0.85, 1.0, 0.9),
    Color::new(1.0, 0.70, 0.88, 0.9),
    Color::new(0.75, 1.0, 0.70, 0.9),
];
const WOOD: Color = Color::new(0.80, 0.56, 0.30, 1.0);
const WOOD_LIGHT: Color = Color::new(0.92, 0.72, 0.45, 1.0);
const HANDLE: Color = Color::new(0.70, 0.46, 0.22, 1.0);
const PANEL: Color = Color::new(0.10, 0.13, 0.32, 0.55);
const PROGRESS_BACK: Color = Color::new(0.10, 0.13, 0.32, 0.7);
const PROGRESS_FILL: Color = Color::new(0.55, 0.90, 0.35, 1.0);
const POPUP_TEXT: Color = Color::new(1.0, 0.86, 0.22, 1.0);
const SPARKLE: Color = Color::new(1.0, 0.97, 0.75, 1.0);

/// 밤에 맵의 임시 그림이 어두워지는 정도 (상자와 비슷하게 덜 어둡게)
const NIGHT_AMOUNT: f32 = 0.6;

// ---- 정보판 아래 재료 판 ----
/// "◀ 고르기"(y 78~138)와 체력 판 아래 왼쪽 끝.
pub const BAG_TOP: f32 = 146.0;
const BAG_LEFT: f32 = 12.0;
const BAG_HEIGHT: f32 = 44.0;
const BAG_ICON: f32 = 30.0;
const BAG_TEXT: f32 = 28.0;
const BAG_PAD: f32 = 14.0;
const BAG_GAP: f32 = 14.0;

// ---- 팝업 (US-065) ----
/// 약 1.5초 보이고 저절로 사라진다. 게임을 멈추지 않는다.
pub const POPUP_SECONDS: f32 = 1.5;
/// 정보판·체력 판 아래, 조이스틱·버튼보다 위
pub const POPUP_TOP: f32 = 228.0;
const POPUP_HEIGHT: f32 = 58.0;

// ---- 곡괭이 만드는 법 (부모님 요청) ----
/// 재료 판 바로 아래 작은 한 줄. 곡괭이가 생기면 숨긴다.
const RECIPE_TOP: f32 = BAG_TOP + BAG_HEIGHT + 4.0;
const RECIPE_HEIGHT: f32 = 32.0;
const RECIPE_TEXT: f32 = 26.0;
/// 처음 나무를 캤을 때 한 번 나오는 안내 두 줄이 보이는 초
const RECIPE_HINT_SECONDS: f32 = 4.0;
/// 안내 상자: 정보판 아래 오른쪽 (재료 판과 같은 높이), 팝업·바꾸기 버튼·캐릭터보다 위
const RECIPE_HINT_TOP: f32 = BAG_TOP;
const RECIPE_HINT_SIZE: f32 = 26.0;
const RECIPE_HINT_LINE: f32 = RECIPE_HINT_SIZE + 6.0;
const RECIPE_HINT_HEIGHT: f32 = RECIPE_HINT_LINE * 2.0 + 16.0;

/// "나무 4개와 철광석 2개로 곡괭이를 만들 수 있어요!" / "집 제작대로 가요". 곡괭이 모양이 바뀌어도 개수가 맞는다.
fn recipe_hint_lines() -> [String; 2] {
    let [(wood, w), (iron, i)] = pickaxe_needs();
    [format!("{} {}개와 {} {}개로 곡괭이를 만들 수 있어요!", wood.name(), w, iron.name(), i), "집 제작대로 가요".to_string()]
}

/// 곡괭이 한 개에 드는 (재료, 개수). `PICKAXE_SHAPE`의 표시를 센다.
pub fn pickaxe_needs() -> [(Material, u32); 2] {
    let count = |mark: char| PICKAXE_SHAPE.iter().flatten().filter(|c| **c == mark).count() as u32;
    [(Material::Wood, count(CRAFT_WOOD_MARK)), (Material::Iron, count(CRAFT_IRON_MARK))]
}

/// "곡괭이: 나무 1/4 + 철광석 0/2 (집 제작대)". Jua 글꼴에 →가 없어서 괄호로 쓴다.
fn recipe_line(bag: &Bag) -> String {
    let parts: Vec<String> = pickaxe_needs().iter().map(|(m, n)| format!("{} {}/{}", m.name(), bag.count(*m).min(*n), n)).collect();
    format!("곡괭이: {} (집 제작대)", parts.join(" + "))
}
const POPUP_TEXT_SIZE: f32 = 40.0;
const POPUP_ICON: f32 = 48.0;

/// 머리 위 캐기 표시 크기와 머리에서 떨어진 거리
const PROGRESS_WIDTH: f32 = 64.0;
const PROGRESS_HEIGHT: f32 = 14.0;
const PROGRESS_ABOVE: f32 = 22.0;

/// 재료 그림 색 [임시-20]: 철광석 회색, 동 주황, 은 밝은 회색, 금 노랑.
pub fn material_color(material: Material) -> Color {
    match material {
        Material::Wood => TRUNK,
        Material::Iron => Color::new(0.55, 0.58, 0.63, 1.0),
        Material::Copper => Color::new(0.95, 0.55, 0.22, 1.0),
        Material::Silver => Color::new(0.88, 0.90, 0.95, 1.0),
        Material::Gold => Color::new(1.0, 0.82, 0.18, 1.0),
    }
}

/// 금·은은 팝업에서 더 반짝인다 (US-065).
pub fn sparkles(material: Material) -> bool {
    matches!(material, Material::Silver | Material::Gold)
}

/// 재료 그림 하나. 나무는 통나무, 광석은 반짝이는 덩이.
pub fn draw_material_icon(material: Material, c: Vec2, size: f32) {
    if material == Material::Wood {
        let (w, h) = (size * 0.9, size * 0.5);
        draw_rectangle(c.x - w / 2.0, c.y - h / 2.0, w, h, TRUNK);
        draw_rectangle_lines(c.x - w / 2.0, c.y - h / 2.0, w, h, 2.0, LINE);
        draw_circle(c.x + w / 2.0, c.y, h / 2.0, TRUNK_TOP);
        draw_circle_lines(c.x + w / 2.0, c.y, h / 2.0, 2.0, LINE);
        draw_circle_lines(c.x + w / 2.0, c.y, h / 4.0, 1.5, TRUNK);
        return;
    }
    let color = material_color(material);
    let dark = Color::new(color.r * 0.55, color.g * 0.55, color.b * 0.55, 1.0);
    draw_poly(c.x, c.y, 6, size * 0.5, 0.0, color);
    draw_poly_lines(c.x, c.y, 6, size * 0.5, 0.0, 2.5, dark);
    draw_circle(c.x - size * 0.15, c.y - size * 0.15, size * 0.1, Color::new(1.0, 1.0, 1.0, 0.8));
}

/// 곡괭이 [임시-19, 임시-20]: 나무 손잡이 + 철 머리. 아이 그림이 오면 `touch::draw_pickaxe_shape`만 바꾼다.
/// `tint`가 흰색이 아니면 그 색 한 가지로 그린다.
pub fn draw_pickaxe(c: Vec2, size: f32, tint: Color) {
    if tint != WHITE {
        draw_pickaxe_shape(c, size, tint, tint);
    } else {
        draw_pickaxe_shape(c, size, HANDLE, material_color(Material::Iron));
    }
}

/// 나무 한 그루 (캘 수 있을 때) 또는 그루터기. `r`은 화면에 가장 가까운 자리다.
fn draw_tree(r: Rect, ready: bool, amount: f32) {
    let trunk_w = r.w * 0.34;
    let tx = r.center().x - trunk_w / 2.0;
    if !ready {
        let h = r.h * 0.2;
        draw_rectangle(tx, r.y + r.h - h, trunk_w, h, night(TRUNK, amount));
        draw_ellipse(r.center().x, r.y + r.h - h, trunk_w / 2.0, trunk_w * 0.22, 0.0, night(TRUNK_TOP, amount));
        // 다시 자라는 새싹
        draw_circle(r.center().x + trunk_w * 0.2, r.y + r.h - h - 6.0, 4.0, night(LEAF_LIGHT, amount));
        return;
    }
    let trunk_h = r.h * 0.45;
    draw_rectangle(tx, r.y + r.h - trunk_h, trunk_w, trunk_h, night(TRUNK, amount));
    draw_rectangle_lines(tx, r.y + r.h - trunk_h, trunk_w, trunk_h, 2.0, night(LINE, amount));
    let crown = r.y + r.h * 0.36;
    let (leaf, light) = (night(LEAF, amount), night(LEAF_LIGHT, amount));
    draw_circle(r.center().x - r.w * 0.3, crown + r.w * 0.1, r.w * 0.38, leaf);
    draw_circle(r.center().x + r.w * 0.3, crown + r.w * 0.1, r.w * 0.38, leaf);
    draw_circle(r.center().x, crown - r.w * 0.2, r.w * 0.45, leaf);
    draw_circle(r.center().x - r.w * 0.12, crown - r.w * 0.32, r.w * 0.16, light);
}

/// 둥근 문 모양: 위는 반원, 아래는 네모. `r` 안을 채운다.
fn draw_arch(r: Rect, color: Color) {
    let radius = r.w / 2.0;
    draw_circle(r.center().x, r.y + radius, radius, color);
    draw_rectangle(r.x, r.y + radius, r.w, r.h - radius, color);
}

/// 동굴 입구 또는 출구: 돌 테두리 + 안쪽. 입구 안은 따뜻한 어둠과 등불, 출구 안은 바깥 빛.
fn draw_cave_door(r: Rect, is_exit: bool, amount: f32) {
    let rim = 12.0;
    draw_arch(r, night(STONE, amount));
    let inner = Rect::new(r.x + rim, r.y + rim, r.w - rim * 2.0, r.h - rim);
    draw_arch(inner, night(if is_exit { OUTSIDE_LIGHT } else { CAVE_INSIDE }, amount));
    for i in 0..5 {
        let a = std::f32::consts::PI * (i as f32 + 0.5) / 5.0;
        let p = vec2(r.center().x - a.cos() * (r.w / 2.0 - rim / 2.0), r.y + r.w / 2.0 - a.sin() * (r.w / 2.0 - rim / 2.0));
        draw_circle(p.x, p.y, rim * 0.45, night(STONE_DARK, amount));
    }
    if is_exit {
        draw_circle(inner.center().x, inner.y + inner.w * 0.45, inner.w * 0.18, Color::new(1.0, 0.95, 0.6, 1.0));
    } else {
        let lamp = vec2(r.x + r.w + 4.0, r.y + r.h * 0.45);
        draw_line(lamp.x, lamp.y, lamp.x, r.y + r.h, 3.0, night(LINE, amount));
        draw_circle(lamp.x, lamp.y, 16.0, LANTERN_GLOW);
        draw_circle(lamp.x, lamp.y, 7.0, LANTERN);
    }
}

/// 광석 덩이 (캘 수 있을 때만). 무엇이 나올지는 캘 때 정해지니 여러 색 반짝이를 박는다.
fn draw_ore_lump(r: Rect) {
    let c = vec2(r.center().x, r.y + r.h * 0.55);
    draw_poly(c.x, c.y, 7, r.w * 0.5, 10.0, STONE);
    draw_poly_lines(c.x, c.y, 7, r.w * 0.5, 10.0, 3.0, STONE_DARK);
    let specks = [(Material::Iron, -0.2, -0.1), (Material::Copper, 0.18, -0.18), (Material::Silver, 0.05, 0.15), (Material::Gold, -0.12, 0.2)];
    for (m, dx, dy) in specks {
        draw_poly(c.x + dx * r.w, c.y + dy * r.w, 4, r.w * 0.09, 45.0, material_color(m));
    }
}

/// 집의 제작대: 나무 탁자 위에 4×3 칸 무늬.
fn draw_workbench(r: Rect, amount: f32) {
    let top_y = r.y + r.h * 0.3;
    let top_h = r.h * 0.2;
    let leg_w = r.w * 0.14;
    for x in [r.x + 4.0, r.x + r.w - 4.0 - leg_w] {
        draw_rectangle(x, top_y, leg_w, r.y + r.h - top_y, night(TRUNK, amount));
    }
    draw_rectangle(r.x, top_y, r.w, top_h, night(WOOD, amount));
    draw_rectangle_lines(r.x, top_y, r.w, top_h, 2.0, night(LINE, amount));
    // 탁자 위 작은 칸 판
    let grid = Rect::new(r.x + r.w * 0.15, r.y, r.w * 0.7, r.h * 0.3);
    draw_rectangle(grid.x, grid.y, grid.w, grid.h, night(WOOD_LIGHT, amount));
    for i in 0..=4 {
        let x = grid.x + grid.w * i as f32 / 4.0;
        draw_line(x, grid.y, x, grid.y + grid.h, 1.5, night(LINE, amount));
    }
    for i in 0..=3 {
        let y = grid.y + grid.h * i as f32 / 3.0;
        draw_line(grid.x, y, grid.x + grid.w, y, 1.5, night(LINE, amount));
    }
    draw_pickaxe(vec2(r.x + r.w * 0.82, top_y + top_h + 14.0), 20.0, WHITE);
}

/// 긴 맵의 나무·동굴 입구·제작대. 카메라가 맵에 걸려 있을 때 부른다 (screens::draw_play).
/// 고리 맵에서는 화면에 가장 가까운 한 자리(`nearest_copy`)에만 그린다 (D61).
pub fn draw_map_things(game: &Game, view_x: f32, d: f32) {
    let amount = d * NIGHT_AMOUNT;
    if let Some(r) = game.world.cave_entrance() {
        draw_cave_door(nearest_copy(r, view_x), false, amount);
    }
    if let Some(r) = game.world.workbench() {
        draw_workbench(nearest_copy(r, view_x), amount);
    }
    for tree in &game.gathering.trees {
        draw_tree(nearest_copy(tree.rect, view_x), tree.is_ready(), amount);
    }
}

// ---- 쓸 수 있는 물건 표시 (부모님 요청: 근처에 가면 눈에 띄게) ----
/// 빛 테두리가 물건 밖으로 가장 작게·크게 나오는 만큼
const GLOW_MIN: f32 = 6.0;
const GLOW_MAX: f32 = 16.0;
/// 밤에도 보이게 밤 색을 섞지 않는 밝은 노란빛
const GLOW: Color = Color::new(1.0, 0.93, 0.45, 1.0);
/// 머리 위 표시가 물건 위에 떠 있는 틈과 통통 튀는 높이·빠르기
const TAG_GAP: f32 = 14.0;
const TAG_BOB: f32 = 10.0;
const TAG_BOUNCES_PER_SECOND: f32 = 2.0;
const KEY_SIZE: f32 = 46.0;
const KEY_FILL: Color = Color::new(1.0, 1.0, 1.0, 0.95);
const KEY_TEXT: Color = Color::new(0.20, 0.22, 0.45, 1.0);

/// 머리 위 표시가 바닥에서 떠오른 높이 (0..=TAG_BOB). 공처럼 튄다.
fn tag_bounce(t: f32) -> f32 {
    TAG_BOB * (t * TAG_BOUNCES_PER_SECOND * std::f32::consts::PI).sin().abs()
}

/// 쓸 수 있는 물건의 그리는 자리. 맵에서는 화면에 가장 가까운 자리, 동굴 안은 그대로다.
fn usable_rect(game: &Game, usable: Usable, view_x: f32) -> Option<Rect> {
    let r = match usable {
        Usable::Tree(i) => game.gathering.trees.get(i)?.rect,
        Usable::OreLump(i) => return Some(game.gathering.ore_lumps.get(i)?.rect),
        Usable::CaveExit => return Some(game.cave_world.cave_exit()),
        Usable::CaveEntrance => game.world.cave_entrance()?,
        Usable::Workbench => game.world.workbench()?,
    };
    Some(nearest_copy(r, view_x))
}

/// 손 닿는 물건 둘레의 빛. 물건 바로 위·캐릭터 아래에 그린다 (맵 좌표).
/// `usable_in_reach`가 수학 창·제작대·끝난 판에는 None이라 그때는 아무것도 안 그린다.
pub fn draw_usable_glow(game: &Game, view_x: f32, t: f32) {
    let Some(r) = game.usable_in_reach().and_then(|u| usable_rect(game, u, view_x)) else { return };
    let p = glow_pulse(t);
    let grow = GLOW_MIN + (GLOW_MAX - GLOW_MIN) * p;
    let (c, rx, ry) = (r.center(), r.w / 2.0 + grow, r.h / 2.0 + grow);
    draw_ellipse(c.x, c.y, rx, ry, 0.0, Color { a: 0.12 + 0.12 * p, ..GLOW });
    draw_ellipse_lines(c.x, c.y, rx, ry, 0.0, 5.0, Color { a: 0.55 + 0.4 * p, ..GLOW });
    // 양옆 위에 작은 별 두 개가 돈다
    for side in [-1.0, 1.0] {
        draw_star(vec2(c.x + side * rx * 0.75, c.y - ry * 0.7), 6.0 + 5.0 * p, t * side, SPARKLE);
    }
}

/// 손 닿는 물건 위에 통통 튀는 표시: PC는 "E" 키, 폰은 캐기 버튼 글 (맵 좌표).
pub fn draw_usable_tag(text: &TextPainter, game: &Game, view_x: f32, camera_left: f32, touch_mode: bool, t: f32) {
    let Some(usable) = game.usable_in_reach() else { return };
    let Some(r) = usable_rect(game, usable, view_x) else { return };
    let bottom = r.y - TAG_GAP - tag_bounce(t);
    if touch_mode {
        super::effects::draw_bubble(text, usable.button_text(), r.center().x, bottom, camera_left, 1.0);
        return;
    }
    let key = Rect::new(r.center().x - KEY_SIZE / 2.0, bottom - KEY_SIZE, KEY_SIZE, KEY_SIZE);
    draw_rectangle(key.x, key.y + 5.0, key.w, key.h, KEY_TEXT); // 키 옆면
    draw_rectangle(key.x, key.y, key.w, key.h, KEY_FILL);
    draw_rectangle_lines(key.x, key.y, key.w, key.h, 3.0, KEY_TEXT);
    text.centered("E", key.center().x, key.y + key.h - 11.0, 34.0, KEY_TEXT);
}

/// 동굴 안 한 화면 (임시-12): 벽, 땅, 수정, 등불, 출구, 광석 덩이. 카메라를 `camera_x`에 건다.
/// 맵의 코인·미스·시민·상자·몬스터는 그리지 않는다 (screens::draw_play가 건너뛴다).
pub fn draw_cave(game: &Game, camera_x: f32) {
    Letterbox::current().look_at(camera_x);
    let (left, ground_y) = (camera_x, game.cave_world.ground_y);
    draw_rectangle(left, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, CAVE_WALL);
    // 벽 무늬: 옅은 돌 덩어리
    for i in 0..9 {
        let x = left + 40.0 + i as f32 * 110.0;
        let y = 150.0 + ((i * 37) % 5) as f32 * 40.0;
        draw_ellipse(x, y, 46.0, 22.0, 0.0, CAVE_WALL_LIGHT);
    }
    // 천장 고드름 돌 (둥근 세모)
    for i in 0..12 {
        let x = left + 20.0 + i as f32 * 82.0;
        let h = 30.0 + ((i * 53) % 4) as f32 * 12.0;
        draw_triangle(vec2(x - 22.0, 0.0), vec2(x + 22.0, 0.0), vec2(x, h), STONE_DARK);
    }
    // 반짝이는 수정 (밝은 색)
    for i in 0..7 {
        let x = left + 150.0 + i as f32 * 120.0;
        let y = 250.0 + ((i * 29) % 3) as f32 * 35.0;
        let color = CRYSTALS[i % CRYSTALS.len()];
        draw_triangle(vec2(x - 8.0, y + 20.0), vec2(x + 8.0, y + 20.0), vec2(x, y - 12.0), color);
        draw_triangle(vec2(x + 4.0, y + 20.0), vec2(x + 16.0, y + 20.0), vec2(x + 12.0, y), color);
    }
    // 매달린 등불
    for i in 0..4 {
        let x = left + 200.0 + i as f32 * 220.0;
        draw_line(x, 0.0, x, 90.0, 2.0, STONE_DARK);
        draw_circle(x, 98.0, 34.0, LANTERN_GLOW);
        draw_circle(x, 98.0, 12.0, LANTERN);
    }
    draw_rectangle(left, ground_y, VIRTUAL_WIDTH, VIRTUAL_HEIGHT - ground_y, CAVE_FLOOR);
    draw_rectangle(left, ground_y, VIRTUAL_WIDTH, 8.0, CAVE_FLOOR_TOP);
    draw_cave_door(game.cave_world.cave_exit(), true, 0.0);
    for lump in game.gathering.ore_lumps.iter().filter(|l| l.is_ready()) {
        draw_ore_lump(lump.rect);
    }
}

/// 캐는 동안 머리 위 표시 (US-022). `head`는 몸 네모(그리는 좌표)다.
pub fn draw_mining_progress(head: Rect, progress: Option<f32>) {
    let Some(progress) = progress else { return };
    let (x, y) = (head.center().x - PROGRESS_WIDTH / 2.0, head.y - PROGRESS_ABOVE - PROGRESS_HEIGHT);
    draw_pill(x - 3.0, y - 3.0, PROGRESS_WIDTH + 6.0, PROGRESS_HEIGHT + 6.0, PROGRESS_BACK);
    if progress > 0.0 {
        draw_pill(x, y, (PROGRESS_WIDTH * progress.clamp(0.0, 1.0)).max(PROGRESS_HEIGHT), PROGRESS_HEIGHT, PROGRESS_FILL);
    }
}

/// 재료 판에 보일 것: 1개 이상인 재료 (정보판 순서). 0개는 안 보인다 (US-020).
pub fn bag_items(bag: &Bag) -> Vec<(Material, u32)> {
    Material::ALL.iter().map(|m| (*m, bag.count(*m))).filter(|(_, n)| *n > 0).collect()
}

/// 재료 판의 폭. 보일 것이 없으면 0 (판도 없다).
pub fn bag_width(text: &TextPainter, bag: &Bag) -> f32 {
    let items = bag_items(bag);
    if items.is_empty() && !bag.has_pickaxe {
        return 0.0;
    }
    let item_w: f32 = items.iter().map(|(_, n)| BAG_ICON + 4.0 + text.width(&n.to_string(), BAG_TEXT) + BAG_GAP).sum();
    BAG_PAD * 2.0 + item_w + if bag.has_pickaxe { BAG_ICON + BAG_GAP } else { 0.0 } - BAG_GAP
}

/// 정보판 아래 왼쪽: 가진 재료 개수와 곡괭이 (US-020, US-021). 화면 좌표.
/// `show_recipe`: 제작대가 있는 맵이면 곡괭이 없을 때 만드는 법 한 줄을 아래에 단다.
pub fn draw_bag(text: &TextPainter, bag: &Bag, show_recipe: bool) {
    if show_recipe && !bag.has_pickaxe {
        let line = recipe_line(bag);
        draw_pill(BAG_LEFT, RECIPE_TOP, text.width(&line, RECIPE_TEXT) + BAG_PAD * 2.0, RECIPE_HEIGHT, PANEL);
        text.left(&line, BAG_LEFT + BAG_PAD, RECIPE_TOP + RECIPE_HEIGHT - 8.0, RECIPE_TEXT, WHITE);
    }
    let w = bag_width(text, bag);
    if w <= 0.0 {
        return;
    }
    draw_pill(BAG_LEFT, BAG_TOP, w, BAG_HEIGHT, PANEL);
    let mid_y = BAG_TOP + BAG_HEIGHT / 2.0;
    let mut x = BAG_LEFT + BAG_PAD;
    for (material, n) in bag_items(bag) {
        draw_material_icon(material, vec2(x + BAG_ICON / 2.0, mid_y), BAG_ICON * 0.9);
        let label = n.to_string();
        text.left(&label, x + BAG_ICON + 4.0, mid_y + 10.0, BAG_TEXT, WHITE);
        x += BAG_ICON + 4.0 + text.width(&label, BAG_TEXT) + BAG_GAP;
    }
    if bag.has_pickaxe {
        draw_pickaxe(vec2(x + BAG_ICON / 2.0, mid_y), BAG_ICON, WHITE);
    }
}

/// 화면 쪽 M7 상태: 캔 것 팝업과 제작대에서 고른 재료.
pub struct GatherView {
    /// 막 캔 것과 보인 지 몇 초
    popup: Option<(Material, f32)>,
    /// 제작대에서 칸에 놓을 재료 (나무 또는 철광석)
    pub craft_pick: Material,
    /// 곡괭이 만드는 법 안내를 보인 지 몇 초. 판마다 처음 나무를 캤을 때 한 번만 켠다.
    recipe_hint: Option<f32>,
    recipe_told: bool,
}

impl GatherView {
    pub fn new() -> GatherView {
        GatherView { popup: None, craft_pick: Material::Wood, recipe_hint: None, recipe_told: false }
    }

    /// `can_craft`: 아직 곡괭이가 없고 제작대가 있는 맵이다.
    pub fn on_mined(&mut self, what: Material, can_craft: bool) {
        self.popup = Some((what, 0.0));
        if what == Material::Wood && can_craft && !self.recipe_told {
            self.recipe_told = true;
            self.recipe_hint = Some(0.0);
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.popup = self.popup.map(|(m, age)| (m, age + dt)).filter(|(_, age)| *age < POPUP_SECONDS);
        self.recipe_hint = self.recipe_hint.map(|age| age + dt).filter(|age| *age < RECIPE_HINT_SECONDS);
    }

    /// 처음 나무를 캤을 때의 곡괭이 안내 두 줄. 화면 좌표.
    fn draw_recipe_hint(&self, text: &TextPainter) {
        let Some(age) = self.recipe_hint else { return };
        let fade = ((RECIPE_HINT_SECONDS - age) / 0.4).clamp(0.0, 1.0);
        let lines = recipe_hint_lines();
        let w = lines.iter().map(|l| text.width(l, RECIPE_HINT_SIZE)).fold(0.0, f32::max) + 48.0;
        let x = VIRTUAL_WIDTH - BAG_LEFT - w;
        draw_pill(x, RECIPE_HINT_TOP, w, RECIPE_HINT_HEIGHT, Color { a: PANEL.a * 1.3 * fade, ..PANEL });
        for (i, line) in lines.iter().enumerate() {
            let baseline = RECIPE_HINT_TOP + 4.0 + RECIPE_HINT_LINE * (i + 1) as f32;
            text.centered(line, x + w / 2.0, baseline, RECIPE_HINT_SIZE, Color { a: fade, ..POPUP_TEXT });
        }
    }

    /// 안내가 보이는 동안은 왼쪽 만드는 법 줄을 숨긴다 (같은 말이고, 길면 안내 상자와 겹친다).
    pub fn recipe_hint_showing(&self) -> bool {
        self.recipe_hint.is_some()
    }

    #[cfg(test)]
    pub fn popup(&self) -> Option<Material> {
        self.popup.map(|(m, _)| m)
    }

    /// 캔 것 팝업 (US-065): 그림 + "철광석을 캤어요!". 금·은은 별이 더 돈다. 화면 좌표.
    pub fn draw_popup(&self, text: &TextPainter) {
        self.draw_recipe_hint(text);
        let Some((material, age)) = self.popup else { return };
        let fade = ((POPUP_SECONDS - age) / 0.3).clamp(0.0, 1.0);
        let words = material.mined_text();
        let w = POPUP_ICON + 16.0 + text.width(words, POPUP_TEXT_SIZE) + 48.0;
        let x = VIRTUAL_WIDTH / 2.0 - w / 2.0;
        draw_pill(x, POPUP_TOP, w, POPUP_HEIGHT, Color { a: PANEL.a * 1.3 * fade, ..PANEL });
        let mid_y = POPUP_TOP + POPUP_HEIGHT / 2.0;
        // 처음 0.2초 동안 그림이 톡 커진다
        let grow = 1.0 + 0.4 * (1.0 - (age / 0.2).min(1.0));
        draw_material_icon(material, vec2(x + 24.0 + POPUP_ICON / 2.0, mid_y), POPUP_ICON * grow);
        text.centered(words, x + 24.0 + POPUP_ICON + 16.0 + text.width(words, POPUP_TEXT_SIZE) / 2.0, mid_y + 14.0, POPUP_TEXT_SIZE, Color { a: fade, ..POPUP_TEXT });
        if sparkles(material) {
            for i in 0..6 {
                let a = age * 3.0 + i as f32 * std::f32::consts::TAU / 6.0;
                let p = vec2(VIRTUAL_WIDTH / 2.0 + a.cos() * (w / 2.0 + 6.0), mid_y + a.sin() * (POPUP_HEIGHT * 0.75));
                draw_star(p, 11.0, a, Color { a: fade, ..SPARKLE });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popup_goes_away_after_about_one_and_a_half_seconds() {
        let mut v = GatherView::new();
        v.on_mined(Material::Gold, true);
        v.update(POPUP_SECONDS - 0.1);
        assert_eq!(v.popup(), Some(Material::Gold));
        v.update(0.2);
        assert_eq!(v.popup(), None);
        v.on_mined(Material::Wood, true);
        assert_eq!(v.popup(), Some(Material::Wood), "새로 캐면 다시 나온다");
    }

    #[test]
    fn pickaxe_recipe_counts_shape_and_hint_shows_once_on_first_wood() {
        assert_eq!(pickaxe_needs(), [(Material::Wood, 4), (Material::Iron, 2)]);
        let mut bag = Bag::default();
        bag.add(Material::Wood);
        assert_eq!(recipe_line(&bag), "곡괭이: 나무 1/4 + 철광석 0/2 (집 제작대)");
        let mut v = GatherView::new();
        v.on_mined(Material::Iron, true);
        assert!(v.recipe_hint.is_none(), "철은 아니다");
        v.on_mined(Material::Wood, false);
        assert!(v.recipe_hint.is_none(), "곡괭이가 있으면 안 나온다");
        v.on_mined(Material::Wood, true);
        assert!(v.recipe_hint.is_some());
        v.update(RECIPE_HINT_SECONDS + 0.1);
        v.on_mined(Material::Wood, true);
        assert!(v.recipe_hint.is_none(), "한 번만");
        // 만드는 법 줄은 재료 판 아래, 팝업 위
        assert!(RECIPE_TOP >= BAG_TOP + BAG_HEIGHT && RECIPE_TOP + RECIPE_HEIGHT < POPUP_TOP);
        assert_eq!(recipe_hint_lines()[0], "나무 4개와 철광석 2개로 곡괭이를 만들 수 있어요!");
    }

    #[test]
    fn recipe_hint_box_stays_above_popup_buttons_and_character() {
        let bottom = RECIPE_HINT_TOP + RECIPE_HINT_HEIGHT;
        assert!(RECIPE_HINT_TOP >= super::super::touch::BACK_BUTTON.y + super::super::touch::BACK_BUTTON.h, "정보판 아래");
        assert!(bottom < POPUP_TOP, "팝업과 안 겹친다");
        assert!(bottom < crate::config::GROUND_Y - crate::config::PLAYER_HEIGHT, "서 있는 캐릭터와 안 겹친다");
        for b in super::super::touch::play_buttons(true) {
            assert!(bottom < b.center.y - b.radius, "{:?} 버튼과 안 겹친다", b.kind);
        }
    }

    #[test]
    fn bag_shows_only_what_we_have_in_order() {
        let mut bag = Bag::default();
        assert!(bag_items(&bag).is_empty());
        bag.add(Material::Gold);
        bag.add(Material::Wood);
        bag.add(Material::Wood);
        assert_eq!(bag_items(&bag), vec![(Material::Wood, 2), (Material::Gold, 1)]);
    }

    #[test]
    fn ore_colors_differ_and_gold_silver_sparkle() {
        let ores = [Material::Iron, Material::Copper, Material::Silver, Material::Gold];
        for (i, a) in ores.iter().enumerate() {
            for b in &ores[i + 1..] {
                assert_ne!(material_color(*a), material_color(*b), "{a:?} {b:?}");
            }
        }
        assert!(sparkles(Material::Gold) && sparkles(Material::Silver));
        assert!(!sparkles(Material::Iron) && !sparkles(Material::Wood));
    }

    #[test]
    fn usable_glow_breathes_and_tag_bounces_gently() {
        // 반짝임은 0..=1 안에서 천천히 커졌다 작아지고, 표시는 바닥 아래로 꺼지지 않고 TAG_BOB까지만 튄다.
        use super::super::touch::GLOW_SECONDS;
        assert!(glow_pulse(0.0) < 0.01 && (glow_pulse(GLOW_SECONDS / 2.0) - 1.0).abs() < 0.01);
        for i in 0..200 {
            let t = i as f32 * 0.037;
            assert!((0.0..=1.0).contains(&glow_pulse(t)), "반짝임 {t}");
            assert!((0.0..=TAG_BOB).contains(&tag_bounce(t)), "튀기 {t}");
        }
        assert!(GLOW_SECONDS >= 1.0, "너무 빨리 깜빡이지 않는다");
    }

    #[test]
    fn popup_and_bag_stay_clear_of_hud_and_touch_buttons() {
        // 체력 판(y 74~122)과 "◀ 고르기"(y 78~138) 아래, 조이스틱 안내(위 끝 315)와 버튼(위 끝 296)보다 위
        assert!(BAG_TOP >= super::super::touch::BACK_BUTTON.y + super::super::touch::BACK_BUTTON.h);
        assert!(BAG_TOP + BAG_HEIGHT < POPUP_TOP);
        assert!(POPUP_TOP + POPUP_HEIGHT < 290.0);
    }
}
