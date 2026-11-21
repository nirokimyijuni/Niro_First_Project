//! 소리 규칙 (US-112~117). 소리를 내지는 않는다. "어떤 소리를, 언제, 얼마나 크게"만 정한다.
//! 소리 파일 읽기와 재생은 로건(view)이 한다. 여기는 macroquad·브라우저를 모른다.
//!
//! 로건이 쓰는 순서:
//! 1. `SoundSettings::from_settings(&settings)`로 소리 설정을 만든다 (M 키는 `toggle`).
//! 2. 게임 이벤트마다 `sfx_for_*` 함수로 `Sfx`를 얻어 `SfxMixer::request`에 넣는다.
//! 3. 프레임 끝에 `SfxMixer::take_playable`이 돌려준 것만 재생한다 (`Sfx::file_name`).
//! 4. 음악은 `choose_track` → `MusicPlan::update`가 `Some`을 줄 때만 곡을 바꾼다.
//! 5. 파일은 `loading_order()` 순서로 받는다. 못 받은 소리는 그냥 안 낸다.

use super::boxes::BoxEvent;
use super::clock::Phase;
use super::combat::CombatEvent;
use super::game::{Game, GameEvent, GatherEvent, ItemEvent};
use super::gathering::Material;
use super::math::MathResult;
use super::mission::MissionEvent;
use super::practice::PracticeEvent;
use super::settings::Settings;
use super::shop::{CantBuy, ShopEvent};
use super::skeleton::ProjectileKind;
use super::tutorial::TutorialEvent;
use super::weapon::Weapon;
use crate::config::*;

// ---------------------------------------------------------------- 소리 설정

/// 소리 켜기/끄기와 음악·효과음 크기. 저장하는 값은 `Settings`와 왕복한다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SoundSettings {
    on: bool,
    music_percent: u8,
    effects_percent: u8,
}

impl SoundSettings {
    pub fn from_settings(s: &Settings) -> SoundSettings {
        SoundSettings {
            on: s.sound_on,
            music_percent: s.music_percent.min(SETTINGS_VOLUME_MAX_PERCENT),
            effects_percent: s.effects_percent.min(SETTINGS_VOLUME_MAX_PERCENT),
        }
    }

    /// `base`에서 소리 값만 바꾼 설정 (튜토리얼 본 것 등 나머지는 그대로). 저장은 로건이 `save_settings`로.
    pub fn settings_after(&self, base: &Settings) -> Settings {
        Settings { sound_on: self.on, music_percent: self.music_percent, effects_percent: self.effects_percent, ..*base }
    }

    pub fn is_on(&self) -> bool {
        self.on
    }

    /// M 키: 켜졌으면 끄고, 꺼졌으면 켠다. 바뀐 뒤 값(true = 켜짐)을 돌려준다.
    pub fn toggle(&mut self) -> bool {
        self.on = !self.on;
        self.on
    }

    pub fn set_on(&mut self, on: bool) {
        self.on = on;
    }

    pub fn music_percent(&self) -> u8 {
        self.music_percent
    }

    pub fn effects_percent(&self) -> u8 {
        self.effects_percent
    }

    /// 0~100으로 자른다 (음수는 0, 100 넘는 수는 100).
    pub fn set_music_percent(&mut self, percent: i32) {
        self.music_percent = clamp_percent(percent);
    }

    pub fn set_effects_percent(&mut self, percent: i32) {
        self.effects_percent = clamp_percent(percent);
    }

    /// 음악 크기 0.0~1.0. 꺼져 있으면 0 (곡은 멈추지 않고 크기만 0으로 해서 다시 켜면 바로 이어진다).
    /// 멈춤 창이 열려 있으면 `SOUND_PAUSED_MUSIC_FACTOR`배로 작게.
    pub fn music_volume(&self, paused: bool) -> f32 {
        if !self.on {
            return 0.0;
        }
        let v = self.music_percent as f32 / 100.0;
        if paused {
            v * SOUND_PAUSED_MUSIC_FACTOR
        } else {
            v
        }
    }

    /// 효과음 크기 0.0~1.0. 꺼져 있으면 0.
    pub fn effects_volume(&self) -> f32 {
        if self.on {
            self.effects_percent as f32 / 100.0
        } else {
            0.0
        }
    }

    /// 음악이 귀에 들리는가 (꺼졌거나 크기 0이면 false).
    // 시험에서만 쓴다 (화면은 아직 안 부른다). 시험 밖 빌드의 경고만 끈다.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn music_audible(&self) -> bool {
        self.on && self.music_percent > 0
    }

    /// 효과음이 나도 되는가: 켜져 있고, 크기가 0이 아니고, 멈춤 창이 닫혀 있다 (임시-115).
    pub fn effects_audible(&self, paused: bool) -> bool {
        self.on && self.effects_percent > 0 && !paused
    }
}

impl Default for SoundSettings {
    fn default() -> Self {
        SoundSettings::from_settings(&Settings::default())
    }
}

fn clamp_percent(percent: i32) -> u8 {
    percent.clamp(0, SETTINGS_VOLUME_MAX_PERCENT as i32) as u8
}

/// 멈춤 창의 4칸 버튼(끔·작게·보통·크게)을 누를 때 다음 칸 값. 맨 끝 다음은 처음(끔)으로 돌아간다.
/// 4칸 사이의 값(예: 70)이면 그보다 큰 가장 가까운 칸으로 간다.
// 시험에서만 쓴다 (화면은 아직 안 부른다). 시험 밖 빌드의 경고만 끈다.
#[cfg_attr(not(test), allow(dead_code))]
pub fn next_volume_step(current_percent: u8) -> u8 {
    SOUND_VOLUME_STEPS.iter().copied().find(|&step| step > current_percent).unwrap_or(SOUND_VOLUME_STEPS[0])
}

// ---------------------------------------------------------------- 효과음 이름표

/// 효과음 이름표. 파일 이름은 `file_name`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    // 싸울 때 (US-115)
    SwingFist,
    SwingBlade,
    ToyGun,
    Hit,
    /// 몬스터 쓰러짐 (뿅/퐁/띠용 중 `MONSTER_DEFEATED_CHOICE`)
    MonsterDefeated,
    PlayerHit,
    ArrowShot,
    CitizenPop,
    HeavyThud,
    PoisonFizz,
    BoxTap,
    // 줍고 캐고 만들 때 (US-116)
    CoinPickup,
    ItemPickup,
    MineWood,
    MineStone,
    MineDone,
    GemSparkle,
    CraftDone,
    ShopTrade,
    NotEnoughCoins,
    PotionGulp,
    /// 자동차 "부릉" (파일이 아직 없다: `file_name`이 None)
    CarVroom,
    CaveDoor,
    // 기쁜 순간 (US-117)
    /// 레벨업 (빰빠밤/별/뿅뿅 중 `LEVEL_UP_CHOICE`)
    LevelUp,
    MissFound,
    /// 퀴즈·깜짝 미션이 나올 때 "띵동"
    QuizStart,
    QuizCorrect,
    QuizWrong,
    MissionSuccess,
    MissionFail,
    NightBell,
    MorningDing,
    RareItemWow,
    GameClear,
    /// 내가 쓰러짐 (`PLAYER_DOWN_CHOICE`)
    PlayerDown,
    // 그 밖
    ButtonClick,
    WindowOpen,
    WindowClose,
    PracticeHit,
    TutorialCheer,
}

impl Sfx {
    /// 이름표 전부 (받는 순서에도 쓴다).
    pub const ALL: [Sfx; 40] = [
        Sfx::ButtonClick,
        Sfx::WindowOpen,
        Sfx::WindowClose,
        Sfx::CoinPickup,
        Sfx::SwingFist,
        Sfx::SwingBlade,
        Sfx::ToyGun,
        Sfx::Hit,
        Sfx::MonsterDefeated,
        Sfx::PlayerHit,
        Sfx::ArrowShot,
        Sfx::CitizenPop,
        Sfx::HeavyThud,
        Sfx::PoisonFizz,
        Sfx::BoxTap,
        Sfx::ItemPickup,
        Sfx::MineWood,
        Sfx::MineStone,
        Sfx::MineDone,
        Sfx::GemSparkle,
        Sfx::CraftDone,
        Sfx::ShopTrade,
        Sfx::NotEnoughCoins,
        Sfx::PotionGulp,
        Sfx::CarVroom,
        Sfx::CaveDoor,
        Sfx::LevelUp,
        Sfx::MissFound,
        Sfx::QuizStart,
        Sfx::QuizCorrect,
        Sfx::QuizWrong,
        Sfx::MissionSuccess,
        Sfx::MissionFail,
        Sfx::NightBell,
        Sfx::MorningDing,
        Sfx::RareItemWow,
        Sfx::GameClear,
        Sfx::PlayerDown,
        Sfx::PracticeHit,
        Sfx::TutorialCheer,
    ];

    /// `sounds/sfx/` 아래 파일 이름 (확장자 포함). 파일이 아직 없는 소리(자동차 부릉)는 None → 그냥 안 낸다.
    pub fn file_name(self) -> Option<&'static str> {
        Some(match self {
            Sfx::SwingFist => "swing_fist.wav",
            Sfx::SwingBlade => "swing_blade.wav",
            Sfx::ToyGun => "toy_gun.wav",
            Sfx::Hit | Sfx::PracticeHit => "hit.wav",
            Sfx::MonsterDefeated => pick_file(&MONSTER_DEFEATED_FILES, MONSTER_DEFEATED_CHOICE),
            Sfx::PlayerHit => "player_hit.wav",
            Sfx::ArrowShot => "arrow_whoosh.wav",
            Sfx::CitizenPop => "bomb_pop.wav",
            Sfx::HeavyThud => "heavy_thud.wav",
            Sfx::PoisonFizz => "poison_fizz.wav",
            Sfx::BoxTap => "box_tap.wav",
            Sfx::CoinPickup => "coin_pickup.wav",
            Sfx::ItemPickup => "item_pickup.wav",
            Sfx::MineWood => "dig_wood.wav",
            Sfx::MineStone => "dig_stone.wav",
            Sfx::MineDone => "dig_done.wav",
            Sfx::GemSparkle => "gem_sparkle.wav",
            Sfx::CraftDone => "craft_done.wav",
            Sfx::ShopTrade => "shop_trade.wav",
            Sfx::NotEnoughCoins => "not_enough_coins.wav",
            Sfx::PotionGulp => "potion_gulp.wav",
            Sfx::CarVroom => return None,
            Sfx::CaveDoor => "cave_door.wav",
            Sfx::LevelUp => pick_file(&LEVEL_UP_FILES, LEVEL_UP_CHOICE),
            Sfx::MissFound | Sfx::TutorialCheer => "miss_found.wav",
            Sfx::QuizStart => "quiz_start.wav",
            Sfx::QuizCorrect | Sfx::MissionSuccess => "correct.wav",
            Sfx::QuizWrong | Sfx::MissionFail => "wrong.wav",
            Sfx::NightBell => "night_bell.wav",
            Sfx::MorningDing => "morning_ding.wav",
            Sfx::RareItemWow => "rare_item_wow.wav",
            Sfx::GameClear => "clear_dance.wav",
            Sfx::PlayerDown => pick_file(&PLAYER_DOWN_FILES, PLAYER_DOWN_CHOICE),
            Sfx::ButtonClick => "button_click.wav",
            Sfx::WindowOpen => "window_open.wav",
            Sfx::WindowClose => "window_close.wav",
        })
    }

    /// `game/assets/` 아래 전체 경로 (예: `sounds/sfx/coin_pickup.wav`). 파일이 없는 소리는 None.
    pub fn asset_path(self) -> Option<String> {
        self.file_name().map(|f| format!("{SOUNDS_SFX_DIR}/{f}"))
    }

    /// 중요한 소리는 한꺼번에 몰릴 때 먼저 난다 (기쁜 순간, 쓰러짐).
    pub fn is_important(self) -> bool {
        matches!(
            self,
            Sfx::LevelUp
                | Sfx::QuizCorrect
                | Sfx::QuizWrong
                | Sfx::MissionSuccess
                | Sfx::MissionFail
                | Sfx::GameClear
                | Sfx::PlayerDown
                | Sfx::MorningDing
                | Sfx::NightBell
                | Sfx::RareItemWow
        )
    }
}

/// 이름 목록에서 `choice`번째 (번호가 틀려도 패닉 없이 첫 번째).
fn pick_file(files: &'static [&'static str], choice: usize) -> &'static str {
    files.get(choice).copied().unwrap_or(files[0])
}

// ---------------------------------------------------------------- 이벤트 → 효과음 (순수 함수)

/// 공격을 휘두를 때 (입력으로 알 수 있다). 맨손은 가볍게, 칼·도끼는 휘익, 총은 장난감 "퓽".
pub fn sfx_for_swing(weapon: Weapon) -> Sfx {
    match weapon {
        Weapon::Hands => Sfx::SwingFist,
        Weapon::Sword | Weapon::Axe => Sfx::SwingBlade,
        Weapon::Gun => Sfx::ToyGun,
    }
}

pub fn sfx_for_combat_event(event: &CombatEvent) -> Option<Sfx> {
    match event {
        CombatEvent::ZombieHit { .. } | CombatEvent::SkeletonHit { .. } | CombatEvent::CitizenDefeated { .. } => Some(Sfx::Hit),
        CombatEvent::ZombieDefeated { .. } | CombatEvent::SkeletonDefeated { .. } => Some(Sfx::MonsterDefeated),
        // 날아오른 좀비가 터지는 "펑"은 풍선 같은 뽕으로 (큰 폭발음 아님)
        CombatEvent::ZombiePopped { .. } => Some(Sfx::CitizenPop),
        CombatEvent::ProjectileFired { kind: ProjectileKind::Arrow | ProjectileKind::Bullet, .. } => Some(Sfx::ArrowShot),
        CombatEvent::LevelUp { .. } => Some(Sfx::LevelUp),
        // 무적 5초 동안은 게임이 이 이벤트를 다시 보내지 않는다
        CombatEvent::PlayerHurt { .. } => Some(Sfx::PlayerHit),
        CombatEvent::PlayerDown => Some(Sfx::PlayerDown),
        CombatEvent::ZombieAppeared { .. }
        | CombatEvent::SkeletonAppeared { .. }
        | CombatEvent::SkeletonDodged { .. }
        | CombatEvent::SkeletonHandHit { .. }
        | CombatEvent::ProjectileBounced { .. }
        | CombatEvent::HealthRefilled => None,
    }
}

pub fn sfx_for_game_event(event: &GameEvent) -> Option<Sfx> {
    match event {
        GameEvent::NightStarted => Some(Sfx::NightBell),
        GameEvent::DayStarted { .. } => Some(Sfx::MorningDing),
        GameEvent::CoinsPicked { .. } => Some(Sfx::CoinPickup), // 여러 개여도 한 번 (`count`는 안 본다)
        GameEvent::Combat(e) => sfx_for_combat_event(e),
        GameEvent::MissFound { .. } => Some(Sfx::MissFound),
        GameEvent::WeaponPicked { .. } => Some(Sfx::ItemPickup),
        GameEvent::CitizenPopped { .. } => Some(Sfx::CitizenPop),
        GameEvent::GameCleared => Some(Sfx::GameClear),
    }
}

/// 다 캠은 "퐁". 보석이 나오면 반짝 하나가 더 있다 (`bonus_sfx_for_gather_event`).
pub fn sfx_for_gather_event(event: &GatherEvent) -> Option<Sfx> {
    match event {
        GatherEvent::Mined { .. } => Some(Sfx::MineDone),
        GatherEvent::CaveEntered | GatherEvent::CaveLeft | GatherEvent::PushedOutOfCave => Some(Sfx::CaveDoor),
        GatherEvent::CraftingOpened => Some(Sfx::WindowOpen),
        GatherEvent::CaveClosed => None,
    }
}

/// 금, 은, 다이아몬드, 에메랄드, 루비를 다 캐면 "퐁"에 더하는 반짝.
pub fn bonus_sfx_for_gather_event(event: &GatherEvent) -> Option<Sfx> {
    match event {
        GatherEvent::Mined { what: Material::Gold | Material::Silver | Material::Diamond | Material::Emerald | Material::Ruby, .. } => {
            Some(Sfx::GemSparkle)
        }
        _ => None,
    }
}

pub fn sfx_for_item_event(event: &ItemEvent) -> Option<Sfx> {
    match event {
        ItemEvent::SyringeGunFired { .. } => Some(Sfx::ToyGun),
        ItemEvent::Slammed { .. } => Some(Sfx::HeavyThud),
        ItemEvent::CarOff | ItemEvent::EnderFruitEaten { .. } | ItemEvent::EnderFruitAlreadyEaten => None,
    }
}

pub fn sfx_for_shop_event(event: &ShopEvent) -> Option<Sfx> {
    match event {
        ShopEvent::Opened => Some(Sfx::WindowOpen),
        ShopEvent::HealthPotionUsed { .. } => Some(Sfx::PotionGulp),
        ShopEvent::PoisonSprayed { .. } => Some(Sfx::PoisonFizz),
        ShopEvent::SkatesOff | ShopEvent::ArmorOff => None,
    }
}

/// `Game::buy`의 결과: 사면 짤랑, 코인이 모자라면 부드러운 소리, 이미 있으면 소리 없음.
pub fn sfx_for_buy(result: &Result<(), CantBuy>) -> Option<Sfx> {
    match result {
        Ok(()) => Some(Sfx::ShopTrade),
        Err(CantBuy::NotEnoughCoins) => Some(Sfx::NotEnoughCoins),
        Err(CantBuy::AlreadyHave) => None,
    }
}

/// `Game::sell`이 팔았으면 짤랑. 못 팔면 소리 없음.
pub fn sfx_for_sell<E>(result: &Result<u32, E>) -> Option<Sfx> {
    result.is_ok().then_some(Sfx::ShopTrade)
}

/// 상자는 부서질 때 소리가 없다 (D87 아이 결정).
pub fn sfx_for_box_event(event: &BoxEvent) -> Option<Sfx> {
    match event {
        BoxEvent::Cracked { .. } | BoxEvent::NeedAxe { .. } => Some(Sfx::BoxTap),
        BoxEvent::Wow { .. } => Some(Sfx::RareItemWow),
        BoxEvent::ItemPicked { .. } => Some(Sfx::ItemPickup),
        BoxEvent::Broken { .. } | BoxEvent::ItemDropped { .. } | BoxEvent::MonsterJumpedOut { .. } | BoxEvent::CameBack { .. } => None,
    }
}

pub fn sfx_for_mission_event(event: &MissionEvent) -> Option<Sfx> {
    match event {
        MissionEvent::Appeared { .. } => Some(Sfx::QuizStart),
        MissionEvent::Succeeded { .. } => Some(Sfx::MissionSuccess),
        MissionEvent::Failed { .. } => Some(Sfx::MissionFail),
    }
}

/// 연습용 몬스터: 맞히면 맞힘 소리, 쓰러지면 쓰러짐 소리.
pub fn sfx_for_practice_event(event: &PracticeEvent) -> Option<Sfx> {
    match event {
        PracticeEvent::Hit { .. } => Some(Sfx::PracticeHit),
        PracticeEvent::Defeated { .. } => Some(Sfx::MonsterDefeated),
        PracticeEvent::Appeared { .. } | PracticeEvent::Vanished { .. } => None,
    }
}

/// 튜토리얼은 단계를 해냈을 때만 칭찬 소리.
pub fn sfx_for_tutorial_event(event: &TutorialEvent) -> Option<Sfx> {
    match event {
        TutorialEvent::StepDone(_) => Some(Sfx::TutorialCheer),
        _ => None,
    }
}

/// 퀴즈 답 결과: 맞으면 딩동댕, 틀리면(다시 해 보기 포함) 부드러운 아쉬운 소리.
pub fn sfx_for_math_result(result: MathResult) -> Sfx {
    match result {
        MathResult::Right => Sfx::QuizCorrect,
        MathResult::TryAgain | MathResult::Wrong => Sfx::QuizWrong,
    }
}

/// 캐는 중 "톡톡". 나무는 톡, 나머지 광석은 캉.
pub fn sfx_for_mining(material: Material) -> Sfx {
    match material {
        Material::Wood => Sfx::MineWood,
        _ => Sfx::MineStone,
    }
}

/// 캐는 동안 일정한 간격으로 "톡톡"을 알려 준다. 매 프레임 `update(dt, 캐는 것)`을 부른다.
#[derive(Debug, Clone, Copy, Default)]
pub struct MiningSound {
    active: bool,
    since_tick: f32,
}

impl MiningSound {
    /// `mining`이 Some이면 캐는 중. 캐기 시작한 첫 프레임과 그 뒤 `MINING_TICK_SECONDS`마다 소리를 돌려준다.
    pub fn update(&mut self, dt: f32, mining: Option<Material>) -> Option<Sfx> {
        let Some(material) = mining else {
            self.active = false;
            self.since_tick = 0.0;
            return None;
        };
        if !self.active {
            self.active = true;
            self.since_tick = 0.0;
            return Some(sfx_for_mining(material));
        }
        self.since_tick += if dt > 0.0 { dt } else { 0.0 };
        if self.since_tick >= MINING_TICK_SECONDS {
            self.since_tick = 0.0;
            return Some(sfx_for_mining(material));
        }
        None
    }
}

// ---------------------------------------------------------------- 요청 모으기 (겹침 막기)

/// 한 프레임에 들어온 효과음 요청을 모아서 시끄럽지 않게 걸러 낸다.
/// - 같은 프레임에 같은 소리는 한 번 (코인 여러 개 = 한 번)
/// - 같은 소리는 `SFX_MIN_GAP_SECONDS` 안에 또 나지 않는다
/// - `SFX_BURST_WINDOW_SECONDS` 안에 시작한 소리는 `SFX_MAX_AT_ONCE`개까지 (중요한 소리가 먼저)
/// - 소리가 꺼졌거나 크기 0이거나 멈춤 창이 열려 있으면 모두 버린다 (나중에 몰아서 나지 않는다)
#[derive(Debug, Clone, Default)]
pub struct SfxMixer {
    clock: f32,
    pending: Vec<Sfx>,
    played: Vec<(Sfx, f32)>,
}

impl SfxMixer {
    pub fn new() -> SfxMixer {
        SfxMixer::default()
    }

    /// 효과음을 부탁한다 (같은 프레임의 같은 소리는 합친다).
    pub fn request(&mut self, sfx: Sfx) {
        if self.pending.len() < SFX_PENDING_LIMIT && !self.pending.contains(&sfx) {
            self.pending.push(sfx);
        }
    }

    /// `sfx_for_*`가 돌려준 `Option`을 그대로 넣는다.
    pub fn request_opt(&mut self, sfx: Option<Sfx>) {
        if let Some(sfx) = sfx {
            self.request(sfx);
        }
    }

    /// 프레임 끝에 부른다. `dt`만큼 시간이 흐르고, 지금 내도 되는 소리만 돌려준다.
    pub fn take_playable(&mut self, dt: f32, sound: &SoundSettings, paused: bool) -> Vec<Sfx> {
        self.clock += if dt > 0.0 { dt } else { 0.0 };
        let mut wanted = std::mem::take(&mut self.pending);
        let now = self.clock;
        let keep_for = SFX_MIN_GAP_SECONDS.max(SFX_BURST_WINDOW_SECONDS);
        self.played.retain(|&(_, at)| now - at < keep_for);
        if !sound.effects_audible(paused) {
            return Vec::new();
        }
        wanted.sort_by_key(|sfx| !sfx.is_important()); // 안정 정렬: 중요한 것 먼저, 나머지는 들어온 순서
        let mut out = Vec::new();
        for sfx in wanted {
            if sfx.file_name().is_none() {
                continue; // 파일이 없는 소리
            }
            if self.played.iter().any(|&(s, at)| s == sfx && now - at < SFX_MIN_GAP_SECONDS) {
                continue;
            }
            if self.played.iter().filter(|&&(_, at)| now - at < SFX_BURST_WINDOW_SECONDS).count() >= SFX_MAX_AT_ONCE {
                continue;
            }
            self.played.push((sfx, now));
            out.push(sfx);
        }
        out
    }
}

// ---------------------------------------------------------------- 배경음악

/// 곡 이름표. 밤과 동굴은 같은 파일이다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Track {
    Title,
    Day,
    Night,
    Cave,
    Shop,
}

impl Track {
    /// 받는 순서: 타이틀 → 낮 → 밤(동굴 포함) → 상점
    pub const LOAD_ORDER: [Track; 5] = [Track::Title, Track::Day, Track::Night, Track::Cave, Track::Shop];

    /// `sounds/music/` 아래 파일 이름
    pub fn file_name(self) -> &'static str {
        match self {
            Track::Title => MUSIC_TITLE_FILE,
            Track::Day => MUSIC_DAY_FILE,
            Track::Night => MUSIC_NIGHT_FILE,
            Track::Cave => MUSIC_CAVE_FILE,
            Track::Shop => MUSIC_SHOP_FILE,
        }
    }

    pub fn asset_path(self) -> String {
        format!("{SOUNDS_MUSIC_DIR}/{}", self.file_name())
    }
}

/// 지금 어느 장면인가.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicScene {
    /// 로고 화면 (음악 없음)
    // 시험에서만 쓴다 (화면은 아직 안 부른다). 시험 밖 빌드의 경고만 끈다.
    #[cfg_attr(not(test), allow(dead_code))]
    Logo,
    Title,
    CharacterSelect,
    Playing,
    /// 쓰러진 뒤 화면 (낮·밤 음악이 멈춘다)
    GameOver,
    /// 7일째 클리어 화면 (낮·밤 음악이 멈춘다)
    Cleared,
}

/// 곡을 고르는 데 필요한 것들.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MusicSituation {
    pub scene: MusicScene,
    pub phase: Phase,
    pub in_cave: bool,
    pub shop_open: bool,
}

impl MusicSituation {
    /// 노는 중 장면을 게임에서 읽는다. 쓰러졌거나 클리어했으면 `GameOver`/`Cleared`.
    pub fn from_game(game: &Game) -> MusicSituation {
        let scene = if game.is_cleared() {
            MusicScene::Cleared
        } else if game.is_over() {
            MusicScene::GameOver
        } else {
            MusicScene::Playing
        };
        MusicSituation { scene, phase: game.clock.phase(), in_cave: game.in_cave, shop_open: game.shop.is_open() }
    }

    /// 타이틀·캐릭터 고르기·로고처럼 게임이 없는 장면.
    pub fn menu(scene: MusicScene) -> MusicSituation {
        MusicSituation { scene, phase: Phase::Day, in_cave: false, shop_open: false }
    }
}

/// 지금 나와야 할 곡. None이면 음악이 없다 (로고, 쓰러진 뒤, 클리어).
/// 우선순위: 상점 > 동굴 > 밤 > 낮. 제작대·가방·퀴즈·멈춤 창은 곡을 바꾸지 않는다 (US-114).
pub fn choose_track(s: &MusicSituation) -> Option<Track> {
    match s.scene {
        MusicScene::Logo | MusicScene::GameOver | MusicScene::Cleared => None,
        MusicScene::Title | MusicScene::CharacterSelect => Some(Track::Title),
        MusicScene::Playing => Some(if s.shop_open {
            Track::Shop
        } else if s.in_cave {
            Track::Cave
        } else if s.phase == Phase::Night {
            Track::Night
        } else {
            Track::Day
        }),
    }
}

/// 곡을 바꾸라는 신호.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicChange {
    /// 이 곡으로 부드럽게 바꾼다 (`MUSIC_FADE_SECONDS`, 섞기는 로건)
    Play(Track),
    /// 음악을 멈춘다
    Stop,
}

/// 지금 틀고 있는 곡을 기억해서 "바뀔 때만" 신호를 준다.
#[derive(Debug, Clone, Copy, Default)]
pub struct MusicPlan {
    current: Option<Track>,
}

impl MusicPlan {
    pub fn new() -> MusicPlan {
        MusicPlan::default()
    }

    // 시험에서만 쓴다 (화면은 아직 안 부른다). 시험 밖 빌드의 경고만 끈다.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn current(&self) -> Option<Track> {
        self.current
    }

    /// 매 프레임 `choose_track`의 결과를 넣는다. 곡 파일이 바뀔 때만 `Some`.
    /// 밤 → 동굴처럼 같은 파일이면 신호가 없어서 곡이 끊기지 않고 이어진다.
    pub fn update(&mut self, wanted: Option<Track>) -> Option<MusicChange> {
        let before = self.current;
        self.current = wanted;
        match (before, wanted) {
            (None, None) => None,
            (Some(_), None) => Some(MusicChange::Stop),
            (None, Some(track)) => Some(MusicChange::Play(track)),
            (Some(old), Some(track)) => (old.file_name() != track.file_name()).then_some(MusicChange::Play(track)),
        }
    }
}

// ---------------------------------------------------------------- 받는 순서

/// 받을 소리 하나.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadItem {
    Sfx(Sfx),
    Music(Track),
}

impl LoadItem {
    /// `game/assets/` 아래 경로.
    pub fn asset_path(self) -> Option<String> {
        match self {
            LoadItem::Sfx(sfx) => sfx.asset_path(),
            LoadItem::Music(track) => Some(track.asset_path()),
        }
    }
}

/// `눌러서 시작` 뒤에 받는 순서: 효과음 전부 → 음악(타이틀 → 낮 → 밤 → 상점).
/// 같은 파일은 한 번만 (밤·동굴, 맞힘·연습 맞힘 등). 파일이 없는 소리는 뺀다.
pub fn loading_order() -> Vec<LoadItem> {
    let mut items: Vec<LoadItem> = Vec::new();
    let mut paths: Vec<String> = Vec::new();
    let all = Sfx::ALL.iter().copied().map(LoadItem::Sfx).chain(Track::LOAD_ORDER.iter().copied().map(LoadItem::Music));
    for item in all {
        let Some(path) = item.asset_path() else { continue };
        if !paths.contains(&path) {
            paths.push(path);
            items.push(item);
        }
    }
    items
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::boxes::SurpriseItem;
    use crate::logic::character::CharacterKind;
    use crate::logic::mission::MissionKind;
    use crate::logic::skeleton::SkeletonStage;
    use crate::logic::tutorial::TutorialStep;
    use macroquad::math::Vec2;
    use std::collections::HashSet;
    use std::path::PathBuf;

    const AT: Vec2 = Vec2::ZERO;

    fn assets_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
    }

    fn files_in(dir: &str) -> Vec<(String, u64)> {
        let mut out = Vec::new();
        for entry in std::fs::read_dir(assets_dir().join(dir)).expect("소리 폴더가 있어야 한다").take(500) {
            let entry = entry.unwrap();
            out.push((entry.file_name().to_string_lossy().to_string(), entry.metadata().unwrap().len()));
        }
        out
    }

    // ---- 설정 ----

    #[test]
    fn settings_round_trip_keeps_other_values() {
        let base = Settings { tutorial_done: true, ..Settings::default() };
        let mut sound = SoundSettings::from_settings(&base);
        sound.set_music_percent(30);
        sound.set_effects_percent(60);
        sound.toggle();
        let after = sound.settings_after(&base);
        assert!(after.tutorial_done);
        assert_eq!((after.sound_on, after.music_percent, after.effects_percent), (!base.sound_on, 30, 60));
        assert_eq!(SoundSettings::from_settings(&after), sound);
    }

    #[test]
    fn settings_default_is_on_and_matches_config() {
        let s = SoundSettings::default();
        assert!(s.is_on());
        assert_eq!(s.music_percent(), SETTINGS_DEFAULT_MUSIC_PERCENT);
        assert_eq!(s.effects_percent(), SETTINGS_DEFAULT_EFFECTS_PERCENT);
    }

    #[test]
    fn strange_saved_values_are_clamped() {
        let weird = Settings { music_percent: 250, effects_percent: 255, ..Settings::default() };
        let s = SoundSettings::from_settings(&weird);
        assert_eq!((s.music_percent(), s.effects_percent()), (100, 100));
    }

    #[test]
    fn percent_is_clamped_to_0_100() {
        let mut s = SoundSettings::default();
        s.set_music_percent(-5);
        s.set_effects_percent(1000);
        assert_eq!((s.music_percent(), s.effects_percent()), (0, 100));
        s.set_music_percent(i32::MIN);
        s.set_effects_percent(i32::MAX);
        assert_eq!((s.music_percent(), s.effects_percent()), (0, 100));
    }

    #[test]
    fn volumes_are_fractions() {
        let mut s = SoundSettings::default();
        s.set_music_percent(50);
        s.set_effects_percent(100);
        assert_eq!(s.music_volume(false), 0.5);
        assert_eq!(s.effects_volume(), 1.0);
        assert!((s.music_volume(true) - 0.5 * SOUND_PAUSED_MUSIC_FACTOR).abs() < 1e-6);
        assert!(s.music_volume(true) < s.music_volume(false));
    }

    #[test]
    fn m_key_toggle_mutes_and_unmutes() {
        let mut s = SoundSettings::default();
        assert!(!s.toggle());
        assert_eq!((s.music_volume(false), s.effects_volume()), (0.0, 0.0));
        assert!(!s.music_audible() && !s.effects_audible(false));
        assert!(s.toggle());
        assert!(s.music_audible() && s.effects_audible(false));
        // 끄고 켜도 고른 크기는 그대로
        assert_eq!(s.effects_percent(), SETTINGS_DEFAULT_EFFECTS_PERCENT);
    }

    #[test]
    fn paused_silences_effects_but_not_music() {
        let s = SoundSettings::default();
        assert!(!s.effects_audible(true));
        assert!(s.music_audible());
        assert!(s.music_volume(true) > 0.0);
    }

    #[test]
    fn zero_percent_is_silent() {
        let mut s = SoundSettings::default();
        s.set_effects_percent(0);
        s.set_music_percent(0);
        assert!(!s.effects_audible(false) && !s.music_audible());
    }

    #[test]
    fn volume_steps_cycle() {
        assert_eq!(next_volume_step(0), 25);
        assert_eq!(next_volume_step(25), 50);
        assert_eq!(next_volume_step(50), 80);
        assert_eq!(next_volume_step(80), 0);
        assert_eq!(next_volume_step(70), 80); // 칸 사이 값
        assert_eq!(next_volume_step(100), 0);
        // 반복에는 횟수 제한: 4번이면 한 바퀴
        let mut v = 0u8;
        for _ in 0..4 {
            v = next_volume_step(v);
        }
        assert_eq!(v, 0);
    }

    // ---- 이벤트 → 효과음 ----

    #[test]
    fn swing_sounds_differ_by_weapon() {
        assert_eq!(sfx_for_swing(Weapon::Hands), Sfx::SwingFist);
        assert_eq!(sfx_for_swing(Weapon::Sword), Sfx::SwingBlade);
        assert_eq!(sfx_for_swing(Weapon::Axe), Sfx::SwingBlade);
        assert_eq!(sfx_for_swing(Weapon::Gun), Sfx::ToyGun);
    }

    #[test]
    fn combat_events_map() {
        use CombatEvent::*;
        assert_eq!(sfx_for_combat_event(&ZombieHit { at: AT, hp_left: 1 }), Some(Sfx::Hit));
        assert_eq!(sfx_for_combat_event(&SkeletonHit { at: AT, hp_left: 1 }), Some(Sfx::Hit));
        assert_eq!(sfx_for_combat_event(&ZombieDefeated { at: AT }), Some(Sfx::MonsterDefeated));
        assert_eq!(sfx_for_combat_event(&SkeletonDefeated { at: AT }), Some(Sfx::MonsterDefeated));
        assert_eq!(sfx_for_combat_event(&ZombiePopped { at: AT }), Some(Sfx::CitizenPop));
        assert_eq!(sfx_for_combat_event(&ProjectileFired { at: AT, kind: ProjectileKind::Arrow }), Some(Sfx::ArrowShot));
        assert_eq!(sfx_for_combat_event(&LevelUp { level: 2 }), Some(Sfx::LevelUp));
        assert_eq!(sfx_for_combat_event(&PlayerHurt { amount: 2, hp_left: 5 }), Some(Sfx::PlayerHit));
        assert_eq!(sfx_for_combat_event(&PlayerDown), Some(Sfx::PlayerDown));
        assert_eq!(sfx_for_combat_event(&ZombieAppeared { at: AT }), None);
        assert_eq!(sfx_for_combat_event(&SkeletonAppeared { at: AT, stage: SkeletonStage::Normal }), None);
        assert_eq!(sfx_for_combat_event(&SkeletonDodged { at: AT }), None);
        assert_eq!(sfx_for_combat_event(&SkeletonHandHit { at: AT }), None);
        assert_eq!(sfx_for_combat_event(&ProjectileBounced { at: AT }), None);
        assert_eq!(sfx_for_combat_event(&HealthRefilled), None);
    }

    #[test]
    fn game_events_map() {
        assert_eq!(sfx_for_game_event(&GameEvent::NightStarted), Some(Sfx::NightBell));
        assert_eq!(sfx_for_game_event(&GameEvent::DayStarted { day_number: 2 }), Some(Sfx::MorningDing));
        assert_eq!(sfx_for_game_event(&GameEvent::CoinsPicked { count: 3 }), Some(Sfx::CoinPickup));
        assert_eq!(sfx_for_game_event(&GameEvent::Combat(CombatEvent::PlayerDown)), Some(Sfx::PlayerDown));
        assert_eq!(sfx_for_game_event(&GameEvent::MissFound { at: AT }), Some(Sfx::MissFound));
        assert_eq!(sfx_for_game_event(&GameEvent::WeaponPicked { kind: Weapon::Axe }), Some(Sfx::ItemPickup));
        assert_eq!(sfx_for_game_event(&GameEvent::CitizenPopped { at: AT }), Some(Sfx::CitizenPop));
        assert_eq!(sfx_for_game_event(&GameEvent::GameCleared), Some(Sfx::GameClear));
    }

    #[test]
    fn gather_events_map_and_gems_add_sparkle() {
        let mined = |what| GatherEvent::Mined { what, at: AT };
        assert_eq!(sfx_for_gather_event(&mined(Material::Wood)), Some(Sfx::MineDone));
        for gem in [Material::Gold, Material::Silver, Material::Diamond, Material::Emerald, Material::Ruby] {
            assert_eq!(bonus_sfx_for_gather_event(&mined(gem)), Some(Sfx::GemSparkle));
        }
        for plain in [Material::Wood, Material::Iron, Material::Copper] {
            assert_eq!(bonus_sfx_for_gather_event(&mined(plain)), None);
        }
        assert_eq!(sfx_for_gather_event(&GatherEvent::CaveEntered), Some(Sfx::CaveDoor));
        assert_eq!(sfx_for_gather_event(&GatherEvent::CaveLeft), Some(Sfx::CaveDoor));
        assert_eq!(sfx_for_gather_event(&GatherEvent::PushedOutOfCave), Some(Sfx::CaveDoor));
        assert_eq!(sfx_for_gather_event(&GatherEvent::CraftingOpened), Some(Sfx::WindowOpen));
        assert_eq!(sfx_for_gather_event(&GatherEvent::CaveClosed), None);
        assert_eq!(bonus_sfx_for_gather_event(&GatherEvent::CaveEntered), None);
    }

    #[test]
    fn item_shop_events_map() {
        assert_eq!(sfx_for_item_event(&ItemEvent::SyringeGunFired { at: AT, monsters: 2 }), Some(Sfx::ToyGun));
        assert_eq!(sfx_for_item_event(&ItemEvent::CarOff), None);
        assert_eq!(sfx_for_shop_event(&ShopEvent::Opened), Some(Sfx::WindowOpen));
        assert_eq!(sfx_for_shop_event(&ShopEvent::HealthPotionUsed { at: AT, healed: 3 }), Some(Sfx::PotionGulp));
        assert_eq!(sfx_for_shop_event(&ShopEvent::PoisonSprayed { at: AT, monsters: 1 }), Some(Sfx::PoisonFizz));
        assert_eq!(sfx_for_shop_event(&ShopEvent::SkatesOff), None);
    }

    #[test]
    fn buy_and_sell_results_map() {
        assert_eq!(sfx_for_buy(&Ok(())), Some(Sfx::ShopTrade));
        assert_eq!(sfx_for_buy(&Err(CantBuy::NotEnoughCoins)), Some(Sfx::NotEnoughCoins));
        assert_eq!(sfx_for_buy(&Err(CantBuy::AlreadyHave)), None);
        assert_eq!(sfx_for_sell::<()>(&Ok(10)), Some(Sfx::ShopTrade));
        assert_eq!(sfx_for_sell(&Err(())), None);
    }

    #[test]
    fn box_breaking_is_silent() {
        assert_eq!(sfx_for_box_event(&BoxEvent::Broken { index: 0, at: AT }), None);
        assert_eq!(sfx_for_box_event(&BoxEvent::Cracked { index: 0, at: AT, cracks: 1 }), Some(Sfx::BoxTap));
        assert_eq!(sfx_for_box_event(&BoxEvent::Wow { at: AT }), Some(Sfx::RareItemWow));
        assert_eq!(sfx_for_box_event(&BoxEvent::ItemPicked { item: SurpriseItem::Sword }), Some(Sfx::ItemPickup));
        assert_eq!(sfx_for_box_event(&BoxEvent::CameBack { index: 0 }), None);
    }

    #[test]
    fn mission_practice_tutorial_quiz_map() {
        let kind = MissionKind::Coins;
        assert_eq!(sfx_for_mission_event(&MissionEvent::Appeared { kind, goal: 3 }), Some(Sfx::QuizStart));
        assert_eq!(sfx_for_mission_event(&MissionEvent::Succeeded { kind }), Some(Sfx::MissionSuccess));
        assert_eq!(sfx_for_mission_event(&MissionEvent::Failed { kind }), Some(Sfx::MissionFail));
        assert_eq!(sfx_for_practice_event(&PracticeEvent::Hit { at: AT, hp_left: 1 }), Some(Sfx::PracticeHit));
        assert_eq!(sfx_for_practice_event(&PracticeEvent::Defeated { at: AT }), Some(Sfx::MonsterDefeated));
        assert_eq!(sfx_for_practice_event(&PracticeEvent::Vanished { at: AT }), None);
        assert_eq!(sfx_for_tutorial_event(&TutorialEvent::StepDone(TutorialStep::Move)), Some(Sfx::TutorialCheer));
        assert_eq!(sfx_for_tutorial_event(&TutorialEvent::StepStarted(TutorialStep::Move)), None);
        assert_eq!(sfx_for_tutorial_event(&TutorialEvent::Skipped), None);
        assert_eq!(sfx_for_math_result(MathResult::Right), Sfx::QuizCorrect);
        assert_eq!(sfx_for_math_result(MathResult::Wrong), Sfx::QuizWrong);
        assert_eq!(sfx_for_math_result(MathResult::TryAgain), Sfx::QuizWrong);
    }

    #[test]
    fn mining_sound_ticks_at_interval_and_resets() {
        let mut m = MiningSound::default();
        assert_eq!(m.update(0.016, None), None);
        assert_eq!(m.update(0.016, Some(Material::Wood)), Some(Sfx::MineWood)); // 시작하자마자 한 번
        assert_eq!(m.update(0.016, Some(Material::Wood)), None);
        let mut ticks = 0;
        for _ in 0..100 {
            // 1초 동안
            if m.update(0.01, Some(Material::Iron)).is_some() {
                ticks += 1;
            }
        }
        assert!((2..=3).contains(&ticks), "ticks = {ticks}");
        assert_eq!(m.update(0.016, None), None);
        assert_eq!(m.update(0.016, Some(Material::Gold)), Some(Sfx::MineStone)); // 멈췄다 다시 시작하면 또 한 번
        assert_eq!(m.update(-1.0, Some(Material::Gold)), None); // 이상한 dt도 패닉 없음
    }

    // ---- 겹침 막기 ----

    fn mixer_out(m: &mut SfxMixer, dt: f32) -> Vec<Sfx> {
        m.take_playable(dt, &SoundSettings::default(), false)
    }

    #[test]
    fn same_frame_same_sound_merges() {
        let mut m = SfxMixer::new();
        for _ in 0..10 {
            m.request(Sfx::CoinPickup);
        }
        assert_eq!(mixer_out(&mut m, 0.016), vec![Sfx::CoinPickup]);
    }

    #[test]
    fn same_sound_waits_min_gap() {
        let mut m = SfxMixer::new();
        m.request(Sfx::Hit);
        assert_eq!(mixer_out(&mut m, 0.016), vec![Sfx::Hit]);
        m.request(Sfx::Hit);
        assert!(mixer_out(&mut m, 0.016).is_empty()); // 0.016초 뒤: 아직 0.05 안
        m.request(Sfx::Hit);
        assert!(mixer_out(&mut m, 0.016).is_empty()); // 0.032초
        m.request(Sfx::Hit);
        assert_eq!(mixer_out(&mut m, 0.03), vec![Sfx::Hit]); // 0.062초
    }

    #[test]
    fn different_sounds_play_together_up_to_limit() {
        let mut m = SfxMixer::new();
        let many = [Sfx::Hit, Sfx::BoxTap, Sfx::CoinPickup, Sfx::ItemPickup, Sfx::PotionGulp, Sfx::CaveDoor];
        for s in many {
            m.request(s);
        }
        let out = mixer_out(&mut m, 0.016);
        assert_eq!(out.len(), SFX_MAX_AT_ONCE);
        assert_eq!(out, many[..SFX_MAX_AT_ONCE].to_vec()); // 들어온 순서대로
        // 같은 창 안(0.25초)에는 더 못 나고, 지나면 다시 난다
        m.request(Sfx::PotionGulp);
        assert!(mixer_out(&mut m, 0.016).is_empty());
        m.request(Sfx::PotionGulp);
        assert_eq!(mixer_out(&mut m, SFX_BURST_WINDOW_SECONDS), vec![Sfx::PotionGulp]);
    }

    #[test]
    fn important_sounds_go_first() {
        let mut m = SfxMixer::new();
        for s in [Sfx::Hit, Sfx::BoxTap, Sfx::CoinPickup, Sfx::ItemPickup, Sfx::LevelUp] {
            m.request(s);
        }
        let out = mixer_out(&mut m, 0.016);
        assert_eq!(out.len(), SFX_MAX_AT_ONCE);
        assert_eq!(out[0], Sfx::LevelUp);
        assert!(!out.contains(&Sfx::ItemPickup)); // 중요하지 않은 맨 뒤 것이 밀린다
    }

    #[test]
    fn muted_or_paused_drops_everything() {
        let mut off = SoundSettings::default();
        off.toggle();
        let mut m = SfxMixer::new();
        m.request(Sfx::LevelUp);
        assert!(m.take_playable(0.016, &off, false).is_empty());
        m.request(Sfx::LevelUp);
        assert!(m.take_playable(0.016, &SoundSettings::default(), true).is_empty());
        // 버려진 소리가 나중에 몰려 나오지 않는다
        assert!(mixer_out(&mut m, 0.016).is_empty());
        // 버려진 것은 "이미 낸 것"으로 세지 않아서, 켜진 뒤 바로 난다
        m.request(Sfx::LevelUp);
        assert_eq!(mixer_out(&mut m, 0.016), vec![Sfx::LevelUp]);
    }

    #[test]
    fn missing_file_sound_is_not_played_and_bad_input_is_safe() {
        let mut m = SfxMixer::new();
        m.request(Sfx::CarVroom);
        assert!(mixer_out(&mut m, f32::NAN).is_empty());
        for _ in 0..(SFX_PENDING_LIMIT * 3) {
            m.request_opt(Some(Sfx::Hit));
            m.request_opt(None);
        }
        assert_eq!(m.pending.len(), 1);
    }

    #[test]
    fn pending_is_bounded() {
        let mut m = SfxMixer::new();
        for s in Sfx::ALL {
            m.request(s);
        }
        assert!(m.pending.len() <= SFX_PENDING_LIMIT);
    }

    // ---- 파일 이름 ----

    #[test]
    fn every_sfx_file_exists() {
        let dir = assets_dir().join(SOUNDS_SFX_DIR);
        for sfx in Sfx::ALL.iter().copied() {
            if let Some(name) = sfx.file_name() {
                assert!(dir.join(name).is_file(), "{sfx:?} 파일 없음: {name}");
            }
        }
        for name in MONSTER_DEFEATED_FILES.iter().chain(&LEVEL_UP_FILES).chain(&PLAYER_DOWN_FILES) {
            assert!(dir.join(name).is_file(), "고를 수 있는 소리 파일 없음: {name}");
        }
    }

    #[test]
    fn every_music_file_exists() {
        let dir = assets_dir().join(SOUNDS_MUSIC_DIR);
        for track in Track::LOAD_ORDER {
            assert!(dir.join(track.file_name()).is_file(), "{track:?} 곡 없음");
        }
    }

    #[test]
    fn every_file_in_sounds_folders_is_used() {
        let mut used: HashSet<&str> = HashSet::new();
        for sfx in Sfx::ALL.iter().copied() {
            used.extend(sfx.file_name());
        }
        used.extend(MONSTER_DEFEATED_FILES.iter().chain(&LEVEL_UP_FILES).chain(&PLAYER_DOWN_FILES));
        for (name, _) in files_in(SOUNDS_SFX_DIR) {
            assert!(used.contains(name.as_str()), "쓰지 않는 효과음 파일: {name}");
        }
        let tracks: HashSet<&str> = Track::LOAD_ORDER.iter().map(|t| t.file_name()).collect();
        for (name, _) in files_in(SOUNDS_MUSIC_DIR) {
            assert!(tracks.contains(name.as_str()), "쓰지 않는 음악 파일: {name}");
        }
    }

    #[test]
    fn sfx_all_has_no_duplicates() {
        let set: HashSet<String> = Sfx::ALL.iter().map(|s| format!("{s:?}")).collect();
        assert_eq!(set.len(), Sfx::ALL.len());
    }

    #[test]
    fn total_sound_size_within_limit() {
        let total: u64 = files_in(SOUNDS_SFX_DIR).iter().chain(files_in(SOUNDS_MUSIC_DIR).iter()).map(|(_, size)| size).sum();
        assert!(total <= SOUNDS_TOTAL_MAX_BYTES, "소리 합계 {total}바이트");
    }

    #[test]
    fn variation_choices_are_in_range() {
        assert!(MONSTER_DEFEATED_CHOICE < MONSTER_DEFEATED_FILES.len());
        assert!(LEVEL_UP_CHOICE < LEVEL_UP_FILES.len());
        assert!(PLAYER_DOWN_CHOICE < PLAYER_DOWN_FILES.len());
        assert_eq!(Sfx::MonsterDefeated.file_name(), Some(MONSTER_DEFEATED_FILES[MONSTER_DEFEATED_CHOICE]));
        assert_eq!(pick_file(&LEVEL_UP_FILES, 99), LEVEL_UP_FILES[0]); // 번호가 틀려도 패닉 없음
    }

    // ---- 음악 ----

    fn playing(phase: Phase, in_cave: bool, shop_open: bool) -> MusicSituation {
        MusicSituation { scene: MusicScene::Playing, phase, in_cave, shop_open }
    }

    #[test]
    fn track_by_scene() {
        assert_eq!(choose_track(&MusicSituation::menu(MusicScene::Logo)), None);
        assert_eq!(choose_track(&MusicSituation::menu(MusicScene::Title)), Some(Track::Title));
        assert_eq!(choose_track(&MusicSituation::menu(MusicScene::CharacterSelect)), Some(Track::Title));
        assert_eq!(choose_track(&MusicSituation::menu(MusicScene::GameOver)), None);
        assert_eq!(choose_track(&MusicSituation::menu(MusicScene::Cleared)), None);
    }

    #[test]
    fn track_priority_shop_cave_night_day() {
        assert_eq!(choose_track(&playing(Phase::Day, false, false)), Some(Track::Day));
        assert_eq!(choose_track(&playing(Phase::Night, false, false)), Some(Track::Night));
        assert_eq!(choose_track(&playing(Phase::Day, true, false)), Some(Track::Cave));
        assert_eq!(choose_track(&playing(Phase::Night, true, false)), Some(Track::Cave));
        assert_eq!(choose_track(&playing(Phase::Day, false, true)), Some(Track::Shop));
        assert_eq!(choose_track(&playing(Phase::Night, true, true)), Some(Track::Shop));
    }

    #[test]
    fn game_over_and_clear_stop_music_even_in_shop_or_cave() {
        for scene in [MusicScene::GameOver, MusicScene::Cleared] {
            let s = MusicSituation { scene, phase: Phase::Night, in_cave: true, shop_open: true };
            assert_eq!(choose_track(&s), None);
        }
    }

    #[test]
    fn music_plan_signals_only_on_change() {
        let mut plan = MusicPlan::new();
        assert_eq!(plan.update(None), None); // 로고
        assert_eq!(plan.update(Some(Track::Title)), Some(MusicChange::Play(Track::Title)));
        for _ in 0..50 {
            assert_eq!(plan.update(Some(Track::Title)), None); // 같은 곡은 계속
        }
        assert_eq!(plan.update(Some(Track::Day)), Some(MusicChange::Play(Track::Day)));
        assert_eq!(plan.update(Some(Track::Shop)), Some(MusicChange::Play(Track::Shop)));
        assert_eq!(plan.update(Some(Track::Day)), Some(MusicChange::Play(Track::Day)));
        assert_eq!(plan.update(None), Some(MusicChange::Stop));
        assert_eq!(plan.update(None), None);
        assert_eq!(plan.current(), None);
        // 새 게임: 낮 음악부터
        assert_eq!(plan.update(Some(Track::Day)), Some(MusicChange::Play(Track::Day)));
    }

    #[test]
    fn night_to_cave_does_not_change_music() {
        assert_eq!(Track::Night.file_name(), Track::Cave.file_name());
        let mut plan = MusicPlan::new();
        plan.update(Some(Track::Night));
        assert_eq!(plan.update(Some(Track::Cave)), None);
        assert_eq!(plan.update(Some(Track::Night)), None);
        assert_eq!(plan.current(), Some(Track::Night));
        // 낮 → 동굴은 곡이 다르니 바뀐다
        plan.update(Some(Track::Day));
        assert_eq!(plan.update(Some(Track::Cave)), Some(MusicChange::Play(Track::Cave)));
    }

    #[test]
    fn music_files_are_distinct_except_night_cave() {
        let names: HashSet<&str> = Track::LOAD_ORDER.iter().map(|t| t.file_name()).collect();
        assert_eq!(names.len(), 4);
    }

    #[test]
    fn situation_from_a_fresh_game_is_day_playing() {
        let game = Game::new(CharacterKind::Niro, 7);
        let s = MusicSituation::from_game(&game);
        assert_eq!(choose_track(&s), Some(Track::Day));
    }

    // ---- 받는 순서 ----

    #[test]
    fn loading_order_sfx_first_then_music_in_order() {
        let order = loading_order();
        let first_music = order.iter().position(|i| matches!(i, LoadItem::Music(_))).unwrap();
        assert!(order[..first_music].iter().all(|i| matches!(i, LoadItem::Sfx(_))));
        assert!(order[first_music..].iter().all(|i| matches!(i, LoadItem::Music(_))));
        let music: Vec<Track> = order
            .iter()
            .filter_map(|i| match i {
                LoadItem::Music(t) => Some(*t),
                _ => None,
            })
            .collect();
        assert_eq!(music, vec![Track::Title, Track::Day, Track::Night, Track::Shop]); // 동굴은 밤 파일이라 따로 안 받는다
    }

    #[test]
    fn loading_order_has_no_duplicate_files_and_covers_all() {
        let order = loading_order();
        let paths: Vec<String> = order.iter().filter_map(|i| i.asset_path()).collect();
        let unique: HashSet<&String> = paths.iter().collect();
        assert_eq!(paths.len(), order.len()); // 파일 없는 소리는 이미 빠져 있다
        assert_eq!(unique.len(), paths.len());
        // 쓰는 효과음 파일 38개(부릉 제외, 같은 파일 합침 전) 중 고른 것들이 모두 들어 있다
        for sfx in Sfx::ALL.iter().copied() {
            if let Some(p) = sfx.asset_path() {
                assert!(paths.contains(&p), "{p} 가 받는 목록에 없다");
            }
        }
        assert!(!order.contains(&LoadItem::Sfx(Sfx::CarVroom)));
    }

    #[test]
    fn asset_paths_look_right() {
        assert_eq!(Sfx::CoinPickup.asset_path().as_deref(), Some("sounds/sfx/coin_pickup.wav"));
        assert_eq!(Sfx::CarVroom.asset_path(), None);
        assert_eq!(Track::Day.asset_path(), "sounds/music/day_overworld.ogg");
    }
}
