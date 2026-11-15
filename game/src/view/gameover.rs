//! 판이 끝난 화면. 쓰러졌을 때 (US-043, US-044, 부모님 결정 2026-11-15로 D30을 바꿈): 머리는 붙은 채 살짝 비틀거리다
//! 털썩 주저앉고 머리 위에 별이 빙글빙글 도는 만화 같은 움직임, 끝나면 "쓰러졌어요"와 기록, 크게 "다시 하기". 무섭지 않게, 피 없이.
//! 클리어했을 때 (D63, US-068): 밝은 아침에 고른 캐릭터가 통통 뛰고 좌우로 홱홱 돌며 축하 춤을 춘다.
//! 쓰러진 끝 화면에도 고른 캐릭터가 다시 멀쩡히 서서 어질어질 흔들린다 (가족 피드백). 점수는 둘 다 보인다 (D64, US-069).

use super::hud::draw_pill;
use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::sky::hash;
use super::text::TextPainter;
use super::touch::draw_star;
use crate::config::GAME_CLEAR_TEXT;
use crate::logic::game::Game;
use crate::logic::player::{Facing, Player};
use crate::logic::world::{nearest_copy, World};
use macroquad::prelude::*;
use std::f32::consts::{PI, TAU};

/// 움직임 전체 길이 (초). 이 뒤에 글자와 버튼이 나온다.
const ANIMATION_SECONDS: f32 = 1.6;
/// 공중에서 쓰러졌을 때 몸이 바닥까지 떨어지는 시간 (초)
const BODY_DROP_SECONDS: f32 = 0.25;
/// 비틀거리다가 주저앉기 시작·끝 (전체를 0~1로 볼 때). 앞쪽은 비틀비틀, 뒤쪽은 앉아서 별만 돈다.
const SLUMP_START: f32 = 0.5;
const SLUMP_END: f32 = 0.7;
/// 다 주저앉으면 키가 이만큼 줄고 옆으로 이만큼 퍼진다 (털썩 앉은 모습)
const SLUMP_SHORTER: f32 = 0.3;
const SLUMP_WIDER: f32 = 0.15;
/// 비틀비틀: 가장 많이 기우는 각도(라디안), 주저앉기 전까지 흔들리는 반 번 수
const WOBBLE_TILT: f32 = 0.22;
const WOBBLE_SWAYS: f32 = 3.0;
/// 머리 위 별이 다 나타나는 때 (전체를 0~1로 볼 때)
const STARS_APPEAR: f32 = 0.2;

/// "다시 하기" 버튼 (터치·클릭 판정과 그림이 같이 쓴다)
pub const RETRY_BUTTON: Rect = Rect::new(290.0, 372.0, 380.0, 108.0);
const RETRY_FILL: Color = Color::new(1.0, 0.66, 0.12, 1.0);
const DIM: Color = Color::new(0.10, 0.13, 0.32, 0.55);
const NEW_RECORD: Color = Color::new(1.0, 0.86, 0.22, 1.0);

// ---- 기록 글자 자리: 왼쪽은 버틴 날, 오른쪽은 점수. 끝 화면의 캐릭터가 그 사이에 선다 ----
const LEFT_X: f32 = 250.0;
const RIGHT_X: f32 = 710.0;
const THIS_TIME_Y: f32 = 178.0;
const BEST_Y: f32 = 232.0;
const NEW_Y: f32 = 284.0;
const TALLY_Y: f32 = 342.0;

/// 클리어 화면: 캐릭터가 나오는 시간. 이 뒤에 버튼이 눌린다 (막 누르던 손가락이 바로 넘어가지 않게).
const CLEAR_ENTER_SECONDS: f32 = 0.8;
/// 끝 화면 캐릭터 키(안테나까지 넣은 그림 전체), 발 자리.
const HERO_HEIGHT: f32 = 150.0;
const HERO_FEET: Vec2 = Vec2::new(VIRTUAL_WIDTH / 2.0, 318.0);
/// 쓰러진 끝 화면: 어질어질 흔들리는 기울기와 빠르기, 머리 위를 도는 별 수·반지름·도는 빠르기
const DIZZY_TILT: f32 = 0.12;
const DIZZY_SWAYS_PER_SECOND: f32 = 0.8;
const DIZZY_STARS: u32 = 3;
const DIZZY_STAR_RADIUS: f32 = 34.0;
const DIZZY_STAR_TURNS_PER_SECOND: f32 = 0.6;
/// 춤: 한 번 뛰는 시간, 뛰는 높이. 세 번째마다 두 배로 뛰며 한 바퀴 돈다.
const HOP_SECONDS: f32 = 0.5;
const HOP_HEIGHT: f32 = 34.0;
const BIG_HOP_EVERY: u32 = 3;
const WIGGLE_TILT: f32 = 0.2;
const WIGGLES_PER_SECOND: f32 = 2.5;
const LANDING_SQUASH: f32 = 0.2;
const CONFETTI: u32 = 18;
const CONFETTI_FALL: f32 = 90.0;
const CONFETTI_COLORS: [Color; 3] = [NEW_RECORD, Color::new(1.0, 0.55, 0.70, 1.0), Color::new(0.45, 0.85, 1.0, 1.0)];

/// 판이 끝난 뒤 화면의 상태. 게임 쪽은 멈춰 있고 이 시간만 흐른다.
#[derive(Debug, PartialEq)]
pub struct EndScreen {
    pub seconds: f32,
    /// 클리어했으면 true, 쓰러졌으면 false
    pub cleared: bool,
    /// 이번에 버틴 날
    pub days: u32,
    /// 이번 판까지 넣은 최고 기록
    pub best: u32,
    pub new_record: bool,
    pub score: u32,
    pub coins: u32,
    pub level_ups: u32,
    pub monsters: u32,
    /// 이번 판까지 넣은 최고 점수
    pub best_score: u32,
    pub new_best_score: bool,
}

/// 이번 값까지 넣은 최고 값과, 넘었는지. 같으면 새 기록이 아니다.
fn new_best(value: u32, old_best: u32) -> (u32, bool) {
    (old_best.max(value), value > old_best)
}

impl EndScreen {
    pub fn new(game: &Game, old_best: u32, old_best_score: u32) -> EndScreen {
        let days = game.clock.nights_survived();
        let score = game.score();
        let (best, new_record) = new_best(days, old_best);
        let (best_score, new_best_score) = new_best(score, old_best_score);
        EndScreen {
            seconds: 0.0,
            cleared: game.is_cleared(),
            days,
            best,
            new_record,
            score,
            coins: game.coins.picked_total,
            level_ups: game.combat.level - 1,
            monsters: game.combat.monsters_defeated,
            best_score,
            new_best_score,
        }
    }

    /// 움직임이 끝나서 글자와 버튼이 보이는지.
    pub fn ready(&self) -> bool {
        self.seconds >= if self.cleared { CLEAR_ENTER_SECONDS } else { ANIMATION_SECONDS }
    }
}

/// 가로 `center_x ± half_width` 자리에서 `from_y` 아래로 떨어지면 닿는 곳 (상자 위, 땅, 구멍 바닥).
/// `surface_y_under`를 쓰지 않는 까닭: 그쪽은 발보다 위의 나뭇가지도 세어서 가지 밑에서 쓰러지면 몸이 위로 뜬다.
fn landing_y(world: &World, center_x: f32, half_width: f32, from_y: f32) -> f32 {
    let body = Rect::new(center_x - half_width, from_y, half_width * 2.0, 0.0);
    world
        .obstacles
        .iter()
        .map(|b| nearest_copy(*b, center_x))
        .filter(|b| b.x < center_x + half_width && center_x - half_width < b.x + b.w && b.y >= from_y)
        .map(|b| b.y)
        .fold(world.floor_y_under(body), f32::min)
}

/// 몸이 비틀거리다 털썩 앉는 정도 (0 ~ 1, 잠깐 더 눌렸다가 돌아온다).
fn slump(t: f32) -> f32 {
    let s = ((t - SLUMP_START) / (SLUMP_END - SLUMP_START)).clamp(0.0, 1.0);
    s * (1.0 + 0.3 * (s * PI).sin())
}

/// 비틀비틀 기울기 (라디안): 주저앉기 전까지 좌우로 세 번 흔들리고, 앉을 때는 똑바로.
fn wobble(t: f32) -> f32 {
    let w = (t / SLUMP_START).clamp(0.0, 1.0);
    (w * WOBBLE_SWAYS * PI).sin() * WOBBLE_TILT
}

/// 머리 위 별의 진하기 (0 ~ 1): 비틀거리기 시작하면 나타나 끝까지 돈다.
fn dizzy_stars_shown(t: f32) -> f32 {
    (t / STARS_APPEAR).clamp(0.0, 1.0)
}

/// 쓰러질 때 (부모님 결정 2026-11-15, D30을 바꿈): 머리는 붙은 채로 살짝 비틀거리다 털썩 주저앉고,
/// 머리 위에 별이 빙글빙글 돈다 (끝 화면의 어지러운 별과 같은 모습). `feet_x`, `height`는 평소 캐릭터 그림과 같은 값.
pub fn draw_fallen_player(texture: &Texture2D, player: &Player, world: &World, feet_x: f32, height: f32, tint: Color, seconds: f32) {
    let t = seconds / ANIMATION_SECONDS;
    let width = height * texture.width() / texture.height();
    let start_feet = player.rect().y + player.rect().h;
    // 공중에서 쓰러졌으면 바닥까지 떨어진다
    let rest = landing_y(world, feet_x, player.size.x / 2.0, start_feet);
    let drop = (seconds / BODY_DROP_SECONDS).min(1.0);
    let feet = vec2(feet_x, start_feet + (rest - start_feet) * drop * drop);
    let s = slump(t);
    let size = vec2(width * (1.0 + SLUMP_WIDER * s), height * (1.0 - SLUMP_SHORTER * s));
    draw_texture_ex(
        texture,
        feet.x - size.x / 2.0,
        feet.y - size.y,
        tint,
        DrawTextureParams { dest_size: Some(size), rotation: wobble(t), pivot: Some(feet), flip_x: player.facing == Facing::Left, ..Default::default() },
    );
    // 머리 위 별 빙글빙글
    let shown = dizzy_stars_shown(t);
    let turn = seconds * DIZZY_STAR_TURNS_PER_SECOND * TAU;
    let head = feet - vec2(0.0, size.y + 6.0);
    let radius = DIZZY_STAR_RADIUS * 0.8;
    for i in 0..DIZZY_STARS {
        let a = turn + i as f32 * TAU / DIZZY_STARS as f32;
        let at = head + vec2(a.cos() * radius, a.sin() * radius * 0.35);
        draw_star(at, 9.0 * shown, turn * 2.0, Color { a: shown, ..NEW_RECORD });
    }
}
/// 미스의 한 순간.
#[derive(Debug, PartialEq)]
pub(super) struct MissDance {
    /// 땅에서 뜬 높이
    pub(super) lift: f32,
    /// 돈 각도 (라디안)
    spin: f32,
    /// 세로로 눌린 양 (0 ~ `LANDING_SQUASH`)
    pub(super) squash: f32,
}

/// `t`초의 춤: 통통 뛰며 좌우로 뒤뚱거리고, 세 번째마다 높이 뛰어 한 바퀴 돈다.
pub(super) fn miss_dance(t: f32) -> MissDance {
    let hop = t / HOP_SECONDS;
    let phase = hop.fract();
    let big = hop as u32 % BIG_HOP_EVERY == BIG_HOP_EVERY - 1;
    let arc = (phase * PI).sin();
    let lift = HOP_HEIGHT * arc * if big { 2.0 } else { 1.0 };
    let spin = if big { phase * TAU } else { (t * WIGGLES_PER_SECOND * TAU).sin() * WIGGLE_TILT };
    MissDance { lift, spin, squash: LANDING_SQUASH * (1.0 - arc).powi(4) }
}

/// 춤추는 캐릭터가 왼쪽을 보는지. 한 번 뛸 때마다 좌우로 홱 돈다 (우스꽝스럽게).
fn dance_faces_left(t: f32) -> bool {
    (t / HOP_SECONDS) as u32 % 2 == 1
}

/// 고른 캐릭터(손그림 그대로, 안테나 포함)가 미스와 같은 춤을 춘다.
fn draw_dancing_hero(texture: &Texture2D, seconds: f32) {
    let dance = miss_dance(seconds);
    // 처음에는 작게 퐁 커지며 나온다
    let grow = (seconds / CLEAR_ENTER_SECONDS).min(1.0);
    let height = HERO_HEIGHT * grow * (1.0 - dance.squash);
    let width = HERO_HEIGHT * grow * (1.0 + dance.squash) * texture.width() / texture.height();
    let center = HERO_FEET - vec2(0.0, dance.lift + height / 2.0);
    let params = DrawTextureParams {
        dest_size: Some(vec2(width, height)),
        rotation: dance.spin,
        pivot: Some(center),
        flip_x: dance_faces_left(seconds),
        ..Default::default()
    };
    draw_texture_ex(texture, center.x - width / 2.0, center.y - height / 2.0, WHITE, params);
}

/// 쓰러진 끝 화면 `t`초: (몸 기울기, 머리 위 별이 돈 각도). 천천히 좌우로 어질어질.
fn dizzy(t: f32) -> (f32, f32) {
    ((t * DIZZY_SWAYS_PER_SECOND * TAU).sin() * DIZZY_TILT, t * DIZZY_STAR_TURNS_PER_SECOND * TAU)
}

/// 쓰러졌던 캐릭터가 다시 멀쩡히 서서, 머리 위에 별이 빙글빙글 돈다. `t`: 끝 화면이 열린 뒤 흐른 초.
fn draw_dizzy_hero(texture: &Texture2D, t: f32) {
    let grow = (t / CLEAR_ENTER_SECONDS).min(1.0);
    let (tilt, turn) = dizzy(t);
    let height = HERO_HEIGHT * grow;
    let width = height * texture.width() / texture.height();
    let params = DrawTextureParams { dest_size: Some(vec2(width, height)), rotation: tilt, pivot: Some(HERO_FEET), ..Default::default() };
    draw_texture_ex(texture, HERO_FEET.x - width / 2.0, HERO_FEET.y - height, WHITE, params);
    let head = HERO_FEET - vec2(0.0, height + 8.0);
    for i in 0..DIZZY_STARS {
        let a = turn + i as f32 * TAU / DIZZY_STARS as f32;
        let at = head + vec2(a.cos() * DIZZY_STAR_RADIUS * grow, a.sin() * DIZZY_STAR_RADIUS * 0.35 * grow);
        draw_star(at, 10.0 * grow, turn * 2.0, NEW_RECORD);
    }
}

/// 천천히 떨어지는 색 별. 맨 위 줄에서 맨 아래 줄까지 되풀이한다.
pub(super) fn draw_confetti(seconds: f32) {
    for i in 0..CONFETTI {
        let x = hash(i * 2) * VIRTUAL_WIDTH;
        let y = (hash(i * 2 + 1) * VIRTUAL_HEIGHT + seconds * CONFETTI_FALL) % VIRTUAL_HEIGHT;
        draw_star(vec2(x, y), 10.0, seconds * 2.0 + i as f32, CONFETTI_COLORS[i as usize % CONFETTI_COLORS.len()]);
    }
}

/// 움직임이 끝나면: 큰 글, 버틴 날과 최고 기록, 점수와 내역과 최고 점수, "다시 하기".
/// `hero`: 플레이어가 고른 캐릭터 그림.
pub fn draw_end_panel(text: &TextPainter, ended: &EndScreen, hero: &Texture2D) {
    let mid = VIRTUAL_WIDTH / 2.0;
    if ended.cleared {
        draw_confetti(ended.seconds);
        draw_dancing_hero(hero, ended.seconds);
        text.centered(GAME_CLEAR_TEXT, mid, 92.0, 60.0, NEW_RECORD);
    }
    if !ended.ready() {
        return;
    }
    if !ended.cleared {
        draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, DIM);
        text.centered("쓰러졌어요", mid, 100.0, 76.0, WHITE);
        draw_dizzy_hero(hero, ended.seconds - ANIMATION_SECONDS);
    }
    text.centered(&format!("버틴 날 {}", ended.days), LEFT_X, THIS_TIME_Y, 50.0, WHITE);
    text.centered(&format!("최고 기록 {}", ended.best), LEFT_X, BEST_Y, 38.0, WHITE);
    text.centered(&format!("점수 {}", ended.score), RIGHT_X, THIS_TIME_Y, 50.0, WHITE);
    text.centered(&format!("최고 점수 {}", ended.best_score), RIGHT_X, BEST_Y, 38.0, WHITE);
    if ended.new_record {
        text.centered("새 기록!", LEFT_X, NEW_Y, 42.0, NEW_RECORD);
    }
    if ended.new_best_score {
        text.centered("새 최고 점수!", RIGHT_X, NEW_Y, 42.0, NEW_RECORD);
    }
    let tally = format!("코인 {} + 레벨업 {} + 해치운 몬스터 {}", ended.coins, ended.level_ups, ended.monsters);
    text.centered(&tally, mid, TALLY_Y, 32.0, WHITE);
    let r = RETRY_BUTTON;
    draw_pill(r.x, r.y, r.w, r.h, RETRY_FILL);
    text.centered("다시 하기", mid, r.y + r.h / 2.0 + 21.0, 62.0, WHITE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_only_goes_up_and_first_zero_is_not_new() {
        assert_eq!(new_best(0, 0), (0, false));
        assert_eq!(new_best(4, 3), (4, true));
        assert_eq!(new_best(3, 3), (3, false));
        assert_eq!(new_best(1, 3), (3, false));
    }

    fn game_with_score_9() -> Game {
        let mut game = Game::new(crate::logic::character::CharacterKind::Niro, 7);
        game.coins.picked_total = 3;
        game.combat.level = 2;
        game.combat.monsters_defeated = 5;
        game
    }

    #[test]
    fn end_screen_shows_the_score_and_keeps_the_best_apart_from_days() {
        let game = game_with_score_9();
        let screen = EndScreen::new(&game, 2, 8);
        assert_eq!((screen.score, screen.coins, screen.level_ups, screen.monsters), (9, 3, 1, 5));
        assert_eq!((screen.best_score, screen.new_best_score), (9, true));
        assert_eq!((screen.best, screen.new_record), (2, false));
        let same = EndScreen::new(&game, 0, 9);
        assert_eq!((same.best_score, same.new_best_score), (9, false));
        let less = EndScreen::new(&game, 0, 20);
        assert_eq!((less.best_score, less.new_best_score), (20, false));
    }

    #[test]
    fn clear_screen_opens_sooner_than_the_down_screen() {
        let game = game_with_score_9();
        let mut screen = EndScreen::new(&game, 0, 0);
        assert!(!screen.cleared);
        screen.seconds = CLEAR_ENTER_SECONDS;
        assert!(!screen.ready(), "쓰러졌을 때는 움직임이 끝나야 한다");
        screen.cleared = true;
        assert!(screen.ready());
        screen.seconds = 0.0;
        assert!(!screen.ready(), "막 누르던 손가락이 바로 다시 하기가 되지 않는다");
    }

    #[test]
    fn miss_hops_lands_and_spins_on_every_third_hop() {
        for i in 0..600 {
            let dance = miss_dance(i as f32 * 0.01);
            assert!((0.0..=HOP_HEIGHT * 2.0 + 0.01).contains(&dance.lift), "{dance:?}");
            assert!((0.0..=LANDING_SQUASH).contains(&dance.squash), "{dance:?}");
        }
        assert!(miss_dance(0.0).lift.abs() < 0.01, "땅에서 시작한다");
        let small = miss_dance(HOP_SECONDS * 0.5);
        let big = miss_dance(HOP_SECONDS * 2.5);
        assert!((big.lift - small.lift * 2.0).abs() < 0.01, "세 번째는 두 배로 뛴다");
        assert!((big.spin - PI).abs() < 0.01, "뛰는 가운데에서 반 바퀴");
        assert!(small.spin.abs() <= WIGGLE_TILT);
    }

    #[test]
    fn hero_stays_below_the_title_and_above_the_tally() {
        let top = HERO_FEET.y - HOP_HEIGHT * 2.0 - HERO_HEIGHT;
        assert!(top > 92.0, "축하 글 아래: {top}");
        let stars_top = HERO_FEET.y - HERO_HEIGHT - 8.0 - DIZZY_STAR_RADIUS * 0.35 - 10.0;
        assert!(stars_top > 100.0, "쓰러졌어요 글 아래: {stars_top}");
        const { assert!(HERO_FEET.y < TALLY_Y - 16.0) };
    }

    #[test]
    fn dancing_hero_turns_around_every_hop() {
        assert!(!dance_faces_left(0.0));
        assert!(dance_faces_left(HOP_SECONDS * 1.5));
        assert!(!dance_faces_left(HOP_SECONDS * 2.5));
    }

    #[test]
    fn dizzy_hero_sways_gently_and_stars_keep_turning() {
        for i in 0..600 {
            let (tilt, _) = dizzy(i as f32 * 0.01);
            assert!(tilt.abs() <= DIZZY_TILT + 1e-4, "살짝만 기운다: {tilt}");
        }
        assert_eq!(dizzy(0.0), (0.0, 0.0), "똑바로 서서 시작");
        assert!(dizzy(1.0).1 > dizzy(0.5).1);
    }

    #[test]
    fn falling_wobbles_a_little_then_sits_with_stars_and_head_stays_on() {
        // 부모님 결정 2026-11-15: 머리가 빠져 날아가지 않는다. 비틀비틀 → 털썩 → 별 빙글빙글
        assert_eq!(wobble(0.0), 0.0, "똑바로 서서 시작");
        let mut biggest: f32 = 0.0;
        for i in 0..=100 {
            let t = i as f32 / 100.0;
            biggest = biggest.max(wobble(t).abs());
            assert!(wobble(t).abs() <= WOBBLE_TILT + 1e-4, "살짝만 기운다");
        }
        assert!(biggest > WOBBLE_TILT * 0.9, "비틀거리는 게 보인다");
        assert!(wobble(SLUMP_START).abs() < 1e-3 && wobble(1.0).abs() < 1e-3, "앉을 때와 끝에는 똑바로");
        assert_eq!(slump(SLUMP_START), 0.0, "비틀거리는 동안은 서 있다");
        assert_eq!(dizzy_stars_shown(0.0), 0.0);
        assert_eq!(dizzy_stars_shown(STARS_APPEAR), 1.0);
        assert_eq!(dizzy_stars_shown(1.0), 1.0, "앉은 뒤에도 별이 돈다");
        // 다 앉은 뒤 잠깐 머물고, 납작해지지 않는다
        const { assert!(SLUMP_END < 1.0 && SLUMP_SHORTER < 0.5) };
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
