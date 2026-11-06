//! 게임 한 판의 상태를 한데 묶는다: 플레이어, 맵, 낮/밤 시계, 코인, 지갑, 싸움(체력·레벨·몬스터).
//! Esc로 나갔다가 새 게임을 하면 `Game::new`로 새로 만든다.


use super::character::CharacterKind;
use super::clock::{Clock, ClockEvent};
use super::coins::{Coins, Wallet};
use super::combat::{Combat, CombatEvent};
use super::player::{Player, PlayerInput};
use super::world::World;
use crate::config::{PLAYER_HEIGHT, PLAYER_SPAWN_X};
use macroquad::math::Vec2;

/// 화면(로건)이 보고 팝업·소리를 낼 일.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameEvent {
    /// 밤이 시작됐다.
    NightStarted,
    /// 밤을 버티고 새 날 아침이 왔다.
    DayStarted { day_number: u32 },
    /// 코인을 주웠다 (`count`개, 보통 1).
    CoinsPicked { count: u32 },
    /// 몬스터·체력·레벨에 생긴 일.
    Combat(CombatEvent),
}

pub struct Game {
    pub player: Player,
    pub world: World,
    pub clock: Clock,
    pub coins: Coins,
    pub wallet: Wallet,
    pub combat: Combat,
}

impl Game {
    /// 고른 캐릭터로 새 게임. 1일째 낮, 코인 0개, 상자마다 코인 하나.
    /// `seed`가 같으면 상자 배치와 나오는 몬스터, 나오는 쪽, 날아오르는 좀비가 늘 똑같다.
    pub fn new(kind: CharacterKind, seed: u32) -> Game {
        Game::on_world(kind, seed, World::with_random_boxes(seed))
    }

    /// 정해진 맵으로 새 게임. 테스트는 `World::m1()`을 넣어 상자 자리를 고정한다.
    pub fn on_world(kind: CharacterKind, seed: u32, world: World) -> Game {
        let spawn = Vec2::new(PLAYER_SPAWN_X, world.ground_y - PLAYER_HEIGHT);
        Game {
            player: Player::new(kind, spawn),
            coins: Coins::on_boxes(&world),
            world,
            clock: Clock::new(),
            wallet: Wallet::new(),
            combat: Combat::new(seed),
        }
    }

    /// 한 프레임 진행. 빨리감기는 main.rs `update_with_time_scale`이 같은 순서로 맡는다.
    /// 쓰러진 뒤에는 시계·플레이어·코인·좀비 모두 멈춘다 (US-043).
    pub fn update(&mut self, input: &PlayerInput, dt: f32) -> Vec<GameEvent> {
        let mut events = Vec::new();
        if self.combat.health.is_down() {
            return events;
        }

        self.player.update(input, dt, &self.world);

        for clock_event in self.clock.update(dt) {
            events.push(match clock_event {
                ClockEvent::NightStarted => GameEvent::NightStarted,
                ClockEvent::DayStarted { day_number } => GameEvent::DayStarted { day_number },
            });
        }

        let picked = self.coins.update(dt, self.player.rect());
        if picked > 0 {
            self.wallet.add(picked);
            events.push(GameEvent::CoinsPicked { count: picked });
        }

        self.update_combat(input, dt, &mut events);
        events
    }

    /// 공격 버튼을 처리하고 몬스터 시간을 `dt`만큼 흘린다. 시계가 움직인 뒤에 불러야
    /// 이번 프레임의 낮/밤을 본다. 빨리감기여도 `dt`는 진짜 시간이다 (D13: 빨라지는 건 시계·코인뿐).
    pub fn update_combat(&mut self, input: &PlayerInput, dt: f32, events: &mut Vec<GameEvent>) {
        let player_rect = self.player.rect();
        if input.attack_pressed {
            let hits = self.combat.player_attack(player_rect, self.player.facing);
            events.extend(hits.into_iter().map(GameEvent::Combat));
        }
        let happened = self.combat.update(dt, self.clock.is_night(), player_rect, &self.world);
        events.extend(happened.into_iter().map(GameEvent::Combat));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{COIN_RESPAWN_SECONDS, DAY_SECONDS, GROUND_Y, WORLD_WIDTH};
    use macroquad::math::Rect;

    const FRAME: f32 = 1.0 / 60.0;
    const SEED: u32 = 7;

    /// 상자 자리가 고정된 맵의 새 게임. 여기 규칙들은 랜덤 상자와 상관없다 (D36은 d36_random_boxes.rs).
    fn m1_game(kind: CharacterKind) -> Game {
        Game::on_world(kind, SEED, World::m1())
    }

    fn no_input() -> PlayerInput {
        PlayerInput::default()
    }

    fn run(game: &mut Game, input: PlayerInput, frames: usize) -> Vec<GameEvent> {
        let mut events = Vec::new();
        for _ in 0..frames {
            events.extend(game.update(&input, FRAME));
        }
        events
    }

    #[test]
    fn new_game_starts_fresh() {
        let g = m1_game(CharacterKind::Ellie);
        assert_eq!(g.player.kind, CharacterKind::Ellie);
        assert_eq!(g.clock.day_number(), 1);
        assert!(!g.clock.is_night());
        assert_eq!(g.wallet.coins(), 0);
        assert_eq!(g.coins.list.len(), g.world.obstacles.len());
        assert!(g.coins.list.iter().all(|c| c.is_visible()));
    }

    #[test]
    fn walking_on_ground_does_not_pick_coins() {
        // 상자 옆을 땅으로 걷기만 하면 코인에 안 닿는다 (상자에 막힐 때까지 밀어 본다).
        let mut g = m1_game(CharacterKind::Niro);
        let events = run(&mut g, PlayerInput { right: true, ..Default::default() }, 120);
        assert!(g.player.on_ground);
        assert_eq!(g.wallet.coins(), 0);
        assert!(!events.iter().any(|e| matches!(e, GameEvent::CoinsPicked { .. })));
    }

    #[test]
    fn standing_on_a_box_touches_its_coin() {
        // 진짜 Player 움직임으로: 첫 상자 앞에서 점프해 올라서기.
        let mut g = m1_game(CharacterKind::Niro);
        let box_ = g.world.obstacles[0];
        g.player.pos.x = box_.x - 100.0;
        run(&mut g, no_input(), 10);

        g.update(&PlayerInput { right: true, jump_pressed: true, ..Default::default() }, FRAME);
        let mut events = Vec::new();
        for _ in 0..120 {
            let over_box = g.player.pos.x > box_.x && g.player.pos.x + g.player.size.x < box_.x + box_.w + 10.0;
            let input = if over_box { no_input() } else { PlayerInput { right: true, ..Default::default() } };
            events.extend(g.update(&input, FRAME));
        }
        assert!(g.player.on_ground);
        assert_eq!(g.player.pos.y + g.player.size.y, box_.y, "상자 위에 서 있다");
        assert_eq!(g.wallet.coins(), 1, "상자 위 코인을 주웠다");
        assert!(events.contains(&GameEvent::CoinsPicked { count: 1 }));
        assert!(!g.coins.list[0].is_visible());

        // 그대로 서 있으면 10초 뒤 다시 생기자마자 또 줍는다.
        run(&mut g, no_input(), ((COIN_RESPAWN_SECONDS + 0.5) / FRAME) as usize);
        assert_eq!(g.wallet.coins(), 2);
    }

    #[test]
    fn standing_still_on_box_coin_stays_at_most_one_per_10_seconds() {
        let mut g = m1_game(CharacterKind::Logan);
        let box_ = g.world.obstacles[0];
        // 상자 위에 떨어뜨린다.
        g.player.pos = Vec2::new(box_.x, box_.y - 200.0);
        run(&mut g, no_input(), (35.0 / FRAME) as usize); // 35초
        assert_eq!(g.player.pos.y + g.player.size.y, box_.y);
        assert_eq!(g.wallet.coins(), 4, "0초, 10초, 20초, 30초쯤에 하나씩");
    }

    #[test]
    fn clock_events_come_through_game() {
        // 벽 사이에 넣는다. 밤새 서 있으면 왼쪽 끝 몬스터가 찾아와 쓰러진다 (D36).
        let wall = |x| Rect::new(x, 0.0, 10.0, GROUND_Y);
        let walled = World { width: WORLD_WIDTH, ground_y: GROUND_Y, obstacles: vec![wall(100.0), wall(180.0)] };
        let mut g = Game::on_world(CharacterKind::Niro, SEED, walled);
        let events = g.update(&no_input(), DAY_SECONDS);
        assert!(events.contains(&GameEvent::NightStarted));
        assert!(g.clock.is_night());
        let events = g.update(&no_input(), DAY_SECONDS);
        assert!(events.contains(&GameEvent::DayStarted { day_number: 2 }));
        assert_eq!(g.clock.nights_survived(), 1);
    }
}
