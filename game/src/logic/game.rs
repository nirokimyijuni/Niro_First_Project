//! 게임 한 판의 상태를 한데 묶는다: 플레이어, 맵, 낮/밤 시계, 코인, 지갑, 싸움(체력·레벨·몬스터).
//! Esc로 나갔다가 새 게임을 하면 `Game::new`로 새로 만든다.


use super::character::CharacterKind;
use super::citizen::Citizens;
use super::clock::{Clock, ClockEvent};
use super::coins::{Coins, Wallet};
use super::combat::{is_reached, is_reached_on_either_side, Combat, CombatEvent};
use super::crafting::Crafting;
use super::gathering::{Gathering, Material, Mineable};
use super::math::{MathQuiz, MathResult};
use super::miss::Miss;
use super::player::{Facing, Player, PlayerInput};
use super::weapon::{Weapon, Weapons};
use super::world::{really_overlaps, shortest_dx, World};
use crate::config::{
    CAVE_AWAY_Y, CAVE_ENTER_BUTTON_TEXT, CAVE_ENTRANCE_X, CAVE_EXIT_BUTTON_TEXT, CAVE_NAME, CAVE_OPEN_AT_NIGHT, CRAFTING_PAUSES_GAME,
    DAYS_TO_CLEAR, MATH_ITEMS_PER_QUESTION, MATH_WRONG_HP_LOSS, MINE_BUTTON_TEXT, MINE_REACH, PLAYER_HEIGHT, PLAYER_SPAWN_X,
    PLAYER_WIDTH, SCORE_PER_COIN, SCORE_PER_LEVEL_UP, SCORE_PER_MONSTER, WORKBENCH_BUTTON_TEXT,
};
use macroquad::math::{Rect, Vec2};

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
    /// 미스를 찾았다 (`MISS_FOUND_TEXT`). `at`은 미스가 있던 몸 가운데. 무기가 떨어지기 시작한다.
    MissFound { at: Vec2 },
    /// 새 무기를 주웠다 (`Weapon::picked_text`). 이미 가진 무기를 또 주우면 오지 않는다.
    WeaponPicked { kind: Weapon },
    /// 폭발형 시민이 터졌다 (`CITIZEN_POP_TEXT`). `at`은 시민 몸 가운데.
    CitizenPopped { at: Vec2 },
    /// 버틴 날이 `DAYS_TO_CLEAR`가 되어 게임을 클리어했다 (`GAME_CLEAR_TEXT`). 한 판에 한 번만 온다.
    GameCleared,
}

/// 캐기·동굴·제작대에서 생긴 일. `GameEvent`와 따로 `Game::take_gather_events`로 받는다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GatherEvent {
    /// 다 캤다 (`Material::mined_text`). `at`은 캔 것의 가운데다. 동굴 안이면 동굴 좌표다.
    Mined { what: Material, at: Vec2 },
    /// 동굴에 들어갔다.
    CaveEntered,
    /// 출구로 나왔다.
    CaveLeft,
    /// 밤이 되어 입구 앞으로 나왔다 (`CAVE_NIGHT_OUT_TEXT`).
    PushedOutOfCave,
    /// 밤이라 들어가지 못했다 (`CAVE_CLOSED_TEXT`).
    CaveClosed,
    /// 제작대를 열었다.
    CraftingOpened,
}

/// E 키와 캐기 버튼으로 쓰는 것 (임시-17, 임시-36). 번호는 `gathering.trees`·`gathering.ore_lumps` 순서다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Usable {
    Tree(usize),
    OreLump(usize),
    CaveEntrance,
    CaveExit,
    Workbench,
}

impl Usable {
    /// 폰 버튼에 쓰는 글 (임시-36).
    pub fn button_text(self) -> &'static str {
        match self {
            Usable::Tree(_) | Usable::OreLump(_) => MINE_BUTTON_TEXT,
            Usable::CaveEntrance => CAVE_ENTER_BUTTON_TEXT,
            Usable::CaveExit => CAVE_EXIT_BUTTON_TEXT,
            Usable::Workbench => WORKBENCH_BUTTON_TEXT,
        }
    }
}

pub struct Game {
    pub player: Player,
    pub world: World,
    pub clock: Clock,
    pub coins: Coins,
    pub wallet: Wallet,
    pub combat: Combat,
    /// 미스. 긴 맵에만 있고, M1 연습 맵은 None이다.
    pub miss: Option<Miss>,
    pub weapons: Weapons,
    pub citizens: Citizens,
    /// 미스의 수학 문제 (US-074~076). 열려 있는 동안 게임 속 시간이 모두 멈춘다.
    pub math: MathQuiz,
    cleared: bool,
    /// 나무, 광석 덩이, 재료와 곡괭이 (`gathering.bag`).
    pub gathering: Gathering,
    pub crafting: Crafting,
    /// 주인공이 동굴 안이면 true. 이때 `player`는 `cave_world` 좌표다.
    pub in_cave: bool,
    pub cave_world: World,
    /// E 키나 캐기 버튼을 누르고 있으면 true. 화면이 `update` 전에 프레임마다 채운다.
    pub mine_held: bool,
    mine_was_held: bool,
    gather_events: Vec<GatherEvent>,
}

impl Game {
    /// 고른 캐릭터로 새 게임. 1일째 낮, 코인 0개, 상자마다 코인 하나, 맨손.
    /// `seed`가 같으면 상자 배치, 미스 자리, 나오는 몬스터, 나오는 쪽, 날아오르는 좀비가 늘 똑같다.
    pub fn new(kind: CharacterKind, seed: u32) -> Game {
        Game::on_world(kind, seed, World::long_map(seed))
    }

    /// 정해진 맵으로 새 게임. 테스트는 `World::m1()`을 넣어 상자 자리를 고정한다.
    pub fn on_world(kind: CharacterKind, seed: u32, world: World) -> Game {
        let spawn = Vec2::new(PLAYER_SPAWN_X, world.ground_y - PLAYER_HEIGHT);
        let cave_world = World::cave();
        Game {
            gathering: Gathering::on_world(&world, &cave_world, seed),
            crafting: Crafting::default(),
            in_cave: false,
            cave_world,
            mine_held: false,
            mine_was_held: false,
            gather_events: Vec::new(),
            player: Player::new(kind, spawn),
            coins: Coins::on_boxes(&world),
            miss: world.is_long_map().then(|| Miss::new(seed)),
            weapons: Weapons::new(seed),
            citizens: Citizens::on_world(&world, seed),
            math: MathQuiz::new(seed),
            cleared: false,
            world,
            clock: Clock::new(),
            wallet: Wallet::new(),
            combat: Combat::new(seed),
        }
    }

    /// 한 프레임 진행. 빨리감기와 같은 길을 `time_scale` 1로 지난다.
    /// 쓰러지거나 클리어한 뒤에는 시계·플레이어·코인·좀비 모두 멈춘다 (US-043, US-068).
    pub fn update(&mut self, input: &PlayerInput, dt: f32) -> Vec<GameEvent> {
        self.update_with_time_scale(input, dt, 1.0)
    }

    /// 빨리감기 한 프레임: 걷기·점프·몬스터는 진짜 시간 `dt`, 시계와 코인은 `dt × time_scale`로 흘린다 (D13).
    /// 수학 문제가 열려 있으면 결과 글 시간만 흐르고 나머지는 모두 멈춘다 (임시-31). 빨리감기여도 같다.
    /// 제작대가 열려 있으면 모두 멈춘다 (임시-18, 임시-37: 18:00도 오지 않는다).
    pub fn update_with_time_scale(&mut self, input: &PlayerInput, dt: f32, time_scale: f32) -> Vec<GameEvent> {
        let mut events = Vec::new();
        let crafting = self.crafting.is_open();
        if self.is_over() || (crafting && CRAFTING_PAUSES_GAME) {
            self.forget_mining();
            return events;
        }
        if self.math.is_open() {
            self.forget_mining();
            self.update_combat(input, dt, &mut events);
            return events;
        }
        let world_dt = dt * time_scale;
        let stand_still = PlayerInput::default();
        let input = if crafting { &stand_still } else { input };

        self.player.update(input, dt, if self.in_cave { &self.cave_world } else { &self.world });

        for clock_event in self.clock.update(world_dt) {
            events.push(match clock_event {
                ClockEvent::NightStarted => GameEvent::NightStarted,
                ClockEvent::DayStarted { day_number } => GameEvent::DayStarted { day_number },
            });
        }

        // 밤 문제(update_combat)보다 먼저 입구 앞으로 나온다 (임시-37).
        if self.in_cave && !self.cave_is_open() {
            self.leave_cave();
            self.gather_events.push(GatherEvent::PushedOutOfCave);
        }

        let picked = self.coins.update(world_dt, self.body_on_map());
        if picked > 0 {
            self.wallet.add(picked);
            events.push(GameEvent::CoinsPicked { count: picked });
        }

        self.update_combat(input, dt, &mut events);
        self.update_gathering(dt, &events);
        events
    }

    /// 공격 버튼을 처리하고 몬스터·미스·무기·시민 시간을 `dt`만큼 흘린다. 시계가 움직인 뒤에 불러야
    /// 이번 프레임의 낮/밤을 본다. 빨리감기여도 `dt`는 진짜 시간이다 (D13: 빨라지는 건 시계·코인뿐).
    /// 빨리감기도 이 함수를 지나서, 클리어와 수학 문제 열기도 여기서 본다.
    pub fn update_combat(&mut self, input: &PlayerInput, dt: f32, events: &mut Vec<GameEvent>) {
        if self.cleared {
            return;
        }
        if self.math.is_open() {
            self.wait_after_math_answer(dt, events);
            return;
        }
        // 밤 문제는 첫 몬스터보다 먼저 연다 (BUG-025): 18:00 프레임에는 몬스터를 움직이지 않는다.
        if events.contains(&GameEvent::NightStarted) {
            self.math.add_waiting();
            self.math.open_next_waiting();
            if self.math.is_open() {
                return;
            }
        }
        let player_rect = self.body_on_map();
        if input.attack_pressed {
            let held = self.weapons.held();
            let reach = self.attack_reach();
            let hits = self.combat.attack_with(player_rect, self.player.facing, held.attack(), reach);
            events.extend(hits.into_iter().map(GameEvent::Combat));
            let facing = self.player.facing;
            let knocked_down = self.citizens.knock_down_reached(|body| is_reached(player_rect, facing, reach, body));
            events.extend(knocked_down.into_iter().map(|at| GameEvent::Combat(CombatEvent::CitizenDefeated { at })));
        }
        let happened = self.combat.update(dt, self.clock.is_night(), player_rect, &self.world);
        events.extend(happened.into_iter().map(GameEvent::Combat));
        if !self.combat.health.is_down() {
            self.update_miss_and_weapons(dt, events);
            self.update_citizens(dt, events);
        }
        // 펑으로도 쓰러질 수 있어서 시민을 움직인 뒤에 다시 본다.
        if self.combat.health.is_down() {
            self.citizens.stop_walkers();
        } else {
            self.citizens.walk(dt, &self.world);
        }

        // 싸움이 끝난 뒤에 본다: 같은 순간 쓰러졌으면 쓰러짐이 먼저다.
        if !self.combat.health.is_down() && self.clock.nights_survived() >= DAYS_TO_CLEAR {
            self.cleared = true;
            events.push(GameEvent::GameCleared);
        }

        // 판이 끝난 순간(쓰러짐·클리어)과 동굴 안에서는 문제를 내지 않는다 (임시-37).
        if !self.is_over() && !self.in_cave {
            self.math.open_next_waiting();
        }
    }

    /// 결과 글 시간을 흘린다. 틀린 문제가 닫히는 순간 체력이 준다. 0이면 보통 쓰러짐과 같다 (부모님 답).
    /// 무적 시간은 생기지 않는다. 기다리는 문제가 있으면 다음 프레임에 연다.
    fn wait_after_math_answer(&mut self, dt: f32, events: &mut Vec<GameEvent>) {
        if self.math.wait_after_answer(dt) != Some(MathResult::Wrong) {
            return;
        }
        let before = self.combat.health.hp();
        self.combat.health.lose_from_quiz(MATH_WRONG_HP_LOSS);
        let hp_left = self.combat.health.hp();
        events.push(GameEvent::Combat(CombatEvent::PlayerHurt { amount: before - hp_left, hp_left }));
        if self.combat.health.is_down() {
            self.citizens.stop_walkers();
            events.push(GameEvent::Combat(CombatEvent::PlayerDown));
        }
    }

    /// 지금 공격이 앞쪽으로 닿는 거리. 총은 첫 벽에서 막히고, 맨손·칼·도끼는 그대로다.
    /// 화면은 `attack_box_with_range(몸, facing, 이 값)`으로 빛줄기 길이를 그린다.
    pub fn attack_reach(&self) -> f32 {
        let held = self.weapons.held();
        if held == Weapon::Gun {
            self.here().clear_reach(self.player.rect(), self.player.facing, held.range())
        } else {
            held.range()
        }
    }

    /// 클리어했으면 true. 쓰러짐은 `combat.health.is_down()`이고, 둘이 함께 true인 일은 없다.
    pub fn is_cleared(&self) -> bool {
        self.cleared
    }

    /// 쓰러졌거나 클리어해서 판이 끝났으면 true.
    pub fn is_over(&self) -> bool {
        self.cleared || self.combat.health.is_down()
    }

    /// 이번 판 점수: 주운 코인 + 레벨업 횟수 + 해치운 몬스터 (D64). 가중치는 config에 있다.
    pub fn score(&self) -> u32 {
        self.coins.picked_total * SCORE_PER_COIN
            + (self.combat.level - 1) * SCORE_PER_LEVEL_UP
            + self.combat.monsters_defeated * SCORE_PER_MONSTER
    }

    /// 카메라 왼쪽 끝 x. 로건은 그릴 때 월드 x에서 이 값을 뺀다.
    /// 동굴 안이면 동굴 화면이라 늘 0이다.
    pub fn camera_left(&self) -> f32 {
        self.here().camera_left(self.player.rect().center().x)
    }

    /// 주인공이 있는 장소 이름.
    pub fn place_name(&self) -> &'static str {
        if self.in_cave {
            return CAVE_NAME;
        }
        self.world.place_name_at(self.player.rect().center().x)
    }

    /// 주인공이 지금 걷는 곳: 동굴 안이면 `cave_world`, 아니면 `world`.
    pub fn here(&self) -> &World {
        if self.in_cave {
            &self.cave_world
        } else {
            &self.world
        }
    }

    /// W 키나 바꾸기 버튼을 "새로" 눌렀을 때 부른다 (US-053).
    pub fn switch_weapon(&mut self) {
        self.weapons.switch();
    }

    /// 지난번 뒤로 생긴 캐기·동굴·제작대 일을 모두 받는다. 화면이 `update` 뒤에 부른다.
    pub fn take_gather_events(&mut self) -> Vec<GatherEvent> {
        std::mem::take(&mut self.gather_events)
    }

    /// 캐는 중이면 0~1. 머리 위 표시에 쓴다. 캐지 않으면 None이다.
    pub fn mining_progress(&self) -> Option<f32> {
        self.gathering.progress()
    }

    /// 지금 E 키나 캐기 버튼을 누르면 쓸 것: 몸 양쪽(등 뒤 포함) `MINE_REACH` 안에서 가장 가까운 하나 (임시-16, 임시-36).
    /// 거리가 같으면 바라보는 쪽을 먼저 쓴다.
    /// 판이 끝났거나 수학 문제·제작대가 열려 있으면 None이다. 폰 버튼은 Some일 때만 보인다 (글은 `button_text`).
    pub fn usable_in_reach(&self) -> Option<Usable> {
        if self.is_over() || self.math.is_open() || self.crafting.is_open() {
            return None;
        }
        let ready = |list: &[Mineable]| -> Vec<(usize, Rect)> {
            list.iter().enumerate().filter(|(_, m)| m.is_ready()).map(|(index, m)| (index, m.rect)).collect()
        };
        let mut spots: Vec<(Usable, Rect)> = Vec::new();
        if self.in_cave {
            spots.push((Usable::CaveExit, self.cave_world.cave_exit()));
            spots.extend(ready(&self.gathering.ore_lumps).into_iter().map(|(i, rect)| (Usable::OreLump(i), rect)));
        } else {
            spots.extend(self.world.cave_entrance().map(|rect| (Usable::CaveEntrance, rect)));
            spots.extend(self.world.workbench().map(|rect| (Usable::Workbench, rect)));
            spots.extend(ready(&self.gathering.trees).into_iter().map(|(i, rect)| (Usable::Tree(i), rect)));
        }
        let body = self.player.rect();
        let dx = |rect: &Rect| shortest_dx(body.center().x, rect.center().x);
        let is_behind = |rect: &Rect| match self.player.facing {
            Facing::Right => dx(rect) < 0.0,
            Facing::Left => dx(rect) > 0.0,
        };
        spots
            .into_iter()
            .filter(|(_, rect)| is_reached_on_either_side(body, MINE_REACH, *rect))
            .min_by(|(_, a), (_, b)| dx(a).abs().total_cmp(&dx(b).abs()).then(is_behind(a).cmp(&is_behind(b))))
            .map(|(thing, _)| thing)
    }

    /// 동굴에 있어도 되는 때인지 (임시-13).
    fn cave_is_open(&self) -> bool {
        CAVE_OPEN_AT_NIGHT || !self.clock.is_night()
    }

    /// 맵의 코인·미스·시민·몬스터가 보는 주인공 몸. 동굴에 있으면 입구 위 먼 하늘이라 아무것도 닿지 않는다.
    fn body_on_map(&self) -> Rect {
        if self.in_cave {
            Rect::new(CAVE_ENTRANCE_X, CAVE_AWAY_Y, PLAYER_WIDTH, PLAYER_HEIGHT)
        } else {
            self.player.rect()
        }
    }

    fn use_thing(&mut self, thing: Usable) {
        match thing {
            Usable::Tree(index) | Usable::OreLump(index) => self.gathering.start_mining(index),
            Usable::CaveEntrance if self.cave_is_open() => {
                self.in_cave = true;
                let exit_x = self.cave_world.cave_exit().center().x;
                self.put_player_on_ground(exit_x);
                self.player.facing = Facing::Right;
                self.gather_events.push(GatherEvent::CaveEntered);
            }
            Usable::CaveEntrance => self.gather_events.push(GatherEvent::CaveClosed),
            Usable::CaveExit => {
                self.leave_cave();
                self.gather_events.push(GatherEvent::CaveLeft);
            }
            Usable::Workbench => {
                self.crafting.open();
                self.gather_events.push(GatherEvent::CraftingOpened);
            }
        }
    }

    fn leave_cave(&mut self) {
        self.in_cave = false;
        self.gathering.stop_mining();
        self.put_player_on_ground(CAVE_ENTRANCE_X);
    }

    /// 몸 가운데가 `center_x`이도록 땅에 세운다. 동굴과 맵의 땅 높이는 같다.
    fn put_player_on_ground(&mut self, center_x: f32) {
        self.player.pos = Vec2::new(center_x - self.player.size.x / 2.0, self.world.ground_y - self.player.size.y);
        self.player.vel = Vec2::ZERO;
    }

    /// 멈춘 동안: 캐던 것은 멈추고, 그동안 누르고 있던 버튼은 "새로 누름"으로 치지 않는다.
    fn forget_mining(&mut self) {
        self.gathering.stop_mining();
        self.mine_was_held = self.mine_held;
    }

    /// 캐기 버튼을 처리하고 나무와 덩이 시간(실제 시간)을 흘린다. 싸움이 끝난 뒤에 불러야 이번 프레임에 맞은 것을 본다.
    /// 시작은 "새로" 눌렀을 때뿐이라 꾹 누르고 있으면 다음 것을 캐지 않는다.
    /// 캐기와 곡괭이는 수학 문제 "아이템"으로 세지 않는다 (임시-37).
    fn update_gathering(&mut self, dt: f32, happened: &[GameEvent]) {
        self.gathering.grow_back(dt);
        let newly_pressed = self.mine_held && !self.mine_was_held;
        self.mine_was_held = self.mine_held;

        let got_hurt = happened.iter().any(|e| matches!(e, GameEvent::Combat(CombatEvent::PlayerHurt { .. })));
        if !self.mine_held || got_hurt || self.is_over() || self.math.is_open() {
            self.gathering.stop_mining();
            return;
        }
        if newly_pressed {
            if let Some(thing) = self.usable_in_reach() {
                self.use_thing(thing);
                if !matches!(thing, Usable::Tree(_) | Usable::OreLump(_)) {
                    return;
                }
            }
        }
        let mined = self.gathering.keep_mining(dt, self.in_cave, self.player.rect());
        if let Some((what, at)) = mined {
            self.gather_events.push(GatherEvent::Mined { what, at });
        }
    }

    fn update_miss_and_weapons(&mut self, dt: f32, events: &mut Vec<GameEvent>) {
        let player_rect = self.body_on_map();
        if let Some(miss) = &mut self.miss {
            miss.wait_to_come_back(dt, player_rect, &self.world);
            if miss.is_hiding() && really_overlaps(miss.rect(&self.world), player_rect) {
                let at = miss.rect(&self.world).center();
                miss.get_found();
                events.push(GameEvent::MissFound { at });
                self.weapons.drop_one(at.x, &self.world);
            }
        }
        let count_before = self.weapons.picked_up_count;
        let picked = self.weapons.update(dt, player_rect);
        // 이미 가진 무기를 또 주워도 아이템 1개다. `MATH_ITEMS_PER_QUESTION`번째마다 문제가 나온다.
        for count in count_before + 1..=self.weapons.picked_up_count {
            if count % MATH_ITEMS_PER_QUESTION == 0 {
                self.math.add_waiting();
            }
        }
        events.extend(picked.into_iter().map(|kind| GameEvent::WeaponPicked { kind }));
    }

    fn update_citizens(&mut self, dt: f32, events: &mut Vec<GameEvent>) {
        let player_rect = self.body_on_map();
        for citizen in &mut self.citizens.list {
            let touched = really_overlaps(citizen.rect, player_rect) || self.combat.any_monster_touches(citizen.rect);
            if citizen.update(dt, touched) {
                let at = citizen.rect.center();
                events.push(GameEvent::CitizenPopped { at });
                let happened = self.combat.citizen_blast(at, player_rect);
                events.extend(happened.into_iter().map(GameEvent::Combat));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{COIN_RESPAWN_SECONDS, DAY_SECONDS, GROUND_Y, NIGHT_SECONDS, WORLD_WIDTH};
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
        g.combat.skeleton_chance = 0.0; // 절반 스켈레톤의 손은 얇은 벽 너머로 닿는다
        let events = g.update(&no_input(), DAY_SECONDS);
        assert!(events.contains(&GameEvent::NightStarted));
        assert!(g.clock.is_night());
        // 밤이 되면 수학 문제가 열린다 (부모님 답). 맞히고 닫은 뒤에 밤을 보낸다.
        let answer = g.math.question().expect("밤 문제").answer;
        g.math.type_and_submit(answer);
        g.update(&no_input(), crate::config::MATH_RIGHT_SHOW_SECONDS);
        assert!(!g.math.is_open());
        let events = g.update(&no_input(), NIGHT_SECONDS);
        assert!(events.contains(&GameEvent::DayStarted { day_number: 2 }));
        assert_eq!(g.clock.nights_survived(), 1);
    }
}
