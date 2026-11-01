//! 미스 찾기 — 게임 루프: 입력 → logic 업데이트 → 그리기.
//! 화면 흐름: 타이틀 → 캐릭터 고르기 → 플레이 (Esc를 누르면 다시 고르기).

mod config;
mod logic;
mod view;

use logic::character::CharacterKind;
use logic::player::Player;
use logic::world::World;
use macroquad::prelude::*;
use view::assets::Assets;
use view::input::{menu_input, player_input};
use view::screen::Letterbox;
use view::screens;
use view::text::TextPainter;

enum Scene {
    Title,
    /// `picked`: 골라진 캐릭터 번호 (CharacterKind::ALL 순서)
    Select { picked: usize },
    Play { world: World, player: Player },
}

fn start_play(kind: CharacterKind) -> Scene {
    let world = World::m1();
    // 왼쪽, 상자가 없는 곳에서 시작한다.
    let spawn = vec2(120.0, world.ground_y - config::PLAYER_HEIGHT);
    let player = Player::new(kind, spawn);
    Scene::Play { world, player }
}

fn kind_index(kind: CharacterKind) -> usize {
    CharacterKind::ALL.iter().position(|k| *k == kind).unwrap_or(0)
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
    let assets = Assets::load().await;
    let mut scene = Scene::Title;
    let mut first_frame = true;

    loop {
        let letterbox = Letterbox::current();
        let menu = menu_input(&letterbox);

        // 1) 입력 → 장면 바꾸기 / logic 업데이트. 장면이 바뀐 프레임에는 새 장면을 업데이트하지 않는다
        //    (같은 Space 한 번이 "결정"과 "점프"로 두 번 쓰이지 않게).
        let next = match &mut scene {
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
            Scene::Play { world, player } => {
                if menu.back {
                    Some(Scene::Select { picked: kind_index(player.kind) })
                } else {
                    player.update(&player_input(), get_frame_time(), world);
                    None
                }
            }
        };
        if let Some(next) = next {
            scene = next;
        }

        // 2) 그리기 (960×540 가상 화면)
        letterbox.begin();
        let text = TextPainter::new(&assets.font, letterbox.scale);
        match &scene {
            Scene::Title => screens::draw_title(&assets, &text, config::GROUND_Y),
            Scene::Select { picked } => screens::draw_select(&assets, &text, config::GROUND_Y, *picked),
            Scene::Play { world, player } => screens::draw_play(&assets, &text, world, player),
        }
        letterbox.end();
        if first_frame {
            // 첫 그림을 그렸으니 웹페이지의 "불러오는 중..." 글자를 치운다.
            view::web::hide_loading_screen();
            first_frame = false;
        }

        next_frame().await
    }
}
