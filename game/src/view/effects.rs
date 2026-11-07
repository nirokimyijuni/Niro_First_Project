//! 싸움 중 잠깐 보이는 효과: 공격 휘두름, 몬스터가 쓰러질 때·펑 터질 때, 화살·총알 번쩍, 스켈레톤 손, "레벨 업!", 아침에 하트 반짝, 코인 반짝.
//! 일어난 일은 logic의 `CombatEvent`가 알려 주고, 여기서는 보여 주기만 한다. 시간은 진짜 시간 `dt`로 흐른다.

use super::assets::M6Art;
use super::coin::CoinPops;
use super::screen::VIRTUAL_WIDTH;
use super::sky::lerp;
use super::sprites::{fit_size, flip_weapon};
use super::text::TextPainter;
use super::touch::draw_star;
use crate::config::{CITIZEN_POP_TEXT, MISS_FOUND_TEXT};
use crate::logic::combat::{attack_box, attack_box_with_range, CombatEvent};
use crate::logic::player::Facing;
use crate::logic::weapon::Weapon;
use macroquad::prelude::*;
use std::f32::consts::TAU;

/// 효과가 보이는 시간 (초)
const SWING_SECONDS: f32 = 0.22;
const BURST_SECONDS: f32 = 0.9;
const HEART_SPARKLE_SECONDS: f32 = 1.2;
const SPARK_SECONDS: f32 = 0.25;
const HAND_SECONDS: f32 = 0.4;
const LEVEL_UP_SECONDS: f32 = 1.8;
const MESSAGE_SECONDS: f32 = 2.2;
/// 손이 플레이어에게 닿는 때 (전체를 0~1로 볼 때)
const HAND_REACH_SHARE: f32 = 0.4;
/// "레벨 업!" 글자 바닥선, 크기, 별이 도는 타원 폭·높이
const LEVEL_UP_Y: f32 = 215.0;
const LEVEL_UP_SIZE: f32 = 84.0;
const LEVEL_UP_RING: Vec2 = Vec2::new(230.0, 90.0);

/// "미스를 찾았어요!" 같은 큰 문구 바닥선과 크기. "레벨 업!"(바닥선 215)과 겹치지 않게 그 아래.
const MESSAGE_Y: f32 = 300.0;
const MESSAGE_SIZE: f32 = 56.0;
/// 휘두를 때 손에 보이는 칼·도끼 크기
const SWING_WEAPON_SIZE: f32 = 46.0;
/// 칼·도끼 그림은 끝이 45도 위를 향해 있다
const WEAPON_ART_ANGLE: f32 = std::f32::consts::FRAC_PI_4;
/// 총 빛줄기 굵기
const GUN_LIGHT_WIDTH: f32 = 7.0;

/// 휘두름: 주먹이 위에서 앞으로 내려오는 각도 (라디안, 0 = 정면)
const SWING_START_ANGLE: f32 = -1.1;
const SWING_END_ANGLE: f32 = 0.7;
const SWING_TRAIL_STEPS: u32 = 8;

const FIST: Color = Color::new(1.0, 0.93, 0.78, 1.0);
const FIST_LINE: Color = Color::new(0.13, 0.16, 0.36, 1.0);
const TRAIL: Color = Color::new(1.0, 1.0, 1.0, 0.8);
const STAR: Color = Color::new(1.0, 0.86, 0.22, 1.0);
const PUFF: Color = Color::new(1.0, 1.0, 1.0, 0.85);
const POP_TEXT: Color = Color::new(1.0, 0.72, 0.20, 1.0);
const SPARK: Color = Color::new(1.0, 0.97, 0.75, 0.9);
const BONE: Color = Color::new(0.96, 0.92, 0.80, 1.0);
const LEVEL_UP_TEXT: Color = Color::new(1.0, 0.86, 0.22, 1.0);
const GUN_LIGHT: Color = Color::new(1.0, 0.95, 0.60, 0.9);

#[derive(Clone, Copy, PartialEq)]
enum BurstKind {
    /// 공격으로 쓰러짐: 별이 퍼지고 "쓰러졌어요"
    Defeated,
    /// 팔을 내려 터짐: 구름과 별, "펑!"
    Popped,
    /// 미스를 찾은 자리: 별만 퍼진다
    Found,
    /// 화살·총알이 나가거나 튕길 때 작은 번쩍
    Spark,
    /// 스켈레톤 손이 플레이어에게 쑥
    Hand,
}

impl BurstKind {
    fn seconds(self) -> f32 {
        match self {
            BurstKind::Defeated | BurstKind::Popped | BurstKind::Found => BURST_SECONDS,
            BurstKind::Spark => SPARK_SECONDS,
            BurstKind::Hand => HAND_SECONDS,
        }
    }
}

struct Burst {
    kind: BurstKind,
    at: Vec2,
    age: f32,
}

#[derive(Default)]
pub struct Effects {
    /// 코인 주울 때 반짝
    pub pops: CoinPops,
    bursts: Vec<Burst>,
    swing_age: Option<f32>,
    /// 휘두를 때 든 것
    swing_weapon: Option<Weapon>,
    /// 큰 문구와 나온 지 지난 시간
    message: Option<(&'static str, f32)>,
    heart_sparkle_age: Option<f32>,
    level_up_age: Option<f32>,
}

impl Effects {
    pub fn new() -> Effects {
        Effects::default()
    }

    pub fn start_swing(&mut self, weapon: Weapon) {
        self.swing_age = Some(0.0);
        self.swing_weapon = Some(weapon);
    }

    /// 칼·도끼를 휘두르는 중이면 손의 무기는 안 그린다 (휘두르는 그림과 둘로 보인다).
    pub fn swinging_blade(&self) -> bool {
        self.swing_age.is_some() && matches!(self.swing_weapon, Some(Weapon::Sword | Weapon::Axe))
    }

    /// 화면 가운데에 큰 문구를 잠깐 보여 준다. 새 문구가 오면 바뀐다.
    pub fn show_message(&mut self, message: &'static str) {
        self.message = Some((message, 0.0));
    }

    pub fn on_miss_found(&mut self, at: Vec2) {
        self.burst(BurstKind::Found, at);
        self.show_message(MISS_FOUND_TEXT);
    }

    pub fn on_citizen_popped(&mut self, at: Vec2) {
        self.burst(BurstKind::Popped, at);
    }

    pub fn on_combat(&mut self, event: CombatEvent) {
        match event {
            CombatEvent::ZombieDefeated { at } | CombatEvent::SkeletonDefeated { at } => self.burst(BurstKind::Defeated, at),
            CombatEvent::ZombiePopped { at } => self.burst(BurstKind::Popped, at),
            CombatEvent::ProjectileFired { at, .. } | CombatEvent::ProjectileBounced { at } => self.burst(BurstKind::Spark, at),
            CombatEvent::SkeletonHandHit { at } => self.burst(BurstKind::Hand, at),
            CombatEvent::LevelUp { .. } => self.level_up_age = Some(0.0),
            CombatEvent::HealthRefilled => self.heart_sparkle_age = Some(0.0),
            _ => {}
        }
    }

    fn burst(&mut self, kind: BurstKind, at: Vec2) {
        self.bursts.push(Burst { kind, at, age: 0.0 });
    }

    /// 진짜 흐른 시간 `dt`만큼 진행하고, 끝난 효과는 지운다.
    pub fn update(&mut self, dt: f32) {
        self.pops.update(dt);
        for burst in &mut self.bursts {
            burst.age += dt;
        }
        self.bursts.retain(|b| b.age < b.kind.seconds());
        self.swing_age = self.swing_age.map(|age| age + dt).filter(|age| *age < SWING_SECONDS);
        self.heart_sparkle_age = self.heart_sparkle_age.map(|age| age + dt).filter(|age| *age < HEART_SPARKLE_SECONDS);
        self.level_up_age = self.level_up_age.map(|age| age + dt).filter(|age| *age < LEVEL_UP_SECONDS);
        self.message = self.message.map(|(message, age)| (message, age + dt)).filter(|(_, age)| *age < MESSAGE_SECONDS);
    }

    /// 하트 반짝임 세기: 1 막 시작 ~ 0 끝.
    pub fn heart_sparkle(&self) -> f32 {
        self.heart_sparkle_age.map_or(0.0, |age| 1.0 - age / HEART_SPARKLE_SECONDS)
    }

    pub fn draw(&self, text: &TextPainter, player: Rect, facing: Facing, art: &M6Art) {
        if let Some(age) = self.swing_age {
            let t = age / SWING_SECONDS;
            match self.swing_weapon.unwrap_or(Weapon::Hands) {
                Weapon::Gun => draw_gun_light(player, facing, t),
                weapon => draw_swing(player, facing, t, weapon, art),
            }
        }
        for burst in &self.bursts {
            match burst.kind {
                BurstKind::Spark => draw_spark(burst),
                BurstKind::Hand => draw_hand(burst, player.center()),
                BurstKind::Defeated | BurstKind::Popped | BurstKind::Found => draw_burst(text, burst),
            }
        }
        self.pops.draw(text);
    }

    /// 화면에 고정된 효과. 카메라를 푼 뒤에 부른다.
    pub fn draw_on_screen(&self, text: &TextPainter) {
        if let Some(age) = self.level_up_age {
            draw_level_up(text, age / LEVEL_UP_SECONDS);
        }
        if let Some((message, age)) = self.message {
            let fade = ((1.0 - age / MESSAGE_SECONDS) * 3.0).min(1.0);
            text.centered(message, VIRTUAL_WIDTH / 2.0, MESSAGE_Y, MESSAGE_SIZE, Color { a: fade, ..LEVEL_UP_TEXT });
        }
    }
}

/// 총: 앞쪽으로 빛줄기가 쭉 나갔다가 사라진다. 닿는 거리는 logic의 공격 네모를 그대로 쓴다.
fn draw_gun_light(player: Rect, facing: Facing, t: f32) {
    let reach = attack_box_with_range(player, facing, Weapon::Gun.range());
    let (from, to) = if facing == Facing::Right { (reach.x, reach.x + reach.w) } else { (reach.x + reach.w, reach.x) };
    let y = player.center().y;
    let tip = lerp(from, to, (t * 2.0).min(1.0));
    let fade = 1.0 - t;
    draw_line(from, y, tip, y, GUN_LIGHT_WIDTH * fade + 2.0, Color { a: GUN_LIGHT.a * fade, ..GUN_LIGHT });
    draw_star(vec2(tip, y), 12.0, t * 6.0, Color { a: fade, ..STAR });
}

/// 플레이어 앞에서 주먹이나 칼·도끼가 휙. `t`: 0 시작 ~ 1 끝. 닿는 거리는 logic의 공격 네모를 그대로 쓴다.
fn draw_swing(player: Rect, facing: Facing, t: f32, weapon: Weapon, art: &M6Art) {
    let side = if facing == Facing::Right { 1.0 } else { -1.0 };
    let shoulder = player.center();
    let length = player.w / 2.0 + attack_box(player, facing).w * 0.8;
    let point = |u: f32| {
        let a = lerp(SWING_START_ANGLE, SWING_END_ANGLE, u);
        shoulder + vec2(side * a.cos(), a.sin()) * length
    };
    let fade = 1.0 - t * t;
    for i in 0..SWING_TRAIL_STEPS {
        let (u0, u1) = (t * i as f32 / SWING_TRAIL_STEPS as f32, t * (i + 1) as f32 / SWING_TRAIL_STEPS as f32);
        let (a, b) = (point(u0), point(u1));
        let strength = (i + 1) as f32 / SWING_TRAIL_STEPS as f32;
        draw_line(a.x, a.y, b.x, b.y, 4.0 + 8.0 * strength, Color { a: TRAIL.a * strength * fade, ..TRAIL });
    }
    let fist = point(t);
    if let Some(texture) = art.weapon(weapon) {
        let size = fit_size(texture, SWING_WEAPON_SIZE);
        let angle = lerp(SWING_START_ANGLE, SWING_END_ANGLE, t) + WEAPON_ART_ANGLE;
        draw_texture_ex(
            texture,
            fist.x - size.x / 2.0,
            fist.y - size.y / 2.0,
            Color { a: fade, ..WHITE },
            DrawTextureParams { dest_size: Some(size), rotation: angle * side, flip_x: flip_weapon(weapon, facing), ..Default::default() },
        );
        return;
    }
    draw_circle(fist.x, fist.y, 13.0, Color { a: fade, ..FIST_LINE });
    draw_circle(fist.x, fist.y, 10.5, Color { a: fade, ..FIST });
}

/// 화살·총알이 나갈 때·튕길 때: 빛이 퍼지며 사라진다.
fn draw_spark(burst: &Burst) {
    let t = burst.age / SPARK_SECONDS;
    draw_circle(burst.at.x, burst.at.y, 5.0 + 10.0 * t, Color { a: SPARK.a * (1.0 - t), ..SPARK });
    draw_star(burst.at, 9.0 * (1.0 - t) + 2.0, t * 3.0, Color { a: 1.0 - t, ..STAR });
}

/// 스켈레톤 손: 몸에서 플레이어 쪽으로 쑥 나가고, 닿으면 별이 튄다. 손가락 세 개 달린 뼈 손.
fn draw_hand(burst: &Burst, target: Vec2) {
    let t = burst.age / HAND_SECONDS;
    let fade = 1.0 - t * t;
    let p = burst.at.lerp(target, (t / HAND_REACH_SHARE).min(1.0));
    let forward = (target - burst.at).normalize_or_zero();
    for finger in [-0.6_f32, 0.0, 0.6] {
        let tip = p + Vec2::from_angle(finger).rotate(forward) * 15.0;
        draw_line(p.x, p.y, tip.x, tip.y, 5.0, Color { a: fade, ..FIST_LINE });
        draw_line(p.x, p.y, tip.x, tip.y, 3.0, Color { a: fade, ..BONE });
    }
    draw_circle(p.x, p.y, 9.0, Color { a: fade, ..FIST_LINE });
    draw_circle(p.x, p.y, 7.0, Color { a: fade, ..BONE });
    if t >= HAND_REACH_SHARE {
        for i in 0..5 {
            let a = i as f32 / 5.0 * TAU + 0.4;
            draw_star(p + vec2(a.cos(), a.sin()) * 34.0 * t, 8.0, t * 4.0 + i as f32, Color { a: fade, ..STAR });
        }
    }
}

/// "레벨 업!": 큰 글자 둘레로 별이 돌고, 끝날 때쯤 흐려진다. `t`: 0 시작 ~ 1 끝.
fn draw_level_up(text: &TextPainter, t: f32) {
    let fade = ((1.0 - t) * 3.0).min(1.0);
    let center = vec2(VIRTUAL_WIDTH / 2.0, LEVEL_UP_Y - LEVEL_UP_SIZE / 3.0);
    let open = (t * 4.0).min(1.0);
    for i in 0..12 {
        let a = i as f32 / 12.0 * TAU + t * 2.0;
        let p = center + vec2(a.cos(), a.sin()) * LEVEL_UP_RING * open;
        draw_star(p, 13.0, t * 6.0 + i as f32, Color { a: fade, ..STAR });
    }
    text.centered("레벨 업!", VIRTUAL_WIDTH / 2.0, LEVEL_UP_Y, LEVEL_UP_SIZE, Color { a: fade, ..LEVEL_UP_TEXT });
}

fn draw_burst(text: &TextPainter, burst: &Burst) {
    let t = (burst.age / BURST_SECONDS).clamp(0.0, 1.0);
    let fade = 1.0 - t;
    let spread = 10.0 + 46.0 * (1.0 - fade * fade);
    if burst.kind == BurstKind::Popped {
        for i in 0..6 {
            let a = i as f32 / 6.0 * TAU;
            let p = burst.at + vec2(a.cos(), a.sin()) * spread * 0.7;
            draw_circle(p.x, p.y, 20.0 * (1.0 - t * 0.5), Color { a: PUFF.a * fade, ..PUFF });
        }
    }
    for i in 0..8 {
        let a = i as f32 / 8.0 * TAU + 0.3;
        let p = burst.at + vec2(a.cos(), a.sin()) * spread;
        draw_star(p, 9.0 * fade + 3.0, t * 5.0 + i as f32, Color { a: fade, ..STAR });
    }
    let rise = 36.0 * t;
    match burst.kind {
        BurstKind::Defeated => text.centered("쓰러졌어요", burst.at.x, burst.at.y - 40.0 - rise, 30.0, Color { a: fade, ..WHITE }),
        BurstKind::Popped => text.centered(CITIZEN_POP_TEXT, burst.at.x, burst.at.y - 30.0 - rise, 48.0, Color { a: fade, ..POP_TEXT }),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bursts_and_sparkle_come_from_events_and_go_away() {
        let mut e = Effects::new();
        e.on_combat(CombatEvent::ZombieDefeated { at: Vec2::ZERO });
        e.on_combat(CombatEvent::ZombiePopped { at: Vec2::ZERO });
        e.on_combat(CombatEvent::ZombieHit { at: Vec2::ZERO, hp_left: 1 });
        e.on_combat(CombatEvent::HealthRefilled);
        assert_eq!(e.bursts.len(), 2, "맞기만 한 좀비는 번쩍이기만 한다");
        assert_eq!(e.heart_sparkle(), 1.0);
        e.update(BURST_SECONDS + 0.01);
        assert!(e.bursts.is_empty());
        e.update(HEART_SPARKLE_SECONDS);
        assert_eq!(e.heart_sparkle(), 0.0);
    }

    #[test]
    fn skeleton_events_show_effects_and_go_away() {
        let mut e = Effects::new();
        e.on_combat(CombatEvent::SkeletonDefeated { at: Vec2::ZERO });
        e.on_combat(CombatEvent::SkeletonHandHit { at: Vec2::ZERO });
        e.on_combat(CombatEvent::ProjectileBounced { at: Vec2::ZERO });
        e.on_combat(CombatEvent::SkeletonHit { at: Vec2::ZERO, hp_left: 2 });
        e.on_combat(CombatEvent::LevelUp { level: 2 });
        assert_eq!(e.bursts.len(), 3);
        e.update(SPARK_SECONDS);
        assert_eq!(e.bursts.len(), 2, "번쩍은 먼저 사라진다");
        e.update(BURST_SECONDS);
        assert!(e.bursts.is_empty());
        assert!(e.level_up_age.is_some());
        e.update(LEVEL_UP_SECONDS);
        assert!(e.level_up_age.is_none());
    }

    #[test]
    fn miss_and_weapon_messages_show_then_go_away() {
        let mut e = Effects::new();
        e.on_miss_found(Vec2::ZERO);
        assert_eq!(e.message.map(|m| m.0), Some(MISS_FOUND_TEXT));
        e.show_message(Weapon::Gun.picked_text());
        assert_eq!(e.message.map(|m| m.0), Some(Weapon::Gun.picked_text()), "새 문구로 바뀐다");
        e.on_citizen_popped(Vec2::ZERO);
        assert_eq!(e.bursts.len(), 2);
        e.update(MESSAGE_SECONDS);
        assert!(e.message.is_none() && e.bursts.is_empty());
    }

    #[test]
    fn swing_ends_by_itself() {
        let mut e = Effects::new();
        e.start_swing(Weapon::Sword);
        e.update(SWING_SECONDS / 2.0);
        assert!(e.swing_age.is_some());
        e.update(SWING_SECONDS);
        assert!(e.swing_age.is_none());
    }
}
