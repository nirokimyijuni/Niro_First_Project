//! 코인 그림(D11)과 주울 때 반짝 효과. 코인이 있는 곳·주운 개수는 logic(coins)이 정하고,
//! 여기서는 보여 주기만 한다. 위아래로 둥실 뜨는 것도 그림만 움직인다 (줍는 네모는 그대로).

use super::places::near_view;
use super::text::TextPainter;
use crate::logic::coins::Coins;
use crate::logic::world::{nearest_copy, shortest_dx};
use macroquad::prelude::*;

const GOLD: Color = Color::new(1.0, 0.80, 0.16, 1.0);
const GOLD_DARK: Color = Color::new(0.72, 0.48, 0.05, 1.0);
const GOLD_INNER: Color = Color::new(1.0, 0.88, 0.36, 1.0);
const SHINE: Color = Color::new(1.0, 1.0, 0.92, 0.9);
/// 밤에 코인 둘레에 은은한 빛 (어두워도 잘 보이게)
const NIGHT_GLOW: Color = Color::new(1.0, 0.85, 0.30, 0.28);
/// "+1" 글자 색
const POP_TEXT: Color = Color::new(1.0, 0.86, 0.22, 1.0);

/// 코인이 위아래로 떠 있는 폭 (픽셀)과 빠르기
const BOB_HEIGHT: f32 = 3.0;
const BOB_SPEED: f32 = 3.0;
/// 반짝 효과가 보이는 시간 (초). 빨리감기와 상관없이 진짜 시간으로 흐른다.
const POP_SECONDS: f32 = 0.8;
/// "+1"이 떠오르는 높이 (픽셀)
const POP_RISE: f32 = 40.0;

/// 금색 동그라미 코인 하나: 진한 금색 테두리 → 금색 → 안쪽 동그라미 → 반짝 하이라이트.
/// `darkness`(0 낮 ~ 1 밤)가 크면 조금만 어두워지고 둘레가 은은히 빛난다.
pub fn draw_coin(center: Vec2, radius: f32, darkness: f32, time: f32) {
    let d = darkness.clamp(0.0, 1.0);
    let dim = |c: Color| Color::new(c.r * (1.0 - 0.2 * d), c.g * (1.0 - 0.2 * d), c.b * (1.0 - 0.1 * d), c.a);
    if d > 0.0 {
        draw_circle(center.x, center.y, radius * 1.6, Color { a: NIGHT_GLOW.a * d, ..NIGHT_GLOW });
    }
    draw_circle(center.x, center.y, radius, dim(GOLD_DARK));
    draw_circle(center.x, center.y, radius * 0.82, dim(GOLD));
    draw_circle_lines(center.x, center.y, radius * 0.55, (radius * 0.1).max(1.0), dim(GOLD_DARK));
    draw_circle(center.x, center.y, radius * 0.45, dim(GOLD_INNER));
    // 왼쪽 위 하이라이트
    draw_circle(center.x - radius * 0.35, center.y - radius * 0.35, radius * 0.18, SHINE);
    // 가끔 반짝: 오른쪽 위에 네 갈래 별이 커졌다 작아진다
    let twinkle = ((time * 2.2).sin() * 0.5 + 0.5).powi(4);
    if twinkle > 0.05 {
        let p = center + vec2(radius * 0.6, -radius * 0.6);
        let len = radius * 0.7 * twinkle;
        let c = Color { a: twinkle, ..SHINE };
        draw_line(p.x - len, p.y, p.x + len, p.y, 2.0, c);
        draw_line(p.x, p.y - len, p.x, p.y + len, 2.0, c);
    }
}

/// 맵 위 코인 전부. 보이는(`is_visible`) 코인만 `view_x`(화면 가운데)에 가장 가까운 자리에 그린다.
pub fn draw_coins(coins: &Coins, darkness: f32, time: f32, view_x: f32) {
    for (i, coin) in coins.list.iter().enumerate() {
        if !coin.is_visible() {
            continue;
        }
        let t = time + i as f32 * 0.9;
        let bob = (t * BOB_SPEED).sin() * BOB_HEIGHT;
        let center = nearest_copy(coin.rect, view_x).center() + vec2(0.0, bob);
        draw_coin(center, coin.rect.w.min(coin.rect.h) / 2.0, darkness, t);
    }
}

/// 코인을 주운 자리에 잠깐 보이는 반짝 효과와 "+1".
#[derive(Default)]
pub struct CoinPops {
    /// (자리, 지난 시간)
    list: Vec<(Vec2, f32)>,
}

impl CoinPops {
    /// 이번 프레임에 주운 코인 자리에 효과를 띄운다.
    /// `visible_before`: 업데이트 전에 각 코인이 보였는지. `picked`: logic이 알려 준 주운 개수.
    /// 보이던 코인이 사라졌으면 그 자리에. 그래도 모자라면(다시 생기자마자 주운 경우)
    /// 플레이어와 가장 가까운 안 보이는 코인 자리에.
    pub fn spawn_for_picks(&mut self, visible_before: &[bool], coins: &Coins, picked: u32, player_center: Vec2) {
        let mut left = picked;
        for (coin, was_visible) in coins.list.iter().zip(visible_before) {
            if left > 0 && *was_visible && !coin.is_visible() {
                self.list.push((coin.rect.center(), 0.0));
                left -= 1;
            }
        }
        if left > 0 {
            let distance = |at: &Vec2| vec2(shortest_dx(player_center.x, at.x), at.y - player_center.y).length();
            let nearest = coins
                .list
                .iter()
                .filter(|c| !c.is_visible())
                .map(|c| c.rect.center())
                .min_by(|a, b| distance(a).total_cmp(&distance(b)));
            if let Some(pos) = nearest {
                for _ in 0..left {
                    self.list.push((pos, 0.0));
                }
            }
        }
    }

    /// 진짜 흐른 시간 `dt`만큼 효과를 진행하고, 끝난 효과는 지운다.
    pub fn update(&mut self, dt: f32) {
        for (_, age) in &mut self.list {
            *age += dt;
        }
        self.list.retain(|(_, age)| *age < POP_SECONDS);
    }

    pub fn draw(&self, text: &TextPainter, view_x: f32) {
        for (pos, age) in &self.list {
            let pos = vec2(near_view(view_x, pos.x), pos.y);
            let t = (age / POP_SECONDS).clamp(0.0, 1.0);
            let fade = 1.0 - t;
            // 동그랗게 퍼지는 반짝이 8개
            for i in 0..8 {
                let a = i as f32 / 8.0 * std::f32::consts::TAU;
                let p = pos + vec2(a.cos(), a.sin()) * (8.0 + 30.0 * t);
                draw_circle(p.x, p.y, 3.5 * fade + 1.0, Color { a: fade, ..GOLD_INNER });
            }
            text.centered("+1", pos.x, pos.y - 14.0 - POP_RISE * t, 30.0, Color { a: fade, ..POP_TEXT });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::coins::Coins;
    use crate::logic::world::World;

    #[test]
    fn pop_appears_where_a_visible_coin_was_picked() {
        let world = World::m1();
        let mut coins = Coins::on_boxes(&world);
        let before: Vec<bool> = coins.list.iter().map(|c| c.is_visible()).collect();
        let picked = coins.update(0.0, coins.list[1].rect);
        assert_eq!(picked, 1);
        let mut pops = CoinPops::default();
        pops.spawn_for_picks(&before, &coins, picked, Vec2::ZERO);
        assert_eq!(pops.list.len(), 1);
        assert_eq!(pops.list[0].0, coins.list[1].rect.center());
    }

    #[test]
    fn pops_go_away_after_a_moment() {
        let mut pops = CoinPops::default();
        pops.list.push((Vec2::ZERO, 0.0));
        pops.update(POP_SECONDS / 2.0);
        assert_eq!(pops.list.len(), 1);
        pops.update(POP_SECONDS);
        assert_eq!(pops.list.len(), 0);
    }
}
