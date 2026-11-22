//! 게임 한 판의 상태를 한데 묶는다: 플레이어, 맵, 낮/밤 시계, 코인, 지갑, 싸움(체력·레벨·몬스터).
//! Esc로 나갔다가 새 게임을 하면 `Game::new`로 새로 만든다.


use super::boxes::{BoxEvent, BoxMonster, Boxes, Surprise, SurpriseItem};
use super::character::CharacterKind;
use super::citizen::Citizens;
use super::clock::{Clock, ClockEvent};
use super::coins::{Coins, Wallet};
use super::combat::{is_in_front_of_door, is_reached, is_reached_on_either_side, Combat, CombatEvent};
use super::crafting::Crafting;
use super::gathering::{Gathering, Material, Mineable};
use super::math::{MathQuiz, MathResult};
use super::miss::Miss;
use super::mission::{Mission, MissionEvent, MissionKind, Missions};
use super::player::{Facing, Player, PlayerInput};
use super::practice::{practice_spawn_x, PracticeEvent, PracticeMonster};
use super::settings::Settings;
use super::tutorial::{Tutorial, TutorialEvent, TutorialFacts};
use super::shop::{sell_price, Armor, BuyRow, CantBuy, CantSell, CantUse, Pocket, Potion, SellRow, Shop, ShopEvent, ShopItem, SELLABLE};
use super::weapon::{Weapon, WeaponEvent, Weapons};
use super::world::{really_overlaps, shortest_dx, World};
use crate::config::{
    CAVE_AWAY_Y, CAVE_ENTER_BUTTON_TEXT, CAVE_ENTRANCE_X, CAVE_EXIT_BUTTON_TEXT, CAVE_NAME, CAVE_OPEN_AT_NIGHT, CRAFTING_PAUSES_GAME,
    DAYS_TO_CLEAR, MATH_ITEMS_PER_QUESTION, MINE_BUTTON_TEXT, MINE_REACH, PLAYER_HEIGHT, PLAYER_SPAWN_X,
    PLAYER_WIDTH, GUN_REACH_MAX_FROM_FRONT, SCORE_PER_COIN, SCORE_PER_LEVEL_UP, SCORE_PER_MONSTER, WORKBENCH_BUTTON_TEXT, HEALTH_POTION_HEAL,
    HOUSE_DOOR_BUTTON_TEXT, PLAYER_WALK_SPEED, SHOP_PAUSES_GAME, CAR_SPEED_MULTIPLIER, DIAMOND_PICKAXE_SLAM_DIAMONDS,
    DIAMOND_PICKAXE_SLAM_DIAMONDS_TEXT, DIAMOND_PICKAXE_SLAM_SCORE, ENDER_FRUIT_SLAM_DIAMONDS, ENDER_FRUIT_SLAM_DIAMONDS_TEXT,
    ENDER_FRUIT_SLAM_SCORE, SLAM_BEACH_LEFT, SLAM_BEACH_RIGHT, SYRINGE_GUN_SCORE, MISSION_FAIL_HP_LOSS, MISSION_REWARD_COINS,
    QUIZ_WAIT_AFTER_DAY_MISSION_FAIL_SECONDS, PHYSICS_STEP_SECONDS, PRACTICE_STEP_LIMIT,
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

/// 바다에 쿵 하는 힘 (임시-42). 둘 다 있으면 엔더열매가 먼저 쓰인다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlamPower {
    EnderFruit,
    DiamondPickaxe,
}

impl SlamPower {
    /// 쿵 하면 얻는 다이아몬드 수.
    pub fn diamonds(self) -> u32 {
        match self {
            SlamPower::EnderFruit => ENDER_FRUIT_SLAM_DIAMONDS,
            SlamPower::DiamondPickaxe => DIAMOND_PICKAXE_SLAM_DIAMONDS,
        }
    }

    /// 몬스터가 1마리 이상 쓰러졌을 때 점수에 한 번 더하는 수 (임시-47).
    pub fn score(self) -> u32 {
        match self {
            SlamPower::EnderFruit => ENDER_FRUIT_SLAM_SCORE,
            SlamPower::DiamondPickaxe => DIAMOND_PICKAXE_SLAM_SCORE,
        }
    }

    /// "다이아몬드를 10개 얻었어요!" 같은 문구.
    pub fn diamonds_text(self) -> &'static str {
        match self {
            SlamPower::EnderFruit => ENDER_FRUIT_SLAM_DIAMONDS_TEXT,
            SlamPower::DiamondPickaxe => DIAMOND_PICKAXE_SLAM_DIAMONDS_TEXT,
        }
    }
}

/// 새 조합 물건(자동차, 주사기총, 엔더열매, 다이아래더라이트 곡괭이)에 생긴 일. `Game::take_item_events`로 받는다.
/// 쓰러진 몬스터마다의 "쓰러졌어요"는 따로 `GameEvent::Combat(ZombieDefeated / SkeletonDefeated)`로 온다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ItemEvent {
    /// 06:00가 되어 자동차에서 내렸다 (`CAR_OFF_TEXT`).
    CarOff,
    /// 주사기총을 쐈다. `at`은 주인공 몸 가운데, `monsters`는 쓰러진 수 (1 이상). 주사기총은 없어졌다.
    SyringeGunFired { at: Vec2, monsters: u32 },
    /// 엔더열매를 먹었다 (`ENDER_FRUIT_EATEN_TEXT`). 엔더열매 힘이 생겼다.
    EnderFruitEaten { at: Vec2 },
    /// 힘이 있는데 또 먹으려 했다 (`ENDER_FRUIT_ALREADY_EATEN_TEXT`). 개수는 그대로다.
    EnderFruitAlreadyEaten,
    /// 바다에 쿵 했다 (`SLAM_TEXT`, 화면 흔들기). `diamonds`개를 얻었다 (`SlamPower::diamonds_text`).
    /// `power`가 `DiamondPickaxe`면 곡괭이가 없어졌다 (`DIAMOND_PICKAXE_GONE_TEXT`). `monsters`는 쓰러진 수 (0일 수 있다).
    Slammed { power: SlamPower, at: Vec2, monsters: u32, diamonds: u32 },
}

/// E 키와 캐기 버튼으로 쓰는 것 (임시-17, 임시-36). 번호는 `gathering.trees`·`gathering.ore_lumps` 순서다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Usable {
    Tree(usize),
    OreLump(usize),
    CaveEntrance,
    CaveExit,
    Workbench,
    /// 집 문 (상점, 임시-22).
    HouseDoor,
}

impl Usable {
    /// 폰 버튼에 쓰는 글 (임시-36).
    pub fn button_text(self) -> &'static str {
        match self {
            Usable::Tree(_) | Usable::OreLump(_) => MINE_BUTTON_TEXT,
            Usable::CaveEntrance => CAVE_ENTER_BUTTON_TEXT,
            Usable::CaveExit => CAVE_EXIT_BUTTON_TEXT,
            Usable::Workbench => WORKBENCH_BUTTON_TEXT,
            Usable::HouseDoor => HOUSE_DOOR_BUTTON_TEXT,
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
    /// 상점 화면과 산 물건 (포션 개수, 스케이트, 갑옷). 열려 있는 동안 게임 속 시간이 모두 멈춘다 (임시-24).
    pub shop: Shop,
    shop_events: Vec<ShopEvent>,
    item_events: Vec<ItemEvent>,
    /// 주사기총처럼 `update` 밖에서 생긴 일 (몬스터 쓰러짐). 다음 `update`가 맨 앞에 돌려준다.
    waiting_events: Vec<GameEvent>,
    /// 부술 수 있는 상자와 상자에서 나온 깜짝 아이템 (D87, US-089). 부서진 상자는 `world.obstacles`에서 빠진다.
    pub boxes: Boxes,
    box_events: Vec<BoxEvent>,
    /// 깜짝 미션 (D81, D103, US-084~086). 화면은 `mission_now`와 `take_mission_events`를 쓴다.
    pub missions: Missions,
    /// 18:00 낮 미션 실패 글을 보여 주는 동안 남은 실제 초 (BUG-034). 0보다 크면 퀴즈를 열기 전에 기다리는 중이다.
    quiz_wait_seconds: f32,
    /// 튜토리얼 말풍선 규칙 (US-119·120). 시작은 `start_tutorial`, 건너뛰기는 `skip_tutorial`, 일어난 일은 `take_tutorial_events`.
    pub tutorial: Tutorial,
    tutorial_events: Vec<TutorialEvent>,
    /// 연습용 몬스터 (US-124). 밤 몬스터 목록(`combat`)과 따로다: 코인·경험치·킬 수·미션에 들지 않는다.
    pub practice: Option<PracticeMonster>,
    practice_events: Vec<PracticeEvent>,
    /// 이번 프레임에 연습용 몬스터를 맞혔다 (튜토리얼 ②를 위해 `update`가 읽고 지운다)
    practice_hit_flag: bool,
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
        let mut missions = Missions::new(seed);
        // 미스가 없는 M1 연습 맵에는 미스 찾기 미션이 나오지 않는다.
        if !world.is_long_map() {
            missions.turned_on.retain(|kind| *kind != MissionKind::FindMiss);
        }
        // 새 게임을 시작하면 1일째 낮 미션이 바로 나온다 (임시-49).
        missions.give_day_mission(1);
        Game {
            missions,
            quiz_wait_seconds: 0.0,
            tutorial: Tutorial::new(),
            tutorial_events: Vec::new(),
            practice: None,
            practice_events: Vec::new(),
            practice_hit_flag: false,
            gathering: Gathering::on_world(&world, &cave_world, seed),
            crafting: Crafting::default(),
            in_cave: false,
            cave_world,
            mine_held: false,
            mine_was_held: false,
            gather_events: Vec::new(),
            shop: Shop::default(),
            shop_events: Vec::new(),
            item_events: Vec::new(),
            waiting_events: Vec::new(),
            player: Player::new(kind, spawn),
            coins: Coins::on_boxes(&world),
            boxes: Boxes::on_world(&world, seed),
            box_events: Vec::new(),
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
        let mut events = std::mem::take(&mut self.waiting_events);
        let crafting = self.crafting.is_open();
        let shopping = self.shop.is_open();
        if self.is_over() || (crafting && CRAFTING_PAUSES_GAME) || (shopping && SHOP_PAUSES_GAME) {
            self.forget_mining();
            return events;
        }
        if self.math.is_open() {
            self.forget_mining();
            self.update_combat(input, dt, &mut events);
            return events;
        }
        if self.is_waiting_to_open_quiz() {
            self.forget_mining();
            self.wait_then_open_night_quiz(dt);
            return events;
        }
        let world_dt = dt * time_scale;
        let stand_still = PlayerInput::default();
        let input = if crafting || shopping { &stand_still } else { input };

        self.player.walk_speed = self.walk_speed();
        let (x_before, was_on_ground, was_in_cave) = (self.player.pos.x, self.player.on_ground, self.in_cave);
        self.player.update(input, dt, if self.in_cave { &self.cave_world } else { &self.world });
        // 튜토리얼이 보는 값: 걸은 거리와 점프 시작 (US-119)
        let walked = if was_in_cave == self.in_cave { shortest_dx(x_before, self.player.pos.x).abs() } else { 0.0 };
        let jumped = input.jump_pressed && was_on_ground && !self.player.on_ground;

        for clock_event in self.clock.update(world_dt) {
            let happened = match clock_event {
                ClockEvent::NightStarted => {
                    // 연습용 몬스터는 밤이 오기 전에 반드시 사라진다 (US-124). 퀴즈가 열려도 남지 않게 여기서 치운다.
                    self.remove_practice_monster();
                    // 18:00: 낮 미션을 먼저 끝내고 잠깐 퀴즈를 연다 (임시-49).
                    // 실패했으면 실패 글을 먼저 보여 주고 퀴즈는 조금 뒤에 연다 (BUG-034).
                    if self.end_day_mission(&mut events) && !self.is_over() {
                        self.quiz_wait_seconds = QUIZ_WAIT_AFTER_DAY_MISSION_FAIL_SECONDS;
                    }
                    GameEvent::NightStarted
                }
                ClockEvent::DayStarted { day_number } => {
                    // 06:00: 밤 미션을 먼저 끝내고 체력을 채운다 (임시-49, D27). 그다음 새 낮 미션.
                    self.end_night_mission_and_start_day(day_number, &mut events);
                    // 스케이트는 산 뒤 처음 오는 06:00에 벗겨진다 (임시-25).
                    if self.shop.take_off_skates() {
                        self.shop_events.push(ShopEvent::SkatesOff);
                    }
                    // 자동차도 탄 뒤 처음 오는 06:00에 내린다 (임시-41).
                    if std::mem::replace(&mut self.gathering.bag.riding_car, false) {
                        self.item_events.push(ItemEvent::CarOff);
                    }
                    // 부서진 상자는 06:00에 돌아오고, 금 간 상자는 새것이 된다 (임시-67).
                    self.boxes.morning_came();
                    GameEvent::DayStarted { day_number }
                }
            };
            events.push(happened);
        }

        // 갑옷 시간은 게임 속 시간이다: 빨리감기 배수만큼 빨리 준다 (임시-26).
        if self.shop.wear_armor(world_dt) {
            self.shop_events.push(ShopEvent::ArmorOff);
        }
        self.combat.health.set_armored(self.shop.armor().is_some());

        // 밤 문제(update_combat)보다 먼저 입구 앞으로 나온다 (임시-37).
        if self.in_cave && !self.cave_is_open() {
            self.leave_cave();
            self.gather_events.push(GatherEvent::PushedOutOfCave);
        }

        let picked = self.coins.update(world_dt, self.body_on_map());
        if picked > 0 {
            self.wallet.add(picked);
            events.push(GameEvent::CoinsPicked { count: picked });
            self.count_for_mission(MissionKind::Coins, picked);
        }

        let monsters_before = self.combat.monsters_given_experience;
        let events_before = events.len();
        self.update_combat(input, dt, &mut events);
        let monsters = self.combat.monsters_given_experience - monsters_before;
        self.count_for_mission(MissionKind::Monsters, monsters);
        let miss_found = events[events_before..].iter().filter(|e| matches!(e, GameEvent::MissFound { .. })).count();
        self.count_for_mission(MissionKind::FindMiss, miss_found as u32);
        self.update_boxes(dt);
        let mined_before = self.count_mined_events();
        self.update_gathering(dt, &mut events);
        let mined = self.count_mined_events() > mined_before;
        self.update_tutorial(dt, walked, jumped, mined, miss_found > 0);
        self.give_night_mission_after_quiz();
        events
    }

    /// 지금 하는 중인 미션 (US-086). 화면 위쪽 진행 글은 `progress_text()`.
    /// 체력 지키기는 `count`가 지금 체력이다. 미션이 없거나, 성공·실패로 끝났거나, 판이 끝났으면 None이다.
    pub fn mission_now(&self) -> Option<Mission> {
        if self.is_over() {
            return None;
        }
        self.missions.going_kind()?;
        let mut mission = *self.missions.current()?;
        if mission.kind == MissionKind::KeepHealth {
            mission.count = self.combat.health.hp();
        }
        Some(mission)
    }

    /// 지난번 뒤로 생긴 미션 일(나옴, 성공, 실패)을 모두 받는다. 화면이 `update` 뒤에 부른다.
    pub fn take_mission_events(&mut self) -> Vec<MissionEvent> {
        self.missions.take_events()
    }

    /// 미션 `kind`를 `amount`만큼 했다 (임시-52). N이 되는 순간 성공하고 코인 `MISSION_REWARD_COINS`를 준다.
    /// 지갑은 `COIN_MAX`에서 멈추지만 점수의 "획득한 동전 수"에는 다 센다 (D103).
    fn count_for_mission(&mut self, kind: MissionKind, amount: u32) {
        if self.is_over() || !self.missions.add(kind, amount) {
            return;
        }
        self.wallet.add(MISSION_REWARD_COINS);
        self.coins.count_mission_reward(MISSION_REWARD_COINS);
    }

    /// 18:00: 하는 중인 낮 미션은 실패로 끝나고 체력이 준다 (D103). 판이 끝났으면 아무 일도 없다.
    /// 낮 미션이 실패했으면 true.
    fn end_day_mission(&mut self, events: &mut Vec<GameEvent>) -> bool {
        if self.is_over() || !self.missions.end_day_mission() {
            return false;
        }
        self.lose_health_for_failed_mission(events);
        true
    }

    /// 18:00 낮 미션 실패 글을 보여 주느라 퀴즈 열기를 기다리는 중이면 true (BUG-034).
    /// 이동안 시계·몬스터·주인공은 퀴즈가 열린 것처럼 멈춰 있다.
    pub fn is_waiting_to_open_quiz(&self) -> bool {
        self.quiz_wait_seconds > 0.0
    }

    /// 기다리는 시간을 실제 `dt`만큼 흘리고, 다 되면 18:00 잠깐 퀴즈를 연다 (BUG-034).
    fn wait_then_open_night_quiz(&mut self, dt: f32) {
        self.quiz_wait_seconds -= dt;
        if self.quiz_wait_seconds > 0.0 {
            return;
        }
        self.quiz_wait_seconds = 0.0;
        if !self.is_over() {
            self.math.add_waiting();
            self.math.open_next_waiting();
        }
    }

    /// 06:00: 밤 미션을 끝낸다. 체력 지키기는 체력이 다시 차기 전의 체력으로 본다.
    /// 실패하면 체력이 준다. 7일째 클리어 아침에는 줄이지 않는다 (니로 임시). 성공하면 코인을 준다.
    /// 그다음 쓰러지지 않았고 클리어 아침이 아니면 새 낮 미션을 낸다.
    fn end_night_mission_and_start_day(&mut self, day_number: u32, events: &mut Vec<GameEvent>) {
        if self.is_over() {
            return;
        }
        let clear_morning = day_number.saturating_sub(1) >= DAYS_TO_CLEAR;
        match self.missions.end_night_mission(self.combat.health.hp()) {
            Some(true) => {
                self.wallet.add(MISSION_REWARD_COINS);
                self.coins.count_mission_reward(MISSION_REWARD_COINS);
            }
            Some(false) if !clear_morning => self.lose_health_for_failed_mission(events),
            _ => {}
        }
        if !self.is_over() && !clear_morning {
            self.missions.give_day_mission(day_number);
        }
    }

    /// 미션 실패: 체력 `MISSION_FAIL_HP_LOSS`. 갑옷이어도 준다 (수학 문제와 같다). 0이면 보통 쓰러짐과 같다.
    fn lose_health_for_failed_mission(&mut self, events: &mut Vec<GameEvent>) {
        let before = self.combat.health.hp();
        self.combat.health.lose_from_quiz(MISSION_FAIL_HP_LOSS);
        let hp_left = self.combat.health.hp();
        events.push(GameEvent::Combat(CombatEvent::PlayerHurt { amount: before - hp_left, hp_left }));
        if self.combat.health.is_down() {
            self.citizens.stop_walkers();
            events.push(GameEvent::Combat(CombatEvent::PlayerDown));
        }
    }

    /// 밤에 잠깐 퀴즈가 닫혀 있으면 그 밤의 밤 미션을 낸다 (임시-49). 같은 밤에 두 번 내지 않는다.
    fn give_night_mission_after_quiz(&mut self) {
        if self.is_over() || !self.clock.is_night() || self.math.is_open() || self.is_waiting_to_open_quiz() {
            return;
        }
        self.missions.give_night_mission(self.clock.day_number());
    }

    /// 공격 버튼을 처리하고 몬스터·미스·무기·시민 시간을 `dt`만큼 흘린다. 시계가 움직인 뒤에 불러야
    /// 이번 프레임의 낮/밤을 본다. 빨리감기여도 `dt`는 진짜 시간이다 (D13: 빨라지는 건 시계·코인뿐).
    /// 빨리감기도 이 함수를 지나서, 클리어와 수학 문제 열기도 여기서 본다.
    pub fn update_combat(&mut self, input: &PlayerInput, dt: f32, events: &mut Vec<GameEvent>) {
        if self.cleared {
            return;
        }
        // 날이 지날수록 몬스터가 많아진다 (상자 몬스터도 같은 상한).
        self.combat.day_number = self.clock.day_number();
        if self.math.is_open() {
            self.wait_after_math_answer(dt, events);
            return;
        }
        // 밤 문제는 첫 몬스터보다 먼저 연다 (BUG-025): 18:00 프레임에는 몬스터를 움직이지 않는다.
        // 낮 미션 실패로 쓰러졌으면 문제를 열지 않는다.
        if events.contains(&GameEvent::NightStarted) && !self.is_over() {
            // 낮 미션 실패 글을 먼저 보여 주는 중이면 문제는 나중에 열고, 몬스터도 움직이지 않는다 (BUG-034).
            if self.is_waiting_to_open_quiz() {
                return;
            }
            self.math.add_waiting();
            self.math.open_next_waiting();
            if self.math.is_open() {
                return;
            }
        }
        let player_rect = self.body_on_map();
        if input.attack_pressed {
            let reach = self.attack_reach();
            let hits = self.combat.attack_with(player_rect, self.player.facing, self.weapons.held_attack(), reach);
            let hit_practice = self.hit_practice_monster(player_rect, reach);
            // 닿는 몬스터가 있으면 몬스터만 맞고 상자는 그대로다 (임시-64).
            let hit_a_monster = hit_practice || hits.iter().any(|hit| {
                matches!(
                    hit,
                    CombatEvent::ZombieHit { .. }
                        | CombatEvent::ZombieDefeated { .. }
                        | CombatEvent::SkeletonHit { .. }
                        | CombatEvent::SkeletonDefeated { .. }
                )
            });
            events.extend(hits.into_iter().map(GameEvent::Combat));
            if !hit_a_monster {
                self.attack_box(player_rect, events);
            }
            let facing = self.player.facing;
            let knocked_down = self.citizens.knock_down_reached(|body| is_reached(player_rect, facing, reach, body));
            events.extend(knocked_down.into_iter().map(|at| GameEvent::Combat(CombatEvent::CitizenDefeated { at })));
        }
        // 동굴 안에 있는 동안 몬스터(상자 몬스터도)는 모두 멈춘다 (부모님 결정 2026-11-14, REVIEW-021 2번).
        if !self.in_cave {
            let happened = self.combat.update(dt, self.clock.is_night(), player_rect, &self.world);
            events.extend(happened.into_iter().map(GameEvent::Combat));
        }
        self.update_practice_monster(dt, player_rect);
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
        // 수학과 영어가 줄이는 체력이 다를 수 있어서 닫기 전에 본다.
        let hp_loss = self.math.wrong_hp_loss();
        if self.math.wait_after_answer(dt) != Some(MathResult::Wrong) {
            return;
        }
        let before = self.combat.health.hp();
        self.combat.health.lose_from_quiz(hp_loss);
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
            // 총 범위 상한(480)은 몸 가운데에서 잰 화면 끝이라, 몸 앞 끝에서는 몸 폭의 절반만큼 짧다 (US-129).
            let reach = self.weapons.held_range().min(GUN_REACH_MAX_FROM_FRONT);
            self.here().clear_reach(self.player.rect(), self.player.facing, reach)
        } else {
            self.weapons.held_range()
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
    /// 순서: 맨손 → 칼 → 도끼 → 총 → 맨손. 없는 무기는 건너뛴다.
    /// 포션·주사기총·엔더열매는 손에 들지 않고 따로 쓴다 (부모님 결정 2026-11-14).
    pub fn switch_weapon(&mut self) {
        let choices: Vec<Weapon> = Weapon::ORDER.into_iter().filter(|w| self.weapons.has(*w)).collect();
        let now = choices.iter().position(|w| *w == self.weapons.held()).unwrap_or(0);
        self.weapons.hold(choices[(now + 1) % choices.len()]);
    }

    /// 가진 무기가 있으면 true. 폰 바꾸기 버튼은 이때만 보인다.
    pub fn can_switch(&self) -> bool {
        !self.weapons.owned().is_empty()
    }

    /// 지금 걷는 빠르기 (초당 픽셀). 자동차를 타면 자동차 빠르기만 쓰고 스케이트와 겹쳐 곱하지 않는다 (임시-41).
    pub fn walk_speed(&self) -> f32 {
        if self.gathering.bag.riding_car {
            PLAYER_WALK_SPEED * CAR_SPEED_MULTIPLIER
        } else {
            PLAYER_WALK_SPEED * self.shop.walk_speed_multiplier()
        }
    }

    /// 주사기총 쏘기 (US-083, 임시-46). 포션처럼 손에 들지 않고 따로 쓴다 (부모님 결정 2026-11-14).
    /// 화면 안 몬스터가 모두 쓰러지고 주사기총이 하나 줄어든다: `ItemEvent::SyringeGunFired`, true.
    /// 쓰러진 몬스터마다의 `GameEvent::Combat`은 다음 `update`가 돌려준다. 경험치는 없다.
    /// 화면에 몬스터가 없거나(동굴 안 포함), 주사기총이 없거나, 상점·제작대·문제 창이 열렸거나, 판이 끝났으면 false이고 그대로다.
    pub fn fire_syringe_gun(&mut self) -> bool {
        if self.gathering.bag.syringe_guns == 0 || !self.can_use_potion_now() {
            return false;
        }
        let mut swept = Vec::new();
        let monsters = self.sweep_monsters_on_screen(&mut swept);
        if monsters == 0 {
            return false;
        }
        self.waiting_events.extend(swept);
        self.gathering.bag.syringe_guns -= 1;
        self.combat.add_sweep_score(SYRINGE_GUN_SCORE);
        self.item_events.push(ItemEvent::SyringeGunFired { at: self.player.rect().center(), monsters });
        true
    }

    /// 엔더열매 먹기 (US-081). 포션처럼 손에 들지 않고 따로 쓴다 (부모님 결정 2026-11-14).
    /// 힘이 생기고 하나 준다: `ItemEvent::EnderFruitEaten`, true.
    /// 힘이 벌써 있으면 `ItemEvent::EnderFruitAlreadyEaten`이고 개수 그대로 false.
    /// 엔더열매가 없거나, 상점·제작대·문제 창이 열렸거나, 판이 끝났으면 아무 일 없이 false.
    pub fn eat_ender_fruit(&mut self) -> bool {
        if self.gathering.bag.ender_fruits == 0 || !self.can_use_potion_now() {
            return false;
        }
        let bag = &mut self.gathering.bag;
        if bag.has_ender_power {
            self.item_events.push(ItemEvent::EnderFruitAlreadyEaten);
            return false;
        }
        bag.ender_fruits -= 1;
        bag.has_ender_power = true;
        self.item_events.push(ItemEvent::EnderFruitEaten { at: self.player.rect().center() });
        true
    }

    /// 지금 화면 안 몬스터를 모두 쓰러뜨리고 그 수를 돌려준다 (임시-43). 동굴 안이면 0이다.
    fn sweep_monsters_on_screen(&mut self, events: &mut Vec<GameEvent>) -> u32 {
        if self.in_cave {
            return 0;
        }
        let swept = self.combat.sweep_monsters_on_screen(self.camera_left());
        let monsters = swept.len() as u32;
        events.extend(swept.into_iter().map(GameEvent::Combat));
        monsters
    }

    /// 여기서 쿵을 할 수 있으면 그 힘 (임시-42): 긴 맵 해변 모래밭(몸 가운데 x가 `SLAM_BEACH_LEFT`~`SLAM_BEACH_RIGHT`)이고,
    /// 엔더열매 힘이나 다이아래더라이트 곡괭이가 있고, E 키로 쓸 다른 것(`usable_in_reach`)이 닿지 않을 때.
    /// 둘 다 있으면 엔더열매가 먼저다. 폰 쿵 버튼(`SLAM_BUTTON_TEXT`)과 머리 위 E는 Some일 때만 보인다.
    /// 쿵은 E 키나 캐기 버튼(`mine_held`)을 새로 누르면 일어난다.
    pub fn slam_power_here(&self) -> Option<SlamPower> {
        if self.is_over() || self.math.is_open() || self.crafting.is_open() || self.shop.is_open() || self.in_cave {
            return None;
        }
        if !self.world.is_long_map() || self.usable_in_reach().is_some() {
            return None;
        }
        let x = self.player.rect().center().x.rem_euclid(self.world.width);
        if !(SLAM_BEACH_LEFT..=SLAM_BEACH_RIGHT).contains(&x) {
            return None;
        }
        let bag = &self.gathering.bag;
        if bag.has_ender_power {
            Some(SlamPower::EnderFruit)
        } else if bag.has_diamond_pickaxe {
            Some(SlamPower::DiamondPickaxe)
        } else {
            None
        }
    }

    /// 바다에 쿵 (US-081, US-082): 화면 안 몬스터가 모두 쓰러지고 다이아몬드를 얻는다. 몬스터가 없어도 쓰인다.
    /// 엔더열매 힘이나 다이아래더라이트 곡괭이가 없어진다. 점수는 몬스터가 1마리 이상일 때만 (임시-47).
    fn slam(&mut self, power: SlamPower, events: &mut Vec<GameEvent>) {
        let monsters = self.sweep_monsters_on_screen(events);
        let bag = &mut self.gathering.bag;
        match power {
            SlamPower::EnderFruit => bag.has_ender_power = false,
            SlamPower::DiamondPickaxe => bag.has_diamond_pickaxe = false,
        }
        for _ in 0..power.diamonds() {
            bag.add(Material::Diamond);
        }
        if monsters > 0 {
            self.combat.add_sweep_score(power.score());
        }
        let at = self.player.rect().center();
        self.item_events.push(ItemEvent::Slammed { power, at, monsters, diamonds: power.diamonds() });
    }

    /// 쌓인 무기 강화 소식을 꺼내 간다 (`WeaponEvent::Upgraded`, D115). 화면이 "+n"과 반짝이를 그린다.
    pub fn take_weapon_events(&mut self) -> Vec<WeaponEvent> {
        self.weapons.take_events()
    }

    /// 지난번 뒤로 생긴 자동차·주사기총·엔더열매·쿵 일을 모두 받는다. 화면이 `update` 뒤에 부른다.
    pub fn take_item_events(&mut self) -> Vec<ItemEvent> {
        std::mem::take(&mut self.item_events)
    }

    /// 지금 포션·주사기총·엔더열매를 쓸 수 있는 때인지: 판이 끝났거나, 상점·제작대·수학 문제 창이 열려 있으면 false.
    fn can_use_potion_now(&self) -> bool {
        !self.is_over() && !self.shop.is_open() && !self.crafting.is_open() && !self.math.is_open()
    }

    /// 포션 키(R·T)나 폰 포션 버튼을 눌렀을 때 부른다 (US-071, 부모님 결정: 포션은 다른 키로 먹는다).
    /// 체력 포션: 체력이 가득이면 안 쓰인다. 독약 포션: 200픽셀 안에 몬스터가 없으면 안 쓰인다.
    /// 쓰였으면 하나 줄고 `ShopEvent`가 생기고 true. 안 쓰였으면 개수 그대로 false.
    /// 손에 든 무기는 바뀌지 않는다.
    pub fn use_potion(&mut self, potion: Potion) -> bool {
        if self.shop.potions(potion) == 0 || !self.can_use_potion_now() {
            return false;
        }
        let at = self.player.rect().center();
        let used = match potion {
            Potion::Health => {
                let healed = self.combat.health.heal(HEALTH_POTION_HEAL);
                (healed > 0).then_some(ShopEvent::HealthPotionUsed { at, healed })
            }
            Potion::Poison => {
                let monsters = if self.in_cave { 0 } else { self.combat.spray_poison(at) };
                (monsters > 0).then_some(ShopEvent::PoisonSprayed { at, monsters })
            }
        };
        let Some(event) = used else {
            return false;
        };
        self.shop_events.push(event);
        self.shop.take_potion(potion);
        true
    }

    /// 쓰는 물건의 가진 개수. 0이면 표시판에 안 나온다.
    pub fn pocket_count(&self, pocket: Pocket) -> u32 {
        let bag = &self.gathering.bag;
        match pocket {
            Pocket::Potion(potion) => self.shop.potions(potion),
            Pocket::SyringeGun => bag.syringe_guns,
            Pocket::EnderFruit => bag.ender_fruits,
        }
    }

    /// 쓰는 물건을 지금 쓸 수 없는 이유 (부모님 결정 2026-11-15). 쓸 수 있으면 None이다.
    /// 상점·제작대·문제 창이 열렸거나 판이 끝났을 때도 None이다: 그때는 안내 글 없이 아무 일도 없다.
    /// `use_potion`·`fire_syringe_gun`·`eat_ender_fruit`과 같은 규칙이다.
    pub fn why_cant_use(&self, pocket: Pocket) -> Option<CantUse> {
        if !self.can_use_potion_now() {
            return None;
        }
        if self.pocket_count(pocket) == 0 {
            return Some(CantUse::NoneLeft);
        }
        let at = self.player.rect().center();
        match pocket {
            Pocket::Potion(Potion::Health) if self.combat.health.hp() >= self.combat.health.max_hp() => Some(CantUse::HealthFull),
            Pocket::Potion(Potion::Poison) if self.in_cave || self.combat.count_monsters_in_poison_range(at) == 0 => {
                Some(CantUse::NoMonsterNear)
            }
            Pocket::SyringeGun if self.in_cave || self.combat.count_monsters_on_screen(self.camera_left()) == 0 => {
                Some(CantUse::NoMonsterOnScreen)
            }
            Pocket::EnderFruit if self.gathering.bag.has_ender_power => Some(CantUse::AlreadyEaten),
            _ => None,
        }
    }

    /// 상점 물건을 살 수 없는 이유. 살 수 있으면 None이다.
    /// 신고 있는 스케이트는 다시 사지 않는다. 갑옷과 포션은 언제나 또 살 수 있다.
    pub fn why_cant_buy(&self, item: ShopItem) -> Option<CantBuy> {
        let already_have = item == ShopItem::Skates && self.shop.has_skates();
        if already_have {
            Some(CantBuy::AlreadyHave)
        } else if self.wallet.coins() < item.price() {
            Some(CantBuy::NotEnoughCoins)
        } else {
            None
        }
    }

    /// 상점 화면의 사기 줄들 (`ShopItem::ALL` 순서).
    pub fn buy_rows(&self) -> Vec<BuyRow> {
        ShopItem::ALL
            .into_iter()
            .map(|item| {
                let reason = self.why_cant_buy(item);
                BuyRow { item, name: item.name(), price: item.price(), can_buy: reason.is_none(), reason }
            })
            .collect()
    }

    /// 물건을 산다. 못 사면 코인도 물건도 그대로다.
    /// 포션은 가방에 넣고, 스케이트와 갑옷은 바로 입는다. 무기는 팔지 않는다 (부모님 결정 2026-11-14).
    /// 산 물건은 수학 문제 "아이템"으로 세지 않는다 (임시). 점수도 줄지 않는다 (US-069).
    pub fn buy(&mut self, item: ShopItem) -> Result<(), CantBuy> {
        if let Some(reason) = self.why_cant_buy(item) {
            return Err(reason);
        }
        self.wallet.spend(item.price());
        match item {
            ShopItem::HealthPotion => self.shop.add_potion(Potion::Health),
            ShopItem::PoisonPotion => self.shop.add_potion(Potion::Poison),
            ShopItem::Skates => self.shop.put_on_skates_when(self.clock.is_night()),
            ShopItem::RainbowArmor => self.shop.put_on_armor(Armor::Rainbow),
            ShopItem::DiamondArmor => self.shop.put_on_armor(Armor::Diamond),
        }
        self.combat.health.set_armored(self.shop.armor().is_some());
        Ok(())
    }

    /// 광물을 팔 수 없는 이유. 팔 수 있으면 None이다. 철광석·나무는 팔지 않아서 `NoneToSell`이다.
    /// 코인이 `COIN_MAX`를 넘게 되어도 판다. 지갑이 `COIN_MAX`에서 멈춘다 (부모님 결정).
    pub fn why_cant_sell(&self, material: Material) -> Option<CantSell> {
        if sell_price(material).is_none() || self.gathering.bag.count(material) == 0 {
            return Some(CantSell::NoneToSell);
        }
        None
    }

    /// 상점 화면의 팔기 줄들: 금, 은, 동 (임시-38).
    pub fn sell_rows(&self) -> Vec<SellRow> {
        SELLABLE
            .into_iter()
            .map(|material| {
                let reason = self.why_cant_sell(material);
                let price = sell_price(material).unwrap_or(0);
                let count = self.gathering.bag.count(material);
                SellRow { material, name: material.name(), price, count, can_sell: reason.is_none(), reason }
            })
            .collect()
    }

    /// 광물 하나를 판다. 돌려주는 값: 지갑에 실제로 들어간 코인.
    /// 판 코인은 점수의 "주운 코인"(`coins.picked_total`)에 넣지 않는다 (니로 해석, US-069).
    pub fn sell(&mut self, material: Material) -> Result<u32, CantSell> {
        if let Some(reason) = self.why_cant_sell(material) {
            return Err(reason);
        }
        self.gathering.bag.take_one(material);
        Ok(self.wallet.add(sell_price(material).unwrap_or(0)))
    }

    /// 상점 화면을 닫는다 (Esc, 나가기 버튼). 문 앞에 그대로 서 있고, 멈춘 때부터 이어진다.
    pub fn close_shop(&mut self) {
        self.shop.close();
    }

    /// 지난번 뒤로 생긴 상점·포션·스케이트·갑옷 일을 모두 받는다. 화면이 `update` 뒤에 부른다.
    pub fn take_shop_events(&mut self) -> Vec<ShopEvent> {
        std::mem::take(&mut self.shop_events)
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
    /// 집 문만은 바로 앞에 섰을 때다 (`is_in_front_of_door`, 부모님 결정 2026-11-15).
    /// 거리가 같으면 바라보는 쪽을 먼저 쓴다.
    /// 판이 끝났거나 수학 문제·제작대가 열려 있으면 None이다. 폰 버튼은 Some일 때만 보인다 (글은 `button_text`).
    pub fn usable_in_reach(&self) -> Option<Usable> {
        if self.is_over() || self.math.is_open() || self.crafting.is_open() || self.shop.is_open() {
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
            spots.extend(self.world.house_door().map(|rect| (Usable::HouseDoor, rect)));
            spots.extend(ready(&self.gathering.trees).into_iter().map(|(i, rect)| (Usable::Tree(i), rect)));
        }
        let body = self.player.rect();
        let dx = |rect: &Rect| shortest_dx(body.center().x, rect.center().x);
        let is_behind = |rect: &Rect| match self.player.facing {
            Facing::Right => dx(rect) < 0.0,
            Facing::Left => dx(rect) > 0.0,
        };
        let is_reached = |thing: &Usable, rect: &Rect| match thing {
            Usable::HouseDoor => is_in_front_of_door(body, *rect),
            _ => is_reached_on_either_side(body, MINE_REACH, *rect),
        };
        spots
            .into_iter()
            .filter(|(thing, rect)| is_reached(thing, rect))
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
            // 낮에도 밤에도 들어간다 (임시-24).
            Usable::HouseDoor => {
                self.shop.open();
                self.shop_events.push(ShopEvent::Opened);
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
    fn update_gathering(&mut self, dt: f32, happened: &mut Vec<GameEvent>) {
        self.gathering.grow_back(dt);
        let newly_pressed = self.mine_held && !self.mine_was_held;
        self.mine_was_held = self.mine_held;

        let got_hurt = happened.iter().any(|e| matches!(e, GameEvent::Combat(CombatEvent::PlayerHurt { .. })));
        if !self.mine_held || got_hurt || self.is_over() || self.math.is_open() {
            self.gathering.stop_mining();
            return;
        }
        // 해변에서 E 키·쿵 버튼을 새로 누르면 쿵 (임시-42). 다른 쓸 것이 닿으면 그것이 먼저라 쿵이 없다.
        if let Some(power) = self.slam_power_here().filter(|_| newly_pressed) {
            self.slam(power, happened);
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
            // 다 캔 것만 센다. 어떤 광물이 나와도 1이다 (임시-52).
            let kind = if self.in_cave { MissionKind::Ores } else { MissionKind::Trees };
            self.count_for_mission(kind, 1);
        }
    }

    /// 지난번 뒤로 생긴 상자 일(금 감, 부서짐, 깜짝 아이템, 와우!, 공격!, 도끼 안내)을 모두 받는다.
    /// 화면이 `update` 뒤에 부른다. 상자에서 나온 몬스터의 `ZombieAppeared`·`SkeletonAppeared`는 `GameEvent::Combat`으로도 온다.
    pub fn take_box_events(&mut self) -> Vec<BoxEvent> {
        std::mem::take(&mut self.box_events)
    }

    /// 공격 버튼으로 상자를 친다 (임시-63, 임시-64). 몬스터가 맞지 않았을 때만 부른다.
    /// 도끼면 바라보는 쪽 가장 가까운 상자 하나가 맞는다. 다른 것이면 상자는 그대로이고 처음 한 번만 도끼 안내가 나온다.
    fn attack_box(&mut self, body: Rect, events: &mut Vec<GameEvent>) {
        if self.in_cave {
            return;
        }
        let facing = self.player.facing;
        let held = self.weapons.held();
        let Some(index) = self.boxes.box_in_reach(body, facing, Weapon::Axe.range()) else {
            return;
        };
        if held != Weapon::Axe {
            if self.boxes.take_need_axe_hint() {
                self.box_events.push(BoxEvent::NeedAxe { at: self.boxes.list[index].rect.center() });
            }
            return;
        }
        let at = self.boxes.list[index].rect.center();
        if self.boxes.hit(index) {
            self.break_box(index, events);
        } else {
            self.box_events.push(BoxEvent::Cracked { index, at, cracks: self.boxes.list[index].cracks() });
        }
    }

    /// 상자 `index`가 부서졌다: 맵에서 빠지고 (길·총·몬스터 나오는 자리에서 없는 것, D48·D66·D62),
    /// 위에 있던 코인·무기·아이템은 땅으로 떨어진다 (임시-66). 그다음 깜짝 아이템이나 몬스터가 나올 수 있다 (둘이 함께는 없다).
    /// 점수와 수학 문제 아이템 수는 늘지 않는다.
    pub(crate) fn break_box(&mut self, index: usize, events: &mut Vec<GameEvent>) {
        let rect = self.boxes.list[index].rect;
        self.boxes.mark_broken(index);
        self.world.obstacles.retain(|obstacle| *obstacle != rect);
        if let Some(coin) = self.coins.list.get_mut(index) {
            coin.fall_to_ground(self.world.ground_y);
        }
        self.land_things_again();
        self.box_events.push(BoxEvent::Broken { index, at: rect.center() });

        match self.boxes.dice.roll_surprise() {
            Surprise::Item => {
                let have: Vec<SurpriseItem> = SurpriseItem::ALL.into_iter().filter(|item| self.has_the_only_one(*item)).collect();
                let item = self.boxes.dice.roll_item(&have);
                let at = self.boxes.drop_item(item, rect, &self.world);
                self.box_events.push(BoxEvent::ItemDropped { item, at });
                if item.is_very_strong() {
                    self.box_events.push(BoxEvent::Wow { at: self.player.rect().center() });
                }
            }
            Surprise::Monster => {
                let monster = self.boxes.dice.roll_monster();
                if let Some(appeared) = self.combat.spawn_from_box(rect.center().x, self.world.ground_y, monster) {
                    let at = match appeared {
                        CombatEvent::ZombieAppeared { at } | CombatEvent::SkeletonAppeared { at, .. } => at,
                        _ => rect.center(),
                    };
                    let monster = if matches!(appeared, CombatEvent::SkeletonAppeared { .. }) {
                        BoxMonster::Skeleton
                    } else {
                        BoxMonster::Zombie
                    };
                    events.push(GameEvent::Combat(appeared));
                    self.box_events.push(BoxEvent::MonsterJumpedOut { monster, at });
                }
            }
            Surprise::Nothing => {}
        }
    }

    /// 상자가 빠지거나 돌아온 뒤: 땅의 무기와 깜짝 아이템이 멈출 곳을 다시 정한다. 공중에 뜨거나 상자에 묻히지 않는다.
    fn land_things_again(&mut self) {
        for weapon in &mut self.weapons.dropped {
            weapon.land_again(&self.world);
        }
        for dropped in &mut self.boxes.items {
            dropped.land_again(&self.world);
        }
    }

    /// 이미 가진 "하나뿐인 것"이면 true (임시-65): 칼, 도끼, 총, 신고 있는 스케이트, 타고 있는 자동차,
    /// 다이아래더라이트 곡괭이, 지금 입고 있는 갑옷. 포션·엔더열매·주사기총은 가지고 있어도 또 나온다.
    /// 갑옷은 입고 있는 것만 빠진다 (무지개를 입으면 다이아 갑옷은 나온다).
    fn has_the_only_one(&self, item: SurpriseItem) -> bool {
        let bag = &self.gathering.bag;
        match item {
            SurpriseItem::Sword => self.weapons.has(Weapon::Sword),
            SurpriseItem::Axe => self.weapons.has(Weapon::Axe),
            SurpriseItem::Gun => self.weapons.has(Weapon::Gun),
            SurpriseItem::Skates => self.shop.has_skates(),
            SurpriseItem::Car => bag.riding_car,
            SurpriseItem::DiamondPickaxe => bag.has_diamond_pickaxe,
            SurpriseItem::RainbowArmor => self.shop.armor() == Some(Armor::Rainbow),
            SurpriseItem::DiamondArmor => self.shop.armor() == Some(Armor::Diamond),
            SurpriseItem::HealthPotion | SurpriseItem::PoisonPotion | SurpriseItem::SyringeGun | SurpriseItem::EnderFruit => false,
        }
    }

    /// 주운 깜짝 아이템을 상점에서 산 것·만든 것과 똑같이 넣는다. 무기는 손에 든다 (미스 무기처럼).
    /// 스케이트·갑옷은 바로 신고 입고, 자동차는 바로 탄다. 이미 가진 하나뿐인 것이면 그대로다.
    fn get_surprise_item(&mut self, item: SurpriseItem) {
        let bag = &mut self.gathering.bag;
        match item {
            SurpriseItem::Sword | SurpriseItem::Axe | SurpriseItem::Gun => {
                let weapon = match item {
                    SurpriseItem::Sword => Weapon::Sword,
                    SurpriseItem::Axe => Weapon::Axe,
                    _ => Weapon::Gun,
                };
                if !self.weapons.has(weapon) {
                    self.weapons.add_bought(weapon);
                }
            }
            SurpriseItem::HealthPotion => self.shop.add_potion(Potion::Health),
            SurpriseItem::PoisonPotion => self.shop.add_potion(Potion::Poison),
            SurpriseItem::Skates => self.shop.put_on_skates_when(self.clock.is_night()),
            SurpriseItem::RainbowArmor => self.shop.put_on_armor(Armor::Rainbow),
            SurpriseItem::DiamondArmor => self.shop.put_on_armor(Armor::Diamond),
            SurpriseItem::SyringeGun => bag.syringe_guns += 1,
            SurpriseItem::EnderFruit => bag.ender_fruits += 1,
            SurpriseItem::Car => bag.riding_car = true,
            SurpriseItem::DiamondPickaxe => bag.has_diamond_pickaxe = true,
        }
        self.combat.health.set_armored(self.shop.armor().is_some());
    }

    /// 상자 시간을 흘린다: 06:00 뒤 자리가 빈 상자는 돌아오고 (임시-67), 깜짝 아이템은 떨어지고 주워진다.
    /// 판이 끝났으면 멈춘다.
    fn update_boxes(&mut self, dt: f32) {
        if self.is_over() {
            return;
        }
        for index in self.boxes.waiting_for_space() {
            let rect = self.boxes.list[index].rect;
            let someone_there = really_overlaps(rect, self.body_on_map())
                || self.combat.any_monster_touches(rect)
                || self.citizens.list.iter().any(|citizen| really_overlaps(citizen.rect, rect))
                || self.citizens.walkers.iter().any(|walker| really_overlaps(walker.rect, rect));
            if someone_there {
                continue;
            }
            self.boxes.stand_again(index);
            self.world.obstacles.push(rect);
            if let Some(coin) = self.coins.list.get_mut(index) {
                coin.go_back_on_box();
            }
            self.land_things_again();
            self.box_events.push(BoxEvent::CameBack { index });
        }
        for item in self.boxes.update_items(dt, self.body_on_map()) {
            self.get_surprise_item(item);
            self.box_events.push(BoxEvent::ItemPicked { item });
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

/// 튜토리얼 말풍선(US-119·120)과 연습용 몬스터(US-124). 화면은 아래 함수만 부르고 읽는다.
impl Game {
    /// 튜토리얼을 시작한다. 이미 본 사람(`settings.tutorial_done`)이나 안내 없음(`?qa`, `no_guide`)이면 시작하지 않고 false.
    /// 새 판 맨 처음 한 번 부른다. 시작하면 말풍선 ①이 나온다.
    pub fn start_tutorial(&mut self, settings: &Settings, no_guide: bool) -> bool {
        let event = self.tutorial.start(settings, no_guide);
        self.tutorial_events.extend(event);
        event.is_some()
    }

    /// 건너뛰기. 말풍선이 모두 끝나고 연습용 몬스터도 사라진다. 진행 중이 아니면 false.
    pub fn skip_tutorial(&mut self) -> bool {
        let Some(event) = self.tutorial.skip() else {
            return false;
        };
        self.tutorial_events.push(event);
        self.practice = None;
        true
    }

    /// 지난번 뒤로 생긴 튜토리얼 일을 모두 받는다. `Finished`·`Skipped`가 오면
    /// `self.tutorial.settings_after(&settings)`를 저장한다.
    pub fn take_tutorial_events(&mut self) -> Vec<TutorialEvent> {
        std::mem::take(&mut self.tutorial_events)
    }

    /// 지난번 뒤로 생긴 연습용 몬스터 일을 모두 받는다.
    pub fn take_practice_events(&mut self) -> Vec<PracticeEvent> {
        std::mem::take(&mut self.practice_events)
    }

    /// 1일째 낮이고 동굴 밖이면 연습용 몬스터를 내보내도 된다.
    fn practice_allowed_now(&self) -> bool {
        self.clock.day_number() == 1 && !self.clock.is_night() && !self.in_cave && !self.is_over()
    }

    /// 연습용 몬스터를 플레이어 앞에 내보낸다. 이미 있거나 낮이 아니거나 자리가 없으면 false.
    /// 튜토리얼 ②가 시작될 때 `update`가 알아서 부른다 (화면이 직접 부를 일은 없다).
    pub fn spawn_practice_monster(&mut self) -> bool {
        if self.practice.is_some() || !self.practice_allowed_now() {
            return false;
        }
        let Some(x) = practice_spawn_x(self.player.rect(), self.player.facing, &self.world) else {
            return false;
        };
        let monster = PracticeMonster::new(x, self.world.ground_y);
        self.practice_events.push(PracticeEvent::Appeared { at: monster.rect().center() });
        self.practice = Some(monster);
        true
    }

    /// 공격이 연습용 몬스터에 닿았으면 한 번 맞힌다 (무기 힘과 상관없이 1번). 맞혔으면 true.
    fn hit_practice_monster(&mut self, player_rect: Rect, reach: f32) -> bool {
        if self.combat.health.is_down() {
            return false;
        }
        let facing = self.player.facing;
        let Some(monster) = self.practice.as_mut().filter(|m| is_reached(player_rect, facing, reach, m.rect())) else {
            return false;
        };
        let at = monster.rect().center();
        if monster.take_hit() {
            self.practice = None;
            self.practice_events.push(PracticeEvent::Defeated { at });
        } else {
            self.practice_events.push(PracticeEvent::Hit { at, hp_left: monster.hp });
        }
        self.practice_hit_flag = true;
        true
    }

    /// 연습용 몬스터를 치운다 (있었으면 `Vanished`가 온다).
    fn remove_practice_monster(&mut self) {
        if let Some(monster) = self.practice.take() {
            self.practice_events.push(PracticeEvent::Vanished { at: monster.rect().center() });
        }
    }

    /// 연습용 몬스터 시간을 흘린다:천천히 따라오고, 60초가 지나거나 밤이 되면 저절로 사라진다. 플레이어는 다치지 않는다.
    fn update_practice_monster(&mut self, dt: f32, player_rect: Rect) {
        let Some(monster) = self.practice.as_mut() else {
            return;
        };
        let mut gone = self.clock.is_night();
        if !gone && !self.in_cave && !self.combat.health.is_down() {
            let mut time_left = dt.max(0.0);
            // 아주 긴 프레임에도 횟수가 정해져 있다.
            for _ in 0..PRACTICE_STEP_LIMIT {
                if time_left <= 0.0 || gone {
                    break;
                }
                let step = time_left.min(PHYSICS_STEP_SECONDS);
                gone = monster.step(step, player_rect.center().x, &self.world);
                time_left -= step;
            }
        }
        if gone {
            self.practice_events.push(PracticeEvent::Vanished { at: monster.rect().center() });
            self.practice = None;
        }
    }

    fn count_mined_events(&self) -> usize {
        self.gather_events.iter().filter(|e| matches!(e, GatherEvent::Mined { .. })).count()
    }

    /// 이번 프레임에 일어난 일을 튜토리얼에 알린다. 말풍선 ②가 시작되면 연습용 몬스터를 내보낸다.
    fn update_tutorial(&mut self, dt: f32, walked: f32, jumped: bool, mined: bool, found_miss: bool) {
        let hit_practice = std::mem::take(&mut self.practice_hit_flag);
        let facts = TutorialFacts {
            walked,
            jumped,
            hit_practice,
            mined,
            found_miss,
            practice_present: self.practice.is_some(),
            practice_allowed: self.practice_allowed_now(),
        };
        for event in self.tutorial.update(dt, &facts) {
            if event == TutorialEvent::StepStarted(super::tutorial::TutorialStep::Attack) {
                self.spawn_practice_monster();
            }
            self.tutorial_events.push(event);
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
        // 1일째 낮 미션을 안 했으니 실패 글을 먼저 보여 주고 (BUG-034), 그 뒤 수학 문제가 열린다 (부모님 답).
        assert!(g.is_waiting_to_open_quiz());
        g.update(&no_input(), crate::config::QUIZ_WAIT_AFTER_DAY_MISSION_FAIL_SECONDS);
        // 맞히고 닫은 뒤에 밤을 보낸다.
        g.math.answer_right(); // 수학이든 영어든 (US-087)
        g.update(&no_input(), crate::config::MATH_RIGHT_SHOW_SECONDS);
        assert!(!g.math.is_open());
        let events = g.update(&no_input(), NIGHT_SECONDS);
        assert!(events.contains(&GameEvent::DayStarted { day_number: 2 }));
        assert_eq!(g.clock.nights_survived(), 1);
    }
}
