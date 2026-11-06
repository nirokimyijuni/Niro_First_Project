//! 미스 찾기 — 게임 루프: 입력 → logic 업데이트 → 이벤트 처리 → 그리기.
//! 화면 흐름: 타이틀 → 캐릭터 고르기 → 플레이 (Esc를 누르면 다시 고르기, 다시 고르면 새 게임).
//! 플레이에서 쓰러지면 같은 장면 위에 쓰러진 화면이 나오고, "다시 하기"로 고르기에 간다.

mod config;
mod logic;
mod view;

use logic::character::CharacterKind;
use logic::clock::ClockEvent;
use logic::combat::CombatEvent;
use logic::game::{Game, GameEvent};
use logic::player::PlayerInput;
use macroquad::prelude::*;
use view::assets::Assets;
use view::effects::Effects;
use view::gameover::{DownScreen, RETRY_BUTTON};
use view::input::{any_key_pressed, fast_forward_pressed, fingers, menu_input, merge_touch, player_input, MenuInput};
use view::touch::{draw_rotate_hint, draw_touch_controls, next_touch_mode, show_rotate_hint, TouchControls, TouchFrame, PLAY_BUTTONS};
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
        down: Option<DownScreen>,
    },
}

fn start_play(kind: CharacterKind) -> Scene {
    // 판마다 좀비가 다르게 나오도록 지금 시각(밀리초)을 시드로 쓴다.
    let seed = (macroquad::miniquad::date::now() * 1000.0) as u64 as u32;
    Scene::Play { game: Game::new(kind, seed), fast_forward: config::DEV_FAST_FORWARD_AT_START, effects: Effects::new(), down: None }
}

fn kind_index(kind: CharacterKind) -> usize {
    CharacterKind::ALL.iter().position(|k| *k == kind).unwrap_or(0)
}

/// 이번 프레임 시간. 다른 탭에 다녀와 아주 길어진 프레임은 MAX_FRAME_SECONDS로 자른다.
fn cap_frame_seconds(frame_time: f32) -> f32 {
    frame_time.clamp(0.0, config::MAX_FRAME_SECONDS)
}

/// 빨리감기 중 한 프레임: `Game::update`와 같은 순서(플레이어 → 시계 → 코인 → 지갑 → 싸움)로 부르되,
/// 걷기·점프·좀비는 진짜 시간 `dt`, 시계와 코인은 `dt × time_scale`로 흘린다 (D13).
/// 규칙은 새로 만들지 않고 logic 함수를 부르는 순서만 따라 한다.
/// `time_scale`이 1이면 `Game::update`와 결과가 똑같다 (아래 테스트).
fn update_with_time_scale(game: &mut Game, input: &PlayerInput, dt: f32, time_scale: f32) -> Vec<GameEvent> {
    let mut events = Vec::new();
    if game.combat.health.is_down() {
        return events;
    }
    let world_dt = dt * time_scale;

    game.player.update(input, dt, &game.world);

    for clock_event in game.clock.update(world_dt) {
        events.push(match clock_event {
            ClockEvent::NightStarted => GameEvent::NightStarted,
            ClockEvent::DayStarted { day_number } => GameEvent::DayStarted { day_number },
        });
    }

    let picked = game.coins.update(world_dt, game.player.rect());
    if picked > 0 {
        game.wallet.add(picked);
        events.push(GameEvent::CoinsPicked { count: picked });
    }

    game.update_combat(input, dt, &mut events);
    events
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
        if rotate_hint || !matches!(scene, Scene::Play { down: None, .. }) {
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
                Scene::Play { game, fast_forward, effects, down } => {
                    // 플레이 중 터치는 조이스틱·버튼으로만. 쓰러진 화면의 터치는 위 idle이 메뉴 탭으로 준다.
                    let touched = if down.is_none() { touch.update(&fingers, &PLAY_BUTTONS, touch_mode) } else { TouchFrame::default() };
                    let retry = down.as_mut().is_some_and(|screen| {
                        screen.seconds += dt;
                        screen.ready() && (menu.confirm || menu.tap.is_some_and(|p| RETRY_BUTTON.contains(p)))
                    });
                    if menu.back || touched.back_pressed || retry {
                        Some(Scene::Select { picked: kind_index(game.player.kind) })
                    } else {
                        if fast_forward_pressed() {
                            *fast_forward = !*fast_forward;
                        }
                        let input = merge_touch(player_input(), &touched);
                        if input.attack_pressed && down.is_none() {
                            effects.start_swing();
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
                                GameEvent::Combat(CombatEvent::PlayerDown) => {
                                    let screen = DownScreen::new(game.clock.nights_survived(), best_days);
                                    if screen.new_record {
                                        best_days = screen.best;
                                        view::web::save_best_days(best_days);
                                    }
                                    *down = Some(screen);
                                }
                                GameEvent::Combat(combat) => effects.on_combat(combat),
                                GameEvent::NightStarted | GameEvent::DayStarted { .. } => {}
                            }
                        }
                        effects.update(dt);
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
            Scene::Play { game, fast_forward, effects, down } => {
                screens::draw_play(&assets, &text, game, *fast_forward, effects, down.as_ref(), touch_mode);
                if touch_mode && down.is_none() {
                    draw_touch_controls(&text, &touch, &PLAY_BUTTONS);
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

    const FRAME: f32 = 1.0 / 60.0;
    const SEED: u32 = 7;

    /// 상자 위로 올라가 코인을 줍고, 걷고, 점프하는 입력 묶음 (몇 초 동안).
    fn script(frame: usize) -> PlayerInput {
        PlayerInput { left: frame % 400 > 300, right: frame % 400 < 250, jump_pressed: frame % 90 == 0, attack_pressed: frame % 40 == 0 }
    }

    #[test]
    fn time_scale_one_matches_game_update() {
        let mut a = Game::new(CharacterKind::Niro, SEED);
        let mut b = Game::new(CharacterKind::Niro, SEED);
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
        let mut normal = Game::new(CharacterKind::Ellie, SEED);
        let mut fast = Game::new(CharacterKind::Ellie, SEED);
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
        let mut g = Game::new(CharacterKind::Logan, SEED);
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
        let mut g = Game::new(CharacterKind::Niro, SEED);
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
        while !g.combat.health.is_down() {
            let zombie = logic::monster::Zombie::new(g.player.pos.x, g.world.ground_y, false);
            g.combat.zombies.list = vec![zombie];
            update_with_time_scale(&mut g, &PlayerInput::default(), FRAME, DEV_FAST_FORWARD_SCALE);
        }
        let left = g.clock.seconds_left_in_phase();
        let right = PlayerInput { right: true, ..Default::default() };
        let x = g.player.pos.x;
        assert!(update_with_time_scale(&mut g, &right, FRAME, DEV_FAST_FORWARD_SCALE).is_empty());
        assert_eq!((g.clock.seconds_left_in_phase(), g.player.pos.x), (left, x));
    }

    #[test]
    fn long_frame_is_capped() {
        // 탭에 30초 다녀와도 한 프레임은 MAX_FRAME_SECONDS까지만. 빨리감기여도 낮/밤을 건너뛰지 않는다.
        let dt = cap_frame_seconds(30.0);
        assert_eq!(dt, config::MAX_FRAME_SECONDS);
        assert!(dt * DEV_FAST_FORWARD_SCALE < config::DUSK_SECONDS);
        let mut g = Game::new(CharacterKind::Niro, SEED);
        let events = update_with_time_scale(&mut g, &PlayerInput::default(), dt, DEV_FAST_FORWARD_SCALE);
        assert!(events.is_empty());
        assert_eq!(g.clock.day_number(), 1);
        assert!(!g.clock.is_night());
    }
}
