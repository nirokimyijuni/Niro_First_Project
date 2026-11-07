//! 쓰러졌을 때 화면 (D30, US-043, US-044): 머리가 퐁 튀어 오르고 몸이 털썩 주저앉는 만화 같은 움직임,
//! 끝나면 "쓰러졌어요"와 기록, 크게 "다시 하기". 무섭지 않게, 피 없이.

use super::assets::head_share;
use super::hud::draw_pill;
use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::text::TextPainter;
use crate::logic::player::{Facing, Player};
use crate::logic::world::World;
use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};

/// 움직임 전체 길이 (초). 이 뒤에 글자와 버튼이 나온다.
const ANIMATION_SECONDS: f32 = 1.6;
/// 공중에서 쓰러졌을 때 몸이 바닥까지 떨어지는 시간 (초)
const BODY_DROP_SECONDS: f32 = 0.25;
/// 몸이 주저앉기 시작·끝 (전체를 0~1로 볼 때)
const SLUMP_START: f32 = 0.12;
const SLUMP_END: f32 = 0.32;
/// 다 주저앉으면 키가 이만큼 줄고 옆으로 이만큼 퍼진다
const SLUMP_SHORTER: f32 = 0.38;
const SLUMP_WIDER: f32 = 0.2;
/// 머리: 날아가는 시간 비율(나머지는 한 번 통 튀기), 솟는 높이, 튀는 높이, 옆으로 굴러가는 거리, 멈췄을 때 기울기
const HEAD_FLIGHT_SHARE: f32 = 0.7;
const HEAD_PEAK: f32 = 130.0;
const HEAD_BOUNCE: f32 = 18.0;
const HEAD_ROLL: f32 = 74.0;
const HEAD_REST_TILT: f32 = 0.3;

/// "다시 하기" 버튼 (터치·클릭 판정과 그림이 같이 쓴다)
pub const RETRY_BUTTON: Rect = Rect::new(290.0, 372.0, 380.0, 108.0);
const RETRY_FILL: Color = Color::new(1.0, 0.66, 0.12, 1.0);
const DIM: Color = Color::new(0.10, 0.13, 0.32, 0.55);
const NEW_RECORD: Color = Color::new(1.0, 0.86, 0.22, 1.0);

/// 쓰러진 뒤 화면의 상태. 게임 쪽은 멈춰 있고 이 시간만 흐른다.
#[derive(Debug, PartialEq)]
pub struct DownScreen {
    pub seconds: f32,
    /// 이번에 버틴 날
    pub days: u32,
    /// 이번 판까지 넣은 최고 기록
    pub best: u32,
    pub new_record: bool,
}

impl DownScreen {
    pub fn new(days: u32, old_best: u32) -> DownScreen {
        DownScreen { seconds: 0.0, days, best: old_best.max(days), new_record: days > old_best }
    }

    /// 움직임이 끝나서 글자와 버튼이 보이는지.
    pub fn ready(&self) -> bool {
        self.seconds >= ANIMATION_SECONDS
    }
}

/// 가로 `center_x ± half_width` 자리에서 `from_y` 아래로 떨어지면 닿는 곳 (상자 위, 땅, 구멍 바닥).
/// `surface_y_under`를 쓰지 않는 까닭: 그쪽은 발보다 위의 나뭇가지도 세어서 가지 밑에서 쓰러지면 몸이 위로 뜬다.
fn landing_y(world: &World, center_x: f32, half_width: f32, from_y: f32) -> f32 {
    let body = Rect::new(center_x - half_width, from_y, half_width * 2.0, 0.0);
    world
        .obstacles
        .iter()
        .filter(|b| b.x < center_x + half_width && center_x - half_width < b.x + b.w && b.y >= from_y)
        .map(|b| b.y)
        .fold(world.floor_y_under(body), f32::min)
}

struct HeadFlight {
    /// 옆으로 간 거리 (0 ~ HEAD_ROLL)
    side: f32,
    /// 머리 아래 끝이 닿을 곳에서 얼마나 떠 있는지
    height: f32,
    spin: f32,
}

/// `t`: 0 시작 ~ 1 끝. `start_height`: 처음(몸 위에 붙어 있을 때) 떠 있는 높이.
fn head_flight(t: f32, start_height: f32) -> HeadFlight {
    let t = t.clamp(0.0, 1.0);
    let flying = (t / HEAD_FLIGHT_SHARE).min(1.0);
    let height = if t < HEAD_FLIGHT_SHARE {
        start_height * (1.0 - flying) + 4.0 * HEAD_PEAK * flying * (1.0 - flying)
    } else {
        let bounce = (t - HEAD_FLIGHT_SHARE) / (1.0 - HEAD_FLIGHT_SHARE);
        4.0 * HEAD_BOUNCE * bounce * (1.0 - bounce)
    };
    let ease = 1.0 - (1.0 - t) * (1.0 - t);
    HeadFlight { side: HEAD_ROLL * ease, height, spin: (TAU + HEAD_REST_TILT) * ease }
}

/// 몸이 주저앉은 정도 (0 ~ 1, 잠깐 더 눌렸다가 돌아온다).
fn slump(t: f32) -> f32 {
    let s = ((t - SLUMP_START) / (SLUMP_END - SLUMP_START)).clamp(0.0, 1.0);
    s * (1.0 + 0.3 * (s * PI).sin())
}

/// 손그림을 머리 윗부분과 몸으로 나눠서 그린다. `feet_x`, `height`는 평소 캐릭터 그림과 같은 값.
pub fn draw_fallen_player(texture: &Texture2D, player: &Player, world: &World, feet_x: f32, height: f32, tint: Color, seconds: f32) {
    let t = seconds / ANIMATION_SECONDS;
    let (tw, th) = (texture.width(), texture.height());
    let width = height * tw / th;
    let share = head_share(player.kind);
    let (head_h, body_h) = (height * share, height * (1.0 - share));
    let flip_x = player.facing == Facing::Left;
    let start_feet = player.rect().y + player.rect().h;

    // 몸: 바닥까지 떨어지고 털썩
    let rest = landing_y(world, feet_x, player.size.x / 2.0, start_feet);
    let drop = (seconds / BODY_DROP_SECONDS).min(1.0);
    let feet_y = start_feet + (rest - start_feet) * drop * drop;
    let s = slump(t);
    let size = vec2(width * (1.0 + SLUMP_WIDER * s), body_h * (1.0 - SLUMP_SHORTER * s));
    draw_texture_ex(
        texture,
        feet_x - size.x / 2.0,
        feet_y - size.y,
        tint,
        DrawTextureParams {
            source: Some(Rect::new(0.0, th * share, tw, th * (1.0 - share))),
            dest_size: Some(size),
            flip_x,
            ..Default::default()
        },
    );

    // 머리: 화면 가운데 쪽으로 튀어 올랐다가 떨어져 통 튄다
    let side = if feet_x < VIRTUAL_WIDTH / 2.0 { 1.0 } else { -1.0 };
    let head_rest = landing_y(world, feet_x + side * HEAD_ROLL, width / 2.0, start_feet);
    let flight = head_flight(t, head_rest - (start_feet - body_h));
    let center = vec2(feet_x + side * flight.side, head_rest - flight.height - head_h / 2.0);
    draw_texture_ex(
        texture,
        center.x - width / 2.0,
        center.y - head_h / 2.0,
        tint,
        DrawTextureParams {
            source: Some(Rect::new(0.0, 0.0, tw, th * share)),
            dest_size: Some(vec2(width, head_h)),
            rotation: flight.spin * side,
            pivot: Some(center),
            flip_x,
            ..Default::default()
        },
    );
}

/// 움직임이 끝나면: "쓰러졌어요", 버틴 날, 최고 기록, "다시 하기".
pub fn draw_down_panel(text: &TextPainter, down: &DownScreen) {
    if !down.ready() {
        return;
    }
    let mid = VIRTUAL_WIDTH / 2.0;
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, DIM);
    text.centered("쓰러졌어요", mid, 130.0, 80.0, WHITE);
    text.centered(&format!("버틴 날 {}", down.days), mid, 212.0, 54.0, WHITE);
    text.centered(&format!("최고 기록 {}", down.best), mid, 276.0, 46.0, WHITE);
    if down.new_record {
        text.centered("새 기록!", mid, 342.0, 50.0, NEW_RECORD);
    }
    let r = RETRY_BUTTON;
    draw_pill(r.x, r.y, r.w, r.h, RETRY_FILL);
    text.centered("다시 하기", mid, r.y + r.h / 2.0 + 21.0, 62.0, WHITE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_only_goes_up_and_first_zero_is_not_new() {
        assert_eq!((DownScreen::new(0, 0).best, DownScreen::new(0, 0).new_record), (0, false));
        let beat = DownScreen::new(4, 3);
        assert_eq!((beat.best, beat.new_record), (4, true));
        let same = DownScreen::new(3, 3);
        assert_eq!((same.best, same.new_record), (3, false));
        let less = DownScreen::new(1, 3);
        assert_eq!((less.best, less.new_record), (3, false));
    }

    #[test]
    fn head_goes_up_then_lands() {
        let start = 40.0;
        assert_eq!(head_flight(0.0, start).height, start, "처음에는 몸 위에 붙어 있다");
        assert!(head_flight(HEAD_FLIGHT_SHARE / 2.0, start).height > start + 50.0, "위로 퐁");
        let end = head_flight(1.0, start);
        assert_eq!(end.height, 0.0, "바닥에 멈춘다");
        assert_eq!(end.side, HEAD_ROLL);
    }

    #[test]
    fn body_slumps_and_stays() {
        assert_eq!(slump(0.0), 0.0);
        assert_eq!(slump(1.0), 1.0);
        assert!(slump((SLUMP_START + SLUMP_END) / 2.0) > 0.5);
    }

    #[test]
    fn falling_body_lands_on_box_or_ground() {
        let world = World { width: 960.0, ground_y: 440.0, obstacles: vec![Rect::new(300.0, 380.0, 100.0, 60.0)] };
        assert_eq!(landing_y(&world, 350.0, 24.0, 300.0), 380.0, "상자 위로");
        assert_eq!(landing_y(&world, 350.0, 24.0, 380.0), 380.0, "상자 위에 서 있었다");
        assert_eq!(landing_y(&world, 100.0, 24.0, 300.0), 440.0, "상자 없는 곳은 땅");
        assert_eq!(landing_y(&world, 350.0, 24.0, 440.0), 440.0, "상자보다 아래면 땅");
        let long = World::long_map(7);
        let hole = long.hole().unwrap();
        let floor = hole.y + hole.h;
        assert_eq!(landing_y(&long, hole.center().x, 24.0, floor), floor, "구멍 안에서는 구멍 바닥");
        let branch = long.tree_branches()[1];
        assert_eq!(landing_y(&long, branch.center().x, 24.0, long.ground_y), long.ground_y, "가지 밑에서 쓰러져도 땅에 있다");
        assert_eq!(landing_y(&long, branch.center().x, 24.0, branch.y - 40.0), branch.y, "가지 위로 떨어지면 가지 위");
    }

    #[test]
    fn retry_button_is_big_and_on_screen() {
        assert!(RETRY_BUTTON.h >= 100.0 && RETRY_BUTTON.w >= 300.0);
        assert!(RETRY_BUTTON.y + RETRY_BUTTON.h <= VIRTUAL_HEIGHT);
    }
}
