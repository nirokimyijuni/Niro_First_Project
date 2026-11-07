//! 화면 그리기: 타이틀, 캐릭터 고르기, 플레이. 좌표는 모두 가상 화면(960×540) 기준.
//! 배경: 화면에 고정된 하늘·바다 빛(sky.rs) → 맵을 따라 움직이는 땅(places.rs)과 해변 물결. 바다는 그림일 뿐 규칙은 없다.
//! 플레이 중에는 logic 시계의 `sky_darkness`(0 한낮 ~ 1 한밤)로 낮 그림과 밤 그림을 부드럽게 섞는다
//! (D9, D10): 노을 때 하늘 아래가 주황·분홍, 밤에는 남색 하늘에 별. 해·달은 시계를 따라 호를 그린다 (D31).

use super::assets::{height_with_antenna, Assets};
use super::coin::draw_coins;
use super::effects::{draw_bubble, Effects};
use super::gameover::{draw_down_panel, draw_fallen_player, DownScreen};
use super::hud::{draw_health, draw_hud, draw_level, draw_weapons};
use super::places;
use super::screen::{Letterbox, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::sky::{self, mix, night, Canvas, GpuCanvas, SkyArt, SkyLook};
use super::sprites::{self, draw_character, draw_projectile, fit_size, flip_weapon, max_half_width, PLAY_SPRITE_HEIGHT, ZOMBIE_SPRITE_HEIGHT};
use super::text::TextPainter;
use crate::config::CITIZEN_HELP_TEXT;
use crate::logic::character::CharacterKind;
use crate::logic::game::Game;
use crate::logic::monster::ZombieState;
use crate::logic::player::{Facing, Motion};
use crate::logic::skeleton::SkeletonState;
use crate::logic::weapon::Weapon;
use macroquad::prelude::*;

// ---- 색 ----
/// 타이틀 구석 버전 글자 (모래 위에 작고 흐리게)
const VERSION_COLOR: Color = Color::new(0.35, 0.25, 0.15, 0.7);
/// 모래 알갱이: 어두운 알갱이와 밝은 알갱이 (반투명)
const SAND_DOT: Color = Color::new(0.62, 0.47, 0.28, 0.45);
const SAND_SPARK: Color = Color::new(1.0, 0.95, 0.82, 0.55);
const SAND_DOT_COUNT: u32 = 90;
const SHELL: Color = Color::new(1.0, 0.72, 0.74, 1.0);
const SHELL_LINE: Color = Color::new(0.80, 0.45, 0.50, 1.0);
const FOAM: Color = Color::new(1.0, 1.0, 1.0, 0.95);
/// 거품 아래 모래 위로 얇게 번지는 물 (반투명)
const WASH: Color = Color::new(0.85, 0.95, 1.0, 0.35);
const WOOD: Color = Color::new(0.80, 0.56, 0.30, 1.0);
const WOOD_DARK: Color = Color::new(0.52, 0.33, 0.15, 1.0);
const WOOD_EDGE: Color = Color::new(0.30, 0.18, 0.08, 1.0);
/// 글자는 흰색 + 진한 테두리 (text.rs)
const TEXT_COLOR: Color = WHITE;
const CARD: Color = Color::new(1.0, 1.0, 1.0, 0.55);
const CARD_PICKED: Color = Color::new(1.0, 0.98, 0.85, 0.9);
const PICK_BORDER: Color = Color::new(1.0, 0.70, 0.10, 1.0);

/// 밤에 캐릭터에 곱하는 색. 배경보다 훨씬 덜 어둡게 해서 캐릭터가 잘 보이게.
const CHARACTER_NIGHT_TINT: Color = Color::new(0.82, 0.84, 0.97, 1.0);
/// 맞은 좀비가 번쩍이는 색
const ZOMBIE_FLASH: Color = Color::new(1.0, 0.95, 0.45, 0.85);
/// 밤에 얼마나 어두워지는지 (1 = 밤빛 그대로, 0 = 그대로). 상자는 덜 어둡게.
const BOX_NIGHT_AMOUNT: f32 = 0.7;
const FOAM_NIGHT_AMOUNT: f32 = 0.7;
const SAND_NIGHT_AMOUNT: f32 = 0.85;

/// 젖은 모래가 마른 모래로 바뀌는 폭
const WET_SAND_THICKNESS: f32 = 16.0;

// ---- 캐릭터 고르기 화면 배치 ----
const CARD_CENTERS_X: [f32; 3] = [240.0, 480.0, 720.0];
const CARD_WIDTH: f32 = 220.0;
const CARD_TOP: f32 = 110.0;
const CARD_HEIGHT: f32 = 340.0;
const CARD_FEET_Y: f32 = 370.0;
/// 안테나를 뺀 몸 높이. 가장 큰 키(고른 Ellie)도 카드 위로 넘치지 않게 맞췄다.
const CARD_SPRITE_HEIGHT: f32 = 170.0;
/// 골라진 캐릭터는 이만큼 크게
const PICKED_GROW: f32 = 1.15;
/// "캐릭터를 골라요" 바닥선. 고른 카드 위 주황 세모와 겹치지 않게 위로 올렸다 (BUG-002).
const SELECT_TITLE_Y: f32 = 62.0;
/// 주황 세모 끝이 카드 위쪽에서 얼마나 떨어져 있는지, 높이, 통통 튀는 폭
const PICK_ARROW_GAP: f32 = 12.0;
const PICK_ARROW_HEIGHT: f32 = 20.0;
const PICK_ARROW_BOB: f32 = 4.0;

// ---- 플레이 화면 배치 ----
/// 조작 안내는 화면 아래 모래 위 (위쪽은 정보판 자리)
const PLAY_HINT_Y: f32 = 526.0;
/// 맞은 뒤 안 맞는 동안 캐릭터가 초당 깜빡이는 번, 깜빡일 때 투명도
const HURT_BLINKS_PER_SECOND: f32 = 8.0;
const HURT_BLINK_ALPHA: f32 = 0.35;
/// 미스·시민 그림 높이를 몸 네모의 몇 배로 할지. 안테나가 그림 안에 있어서 조금 크게.
const MISS_SPRITE_SCALE: f32 = 1.3;
/// 손에 든 무기: 크기, 발 가운데에서 앞으로, 발에서 위로
const HELD_WEAPON_SIZE: f32 = 40.0;
const HELD_WEAPON_FORWARD: f32 = 34.0;
const HELD_WEAPON_UP: f32 = 44.0;
/// 땅에 놓인 무기 뒤의 빛 (밤에도 보이게)
const WEAPON_GLOW: Color = Color::new(1.0, 1.0, 0.85, 0.35);
const WEAPON_GLOW_RADIUS: f32 = 24.0;
/// 땅의 무기가 사라지기 이만큼 전부터 깜빡인다 (초), 초당 깜빡임
const WEAPON_BLINK_FROM: f32 = 3.0;
const WEAPON_BLINKS_PER_SECOND: f32 = 4.0;
/// 일반 시민 말풍선 꼬리 끝이 머리 위로 떨어진 거리
const CITIZEN_BUBBLE_GAP: f32 = 22.0;
/// 멈춘 일반 시민이 고개를 돌리는 간격 (초). 멈추는 2초 동안 두어 번 두리번거린다.
const CITIZEN_LOOK_SECONDS: f32 = 0.6;

/// 캐릭터 고르기 화면에서 `index`번째 카드 네모.
pub fn select_card_rect(index: usize) -> Rect {
    Rect::new(CARD_CENTERS_X[index] - CARD_WIDTH / 2.0, CARD_TOP, CARD_WIDTH, CARD_HEIGHT)
}

/// 누른 곳(가상 좌표)이 몇 번째 카드인지. 카드 밖이면 None.
pub fn select_card_at(point: Vec2) -> Option<usize> {
    (0..CharacterKind::ALL.len()).find(|&i| select_card_rect(i).contains(point))
}

fn time() -> f32 {
    get_time() as f32
}

// ---- 배경 (해변) ----

/// 바다와 모래가 만나는 곳의 하얀 물결: 둥근 거품이 천천히 밀려왔다 빠진다.
fn draw_foam(ground_y: f32, d: f32) {
    let foam = night(FOAM, d * FOAM_NIGHT_AMOUNT);
    let surge = (time() * 0.9).sin() * 5.0; // 밀려왔다(아래) 빠졌다(위)
    let mut x = -20.0;
    let mut i = 0;
    while x < places::BEACH_END {
        let wobble = (x * 0.03 + time() * 1.6).sin() * 3.0;
        let y = ground_y - 6.0 + surge + wobble;
        draw_circle(x, y, 9.0 + (i % 3) as f32 * 2.0, foam);
        x += 16.0;
        i += 1;
    }
    draw_rectangle(0.0, ground_y - 6.0 + surge, places::BEACH_END, 6.0, foam);
}

/// 아래가 평평한 반원 (부채꼴 조개 몸통). 모래 위에 덧칠하지 않고 반원만 그린다.
fn draw_half_disc_up(c: Vec2, r: f32, color: Color) {
    const STEPS: usize = 14;
    for i in 0..STEPS {
        let a0 = std::f32::consts::PI * (1.0 + i as f32 / STEPS as f32);
        let a1 = std::f32::consts::PI * (1.0 + (i + 1) as f32 / STEPS as f32);
        draw_triangle(c, c + vec2(a0.cos(), a0.sin()) * r, c + vec2(a1.cos(), a1.sin()) * r, color);
    }
}

fn draw_shell(c: Vec2, s: f32, d: f32) {
    let (shell, line) = (night(SHELL, d), night(SHELL_LINE, d));
    // 부채 모양 조개: 반원 + 줄무늬 + 아래 꼭지
    draw_half_disc_up(c, 9.0 * s, shell);
    draw_triangle(c + vec2(-4.0, 0.0) * s, c + vec2(4.0, 0.0) * s, c + vec2(0.0, 5.0) * s, shell);
    for dx in [-5.0, 0.0, 5.0] {
        draw_line(c.x, c.y + 3.0 * s, c.x + dx * s, c.y - 7.0 * s, 1.5, line);
    }
}

/// 해변 위 그림: 알갱이·조개 → 물결 거품. 맵 좌표로 그린다.
fn draw_beach(art: &SkyArt, look: &SkyLook, ground_y: f32) {
    let d = look.d;
    let mut canvas = GpuCanvas { art };
    // 모래는 조금 덜 어둡게: 캐릭터가 서 있는 바닥이 너무 깜깜하지 않게
    let sd = d * SAND_NIGHT_AMOUNT;
    // 모래 알갱이: 크기·밝기가 제각각인 작은 점 (늘 같은 자리)
    for i in 0..SAND_DOT_COUNT {
        let x = hash01(i * 3) * VIRTUAL_WIDTH;
        let y = ground_y + WET_SAND_THICKNESS + 4.0 + hash01(i * 3 + 1) * (VIRTUAL_HEIGHT - ground_y - WET_SAND_THICKNESS - 6.0);
        let r = 0.8 + hash01(i * 3 + 2) * 1.8;
        let c = if i % 4 == 0 { SAND_SPARK } else { SAND_DOT };
        draw_circle(x, y, r, night(c, sd));
    }
    // 조개 몇 개
    for (x, y, s) in [(90.0, 488.0, 1.0), (430.0, 520.0, 0.8), (650.0, 478.0, 1.1), (900.0, 505.0, 0.9)] {
        draw_shell(vec2(x, y), s, sd);
    }
    // 거품 아래로 얇게 번지는 물
    let surge = (time() * 0.9).sin() * 5.0;
    let wash = night(WASH, d * FOAM_NIGHT_AMOUNT);
    canvas.vgradient(0.0, places::BEACH_END, &[(ground_y - 2.0 + surge, wash), (ground_y + 14.0 + surge, Color { a: 0.0, ..wash })]);
    draw_foam(ground_y, d);
}

/// 번호 → 0 이상 1 미만의 늘 같은 수 (모래 알갱이 자리)
fn hash01(n: u32) -> f32 {
    let mut x = n.wrapping_mul(0x2C1B_3C6D) ^ 0x297A_2D39;
    x ^= x >> 15;
    x = x.wrapping_mul(0x8EBC_6AF1);
    x ^= x >> 13;
    (x >> 8) as f32 / (1u32 << 24) as f32
}

/// 나무 상자: 테두리 + 가로 판자 줄 + X자 버팀목. 밤에는 배경보다 덜 어둡게.
fn draw_wood_box(r: Rect, d: f32) {
    let amount = d * BOX_NIGHT_AMOUNT;
    let (wood, dark) = (night(WOOD, amount), night(WOOD_DARK, amount));
    let edge = (r.w.min(r.h) * 0.12).max(4.0);
    draw_rectangle(r.x, r.y, r.w, r.h, wood);
    draw_line(r.x + edge, r.y + edge, r.x + r.w - edge, r.y + r.h - edge, edge * 0.8, dark);
    draw_line(r.x + r.w - edge, r.y + edge, r.x + edge, r.y + r.h - edge, edge * 0.8, dark);
    draw_rectangle_lines(r.x + edge / 2.0, r.y + edge / 2.0, r.w - edge, r.h - edge, edge, dark);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, 2.0, night(WOOD_EDGE, amount));
}

/// 화면에 고정된 하늘·바다 → 카메라를 걸고 맵의 땅. 타이틀·고르기·플레이가 모두 같은 하늘을 쓴다.
/// 끝나면 카메라가 `camera_x`에 걸려 있다.
fn draw_background(assets: &Assets, look: &SkyLook, ground_y: f32, camera_x: f32) {
    let mut canvas = GpuCanvas { art: &assets.sky };
    sky::draw_sky(&mut canvas, look);
    if places::sea_visible(camera_x) {
        sky::draw_sea(&mut canvas, look, ground_y);
    }
    Letterbox::current().look_at(camera_x);
    places::draw_ground_band(look, ground_y, camera_x);
    if camera_x < places::BEACH_END {
        draw_beach(&assets.sky, look, ground_y);
    }
    places::draw_land(&assets.sky, look, ground_y, camera_x);
}

// ---- 타이틀 ----

/// 타이틀. `version`이 있으면(웹 배포판) 오른쪽 아래 구석에 작게 보여 준다 (US-031).
pub fn draw_title(assets: &Assets, text: &TextPainter, ground_y: f32, version: Option<&str>) {
    draw_background(assets, &SkyLook::menu(time()), ground_y, 0.0);
    for (i, kind) in CharacterKind::ALL.into_iter().enumerate() {
        let pose = sprites::pose(Motion::Idle, time() + i as f32 * 0.37, false);
        draw_character(assets.character(kind), vec2(CARD_CENTERS_X[i], ground_y + 4.0), height_with_antenna(kind, 150.0), Facing::Right, pose, WHITE);
    }
    // 제목: 하늘 부분에서 살짝 둥실둥실. 자모마다 아이가 정한 색 (view/title.rs)
    let bob = (time() * 2.0).sin() * 6.0;
    assets.title.draw(VIRTUAL_WIDTH / 2.0, 150.0 + bob, 120.0);
    // 안내 문구는 천천히 깜빡
    let alpha = 0.65 + 0.35 * (time() * 3.0).sin();
    let hint = Color { a: alpha, ..TEXT_COLOR };
    text.centered("시작하려면 Space/Enter 또는 화면을 눌러요", VIRTUAL_WIDTH / 2.0, 240.0, 38.0, hint);
    if let Some(v) = version {
        let label = format!("v {v}");
        let size = 18.0;
        let w = text.width(&label, size);
        text.left(&label, VIRTUAL_WIDTH - w - 10.0, VIRTUAL_HEIGHT - 10.0, size, VERSION_COLOR);
    }
}

// ---- 캐릭터 고르기 ----

pub fn draw_select(assets: &Assets, text: &TextPainter, ground_y: f32, picked: usize) {
    draw_background(assets, &SkyLook::menu(time()), ground_y, 0.0);
    text.centered("캐릭터를 골라요", VIRTUAL_WIDTH / 2.0, SELECT_TITLE_Y, 56.0, TEXT_COLOR);

    for (i, kind) in CharacterKind::ALL.into_iter().enumerate() {
        let r = select_card_rect(i);
        let is_picked = i == picked;
        if is_picked {
            let pad = 8.0;
            draw_rectangle(r.x - pad, r.y - pad, r.w + pad * 2.0, r.h + pad * 2.0, CARD_PICKED);
            draw_rectangle_lines(r.x - pad, r.y - pad, r.w + pad * 2.0, r.h + pad * 2.0, 8.0, PICK_BORDER);
            // 위에서 콕콕 가리키는 세모
            let bob = (time() * 5.0).sin() * PICK_ARROW_BOB;
            let tip = vec2(r.x + r.w / 2.0, r.y - PICK_ARROW_GAP + bob);
            draw_triangle(
                tip,
                tip + vec2(-18.0, -PICK_ARROW_HEIGHT),
                tip + vec2(18.0, -PICK_ARROW_HEIGHT),
                PICK_BORDER,
            );
        } else {
            draw_rectangle(r.x, r.y, r.w, r.h, CARD);
        }
        let height = if is_picked { CARD_SPRITE_HEIGHT * PICKED_GROW } else { CARD_SPRITE_HEIGHT };
        let pose = sprites::pose(Motion::Idle, time() + i as f32 * 0.37, false);
        draw_character(assets.character(kind), vec2(r.x + r.w / 2.0, CARD_FEET_Y), height_with_antenna(kind, height), Facing::Right, pose, WHITE);
        let name_size = if is_picked { 50.0 } else { 42.0 };
        text.centered(kind.name(), r.x + r.w / 2.0, r.y + r.h - 22.0, name_size, TEXT_COLOR);
    }

    text.centered("화살표로 고르고 Space/Enter, 또는 캐릭터를 눌러요", VIRTUAL_WIDTH / 2.0, 515.0, 30.0, TEXT_COLOR);
}

// ---- 플레이 ----

/// 그림을 그릴 발 x: 충돌 네모 가운데지만, 그림(옆으로 뻗은 팔 포함)이 맵 밖으로
/// 나가지 않게 맵 안으로 당긴다 (BUG-001). 규칙(충돌 네모)은 그대로다.
fn feet_x_on_map(center_x: f32, half_width: f32, map_width: f32) -> f32 {
    center_x.clamp(half_width, (map_width - half_width).max(half_width))
}

/// 좀비 카스테라: 플레이어 쪽을 보고, 걸을 때 통통, 떠오르면 살랑살랑, 맞으면 번쩍.
/// 쓰러진 화면에서는 멈춘다 (US-043). 밤 어두움은 주인공처럼 살짝만 받는다.
fn draw_zombies(assets: &Assets, game: &Game, tint: Color) {
    let frozen = game.combat.health.is_down();
    let player_x = game.player.rect().center().x;
    let half = max_half_width(&assets.zombie, ZOMBIE_SPRITE_HEIGHT);
    for (i, zombie) in game.combat.zombies.list.iter().enumerate() {
        let r = zombie.rect();
        let t = if frozen { 0.0 } else { time() + i as f32 * 0.37 };
        let pose = match zombie.state {
            ZombieState::Walking => sprites::pose(Motion::Walk, t, false),
            ZombieState::Floating => sprites::float_pose(t),
        };
        let facing = if player_x < r.center().x { Facing::Left } else { Facing::Right };
        let feet = vec2(feet_x_on_map(r.center().x, half, game.world.width), r.y + r.h);
        draw_character(&assets.zombie, feet, ZOMBIE_SPRITE_HEIGHT, facing, pose, tint);
        if zombie.is_flashing() {
            draw_character(&assets.zombie_flash, feet, ZOMBIE_SPRITE_HEIGHT, facing, pose, ZOMBIE_FLASH);
        }
    }
}

/// 스켈레톤: 플레이어 쪽을 보고, 걸을 때 통통, 서 있으면 숨쉬기, 피할 때 뒤로 젖히고, 맞으면 번쩍.
/// 좀비와 같은 밤 색을 받고, 쓰러진 화면에서는 멈춘다.
fn draw_skeletons(assets: &Assets, game: &Game, tint: Color) {
    let frozen = game.combat.health.is_down();
    let player_x = game.player.rect().center().x;
    for (i, skeleton) in game.combat.skeletons.list.iter().enumerate() {
        let (picture, flash) = assets.skeleton(skeleton.stage);
        let height = sprites::skeleton_sprite_height(skeleton.stage);
        let r = skeleton.rect();
        let t = if frozen { 0.0 } else { time() + i as f32 * 0.37 };
        let player_on_left = player_x < r.center().x;
        let pose = match skeleton.state {
            SkeletonState::Walking => sprites::pose(Motion::Walk, t, false),
            SkeletonState::Standing => sprites::pose(Motion::Idle, t, false),
            SkeletonState::Dodging => {
                let lean = if player_on_left { sprites::DODGE_LEAN } else { -sprites::DODGE_LEAN };
                sprites::Pose { tilt: lean, ..sprites::pose(Motion::Jump, t, true) }
            }
        };
        // `draw_character`는 Left일 때 뒤집으니, 왼쪽을 보는 그림은 반대로 준다.
        let facing = if player_on_left != sprites::skeleton_art_faces_left(skeleton.stage) { Facing::Left } else { Facing::Right };
        let feet = vec2(feet_x_on_map(r.center().x, max_half_width(picture, height), game.world.width), r.y + r.h);
        draw_character(picture, feet, height, facing, pose, tint);
        if skeleton.is_flashing() {
            draw_character(flash, feet, height, facing, pose, ZOMBIE_FLASH);
        }
    }
}

/// 멈춘 일반 시민이 `t`초에 보는 쪽: 왼쪽·오른쪽을 번갈아 본다.
fn looking_around(t: f32) -> Facing {
    if (t / CITIZEN_LOOK_SECONDS) as u32 % 2 == 0 {
        Facing::Left
    } else {
        Facing::Right
    }
}

/// 미스·시민: 네모 아래 끝을 발로 두고 숨쉬기만 한다.
fn draw_miss_like(texture: &Texture2D, r: Rect, t: f32, tint: Color) {
    let feet = vec2(r.center().x, r.y + r.h);
    draw_character(texture, feet, r.h * MISS_SPRITE_SCALE, Facing::Right, sprites::pose(Motion::Idle, t, false), tint);
}

/// 하늘에서 떨어지는 무기와 놓인 무기. 놓이면 뒤에 빛이 깜빡여서 밤에도 찾기 쉽다.
fn draw_dropped_weapons(assets: &Assets, game: &Game) {
    for w in &game.weapons.dropped {
        if w.seconds_left() <= WEAPON_BLINK_FROM && (time() * WEAPON_BLINKS_PER_SECOND).fract() < 0.5 {
            continue;
        }
        let c = w.rect.center();
        if !w.is_falling() {
            let glow = Color { a: WEAPON_GLOW.a * (0.7 + 0.3 * (time() * 4.0).sin()), ..WEAPON_GLOW };
            draw_circle(c.x, c.y, WEAPON_GLOW_RADIUS, glow);
        }
        if let Some(texture) = assets.m6.weapon(w.kind) {
            let size = fit_size(texture, w.rect.w.max(w.rect.h));
            draw_texture_ex(texture, c.x - size.x / 2.0, c.y - size.y / 2.0, WHITE, DrawTextureParams { dest_size: Some(size), ..Default::default() });
        }
    }
}

/// 손에 든 무기. 맨손이면 아무것도 없고, 왼쪽을 보면 함께 뒤집힌다.
fn draw_held_weapon(assets: &Assets, weapon: Weapon, feet: Vec2, facing: Facing, tint: Color) {
    let Some(texture) = assets.m6.weapon(weapon) else { return };
    let side = if facing == Facing::Right { 1.0 } else { -1.0 };
    let size = fit_size(texture, HELD_WEAPON_SIZE);
    let c = feet + vec2(side * HELD_WEAPON_FORWARD, -HELD_WEAPON_UP);
    draw_texture_ex(texture, c.x - size.x / 2.0, c.y - size.y / 2.0, tint, DrawTextureParams { dest_size: Some(size), flip_x: flip_weapon(weapon, facing), ..Default::default() });
}

/// `touch_mode`면 키보드 안내 글자는 숨긴다 (터치 조이스틱·버튼은 main이 위에 그린다).
/// `down`이 있으면 쓰러진 화면이다.
pub fn draw_play(
    assets: &Assets,
    text: &TextPainter,
    game: &Game,
    fast_forward: bool,
    effects: &Effects,
    down: Option<&DownScreen>,
    touch_mode: bool,
) {
    let d = game.clock.sky_darkness();
    // 여기부터 맵 좌표: 아래 `look_at(0.0)`까지는 카메라를 따라 움직인다.
    draw_background(assets, &SkyLook::play(d, game.clock.time_of_day_minutes(), time()), game.world.ground_y, game.camera_left());
    places::draw_structures(&game.world, d);
    // 미스 → 가리는 앞그림 → 상자. 상자가 먼저면 집 앞 상자가 집에 묻힌다.
    if let Some(miss) = game.miss.as_ref().filter(|m| m.is_hiding()) {
        draw_miss_like(&assets.m6.miss, miss.rect(&game.world), time(), WHITE);
    }
    places::draw_covers(&game.world, d);
    for b in game.world.boxes() {
        draw_wood_box(b, d);
    }
    draw_coins(&game.coins, d, time());

    let player = &game.player;
    let texture = assets.character(player.kind);
    let r = player.rect();
    let height = height_with_antenna(player.kind, PLAY_SPRITE_HEIGHT);
    let half = max_half_width(texture, height);
    let feet = vec2(feet_x_on_map(r.x + r.w / 2.0, half, game.world.width), r.y + r.h);
    let tint = mix(WHITE, CHARACTER_NIGHT_TINT, d);
    for (i, citizen) in game.citizens.list.iter().enumerate() {
        if citizen.is_standing() {
            draw_miss_like(&assets.m6.miss_citizen_bomb, citizen.rect, time() + i as f32 * 0.37, tint);
        }
    }
    for (i, walker) in game.citizens.walkers.iter().enumerate() {
        let r = walker.rect;
        let motion = if walker.is_walking() { Motion::Walk } else { Motion::Idle };
        // 쓰러진 화면에서는 시간을 멈춰서 숨쉬기 흔들림도 멈춘다
        let t = if down.is_some() { 0.0 } else { time() + i as f32 * 0.37 };
        let pose = sprites::pose(motion, t, false);
        let facing = if !walker.is_walking() && down.is_none() { looking_around(t) } else { walker.facing };
        draw_character(&assets.m6.miss, vec2(r.center().x, r.y + r.h), r.h * MISS_SPRITE_SCALE, facing, pose, tint);
    }
    draw_dropped_weapons(assets, game);
    draw_zombies(assets, game, tint);
    draw_skeletons(assets, game, tint);
    if let Some(down) = down {
        draw_fallen_player(texture, player, &game.world, feet.x, height, tint, down.seconds);
    } else {
        let blink = game.combat.health.is_invincible() && (time() * HURT_BLINKS_PER_SECOND).fract() < 0.5;
        let player_tint = Color { a: if blink { HURT_BLINK_ALPHA } else { 1.0 }, ..tint };
        let pose = sprites::pose(player.motion, time(), player.vel.y < 0.0);
        draw_character(texture, feet, height, player.facing, pose, player_tint);
        if !effects.swinging_blade() {
            draw_held_weapon(assets, game.weapons.held(), feet, player.facing, player_tint);
        }
    }

    game.combat.skeletons.projectiles.iter().for_each(draw_projectile);
    effects.draw(text, r, player.facing, &assets.m6, game.camera_left());
    if down.is_none() {
        for (i, walker) in game.citizens.walkers.iter().enumerate() {
            let head = vec2(walker.rect.center().x, walker.rect.y - CITIZEN_BUBBLE_GAP);
            if !walker.is_walking() && effects.citizen_talks(i) && game.world.is_on_screen(head.x, r.center().x) {
                draw_bubble(text, CITIZEN_HELP_TEXT, head.x, head.y, game.camera_left(), 1.0);
            }
        }
    }

    // 여기부터 화면 좌표: 정보판·안내는 걸어도 같은 자리다. main이 이어서 그리는 터치 버튼도 같다.
    Letterbox::current().look_at(0.0);
    effects.draw_on_screen(text);
    draw_hud(text, &game.clock, game.wallet.coins(), fast_forward, game.place_name());
    draw_health(text, game.combat.health.hp(), game.combat.health.max_hp(), effects.heart_sparkle());
    draw_level(text, game.combat.level, game.combat.experience);
    let owned = game.weapons.owned();
    draw_weapons(&assets.m6, &owned, game.weapons.held());
    if !touch_mode && down.is_none() {
        let hint = if owned.is_empty() { "화살표: 걷기   Space: 점프   Q: 공격   Esc: 캐릭터 고르기" } else { "화살표: 걷기   Space: 점프   Q: 공격   W: 바꾸기   Esc: 고르기" };
        text.centered(hint, VIRTUAL_WIDTH / 2.0, PLAY_HINT_Y, 26.0, TEXT_COLOR);
    }
    if let Some(down) = down {
        draw_down_panel(text, down);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tapping_a_card_picks_that_character() {
        for i in 0..3 {
            let r = select_card_rect(i);
            assert_eq!(select_card_at(r.center()), Some(i));
        }
        assert_eq!(select_card_at(vec2(10.0, 10.0)), None);
    }

    #[test]
    fn cards_fit_on_screen() {
        for i in 0..3 {
            let r = select_card_rect(i);
            assert!(r.x >= 0.0 && r.x + r.w <= VIRTUAL_WIDTH && r.y + r.h <= VIRTUAL_HEIGHT);
        }
    }

    #[test]
    fn picked_character_stays_inside_card_top() {
        // REVIEW-005 5번: 숨쉬기로 가장 늘어나도 고른 캐릭터(안테나 포함)가 카드 위로 넘치지 않는다.
        for kind in CharacterKind::ALL {
            let top = CARD_FEET_Y - height_with_antenna(kind, CARD_SPRITE_HEIGHT * PICKED_GROW) * (1.0 + sprites::IDLE_STRETCH);
            assert!(top >= CARD_TOP, "{kind:?} 맨 위 {top}");
        }
    }

    #[test]
    fn sprite_is_pulled_inside_map_edges() {
        // BUG-001: 반폭 40인 그림은 발 x가 맵 양 끝에서 40 안쪽에 있어야 잘리지 않는다.
        assert_eq!(feet_x_on_map(24.0, 40.0, 3840.0), 40.0);
        assert_eq!(feet_x_on_map(3840.0 - 24.0, 40.0, 3840.0), 3840.0 - 40.0);
        assert_eq!(feet_x_on_map(2000.0, 40.0, 3840.0), 2000.0);
    }

    #[test]
    fn stopped_citizen_looks_both_ways() {
        assert_eq!(looking_around(0.0), Facing::Left);
        assert_eq!(looking_around(CITIZEN_LOOK_SECONDS), Facing::Right);
        assert_eq!(looking_around(CITIZEN_LOOK_SECONDS * 2.0), Facing::Left);
    }

    #[test]
    fn pick_arrow_stays_below_select_title() {
        // BUG-002: 세모가 가장 높이 튀어도 그 위 끝이 제목 바닥선보다 아래에 있다.
        let arrow_top = CARD_TOP - PICK_ARROW_GAP - PICK_ARROW_BOB - PICK_ARROW_HEIGHT;
        assert!(arrow_top > SELECT_TITLE_Y + 4.0, "세모 위 끝 {arrow_top}");
    }

    #[test]
    fn horizon_is_above_ground() {
        assert!(sky::HORIZON_Y < crate::config::GROUND_Y);
        assert!(sky::HORIZON_Y + 40.0 < crate::config::GROUND_Y, "안개가 모래까지 내려오지 않는다");
    }
}
