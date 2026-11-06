//! 싸움 중 잠깐 보이는 효과: 공격 휘두름, 좀비가 쓰러질 때·펑 터질 때, 아침에 하트 반짝, 코인 반짝.
//! 일어난 일은 logic의 `CombatEvent`가 알려 주고, 여기서는 보여 주기만 한다. 시간은 진짜 시간 `dt`로 흐른다.

use super::coin::CoinPops;
use super::sky::lerp;
use super::text::TextPainter;
use super::touch::draw_star;
use crate::logic::combat::{attack_box, CombatEvent};
use crate::logic::player::Facing;
use macroquad::prelude::*;
use std::f32::consts::TAU;

/// 효과가 보이는 시간 (초)
const SWING_SECONDS: f32 = 0.22;
const BURST_SECONDS: f32 = 0.9;
const HEART_SPARKLE_SECONDS: f32 = 1.2;

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

#[derive(Clone, Copy, PartialEq)]
enum BurstKind {
    /// 공격으로 쓰러짐: 별이 퍼지고 "쓰러졌어요"
    Defeated,
    /// 팔을 내려 터짐: 구름과 별, "펑!"
    Popped,
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
    heart_sparkle_age: Option<f32>,
}

impl Effects {
    pub fn new() -> Effects {
        Effects::default()
    }

    pub fn start_swing(&mut self) {
        self.swing_age = Some(0.0);
    }

    pub fn on_combat(&mut self, event: CombatEvent) {
        match event {
            CombatEvent::ZombieDefeated { at } => self.bursts.push(Burst { kind: BurstKind::Defeated, at, age: 0.0 }),
            CombatEvent::ZombiePopped { at } => self.bursts.push(Burst { kind: BurstKind::Popped, at, age: 0.0 }),
            CombatEvent::HealthRefilled => self.heart_sparkle_age = Some(0.0),
            _ => {}
        }
    }

    /// 진짜 흐른 시간 `dt`만큼 진행하고, 끝난 효과는 지운다.
    pub fn update(&mut self, dt: f32) {
        self.pops.update(dt);
        for burst in &mut self.bursts {
            burst.age += dt;
        }
        self.bursts.retain(|b| b.age < BURST_SECONDS);
        self.swing_age = self.swing_age.map(|age| age + dt).filter(|age| *age < SWING_SECONDS);
        self.heart_sparkle_age = self.heart_sparkle_age.map(|age| age + dt).filter(|age| *age < HEART_SPARKLE_SECONDS);
    }

    /// 하트 반짝임 세기: 1 막 시작 ~ 0 끝.
    pub fn heart_sparkle(&self) -> f32 {
        self.heart_sparkle_age.map_or(0.0, |age| 1.0 - age / HEART_SPARKLE_SECONDS)
    }

    pub fn draw(&self, text: &TextPainter, player: Rect, facing: Facing) {
        if let Some(age) = self.swing_age {
            draw_swing(player, facing, age / SWING_SECONDS);
        }
        for burst in &self.bursts {
            draw_burst(text, burst);
        }
        self.pops.draw(text);
    }
}

/// 플레이어 앞에서 주먹이 휙. `t`: 0 시작 ~ 1 끝. 닿는 거리는 logic의 공격 네모를 그대로 쓴다.
fn draw_swing(player: Rect, facing: Facing, t: f32) {
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
    draw_circle(fist.x, fist.y, 13.0, Color { a: fade, ..FIST_LINE });
    draw_circle(fist.x, fist.y, 10.5, Color { a: fade, ..FIST });
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
        BurstKind::Popped => text.centered("펑!", burst.at.x, burst.at.y - 30.0 - rise, 48.0, Color { a: fade, ..POP_TEXT }),
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
    fn swing_ends_by_itself() {
        let mut e = Effects::new();
        e.start_swing();
        e.update(SWING_SECONDS / 2.0);
        assert!(e.swing_age.is_some());
        e.update(SWING_SECONDS);
        assert!(e.swing_age.is_none());
    }
}
