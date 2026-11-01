// M0 "안녕" 화면: 하늘색 배경 + 제목 자리 + 캐릭터 자리 표시 네모 3개.
// 한글 폰트가 아직 없어서 화면 글자는 영어로 쓴다.
// 게임 제목은 아이가 정한다 (stories/backlog.md US-001).

mod config;

use macroquad::prelude::*;

const SKY: Color = Color::new(0.53, 0.81, 0.98, 1.0);
const PINK: Color = Color::new(0.93, 0.36, 0.55, 1.0);
const NIRO_YELLOW: Color = Color::new(0.95, 0.82, 0.15, 1.0);
const ELLIE_BLUE: Color = Color::new(0.30, 0.65, 0.95, 1.0);
const LOGAN_RED: Color = Color::new(0.78, 0.15, 0.25, 1.0);
const CROWN_GOLD: Color = Color::new(0.98, 0.78, 0.10, 1.0);

fn window_conf() -> Conf {
    Conf {
        window_title: "Game title coming soon".to_owned(),
        window_width: 960,
        window_height: 540,
        high_dpi: true,
        ..Default::default()
    }
}

fn draw_centered(text: &str, y: f32, size: f32, color: Color) {
    let dim = measure_text(text, None, size as u16, 1.0);
    draw_text(text, (screen_width() - dim.width) / 2.0, y, size, color);
}

/// 캐릭터 자리 표시: 몸통 네모 + 이름. 모자/왕관은 따로 그린다.
fn draw_placeholder(cx: f32, ground: f32, w: f32, h: f32, body: Color, name: &str) {
    draw_rectangle(cx - w / 2.0, ground - h, w, h, body);
    draw_rectangle_lines(cx - w / 2.0, ground - h, w, h, 3.0, BLACK);
    let size = (w * 0.35).max(16.0);
    let dim = measure_text(name, None, size as u16, 1.0);
    draw_text(name, cx - dim.width / 2.0, ground + size, size, BLACK);
}

#[macroquad::main(window_conf)]
async fn main() {
    loop {
        clear_background(SKY);
        let sw = screen_width();
        let sh = screen_height();
        let unit = (sw / 10.0).min(sh / 6.0);

        let title_size = (unit * 0.6).max(24.0);
        draw_centered("Hello! Game title coming soon", sh * 0.2, title_size, DARKBLUE);

        let ground = sh * 0.72;
        let w = unit * 1.2;
        let h = unit * 1.6;
        let gap = sw / 4.0;

        // 캐릭터 Niro: 노란 몸 + 분홍 모자
        let x = gap;
        draw_placeholder(x, ground, w, h, NIRO_YELLOW, "Niro");
        draw_rectangle(x - w * 0.35, ground - h - w * 0.35, w * 0.7, w * 0.35, PINK);

        // 캐릭터 Ellie: 왼쪽 파랑, 오른쪽 분홍
        let x = gap * 2.0;
        draw_placeholder(x, ground, w, h, ELLIE_BLUE, "Ellie");
        draw_rectangle(x, ground - h, w / 2.0, h, PINK);
        draw_rectangle_lines(x - w / 2.0, ground - h, w, h, 3.0, BLACK);

        // 캐릭터 Logan: 빨간 몸 + 금 왕관
        let x = gap * 3.0;
        draw_placeholder(x, ground, w, h, LOGAN_RED, "Logan");
        let top = ground - h;
        let cw = w * 0.7;
        let cl = x - cw / 2.0;
        draw_rectangle(cl, top - w * 0.15, cw, w * 0.15, CROWN_GOLD);
        for i in 0..3 {
            let px = cl + cw * (i as f32) / 2.0;
            let half = cw / 6.0;
            let bx = (px - half).max(cl);
            let ex = (px + half).min(cl + cw);
            draw_triangle(
                vec2(bx, top - w * 0.15),
                vec2(ex, top - w * 0.15),
                vec2(px, top - w * 0.4),
                CROWN_GOLD,
            );
        }

        next_frame().await
    }
}
