//! 판이 끝난 화면. 쓰러졌을 때 (US-043, US-044, 부모님 결정 2026-11-15로 D30을 바꿈): 머리는 붙은 채 살짝 비틀거리다
//! 털썩 주저앉고 머리 위에 별이 빙글빙글 도는 만화 같은 움직임, 끝나면 "쓰러졌어요"와 기록, 크게 "다시 하기". 무섭지 않게, 피 없이.
//! 클리어했을 때 (D63, US-068): 밝은 아침에 고른 캐릭터가 통통 뛰고 좌우로 홱홱 돌며 축하 춤을 춘다.
//! 쓰러진 끝 화면에도 고른 캐릭터가 다시 멀쩡히 서서 어질어질 흔들린다 (가족 피드백). 점수는 둘 다 보인다 (D64, US-069).

use super::hud::draw_pill;
use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::sky::hash;
use super::text::TextPainter;
use super::touch::draw_star;
use super::assets::FireworkArt;
use crate::config::*;
use crate::logic::game::Game;
use crate::logic::records::RunRecord;
use crate::logic::sound::{sfx_for_end_screen, Sfx};
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
/// 머리 위 별: 노란 별 5개 (아이 결정 2026-11-20, 쓰러지는 순간과 끝 화면 모두)
const DIZZY_STARS: u32 = 5;
const DIZZY_STAR_COLOR: Color = Color::new(1.0, 0.86, 0.22, 1.0);
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
    /// 새 기록일 때만 있는 폭죽 계획 (끝 화면에 들어갈 때 한 번 정한다). 아니면 비어 있다.
    pub fireworks: Vec<Firework>,
    /// "최근 5판" 표에 보일 판들, 최신이 위 (`set_recent`). 방금 판이 첫 줄.
    pub recent: Vec<RunRecord>,
    /// 새 기록 소리를 이미 냈는지 (한 판에 한 번만)
    celebrated: bool,
}

/// 폭죽 하나: 끝 화면이 "준비된" 뒤 `start`초에 터져 `life`초 동안 커졌다 사라진다. `picture`는 모양 * 3 + 색.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Firework {
    pub start: f32,
    pub life: f32,
    pub center: Vec2,
    pub picture: usize,
}

/// 끝 화면에 들어갈 때 폭죽을 정한다. 같은 판(같은 `seed`)이면 늘 같다. 모양과 색은 연달아 겹치지 않게 번갈아 나온다.
pub fn fireworks_plan(seed: u32) -> Vec<Firework> {
    let (ax, ay, aw, ah) = FIREWORK_AREA;
    let n = FIREWORKS_COUNT;
    (0..n)
        .map(|i| {
            let h = |k: u32| hash(seed.wrapping_mul(64).wrapping_add(i as u32 * 4 + k));
            let life = FIREWORK_LIFE_MIN_SECONDS + h(0) * (FIREWORK_LIFE_MAX_SECONDS - FIREWORK_LIFE_MIN_SECONDS);
            // 가로는 칸을 3칸씩 건너뛰며 써서 가까운 것끼리 연달아 터지지 않게 (개수가 3의 배수가 아닐 때 칸이 겹치지 않는다)
            let column = (i * 3 + 1) % n;
            let x = ax + (column as f32 + 0.15 + h(1) * 0.7) / n as f32 * aw;
            let y = ay + h(2) * ah;
            let shape = i % 3;
            let color = (i + i / 3) % 3;
            let start = if n > 1 { (FIREWORKS_TOTAL_SECONDS - life) * i as f32 / (n - 1) as f32 } else { 0.0 };
            Firework { start, life, center: vec2(x, y), picture: shape * 3 + color }
        })
        .collect()
}

/// `t`초(폭죽이 시작된 뒤)의 폭죽 하나: (크기 비율, 투명도). 아직 안 터졌거나 사라졌으면 None.
/// 크기는 0 → 1.0 → `FIREWORK_END_SCALE`, 투명도는 커지는 동안 진해졌다가 줄며 사라진다.
pub fn firework_look(f: &Firework, t: f32) -> Option<(f32, f32)> {
    let p = (t - f.start) / f.life;
    if !(0.0..1.0).contains(&p) {
        return None;
    }
    let grow = FIREWORK_GROW_SHARE;
    if p < grow {
        let g = p / grow;
        Some((g, FIREWORK_MAX_ALPHA * g.sqrt()))
    } else {
        let s = (p - grow) / (1.0 - grow);
        Some((1.0 - (1.0 - FIREWORK_END_SCALE) * s, FIREWORK_MAX_ALPHA * (1.0 - s)))
    }
}

/// 표에 쓸 줄들: (글, 방금 판이면 true). 비었으면 "아직 없어요" 한 줄.
pub fn recent_lines(runs: &[RunRecord]) -> Vec<(String, bool)> {
    if runs.is_empty() {
        return vec![(RECENT_RUNS_EMPTY_TEXT.to_string(), false)];
    }
    runs.iter().take(RECENT_RUNS_MAX).enumerate().map(|(i, r)| (r.line_text(), i == 0)).collect()
}

/// 표 안의 자리: (제목 글자 아래선 y, 줄마다 글자 아래선 y). 줄 수만큼.
pub fn recent_table_rows(lines: usize) -> (f32, Vec<f32>) {
    let (_, y, _, _) = RECENT_TABLE_RECT;
    let title = y + RECENT_TABLE_LINE_HEIGHT;
    (title, (1..=lines).map(|k| title + RECENT_TABLE_LINE_HEIGHT * k as f32).collect())
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
            fireworks: if new_record || new_best_score { fireworks_plan(days.wrapping_mul(7919).wrapping_add(score)) } else { Vec::new() },
            recent: Vec::new(),
            celebrated: false,
        }
    }

    /// 움직임이 끝나서 글자와 버튼이 보이는지.
    pub fn ready(&self) -> bool {
        self.seconds >= self.ready_at()
    }

    fn ready_at(&self) -> f32 {
        if self.cleared { CLEAR_ENTER_SECONDS } else { ANIMATION_SECONDS }
    }

    /// 표에 보일 판들 (최신이 위). 방금 판을 기록에 더한 뒤 한 번 넣는다.
    pub fn set_recent(&mut self, latest_first: Vec<RunRecord>) {
        self.recent = latest_first;
    }

    /// 새 기록 소리: "새 기록!" 글이 나타나는 순간(`ready`가 처음 참일 때) 한 번만 소리를 돌려준다. 그 뒤에는 늘 None.
    /// 새 기록이 아니면 처음에도 None (규칙은 `sfx_for_end_screen`).
    pub fn take_celebration(&mut self) -> Option<Sfx> {
        if self.celebrated || !self.ready() {
            return None;
        }
        self.celebrated = true;
        sfx_for_end_screen(self.new_record, self.new_best_score)
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
        draw_star(at, 9.0 * shown, turn * 2.0, Color { a: shown, ..DIZZY_STAR_COLOR });
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
        draw_star(at, 10.0 * grow, turn * 2.0, DIZZY_STAR_COLOR);
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
pub fn draw_end_panel(text: &TextPainter, ended: &EndScreen, hero: &Texture2D, art: &FireworkArt) {
    let mid = VIRTUAL_WIDTH / 2.0;
    if ended.cleared {
        draw_confetti(ended.seconds);
        draw_fireworks(art, ended);
        draw_dancing_hero(hero, ended.seconds);
        text.centered(GAME_CLEAR_TEXT, mid, 92.0, 60.0, NEW_RECORD);
    }
    if !ended.ready() {
        return;
    }
    if !ended.cleared {
        draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, DIM);
        draw_fireworks(art, ended);
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
    draw_recent_table(text, &ended.recent);
}

/// 새 기록일 때만 폭죽 그림을 뿌린다. 글자보다 먼저 그려서 글자를 가리지 않는다. 시작은 끝 화면이 준비된 순간 (새 기록 소리와 같은 때).
fn draw_fireworks(art: &FireworkArt, ended: &EndScreen) {
    let t = ended.seconds - ended.ready_at();
    for f in &ended.fireworks {
        let Some((scale, alpha)) = firework_look(f, t) else { continue };
        if let Some(texture) = art.picture(f.picture) {
            let size = FIREWORK_SIZE * scale;
            let params = DrawTextureParams { dest_size: Some(vec2(size, size)), ..Default::default() };
            draw_texture_ex(texture, f.center.x - size / 2.0, f.center.y - size / 2.0, Color::new(1.0, 1.0, 1.0, alpha), params);
        }
    }
}

/// 작은 표 "최근 5판": 최신이 위, 방금 판은 노란색, 비었으면 "아직 없어요". 줄이 칸보다 길면 글자를 줄인다.
fn draw_recent_table(text: &TextPainter, runs: &[RunRecord]) {
    let (x, y, w, h) = RECENT_TABLE_RECT;
    draw_rectangle(x, y, w, h, DIM);
    let mid = x + w / 2.0;
    let lines = recent_lines(runs);
    let (title_y, rows) = recent_table_rows(lines.len());
    text.centered(RECENT_TABLE_TITLE, mid, title_y, RECENT_TABLE_FONT, WHITE);
    let room = w - RECENT_TABLE_PADDING * 2.0;
    for ((line, newest), row_y) in lines.iter().zip(rows) {
        let size = RECENT_TABLE_FONT.min(RECENT_TABLE_FONT * room / text.width(line, RECENT_TABLE_FONT).max(1.0));
        text.centered(line, mid, row_y, size, if *newest { NEW_RECORD } else { WHITE });
    }
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
    fn dizzy_stars_are_five_yellow_stars() {
        assert_eq!(DIZZY_STARS, 5, "아이 결정 2026-11-20: 5개");
        let c = DIZZY_STAR_COLOR;
        assert!(c.r > 0.9 && c.g > 0.75 && c.b < 0.4, "노란색: {c:?}");
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

    fn run(days: u32, score: u32, cleared: bool) -> RunRecord {
        RunRecord { days, score, cleared }
    }

    #[test]
    fn fireworks_are_within_count_time_area_and_alternate_shape_and_color() {
        for seed in 0..200 {
            let plan = fireworks_plan(seed);
            assert_eq!(plan.len(), FIREWORKS_COUNT);
            assert!((6..=10).contains(&plan.len()));
            let (ax, ay, aw, ah) = FIREWORK_AREA;
            for f in &plan {
                assert!((FIREWORK_LIFE_MIN_SECONDS..=FIREWORK_LIFE_MAX_SECONDS).contains(&f.life), "{f:?}");
                assert!(f.start >= 0.0 && f.start + f.life <= FIREWORKS_TOTAL_SECONDS + 1e-3, "시간 상한: {f:?}");
                assert!(f.center.x >= ax && f.center.x <= ax + aw && f.center.y >= ay && f.center.y <= ay + ah);
                assert!(f.picture < FIREWORK_FILES.len());
            }
            for pair in plan.windows(2) {
                assert_ne!(pair[0].picture / 3, pair[1].picture / 3, "모양이 연달아 같다");
                assert_ne!(pair[0].picture % 3, pair[1].picture % 3, "색이 연달아 같다");
            }
        }
        assert_eq!(fireworks_plan(7), fireworks_plan(7), "같은 판이면 같은 폭죽");
        const { assert!(FIREWORK_AREA.1 + FIREWORK_AREA.3 + FIREWORK_SIZE / 2.0 <= VIRTUAL_HEIGHT) };
    }

    #[test]
    fn firework_grows_then_shrinks_and_fades_and_ends_before_total_time() {
        let f = Firework { start: 1.0, life: 1.0, center: Vec2::ZERO, picture: 0 };
        assert_eq!(firework_look(&f, 0.9), None, "아직");
        assert_eq!(firework_look(&f, 2.0), None, "사라짐");
        let (s0, _) = firework_look(&f, 1.0).unwrap();
        assert!(s0.abs() < 1e-3, "크기 0에서 시작");
        let (peak, a_peak) = firework_look(&f, 1.0 + FIREWORK_GROW_SHARE).unwrap();
        assert!((peak - 1.0).abs() < 1e-3 && a_peak <= FIREWORK_MAX_ALPHA + 1e-3);
        let (late, a_late) = firework_look(&f, 1.99).unwrap();
        assert!((late - FIREWORK_END_SCALE).abs() < 0.02 && a_late < 0.05, "0.8 크기로 흐려진다");
        for i in 0..100 {
            let (s, a) = firework_look(&f, 1.0 + i as f32 * 0.0099).unwrap();
            assert!((0.0..=1.0 + 1e-3).contains(&s) && (0.0..=1.0).contains(&a));
        }
    }

    #[test]
    fn fireworks_only_on_a_new_record_and_celebration_sound_comes_once() {
        let game = game_with_score_9();
        let mut none = EndScreen::new(&game, 5, 50);
        assert!(none.fireworks.is_empty(), "새 기록이 아니면 폭죽이 없다");
        none.seconds = 10.0;
        assert_eq!(none.take_celebration(), None);
        let mut best = EndScreen::new(&game, 5, 3);
        assert_eq!(best.fireworks.len(), FIREWORKS_COUNT);
        assert_eq!(best.take_celebration(), None, "글이 나타나기 전에는 안 난다");
        best.seconds = ANIMATION_SECONDS;
        assert_eq!(best.take_celebration(), Some(Sfx::NewRecord));
        for _ in 0..100 {
            best.seconds += 0.016;
            assert_eq!(best.take_celebration(), None, "다시 그려져도 또 나지 않는다");
        }
        let mut both = EndScreen::new(&game, 0, 3);
        both.new_record = true; // 둘 다 넘은 판 (새 게임은 날이 0이라 직접 맞춘다)
        both.new_best_score = true;
        both.cleared = true;
        both.seconds = CLEAR_ENTER_SECONDS;
        assert_eq!(both.take_celebration(), Some(Sfx::NewRecord), "클리어 화면도 같다, 둘 다 넘어도 하나");
        assert_eq!(both.take_celebration(), None);
    }

    #[test]
    fn recent_lines_are_newest_first_yellow_first_and_empty_says_none_yet() {
        assert_eq!(recent_lines(&[]), vec![("아직 없어요".to_string(), false)]);
        let lines = recent_lines(&[run(7, 2400, true), run(3, 120, false)]);
        assert_eq!(lines, vec![("7일 2400점 클리어".to_string(), true), ("3일 120점".to_string(), false)]);
        let many: Vec<RunRecord> = (1..=9).map(|i| run(i, i * 10, false)).collect();
        assert_eq!(recent_lines(&many).len(), RECENT_RUNS_MAX);
    }

    #[test]
    fn recent_table_with_five_rows_fits_and_stays_clear_of_button_text_and_edges() {
        let (x, y, w, h) = RECENT_TABLE_RECT;
        assert!(x >= 0.0 && y >= 0.0 && x + w <= VIRTUAL_WIDTH && y + h <= VIRTUAL_HEIGHT, "화면 안");
        assert!(x >= RETRY_BUTTON.x + RETRY_BUTTON.w, "다시 하기 버튼 오른쪽");
        assert!(y >= TALLY_Y + 8.0, "코인 내역 글 아래 (글 아래선 + 내려가는 부분)");
        assert!(x > RIGHT_X + 150.0 || y > BEST_Y + 10.0, "점수 글과 겹치지 않는다");
        let (title, rows) = recent_table_rows(RECENT_RUNS_MAX);
        assert_eq!(rows.len(), RECENT_RUNS_MAX);
        let descent = RECENT_TABLE_FONT * 0.3;
        assert!(title - RECENT_TABLE_FONT * 0.9 >= y - 2.0, "제목이 표 위로 안 나간다");
        assert!(rows[RECENT_RUNS_MAX - 1] + descent <= y + h, "다섯 줄이 모두 표 안");
        for pair in rows.windows(2) {
            assert!(pair[1] - pair[0] >= RECENT_TABLE_FONT, "줄끼리 겹치지 않는다");
        }
        assert!(title + RECENT_TABLE_FONT * 0.3 < rows[0] - RECENT_TABLE_FONT * 0.6, "제목과 첫 줄이 겹치지 않는다");
        assert!(RECENT_TABLE_FONT >= 27.0, "폰 가로(배율 0.667)에서 18 화면 픽셀");
        assert!(w > RECENT_TABLE_PADDING * 2.0 + 150.0);
    }

    #[test]
    fn retry_button_is_big_and_on_screen() {
        assert!(RETRY_BUTTON.h >= 100.0 && RETRY_BUTTON.w >= 300.0);
        assert!(RETRY_BUTTON.y + RETRY_BUTTON.h <= VIRTUAL_HEIGHT);
    }
}
