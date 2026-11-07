//! 긴 맵의 배경 (US-050): 해변 → 화산 → 산 → 집. 하늘과 바다는 sky.rs, 여기는 그 앞의 땅이다.
//!
//! - 좌표는 모두 맵 x라서 `Letterbox::look_at`을 건 뒤에 그린다. 하늘·해·달·바다는 화면에 고정이다.
//! - 끊긴 곳이 없게 땅을 장소마다 따로 그리지 않고, 맵 x로 높이·색을 구해 세로 띠로 잇는다.
//!   색은 장소 경계 양쪽에서 부드럽게 섞이고, 먼 땅은 해변 끝에서 바다 위로 천천히 솟는다.
//! - 그림일 뿐 규칙은 없다 (용암도 닿아도 아무 일이 없다). 밟는 구조물·숨는 곳은 logic이 정한다.
//! - 색·자리는 그림 값이라 이 파일 위쪽 상수에 둔다. 아이가 색을 고르면 여기만 바꾸면 된다.

use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::sky::{hash, lerp, mix, night, ramp, with_alpha, Canvas, GpuCanvas, SkyArt, SkyLook, Tex, HORIZON_Y};
use crate::config::{HIDE_SPOTS, MISS_HEIGHT};
use crate::logic::world::World;
use macroquad::prelude::*;

// ---- 장소 ----
/// 장소 하나의 폭 (M6 맨 위 표: 해변 0~960, 화산 ~1,920, 산 ~2,880, 집 ~3,840)
const PLACE_WIDTH: f32 = 960.0;
/// 장소가 바뀌는 곳 양쪽으로 이만큼에 걸쳐 색이 섞인다
const PLACE_BLEND: f32 = 110.0;
/// 먼 땅이 바다 위로 솟기 시작하는 곳과 수평선을 다 가리는 곳
const LAND_START: f32 = 920.0;
const LAND_FULL: f32 = 1150.0;
/// 이 x까지 해변 그림(알갱이·조개·물결)이 있다. 끝은 [`SHORE_ROCKS`]가 가린다.
pub const BEACH_END: f32 = 1000.0;
/// 띠 한 칸 폭. 칸을 맵 격자에 맞춰서 걸어도 땅 모양이 떨리지 않는다.
const STRIP_STEP: f32 = 8.0;
/// 땅 윗부분 띠 두께 (해변은 젖은 모래)
const GROUND_LIP: f32 = 16.0;

// ---- 색: `[해변, 화산, 산, 집]` ----
const GROUND_TOP: [Color; 4] = [
    Color::new(0.74, 0.59, 0.40, 1.0),
    Color::new(0.47, 0.37, 0.36, 1.0),
    Color::new(0.45, 0.62, 0.30, 1.0),
    Color::new(0.52, 0.72, 0.34, 1.0),
];
/// 모래: 아이 그림의 흰 얼굴이 묻히지 않게 너무 하얗지 않은 따뜻한 베이지. 화산 땅은 나무 상자와 구별되게 보랏빛.
const GROUND_MID: [Color; 4] = [
    Color::new(0.90, 0.76, 0.52, 1.0),
    Color::new(0.58, 0.47, 0.46, 1.0),
    Color::new(0.55, 0.73, 0.38, 1.0),
    Color::new(0.63, 0.82, 0.42, 1.0),
];
const GROUND_DEEP: [Color; 4] = [
    Color::new(0.84, 0.68, 0.45, 1.0),
    Color::new(0.50, 0.40, 0.40, 1.0),
    Color::new(0.46, 0.64, 0.32, 1.0),
    Color::new(0.54, 0.73, 0.36, 1.0),
];
const FAR_TOP: [Color; 4] = [
    Color::new(0.72, 0.60, 0.46, 1.0),
    Color::new(0.60, 0.42, 0.42, 1.0),
    Color::new(0.42, 0.62, 0.56, 1.0),
    Color::new(0.56, 0.76, 0.50, 1.0),
];
const FAR_BOTTOM: [Color; 4] = [
    Color::new(0.62, 0.50, 0.38, 1.0),
    Color::new(0.48, 0.34, 0.36, 1.0),
    Color::new(0.34, 0.54, 0.42, 1.0),
    Color::new(0.46, 0.68, 0.42, 1.0),
];
const NEAR_TOP: [Color; 4] = [
    Color::new(0.70, 0.58, 0.44, 1.0),
    Color::new(0.54, 0.38, 0.34, 1.0),
    Color::new(0.36, 0.60, 0.30, 1.0),
    Color::new(0.42, 0.68, 0.32, 1.0),
];
const NEAR_BOTTOM: [Color; 4] = [
    Color::new(0.62, 0.50, 0.38, 1.0),
    Color::new(0.44, 0.31, 0.29, 1.0),
    Color::new(0.30, 0.50, 0.26, 1.0),
    Color::new(0.36, 0.58, 0.28, 1.0),
];
/// 먼 땅에 하늘빛이 섞이는 양. 노을·밤 색을 따라가게 한다.
const FAR_HAZE: f32 = 0.22;
/// 밤에 어두워지는 양. 캐릭터가 서 있는 쪽일수록 덜 어둡다 (US-050: 밤에도 잘 보이게).
const FAR_NIGHT_AMOUNT: f32 = 0.92;
const NEAR_NIGHT_AMOUNT: f32 = 0.88;
const GROUND_NIGHT_AMOUNT: f32 = 0.85;

// ---- 땅 높이 (땅 선에서 위로 픽셀) ----
const FAR_HILL_HEIGHT: f32 = 136.0;
const NEAR_HILL_HEIGHT: f32 = 58.0;
const NEAR_START: (f32, f32) = (960.0, 1120.0);

// ---- 화산: 동글동글하고 귀여운 불꽃산. 용암은 밝은 주황이고 연기는 뭉게구름이다 ----
const VOLCANO_X: f32 = 1440.0;
const VOLCANO_HALF_WIDTH: f32 = 420.0;
const VOLCANO_HEIGHT: f32 = 310.0;
/// 분화구 높이: 꼭대기를 여기서 평평하게 자른다
const VOLCANO_RIM: f32 = 262.0;
const VOLCANO_SLOPE: f32 = 1.3;
const LAVA: Color = Color::new(1.0, 0.52, 0.12, 1.0);
const LAVA_CORE: Color = Color::new(1.0, 0.86, 0.32, 1.0);
const LAVA_WIDTH: f32 = 13.0;
/// 분화구에서 흘러내리는 용암 줄기: 분화구에서 얼마나 옆으로 (픽셀), 얼마나 내려오는지 (높이 비율)
const LAVA_STREAMS: [(f32, f32); 3] = [(-34.0, 0.62), (6.0, 0.78), (40.0, 0.55)];
const GLOW_DAY_ALPHA: f32 = 0.35;
const GLOW_NIGHT_ALPHA: f32 = 0.8;
const SMOKE: Color = Color::new(0.92, 0.90, 0.92, 1.0);
const SMOKE_PUFFS: u32 = 6;
const SMOKE_RISE: f32 = 130.0;
const SMOKE_ALPHA: f32 = 0.6;
const ROCK: Color = Color::new(0.42, 0.34, 0.36, 1.0);
const ROCK_LIGHT: Color = Color::new(0.56, 0.47, 0.48, 1.0);
/// 화산 쪽 앞 언덕의 바위 (x, 크기)
const VOLCANO_ROCKS: [(f32, f32); 4] = [(1090.0, 1.0), (1210.0, 0.7), (1620.0, 0.9), (1790.0, 1.1)];
/// 해변이 끝나는 곳의 바위 더미 (x, y, 반지름). 물결·모래 그림의 끝을 가린다.
const SHORE_ROCKS: [(f32, f32, f32); 3] = [(1016.0, 444.0, 30.0), (982.0, 450.0, 25.0), (1040.0, 452.0, 20.0)];

// ---- 산: 뾰족한 세모 산에 하얀 눈 ----
/// (꼭대기 x, 높이, 반폭). x는 `STRIP_STEP` 배수라야 꼭대기가 깎이지 않는다.
const MOUNTAINS: [(f32, f32, f32); 3] = [(2152.0, 232.0, 270.0), (2424.0, 292.0, 310.0), (2696.0, 214.0, 250.0)];
const MOUNTAIN_SLOPE: f32 = 1.1;
/// 눈이 시작되는 맵 x (화산 꼭대기에 눈이 오지 않게)
const SNOW_FROM_X: f32 = 1920.0;
/// 눈 아래 끝 y와 삐죽삐죽한 폭·간격
const SNOW_Y: f32 = 238.0;
const SNOW_ZIG: f32 = 14.0;
const SNOW_ZIG_EVERY: f32 = 48.0;
const SNOW: Color = Color::new(0.97, 0.98, 1.0, 1.0);
const PINE: Color = Color::new(0.20, 0.47, 0.30, 1.0);
const PINE_LIGHT: Color = Color::new(0.28, 0.58, 0.34, 1.0);
const TRUNK: Color = Color::new(0.48, 0.32, 0.20, 1.0);
/// 앞 언덕의 뾰족나무 (x, 크기). 큰 나무(x 1,920~2,216)와 바위 계단(x 2,440~2,760) 자리는 비워 둔다.
const PINES: [(f32, f32); 4] = [(2290.0, 1.0), (2370.0, 0.85), (2810.0, 1.05), (2860.0, 0.8)];

// ---- 집 ----
/// 집 벽 왼쪽·오른쪽 끝. 오른쪽 끝은 "집 뒤" 숨는 곳(x 3,700)의 몸을 3/4쯤 가리는 자리다.
const HOUSE_LEFT: f32 = 3405.0;
const HOUSE_RIGHT: f32 = 3705.0;
const HOUSE_WALL_HEIGHT: f32 = 170.0;
const HOUSE_ROOF_HEIGHT: f32 = 110.0;
const HOUSE_ROOF_OVERHANG: f32 = 26.0;
const WALL: Color = Color::new(0.98, 0.92, 0.76, 1.0);
const ROOF: Color = Color::new(0.90, 0.36, 0.32, 1.0);
const DOOR: Color = Color::new(0.62, 0.40, 0.24, 1.0);
const WINDOW_DAY: Color = Color::new(0.62, 0.84, 0.96, 1.0);
const WINDOW_NIGHT: Color = Color::new(1.0, 0.90, 0.52, 1.0);
const LINE: Color = Color::new(0.30, 0.20, 0.16, 1.0);
/// 벽·창문은 밤에 덜 어둡게: 불 켜진 집이 포근해 보이게
const HOUSE_NIGHT_AMOUNT: f32 = 0.6;
const LEAF: Color = Color::new(0.34, 0.66, 0.30, 1.0);
const LEAF_LIGHT: Color = Color::new(0.46, 0.76, 0.36, 1.0);
const FENCE: Color = Color::new(0.98, 0.96, 0.90, 1.0);
const FENCE_FROM_TO: (f32, f32) = (2980.0, 3390.0);
const FENCE_EVERY: f32 = 42.0;
const FENCE_HEIGHT: f32 = 40.0;
/// 앞 언덕의 둥근 나무 (x, 크기)
const ROUND_TREES: [(f32, f32); 5] = [(2950.0, 1.0), (3110.0, 1.15), (3260.0, 0.9), (3370.0, 1.05), (3790.0, 1.1)];

// ---- 밟는 구조물과 숨는 곳 앞그림 (자리는 logic이 준다) ----
const HOLE_BACK: Color = Color::new(0.34, 0.26, 0.28, 1.0);
const HOLE_FLOOR: Color = Color::new(0.26, 0.20, 0.22, 1.0);
const BARK: Color = Color::new(0.56, 0.38, 0.24, 1.0);
const BARK_LIGHT: Color = Color::new(0.72, 0.54, 0.34, 1.0);
const BIG_TREE_TRUNK_WIDTH: f32 = 40.0;
/// 큰 나무 잎: 줄기 위 끝에서 (옆으로, 위로, 반지름)
const BIG_TREE_LEAVES: [(f32, f32, f32); 4] = [(-74.0, -16.0, 50.0), (78.0, -6.0, 54.0), (0.0, 26.0, 62.0), (0.0, -24.0, 58.0)];
/// 큰 나무 줄기 위 끝이 맨 위 발판보다 이만큼 위
const BIG_TREE_ABOVE_TOP_BRANCH: f32 = 70.0;
const STEP_LIP: f32 = 10.0;
/// 앞그림은 미스 몸의 아래 3/4를 가려서 머리만 빼꼼 보인다 (US-051)
const COVER_HEIGHT: f32 = MISS_HEIGHT * 0.75;
const COVER_HALF_WIDTH: f32 = 58.0;
const PEAK_COVER_HALF_WIDTH: f32 = 34.0;

// ---- 땅 위 작은 것 ----
const SPECK_EVERY: f32 = 36.0;
const PEBBLE: Color = Color::new(0.36, 0.29, 0.30, 0.8);
const GRASS: Color = Color::new(0.30, 0.52, 0.22, 0.9);
const FLOWERS: [Color; 3] = [Color::new(1.0, 0.95, 0.45, 1.0), Color::new(1.0, 0.62, 0.74, 1.0), WHITE];

/// `[해변, 화산, 산, 집]` 색표에서 맵 `x`의 색. 가장 가까운 경계 양쪽 [`PLACE_BLEND`]에서만 섞인다.
fn place_color(table: &[Color; 4], x: f32) -> Color {
    let edge = (x / PLACE_WIDTH).round().clamp(1.0, 3.0) as usize;
    let at = edge as f32 * PLACE_WIDTH;
    mix(table[edge - 1], table[edge], ramp(x, at - PLACE_BLEND, at + PLACE_BLEND))
}

/// `center`에서 `height`, `half_width` 밖에서 0인 산 모양. `slope`가 크면 옆구리가 오목하다.
fn peak(x: f32, center: f32, half_width: f32, height: f32, slope: f32) -> f32 {
    height * (1.0 - (x - center).abs() / half_width).max(0.0).powf(slope)
}

/// 먼 땅 높이: 낮은 언덕 위로 화산과 산이 솟는다. 해변에서는 0.
fn far_height(x: f32) -> f32 {
    let hills = ramp(x, LAND_START, LAND_FULL) * (FAR_HILL_HEIGHT + (x * 0.011).sin() * 12.0 + (x * 0.027 + 1.0).sin() * 7.0);
    let volcano = peak(x, VOLCANO_X, VOLCANO_HALF_WIDTH, VOLCANO_HEIGHT, VOLCANO_SLOPE).min(VOLCANO_RIM);
    MOUNTAINS.iter().map(|&(c, h, w)| peak(x, c, w, h, MOUNTAIN_SLOPE)).fold(hills.max(volcano), f32::max)
}

/// 앞 언덕 높이
fn near_height(x: f32) -> f32 {
    ramp(x, NEAR_START.0, NEAR_START.1) * (NEAR_HILL_HEIGHT + (x * 0.017 + 2.0).sin() * 12.0 + (x * 0.041).sin() * 6.0)
}

/// 눈 아래 끝 y. 산이 눈 선보다 낮으면 `top`이라 아무것도 그리지 않는다.
fn snow_bottom(x: f32, top: f32) -> f32 {
    if x < SNOW_FROM_X {
        return top;
    }
    let zig = ((x / SNOW_ZIG_EVERY).fract() * 2.0 - 1.0).abs();
    top.max(SNOW_Y + zig * SNOW_ZIG)
}

/// 바다가 화면에 보이는지. 먼 땅이 수평선을 다 가리면 안 그려도 된다.
pub fn sea_visible(camera_x: f32) -> bool {
    camera_x < LAND_FULL
}

/// 화면 폭만큼 세로 띠를 잇는다. 윗선·아랫선과 그 색을 맵 x로 받는다.
fn draw_strip(camera_x: f32, top: impl Fn(f32) -> f32, bottom: impl Fn(f32) -> f32, top_color: impl Fn(f32) -> Color, bottom_color: impl Fn(f32) -> Color) {
    let first = (camera_x / STRIP_STEP).floor() * STRIP_STEP;
    let columns = (VIRTUAL_WIDTH / STRIP_STEP) as u16 + 2;
    let mut vertices = Vec::with_capacity(columns as usize * 2 + 2);
    let mut indices = Vec::with_capacity(columns as usize * 6);
    for i in 0..=columns {
        let x = first + i as f32 * STRIP_STEP;
        vertices.push(Vertex::new(x, top(x), 0.0, 0.0, 0.0, top_color(x)));
        vertices.push(Vertex::new(x, bottom(x), 0.0, 0.0, 0.0, bottom_color(x)));
        if i > 0 {
            let b = (i - 1) * 2;
            indices.extend_from_slice(&[b, b + 1, b + 2, b + 1, b + 3, b + 2]);
        }
    }
    draw_mesh(&Mesh { vertices, indices, texture: None });
}

fn on_screen(camera_x: f32, left: f32, right: f32) -> bool {
    right > camera_x && left < camera_x + VIRTUAL_WIDTH
}

/// 땅 선 아래: 장소마다 다른 바닥 + 조약돌·풀·꽃 (해변 알갱이는 screens.rs).
pub fn draw_ground_band(look: &SkyLook, ground_y: f32, camera_x: f32) {
    let d = look.d * GROUND_NIGHT_AMOUNT;
    let lip = ground_y + GROUND_LIP;
    let color = |table: &'static [Color; 4]| move |x: f32| night(place_color(table, x), d);
    draw_strip(camera_x, |_| ground_y, |_| lip, color(&GROUND_TOP), color(&GROUND_MID));
    draw_strip(camera_x, |_| lip, |_| VIRTUAL_HEIGHT, color(&GROUND_MID), color(&GROUND_DEEP));

    let first = (camera_x / SPECK_EVERY).floor() as u32;
    for cell in first..=first + (VIRTUAL_WIDTH / SPECK_EVERY) as u32 + 1 {
        let n = cell * 4;
        let x = (cell as f32 + hash(n)) * SPECK_EVERY;
        if x < PLACE_WIDTH + PLACE_BLEND {
            continue;
        }
        let y = lip + 2.0 + hash(n + 1) * (VIRTUAL_HEIGHT - lip - 8.0);
        if x < PLACE_WIDTH * 2.0 {
            draw_circle(x, y, 1.5 + hash(n + 2) * 3.0, night(PEBBLE, d));
        } else {
            draw_grass(x, y, hash(n + 2), d);
        }
    }
}

/// 풀 한 포기. 네 번에 한 번은 작은 꽃이 핀다.
fn draw_grass(x: f32, y: f32, pick: f32, d: f32) {
    let green = night(GRASS, d);
    for dx in [-4.0, 0.0, 4.0] {
        draw_line(x + dx * 0.4, y, x + dx, y - 8.0, 2.0, green);
    }
    if pick < 0.25 {
        draw_circle(x, y - 10.0, 3.5, night(FLOWERS[(pick * 12.0) as usize % FLOWERS.len()], d));
    }
}

/// 땅 선 위: 먼 땅(화산·산) → 앞 언덕 → 나무·바위·집.
pub fn draw_land(art: &SkyArt, look: &SkyLook, ground_y: f32, camera_x: f32) {
    let d = look.d;
    let bottom = |_: f32| ground_y;
    let far_top = |x: f32| ground_y - far_height(x);
    let haze = look.sky_at(HORIZON_Y);
    let far = |table: &'static [Color; 4]| move |x: f32| night(mix(place_color(table, x), haze, FAR_HAZE), d * FAR_NIGHT_AMOUNT);
    draw_strip(camera_x, far_top, bottom, far(&FAR_TOP), far(&FAR_BOTTOM));
    let snow = |_: f32| night(mix(SNOW, haze, FAR_HAZE), d * FAR_NIGHT_AMOUNT);
    draw_strip(camera_x, far_top, |x| snow_bottom(x, far_top(x)), snow, snow);
    if on_screen(camera_x, VOLCANO_X - VOLCANO_HALF_WIDTH, VOLCANO_X + VOLCANO_HALF_WIDTH) {
        draw_volcano_top(art, look, ground_y);
    }

    let near_d = d * NEAR_NIGHT_AMOUNT;
    let near = |table: &'static [Color; 4]| move |x: f32| night(place_color(table, x), near_d);
    draw_strip(camera_x, |x| ground_y - near_height(x), bottom, near(&NEAR_TOP), near(&NEAR_BOTTOM));
    let base_y = |x: f32| ground_y - near_height(x) * 0.45;
    for (x, s) in VOLCANO_ROCKS {
        if on_screen(camera_x, x - 50.0, x + 50.0) {
            draw_rock(vec2(x, base_y(x)), 26.0 * s, near_d);
        }
    }
    for (x, s) in PINES {
        if on_screen(camera_x, x - 60.0, x + 60.0) {
            draw_pine(vec2(x, base_y(x)), s, near_d);
        }
    }
    for (x, s) in ROUND_TREES {
        if on_screen(camera_x, x - 70.0, x + 70.0) {
            draw_round_tree(vec2(x, base_y(x)), s, near_d);
        }
    }
    if on_screen(camera_x, FENCE_FROM_TO.0, FENCE_FROM_TO.1) {
        draw_fence(ground_y, near_d);
    }
    if on_screen(camera_x, 950.0, 1070.0) {
        for (x, y, r) in SHORE_ROCKS {
            draw_rock(vec2(x, y), r, d * GROUND_NIGHT_AMOUNT);
        }
    }
}

/// 밟는 구조물: 구멍, 큰 나무와 발판, 바위 계단. 미스보다 먼저 그린다.
pub fn draw_structures(world: &World, d: f32) {
    let d = d * NEAR_NIGHT_AMOUNT;
    if let Some(hole) = world.hole() {
        draw_rectangle(hole.x, hole.y, hole.w, hole.h, night(HOLE_BACK, d));
        draw_rectangle(hole.x, hole.y + hole.h - 8.0, hole.w, 8.0, night(HOLE_FLOOR, d));
    }
    draw_big_tree(&world.tree_branches(), world.ground_y, d);
    for step in world.rock_steps() {
        draw_rectangle(step.x, step.y, step.w, step.h, night(ROCK, d));
        draw_rectangle(step.x, step.y, step.w, STEP_LIP, night(ROCK_LIGHT, d));
    }
}

/// 그루터기 + 줄기 + 가지 발판 + 뒤쪽 잎. 줄기는 그림일 뿐이고 발판만 밟힌다.
fn draw_big_tree(branches: &[Rect], ground_y: f32, d: f32) {
    let [stump, low, top] = branches else { return };
    let (trunk_x, trunk_top) = ((low.x + low.w + top.x) / 2.0, top.y - BIG_TREE_ABOVE_TOP_BRANCH);
    for (dx, dy, r) in BIG_TREE_LEAVES {
        draw_circle(trunk_x + dx, trunk_top - dy, r, night(if dy > 0.0 { LEAF_LIGHT } else { LEAF }, d));
    }
    draw_rectangle(trunk_x - BIG_TREE_TRUNK_WIDTH / 2.0, trunk_top, BIG_TREE_TRUNK_WIDTH, ground_y - trunk_top, night(BARK, d));
    draw_rectangle(stump.x, stump.y, stump.w, stump.h, night(BARK, d));
    draw_ellipse(stump.x + stump.w / 2.0, stump.y, stump.w / 2.0, 9.0, 0.0, night(BARK_LIGHT, d));
    for b in [low, top] {
        // 가지가 줄기까지 이어지게 줄기 쪽으로 늘린다
        let (left, right) = (b.x.min(trunk_x), (b.x + b.w).max(trunk_x));
        draw_rectangle(left, b.y, right - left, b.h, night(BARK, d));
        draw_rectangle(b.x, b.y, b.w, 5.0, night(LEAF_LIGHT, d));
    }
}

/// 아래가 평평한 반타원. 땅에 얹힌 돌·덤불.
fn draw_dome(center_x: f32, base_y: f32, half_width: f32, height: f32, color: Color) {
    const STEPS: usize = 16;
    let c = vec2(center_x, base_y);
    let rim = |i: usize| {
        let a = std::f32::consts::PI * i as f32 / STEPS as f32;
        c + vec2(-a.cos() * half_width, -a.sin() * height)
    };
    for i in 0..STEPS {
        draw_triangle(c, rim(i), rim(i + 1), color);
    }
}

/// 미스를 가리는 앞그림: 큰 돌, 바위, 나뭇잎, 집. 미스 다음, 상자·캐릭터보다 먼저 그린다.
/// 자리는 `HIDE_SPOTS`를 따라가서 숨는 곳을 옮기면 같이 옮겨진다. 미스가 없는 곳에도 늘 있다.
pub fn draw_covers(world: &World, d: f32) {
    if !world.is_long_map() {
        return;
    }
    let near_d = d * NEAR_NIGHT_AMOUNT;
    for (name, x, feet_height) in HIDE_SPOTS {
        let base_y = world.ground_y - feet_height;
        let (color, light, half_width) = match name {
            "집 뒤" => continue,
            "나무 꼭대기" => (LEAF, LEAF_LIGHT, COVER_HALF_WIDTH),
            // 계단 꼭대기 칸(폭 64)보다 넓으면 버섯처럼 보인다
            "산꼭대기" => (ROCK, ROCK_LIGHT, PEAK_COVER_HALF_WIDTH),
            _ => (ROCK, ROCK_LIGHT, COVER_HALF_WIDTH),
        };
        draw_dome(x, base_y, half_width, COVER_HEIGHT, night(color, near_d));
        draw_dome(x - 8.0, base_y - COVER_HEIGHT * 0.45, half_width * 0.5, COVER_HEIGHT * 0.34, night(light, near_d));
    }
    draw_house(world.ground_y, d);
}

/// 분화구 빛, 용암, 뭉게 연기. 용암은 밤에도 어두워지지 않아서 밤 화산이 예쁘게 빛난다.
fn draw_volcano_top(art: &SkyArt, look: &SkyLook, ground_y: f32) {
    let rim_y = ground_y - VOLCANO_RIM;
    let mut canvas = GpuCanvas { art };
    let glow = lerp(GLOW_DAY_ALPHA, GLOW_NIGHT_ALPHA, look.d);
    canvas.image(Tex::Glow, Rect::new(VOLCANO_X - 170.0, rim_y - 150.0, 340.0, 300.0), with_alpha(LAVA, glow));
    for (i, (dx, reach)) in LAVA_STREAMS.into_iter().enumerate() {
        // 줄기 끝이 조금씩 늘었다 줄었다 해서 천천히 흐르는 것처럼 보인다
        let len = VOLCANO_RIM * reach * (1.0 + 0.03 * (look.time * 0.8 + i as f32 * 2.0).sin());
        let (start, middle, end) = (vec2(VOLCANO_X + dx * 0.6, rim_y), vec2(VOLCANO_X + dx * 1.5, rim_y + len * 0.5), vec2(VOLCANO_X + dx * 3.2, rim_y + len));
        for (width, color) in [(LAVA_WIDTH, LAVA), (LAVA_WIDTH * 0.45, LAVA_CORE)] {
            draw_line(start.x, start.y, middle.x, middle.y, width, color);
            draw_line(middle.x, middle.y, end.x, end.y, width, color);
            draw_circle(middle.x, middle.y, width / 2.0, color);
            draw_circle(end.x, end.y, width / 2.0, color);
        }
    }
    draw_ellipse(VOLCANO_X, rim_y, 58.0, 10.0, 0.0, LAVA);
    draw_ellipse(VOLCANO_X, rim_y, 40.0, 5.0, 0.0, LAVA_CORE);
    for i in 0..SMOKE_PUFFS {
        let age = (look.time * 0.07 + i as f32 / SMOKE_PUFFS as f32).fract();
        let size = 60.0 + age * 110.0;
        let x = VOLCANO_X + (age * 3.0 + i as f32).sin() * (8.0 + age * 34.0);
        let y = rim_y - 14.0 - age * SMOKE_RISE;
        let alpha = SMOKE_ALPHA * (1.0 - age) * ramp(age, 0.0, 0.12);
        let tint = with_alpha(night(SMOKE, look.d * 0.7), alpha);
        canvas.image(Tex::Cloud(i as usize), Rect::new(x - size / 2.0, y - size * 0.3, size, size * 0.6), tint);
    }
}

/// 둥근 바위: 아래가 땅에 묻힌 납작한 동그라미 + 밝은 윗부분.
fn draw_rock(c: Vec2, r: f32, d: f32) {
    draw_ellipse(c.x, c.y, r * 1.15, r * 0.85, 0.0, night(ROCK, d));
    draw_ellipse(c.x - r * 0.2, c.y - r * 0.3, r * 0.6, r * 0.32, 0.0, night(ROCK_LIGHT, d));
}

/// 세모 세 개를 쌓은 뾰족나무. `base`는 줄기 아래 가운데.
fn draw_pine(base: Vec2, s: f32, d: f32) {
    draw_rectangle(base.x - 6.0 * s, base.y - 20.0 * s, 12.0 * s, 22.0 * s, night(TRUNK, d));
    for (i, (y, half)) in [(16.0, 38.0), (44.0, 31.0), (70.0, 23.0)].into_iter().enumerate() {
        let color = night(if i % 2 == 0 { PINE } else { PINE_LIGHT }, d);
        let low = base.y - y * s;
        draw_triangle(vec2(base.x - half * s, low), vec2(base.x + half * s, low), vec2(base.x, low - 46.0 * s), color);
    }
}

/// 동글동글 나무: 줄기 + 잎 동그라미 셋.
fn draw_round_tree(base: Vec2, s: f32, d: f32) {
    draw_rectangle(base.x - 8.0 * s, base.y - 46.0 * s, 16.0 * s, 48.0 * s, night(TRUNK, d));
    for (dx, dy, r, color) in [(-22.0, 56.0, 28.0, LEAF), (22.0, 56.0, 28.0, LEAF), (0.0, 84.0, 34.0, LEAF_LIGHT)] {
        draw_circle(base.x + dx * s, base.y - dy * s, r * s, night(color, d));
    }
}

fn draw_fence(ground_y: f32, d: f32) {
    let color = night(FENCE, d);
    let (from, to) = FENCE_FROM_TO;
    draw_rectangle(from, ground_y - FENCE_HEIGHT * 0.62, to - from, 6.0, color);
    let mut x = from;
    while x <= to {
        draw_rectangle(x - 5.0, ground_y - FENCE_HEIGHT, 10.0, FENCE_HEIGHT, color);
        draw_triangle(vec2(x - 5.0, ground_y - FENCE_HEIGHT), vec2(x + 5.0, ground_y - FENCE_HEIGHT), vec2(x, ground_y - FENCE_HEIGHT - 8.0), color);
        x += FENCE_EVERY;
    }
}

/// 빨간 지붕 집. 밤에는 창문에 노란 불이 켜진다.
fn draw_house(ground_y: f32, d: f32) {
    let amount = d * HOUSE_NIGHT_AMOUNT;
    let (left, right, top) = (HOUSE_LEFT, HOUSE_RIGHT, ground_y - HOUSE_WALL_HEIGHT);
    let (w, mid) = (right - left, (left + right) / 2.0);
    let line = night(LINE, amount);
    // 굴뚝은 지붕 뒤
    draw_rectangle(left + 56.0, top - 92.0, 36.0, 92.0, night(ROOF, amount));
    draw_rectangle_lines(left + 56.0, top - 92.0, 36.0, 92.0, 4.0, line);
    draw_rectangle(left, top, w, HOUSE_WALL_HEIGHT, night(WALL, amount));
    draw_rectangle_lines(left, top, w, HOUSE_WALL_HEIGHT, 4.0, line);
    let (a, b, apex) = (vec2(left - HOUSE_ROOF_OVERHANG, top), vec2(right + HOUSE_ROOF_OVERHANG, top), vec2(mid, top - HOUSE_ROOF_HEIGHT));
    draw_triangle(a, b, apex, night(ROOF, amount));
    draw_triangle_lines(a, b, apex, 4.0, line);
    // 문과 손잡이
    let door = Rect::new(mid - 34.0, ground_y - 112.0, 68.0, 112.0);
    draw_rectangle(door.x, door.y, door.w, door.h, night(DOOR, amount));
    draw_rectangle_lines(door.x, door.y, door.w, door.h, 4.0, line);
    draw_circle(door.x + door.w - 14.0, door.y + door.h * 0.55, 5.0, night(WINDOW_NIGHT, amount));
    // 창문 둘: 밤에는 어두워지는 대신 불이 켜진다
    let window = mix(night(WINDOW_DAY, amount), WINDOW_NIGHT, ramp(d, 0.35, 0.75));
    for x in [left + 34.0, right - 34.0 - 64.0] {
        draw_rectangle(x, top + 34.0, 64.0, 64.0, window);
        draw_rectangle_lines(x, top + 34.0, 64.0, 64.0, 4.0, line);
        draw_line(x + 32.0, top + 34.0, x + 32.0, top + 98.0, 3.0, line);
        draw_line(x, top + 66.0, x + 64.0, top + 66.0, 3.0, line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAP_WIDTH: f32 = PLACE_WIDTH * 4.0;

    #[test]
    fn colors_change_smoothly_across_places() {
        // US-050: 장소가 바뀌는 곳에서 띠 한 칸 사이 색이 튀지 않는다.
        for table in [&GROUND_TOP, &GROUND_MID, &GROUND_DEEP, &FAR_TOP, &FAR_BOTTOM, &NEAR_TOP, &NEAR_BOTTOM] {
            let mut x = 0.0;
            while x < MAP_WIDTH {
                let (a, b) = (place_color(table, x), place_color(table, x + STRIP_STEP));
                let jump = (a.r - b.r).abs().max((a.g - b.g).abs()).max((a.b - b.b).abs());
                assert!(jump < 0.03, "x={x} jump={jump}");
                x += STRIP_STEP;
            }
        }
    }

    #[test]
    fn each_place_keeps_its_own_color_in_the_middle() {
        for (i, c) in GROUND_MID.iter().enumerate() {
            assert_eq!(place_color(&GROUND_MID, (i as f32 + 0.5) * PLACE_WIDTH), *c);
        }
    }

    #[test]
    fn land_hides_the_sea_where_it_is_not_drawn() {
        // 바다를 건너뛰는 곳부터 맵 끝까지 먼 땅이 수평선 위로 나와 있어야 빈 곳이 안 보인다.
        let ground_y = crate::config::GROUND_Y;
        let mut x = LAND_FULL;
        while x <= MAP_WIDTH + VIRTUAL_WIDTH {
            assert!(ground_y - far_height(x) < HORIZON_Y, "x={x}");
            x += STRIP_STEP;
        }
        assert_eq!(far_height(LAND_START), 0.0);
        assert!(sea_visible(0.0) && !sea_visible(LAND_FULL));
    }

    #[test]
    fn peaks_stay_below_the_info_panel() {
        // 정보판은 y 66까지. 화산·산 꼭대기가 그 아래에 있다.
        let ground_y = crate::config::GROUND_Y;
        let highest = (0..MAP_WIDTH as u32).map(|x| far_height(x as f32)).fold(0.0, f32::max);
        assert!(ground_y - highest > 66.0, "{highest}");
    }

    #[test]
    fn snow_is_only_on_the_mountains() {
        assert_eq!(snow_bottom(VOLCANO_X, 178.0), 178.0);
        assert!(snow_bottom(MOUNTAINS[1].0, 148.0) >= SNOW_Y);
        assert_eq!(snow_bottom(3000.0, 300.0), 300.0);
    }
}
