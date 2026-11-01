//! 한글 글자 그리기 (Jua 글꼴). 크기는 가상 화면(960×540) 기준 픽셀.
//! 창이 커지면 글꼴을 그만큼 크게 그려서 확대해도 흐려지지 않게 한다.
//! 모든 글자에 진한 테두리를 둘러서 하늘·바다·모래 어디서나 또렷하게 보이게 한다.

use macroquad::prelude::*;

/// 글자 테두리 색 (진한 남색)
pub const OUTLINE_COLOR: Color = Color::new(0.13, 0.16, 0.36, 1.0);
/// 타이틀 제목 테두리 두께: 글자 크기에 대한 비율. 아이가 "더 굵게!"라고 해서 0.09.
/// 이 값 하나로 모든 글자 테두리가 같이 굵어지거나 얇아진다.
pub const TITLE_OUTLINE_RATIO: f32 = 0.09;
/// 다른 글자(안내 문구, 캐릭터 이름 등)는 제목보다 조금 얇게: 제목 비율에 이만큼 곱한다.
const TEXT_OUTLINE_SHARE: f32 = 0.6;

/// 둥근 테두리: 원 둘레 16방향으로, 바깥 원과 그 절반 크기 원 두 겹으로 밀어 그린다.
/// 굵게 해도 뭉개지거나 빈틈이 생기지 않는다. `draw(offset)`을 여러 번 부른다.
pub fn draw_outline_rings(radius: f32, mut draw: impl FnMut(Vec2)) {
    const DIRECTIONS: usize = 16;
    for r in [radius * 0.5, radius] {
        for i in 0..DIRECTIONS {
            let a = i as f32 / DIRECTIONS as f32 * std::f32::consts::TAU;
            draw(vec2(a.cos(), a.sin()) * r);
        }
    }
}

pub struct TextPainter<'a> {
    font: &'a Font,
    /// 가상 1픽셀이 창에서 몇 픽셀인지 (Letterbox::scale)
    scale: f32,
}

impl<'a> TextPainter<'a> {
    pub fn new(font: &'a Font, letterbox_scale: f32) -> TextPainter<'a> {
        // 창 크기를 끌어서 바꿀 때 글꼴 크기가 너무 많이 생기지 않게 0.1 단위로 맞춘다.
        let scale = ((letterbox_scale * 10.0).round() / 10.0).max(0.1);
        TextPainter { font, scale }
    }

    fn params(&self, size: f32, color: Color) -> TextParams<'a> {
        let font_size = (size * self.scale).round().max(1.0) as u16;
        TextParams {
            font: Some(self.font),
            font_size,
            font_scale: size / font_size as f32,
            color,
            ..Default::default()
        }
    }

    pub fn width(&self, text: &str, size: f32) -> f32 {
        let p = self.params(size, WHITE);
        measure_text(text, p.font, p.font_size, p.font_scale).width
    }

    /// `left_x`를 왼쪽 끝으로 그린다 (정보판 숫자처럼 왼쪽에 붙일 때).
    pub fn left(&self, text: &str, left_x: f32, baseline_y: f32, size: f32, color: Color) {
        self.centered(text, left_x + self.width(text, size) / 2.0, baseline_y, size, color);
    }

    /// `center_x`를 가운데로, `baseline_y`를 글자 바닥선으로 그린다.
    /// 그림자 → 진한 테두리 → 글자 순서. 테두리·그림자도 `color`의 투명도를 따른다.
    pub fn centered(&self, text: &str, center_x: f32, baseline_y: f32, size: f32, color: Color) {
        let x = center_x - self.width(text, size) / 2.0;
        let shadow = (size * 0.06).max(2.0);
        draw_text_ex(text, x + shadow, baseline_y + shadow, self.params(size, Color::new(0.0, 0.0, 0.0, 0.35 * color.a)));
        let outline = Color { a: color.a, ..OUTLINE_COLOR };
        draw_outline_rings(size * TITLE_OUTLINE_RATIO * TEXT_OUTLINE_SHARE, |d| {
            draw_text_ex(text, x + d.x, baseline_y + d.y, self.params(size, outline));
        });
        draw_text_ex(text, x, baseline_y, self.params(size, color));
    }
}
