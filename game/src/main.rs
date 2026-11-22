//! 미스 찾기 — 게임 루프: 입력 → logic 업데이트 → 이벤트 처리 → 그리기.
//! 화면 흐름: 타이틀 → 캐릭터 고르기 → 플레이 (Esc를 누르면 멈춤 창, 거기서 "처음으로"를 누르면 다시 고르기).
//! 플레이에서 쓰러지거나 클리어하면 같은 장면 위에 끝난 화면이 나오고, "다시 하기"로 고르기에 간다.

mod config;
mod logic;
mod view;

use logic::character::CharacterKind;
use logic::combat::CombatEvent;
use logic::game::{Game, GameEvent, GatherEvent};
use logic::help::Help;
use logic::mission::MissionEvent;
use logic::player::PlayerInput;
use logic::settings::Settings;
use logic::tutorial::{Tutorial, TutorialEvent, TutorialState};
use logic::window_tips::{TipWindow, WindowTips};
use macroquad::prelude::*;
use logic::gathering::Material;
use logic::sound::{
    bonus_sfx_for_gather_event, choose_track, sfx_for_box_event, sfx_for_game_event, sfx_for_gather_event, sfx_for_item_event, sfx_for_math_result, sfx_for_mission_event, sfx_for_practice_event, sfx_for_weapon_event,
    sfx_for_shop_event, sfx_for_swing, sfx_for_tutorial_event, MiningSound, MusicChange, MusicPlan, MusicScene, MusicSituation, Sfx, SfxMixer, SoundSettings,
};
use view::assets::Assets;
use view::audio::{draw_toast, sound_key_pressed, Audio, SoundToast, WindowFlags, WindowSounds};
use view::craft::{apply_craft_tap, craft_tap_at, draw_craft};
use view::pause::{draw_pause, pause_choose, pause_sound_tap_at, PauseBuddy, PauseKeys, PauseSoundTap, PauseTap};
use view::shop::Pocket;
use view::effects::Effects;
use view::gameover::{EndScreen, RETRY_BUTTON};
use view::bag::{bag_button_rect, bag_tap_at, draw_bag_window, use_pocket_or_why, BagView};
use view::input::{
    any_key_pressed, bag_key_pressed, fast_forward_pressed, fingers, hold_walk_until_released, math_keys_pressed, menu_input, merge_touch, mine_key_down, player_input,
    pocket_keys_pressed, switch_weapon_pressed, up_enters, MenuInput,
};
use view::gather::GatherView;
use view::math::{apply_quiz_input, draw_math};
use view::touch::{
    draw_back_button, draw_rotate_hint, draw_touch_controls, next_touch_mode, play_buttons_with_mine, show_rotate_hint, ButtonKind, TouchControls,
    TouchFrame, BACK_BUTTON,
};
use view::screen::Letterbox;
use view::screens;
use view::shop::{draw_shop, shop_tap_at, ShopView};
use logic::shop::ShopEvent;
use view::text::TextPainter;

enum Scene {
    Title,
    /// `picked`: 골라진 캐릭터 번호 (CharacterKind::ALL 순서)
    Select { picked: usize },
    Play {
        game: Game,
        /// D12 빨리감기(F). 새 게임은 `config::DEV_FAST_FORWARD_AT_START`대로 시작한다 (D33).
        fast_forward: bool,
        /// 튜토리얼 중에 사용자가 F를 직접 눌렀으면 true. 그러면 튜토리얼 중에도 빨리감기를 따른다 (BUG-039).
        ff_by_hand: bool,
        /// 코인·공격·좀비 효과 (화면 전용)
        effects: Effects,
        /// 쓰러졌으면 그 화면 상태 (US-043)
        ended: Option<EndScreen>,
        /// 수학 문제 창이 열려 있던 동안 누르던 ←/→는 뗄 때까지 걷기가 되지 않는다 (US-076). 제작대 화면도 같다.
        walk_lock: bool,
        /// 캔 것 팝업과 제작대에서 고른 재료 (M7, 화면 전용)
        gathered: GatherView,
        /// 상점 화면의 "샀어요!" 같은 잠깐 효과 (M8, 화면 전용)
        shop_view: ShopView,
        /// 멈춤 창이 떠 있으면 true (부모님 요청: Esc로 멈추고 이어서 하기). 그동안 logic을 부르지 않아 시계·몬스터가 멈춘다.
        paused: bool,
        /// 멈춤 창에서 ↑/↓로 고른 칸 (주황색). 창을 열 때마다 "이어서 하기"로 돌아간다.
        pause_pick: PauseTap,
        /// 멈춤 창 옆에서 기다려 주는 친구 (아이 결정 2026-11-20). 창을 열 때마다 랜덤으로 다시 고른다.
        pause_buddy: PauseBuddy,
        /// 가방 (US-091): 열려 있는지, 장난감 상자 반짝임, 아래 키 안내 8초. 열린 동안은 멈춤 창처럼 logic을 부르지 않는다.
        bag: BagView,
        /// 물음표(?) 버튼과 조작 안내 한 장 (US-122): 버튼은 판을 시작하고 30초만 보인다. 열려 있는 동안은 멈춤 창처럼 logic을 부르지 않는다.
        /// 판이 새로 시작되면 새로 만든다 (`start_play`).
        help: Help,
    },
}

fn start_play(kind: CharacterKind, settings: &Settings, no_guide: bool, carried: Option<Tutorial>) -> Scene {
    // 판마다 좀비가 다르게 나오도록 지금 시각(밀리초)을 시드로 쓴다.
    let seed = (macroquad::miniquad::date::now() * 1000.0) as u64 as u32;
    let mut game = Game::new(kind, seed);
    match carried {
        // "다시하기"·"처음으로" 뒤: 이미 시작한 안내는 다시 나오지 않는다 (저장은 하지 않는다)
        Some(tutorial) => game.tutorial = tutorial,
        // 처음 한 번만: 새 판 맨 처음에 시작한다. 이미 본 사람과 ?qa(안내 없음)는 시작하지 않는다
        None => {
            game.start_tutorial(settings, no_guide);
        }
    }
    Scene::Play { game, fast_forward: config::DEV_FAST_FORWARD_AT_START, ff_by_hand: false, effects: Effects::new(), ended: None, walk_lock: false, gathered: GatherView::new(), shop_view: ShopView::new(), paused: false, pause_pick: PauseTap::Resume, pause_buddy: PauseBuddy::Miss, bag: BagView::new(), help: Help::new(no_guide) }
}

/// 창 한 줄 안내를 "봤다"로 적었으면 설정에 넣고 저장한다 (바뀔 때만 쓴다. ?qa에서는 save_settings가 안 쓴다).
fn keep_tips_seen(tips: &WindowTips, settings: &mut Settings) {
    let after = tips.settings_after(settings);
    if after != *settings {
        *settings = after;
        view::web::save_settings(settings);
    }
}

/// 이번 프레임에 빨리감기를 쓸지 (BUG-039, 부모님 결정 2026-11-21). 튜토리얼이 진행 중이면 시계를 보통 속도로 둔다.
/// 그때 사용자가 F를 직접 눌렀으면(`by_hand`) 존중한다. 끝나거나 건너뛰면, 시작 안 했으면 설정(D33)·F 상태를 그대로 따른다.
fn fast_forward_now(on: bool, by_hand: bool, tutorial: TutorialState) -> bool {
    on && (by_hand || !matches!(tutorial, TutorialState::Running(_)))
}

/// 튜토리얼이 끝났거나 건너뛰었으면(`Finished`/`Skipped`) 저장할 새 설정. 이미 같은 값이면 None이라 다시 쓰지 않는다.
fn settings_to_save(events: &[TutorialEvent], tutorial: &Tutorial, settings: &Settings) -> Option<Settings> {
    let over = events.iter().any(|e| matches!(e, TutorialEvent::Finished | TutorialEvent::Skipped));
    let after = tutorial.settings_after(settings);
    (over && after != *settings).then_some(after)
}

fn kind_index(kind: CharacterKind) -> usize {
    CharacterKind::ALL.iter().position(|k| *k == kind).unwrap_or(0)
}

/// 이번 프레임 시간. 다른 탭에 다녀와 아주 길어진 프레임은 MAX_FRAME_SECONDS로 자른다.
fn cap_frame_seconds(frame_time: f32) -> f32 {
    frame_time.clamp(0.0, config::MAX_FRAME_SECONDS)
}

/// 빨리감기 중 한 프레임: 걷기·점프·몬스터는 진짜 시간 `dt`, 시계와 코인은 `dt × time_scale` (D13).
/// 규칙(수학 문제가 열리면 멈추기 포함)은 `Game::update_with_time_scale`에 있다.
fn update_with_time_scale(game: &mut Game, input: &PlayerInput, dt: f32, time_scale: f32) -> Vec<GameEvent> {
    game.update_with_time_scale(input, dt, time_scale)
}

/// 버틴 날이 최고 기록보다 크면 기록을 올리고 저장한다 (US-044, D35: 쓰러질 때도, 나갈 때도).
/// 수학 창·제작대가 열린 동안 글·말풍선·팝업·연기 시간을 멈춘다. 창 뒤에 깔린 글이 창을 닫은 뒤 보이게 (BUG-027)
fn update_screen_effects(effects: &mut Effects, gathered: &mut GatherView, dt: f32, window_open: bool) {
    if !window_open {
        effects.update(dt);
        gathered.update(dt);
    }
}

/// 공격을 눌렀을 때 휘두르기(맨손·칼·도끼·총 빛)를 그릴지. 포션은 손에 들지 않으니 공격은 늘 휘두르기다.
fn shows_swing(attack_pressed: bool, playing: bool) -> bool {
    attack_pressed && playing
}

/// 이번 프레임에 쓰려는 물건들: PC의 1·2·R·3 키와 폰에서 누른 정보판 그림 (부모님 결정).
/// 한 물건을 쓰는 일(`use_pocket`)은 가방 칸과 같은 길이다 (view::bag).
fn pockets_to_use(keys: Vec<Pocket>, tapped: Option<Pocket>) -> Vec<Pocket> {
    let mut list = keys;
    if let Some(p) = tapped.filter(|p| !list.contains(p)) {
        list.push(p);
    }
    list
}

/// 가방을 열지 (US-091): I 키, PC 작은 장난감 상자를 마우스로 누름, 폰 장난감 상자 버튼 중 하나라도.
/// 잠깐 퀴즈·상점·제작대·멈춤 창·끝난 화면에서는 열리지 않는다 (main이 그때는 부르지 않는다).
fn bag_opens(key: bool, click: Option<Vec2>, touch_mode: bool, phone_button: bool) -> bool {
    key || phone_button || click.is_some_and(|p| bag_button_rect(touch_mode).contains(p))
}

/// 가방이 열린 동안 받은 입력 (US-091): I나 Esc, 가방 버튼, "닫기"면 닫고, 칸을 누르면 그 칸.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BagInput {
    Close,
    Tap(view::bag::BagTap),
}

fn bag_input(key: bool, esc: bool, tap: Option<Vec2>, touch_mode: bool) -> Option<BagInput> {
    if key || esc || tap.is_some_and(|p| bag_button_rect(touch_mode).contains(p)) {
        return Some(BagInput::Close);
    }
    tap.and_then(bag_tap_at).map(BagInput::Tap)
}

/// 폰 캐기 버튼 글: 손 닿는 것(캐기·들어가기·만들기 …)이 먼저, 없으면 해변 쿵 (US-081/082). 버튼을 안 보이면 None.
fn mine_button_label(game: &Game) -> Option<&'static str> {
    game.usable_in_reach().map(|u| u.button_text()).or_else(|| game.slam_power_here().map(|_| config::SLAM_BUTTON_TEXT))
}

/// 멈춤 창 친구의 그림: 캐릭터는 손그림 그대로, 미스는 miss.png.
fn pause_buddy_texture(assets: &Assets, buddy: PauseBuddy) -> &Texture2D {
    match buddy {
        PauseBuddy::Hero(kind) => assets.character(kind),
        PauseBuddy::Miss => &assets.m6.miss,
    }
}

/// 플레이 중 Esc·"멈춤" 버튼으로 멈춘 상태에서 받은 입력 (부모님 요청 2026-11-14, 키보드 고르기 2026-11-15).
/// Esc를 다시 누르거나 "이어서 하기"를 누르면 이어서, "처음으로"를 누르면 캐릭터 고르기로.
/// ↑/↓로 `picked`를 옮기고 Enter·Space로 고른다 (`view::pause::pause_choose`).
fn pause_choice(menu: &MenuInput, picked: &mut PauseTap) -> Option<PauseTap> {
    let keys = PauseKeys { up: menu.up, down: menu.down, confirm: menu.confirm, back: menu.back };
    pause_choose(picked, keys, menu.tap)
}

/// Esc(PC)·폰 "멈춤"·문제 창 "◀ 고르기"를 눌렀을 때 할 일 (멈춤 창이 떠 있을 때는 `pause_choice`가 받는다).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackAction {
    CloseCraft,
    CloseShop,
    Pause,
    Leave,
}

/// 제작대·상점은 그 창만 닫고, 게임 중과 수학 문제 창은 멈춤 창을 띄운다 (REVIEW-020 1번).
/// 캐릭터 고르기로 바로 나가는 것은 끝난 화면뿐이다.
fn back_action(back: bool, ended: bool, craft_open: bool, shop_open: bool) -> Option<BackAction> {
    if !back {
        None
    } else if ended {
        Some(BackAction::Leave)
    } else if craft_open {
        Some(BackAction::CloseCraft)
    } else if shop_open {
        Some(BackAction::CloseShop)
    } else {
        Some(BackAction::Pause)
    }
}

/// 상점·포션 일을 효과로 보여 준다 (US-070, US-071).
fn show_shop_event(effects: &mut Effects, event: ShopEvent) {
    match event {
        ShopEvent::Opened => {}
        ShopEvent::HealthPotionUsed { at, .. } => effects.on_health_potion(at),
        ShopEvent::PoisonSprayed { at, .. } => effects.on_poison_sprayed(at),
        ShopEvent::SkatesOff => effects.show_message(config::SKATES_OFF_TEXT),
        ShopEvent::ArmorOff => effects.show_message(config::ARMOR_OFF_TEXT),
    }
}

fn keep_record(days: u32, best_days: &mut u32) {
    if days > *best_days {
        *best_days = days;
        view::web::save_best_days(days);
    }
}

/// 점수가 최고 점수보다 크면 올리고 저장한다 (US-069: 쓰러질 때도, 클리어할 때도, 나갈 때도).
fn keep_score(score: u32, best_score: &mut u32) {
    if score > *best_score {
        *best_score = score;
        view::web::save_best_score(score);
    }
}

/// 판이 끝나는 순간(쓰러짐·클리어) **한 번만** 부른다 (US-133). 이미 끝 화면이면 아무것도 안 한다.
/// 순서: "새 기록!" 판정(`EndScreen::new`)을 먼저 하고, 그 뒤에 최근 판 기록에 더한다 (서로 영향 없음). 최고 기록·점수 저장은 그대로.
/// 기본값(부모님이 나중에 바꿀 수 있다): Esc로 도중에 나가는 판은 최근 판 기록에 더하지 않는다. 끝 화면에 들어간 판만 더한다.
fn enter_end_screen(ended: &mut Option<EndScreen>, game: &Game, best_days: &mut u32, best_score: &mut u32, settings: &mut Settings) {
    if ended.is_some() {
        return;
    }
    let mut screen = EndScreen::new(game, *best_days, *best_score);
    settings.recent_runs.add(screen.days, screen.score, screen.cleared);
    screen.set_recent(settings.recent_runs.latest_first());
    view::web::save_settings(settings); // 값이 바뀐 이때 한 번만 (?qa에서는 save_settings가 안 쓴다)
    keep_record(game.clock.nights_survived(), best_days);
    keep_score(game.score(), best_score);
    *ended = Some(screen);
}

fn window_conf() -> Conf {
    Conf {
        window_title: "미스 찾기".to_owned(),
        window_width: 960,
        window_height: 540,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // 흉내 마우스는 다른 손가락이 움직이면 탭 자리가 덮어써진다. 터치는 fingers()로만 받는다.
    simulate_mouse_with_touch(false);
    // 폰 터치 조작 (D16). 터치가 들어오면 터치 모드(조이스틱·버튼 보이기), 키보드를 누르면 다시 숨긴다.
    // 로고도 같은 것을 써서, 로고를 넘긴 손가락이 타이틀에서 새 누름이 되지 않는다 (US-109).
    let mut touch = TouchControls::new();
    // 시작 화면 (US-109): 글꼴만 먼저 받고, Now Loading 동안 나머지 그림을 받는다 → "눌러서 시작" → 니로 스튜디오 로고.
    // 게임을 열 때 한 번만 (장면 루프 밖). ?qa에서는 눌러서 시작과 로고를 건너뛴다.
    let (font, font_bytes) = Assets::load_font().await;
    let loading = macroquad::experimental::coroutines::start_coroutine(Assets::load_with_font(font.clone(), font_bytes));
    // 소리 (US-112): 파일은 "눌러서 시작"을 누른 뒤에 뒤에서 하나씩 받는다. 첫 화면(Now Loading) 동안은 소리 파일을 받지 않는다.
    let mut audio = Audio::new();
    let assets = view::logo::play_logo(&font, &mut touch, view::web::qa_mode(), || loading.retrieve(), || audio.start_loading()).await;
    audio.start_loading(); // ?qa처럼 누름 없이 넘어온 경우 (두 번째 부름은 아무 일 없다)
    let mut scene = Scene::Title;
    let mut first_frame = true;
    let mut touch_mode = false;
    // 웹 배포 버전 (타이틀 구석에 작게). 로컬 실행이면 None.
    let version = view::web::game_version();
    // 최고 기록 (US-044). 브라우저 저장소에서 읽고, 새 기록이 나오면 저장한다.
    let mut best_days = view::web::load_best_days();
    let mut best_score = view::web::load_best_score();
    // 설정 (US-125): 시작할 때 한 번 읽는다. 값을 바꾸는 쪽(튜토리얼·소리 스토리)이 바뀔 때만 view::web::save_settings(&settings)를 부른다.
    // 게임 규칙 상태(Game)가 아니라 여기(화면 쪽)에 둔다. 지금은 튜토리얼이 끝나거나 건너뛸 때만 바뀐다 (US-120).
    let mut settings = view::web::load_settings();
    // 소리 (US-112~117): 켜짐/크기는 저장된 설정에서, 나머지는 소리 규칙(logic::sound)이 정한다
    let mut sound = SoundSettings::from_settings(&settings);
    let mut mixer = SfxMixer::new();
    let mut music_plan = MusicPlan::new();
    let mut mining_sound = MiningSound::default();
    let mut window_sounds = WindowSounds::default();
    let mut toast = SoundToast::default();
    let mut prev_quiz_open = false;
    // ?qa에서는 튜토리얼 말풍선이 나오지 않는다. ?qa=tutorial이면 나온다 (US-123). 저장은 둘 다 읽지도 쓰지도 않는다.
    let no_guide = view::web::no_guide();
    // 창 한 줄 안내 (US-121): 판이 바뀌어도 이어 쓴다 (본 창은 다시하기를 해도 또 나오지 않는다). 본 창은 설정에 저장된다.
    let mut tips = WindowTips::from_settings(&settings);
    // 지난 프레임에 열려 있던 안내 창: 바뀌는 프레임이 "창이 열리고 닫힌 순간"이다
    let mut prev_tip_window: Option<TipWindow> = None;
    // 쓰러진 뒤 "다시하기"·"처음으로"를 하면 새 판에 이어 줄 튜토리얼 상태 (이미 시작했으면 다시 안 나온다)
    let mut carried_tutorial: Option<Tutorial> = None;
    // 건너뛰기 버튼을 누른 손가락: 뗄 때까지 조이스틱·버튼에 넘기지 않는다
    let mut skip_finger: Option<u64> = None;
    // 시험용 단축 (엘리용, 부모님 결정 2026-11-14). 주소에 ?qa가 있을 때만 켜진다.
    let mut qa = view::qa::Qa::new(view::web::qa_mode());

    loop {
        let letterbox = Letterbox::current();
        let fingers = fingers(&letterbox);
        touch_mode = next_touch_mode(touch_mode, !fingers.is_empty(), any_key_pressed());
        let (sw, sh) = (screen_width(), screen_height());
        // 세로 화면 안내 중에는 어떤 입력도 받지 않고 게임도 멈춘다 (아래).
        let rotate_hint = show_rotate_hint(touch_mode, sw, sh);
        let mut menu = if rotate_hint { MenuInput::default() } else { menu_input(&letterbox) };
        let dt = cap_frame_seconds(get_frame_time());
        // 한 줄 안내의 5초는 실제 시간이다: 창이 열려 게임이 멈춰도 센다
        tips.update(dt);
        // 플레이 중이 아니거나(쓰러진 화면 포함) 안내 중이면 조이스틱·버튼을 놓고, 지금 닿아 있는 손가락만 기억한다.
        // (메뉴에서 누른 손가락, 안내 전에 대고 있던 손가락이 플레이에서 저절로 걷기·점프가 되지 않게)
        // 수학 문제 창도 같다: 숫자판을 누르던 손가락이 창이 닫힌 뒤 조이스틱·버튼이 되지 않는다 (US-076).
        let quiz_open = matches!(&scene, Scene::Play { game, ended: None, .. } if game.math.is_open());
        // 제작대 화면도 같다: 칸을 누르던 손가락이 조이스틱·버튼이 되지 않고, 걷기·공격이 게임에 들어가지 않는다 (US-021).
        let craft_open = !quiz_open && matches!(&scene, Scene::Play { game, ended: None, .. } if game.crafting.is_open());
        // 상점 화면도 같다 (US-062): 물건을 누르던 손가락이 조이스틱·버튼이 되지 않고, 걷기·공격이 게임에 들어가지 않는다.
        let shop_open = !quiz_open && !craft_open && matches!(&scene, Scene::Play { game, ended: None, .. } if game.shop.is_open());
        let window_open = quiz_open || craft_open || shop_open;
        // 멈춤 창 (부모님 요청): 창처럼 손가락은 메뉴 탭으로만 받는다
        let paused_now = matches!(&scene, Scene::Play { paused: true, ended: None, .. });
        // QA 창도 멈춤처럼: 손가락은 메뉴 탭으로만, logic은 부르지 않는다
        qa.open &= matches!(&scene, Scene::Play { ended: None, .. });
        // 플레이 중 새로 댄 손가락 (QA 표 누르기용. 조이스틱·버튼과 같은 손가락이어도 QA 표 자리에는 버튼이 없다)
        let new_finger = fingers.iter().find(|f| f.phase == TouchPhase::Started).map(|f| f.pos);
        // 가방 창도 멈춤처럼 (US-091): 손가락은 메뉴 탭으로만, logic은 부르지 않는다
        let bag_open = matches!(&scene, Scene::Play { bag, ended: None, .. } if bag.open);
        // 물음표 조작 안내가 열려 있어도 멈춤처럼 (US-122): 손가락은 메뉴 탭으로만, logic은 부르지 않는다
        let help_open = matches!(&scene, Scene::Play { help, ended: None, .. } if help.is_open());
        let paused_now = paused_now || qa.open || bag_open || help_open;
        // 가방 키(I)는 장면과 상관없이 매 프레임 한 번만 읽는다
        let bag_key = bag_key_pressed();
        // 글자 줄은 장면과 상관없이 매 프레임 꺼낸다 (닫혀 있으면 버린다)
        let quiz_keys = math_keys_pressed(quiz_open && !rotate_hint);
        if rotate_hint || window_open || paused_now || !matches!(scene, Scene::Play { ended: None, .. }) {
            let tap = touch.idle(&fingers);
            if !rotate_hint {
                menu.tap = menu.tap.or(tap);
            }
        }

        // 소리 켜기/끄기: M 키(어디서나)와 멈춤 창의 소리 버튼 (US-112)
        if !rotate_hint && sound_key_pressed() {
            let on = sound.toggle();
            toast.show(on);
        }
        toast.update(dt);

        // 1) 입력 → 장면 바꾸기 / logic 업데이트. 장면이 바뀐 프레임에는 새 장면을 업데이트하지 않는다
        //    (같은 Space 한 번이 "결정"과 "점프"로 두 번 쓰이지 않게).
        //    세로 안내 중에는 아무것도 업데이트하지 않는다: 시계·코인·캐릭터가 그대로 멈춘다
        //    (dt = 0과 같다. 규칙을 바꾸는 게 아니라 logic을 부르지 않을 뿐).
        let next = if rotate_hint {
            None
        } else {
            match &mut scene {
                Scene::Title => {
                    if menu.confirm || menu.tap.is_some() {
                        mixer.request(Sfx::ButtonClick);
                        Some(Scene::Select { picked: 0 })
                    } else {
                        None
                    }
                }
                Scene::Select { picked } => {
                    if menu.left && *picked > 0 {
                        *picked -= 1;
                    }
                    if menu.right && *picked + 1 < CharacterKind::ALL.len() {
                        *picked += 1;
                    }
                    if let Some(i) = menu.tap.and_then(screens::select_card_at) {
                        mixer.request(Sfx::ButtonClick);
                        Some(start_play(CharacterKind::ALL[i], &settings, no_guide, carried_tutorial.take()))
                    } else if menu.confirm {
                        mixer.request(Sfx::ButtonClick);
                        Some(start_play(CharacterKind::ALL[*picked], &settings, no_guide, carried_tutorial.take()))
                    } else {
                        None
                    }
                }
                Scene::Play { game, fast_forward, ff_by_hand, effects, ended, walk_lock, gathered, shop_view, paused, pause_pick, pause_buddy, bag, help } => {
                    // 말풍선 자리를 다른 안내가 쓰는 동안은 "잘했어요!" 시간이 흐르지 않는다 (BUG-040). 매 프레임 logic 업데이트 전에 알려 준다.
                    game.tutorial.set_notice_up(view::tutorial::other_notice_up(effects, gathered));
                    // 물음표 버튼의 30초는 실제 시간이고, 창·멈춤으로 게임이 멈춘 동안은 세지 않는다 (US-122)
                    if ended.is_none() {
                        help.update(dt, window_open || *paused || bag.open || qa.open);
                    }
                    // 상점의 "샀어요!"는 진짜 시간으로 흐른다 (게임은 멈춰 있어도)
                    shop_view.update(dt);
                    // 가방 버튼 반짝임과 아래 키 안내 8초: 창(멈춤·가방·퀴즈·상점·제작대)이 열린 동안은 안내 시간이 줄지 않는다 (임시-76)
                    bag.update(dt, ended.is_none() && !window_open && !*paused && !bag.open && !qa.open);
                    if ended.is_none() {
                        bag.watch(view::bag::all_counts(game));
                    }
                    // 플레이 중 터치는 조이스틱·버튼으로만. 쓰러진 화면의 터치는 위 idle이 메뉴 탭으로 준다.
                    // 바꾸기 버튼은 무기가 있을 때만. 포션·주사기총·엔더열매는 정보판의 그림을 누르면 쓴다 (부모님 결정)
                    // 누르는 자리 목록 맨 뒤는 가방 버튼(장난감 상자)이다: `pocket_pressed == Some(물건 수)`면 가방을 연다
                    let buttons = play_buttons_with_mine(game.can_switch(), mine_button_label(game).is_some());
                    let pocket_spots = view::hud::pocket_tap_rects(game);
                    // 튜토리얼 건너뛰기 (US-120): N 키, 버튼 클릭, 버튼에 새로 댄 손가락. 누른 손가락은 뗄 때까지
                    // 조이스틱·버튼에 넘기지 않는다 (왼쪽 반을 누르면 조이스틱이 되어 걷는 것을 막는다).
                    let skip_on = ended.is_none() && !window_open && !*paused && !qa.open && !bag.open && !help.pauses_game() && view::tutorial::bubble_text(game, touch_mode, view::tutorial::other_notice_up(effects, gathered)).is_some();
                    let skip_rect = view::tutorial::skip_rect(touch_mode);
                    if skip_finger.is_some_and(|id| !fingers.iter().any(|f| f.id == id)) {
                        skip_finger = None;
                    }
                    let claimed = if skip_on { view::tutorial::claim_skip_finger(&fingers, skip_rect) } else { None };
                    if let Some(finger) = claimed {
                        skip_finger = Some(finger.id);
                    }
                    if skip_on && view::tutorial::skip_pressed(view::input::tutorial_skip_key_pressed(), menu.tap, claimed.map(|f| f.pos), skip_rect) {
                        game.skip_tutorial();
                        mixer.request(Sfx::ButtonClick);
                    }
                    // 물음표(?) 버튼 (US-122): 보이는 동안만 받는다. 누른 손가락도 건너뛰기처럼 뗄 때까지 조이스틱·버튼에 넘기지 않는다.
                    let help_on = ended.is_none() && !window_open && !*paused && !qa.open && !bag.open && !help.pauses_game() && help.button_visible();
                    let help_rect = view::help::button_rect(touch_mode);
                    let help_claimed = if help_on { view::tutorial::claim_skip_finger(&fingers, help_rect) } else { None };
                    if let Some(finger) = help_claimed {
                        skip_finger = Some(finger.id);
                    }
                    let help_pressed = help_on && view::help::pressed(menu.tap, help_claimed.map(|f| f.pos), help_rect);
                    let game_fingers = view::tutorial::without_finger(&fingers, skip_finger);
                    let touched = if ended.is_none() && !window_open && !*paused && !qa.open && !bag.open && !help.pauses_game() {
                        let mut spots: Vec<Rect> = pocket_spots.iter().map(|(_, r)| *r).collect();
                        spots.push(bag_button_rect(true));
                        touch.update_with_pockets(&game_fingers, &buttons, &spots, touch_mode)
                    } else {
                        TouchFrame::default()
                    };
                    let retry = ended.as_mut().is_some_and(|screen| {
                        screen.seconds += dt;
                        // 새 기록 소리 (US-132): 글이 나타나는 순간 한 번만. 다시 그려져도 또 나지 않는다.
                        mixer.request_opt(screen.take_celebration());
                        screen.ready() &&(menu.confirm || menu.tap.is_some_and(|p| RETRY_BUTTON.contains(p)))
                    });
                    // 문제 창에서는 터치 "◀ 고르기"가 위 idle의 탭으로 온다. 누르면 멈춤 창 (REVIEW-020 1번)
                    let quiz_back = quiz_open && touch_mode && menu.tap.is_some_and(|p| BACK_BUTTON.contains(p));
                    let back = back_action(menu.back || touched.back_pressed || quiz_back, ended.is_some(), craft_open, shop_open);
                    if qa.open {
                        // QA 창: 칸을 누르면 그 단축을 하고 닫는다. 밖을 누르거나 Esc를 누르면 그냥 닫는다
                        if let Some(action) = menu.tap.and_then(view::qa::qa_tap_at) {
                            view::qa::apply(game, action);
                        }
                        if menu.tap.is_some() || menu.back {
                            qa.open = false;
                            *walk_lock = true;
                        }
                        None
                    } else if *paused && ended.is_none() {
                        // 멈춤 창: logic을 부르지 않는다 (시계·몬스터·효과가 그대로 멈춘다)
                        // 소리 버튼 (US-112): 켜짐/꺼짐과 음악·효과음 크기 4칸. 멈춤 중에는 효과음이 막혀 있어서 눌림 소리는 바로 낸다.
                        if let Some(tap) = menu.tap.and_then(pause_sound_tap_at) {
                            match tap {
                                PauseSoundTap::Toggle => {
                                    let on = sound.toggle();
                                    toast.show(on);
                                }
                                PauseSoundTap::Music(i) => sound.set_music_percent(config::SOUND_VOLUME_STEPS[i] as i32),
                                PauseSoundTap::Effects(i) => sound.set_effects_percent(config::SOUND_VOLUME_STEPS[i] as i32),
                            }
                            // 꺼진 채로 크기 칸을 눌러도 소리가 켜지고 칸 색이 바뀐다
                            if tap.turns_sound_on() && !sound.is_on() {
                                sound.set_on(true);
                                toast.show(true);
                            }
                            audio.play_sfx(Sfx::ButtonClick, sound.effects_volume());
                        }
                        match pause_choice(&menu, pause_pick) {
                            Some(PauseTap::Resume) => {
                                mixer.request(Sfx::WindowClose);
                                *paused = false;
                                // 멈춘 동안 누르던 ←/→는 뗄 때까지 걷기가 되지 않는다
                                *walk_lock = true;
                                None
                            }
                            Some(PauseTap::Quit) => {
                                audio.play_sfx(Sfx::ButtonClick, sound.effects_volume());
                                keep_record(game.clock.nights_survived(), &mut best_days);
                                keep_score(game.score(), &mut best_score);
                                carried_tutorial = Some(game.tutorial.for_next_game());
                                Some(Scene::Select { picked: kind_index(game.player.kind) })
                            }
                            None => None,
                        }
                    } else if bag.open && ended.is_none() {
                        // 가방 창 (US-091): logic을 부르지 않는다 (시계·몬스터·효과가 그대로 멈춘다). Esc는 가방만 닫는다.
                        // 칸을 누르면 쓰거나 손에 들고 바로 닫힌다. 쓸 수 없으면 창 그대로 (view::bag).
                        let closed = match bag_input(bag_key, menu.back, menu.tap, touch_mode) {
                            Some(BagInput::Close) => {
                                bag.close();
                                true
                            }
                            Some(BagInput::Tap(tap)) => bag.apply(game, tap, touch_mode),
                            None => false,
                        };
                        if closed {
                            // 가방이 열린 동안 누르던 ←/→는 뗄 때까지 걷기가 되지 않는다
                            *walk_lock = true;
                        }
                        None
                    } else if help.is_open() && ended.is_none() {
                        // 조작 안내 한 장 (US-122): logic을 부르지 않는다 (시계·몬스터가 그대로 멈춘다). Esc·Enter·Space나 "알았어요!"로 닫으면 하던 판으로 돌아온다.
                        if menu.back || menu.confirm || view::help::pressed(menu.tap, None, view::help::close_rect()) {
                            help.close();
                            // 안내가 열린 동안 누르던 ←/→는 뗄 때까지 걷기가 되지 않는다
                            *walk_lock = true;
                        }
                        None
                    } else if back == Some(BackAction::CloseCraft) {
                        // 제작대 화면의 Esc는 고르기로 나가지 않고 제작대만 닫는다 (US-021)
                        game.crafting.close();
                        None
                    } else if back == Some(BackAction::CloseShop) {
                        // 상점 화면의 Esc도 상점만 닫는다 (US-062). 문 앞에 그대로 서 있다.
                        shop_view.apply(game, view::shop::ShopTap::Close);
                        None
                    } else if back == Some(BackAction::Pause) {
                        // 게임 중·문제 창의 Esc나 폰 "멈춤"·"◀ 고르기": 바로 나가지 않고 멈춤 창을 띄운다 (부모님 요청 2026-11-14)
                        // 문제 창은 그대로 열려 있고, 이어서 하면 그 창으로 돌아온다
                        *paused = true;
                        audio.play_sfx(Sfx::WindowOpen, sound.effects_volume());
                        *pause_pick = PauseTap::Resume;
                        *pause_buddy = PauseBuddy::pick((macroquad::miniquad::date::now() * 1000.0) as u64 as u32);
                        game.mine_held = false;
                        None
                    } else if back == Some(BackAction::Leave) || retry {
                        mixer.request(Sfx::ButtonClick);
                        keep_record(game.clock.nights_survived(), &mut best_days);
                        keep_score(game.score(), &mut best_score);
                        carried_tutorial = Some(game.tutorial.for_next_game());
                        Some(Scene::Select { picked: kind_index(game.player.kind) })
                    } else if help_pressed {
                        // 물음표 버튼: 이번 프레임부터 게임이 멈춘다
                        help.press_button();
                        game.mine_held = false;
                        None
                    } else if ended.is_none() && !window_open && bag_opens(bag_key, menu.tap, touch_mode, touched.pocket_pressed == Some(pocket_spots.len())) {
                        // 가방 열기 (I, 장난감 상자 버튼). 이번 프레임부터 게임이 멈춘다
                        bag.open_bag();
                        game.mine_held = false;
                        None
                    } else {
                        // 문제 창이 열린 동안: 숫자 키·숫자판(수학) 또는 1~4 키·보기 칸(영어)만 받고,
                        // 걷기·점프·공격·바꾸기·빨리감기는 게임에 넣지 않는다.
                        if quiz_open {
                            if let Some(result) = apply_quiz_input(&mut game.math, &quiz_keys, menu.tap) {
                                mixer.request(sfx_for_math_result(result));
                            }
                        } else if craft_open {
                            // "…을 만들었어요!"는 제작대 화면이 바로 보여 준다 (craft::draw_craft)
                            if let Some(tap) = menu.tap.and_then(craft_tap_at) {
                                apply_craft_tap(game, &mut gathered.craft, tap);
                            }
                        } else if shop_open {
                            // 사기·팔기·닫기: 마우스 클릭과 손가락 탭이 같은 길 (view::shop)
                            if let Some(tap) = menu.tap.and_then(shop_tap_at) {
                                shop_view.apply(game, tap);
                                mixer.request_opt(shop_view.take_sfx());
                            }
                        } else {
                            // 시험용 단축: Shift+숫자 키 1~9·0, 또는 "QA" 표를 누르면 QA 창 (?qa일 때만)
                            if ended.is_none() {
                                if let Some(action) = qa.key_action() {
                                    view::qa::apply(game, action);
                                }
                                if qa.badge_tapped(menu.tap.or(new_finger)) {
                                    qa.open = true;
                                    game.mine_held = false;
                                }
                            }
                            if fast_forward_pressed() {
                                *fast_forward = !*fast_forward;
                                *ff_by_hand = true;
                            }
                            if ended.is_none() && (switch_weapon_pressed() || touched.switch_pressed) {
                                game.switch_weapon();
                            }
                            // 1 = 체력 포션, 2 = 독약, R = 주사기총, 3 = 엔더열매 (부모님 결정 2026-11-15). 폰은 정보판 그림 누르기. 쓸 수 있는지는 logic이 정한다
                            if ended.is_none() {
                                let tapped = touched.pocket_pressed.and_then(|i| pocket_spots.get(i)).map(|(p, _)| *p);
                                for pocket in pockets_to_use(pocket_keys_pressed(), tapped) {
                                    // 쓸 수 없으면 까닭을 짧은 알림으로 (부모님 결정 2026-11-15)
                                    if let Some(why) = use_pocket_or_why(game, pocket) {
                                        effects.show_message(why);
                                    }
                                }
                            }
                        }
                        // 집 문·동굴 입구·출구에 닿아 있으면 ↑는 점프가 아니라 들어가기다 (부모님 결정 2026-11-15)
                        let up_enter = up_enters(game.usable_in_reach());
                        let (keys, lock) = hold_walk_until_released(*walk_lock || window_open, merge_touch(player_input(up_enter), &touched));
                        *walk_lock = lock;
                        let input = if window_open { PlayerInput::default() } else { keys };
                        // 캐기·들어가기·만들기: E 키나 캐기 버튼을 누르고 있는 동안 (임시-16, 임시-17)
                        // 창이 열린 동안에도 실제 누름을 넘긴다: 멈춘 동안은 logic의 forget_mining이 막고, 닫은 뒤 꾹 누른 E를 새 누름으로 보지 않는다 (REVIEW-017 1번)
                        game.mine_held = mine_key_down(up_enter) || touch.is_held(ButtonKind::Mine) || touched.mine_pressed;
                        if shows_swing(input.attack_pressed, ended.is_none()) {
                            effects.start_swing(game.weapons.held());
                            mixer.request(sfx_for_swing(game.weapons.held()));
                        }
                        let visible_before: Vec<bool> = game.coins.list.iter().map(|c| c.is_visible()).collect();
                        let events = if fast_forward_now(*fast_forward, *ff_by_hand, game.tutorial.state()) {
                            update_with_time_scale(game, &input, dt, config::DEV_FAST_FORWARD_SCALE)
                        } else {
                            game.update(&input, dt)
                        };
                        // 2) 이벤트 처리: 효과와 쓰러진 화면. 낮/밤이 바뀌는 건 하늘 색과 정보판이 보여 준다.
                        //    (아침 인사 문구·소리는 아이와 정한 뒤에 — US-012, M7 사운드)
                        // 상자 일은 먼저 받아 둔다: 상자에서 나온 몬스터는 "얍얍 공격하라" 없이 머리 위 "공격!"만 (부모님 결정 2026-11-14)
                        let box_events = game.take_box_events();
                        let box_spots = view::boxes::box_monster_spots(&box_events);
                        for event in events {
                            mixer.request_opt(sfx_for_game_event(&event));
                            match event {
                                GameEvent::CoinsPicked { count } => {
                                    effects.pops.spawn_for_picks(&visible_before, &game.coins, count, game.player.rect().center());
                                }
                                GameEvent::Combat(CombatEvent::PlayerDown) | GameEvent::GameCleared => {
                                    enter_end_screen(ended, game, &mut best_days, &mut best_score, &mut settings);
                                }
                                GameEvent::Combat(combat) => effects.on_combat_near_boxes(combat, &box_spots),
                                GameEvent::NightStarted | GameEvent::DayStarted { .. } => {}
                                GameEvent::MissFound { at } => effects.on_miss_found(at),
                                GameEvent::WeaponPicked { kind } => effects.show_message(kind.picked_text()),
                                GameEvent::CitizenPopped { at } => effects.on_citizen_popped(at),
                            }
                        }
                        for event in game.take_gather_events() {
                            mixer.request_opt(sfx_for_gather_event(&event));
                            mixer.request_opt(bonus_sfx_for_gather_event(&event));
                            match event {
                                GatherEvent::Mined { what, .. } => gathered.on_mined(what, !game.gathering.bag.has_pickaxe && game.world.workbench().is_some()),
                                GatherEvent::PushedOutOfCave => effects.show_message(config::CAVE_NIGHT_OUT_TEXT),
                                GatherEvent::CaveClosed => effects.show_message(config::CAVE_CLOSED_TEXT),
                                GatherEvent::CraftingOpened => gathered.craft = view::craft::CraftView::opened(game.crafting.chosen()),
                                GatherEvent::CaveEntered | GatherEvent::CaveLeft => {}
                            }
                        }
                        for event in game.take_shop_events() {
                            mixer.request_opt(sfx_for_shop_event(&event));
                            show_shop_event(effects, event);
                        }
                        // 자동차·주사기총·엔더열매·쿵 (US-080~083)
                        for event in game.take_item_events() {
                            mixer.request_opt(sfx_for_item_event(&event));
                            effects.on_item_event(event);
                        }
                        // 무기 강화 (US-130): +n 글자·별·반짝이·큰 알림 글·소리. 주운 문구보다 나중에 받아서 강화 글이 보인다
                        for event in game.take_weapon_events() {
                            mixer.request_opt(sfx_for_weapon_event(&event));
                            effects.on_weapon_event(event);
                        }
                        // 연습용 몬스터 (US-124): 나타남·맞음·쓰러짐·사라짐 효과. 소리는 소리 스토리(US-112~)가 오면 여기서 낸다
                        for event in game.take_practice_events() {
                            mixer.request_opt(sfx_for_practice_event(&event));
                            effects.on_practice_event(event);
                        }
                        // 튜토리얼 (US-119·120): 끝까지 봤거나 건너뛰면 "봤다"를 저장한다 (바뀔 때만, ?qa에서는 save_settings가 안 쓴다)
                        let tutorial_events = game.take_tutorial_events();
                        for event in &tutorial_events {
                            mixer.request_opt(sfx_for_tutorial_event(event));
                        }
                        if let Some(after) = settings_to_save(&tutorial_events, &game.tutorial, &settings) {
                            settings = after;
                            view::web::save_settings(&settings);
                        }
                        // 깜짝 미션 (US-086): 미스 말풍선, 성공·실패 글. 클리어한 아침의 실패 글은 띄우지 않는다
                        for event in game.take_mission_events() {
                            mixer.request_opt(sfx_for_mission_event(&event));
                            // 깜짝 미션이 처음 나올 때 한 줄 안내 (US-121). 말풍선이 떠 있거나 ?qa면 규칙이 막는다
                            if matches!(event, MissionEvent::Appeared { .. }) && tips.window_opened(TipWindow::SurpriseMission, no_guide, game.tutorial.current_step().is_some()).is_some() {
                                keep_tips_seen(&tips, &mut settings);
                            }
                            effects.mission.on_event(event, game.is_cleared());
                        }
                        // 상자 부수기 (US-089): 금·나뭇조각·도끼 안내·"와우!"는 효과가, "○○을 얻었어요!"는 큰 문구로
                        for event in &box_events {
                            mixer.request_opt(sfx_for_box_event(event));
                        }
                        for event in box_events {
                            if let Some(message) = effects.boxes.on_event(event) {
                                effects.show_message(message);
                            }
                        }
                        effects.watch_car(screens::car_is_moving(game.gathering.bag.riding_car, game.player.vel.x) && ended.is_none(), game.player.rect().center());
                        effects.watch_citizens(game.citizens.walkers.iter().map(|w| w.is_walking()));
                        // 소리: 퀴즈 창이 열릴 때 "띵동", 캐는 동안 "톡톡" (US-112~117)
                        let quiz_now = game.math.is_open();
                        if quiz_now && !prev_quiz_open {
                            mixer.request(Sfx::QuizStart);
                        }
                        prev_quiz_open = quiz_now;
                        let mining = game.mining_progress().map(|_| if game.in_cave { Material::Iron } else { Material::Wood });
                        mixer.request_opt(mining_sound.update(dt, mining));
                        // 이번 프레임에 상점이 열렸으면 그때부터 멈춘다
                        update_screen_effects(effects, gathered, dt, window_open || game.shop.is_open());
                        None
                    }
                }
            }
        };
        if let Some(next) = next {
            scene = next;
        }
        // 창 한 줄 안내 (US-121): 안내 창이 열리거나 닫힌 순간. 가방은 열린 프레임에, 퀴즈·제작대·상점은 logic이 연 프레임에 잡힌다.
        let now_tip_window = match &scene {
            Scene::Play { game, bag, ended: None, .. } => view::window_tip::open_window(game, bag.open),
            _ => None,
        };
        if now_tip_window != prev_tip_window {
            prev_tip_window = now_tip_window;
            tips.window_closed();
            if let (Some(window), Scene::Play { game, .. }) = (now_tip_window, &scene) {
                if tips.window_opened(window, no_guide, game.tutorial.current_step().is_some()).is_some() {
                    keep_tips_seen(&tips, &mut settings);
                }
            }
        }

        // 소리 (US-112~117): 창이 열리고 닫힐 때, 설정 저장, 음악, 효과음. 멈춤 창이 떠 있으면 효과음은 막히고 음악은 작아진다.
        let window_flags = match &scene {
            Scene::Play { game, bag, help, ended: None, .. } => {
                WindowFlags { bag: bag.open, help: help.is_open(), craft: game.crafting.is_open(), shop: game.shop.is_open() }
            }
            _ => WindowFlags::default(),
        };
        for sfx in window_sounds.update(window_flags) {
            mixer.request(sfx);
        }
        let esc_paused = matches!(&scene, Scene::Play { paused: true, ended: None, .. });
        // 바뀔 때만 저장한다 (?qa에서는 save_settings가 안 쓴다)
        let with_sound = sound.settings_after(&settings);
        if with_sound != settings {
            settings = with_sound;
            view::web::save_settings(&settings);
        }
        let situation = match &scene {
            Scene::Title => MusicSituation::menu(MusicScene::Title),
            Scene::Select { .. } => MusicSituation::menu(MusicScene::CharacterSelect),
            Scene::Play { game, .. } => MusicSituation::from_game(game),
        };
        match music_plan.update(choose_track(&situation)) {
            Some(MusicChange::Play(track)) => audio.set_music(Some(track)),
            Some(MusicChange::Stop) => audio.set_music(None),
            None => {}
        }
        audio.update_music(dt, sound.music_volume(esc_paused));
        for sfx in mixer.take_playable(dt, &sound, esc_paused) {
            audio.play_sfx(sfx, sound.effects_volume());
        }

        // 3) 그리기 (960×540 가상 화면)
        letterbox.begin();
        let text = TextPainter::new(&assets.font, letterbox.scale);
        match &scene {
            Scene::Title => screens::draw_title(&assets, &text, config::GROUND_Y, version.as_deref()),
            Scene::Select { picked } => screens::draw_select(&assets, &text, config::GROUND_Y, *picked),
            Scene::Play { game, fast_forward, ff_by_hand, effects, ended, gathered, shop_view, paused, pause_pick, pause_buddy, bag, help, .. } => {
                screens::draw_play(&assets, &text, game, fast_forward_now(*fast_forward, *ff_by_hand, game.tutorial.state()), effects, ended.as_ref(), touch_mode, gathered, bag, view::window_tip::bottom(tips.current()));
                if ended.is_none() && game.math.is_open() {
                    draw_math(&text, &game.math, &assets.m6.miss, &assets.english);
                    if *paused {
                        // 멈춤 창이 문제 창 위에 뜬다. 이어서 하면 문제 창이 그대로 보인다
                        draw_pause(&text, touch_mode, *pause_pick, (pause_buddy_texture(&assets, *pause_buddy), get_time() as f32), (sound.is_on(), sound.music_percent(), sound.effects_percent()));
                    } else if touch_mode {
                        draw_back_button(&text);
                    }
                } else if ended.is_none() && game.crafting.is_open() {
                    draw_craft(&text, game, &gathered.craft, &assets.m6);
                } else if ended.is_none() && game.shop.is_open() {
                    draw_shop(&text, game, shop_view, touch_mode);
                } else if ended.is_none() && *paused {
                    draw_pause(&text, touch_mode, *pause_pick, (pause_buddy_texture(&assets, *pause_buddy), get_time() as f32), (sound.is_on(), sound.music_percent(), sound.effects_percent()));
                } else if ended.is_none() && bag.open {
                    let show_recipe = game.world.workbench().is_some();
                    draw_bag_window(&text, game, &assets.m6, bag, &effects.upgrade, touch_mode, show_recipe);
                } else if touch_mode && ended.is_none() && !qa.open && !help.is_open() {
                    let label = mine_button_label(game);
                    let buttons = play_buttons_with_mine(game.can_switch(), label.is_some());
                    draw_touch_controls(&text, &touch, &buttons, label.unwrap_or(""));
                }
                if ended.is_none() {
                    let a_window_is_up = game.math.is_open() || game.crafting.is_open() || game.shop.is_open() || *paused || bag.open || qa.open;
                    // 물음표(?) 버튼 (US-122): 창이 없을 때, 처음 30초만. 열려 있으면 조작 안내 한 장이 화면을 덮는다
                    if help.is_open() {
                        view::help::draw_sheet(&text, Help::lines(touch_mode));
                    } else if !a_window_is_up && help.button_visible() {
                        view::help::draw_button(&text, touch_mode);
                    }
                    // 창 한 줄 안내 (US-121): 창 위 빈 곳에 5초. 멈춤 창이 뜨면 가린다
                    if let Some(window) = tips.current().filter(|_| !*paused && !help.is_open() && !qa.open) {
                        view::window_tip::draw(&text, window, touch_mode);
                    }
                }
                if ended.is_none() {
                    // 창(문제·제작대·상점·멈춤·가방)이 열려 있으면 QA 표를 숨긴다: 그때는 눌러도 안 열리고 창의 칸을 가린다 (BUG-031)
                    let window_or_pause = game.math.is_open() || game.crafting.is_open() || game.shop.is_open() || *paused || bag.open;
                    if !window_or_pause {
                        view::qa::draw_badge(&text, &qa);
                    }
                    view::qa::draw_menu(&text, &qa);
                }
            }
        }
        // 멈춤 창이 열려 있으면 글은 안 띄운다 (창 안의 켜짐/꺼짐 버튼이 바뀐다)
        if !matches!(&scene, Scene::Play { paused: true, ended: None, .. }) {
            draw_toast(&text, &toast);
        }
        letterbox.end();
        if rotate_hint {
            // 창 전체(띠 포함)를 덮는다. 글자 크기는 창(CSS) 픽셀 기준이라 dpi 붓을 쓴다.
            draw_rotate_hint(&TextPainter::new(&assets.font, screen_dpi_scale()), sw, sh);
        }
        if first_frame {
            // 첫 그림을 그렸으니 웹페이지의 "Now Loading..." 화면을 치운다.
            view::web::hide_loading_screen();
            first_frame = false;
        }

        next_frame().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use config::{COIN_RESPAWN_SECONDS, DAY_SECONDS, DEV_FAST_FORWARD_SCALE, PLAYER_WALK_SPEED};
    use logic::game::ItemEvent;
    use logic::shop::Potion;
    use logic::world::World;
    use view::bag::use_pocket;

    const FRAME: f32 = 1.0 / 60.0;
    const SEED: u32 = 7;

    /// 상자 자리가 고정된 맵의 새 게임. 여기 규칙들은 랜덤 상자와 상관없다 (D36은 d36_random_boxes.rs).
    fn m1_game(kind: CharacterKind) -> Game {
        Game::on_world(kind, SEED, World::m1())
    }

    /// 상자 위로 올라가 코인을 줍고, 걷고, 점프하는 입력 묶음 (몇 초 동안).
    fn script(frame: usize) -> PlayerInput {
        PlayerInput { left: frame % 400 > 300, right: frame % 400 < 250, jump_pressed: frame % 90 == 0, attack_pressed: frame % 40 == 0 }
    }

    #[test]
    fn message_behind_a_window_waits_until_the_window_closes() {
        let mut effects = Effects::new();
        let mut gathered = GatherView::new();
        effects.show_message(config::CAVE_NIGHT_OUT_TEXT);
        for _ in 0..600 {
            update_screen_effects(&mut effects, &mut gathered, FRAME, true);
        }
        assert_eq!(effects.message(), Some(config::CAVE_NIGHT_OUT_TEXT), "창이 열린 10초 동안 글이 그대로");
        for _ in 0..60 {
            update_screen_effects(&mut effects, &mut gathered, FRAME, false);
        }
        assert!(effects.message().is_some(), "창을 닫은 뒤 1초에도 보인다");
        for _ in 0..180 {
            update_screen_effects(&mut effects, &mut gathered, FRAME, false);
        }
        assert!(effects.message().is_none(), "그 뒤에는 사라진다");
    }

    #[test]
    fn time_scale_one_matches_game_update() {
        let mut a = m1_game(CharacterKind::Niro);
        let mut b = m1_game(CharacterKind::Niro);
        for f in 0..3000 {
            let input = script(f);
            let ea = a.update(&input, FRAME);
            let eb = update_with_time_scale(&mut b, &input, FRAME, 1.0);
            assert_eq!(ea, eb, "frame {f}");
            assert_eq!(a.player.pos, b.player.pos);
            assert_eq!(a.wallet.coins(), b.wallet.coins());
            assert_eq!(a.clock.seconds_left_in_phase(), b.clock.seconds_left_in_phase());
            let va: Vec<bool> = a.coins.list.iter().map(|c| c.is_visible()).collect();
            let vb: Vec<bool> = b.coins.list.iter().map(|c| c.is_visible()).collect();
            assert_eq!(va, vb);
        }
        assert!(a.wallet.coins() > 0, "스크립트가 코인을 하나는 주워야 의미가 있다");
    }

    #[test]
    fn fast_forward_speeds_clock_but_not_walking() {
        let mut normal = m1_game(CharacterKind::Ellie);
        let mut fast = m1_game(CharacterKind::Ellie);
        let right = PlayerInput { right: true, ..Default::default() };
        for _ in 0..60 {
            normal.update(&right, FRAME);
            update_with_time_scale(&mut fast, &right, FRAME, DEV_FAST_FORWARD_SCALE);
        }
        // 걷기는 같다 (1초 동안 같은 거리)
        assert_eq!(normal.player.pos, fast.player.pos);
        assert!(normal.player.pos.x > 120.0 + PLAYER_WALK_SPEED * 0.5);
        // 시계는 10배 (1초 vs 10초)
        let used_normal = DAY_SECONDS - normal.clock.seconds_left_in_phase();
        let used_fast = DAY_SECONDS - fast.clock.seconds_left_in_phase();
        assert!((used_normal - 1.0).abs() < 0.01, "{used_normal}");
        assert!((used_fast - 10.0).abs() < 0.05, "{used_fast}");
    }

    #[test]
    fn fast_forward_respawns_coin_in_about_one_second() {
        let mut g = m1_game(CharacterKind::Logan);
        let coin = g.coins.list[0].rect;
        // 코인 자리에 플레이어를 잠깐 놓아 줍게 한 뒤 멀리 치운다.
        g.player.pos = coin.point();
        update_with_time_scale(&mut g, &PlayerInput::default(), 0.0, DEV_FAST_FORWARD_SCALE);
        assert_eq!(g.wallet.coins(), 1);
        g.player.pos.x = 0.0;
        g.player.pos.y = g.world.ground_y - g.player.size.y;
        let frames = (COIN_RESPAWN_SECONDS / DEV_FAST_FORWARD_SCALE / FRAME) as usize;
        for _ in 0..frames - 3 {
            update_with_time_scale(&mut g, &PlayerInput::default(), FRAME, DEV_FAST_FORWARD_SCALE);
        }
        assert!(!g.coins.list[0].is_visible());
        for _ in 0..6 {
            update_with_time_scale(&mut g, &PlayerInput::default(), FRAME, DEV_FAST_FORWARD_SCALE);
        }
        assert!(g.coins.list[0].is_visible());
    }

    /// 밤이 막 시작된 게임. 왼쪽 상자 왼쪽에 서 있어서 왼쪽 끝 좀비가 걸어올 수 있다.
    fn night_game() -> Game {
        let mut g = m1_game(CharacterKind::Niro);
        g.combat.skeleton_chance = 0.0;
        g.clock.update(DAY_SECONDS);
        g.player.pos.x = 230.0;
        g
    }

    #[test]
    fn time_scale_one_matches_game_update_at_night() {
        let (mut a, mut b) = (night_game(), night_game());
        let mut saw_combat = false;
        for f in 0..3000 {
            let input = script(f);
            let ea = a.update(&input, FRAME);
            let eb = update_with_time_scale(&mut b, &input, FRAME, 1.0);
            assert_eq!(ea, eb, "frame {f}");
            saw_combat |= ea.iter().any(|e| matches!(e, GameEvent::Combat(_)));
            assert_eq!(a.combat.health.hp(), b.combat.health.hp());
            let xa: Vec<f32> = a.combat.zombies.list.iter().map(|z| z.pos.x).collect();
            let xb: Vec<f32> = b.combat.zombies.list.iter().map(|z| z.pos.x).collect();
            assert_eq!(xa, xb);
        }
        assert!(saw_combat, "좀비가 나와야 의미가 있다");
    }

    #[test]
    fn fast_forward_does_not_speed_up_skeletons_or_shots() {
        let night_with_skeletons = || {
            let mut g = night_game();
            g.combat.skeleton_chance = 1.0;
            g
        };
        let (mut normal, mut fast) = (night_with_skeletons(), night_with_skeletons());
        let none = PlayerInput::default();
        for _ in 0..(config::ARROW_INTERVAL_SECONDS / FRAME) as usize + 30 {
            let events = normal.update(&none, FRAME);
            let events_fast = update_with_time_scale(&mut fast, &none, FRAME, DEV_FAST_FORWARD_SCALE);
            let combat_only = |list: Vec<GameEvent>| list.into_iter().filter(|e| matches!(e, GameEvent::Combat(_))).collect::<Vec<_>>();
            assert_eq!(combat_only(events), combat_only(events_fast));
        }
        assert_eq!(normal.combat.skeletons.list[0].pos, fast.combat.skeletons.list[0].pos);
        assert_eq!(normal.combat.skeletons.projectiles.len(), 2);
        assert_eq!(normal.combat.skeletons.projectiles[0].pos, fast.combat.skeletons.projectiles[0].pos);
    }

    #[test]
    fn fast_forward_does_not_speed_up_zombies_d13() {
        let (mut normal, mut fast) = (night_game(), night_game());
        let none = PlayerInput::default();
        for _ in 0..120 {
            normal.update(&none, FRAME);
            update_with_time_scale(&mut fast, &none, FRAME, DEV_FAST_FORWARD_SCALE);
        }
        assert_eq!(normal.combat.zombies.list[0].pos, fast.combat.zombies.list[0].pos);
        assert!(fast.clock.seconds_left_in_phase() < normal.clock.seconds_left_in_phase() - 10.0);
    }

    #[test]
    fn fast_forward_also_stops_after_going_down() {
        let mut g = night_game();
        let mut frames = 0;
        while !g.combat.health.is_down() {
            let zombie = logic::monster::Zombie::new(g.player.pos.x, g.world.ground_y, false);
            g.combat.zombies.list = vec![zombie];
            update_with_time_scale(&mut g, &PlayerInput::default(), FRAME, DEV_FAST_FORWARD_SCALE);
            frames += 1;
            assert!(frames < 36_000, "10분 안에 쓰러져야 한다");
        }
        let left = g.clock.seconds_left_in_phase();
        let right = PlayerInput { right: true, ..Default::default() };
        let x = g.player.pos.x;
        assert!(update_with_time_scale(&mut g, &right, FRAME, DEV_FAST_FORWARD_SCALE).is_empty());
        assert_eq!((g.clock.seconds_left_in_phase(), g.player.pos.x), (left, x));
    }

    #[test]
    fn fast_forward_also_stops_after_clearing() {
        let mut g = m1_game(CharacterKind::Niro);
        for _ in 0..config::DAYS_TO_CLEAR {
            g.clock.update(DAY_SECONDS);
            g.clock.update(config::NIGHT_SECONDS);
        }
        let events = update_with_time_scale(&mut g, &PlayerInput::default(), FRAME, DEV_FAST_FORWARD_SCALE);
        assert!(events.contains(&GameEvent::GameCleared));
        assert!(g.is_cleared());
        let (left, x) = (g.clock.seconds_left_in_phase(), g.player.pos.x);
        let right = PlayerInput { right: true, ..Default::default() };
        assert!(update_with_time_scale(&mut g, &right, FRAME, DEV_FAST_FORWARD_SCALE).is_empty());
        assert_eq!((g.clock.seconds_left_in_phase(), g.player.pos.x), (left, x));
    }

    #[test]
    fn attack_swings_only_while_playing() {
        assert!(shows_swing(true, true));
        assert!(!shows_swing(true, false), "끝난 화면에서는 안 보인다");
        assert!(!shows_swing(false, true));
    }

    #[test]
    fn pocket_keys_and_taps_use_each_thing_once() {
        let health = Pocket::Potion(Potion::Health);
        assert_eq!(pockets_to_use(vec![], None), vec![]);
        assert_eq!(pockets_to_use(vec![health], None), vec![health]);
        assert_eq!(pockets_to_use(vec![], Some(Pocket::EnderFruit)), vec![Pocket::EnderFruit]);
        assert_eq!(pockets_to_use(vec![health], Some(health)), vec![health], "키와 탭이 같은 프레임이어도 하나만");
        // R로 먹으면 체력이 오르고 포션이 하나 준다 (먹는 규칙은 logic)
        let mut g = m1_game(CharacterKind::Niro);
        g.wallet.add(100);
        g.buy(logic::shop::ShopItem::HealthPotion).unwrap();
        g.combat.health.lose_from_quiz(1);
        for p in pockets_to_use(vec![health], None) {
            assert!(use_pocket(&mut g, p));
        }
        assert_eq!(g.shop.potions(Potion::Health), 0);
        assert!(matches!(g.take_shop_events()[..], [ShopEvent::HealthPotionUsed { .. }]));
    }

    #[test]
    fn y_fires_the_syringe_gun_and_g_eats_an_ender_fruit() {
        let mut g = night_game();
        g.gathering.bag.syringe_guns = 1;
        g.gathering.bag.ender_fruits = 1;
        g.combat.zombies.list = vec![logic::monster::Zombie::new(g.player.pos.x + 120.0, g.world.ground_y, false)];
        assert!(use_pocket(&mut g, Pocket::SyringeGun), "화면에 몬스터가 있으면 쏜다");
        assert_eq!(g.gathering.bag.syringe_guns, 0);
        assert!(use_pocket(&mut g, Pocket::EnderFruit));
        assert!(g.gathering.bag.has_ender_power && g.gathering.bag.ender_fruits == 0);
        let events = g.take_item_events();
        assert!(matches!(events[..], [ItemEvent::SyringeGunFired { .. }, ItemEvent::EnderFruitEaten { .. }]), "{events:?}");
        // 화면에서: 침 총알과 금빛 글
        let mut effects = Effects::new();
        for e in events {
            effects.on_item_event(e);
        }
        assert_eq!(effects.message(), Some(config::ENDER_FRUIT_EATEN_TEXT));
        assert!(!use_pocket(&mut g, Pocket::SyringeGun), "주사기총이 없으면 아무 일 없다");
    }

    #[test]
    fn esc_pauses_and_pause_window_resumes_or_quits() {
        use view::pause::{QUIT_BUTTON, RESUME_BUTTON};
        let mut pick = PauseTap::Resume;
        let esc = MenuInput { back: true, ..Default::default() };
        let tap = |p: Vec2| MenuInput { tap: Some(p), ..Default::default() };
        assert_eq!(pause_choice(&esc, &mut pick), Some(PauseTap::Resume), "Esc를 다시 누르면 이어서");
        assert_eq!(pause_choice(&tap(RESUME_BUTTON.center()), &mut pick), Some(PauseTap::Resume));
        assert_eq!(pause_choice(&tap(QUIT_BUTTON.center()), &mut pick), Some(PauseTap::Quit));
        assert_eq!(pause_choice(&tap(vec2(5.0, 5.0)), &mut pick), None, "빈 곳을 눌러도 그대로 멈춰 있다");
        assert_eq!(pause_choice(&MenuInput::default(), &mut pick), None);
        // 키보드 (부모님 요청 2026-11-15): ↓ 다음 Enter면 처음으로
        assert_eq!(pause_choice(&MenuInput { down: true, ..Default::default() }, &mut pick), None);
        assert_eq!(pause_choice(&MenuInput { confirm: true, ..Default::default() }, &mut pick), Some(PauseTap::Quit));
        // 폰 "멈춤" 버튼은 예전 "◀ 고르기" 자리: 누르면 back_pressed가 와서 멈춤 창이 뜬다
        let mut t = TouchControls::new();
        let p = BACK_BUTTON.center();
        let o = t.update(&[view::touch::Finger { id: 1, phase: TouchPhase::Started, pos: p }], &view::touch::PLAY_BUTTONS, true);
        assert!(o.back_pressed);
    }

    #[test]
    fn esc_in_the_quiz_pauses_instead_of_leaving() {
        // REVIEW-020 1번: 문제 창에서 Esc(또는 폰 "◀ 고르기")는 고르기로 나가지 않고 멈춤 창
        assert_eq!(back_action(true, false, false, false), Some(BackAction::Pause), "게임 중·문제 창");
        assert_eq!(back_action(true, false, true, false), Some(BackAction::CloseCraft));
        assert_eq!(back_action(true, false, false, true), Some(BackAction::CloseShop));
        assert_eq!(back_action(true, true, false, false), Some(BackAction::Leave), "끝난 화면만 바로 나간다");
        assert_eq!(back_action(false, false, false, false), None);
        let esc = MenuInput { back: true, ..Default::default() };
        assert_eq!(pause_choice(&esc, &mut PauseTap::Quit), Some(PauseTap::Resume), "멈춤 창의 Esc는 이어서 하기 (문제 창으로 돌아온다)");
    }

    #[test]
    fn mine_button_turns_into_slam_on_the_beach() {
        let mut g = Game::new(CharacterKind::Niro, SEED);
        assert_eq!(mine_button_label(&g), g.usable_in_reach().map(|u| u.button_text()));
        g.gathering.bag.has_ender_power = true;
        let beach = (config::SLAM_BEACH_LEFT + config::SLAM_BEACH_RIGHT) / 2.0;
        g.player.pos.x = beach - g.player.size.x / 2.0;
        if g.usable_in_reach().is_none() {
            assert_eq!(mine_button_label(&g), Some(config::SLAM_BUTTON_TEXT), "쿵 할 수 있으면 캐기 버튼이 쿵!");
            assert_eq!(play_buttons_with_mine(g.can_switch(), mine_button_label(&g).is_some()).len(), 2);
        }
        g.gathering.bag.has_ender_power = false;
        assert_eq!(mine_button_label(&g), g.usable_in_reach().map(|u| u.button_text()));
    }

    #[test]
    fn shop_events_become_effects() {
        let mut effects = Effects::new();
        show_shop_event(&mut effects, ShopEvent::ArmorOff);
        assert_eq!(effects.message(), Some(config::ARMOR_OFF_TEXT));
        show_shop_event(&mut effects, ShopEvent::SkatesOff);
        assert_eq!(effects.message(), Some(config::SKATES_OFF_TEXT));
        show_shop_event(&mut effects, ShopEvent::PoisonSprayed { at: vec2(10.0, 10.0), monsters: 2 });
        assert_eq!(effects.poison_clouds(), 1);
        show_shop_event(&mut effects, ShopEvent::HealthPotionUsed { at: vec2(10.0, 10.0), healed: 5 });
        assert!(effects.heart_sparkle() > 0.0);
    }

    #[test]
    fn switch_button_does_not_show_with_only_a_potion() {
        let mut g = m1_game(CharacterKind::Niro);
        assert!(!g.can_switch());
        g.wallet.add(100);
        g.buy(logic::shop::ShopItem::HealthPotion).unwrap();
        assert!(g.weapons.owned().is_empty() && !g.can_switch(), "포션만 있으면 바꾸기 버튼이 없다 (포션은 그림을 눌러 먹는다)");
        assert_eq!(play_buttons_with_mine(g.can_switch(), false).len(), 1);
        assert_eq!(view::hud::pocket_tap_rects(&g).len(), 1, "대신 포션 그림을 누를 수 있다");
    }

    #[test]
    fn score_keeps_only_a_better_best() {
        let mut best = 9;
        keep_score(9, &mut best);
        assert_eq!(best, 9);
        keep_score(10, &mut best);
        assert_eq!(best, 10);
    }

    #[test]
    fn leaving_keeps_only_a_better_record() {
        let mut best = 3;
        keep_record(2, &mut best);
        assert_eq!(best, 3);
        keep_record(4, &mut best);
        assert_eq!(best, 4);
    }

    fn run_with(coins: u32) -> Game {
        let mut game = m1_game(CharacterKind::Niro);
        game.coins.picked_total = coins;
        game
    }

    #[test]
    fn a_finished_run_is_added_to_recent_runs_once_even_if_the_event_repeats() {
        let (mut ended, mut best_days, mut best_score, mut settings) = (None, 0, 0, Settings::default());
        let game = run_with(5);
        enter_end_screen(&mut ended, &game, &mut best_days, &mut best_score, &mut settings);
        enter_end_screen(&mut ended, &game, &mut best_days, &mut best_score, &mut settings); // 같은 끝 화면에서 또 와도
        assert_eq!(settings.recent_runs.len(), 1);
        let screen = ended.expect("끝 화면");
        assert_eq!(screen.recent.len(), 1);
        assert_eq!((screen.recent[0].score, screen.recent[0].days), (screen.score, screen.days));
        assert_eq!(best_score, screen.score, "최고 점수 저장은 그대로");
    }

    #[test]
    fn leaving_with_esc_adds_nothing_because_only_end_screens_add() {
        // Esc로 나가는 길은 enter_end_screen을 부르지 않는다 (기본값, 부모님이 바꿀 수 있다). 기록 목록은 그대로.
        let settings = Settings::default();
        let mut best = 0;
        keep_record(3, &mut best);
        assert!(settings.recent_runs.is_empty());
    }

    #[test]
    fn same_record_goes_in_the_list_but_is_not_a_new_record_and_sixth_run_drops_the_first() {
        let (mut best_days, mut best_score, mut settings) = (0, 0, Settings::default());
        let mut ended = None;
        enter_end_screen(&mut ended, &run_with(5), &mut best_days, &mut best_score, &mut settings);
        assert!(ended.as_ref().unwrap().new_best_score, "처음 한 판은 새 기록");
        ended = None;
        enter_end_screen(&mut ended, &run_with(5), &mut best_days, &mut best_score, &mut settings);
        let same = ended.take().unwrap();
        assert!(!same.new_record && !same.new_best_score, "같은 기록은 새 기록이 아니다");
        assert!(same.fireworks.is_empty());
        assert_eq!(same.recent.len(), 2, "그래도 목록에는 들어간다");
        for coins in 6..=9 {
            enter_end_screen(&mut ended, &run_with(coins), &mut best_days, &mut best_score, &mut settings);
            ended = None;
        }
        assert_eq!(settings.recent_runs.len(), 5);
        let scores: Vec<u32> = settings.recent_runs.oldest_first().iter().map(|r| r.score).collect();
        let first = run_with(5).score();
        assert_eq!(scores.iter().filter(|s| **s == first).count(), 1, "첫 판이 빠지고 두 번째 같은 점수만 남는다: {scores:?}");
    }

    #[test]
    fn fast_forward_is_off_while_tutorial_runs_unless_pressed_by_hand() {
        use logic::tutorial::TutorialStep;
        let running = TutorialState::Running(TutorialStep::Move);
        assert!(!fast_forward_now(true, false, running), "튜토리얼 중 시작 상태는 보통 속도");
        assert!(fast_forward_now(true, true, running), "F를 직접 눌렀으면 존중");
        assert!(!fast_forward_now(false, true, running), "꺼져 있으면 그대로 꺼짐");
        assert!(fast_forward_now(true, false, TutorialState::Finished));
        assert!(fast_forward_now(true, false, TutorialState::Skipped));
        // 안 시작한 경우(이미 봄, ?qa)는 아무것도 안 바뀐다
        let game = Game::new(CharacterKind::ALL[0], 1);
        assert!(fast_forward_now(true, false, game.tutorial.state()));
        assert!(!fast_forward_now(false, false, game.tutorial.state()));
    }

    #[test]
    fn tutorial_is_saved_once_only_when_it_ends_or_is_skipped() {
        let fresh = Settings::default();
        let mut game = Game::new(CharacterKind::Niro, 1);
        assert!(game.start_tutorial(&fresh, false));
        let started = game.take_tutorial_events();
        assert_eq!(settings_to_save(&started, &game.tutorial, &fresh), None, "시작만으로는 저장하지 않는다");
        assert!(game.skip_tutorial());
        let skipped = game.take_tutorial_events();
        let saved = settings_to_save(&skipped, &game.tutorial, &fresh).expect("건너뛰면 저장");
        assert!(saved.tutorial_done);
        assert_eq!(settings_to_save(&skipped, &game.tutorial, &saved), None, "이미 저장한 값이면 다시 쓰지 않는다");
    }

    #[test]
    fn first_game_starts_the_tutorial_but_seen_and_qa_do_not() {
        let fresh = Settings::default();
        let Scene::Play { game, .. } = start_play(CharacterKind::Ellie, &fresh, false, None) else { panic!() };
        assert!(game.tutorial.current_step().is_some(), "처음 하는 판은 말풍선 ①");
        let Scene::Play { game, .. } = start_play(CharacterKind::Ellie, &fresh, true, None) else { panic!() };
        assert!(game.tutorial.current_step().is_none(), "?qa는 안내 없음");
        let seen = Settings { tutorial_done: true, ..fresh };
        let Scene::Play { game, .. } = start_play(CharacterKind::Ellie, &seen, false, None) else { panic!() };
        assert!(game.tutorial.current_step().is_none(), "이미 본 사람은 안 나온다");
    }

    #[test]
    fn retry_carries_the_tutorial_so_it_does_not_show_again() {
        let fresh = Settings::default();
        let Scene::Play { game, .. } = start_play(CharacterKind::Niro, &fresh, false, None) else { panic!() };
        // 쓰러져서 다시하기: 저장은 안 했어도(settings 그대로) 새 판에 말풍선이 다시 나오지 않는다
        let carried = game.tutorial.for_next_game();
        let Scene::Play { game: next, .. } = start_play(CharacterKind::Niro, &fresh, false, Some(carried)) else { panic!() };
        assert!(next.tutorial.current_step().is_none());
    }

    #[test]
    fn long_frame_is_capped() {
        // 탭에 30초 다녀와도 한 프레임은 MAX_FRAME_SECONDS까지만. 빨리감기여도 낮/밤을 건너뛰지 않는다.
        let dt = cap_frame_seconds(30.0);
        assert_eq!(dt, config::MAX_FRAME_SECONDS);
        assert!(dt * DEV_FAST_FORWARD_SCALE < config::DUSK_SECONDS);
        let mut g = m1_game(CharacterKind::Niro);
        g.combat.skeleton_chance = 0.0;
        let events = update_with_time_scale(&mut g, &PlayerInput::default(), dt, DEV_FAST_FORWARD_SCALE);
        assert!(events.is_empty());
        assert_eq!(g.clock.day_number(), 1);
        assert!(!g.clock.is_night());
    }

    // ---- REVIEW-022 1번: ↑로 들어가기는 새로 누를 때만 ----

    /// 집 문 앞(또는 `x`)에 서게 한다.
    fn stand_at(g: &mut Game, center_x: f32) {
        g.player.pos = Vec2::new(center_x - config::PLAYER_WIDTH / 2.0, config::GROUND_Y - config::PLAYER_HEIGHT);
        g.player.vel = Vec2::ZERO;
        g.update(&PlayerInput::default(), 0.0);
    }

    /// main 루프와 같은 순서로 한 프레임: ↑가 지금 눌려 있는지(`up_down`)와 지난 프레임 상태로 새 누름을 정한다.
    fn frame_with_up(g: &mut Game, up_down: bool, up_was_down: &mut bool) {
        let up_pressed = up_down && !*up_was_down;
        *up_was_down = up_down;
        let enter = up_enters(g.usable_in_reach());
        g.mine_held = view::input::keys_to_mine(false, up_pressed, enter);
        g.update(&PlayerInput::default(), FRAME);
    }

    #[test]
    fn holding_up_into_the_door_does_not_enter() {
        let mut g = Game::new(CharacterKind::Niro, SEED);
        let mut up = false;
        // 문에서 먼 곳에서 ↑를 누르고 (점프 자리), 누른 채로 문에 닿는다.
        stand_at(&mut g, config::HOUSE_DOOR_X - 600.0);
        assert_ne!(g.usable_in_reach(), Some(logic::game::Usable::HouseDoor));
        frame_with_up(&mut g, true, &mut up);
        stand_at(&mut g, config::HOUSE_DOOR_X);
        assert_eq!(g.usable_in_reach(), Some(logic::game::Usable::HouseDoor));
        for _ in 0..60 {
            frame_with_up(&mut g, true, &mut up);
        }
        assert!(!g.shop.is_open(), "↑를 누르고 있기만 하면 들어가지 않는다");
        // 떼었다 다시 누르면 들어간다.
        frame_with_up(&mut g, false, &mut up);
        frame_with_up(&mut g, true, &mut up);
        assert!(g.shop.is_open());
    }

    #[test]
    fn closing_the_shop_with_up_held_does_not_reopen_it() {
        let mut g = Game::new(CharacterKind::Niro, SEED);
        let mut up = false;
        stand_at(&mut g, config::HOUSE_DOOR_X);
        frame_with_up(&mut g, true, &mut up);
        assert!(g.shop.is_open(), "↑로 상점에 들어간다");
        for _ in 0..10 {
            frame_with_up(&mut g, true, &mut up);
        }
        g.close_shop(); // ↑를 누른 채 Esc
        for _ in 0..60 {
            frame_with_up(&mut g, true, &mut up);
        }
        assert!(!g.shop.is_open(), "↑를 누른 채 닫아도 다시 열리지 않는다");
    }

    // ---- REVIEW-022 3번: 가방 입력 ----

    #[test]
    fn esc_or_i_closes_the_bag() {
        assert_eq!(bag_input(false, true, None, false), Some(BagInput::Close));
        assert_eq!(bag_input(true, false, None, false), Some(BagInput::Close));
        assert_eq!(bag_input(false, false, None, false), None);
    }

    #[test]
    fn bag_button_or_close_closes_the_bag() {
        for touch_mode in [false, true] {
            assert_eq!(bag_input(false, false, Some(bag_button_rect(touch_mode).center()), touch_mode), Some(BagInput::Close));
        }
        // "닫기" 버튼: 가방이 닫는다 (PC는 BagTap::Close, 폰은 가방 버튼과 겹쳐 바로 Close)
        assert_eq!(bag_input(false, false, Some(view::bag::CLOSE.center()), false), Some(BagInput::Tap(view::bag::BagTap::Close)));
        assert!(matches!(bag_input(false, false, Some(view::bag::CLOSE.center()), true), Some(BagInput::Close | BagInput::Tap(view::bag::BagTap::Close))));
    }

    #[test]
    fn tapping_a_slot_picks_that_slot() {
        use view::bag::{slot_rect, BagSlot, BagTap};
        for touch_mode in [false, true] {
            assert_eq!(
                bag_input(false, false, Some(slot_rect(0, 0).center()), touch_mode),
                Some(BagInput::Tap(BagTap::Slot(BagSlot::Weapon(logic::weapon::Weapon::Sword))))
            );
            assert_eq!(
                bag_input(false, false, Some(slot_rect(1, 3).center()), touch_mode),
                Some(BagInput::Tap(BagTap::Slot(BagSlot::Pocket(Pocket::EnderFruit))))
            );
        }
        // 빈 곳은 아무 일 없다
        assert_eq!(bag_input(false, false, Some(Vec2::new(2.0, 530.0)), false), None);
    }

    #[test]
    fn bag_opens_by_key_button_or_phone_box() {
        assert!(bag_opens(true, None, false, false));
        assert!(bag_opens(false, None, true, true));
        for touch_mode in [false, true] {
            assert!(bag_opens(false, Some(bag_button_rect(touch_mode).center()), touch_mode, false));
        }
        assert!(!bag_opens(false, None, false, false));
        assert!(!bag_opens(false, Some(Vec2::new(480.0, 270.0)), false, false), "화면 가운데를 누르면 열리지 않는다");
    }
}
