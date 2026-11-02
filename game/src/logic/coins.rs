//! 상자 위 코인(D11)과 지갑.
//!
//! - 상자마다 위쪽 가운데에 코인이 하나 떠 있다.
//! - 플레이어가 닿으면 사라지고, COIN_RESPAWN_SECONDS 뒤 같은 자리에 다시 생긴다.
//! - 지갑은 코인을 COIN_MAX개보다 많이 담지 않는다. 상점(M7)에서도 쓴다.


use super::world::World;
use crate::config::{COIN_GAP_ABOVE_BOX, COIN_MAX, COIN_RESPAWN_SECONDS, COIN_SIZE};
use macroquad::math::Rect;

/// 코인 하나.
pub struct Coin {
    /// 코인이 차지하는 네모 (왼쪽 위 기준). 그림은 이 안에 동그라미로 그린다.
    pub rect: Rect,
    /// 다시 생길 때까지 남은 초. 0이면 지금 보인다.
    respawn_seconds_left: f32,
}

impl Coin {
    /// 상자 `box_` 위쪽 가운데에 놓인 코인.
    pub fn above_box(box_: Rect) -> Coin {
        let x = box_.x + (box_.w - COIN_SIZE) / 2.0;
        let y = box_.y - COIN_GAP_ABOVE_BOX - COIN_SIZE;
        Coin { rect: Rect::new(x, y, COIN_SIZE, COIN_SIZE), respawn_seconds_left: 0.0 }
    }

    /// 지금 보이는지 (주울 수 있는지).
    pub fn is_visible(&self) -> bool {
        self.respawn_seconds_left <= 0.0
    }
}

/// 맵에 있는 코인 전부.
pub struct Coins {
    pub list: Vec<Coin>,
}

impl Coins {
    /// 월드의 상자마다 코인 하나씩.
    pub fn on_boxes(world: &World) -> Coins {
        Coins { list: world.obstacles.iter().map(|b| Coin::above_box(*b)).collect() }
    }

    /// 시간을 흘리고, 플레이어 몸(`player_rect`)에 닿은 코인을 줍는다.
    /// 돌려주는 값: 이번에 주운 코인 개수.
    pub fn update(&mut self, dt: f32, player_rect: Rect) -> u32 {
        let mut picked = 0;
        for coin in &mut self.list {
            if !coin.is_visible() {
                coin.respawn_seconds_left = (coin.respawn_seconds_left - dt.max(0.0)).max(0.0);
            }
            if coin.is_visible() && touches(player_rect, coin.rect) {
                coin.respawn_seconds_left = COIN_RESPAWN_SECONDS;
                picked += 1;
            }
        }
        picked
    }
}

/// 코인 지갑. 0 ~ COIN_MAX개.
#[derive(Debug, Clone, Default)]
pub struct Wallet {
    coins: u32,
}

impl Wallet {
    pub fn new() -> Wallet {
        Wallet { coins: 0 }
    }

    /// 지금 가진 코인 수.
    pub fn coins(&self) -> u32 {
        self.coins
    }

    /// 코인을 넣는다. 꽉 차면 넘치는 만큼은 버린다. 돌려주는 값: 실제로 들어간 개수.
    pub fn add(&mut self, amount: u32) -> u32 {
        let room = COIN_MAX - self.coins;
        let added = amount.min(room);
        self.coins += added;
        added
    }

    /// 코인을 쓴다. 모자라면 쓰지 않고 false.
    // 상점(M7)에서 쓸 함수라 아직 부르는 곳이 없다. 상점을 붙이면 이 줄을 지운다.
    #[allow(dead_code)]
    pub fn spend(&mut self, amount: u32) -> bool {
        if amount > self.coins {
            return false;
        }
        self.coins -= amount;
        true
    }
}

/// 두 네모가 겹치는지. 모서리끼리 딱 닿아도 주운 것으로 친다 (줍기는 너그럽게).
fn touches(a: Rect, b: Rect) -> bool {
    a.x <= b.x + b.w && b.x <= a.x + a.w && a.y <= b.y + b.h && b.y <= a.y + a.h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one_box_world() -> World {
        World { width: 960.0, ground_y: 440.0, obstacles: vec![Rect::new(300.0, 392.0, 48.0, 48.0)] }
    }

    /// 코인과 딱 겹치는 몸.
    fn body_on(coin: &Coin) -> Rect {
        coin.rect
    }

    /// 아무 코인에도 안 닿는 먼 곳.
    fn far_away() -> Rect {
        Rect::new(0.0, 0.0, 10.0, 10.0)
    }

    #[test]
    fn one_coin_above_each_box_centered() {
        let w = World::m1();
        let coins = Coins::on_boxes(&w);
        assert_eq!(coins.list.len(), w.obstacles.len());
        for (coin, b) in coins.list.iter().zip(&w.obstacles) {
            assert!(coin.is_visible());
            assert_eq!(coin.rect.x + coin.rect.w / 2.0, b.x + b.w / 2.0, "가운데");
            assert!(coin.rect.y + coin.rect.h < b.y, "상자 윗면보다 위");
        }
    }

    #[test]
    fn picking_a_coin_counts_one_and_hides_it() {
        let mut coins = Coins::on_boxes(&one_box_world());
        let body = body_on(&coins.list[0]);
        assert_eq!(coins.update(1.0 / 60.0, body), 1);
        assert!(!coins.list[0].is_visible());
    }

    #[test]
    fn cannot_pick_while_hidden_then_comes_back_after_10_seconds() {
        let mut coins = Coins::on_boxes(&one_box_world());
        let body = body_on(&coins.list[0]);
        assert_eq!(coins.update(0.0, body), 1);
        // 계속 그 자리에 서 있어도 9.9초 동안은 못 줍는다.
        let mut picked = 0;
        for _ in 0..99 {
            picked += coins.update(0.1, body);
        }
        assert_eq!(picked, 0, "사라진 동안은 못 줍는다");
        assert!(!coins.list[0].is_visible());
        // 멀리 가서 10초를 채우면 다시 보인다.
        assert_eq!(coins.update(0.2, far_away()), 0);
        assert!(coins.list[0].is_visible(), "10초 뒤 다시 생긴다");
        assert_eq!(coins.list[0].rect, Coin::above_box(one_box_world().obstacles[0]).rect, "같은 자리");
        assert_eq!(coins.update(0.0, body), 1, "다시 주울 수 있다");
    }

    #[test]
    fn big_dt_respawns_too() {
        let mut coins = Coins::on_boxes(&one_box_world());
        let body = body_on(&coins.list[0]);
        coins.update(0.0, body);
        assert_eq!(coins.update(COIN_RESPAWN_SECONDS * 5.0, far_away()), 0);
        assert!(coins.list[0].is_visible());
    }

    #[test]
    fn wallet_adds_and_spends() {
        let mut w = Wallet::new();
        assert_eq!(w.coins(), 0);
        assert_eq!(w.add(3), 3);
        assert!(w.spend(2));
        assert_eq!(w.coins(), 1);
        assert!(!w.spend(5), "모자라면 못 쓴다");
        assert_eq!(w.coins(), 1);
    }

    #[test]
    fn wallet_never_goes_over_5000() {
        let mut w = Wallet::new();
        assert_eq!(w.add(4999), 4999);
        assert_eq!(w.add(10), 1, "꽉 찰 만큼만 들어간다");
        assert_eq!(w.coins(), COIN_MAX);
        assert_eq!(w.add(1), 0);
        assert_eq!(w.coins(), 5000);
        assert_eq!(w.add(u32::MAX), 0);
        assert_eq!(w.coins(), 5000);
    }
}
