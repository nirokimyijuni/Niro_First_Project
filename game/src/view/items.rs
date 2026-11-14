//! 아이가 디자인한 새 물건의 화면 (로건, US-078~083): 빨간 자동차 타기, 엔더열매 금빛 기운,
//! 주사기총 침 총알, 쿵 글자와 화면 흔들기, "부릉!" 글자, 제작대 그림 버튼 그림.
//! 언제 무엇이 일어나는지는 logic(`Game`, `ItemEvent`)이 정하고, 여기는 보여 주기만 한다.

use super::assets::{ItemPicture, M6Art};
use super::gather::draw_pickaxe;
use super::sky::hash;
use super::sprites::fit_size;
use super::text::{TextPainter, TEXT_ABOVE, TEXT_BELOW};
use super::touch::draw_star;
use crate::logic::crafting::Recipe;
use crate::logic::player::Facing;
use macroquad::prelude::*;
use std::f32::consts::TAU;

// ---- 자동차 (아이: 빨간색, 캐릭터가 타고 다닌다) ----
/// 헤일리 그림은 회색이라 빨갛게 물들인다 (흰 몸은 빨강, 검은 테두리는 그대로).
pub const CAR_RED: Color = Color::new(1.0, 0.22, 0.20, 1.0);
/// 맵에서 자동차 그림 가로 (가상 픽셀). 캐릭터(약 60)의 두 배쯤.
pub const CAR_WIDTH: f32 = 128.0;
/// 캐릭터는 자동차 높이의 이만큼 위에 앉는다 (다리는 차에 가려진다).
const SEAT_SHARE: f32 = 0.45;
/// 달릴 때 차가 통통 튀는 높이와 빠르기
const CAR_BOB: f32 = 2.5;
const CAR_BOBS_PER_SECOND: f32 = 6.0;

/// "부릉!" (아이: 자동차 소리). 효과음 파일이 없어서 글자만 띄운다.
pub const VROOM_TEXT: &str = crate::config::CAR_VROOM_TEXT;
/// 자동차를 타고 움직이는 동안 이 초마다 한 번 "부릉!"이 떠오른다.
pub const VROOM_EVERY_SECONDS: f32 = 1.6;
pub const VROOM_SECONDS: f32 = 1.0;
const VROOM_SIZE: f32 = 30.0;
const VROOM_COLOR: Color = Color::new(1.0, 0.95, 0.85, 1.0);

// ---- 엔더열매 금빛 기운 (아이: 몸에서 금빛 기운이 활활) ----
const AURA_GOLD: Color = Color::new(1.0, 0.82, 0.20, 1.0);
const AURA_LIGHT: Color = Color::new(1.0, 0.96, 0.62, 1.0);
const AURA_ORANGE: Color = Color::new(1.0, 0.58, 0.12, 1.0);
const AURA_FLAMES: u32 = 11;
pub const GOLDEN_PUFF_SECONDS: f32 = 1.0;

// ---- 주사기총 침 총알 ----
pub const NEEDLE_SECONDS: f32 = 0.8;
/// 침이 날아가는 거리: 화면 반쯤을 넘게 (화면 안 몬스터가 다 쓰러지니 양쪽 끝까지)
const NEEDLE_FLY: f32 = 560.0;
const NEEDLE_BODY: Color = Color::new(0.80, 0.95, 1.0, 1.0);
const NEEDLE_TIP: Color = Color::new(0.85, 0.88, 0.92, 1.0);
const NEEDLE_LIQUID: Color = Color::new(0.55, 0.92, 0.35, 1.0);
const NEEDLE_LINE: Color = Color::new(0.13, 0.16, 0.36, 1.0);

// ---- 쿵 (아이: "쿵! 몬스터들을 다 없애버려!" + 화면 흔들기) ----
/// 아이 글 그대로 (config `SLAM_TEXT`)
pub const SLAM_SHOUT_TEXT: &str = crate::config::SLAM_TEXT;
pub const SLAM_SECONDS: f32 = 2.8;
/// 흔들기는 처음 이만큼만, 점점 잦아든다
pub const SHAKE_SECONDS: f32 = 0.7;
const SHAKE_PIXELS: f32 = 14.0;
const SLAM_SHOUT_SIZE: f32 = 52.0;
const SLAM_LINE_SIZE: f32 = 36.0;
const SLAM_TEXT_COLOR: Color = Color::new(1.0, 0.86, 0.22, 1.0);
const SLAM_WAVE: Color = Color::new(0.75, 0.92, 1.0, 1.0);

/// 쿵 뒤 `age`초의 화면 흔들림 (가상 픽셀). 처음이 가장 크고 `SHAKE_SECONDS` 뒤에는 0이다.
pub fn shake_offset(age: f32) -> Vec2 {
    if !(0.0..SHAKE_SECONDS).contains(&age) {
        return Vec2::ZERO;
    }
    let strength = SHAKE_PIXELS * (1.0 - age / SHAKE_SECONDS);
    vec2((age * 53.0).sin(), (age * 41.0 + 1.3).sin()) * strength
}

/// 자동차를 탄 캐릭터의 발 자리: 자동차 바닥(`feet`)에서 앉는 높이만큼 위.
pub fn seat_feet(art: &M6Art, feet: Vec2, moving: bool, t: f32) -> Vec2 {
    let size = fit_size(art.item(ItemPicture::Car), CAR_WIDTH);
    feet - vec2(0.0, size.y * SEAT_SHARE + car_bob(moving, t))
}

fn car_bob(moving: bool, t: f32) -> f32 {
    if moving {
        CAR_BOB * (t * CAR_BOBS_PER_SECOND * TAU).sin().abs()
    } else {
        0.0
    }
}

/// 빨간 자동차 그림 (캐릭터 다음에 그려서 다리를 가린다). `feet`는 바퀴가 닿는 땅 가운데.
/// 그림은 오른쪽(운전석 쪽)을 본다.
pub fn draw_car(art: &M6Art, feet: Vec2, facing: Facing, tint: Color, moving: bool, t: f32) {
    let texture = art.item(ItemPicture::Car);
    let size = fit_size(texture, CAR_WIDTH);
    let y = feet.y - size.y - car_bob(moving, t) * 0.5;
    let color = Color::new(CAR_RED.r * tint.r, CAR_RED.g * tint.g, CAR_RED.b * tint.b, tint.a);
    draw_texture_ex(texture, feet.x - size.x / 2.0, y, color, DrawTextureParams { dest_size: Some(size), flip_x: facing == Facing::Left, ..Default::default() });
}

/// 떠오르는 "부릉!" 하나. `t`는 0..1, `at`은 처음 자리 (캐릭터 뒤쪽 옆).
pub fn draw_vroom(text: &TextPainter, at: Vec2, t: f32) {
    let fade = (1.0 - t).clamp(0.0, 1.0);
    let p = at + vec2(-18.0 * t, -40.0 * t);
    // 글꼴 크기는 고정하고 확대만 바꾼다 (프레임마다 새 글꼴 크기를 만들지 않게, qa/perf-2026-11-14.md)
    let zoom = 1.0 + 0.25 * (1.0 - (t * 5.0).min(1.0));
    text.centered_zoomed(VROOM_TEXT, p.x, p.y, VROOM_SIZE, zoom, Color { a: fade, ..VROOM_COLOR });
}

/// 엔더열매 힘이 있는 동안 몸에서 뿜어 나오는 금빛 불꽃 기운 (캐릭터보다 먼저 그린다).
/// 둥근 금빛 빛 + 위로 날름거리는 불꽃 혀 + 올라가는 반짝이. `body`는 몸 네모.
pub fn draw_golden_aura(body: Rect, t: f32) {
    let c = body.center();
    let pulse = 0.5 + 0.5 * (t * 5.0).sin();
    draw_ellipse(c.x, c.y + 4.0, body.w * 0.95 + 10.0 + 5.0 * pulse, body.h * 0.7 + 8.0, 0.0, Color { a: 0.18 + 0.08 * pulse, ..AURA_GOLD });
    for i in 0..AURA_FLAMES {
        let share = i as f32 / (AURA_FLAMES - 1) as f32;
        // 몸 둘레 아래 반쪽에서 위로 솟는다: 가운데 불꽃이 가장 크다
        let x = body.x - 10.0 + share * (body.w + 20.0);
        let middle = 1.0 - (share - 0.5).abs() * 2.0;
        let flicker = 0.5 + 0.5 * (t * (9.0 + hash(i) * 5.0) + hash(i + 40) * TAU).sin();
        let height = body.h * (0.55 + 0.6 * middle) * (0.75 + 0.35 * flicker);
        let base_y = body.y + body.h - 6.0 - middle * body.h * 0.15;
        let sway = (t * 4.0 + i as f32).sin() * 5.0;
        let tip = vec2(x + sway, base_y - height);
        let half = 7.0 + 4.0 * middle;
        let color = if i % 3 == 0 { AURA_ORANGE } else { AURA_GOLD };
        draw_triangle(vec2(x - half, base_y), vec2(x + half, base_y), tip, Color { a: 0.55, ..color });
        draw_triangle(vec2(x - half * 0.45, base_y), vec2(x + half * 0.45, base_y), tip + vec2(0.0, height * 0.3), Color { a: 0.6, ..AURA_LIGHT });
        draw_circle(x, base_y, half, Color { a: 0.5, ..color });
    }
    for i in 0..5 {
        let rise = (t * 0.8 + i as f32 / 5.0).fract();
        let x = c.x + (hash(i + 7) - 0.5) * body.w * 1.4;
        let p = vec2(x + (t * 3.0 + i as f32).sin() * 4.0, body.y + body.h - rise * (body.h + 40.0));
        draw_star(p, 4.0 + 3.0 * (1.0 - rise), t * 2.0, Color { a: 1.0 - rise, ..AURA_LIGHT });
    }
}

/// 엔더열매를 먹은 순간: 금빛 고리가 퍼지고 별이 튄다. `t`는 0..1.
pub fn draw_golden_puff(at: Vec2, t: f32) {
    let fade = 1.0 - t;
    let r = 20.0 + 90.0 * t;
    draw_circle_lines(at.x, at.y, r, 6.0 * fade + 1.0, Color { a: fade, ..AURA_GOLD });
    draw_circle(at.x, at.y, r * 0.6, Color { a: 0.25 * fade, ..AURA_LIGHT });
    for i in 0..8 {
        let a = i as f32 * TAU / 8.0 + t;
        draw_star(at + vec2(a.cos(), a.sin()) * r, 9.0 * fade + 3.0, a, Color { a: fade, ..AURA_LIGHT });
    }
}

/// 침 총알 하나: 몸통(초록 약), 바늘 끝. `dir`은 1(오른쪽) 또는 -1(왼쪽).
fn draw_needle(p: Vec2, dir: f32, alpha: f32) {
    let len = 34.0;
    let back = p - vec2(dir * len, 0.0);
    draw_line(back.x, back.y, p.x - dir * 10.0, p.y, 9.0, Color { a: alpha, ..NEEDLE_LINE });
    draw_line(back.x + dir, back.y, p.x - dir * 11.0, p.y, 6.0, Color { a: alpha, ..NEEDLE_BODY });
    draw_line(back.x + dir * 6.0, back.y, p.x - dir * 16.0, p.y, 3.0, Color { a: alpha, ..NEEDLE_LIQUID });
    draw_line(p.x - dir * 10.0, p.y, p.x, p.y, 2.0, Color { a: alpha, ..NEEDLE_TIP });
    // 뒤에 하얀 바람 줄
    draw_line(back.x - dir * 4.0, back.y, back.x - dir * 22.0, back.y, 2.0, Color { a: alpha * 0.6, ..WHITE });
}

/// 주사기총을 쐈다: 침 총알이 주인공에게서 양쪽으로 줄지어 날아간다. `t`는 0..1.
pub fn draw_needles(at: Vec2, t: f32) {
    let alpha = ((1.0 - t) * 4.0).min(1.0);
    for dir in [-1.0, 1.0] {
        for (i, dy) in [-18.0, 0.0, 18.0].into_iter().enumerate() {
            let delay = i as f32 * 0.08;
            let k = ((t - delay) / (1.0 - delay)).clamp(0.0, 1.0);
            if k <= 0.0 {
                continue;
            }
            draw_needle(at + vec2(dir * (20.0 + NEEDLE_FLY * k), dy), dir, alpha);
        }
    }
}

/// 쿵 하는 순간 땅에서 퍼지는 물결 고리 (맵 좌표). `t`는 0..1.
pub fn draw_slam_wave(at: Vec2, t: f32) {
    let fade = (1.0 - t).max(0.0);
    for k in 0..3 {
        let r = (40.0 + 420.0 * t) * (1.0 - k as f32 * 0.22);
        draw_ellipse_lines(at.x, at.y + 30.0, r, r * 0.25, 0.0, 6.0 * fade + 1.0, Color { a: fade * 0.8, ..SLAM_WAVE });
    }
}

/// 쿵 큰 글 바닥선에서 아래 첫 줄 바닥선까지, 그 아래 줄 사이 (배율 1)
const SLAM_LINE_GAP: f32 = 52.0;
const SLAM_LINE_STEP: f32 = 44.0;
/// 글이 들어갈 화면 폭 (양쪽 20 여유)
const FIT_WIDTH: f32 = 920.0;

/// 쿵 글 덩이(큰 글 + 아래 `lines`줄)의 높이 (배율 1). 위 끝은 큰 글 위, 아래 끝은 마지막 줄 아래.
pub fn slam_block_height(lines: usize) -> f32 {
    let above = SLAM_SHOUT_SIZE * TEXT_ABOVE;
    match lines {
        0 => above + SLAM_SHOUT_SIZE * TEXT_BELOW,
        n => above + SLAM_LINE_GAP + (n - 1) as f32 * SLAM_LINE_STEP + SLAM_LINE_SIZE * TEXT_BELOW,
    }
}

/// 쿵 글 덩이의 폭 (배율 1, 큰 글은 화면 폭에 맞게 줄인 뒤)
pub fn slam_block_width(text: &TextPainter, lines: &[&str]) -> f32 {
    let shout = text.width(SLAM_SHOUT_TEXT, SLAM_SHOUT_SIZE).min(FIT_WIDTH);
    lines.iter().map(|l| text.width(l, SLAM_LINE_SIZE)).fold(shout, f32::max)
}

/// 쿵 글자 (화면 좌표): 큰 "쿵! 몬스터들을 다 없애버려!", 그 아래 얻은 다이아몬드와 (있으면) 곡괭이가 사라진 글.
/// `top`은 덩이 위 끝, `scale`은 다른 큰 글과 함께 화면에 들어가게 줄인 배율 (`effects`가 정한다, BUG-031).
pub fn draw_slam_shout(text: &TextPainter, age: f32, lines: &[&str], top: f32, scale: f32) {
    let t = age / SLAM_SECONDS;
    let fade = ((1.0 - t) * 4.0).clamp(0.0, 1.0);
    let pop = 1.0 + 0.35 * (1.0 - (age / 0.25).min(1.0));
    // 글꼴 크기(SLAM_SHOUT_SIZE)는 고정하고 확대만 바꾼다. 화면 폭을 넘으면 확대를 줄인다
    let shown = scale * pop;
    let zoom = shown * (FIT_WIDTH / (text.width(SLAM_SHOUT_TEXT, SLAM_SHOUT_SIZE) * shown).max(1.0)).min(1.0);
    let baseline = top + SLAM_SHOUT_SIZE * TEXT_ABOVE * scale;
    text.centered_zoomed(SLAM_SHOUT_TEXT, 480.0, baseline, SLAM_SHOUT_SIZE, zoom, Color { a: fade, ..SLAM_TEXT_COLOR });
    for (i, line) in lines.iter().enumerate() {
        let y = baseline + (SLAM_LINE_GAP + i as f32 * SLAM_LINE_STEP) * scale;
        text.centered(line, 480.0, y, SLAM_LINE_SIZE * scale, Color { a: fade, ..WHITE });
    }
}

/// 만들 것 그림 (제작대 그림 버튼·결과 칸). 곡괭이는 코드 그림, 나머지는 헤일리 그림. 자동차는 빨갛게.
pub fn draw_recipe_icon(recipe: Recipe, art: &M6Art, c: Vec2, size: f32) {
    let picture = match recipe {
        Recipe::Pickaxe => {
            draw_pickaxe(c, size * 0.9, WHITE);
            return;
        }
        Recipe::Car => ItemPicture::Car,
        Recipe::EnderFruit => ItemPicture::EnderFruit,
        Recipe::DiamondPickaxe => ItemPicture::DiamondPickaxe,
        Recipe::SyringeGun => ItemPicture::SyringeGun,
    };
    draw_item_picture(art, picture, c, size);
}

/// 헤일리 그림 하나를 `size` 안에 맞춰 그린다. 자동차는 빨갛게 물들인다.
pub fn draw_item_picture(art: &M6Art, picture: ItemPicture, c: Vec2, size: f32) {
    let texture = art.item(picture);
    let s = fit_size(texture, size);
    let color = if picture == ItemPicture::Car { CAR_RED } else { WHITE };
    draw_texture_ex(texture, c.x - s.x / 2.0, c.y - s.y / 2.0, color, DrawTextureParams { dest_size: Some(s), ..Default::default() });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shake_fades_out_and_stops() {
        assert_eq!(shake_offset(-0.1), Vec2::ZERO);
        assert_eq!(shake_offset(SHAKE_SECONDS), Vec2::ZERO);
        assert_eq!(shake_offset(5.0), Vec2::ZERO);
        let mut biggest_early: f32 = 0.0;
        let mut biggest_late: f32 = 0.0;
        for i in 0..70 {
            let age = i as f32 * 0.01;
            let len = shake_offset(age).length();
            assert!(len <= SHAKE_PIXELS * 1.5, "너무 세게 흔들지 않는다");
            if age < SHAKE_SECONDS / 3.0 {
                biggest_early = biggest_early.max(len);
            } else if age > SHAKE_SECONDS * 2.0 / 3.0 {
                biggest_late = biggest_late.max(len);
            }
        }
        assert!(biggest_early > 5.0, "처음엔 눈에 띄게 흔들린다");
        assert!(biggest_late < biggest_early, "점점 잦아든다");
    }

    #[test]
    fn car_is_red_and_kid_words_are_kept() {
        assert!(CAR_RED.r > 0.9 && CAR_RED.g < 0.4 && CAR_RED.b < 0.4, "아이: 빨간 자동차");
        assert_eq!(VROOM_TEXT, "부릉!");
        assert_eq!(SLAM_SHOUT_TEXT, "쿵! 몬스터들을 다 없애버려!");
        assert!(SHAKE_SECONDS < SLAM_SECONDS, "흔들기는 글보다 짧다");
        assert!(VROOM_SECONDS <= VROOM_EVERY_SECONDS, "부릉 글자가 겹겹이 쌓이지 않는다");
    }

    #[test]
    fn car_bob_stays_small_and_stops_when_parked() {
        for i in 0..100 {
            let t = i as f32 * 0.013;
            assert_eq!(car_bob(false, t), 0.0);
            assert!((0.0..=CAR_BOB).contains(&car_bob(true, t)));
        }
    }
}
