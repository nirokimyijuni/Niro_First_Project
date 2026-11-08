//! 제작대 화면 (US-021, 로건): 재료 고르기 버튼 2개, 4×3 칸, 결과 칸, "닫기".
//! 마우스 클릭과 손가락 탭이 같은 `craft_tap_at` → `apply_craft_tap` 한 길로 간다.
//! 놓기·빼기·곡괭이 모양 판정은 logic(`Crafting`)이 하고, 여기는 어디를 눌렀는지와 그리기만 한다.

use super::gather::{draw_material_icon, draw_pickaxe, pickaxe_needs};
use super::hud::draw_pill;
use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::text::TextPainter;
use crate::config::{CRAFT_GRID_HEIGHT, CRAFT_GRID_WIDTH, CRAFT_IRON_MARK, CRAFT_WOOD_MARK, PICKAXE_SHAPE};
use crate::logic::game::Game;
use crate::logic::gathering::{Bag, Material};
use macroquad::prelude::*;

/// 화면 판 (가상 960×540 안). 크게 눌러도 되게 칸이 크다.
const PANEL: Rect = Rect::new(90.0, 70.0, 780.0, 440.0);
const TITLE_Y: f32 = 122.0;
/// 칸 하나 크기와 칸 사이
pub const CELL: f32 = 86.0;
const CELL_GAP: f32 = 6.0;
const GRID_LEFT: f32 = 300.0;
const GRID_TOP: f32 = 160.0;
/// 왼쪽 재료 고르기 버튼 (나무, 철광석)
const CHOICES: [Material; 2] = [Material::Wood, Material::Iron];
const CHOICE_LEFT: f32 = 115.0;
const CHOICE_TOP: f32 = 170.0;
const CHOICE_W: f32 = 165.0;
const CHOICE_H: f32 = 108.0;
const CHOICE_GAP: f32 = 22.0;
/// 결과 칸과 닫기
pub const RESULT: Rect = Rect::new(715.0, 190.0, 120.0, 120.0);
pub const CLOSE: Rect = Rect::new(705.0, 400.0, 140.0, 72.0);

const DIM: Color = Color::new(0.0, 0.0, 0.0, 0.35);
const PANEL_FILL: Color = Color::new(0.45, 0.30, 0.18, 0.96);
const CELL_FILL: Color = Color::new(0.92, 0.78, 0.56, 1.0);
const CELL_LINE: Color = Color::new(0.30, 0.18, 0.08, 1.0);
const PICKED: Color = Color::new(1.0, 0.86, 0.22, 1.0);
const BUTTON_FILL: Color = Color::new(0.10, 0.13, 0.32, 0.7);
const RESULT_GLOW: Color = Color::new(1.0, 0.95, 0.55, 1.0);
const ARROW: Color = Color::new(1.0, 1.0, 1.0, 0.85);
/// 흐린 곡괭이 모양 힌트: 아이콘 위에 칸 색을 이만큼 덮는다
const HINT_FADE: f32 = 0.6;
const STEP_DONE: Color = Color::new(1.0, 1.0, 1.0, 0.75);

// ---- 순서 안내 (부모님 요청: 처음 보는 사람도 알게) ----
/// 칸 아래, "닫기" 왼쪽에 두 줄. 지금 할 차례만 노랗게.
const STEPS: [&str; 3] = ["1. 나무나 철광석을 골라요", "2. 흐린 칸을 눌러 채워요", "3. 곡괭이를 눌러 가져가요"]; // Jua 글꼴에 ①·→가 없다
const STEP_SIZE: f32 = 26.0;
const STEP_LINE_1: f32 = 468.0;
const STEP_LINE_2: f32 = 500.0;
const STEP_RIGHT: f32 = 690.0;
const SHORT_SIZE: f32 = 26.0;
/// 곡괭이를 가져가면 나오는 효과 글
pub const PICKAXE_MADE_TEXT: &str = "곡괭이를 만들었어요!";

/// 제작대 화면에서 누른 곳.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CraftTap {
    Choose(Material),
    Cell { row: usize, column: usize },
    Result,
    Close,
}

pub fn cell_rect(row: usize, column: usize) -> Rect {
    Rect::new(GRID_LEFT + column as f32 * (CELL + CELL_GAP), GRID_TOP + row as f32 * (CELL + CELL_GAP), CELL, CELL)
}

fn choice_rect(index: usize) -> Rect {
    Rect::new(CHOICE_LEFT, CHOICE_TOP + index as f32 * (CHOICE_H + CHOICE_GAP), CHOICE_W, CHOICE_H)
}

/// 누른 곳(가상 좌표)이 무엇인지. 판 밖이나 빈 곳은 None.
pub fn craft_tap_at(point: Vec2) -> Option<CraftTap> {
    if CLOSE.contains(point) {
        return Some(CraftTap::Close);
    }
    if RESULT.contains(point) {
        return Some(CraftTap::Result);
    }
    if let Some(i) = (0..CHOICES.len()).find(|i| choice_rect(*i).contains(point)) {
        return Some(CraftTap::Choose(CHOICES[i]));
    }
    (0..CRAFT_GRID_HEIGHT)
        .flat_map(|row| (0..CRAFT_GRID_WIDTH).map(move |column| (row, column)))
        .find(|(row, column)| cell_rect(*row, *column).contains(point))
        .map(|(row, column)| CraftTap::Cell { row, column })
}

/// 누른 것을 logic에 넘긴다. `pick`은 고른 재료 (화면 상태). 곡괭이를 가져갔으면 true.
pub fn apply_craft_tap(game: &mut Game, pick: &mut Material, tap: CraftTap) -> bool {
    match tap {
        CraftTap::Choose(material) => *pick = material,
        CraftTap::Cell { row, column } => game.crafting.tap_cell(row, column, *pick, &game.gathering.bag),
        CraftTap::Result => return game.crafting.take_pickaxe(&mut game.gathering.bag),
        CraftTap::Close => game.crafting.close(),
    }
    false
}

/// 받침이 있으면 "이", 없으면 "가".
fn subject_particle(word: &str) -> &'static str {
    match word.chars().last() {
        Some(c @ '가'..='힣') if (c as u32 - '가' as u32) % 28 != 0 => "이",
        _ => "가",
    }
}

/// 곡괭이에 모자란 재료 글. 칸에 놓은 것도 아직 가방에 있으니 가방 개수로만 센다.
/// 예: "나무가 2개 더 필요해요", "나무 2개, 철광석 1개가 더 필요해요". 넉넉하면 None.
fn shortage_text(bag: &Bag) -> Option<String> {
    let short: Vec<(Material, u32)> = pickaxe_needs().iter().map(|(m, n)| (*m, n.saturating_sub(bag.count(*m)))).filter(|(_, k)| *k > 0).collect();
    match short.as_slice() {
        [] => None,
        [(m, k)] => Some(format!("{}{} {}개 더 필요해요", m.name(), subject_particle(m.name()), k)),
        _ => {
            let parts: Vec<String> = short.iter().map(|(m, k)| format!("{} {}개", m.name(), k)).collect();
            Some(format!("{}가 더 필요해요", parts.join(", ")))
        }
    }
}

/// 지금 할 차례 (0: 재료 고르기, 1: 칸 채우기, 2: 곡괭이 가져가기).
fn current_step(game: &Game) -> usize {
    if game.crafting.pickaxe_is_ready(&game.gathering.bag) {
        2
    } else if game.crafting.grid.iter().flatten().any(Option::is_some) {
        1
    } else {
        0
    }
}

/// 제작대 화면 전체. 화면 좌표.
pub fn draw_craft(text: &TextPainter, game: &Game, pick: Material) {
    let bag = &game.gathering.bag;
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, DIM);
    draw_rectangle(PANEL.x, PANEL.y, PANEL.w, PANEL.h, PANEL_FILL);
    draw_rectangle_lines(PANEL.x, PANEL.y, PANEL.w, PANEL.h, 6.0, CELL_LINE);
    text.left("제작대", CHOICE_LEFT, TITLE_Y, 44.0, PICKED);
    // 결과 칸을 누른 순간 제작대 안에서 축하 글 (창이 열린 동안 효과 시간은 멈춰 있다)
    let top_right = if bag.has_pickaxe { Some(PICKAXE_MADE_TEXT.to_string()) } else { shortage_text(bag) };
    if let Some(words) = top_right {
        let right = PANEL.right() - 25.0;
        text.left(&words, right - text.width(&words, SHORT_SIZE), TITLE_Y - 6.0, SHORT_SIZE, PICKED);
    }
    // 순서 안내: 첫 줄에 이어 쓰다가 "닫기"에 닿으면 둘째 줄로
    let step = current_step(game);
    let (mut x, mut y) = (CHOICE_LEFT, STEP_LINE_1);
    for (i, words) in STEPS.iter().enumerate() {
        let w = text.width(words, STEP_SIZE);
        if x + w > STEP_RIGHT && x > CHOICE_LEFT {
            (x, y) = (CHOICE_LEFT, STEP_LINE_2);
        }
        text.left(words, x, y, STEP_SIZE, if i == step { PICKED } else { STEP_DONE });
        x += w + 12.0;
    }

    // 재료 고르기: 그림 + 이름 + 남은 개수 (칸에 놓은 것은 아직 빼지 않는다)
    for (i, material) in CHOICES.iter().enumerate() {
        let r = choice_rect(i);
        draw_rectangle(r.x, r.y, r.w, r.h, BUTTON_FILL);
        if *material == pick {
            draw_rectangle_lines(r.x - 3.0, r.y - 3.0, r.w + 6.0, r.h + 6.0, 7.0, PICKED);
        }
        draw_material_icon(*material, vec2(r.x + 40.0, r.y + r.h / 2.0 - 4.0), 50.0);
        text.left(material.name(), r.x + 74.0, r.y + 46.0, 30.0, WHITE);
        text.left(&format!("{}개", bag.count(*material)), r.x + 74.0, r.y + 86.0, 30.0, WHITE);
    }

    let ready = game.crafting.pickaxe_is_ready(bag);
    for row in 0..CRAFT_GRID_HEIGHT {
        for column in 0..CRAFT_GRID_WIDTH {
            let r = cell_rect(row, column);
            draw_rectangle(r.x, r.y, r.w, r.h, CELL_FILL);
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 4.0, CELL_LINE);
            if let Some(material) = game.crafting.grid[row][column] {
                draw_material_icon(material, r.center(), CELL * 0.62);
            } else if !ready {
                // 아직 안 놓은 칸: 곡괭이 모양 힌트를 흐리게
                let hint = match PICKAXE_SHAPE[row][column] {
                    m if m == CRAFT_WOOD_MARK => Some(Material::Wood),
                    m if m == CRAFT_IRON_MARK => Some(Material::Iron),
                    _ => None,
                };
                if let Some(material) = hint {
                    draw_material_icon(material, r.center(), CELL * 0.62);
                    draw_rectangle(r.x + 2.0, r.y + 2.0, r.w - 4.0, r.h - 4.0, Color { a: HINT_FADE, ..CELL_FILL });
                }
            }
        }
    }

    // 화살표 → 결과 칸. 곡괭이가 나오면 빛나고 "눌러요"
    let arrow_y = RESULT.center().y;
    let grid_right = cell_rect(0, CRAFT_GRID_WIDTH - 1).right();
    draw_line(grid_right + 10.0, arrow_y, RESULT.x - 16.0, arrow_y, 8.0, ARROW);
    draw_triangle(vec2(RESULT.x - 4.0, arrow_y), vec2(RESULT.x - 20.0, arrow_y - 14.0), vec2(RESULT.x - 20.0, arrow_y + 14.0), ARROW);
    if ready {
        let pulse = 6.0 + 3.0 * (get_time() as f32 * 6.0).sin();
        draw_rectangle(RESULT.x - pulse, RESULT.y - pulse, RESULT.w + pulse * 2.0, RESULT.h + pulse * 2.0, RESULT_GLOW);
    }
    draw_rectangle(RESULT.x, RESULT.y, RESULT.w, RESULT.h, CELL_FILL);
    draw_rectangle_lines(RESULT.x, RESULT.y, RESULT.w, RESULT.h, 5.0, CELL_LINE);
    if ready {
        draw_pickaxe(RESULT.center(), RESULT.w * 0.75, WHITE);
        text.centered("눌러요!", RESULT.center().x, RESULT.bottom() + 40.0, 30.0, PICKED);
    } else if bag.has_pickaxe {
        text.centered("곡괭이 있음", RESULT.center().x, RESULT.bottom() + 40.0, 26.0, WHITE);
    }

    draw_pill(CLOSE.x, CLOSE.y, CLOSE.w, CLOSE.h, BUTTON_FILL);
    text.centered("닫기", CLOSE.center().x, CLOSE.center().y + 14.0, 40.0, WHITE);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PICKAXE_SHAPE;
    use crate::logic::character::CharacterKind;
    use crate::logic::world::World;

    fn all_targets() -> Vec<Rect> {
        let mut list = vec![RESULT, CLOSE, choice_rect(0), choice_rect(1)];
        list.extend((0..CRAFT_GRID_HEIGHT).flat_map(|row| (0..CRAFT_GRID_WIDTH).map(move |column| cell_rect(row, column))));
        list
    }

    #[test]
    fn everything_fits_in_the_panel_without_overlap_and_is_big() {
        let targets = all_targets();
        for (i, a) in targets.iter().enumerate() {
            assert!(PANEL.contains(a.point()) && PANEL.contains(vec2(a.right(), a.bottom())), "{a:?} 판 밖");
            // 740×360 화면은 0.667배라서 가장 작은 칸도 약 57 화면 픽셀이다
            assert!(a.w.min(a.h) >= 72.0, "{a:?} 작다");
            for b in &targets[i + 1..] {
                assert!(a.intersect(*b).is_none(), "{a:?} {b:?} 겹친다");
            }
        }
    }

    #[test]
    fn guide_says_what_is_missing_and_which_step_is_next() {
        let mut game = Game::on_world(CharacterKind::Niro, 7, World::m1());
        assert_eq!(shortage_text(&game.gathering.bag).as_deref(), Some("나무 4개, 철광석 2개가 더 필요해요"));
        for _ in 0..4 {
            game.gathering.bag.add(Material::Wood);
        }
        assert_eq!(shortage_text(&game.gathering.bag).as_deref(), Some("철광석이 2개 더 필요해요"));
        game.gathering.bag.add(Material::Iron);
        game.gathering.bag.add(Material::Iron);
        game.gathering.bag.add(Material::Wood);
        assert_eq!(shortage_text(&game.gathering.bag), None, "넉넉하면 말하지 않는다");
        assert_eq!(subject_particle("나무"), "가");
        game.crafting.open();
        assert_eq!(current_step(&game), 0);
        let mut pick = Material::Wood;
        apply_craft_tap(&mut game, &mut pick, CraftTap::Cell { row: 0, column: 1 });
        assert_eq!(current_step(&game), 1);
        // 안내 두 줄은 칸 아래, 판 안, "닫기" 왼쪽
        let grid_bottom = cell_rect(CRAFT_GRID_HEIGHT - 1, 0).bottom();
        assert!(STEP_LINE_1 - STEP_SIZE > grid_bottom && STEP_LINE_2 + 6.0 < PANEL.bottom());
        assert!(STEP_RIGHT < CLOSE.x);
    }

    #[test]
    fn taps_find_the_right_target() {
        assert_eq!(craft_tap_at(CLOSE.center()), Some(CraftTap::Close));
        assert_eq!(craft_tap_at(RESULT.center()), Some(CraftTap::Result));
        assert_eq!(craft_tap_at(choice_rect(1).center()), Some(CraftTap::Choose(Material::Iron)));
        assert_eq!(craft_tap_at(cell_rect(2, 3).center()), Some(CraftTap::Cell { row: 2, column: 3 }));
        assert_eq!(craft_tap_at(vec2(5.0, 5.0)), None);
    }

    #[test]
    fn tapping_the_pickaxe_shape_makes_a_pickaxe() {
        let mut game = Game::on_world(CharacterKind::Niro, 7, World::m1());
        for _ in 0..4 {
            game.gathering.bag.add(Material::Wood);
        }
        for _ in 0..2 {
            game.gathering.bag.add(Material::Iron);
        }
        game.crafting.open();
        let mut pick = Material::Wood;
        for (row, line) in PICKAXE_SHAPE.iter().enumerate() {
            for (column, mark) in line.iter().enumerate() {
                let material = match mark {
                    '나' => Material::Wood,
                    '철' => Material::Iron,
                    _ => continue,
                };
                let choice = CHOICES.iter().position(|m| *m == material).unwrap();
                apply_craft_tap(&mut game, &mut pick, craft_tap_at(choice_rect(choice).center()).unwrap());
                apply_craft_tap(&mut game, &mut pick, craft_tap_at(cell_rect(row, column).center()).unwrap());
            }
        }
        assert!(game.crafting.pickaxe_is_ready(&game.gathering.bag));
        apply_craft_tap(&mut game, &mut pick, CraftTap::Result);
        assert!(game.gathering.bag.has_pickaxe);
        assert_eq!(game.gathering.bag.count(Material::Wood), 0);
        apply_craft_tap(&mut game, &mut pick, CraftTap::Close);
        assert!(!game.crafting.is_open());
    }
}
