//! 제작대 화면 (US-021, US-078, 로건): 위쪽 "만들 것" 그림 버튼 5개, 왼쪽 재료 버튼 7개, 4×3 칸, 결과 칸, "닫기".
//! 마우스 클릭과 손가락 탭이 같은 `craft_tap_at` → `apply_craft_tap` 한 길로 간다.
//! 고르기·놓기·빼기·조합 판정·모자란 재료 글은 logic(`Crafting`, `Recipe`)이 하고, 여기는 어디를 눌렀는지와 그리기만 한다.

use super::assets::M6Art;
use super::gather::draw_material_icon;
use super::hud::draw_pill;
use super::items::draw_recipe_icon;
use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::text::TextPainter;
use crate::config::{CRAFT_GRID_HEIGHT, CRAFT_GRID_WIDTH, CRAFT_STEPS_TEXT};
use crate::logic::crafting::Recipe;
use crate::logic::game::Game;
use crate::logic::gathering::Material;
use macroquad::prelude::*;

/// 화면 판 (가상 960×540 안). 크게 눌러도 되게 칸이 크다.
const PANEL: Rect = Rect::new(20.0, 10.0, 920.0, 520.0);
const TITLE_Y: f32 = 62.0;
const TITLE_LEFT: f32 = 36.0;

/// 위쪽 "만들 것" 그림 버튼 (`Recipe::ALL` 순서)
const RECIPE_LEFT: f32 = 170.0;
const RECIPE_TOP: f32 = 22.0;
const RECIPE_W: f32 = 104.0;
const RECIPE_H: f32 = 86.0;
const RECIPE_GAP: f32 = 10.0;
/// 고른 것 이름과 상태 글 줄
const INFO_Y: f32 = 146.0;

/// 칸 하나 크기와 칸 사이
pub const CELL: f32 = 80.0;
const CELL_GAP: f32 = 6.0;
const GRID_LEFT: f32 = 344.0;
const GRID_TOP: f32 = 168.0;

/// 왼쪽 재료 버튼 (금은 조합에 안 쓰니 없다): 두 칸씩 네 줄
pub const CRAFT_MATERIALS: [Material; 7] =
    [Material::Wood, Material::Iron, Material::Copper, Material::Silver, Material::Emerald, Material::Ruby, Material::Diamond];
const CHOICE_LEFT: f32 = 34.0;
const CHOICE_TOP: f32 = 168.0;
const CHOICE_W: f32 = 140.0;
const CHOICE_H: f32 = 74.0;
const CHOICE_GAP: f32 = 8.0;

/// 결과 칸과 닫기
pub const RESULT: Rect = Rect::new(764.0, 210.0, 120.0, 120.0);
pub const CLOSE: Rect = Rect::new(780.0, 26.0, 140.0, 76.0);

const DIM: Color = Color::new(0.0, 0.0, 0.0, 0.35);
const PANEL_FILL: Color = Color::new(0.45, 0.30, 0.18, 0.96);
const CELL_FILL: Color = Color::new(0.92, 0.78, 0.56, 1.0);
const CELL_LINE: Color = Color::new(0.30, 0.18, 0.08, 1.0);
const PICKED: Color = Color::new(1.0, 0.86, 0.22, 1.0);
const BUTTON_FILL: Color = Color::new(0.10, 0.13, 0.32, 0.7);
const EMPTY_FILL: Color = Color::new(0.10, 0.13, 0.32, 0.35);
const RESULT_GLOW: Color = Color::new(1.0, 0.95, 0.55, 1.0);
const ARROW: Color = Color::new(1.0, 1.0, 1.0, 0.85);
const NEED: Color = Color::new(1.0, 0.75, 0.45, 1.0);
/// 흐린 칸 힌트: 아이콘 위에 칸 색을 이만큼 덮는다
const HINT_FADE: f32 = 0.6;
const STEP_DONE: Color = Color::new(1.0, 1.0, 1.0, 0.75);

// ---- 순서 안내 (부모님 요청: 처음 보는 사람도 알게) ----
const STEP_SIZE: f32 = 24.0;
const STEP_LINE_1: f32 = 476.0;
const STEP_LINE_2: f32 = 506.0;
const STEP_LEFT: f32 = GRID_LEFT;
const STEP_RIGHT: f32 = 920.0;
const INFO_SIZE: f32 = 28.0;

/// 제작대 화면에서 누른 곳.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CraftTap {
    Recipe(Recipe),
    Choose(Material),
    Cell { row: usize, column: usize },
    Result,
    Close,
}

pub fn cell_rect(row: usize, column: usize) -> Rect {
    Rect::new(GRID_LEFT + column as f32 * (CELL + CELL_GAP), GRID_TOP + row as f32 * (CELL + CELL_GAP), CELL, CELL)
}

fn choice_rect(index: usize) -> Rect {
    let (row, column) = (index / 2, index % 2);
    Rect::new(CHOICE_LEFT + column as f32 * (CHOICE_W + CHOICE_GAP), CHOICE_TOP + row as f32 * (CHOICE_H + CHOICE_GAP), CHOICE_W, CHOICE_H)
}

fn recipe_rect(index: usize) -> Rect {
    Rect::new(RECIPE_LEFT + index as f32 * (RECIPE_W + RECIPE_GAP), RECIPE_TOP, RECIPE_W, RECIPE_H)
}

/// 순서 안내 네 마디. config 글의 ①~④와 →는 Jua 글꼴에 없어서 "1." 같은 글자로 바꾼다.
fn step_phrases() -> Vec<String> {
    CRAFT_STEPS_TEXT
        .split('→')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .enumerate()
        .map(|(i, s)| {
            let words = s.trim_start_matches(|c: char| ('①'..='⑳').contains(&c)).trim();
            format!("{}. {}", i + 1, words)
        })
        .collect()
}

/// 누른 곳(가상 좌표)이 무엇인지. 판 밖이나 빈 곳은 None.
pub fn craft_tap_at(point: Vec2) -> Option<CraftTap> {
    if CLOSE.contains(point) {
        return Some(CraftTap::Close);
    }
    if RESULT.contains(point) {
        return Some(CraftTap::Result);
    }
    if let Some(i) = (0..Recipe::ALL.len()).find(|i| recipe_rect(*i).contains(point)) {
        return Some(CraftTap::Recipe(Recipe::ALL[i]));
    }
    if let Some(i) = (0..CRAFT_MATERIALS.len()).find(|i| choice_rect(*i).contains(point)) {
        return Some(CraftTap::Choose(CRAFT_MATERIALS[i]));
    }
    (0..CRAFT_GRID_HEIGHT)
        .flat_map(|row| (0..CRAFT_GRID_WIDTH).map(move |column| (row, column)))
        .find(|(row, column)| cell_rect(*row, *column).contains(point))
        .map(|(row, column)| CraftTap::Cell { row, column })
}

/// 화면 쪽 제작대 상태: 칸에 놓을 재료와 막 꺼낸 것 (축하 글).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CraftView {
    pub pick: Material,
    pub made: Option<Recipe>,
}

impl CraftView {
    pub fn new() -> CraftView {
        CraftView { pick: Material::Wood, made: None }
    }

    /// 제작대를 열 때: 골라져 있는 것(닫았다 열어도 남는다)에 드는 첫 재료를 골라 둔다.
    pub fn opened(chosen: Recipe) -> CraftView {
        CraftView { pick: chosen.needs().first().map_or(Material::Wood, |(m, _)| *m), made: None }
    }
}

/// 누른 것을 logic에 넘긴다. 결과 칸에서 꺼냈으면 그것을 돌려준다.
/// 만들 것을 고르면 그것에 드는 첫 재료가 바로 골라진다 (아이가 덜 헤매게).
pub fn apply_craft_tap(game: &mut Game, view: &mut CraftView, tap: CraftTap) -> Option<Recipe> {
    match tap {
        CraftTap::Recipe(recipe) => {
            game.crafting.choose(recipe);
            if let Some((first, _)) = recipe.needs().first() {
                view.pick = *first;
            }
            view.made = None;
        }
        CraftTap::Choose(material) => view.pick = material,
        CraftTap::Cell { row, column } => {
            game.crafting.tap_cell(row, column, view.pick, &game.gathering.bag);
            view.made = None;
        }
        CraftTap::Result => {
            let made = game.crafting.take_result(&mut game.gathering.bag);
            if made.is_some() {
                view.made = made;
            }
            return made;
        }
        CraftTap::Close => {
            game.crafting.close();
            view.made = None;
        }
    }
    None
}

/// 지금 할 차례 (0: 만들 것 고르기, 1: 재료 고르기, 2: 칸 채우기, 3: 결과 가져가기).
fn current_step(game: &Game) -> usize {
    if game.crafting.ready_recipe(&game.gathering.bag).is_some() {
        3
    } else if game.crafting.grid.iter().flatten().any(Option::is_some) {
        2
    } else {
        1
    }
}

/// 위쪽 상태 글: 막 만들었으면 축하, 아니면 고른 것을 왜 못 만드는지 (재료 모자람, 벌써 가짐). 할 말이 없으면 None.
fn info_text(game: &Game, view: &CraftView) -> Option<String> {
    if let Some(made) = view.made {
        return Some(made.crafted_text().to_string());
    }
    game.crafting.chosen().why_cant_make_text(&game.gathering.bag)
}

/// 글이 `max_w` 안에 들어가는 크기.
fn fit(text: &TextPainter, words: &str, size: f32, max_w: f32) -> f32 {
    let w = text.width(words, size).max(1.0);
    size * (max_w / w).min(1.0)
}

/// 제작대 화면 전체. 화면 좌표.
pub fn draw_craft(text: &TextPainter, game: &Game, view: &CraftView, art: &M6Art) {
    let bag = &game.gathering.bag;
    let chosen = game.crafting.chosen();
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, DIM);
    draw_rectangle(PANEL.x, PANEL.y, PANEL.w, PANEL.h, PANEL_FILL);
    draw_rectangle_lines(PANEL.x, PANEL.y, PANEL.w, PANEL.h, 6.0, CELL_LINE);
    text.left("제작대", TITLE_LEFT, TITLE_Y, 40.0, PICKED);

    // 1. 만들 것 그림 버튼: 고른 것은 노란 테두리
    for (i, recipe) in Recipe::ALL.iter().enumerate() {
        let r = recipe_rect(i);
        draw_rectangle(r.x, r.y, r.w, r.h, BUTTON_FILL);
        if *recipe == chosen {
            draw_rectangle_lines(r.x - 3.0, r.y - 3.0, r.w + 6.0, r.h + 6.0, 7.0, PICKED);
        }
        draw_recipe_icon(*recipe, art, r.center(), r.h * 0.78);
        if recipe.already_have(bag) {
            // 벌써 가진 것: 작은 체크
            let c = vec2(r.right() - 16.0, r.y + 16.0);
            draw_circle(c.x, c.y, 12.0, Color::new(0.35, 0.80, 0.35, 1.0));
            draw_line(c.x - 6.0, c.y, c.x - 1.0, c.y + 5.0, 3.0, WHITE);
            draw_line(c.x - 1.0, c.y + 5.0, c.x + 7.0, c.y - 5.0, 3.0, WHITE);
        }
    }
    // 고른 것 이름 + 상태 글 (모자란 재료, 벌써 가짐, 만들었어요!)
    let name = format!("만들 것: {}", chosen.name());
    text.left(&name, CHOICE_LEFT, INFO_Y, fit(text, &name, INFO_SIZE, 410.0), WHITE);
    if let Some(words) = info_text(game, view) {
        let color = if view.made.is_some() { PICKED } else { NEED };
        let size = fit(text, &words, INFO_SIZE, PANEL.right() - 470.0);
        text.left(&words, PANEL.right() - 20.0 - text.width(&words, size), INFO_Y, size, color);
    }

    // 2. 재료 고르기: 그림 + 이름 + 가진 개수 (칸에 놓은 것은 아직 빼지 않는다)
    for (i, material) in CRAFT_MATERIALS.iter().enumerate() {
        let r = choice_rect(i);
        let have = bag.count(*material);
        draw_rectangle(r.x, r.y, r.w, r.h, if have > 0 { BUTTON_FILL } else { EMPTY_FILL });
        if *material == view.pick {
            draw_rectangle_lines(r.x - 3.0, r.y - 3.0, r.w + 6.0, r.h + 6.0, 6.0, PICKED);
        }
        draw_material_icon(*material, vec2(r.x + 28.0, r.center().y), 40.0);
        let label = material.name();
        text.left(label, r.x + 54.0, r.y + 32.0, fit(text, label, 24.0, r.w - 60.0), WHITE);
        text.left(&format!("{have}개"), r.x + 54.0, r.y + 64.0, 26.0, if have > 0 { WHITE } else { STEP_DONE });
    }

    // 3. 칸: 놓인 재료, 아니면 고른 것 모양을 흐리게
    let ready = game.crafting.ready_recipe(bag);
    for row in 0..CRAFT_GRID_HEIGHT {
        for column in 0..CRAFT_GRID_WIDTH {
            let r = cell_rect(row, column);
            draw_rectangle(r.x, r.y, r.w, r.h, CELL_FILL);
            draw_rectangle_lines(r.x, r.y, r.w, r.h, 4.0, CELL_LINE);
            if let Some(material) = game.crafting.grid[row][column] {
                draw_material_icon(material, r.center(), CELL * 0.62);
            } else if ready.is_none() {
                if let Some(material) = game.crafting.hint_at(row, column) {
                    draw_material_icon(material, r.center(), CELL * 0.62);
                    draw_rectangle(r.x + 2.0, r.y + 2.0, r.w - 4.0, r.h - 4.0, Color { a: HINT_FADE, ..CELL_FILL });
                }
            }
        }
    }

    // 4. 화살표 → 결과 칸. 무엇이 나오면 빛나고 "눌러요!"
    let arrow_y = RESULT.center().y;
    let grid_right = cell_rect(0, CRAFT_GRID_WIDTH - 1).right();
    draw_line(grid_right + 8.0, arrow_y, RESULT.x - 14.0, arrow_y, 8.0, ARROW);
    draw_triangle(vec2(RESULT.x - 2.0, arrow_y), vec2(RESULT.x - 18.0, arrow_y - 14.0), vec2(RESULT.x - 18.0, arrow_y + 14.0), ARROW);
    if ready.is_some() {
        let pulse = 6.0 + 3.0 * (get_time() as f32 * 6.0).sin();
        draw_rectangle(RESULT.x - pulse, RESULT.y - pulse, RESULT.w + pulse * 2.0, RESULT.h + pulse * 2.0, RESULT_GLOW);
    }
    draw_rectangle(RESULT.x, RESULT.y, RESULT.w, RESULT.h, CELL_FILL);
    draw_rectangle_lines(RESULT.x, RESULT.y, RESULT.w, RESULT.h, 5.0, CELL_LINE);
    if let Some(recipe) = ready {
        draw_recipe_icon(recipe, art, RESULT.center(), RESULT.w * 0.8);
        text.centered("눌러요!", RESULT.center().x, RESULT.bottom() + 40.0, 30.0, PICKED);
    }

    // 순서 안내: 칸 아래 두 줄. 지금 할 차례만 노랗게
    let step = current_step(game);
    let (mut x, mut y) = (STEP_LEFT, STEP_LINE_1);
    for (i, words) in step_phrases().iter().enumerate() {
        let w = text.width(words, STEP_SIZE);
        if x + w > STEP_RIGHT && x > STEP_LEFT {
            (x, y) = (STEP_LEFT, STEP_LINE_2);
        }
        text.left(words, x, y, STEP_SIZE, if i == step { PICKED } else { STEP_DONE });
        x += w + 16.0;
    }

    draw_pill(CLOSE.x, CLOSE.y, CLOSE.w, CLOSE.h, BUTTON_FILL);
    text.centered("닫기", CLOSE.center().x, CLOSE.center().y + 14.0, 40.0, WHITE);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::character::CharacterKind;
    use crate::logic::world::World;

    fn all_targets() -> Vec<Rect> {
        let mut list = vec![RESULT, CLOSE];
        list.extend((0..Recipe::ALL.len()).map(recipe_rect));
        list.extend((0..CRAFT_MATERIALS.len()).map(choice_rect));
        list.extend((0..CRAFT_GRID_HEIGHT).flat_map(|row| (0..CRAFT_GRID_WIDTH).map(move |column| cell_rect(row, column))));
        list
    }

    #[test]
    fn everything_fits_in_the_panel_without_overlap_and_is_big() {
        let targets = all_targets();
        assert_eq!(targets.len(), 2 + 5 + 7 + 12);
        for (i, a) in targets.iter().enumerate() {
            assert!(PANEL.contains(a.point()) && PANEL.contains(vec2(a.right(), a.bottom())), "{a:?} 판 밖");
            // 740×360 화면은 0.667배: 가장 작은 칸도 48 화면 픽셀 넘게
            assert!(a.w.min(a.h) * 360.0 / 540.0 >= 48.0, "{a:?} 작다");
            for b in &targets[i + 1..] {
                assert!(a.intersect(*b).is_none(), "{a:?} {b:?} 겹친다");
            }
        }
        // 안내 두 줄은 칸·재료 버튼·결과 글 아래, 판 안
        let grid_bottom = cell_rect(CRAFT_GRID_HEIGHT - 1, 0).bottom();
        assert!(STEP_LINE_1 - STEP_SIZE > grid_bottom && STEP_LINE_2 + 6.0 < PANEL.bottom());
        assert!(STEP_LINE_1 - STEP_SIZE > RESULT.bottom() + 40.0, "\"눌러요!\"와 안 겹친다");
        // 상태 글 줄은 그림 버튼 아래, 칸 위
        assert!(INFO_Y - INFO_SIZE > RECIPE_TOP + RECIPE_H && INFO_Y + 8.0 < GRID_TOP);
    }

    #[test]
    fn step_guide_turns_circled_numbers_into_plain_ones() {
        let steps = step_phrases();
        assert_eq!(steps.len(), 4, "{steps:?}");
        for (i, s) in steps.iter().enumerate() {
            assert!(s.starts_with(&format!("{}. ", i + 1)), "{s}");
            assert!(!s.contains('→') && !s.chars().any(|c| ('①'..='⑳').contains(&c)), "Jua에 없는 글자: {s}");
        }
    }

    #[test]
    fn taps_find_the_right_target() {
        assert_eq!(craft_tap_at(CLOSE.center()), Some(CraftTap::Close));
        assert_eq!(craft_tap_at(RESULT.center()), Some(CraftTap::Result));
        assert_eq!(craft_tap_at(recipe_rect(4).center()), Some(CraftTap::Recipe(Recipe::SyringeGun)));
        assert_eq!(craft_tap_at(choice_rect(6).center()), Some(CraftTap::Choose(Material::Diamond)));
        assert_eq!(craft_tap_at(choice_rect(1).center()), Some(CraftTap::Choose(Material::Iron)));
        assert_eq!(craft_tap_at(cell_rect(2, 3).center()), Some(CraftTap::Cell { row: 2, column: 3 }));
        assert_eq!(craft_tap_at(vec2(5.0, 5.0)), None);
    }

    /// 화면을 눌러서(그림 버튼 → 재료 → 칸) `recipe`를 만든다. 재료는 딱 맞게 넣어 둔다.
    fn make_by_tapping(recipe: Recipe) -> (Game, CraftView, Option<Recipe>) {
        let mut game = Game::on_world(CharacterKind::Niro, 7, World::m1());
        for (material, n) in recipe.needs() {
            for _ in 0..n {
                game.gathering.bag.add(material);
            }
        }
        game.crafting.open();
        let mut view = CraftView::new();
        let i = Recipe::ALL.iter().position(|r| *r == recipe).unwrap();
        apply_craft_tap(&mut game, &mut view, craft_tap_at(recipe_rect(i).center()).unwrap());
        assert_eq!(game.crafting.chosen(), recipe);
        assert_eq!(view.pick, recipe.needs()[0].0, "고르면 첫 재료가 바로 골라진다");
        assert_eq!(current_step(&game), 1);
        for row in 0..CRAFT_GRID_HEIGHT {
            for column in 0..CRAFT_GRID_WIDTH {
                let Some(material) = game.crafting.hint_at(row, column) else { continue };
                let choice = CRAFT_MATERIALS.iter().position(|m| *m == material).expect("재료 버튼에 있다");
                apply_craft_tap(&mut game, &mut view, craft_tap_at(choice_rect(choice).center()).unwrap());
                apply_craft_tap(&mut game, &mut view, craft_tap_at(cell_rect(row, column).center()).unwrap());
            }
        }
        assert_eq!(current_step(&game), 3, "{recipe:?} 결과가 나왔다");
        let made = apply_craft_tap(&mut game, &mut view, craft_tap_at(RESULT.center()).unwrap());
        (game, view, made)
    }

    #[test]
    fn every_recipe_can_be_made_by_tapping() {
        for recipe in Recipe::ALL {
            let (game, view, made) = make_by_tapping(recipe);
            assert_eq!(made, Some(recipe));
            assert!(recipe.already_have(&game.gathering.bag) || recipe == Recipe::EnderFruit, "{recipe:?}를 얻었다");
            assert_eq!(info_text(&game, &view), Some(recipe.crafted_text().to_string()), "축하 글");
            assert!(CRAFT_MATERIALS.iter().all(|m| game.gathering.bag.count(*m) == 0), "재료를 다 썼다");
        }
        let (game, ..) = make_by_tapping(Recipe::EnderFruit);
        assert_eq!(game.gathering.bag.ender_fruits, 1);
        let (game, ..) = make_by_tapping(Recipe::Car);
        assert!(game.gathering.bag.riding_car, "자동차는 만들면 바로 탄다");
    }

    #[test]
    fn info_says_what_is_missing_and_close_clears_it() {
        let mut game = Game::on_world(CharacterKind::Niro, 7, World::m1());
        game.crafting.open();
        let mut view = CraftView::new();
        apply_craft_tap(&mut game, &mut view, CraftTap::Recipe(Recipe::SyringeGun));
        assert_eq!(info_text(&game, &view), Recipe::SyringeGun.why_cant_make_text(&game.gathering.bag), "logic 글 그대로");
        assert!(info_text(&game, &view).is_some());
        assert_eq!(apply_craft_tap(&mut game, &mut view, CraftTap::Result), None, "판이 비면 아무것도 안 나온다");
        apply_craft_tap(&mut game, &mut view, CraftTap::Close);
        assert!(!game.crafting.is_open() && view.made.is_none());
    }
}
