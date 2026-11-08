//! 미스 찾기 — 게임 루프: 입력 → logic 업데이트 → 이벤트 처리 → 그리기.
//! 화면 흐름: 타이틀 → 캐릭터 고르기 → 플레이 (Esc를 누르면 다시 고르기, 다시 고르면 새 게임).
//! 플레이에서 쓰러지거나 클리어하면 같은 장면 위에 끝난 화면이 나오고, "다시 하기"로 고르기에 간다.

mod config;
mod logic;
mod view;

use logic::character::CharacterKind;
use logic::combat::CombatEvent;
use logic::game::{Game, GameEvent, GatherEvent};
use logic::player::PlayerInput;
use macroquad::prelude::*;
use view::assets::Assets;
use view::craft::{apply_craft_tap, craft_tap_at, draw_craft};
use view::effects::Effects;
use view::gameover::{EndScreen, RETRY_BUTTON};
use view::input::{
    any_key_pressed, fast_forward_pressed, fingers, hold_walk_until_released, math_keys_pressed, menu_input, merge_touch, mine_key_down, player_input,
    switch_weapon_pressed, MenuInput,
};
use view::gather::GatherView;
use view::math::{draw_math, pad_key_at};
use view::touch::{
    draw_back_button, draw_rotate_hint, draw_touch_controls, next_touch_mode, play_buttons_with_mine, show_rotate_hint, ButtonKind, TouchControls,
    TouchFrame, BACK_BUTTON,
};
use view::screen::Letterbox;
use view::screens;
use view::text::TextPainter;

enum Scene {
    Title,
    /// `picked`: 골라진 캐릭터 번호 (CharacterKind::ALL 순서)
    Select { picked: usize },
    Play {
        game: Game,
        /// D12 빨리감기(F). 새 게임은 `config::DEV_FAST_FORWARD_AT_START`대로 시작한다 (D33).
        fast_forward: bool,
        /// 코인·공격·좀비 효과 (화면 전용)
        effects: Effects,
        /// 쓰러졌으면 그 화면 상태 (US-043)
        ended: Option<EndScreen>,
        /// 수학 문제 창이 열려 있던 동안 누르던 ←/→는 뗄 때까지 걷기가 되지 않는다 (US-076). 제작대 화면도 같다.
        walk_lock: bool,
        /// 캔 것 팝업과 제작대에서 고른 재료 (M7, 화면 전용)
        gathered: GatherView,
    },
}

fn start_play(kind: CharacterKind) -> Scene {
    // 판마다 좀비가 다르게 나오도록 지금 시각(밀리초)을 시드로 쓴다.
    let seed = (macroquad::miniquad::date::now() * 1000.0) as u64 as u32;
    Scene::Play { game: Game::new(kind, seed), fast_forward: config::DEV_FAST_FORWARD_AT_START, effects: Effects::new(), ended: None, walk_lock: false, gathered: GatherView::new() }
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
    let assets = Assets::load().await;
    let mut scene = Scene::Title;
    let mut first_frame = true;
    // 폰 터치 조작 (D16). 터치가 들어오면 터치 모드(조이스틱·버튼 보이기), 키보드를 누르면 다시 숨긴다.
    let mut touch = TouchControls::new();
    let mut touch_mode = false;
    // 웹 배포 버전 (타이틀 구석에 작게). 로컬 실행이면 None.
    let version = view::web::game_version();
    // 최고 기록 (US-044). 브라우저 저장소에서 읽고, 새 기록이 나오면 저장한다.
    let mut best_days = view::web::load_best_days();
    let mut best_score = view::web::load_best_score();

    loop {
        let letterbox = Letterbox::current();
        let fingers = fingers(&letterbox);
        touch_mode = next_touch_mode(touch_mode, !fingers.is_empty(), any_key_pressed());
        let (sw, sh) = (screen_width(), screen_height());
        // 세로 화면 안내 중에는 어떤 입력도 받지 않고 게임도 멈춘다 (아래).
        let rotate_hint = show_rotate_hint(touch_mode, sw, sh);
        let mut menu = if rotate_hint { MenuInput::default() } else { menu_input(&letterbox) };
        let dt = cap_frame_seconds(get_frame_time());
        // 플레이 중이 아니거나(쓰러진 화면 포함) 안내 중이면 조이스틱·버튼을 놓고, 지금 닿아 있는 손가락만 기억한다.
        // (메뉴에서 누른 손가락, 안내 전에 대고 있던 손가락이 플레이에서 저절로 걷기·점프가 되지 않게)
        // 수학 문제 창도 같다: 숫자판을 누르던 손가락이 창이 닫힌 뒤 조이스틱·버튼이 되지 않는다 (US-076).
        let quiz_open = matches!(&scene, Scene::Play { game, ended: None, .. } if game.math.is_open());
        // 제작대 화면도 같다: 칸을 누르던 손가락이 조이스틱·버튼이 되지 않고, 걷기·공격이 게임에 들어가지 않는다 (US-021).
        let craft_open = !quiz_open && matches!(&scene, Scene::Play { game, ended: None, .. } if game.crafting.is_open());
        // 글자 줄은 장면과 상관없이 매 프레임 꺼낸다 (닫혀 있으면 버린다)
        let quiz_keys = math_keys_pressed(quiz_open && !rotate_hint);
        if rotate_hint || quiz_open || craft_open || !matches!(scene, Scene::Play { ended: None, .. }) {
            let tap = touch.idle(&fingers);
            if !rotate_hint {
                menu.tap = menu.tap.or(tap);
            }
        }

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
                        Some(start_play(CharacterKind::ALL[i]))
                    } else if menu.confirm {
                        Some(start_play(CharacterKind::ALL[*picked]))
                    } else {
                        None
                    }
                }
                Scene::Play { game, fast_forward, effects, ended, walk_lock, gathered } => {
                    // 플레이 중 터치는 조이스틱·버튼으로만. 쓰러진 화면의 터치는 위 idle이 메뉴 탭으로 준다.
                    let buttons = play_buttons_with_mine(!game.weapons.owned().is_empty(), game.usable_in_reach().is_some());
                    let touched = if ended.is_none() && !quiz_open && !craft_open { touch.update(&fingers, &buttons, touch_mode) } else { TouchFrame::default() };
                    let retry = ended.as_mut().is_some_and(|screen| {
                        screen.seconds += dt;
                        screen.ready() && (menu.confirm || menu.tap.is_some_and(|p| RETRY_BUTTON.contains(p)))
                    });
                    // 문제 창에서는 터치 "◀ 고르기"가 위 idle의 탭으로 온다 (D35: 창이 열려도 나가기)
                    let quiz_back = quiz_open && touch_mode && menu.tap.is_some_and(|p| BACK_BUTTON.contains(p));
                    if craft_open && menu.back {
                        // 제작대 화면의 Esc는 고르기로 나가지 않고 제작대만 닫는다 (US-021)
                        game.crafting.close();
                        None
                    } else if menu.back || touched.back_pressed || retry || quiz_back {
                        keep_record(game.clock.nights_survived(), &mut best_days);
                        keep_score(game.score(), &mut best_score);
                        Some(Scene::Select { picked: kind_index(game.player.kind) })
                    } else {
                        // 문제 창이 열린 동안: 숫자 키·숫자판만 받고, 걷기·점프·공격·바꾸기·빨리감기는 게임에 넣지 않는다.
                        if quiz_open {
                            let tapped = menu.tap.and_then(pad_key_at);
                            for key in quiz_keys.iter().copied().chain(tapped) {
                                key.apply(&mut game.math);
                            }
                        } else if craft_open {
                            if let Some(tap) = menu.tap.and_then(craft_tap_at) {
                                apply_craft_tap(game, &mut gathered.craft_pick, tap);
                            }
                        } else {
                            if fast_forward_pressed() {
                                *fast_forward = !*fast_forward;
                            }
                            if ended.is_none() && (switch_weapon_pressed() || touched.switch_pressed) {
                                game.switch_weapon();
                            }
                        }
                        let (keys, lock) = hold_walk_until_released(*walk_lock || quiz_open || craft_open, merge_touch(player_input(), &touched));
                        *walk_lock = lock;
                        let input = if quiz_open || craft_open { PlayerInput::default() } else { keys };
                        // 캐기·들어가기·만들기: E 키나 캐기 버튼을 누르고 있는 동안 (임시-16, 임시-17)
                        // 창이 열린 동안에도 실제 누름을 넘긴다: 멈춘 동안은 logic의 forget_mining이 막고, 닫은 뒤 꾹 누른 E를 새 누름으로 보지 않는다 (REVIEW-017 1번)
                        game.mine_held = mine_key_down() || touch.is_held(ButtonKind::Mine) || touched.mine_pressed;
                        if input.attack_pressed && ended.is_none() {
                            effects.start_swing(game.weapons.held());
                        }
                        let visible_before: Vec<bool> = game.coins.list.iter().map(|c| c.is_visible()).collect();
                        let events = if *fast_forward {
                            update_with_time_scale(game, &input, dt, config::DEV_FAST_FORWARD_SCALE)
                        } else {
                            game.update(&input, dt)
                        };
                        // 2) 이벤트 처리: 효과와 쓰러진 화면. 낮/밤이 바뀌는 건 하늘 색과 정보판이 보여 준다.
                        //    (아침 인사 문구·소리는 아이와 정한 뒤에 — US-012, M7 사운드)
                        for event in events {
                            match event {
                                GameEvent::CoinsPicked { count } => {
                                    effects.pops.spawn_for_picks(&visible_before, &game.coins, count, game.player.rect().center());
                                }
                                GameEvent::Combat(CombatEvent::PlayerDown) | GameEvent::GameCleared => {
                                    *ended = Some(EndScreen::new(game, best_days, best_score));
                                    keep_record(game.clock.nights_survived(), &mut best_days);
                                    keep_score(game.score(), &mut best_score);
                                }
                                GameEvent::Combat(combat) => effects.on_combat(combat),
                                GameEvent::NightStarted | GameEvent::DayStarted { .. } => {}
                                GameEvent::MissFound { at } => effects.on_miss_found(at),
                                GameEvent::WeaponPicked { kind } => effects.show_message(kind.picked_text()),
                                GameEvent::CitizenPopped { at } => effects.on_citizen_popped(at),
                            }
                        }
                        for event in game.take_gather_events() {
                            match event {
                                GatherEvent::Mined { what, .. } => gathered.on_mined(what),
                                GatherEvent::PushedOutOfCave => effects.show_message(config::CAVE_NIGHT_OUT_TEXT),
                                GatherEvent::CaveClosed => effects.show_message(config::CAVE_CLOSED_TEXT),
                                GatherEvent::CraftingOpened => gathered.craft_pick = logic::gathering::Material::Wood,
                                GatherEvent::CaveEntered | GatherEvent::CaveLeft => {}
                            }
                        }
                        effects.watch_citizens(game.citizens.walkers.iter().map(|w| w.is_walking()));
                        update_screen_effects(effects, gathered, dt, quiz_open || craft_open);
                        None
                    }
                }
            }
        };
        if let Some(next) = next {
            scene = next;
        }

        // 3) 그리기 (960×540 가상 화면)
        letterbox.begin();
        let text = TextPainter::new(&assets.font, letterbox.scale);
        match &scene {
            Scene::Title => screens::draw_title(&assets, &text, config::GROUND_Y, version.as_deref()),
            Scene::Select { picked } => screens::draw_select(&assets, &text, config::GROUND_Y, *picked),
            Scene::Play { game, fast_forward, effects, ended, gathered, .. } => {
                screens::draw_play(&assets, &text, game, *fast_forward, effects, ended.as_ref(), touch_mode, gathered);
                if ended.is_none() && game.math.is_open() {
                    draw_math(&text, &game.math, &assets.m6.miss);
                    if touch_mode {
                        draw_back_button(&text);
                    }
                } else if ended.is_none() && game.crafting.is_open() {
                    draw_craft(&text, game, gathered.craft_pick);
                } else if touch_mode && ended.is_none() {
                    let usable = game.usable_in_reach();
                    let buttons = play_buttons_with_mine(!game.weapons.owned().is_empty(), usable.is_some());
                    draw_touch_controls(&text, &touch, &buttons, usable.map_or("", |u| u.button_text()));
                }
            }
        }
        letterbox.end();
        if rotate_hint {
            // 창 전체(띠 포함)를 덮는다. 글자 크기는 창(CSS) 픽셀 기준이라 dpi 붓을 쓴다.
            draw_rotate_hint(&TextPainter::new(&assets.font, screen_dpi_scale()), sw, sh);
        }
        if first_frame {
            // 첫 그림을 그렸으니 웹페이지의 "불러오는 중..." 글자를 치운다.
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
    use logic::world::World;

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
}
