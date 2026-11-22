//! 화면 그리기: 타이틀, 캐릭터 고르기, 플레이. 좌표는 모두 가상 화면(960×540) 기준.
//! 배경: 화면에 고정된 하늘·바다 빛(sky.rs) → 맵을 따라 움직이는 땅(places.rs)과 해변 물결. 바다는 그림일 뿐 규칙은 없다.
//! 플레이 중에는 logic 시계의 `sky_darkness`(0 한낮 ~ 1 한밤)로 낮 그림과 밤 그림을 부드럽게 섞는다
//! (D9, D10): 노을 때 하늘 아래가 주황·분홍, 밤에는 남색 하늘에 별. 해·달은 시계를 따라 호를 그린다 (D31).

use super::assets::{height_with_antenna, Assets};
use super::boxes;
use super::coin::draw_coins;
use super::effects::{draw_bubble, Effects};
use super::gameover::{draw_end_panel, draw_fallen_player, EndScreen};
use super::gather::{self, GatherView};
use super::bag::{self, BagView};
use super::hud;
use super::mission;
use super::places;
use super::screen::{set_shake, shake, Letterbox, VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::sky::{self, mix, night, Canvas, GpuCanvas, SkyArt, SkyLook};
use super::sprites::{self, draw_character, draw_projectile, fit_size, flip_weapon, PLAY_SPRITE_HEIGHT, ZOMBIE_SPRITE_HEIGHT};
use super::text::TextPainter;
use super::touch;
use super::tutorial;
use super::items;
use super::shop;
use super::skates;
use crate::config::{
    CITIZEN_HELP_TEXT, DEV_FAST_FORWARD_SCALE, PRACTICE_MARK_BOB, PRACTICE_MARK_ABOVE, PRACTICE_MARK_COLOR, PRACTICE_MARK_RADIUS, PRACTICE_MARK_TURN_SPEED, PRACTICE_MONSTER_HEIGHT,
    PRACTICE_MONSTER_STOP_DISTANCE, PRACTICE_TINT, ZOMBIE_HEIGHT,
};
use crate::logic::character::CharacterKind;
use crate::logic::game::Game;
use crate::logic::monster::ZombieState;
use crate::logic::player::{Facing, Motion};
use crate::logic::skeleton::SkeletonState;
use crate::logic::weapon::Weapon;
use crate::logic::world::{nearest_copy, shortest_dx};
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

/// 화면 양옆(또는 위아래)의 남는 띠를 장면에 맞춰 이어 칠한다: 하늘·바다·땅 색을 가장자리에서 늘린다.
/// 창 전체를 기본 카메라(창 좌표)로 칠하고 나서 게임 그림이 가운데를 덮는다. 규칙·터치 좌표와 상관없다.
fn draw_side_fill(look: &SkyLook, ground_y: f32, camera_x: f32) {
    let lb = Letterbox::current();
    if lb.offset.x < 0.5 && lb.offset.y < 0.5 {
        return;
    }
    lb.end();
    let (sw, sh) = (screen_width(), screen_height());
    let sea = places::sea_visible(camera_x);
    let sky = sky::edge_column(look, ground_y, sea);
    let lip = ground_y + places::GROUND_LIP;
    let to_window = |y: f32| lb.offset.y + y * lb.scale;
    // 땅 색은 왼쪽 가장자리 → 오른쪽 가장자리로 가로로 서서히 섞는다 (세로로 긴 창에서 가운데 줄이 안 보이게)
    let left = places::edge_ground(look, camera_x);
    let right = places::edge_ground(look, camera_x + VIRTUAL_WIDTH);
    let mut stops: Vec<(f32, Color, Color)> = vec![(-1.0, sky[0].1, sky[0].1)];
    stops.extend(sky.iter().map(|&(y, c)| (to_window(y), c, c)));
    stops.extend([
        (to_window(ground_y), left[0], right[0]),
        (to_window(lip), left[1], right[1]),
        (to_window(VIRTUAL_HEIGHT), left[2], right[2]),
        (sh + 1.0, left[2], right[2]),
    ]);
    // 창 y가 커지는 순서만 남긴다
    stops.dedup_by(|b, a| b.0 <= a.0);
    let mut vertices = Vec::with_capacity(stops.len() * 2);
    let mut indices = Vec::new();
    for (i, &(y, l, r)) in stops.iter().enumerate() {
        vertices.push(Vertex::new(0.0, y, 0.0, 0.0, 0.0, l));
        vertices.push(Vertex::new(sw, y, 0.0, 1.0, 0.0, r));
        if i > 0 {
            let b = (i as u16 - 1) * 2;
            indices.extend_from_slice(&[b, b + 1, b + 2, b + 1, b + 3, b + 2]);
        }
    }
    draw_mesh(&Mesh { vertices, indices, texture: None });
    // 하늘·바다는 흔들리지 않으므로 흔들기 없이 카메라를 다시 건다
    let keep = shake();
    set_shake(Vec2::ZERO);
    lb.look_at(0.0);
    set_shake(keep);
}

/// 화면에 고정된 하늘·바다 → 카메라를 걸고 맵의 땅. 타이틀·고르기·플레이가 모두 같은 하늘을 쓴다.
/// 끝나면 카메라가 `camera_x`에 걸려 있다.
fn draw_background(assets: &Assets, look: &SkyLook, ground_y: f32, camera_x: f32) {
    draw_side_fill(look, ground_y, camera_x);
    let mut canvas = GpuCanvas { art: &assets.sky };
    sky::draw_sky(&mut canvas, look);
    if places::sea_visible(camera_x) {
        sky::draw_sea(&mut canvas, look, ground_y);
    }
    Letterbox::current().look_at(camera_x);
    places::draw_ground_band(look, ground_y, camera_x);
    places::on_ring_cameras(camera_x, |camera| {
        if camera < places::BEACH_END && camera + VIRTUAL_WIDTH > 0.0 {
            draw_beach(&assets.sky, look, ground_y);
        }
    });
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
fn view_center(game: &Game) -> f32 {
    game.camera_left() + VIRTUAL_WIDTH / 2.0
}

/// 좀비 카스테라: 플레이어 쪽을 보고, 걸을 때 통통, 떠오르면 살랑살랑, 맞으면 번쩍.
/// 쓰러진 화면에서는 멈춘다 (US-043). 밤 어두움은 주인공처럼 살짝만 받는다.
fn draw_zombies(assets: &Assets, game: &Game, tint: Color) {
    let frozen = game.is_over();
    let player_x = game.player.rect().center().x;
    for (i, zombie) in game.combat.zombies.list.iter().enumerate() {
        let r = nearest_copy(zombie.rect(), view_center(game));
        let t = if frozen { 0.0 } else { time() + i as f32 * 0.37 };
        let pose = match zombie.state {
            ZombieState::Walking => sprites::pose(Motion::Walk, t, false),
            ZombieState::Floating => sprites::float_pose(t),
        };
        let facing = if shortest_dx(player_x, r.center().x) > 0.0 { Facing::Left } else { Facing::Right };
        let feet = vec2(r.center().x, r.y + r.h);
        let tint = if zombie.poison.is_on() {
            shop::draw_poison_aura(r, time() + i as f32);
            shop::poison_tint(tint)
        } else {
            tint
        };
        draw_character(&assets.zombie, feet, ZOMBIE_SPRITE_HEIGHT, facing, pose, tint);
        if zombie.is_flashing() {
            draw_character(&assets.zombie_flash, feet, ZOMBIE_SPRITE_HEIGHT, facing, pose, ZOMBIE_FLASH);
        }
    }
}

/// 연습용 몬스터 그림 높이: 좀비 그림을 몸 네모 비율만큼 줄인다 (밤 좀비보다 작다).
fn practice_sprite_height() -> f32 {
    ZOMBIE_SPRITE_HEIGHT * PRACTICE_MONSTER_HEIGHT / ZOMBIE_HEIGHT
}

/// 연습용 몬스터 (US-124): 좀비 그림을 부드러운 민트색·작은 크기로 쓰고 머리 위에 반짝 별 표시를 단다 (밤 몬스터와 한눈에 다르다).
/// 새 그림 파일은 없다. 이름·생김새·색은 아이 질문 카드라서 임시 모양이다. 맞으면 좀비처럼 번쩍, 피는 없다.
fn draw_practice(assets: &Assets, game: &Game, tint: Color) {
    let Some(monster) = &game.practice else { return };
    let frozen = game.is_over();
    let player_x = game.player.rect().center().x;
    let r = nearest_copy(monster.rect(), view_center(game));
    let t = if frozen { 0.0 } else { time() };
    let near = shortest_dx(player_x, r.center().x).abs() <= PRACTICE_MONSTER_STOP_DISTANCE;
    let pose = sprites::pose(if near { Motion::Idle } else { Motion::Walk }, t, false);
    let facing = if shortest_dx(player_x, r.center().x) > 0.0 { Facing::Left } else { Facing::Right };
    let feet = vec2(r.center().x, r.y + r.h);
    let soft = PRACTICE_TINT;
    let tint = Color::new(tint.r * soft[0], tint.g * soft[1], tint.b * soft[2], tint.a * soft[3]);
    let height = practice_sprite_height();
    draw_character(&assets.zombie, feet, height, facing, pose, tint);
    if monster.is_flashing() {
        draw_character(&assets.zombie_flash, feet, height, facing, pose, ZOMBIE_FLASH);
    }
    // 머리 위 별 표시 (살랑살랑 위아래, 빙글)
    let bob = (t * 3.0).sin() * PRACTICE_MARK_BOB;
    let mark = vec2(r.center().x, feet.y - height - PRACTICE_MARK_ABOVE - PRACTICE_MARK_RADIUS + bob);
    let c = PRACTICE_MARK_COLOR;
    touch::draw_star(mark, PRACTICE_MARK_RADIUS, t * PRACTICE_MARK_TURN_SPEED, Color::new(c[0], c[1], c[2], c[3]));
}

/// 스켈레톤: 플레이어 쪽을 보고, 걸을 때 통통, 서 있으면 숨쉬기, 피할 때 뒤로 젖히고, 맞으면 번쩍.
/// 좀비와 같은 밤 색을 받고, 쓰러진 화면에서는 멈춘다.
fn draw_skeletons(assets: &Assets, game: &Game, tint: Color) {
    let frozen = game.is_over();
    let player_x = game.player.rect().center().x;
    for (i, skeleton) in game.combat.skeletons.list.iter().enumerate() {
        let (picture, flash) = assets.skeleton(skeleton.stage);
        let height = sprites::skeleton_sprite_height(skeleton.stage);
        let r = nearest_copy(skeleton.rect(), view_center(game));
        let t = if frozen { 0.0 } else { time() + i as f32 * 0.37 };
        let player_on_left = shortest_dx(player_x, r.center().x) > 0.0;
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
        let feet = vec2(r.center().x, r.y + r.h);
        let tint = if skeleton.poison.is_on() {
            shop::draw_poison_aura(r, time() + i as f32);
            shop::poison_tint(tint)
        } else {
            tint
        };
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
        let c = nearest_copy(w.rect, view_center(game)).center();
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

/// 자동차를 타고 달리는 중인지 (차가 통통 튀고 "부릉!"이 뜬다). 걷는 빠르기 판정은 logic(`walk_speed`)이 한다.
pub fn car_is_moving(riding_car: bool, speed_x: f32) -> bool {
    riding_car && speed_x.abs() > 1.0
}


/// 플레이 화면. 터치 조이스틱·버튼과 가방 창은 main이 위에 그린다. `bag_view`는 가방 버튼 반짝임과 아래 키 안내 시간.
/// `ended`가 있으면 판이 끝난 화면이다 (쓰러짐이나 클리어).
#[allow(clippy::too_many_arguments)]
pub fn draw_play(
    assets: &Assets,
    text: &TextPainter,
    game: &Game,
    fast_forward: bool,
    effects: &Effects,
    ended: Option<&EndScreen>,
    touch_mode: bool,
    gathered: &GatherView,
    bag_view: &BagView,
    window_tip_bottom: Option<f32>,
) {
    let d = game.clock.sky_darkness();
    let (camera_x, view_x) = (game.camera_left(), view_center(game));
    // 쿵 하면 맵 그림이 잠깐 흔들린다 (정보판·버튼은 그대로)
    set_shake(effects.shake());
    // 여기부터 맵 좌표: 아래 `look_at(0.0)`(화면 좌표로 되돌리는 곳)까지는 카메라를 따라 움직인다. 이 줄을 지우면 정보판·터치 버튼이 걸을 때 같이 밀린다 (BUG-037).
    // 움직이는 것은 화면에 가장 가까운 한 자리(`nearest_copy`)에만 그려서 두 번 보이지 않는다 (D61).
    // 동굴 안(임시-12)은 따로 된 한 화면: 맵의 미스·상자·코인·시민·무기·몬스터를 그리지 않는다.
    let on_map = !game.in_cave;
    if on_map {
        draw_background(assets, &SkyLook::play(d, game.clock.time_of_day_minutes(), time()), game.world.ground_y, camera_x);
        places::on_ring_cameras(camera_x, |_| places::draw_structures(&game.world, d));
        // 미스 → 가리는 앞그림 → 상자. 상자가 먼저면 집 앞 상자가 집에 묻힌다.
        if let Some(miss) = game.miss.as_ref().filter(|m| m.is_hiding()) {
            draw_miss_like(&assets.m6.miss, nearest_copy(miss.rect(&game.world), view_x), time(), WHITE);
        }
        places::on_ring_cameras(camera_x, |_| places::draw_covers(&game.world, d));
        gather::draw_map_things(game, view_x, d);
        for b in game.world.boxes() {
            let r = nearest_copy(b, view_x);
            draw_wood_box(r, d);
            boxes::draw_cracks(r, game.boxes.cracks_at(b), d);
        }
        draw_coins(&game.coins, d, time(), view_x);
    } else {
        gather::draw_cave(game, camera_x);
    }
    if ended.is_none() {
        gather::draw_usable_glow(game, view_x, time());
    }

    let player = &game.player;
    let texture = assets.character(player.kind);
    let r = player.rect();
    let height = height_with_antenna(player.kind, PLAY_SPRITE_HEIGHT);
    let feet = vec2(r.center().x, r.y + r.h);
    let tint = mix(WHITE, CHARACTER_NIGHT_TINT, d);
    for (i, citizen) in game.citizens.list.iter().enumerate().filter(|_| on_map) {
        if citizen.is_standing() {
            draw_miss_like(&assets.m6.miss_citizen_bomb, nearest_copy(citizen.rect, view_x), time() + i as f32 * 0.37, tint);
        }
    }
    for (i, walker) in game.citizens.walkers.iter().enumerate().filter(|_| on_map) {
        let r = nearest_copy(walker.rect, view_x);
        let motion = if walker.is_walking() { Motion::Walk } else { Motion::Idle };
        // 쓰러진 화면에서는 시간을 멈춰서 숨쉬기 흔들림도 멈춘다
        let t = if ended.is_some() { 0.0 } else { time() + i as f32 * 0.37 };
        let pose = sprites::pose(motion, t, false);
        let facing = if !walker.is_walking() && ended.is_none() { looking_around(t) } else { walker.facing };
        draw_character(&assets.m6.miss, vec2(r.center().x, r.y + r.h), r.h * MISS_SPRITE_SCALE, facing, pose, tint);
    }
    if on_map {
        draw_dropped_weapons(assets, game);
        boxes::draw_dropped_items(&assets.m6, game, view_x, time());
        draw_zombies(assets, game, tint);
        draw_practice(assets, game, tint);
        draw_skeletons(assets, game, tint);
    }
    if let Some(fallen) = ended.filter(|e| !e.cleared) {
        draw_fallen_player(texture, player, &game.world, feet.x, height, tint, fallen.seconds);
    } else if ended.is_some_and(|e| e.cleared) {
        // 클리어하면 맵 위 캐릭터 대신 가운데에서 크게 춤춘다 (draw_end_panel). 같은 캐릭터가 둘 보이지 않게.
    } else {
        let blink = ended.is_none() && game.combat.health.is_invincible() && (time() * HURT_BLINKS_PER_SECOND).fract() < 0.5;
        // 갑옷 (US-070): 둘레 빛, 무지개는 캐릭터 색도 바뀐다. 쓰러진 화면에서는 안 그린다.
        let armor = game.shop.armor().filter(|_| ended.is_none());
        let bag = &game.gathering.bag;
        // 자동차 (US-080, 아이: 빨간 차를 타고 다닌다): 캐릭터는 차 위에 앉고, 차를 나중에 그려서 다리를 가린다
        let moving = car_is_moving(bag.riding_car, player.vel.x);
        let body_feet = if bag.riding_car { items::seat_feet(&assets.m6, feet, moving, time()) } else { feet };
        let body = Rect { y: r.y - (feet.y - body_feet.y), ..r };
        // 엔더열매 힘 (US-081, 아이: 몸에서 금빛 기운이 활활): 캐릭터 뒤에 금빛 불꽃
        if bag.has_ender_power && ended.is_none() {
            items::draw_golden_aura(body, time());
        }
        if let Some(armor) = armor {
            shop::draw_armor_glow(armor, body, time());
        }
        let player_tint = Color { a: if blink { HURT_BLINK_ALPHA } else { 1.0 }, ..shop::armor_tint(armor, tint, time()) };
        // 클리어하면 걷던 그대로 굳지 않고 서서 숨 쉰다. 자동차에 앉아 있으면 다리를 움직이지 않는다.
        let motion = if ended.is_some() || bag.riding_car { Motion::Idle } else { player.motion };
        let pose = sprites::pose(motion, time(), player.vel.y < 0.0);
        draw_character(texture, body_feet, height, player.facing, pose, player_tint);
        // 스케이트 (US-134): 신었을 때만 발에 스케이트 색. 밤 빛과 깜박임은 캐릭터를 따라간다
        if skates::skates_shown(game.shop.has_skates(), ended.is_some()) {
            let aspect = texture.width() / texture.height();
            let shape = skates::skate_shape(player.kind, body_feet, height, aspect, player.facing, pose);
            skates::draw_skate_shape(&shape, Color { a: player_tint.a, ..tint });
        }
        if !effects.swinging_blade() {
            draw_held_weapon(assets, game.weapons.held(), body_feet, player.facing, player_tint);
        }
        if bag.riding_car {
            items::draw_car(&assets.m6, feet, player.facing, Color { a: player_tint.a, ..tint }, moving, time());
        }
        if let Some(armor) = armor {
            shop::draw_armor_sparkles(armor, body, time());
        }
    }

    if on_map {
        game.combat.skeletons.projectiles.iter().for_each(|shot| draw_projectile(shot, view_x));
    }
    // 화면에 고정된 정보판·큰 알림 글의 자리 (화면 좌표). 머리 위 "쓰러졌어요"가 이 칸들을 피한다 (BUG-032).
    let tag_space = if ended.is_none() { gather::usable_tag_screen_rect(game, view_x, camera_x) } else { None };
    // 끝난 화면은 큰 글이 맨 위에 온다 (BUG-024). 쓰러질 때는 움직임이 끝난 뒤에 숨긴다
    let show_info = !game.is_cleared() && !ended.is_some_and(EndScreen::ready);
    // 큰 알림 글은 정보판(효과 줄·물건 칸, 폰 멈춤 버튼까지)과 곡괭이 안내 상자 아래에 쌓는다 (BUG-031, 임시-78)
    // 깜짝 미션 글 (US-086): 위쪽 가운데. 판이 끝나면 보이지 않는다
    let mission_look = if ended.is_none() {
        let gap = (hud::left_panel_right(text, game.combat.level), hud::right_row1_left(text, game.clock.day_number(), game.wallet.coins()));
        mission::look(&effects.mission, game.mission_now().as_ref(), gap, |s, size| text.width(s, size))
    } else {
        None
    };
    // 튜토리얼 말풍선 (US-119): 있으면 큰 알림 글이 그 아래로 쌓인다
    let tutorial_text = if ended.is_none() { tutorial::bubble_text(game, touch_mode, tutorial::other_notice_up(effects, gathered)) } else { None };
    let info_bottom = if show_info {
        let hud = hud::hud_bottom(hud::has_row(game), touch_mode && ended.is_none());
        let hud = mission_look.as_ref().map_or(hud, |l| l.rect().bottom().max(hud));
        let hud = gathered.recipe_hint_bottom().map_or(hud, |b| b.max(hud));
        let hud = window_tip_bottom.map_or(hud, |b| b.max(hud));
        tutorial::bottom(game, touch_mode, tutorial::other_notice_up(effects, gathered)).filter(|_| tutorial_text.is_some()).map_or(hud, |b| b.max(hud))
    } else {
        0.0
    };
    let mut keep_words_off = effects.big_text_rects(text, info_bottom, tag_space);
    keep_words_off.push(Rect::new(0.0, 0.0, VIRTUAL_WIDTH, info_bottom));
    effects.draw(text, r, player.facing, &assets.m6, camera_x, game.attack_reach(), &keep_words_off);
    if ended.is_none() {
        gather::draw_mining_progress(r, game.mining_progress());
        gather::draw_usable_tag(text, game, view_x, camera_x, touch_mode, time());
    }
    if ended.is_none() && on_map {
        boxes::draw_monster_shouts(text, game, view_x, camera_x);
        for (i, walker) in game.citizens.walkers.iter().enumerate() {
            let head = vec2(places::near_view(view_x, walker.rect.center().x), walker.rect.y - CITIZEN_BUBBLE_GAP);
            if !walker.is_walking() && effects.citizen_talks(i) && game.world.is_on_screen(head.x, r.center().x) {
                draw_bubble(text, CITIZEN_HELP_TEXT, head.x, head.y, camera_x, 1.0);
            }
        }
    }

    // 여기부터 화면 좌표: 정보판·안내는 걸어도 같은 자리다. main이 이어서 그리는 터치 버튼도 같다. 흔들지 않는다.
    set_shake(Vec2::ZERO);
    Letterbox::current().look_at(0.0);
    let time_scale = if fast_forward { DEV_FAST_FORWARD_SCALE } else { 1.0 };
    effects.draw_on_screen(text, info_bottom, tag_space);
    gathered.draw_popup(text);
    if show_info {
        // 롤플레잉 게임처럼 (US-091): 왼쪽 위 얼굴·체력·레벨·경험치, 그 아래 효과·물건 칸, 오른쪽 위 날짜·코인·시계와 가방, 아래 가운데 든 무기
        hud::draw_face(assets, player.kind);
        hud::draw_bars(text, game.combat.health.hp(), game.combat.health.max_hp(), effects.heart_sparkle(), game.combat.level, game.combat.experience, effects.lost_hp());
        hud::draw_effects_and_pockets(text, game, &assets.m6, time_scale, touch_mode);
        hud::draw_coins_and_clock(text, &game.clock, game.wallet.coins());
        hud::draw_held_weapon(&assets.m6, game.weapons.held());
        if ended.is_none() {
            bag::draw_bag_button(text, bag_view, touch_mode);
        }
        if let Some(look) = &mission_look {
            mission::draw(text, look, &assets.m6.miss, effects.mission.banner().map_or(0.0, |(_, age)| age));
        }
        if let Some(words) = tutorial_text {
            tutorial::draw(assets, text, words, game.tutorial.is_cheering(), touch_mode, super::input::TUTORIAL_SKIP_KEY.1);
        }
    }
    // PC 아래 키 안내 한 줄은 처음 8초만 (임시-76). 폰에는 없다.
    if !touch_mode && ended.is_none() && !bag_view.open {
        bag::draw_short_hint(text, bag_view);
    }
    if let Some(ended) = ended {
        draw_end_panel(text, ended, texture, &assets.fireworks);
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn car_moves_only_while_riding_and_rolling() {
        assert!(car_is_moving(true, 660.0) && car_is_moving(true, -660.0));
        assert!(!car_is_moving(true, 0.0), "서 있으면 안 튄다");
        assert!(!car_is_moving(false, 220.0), "차가 없으면 아니다");
    }

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

#[cfg(test)]
mod hud_camera_tests {
    /// BUG-037: 맵을 그린 뒤 정보판·터치 버튼 전에 카메라를 화면 좌표로 되돌리는 `look_at(0.0)`이 있어야 한다.
    /// (GPU 그림이라 좌표를 직접 못 재므로, 지워지면 바로 알아채게 소스에서 순서를 확인한다.)
    #[test]
    fn hud_is_drawn_with_the_screen_camera_after_the_map() {
        let src = include_str!("screens.rs");
        let play = &src[src.find("pub fn draw_play(").unwrap()..];
        let reset = play.find("set_shake(Vec2::ZERO);").unwrap();
        let after = &play[reset..];
        let look = after.find("Letterbox::current().look_at(0.0);").expect("look_at(0.0) 사라짐");
        let hud = after.find("effects.draw_on_screen(").unwrap();
        assert!(look < hud, "정보판보다 먼저 카메라를 되돌려야 한다");
    }
}
