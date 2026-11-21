//! 한글 글자 그리기 (Jua 글꼴). 크기는 가상 화면(960×540) 기준 픽셀.
//! 창이 커지면 글꼴을 크게 그려서 확대해도 흐려지지 않게 한다. 단, 글꼴 크기는 정해진 몇 단계(`FONT_PIXEL_STEPS`)만 쓴다.
//! 모든 글자에 진한 테두리를 둘러서 하늘·바다·모래 어디서나 또렷하게 보이게 한다.

use macroquad::prelude::*;
use std::cell::RefCell;
use std::collections::HashSet;

thread_local! {
    /// 이미 쓴 (글자, 글꼴 단계 크기). [`TextPainter::flush_before_new_glyphs`]만 쓴다.
    static SEEN_GLYPHS: RefCell<HashSet<(char, u16)>> = RefCell::new(HashSet::new());
}

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
    /// 가상 1픽셀이 창에서 몇 픽셀인지 (Letterbox::scale). `MAX_TEXT_SCALE`에서 멈춘다.
    scale: f32,
}

/// 글꼴을 실제로 그려 두는 크기(픽셀)는 이 몇 단계뿐이다. 사이 크기는 `font_scale`로 늘리고 줄인다.
/// 크기마다 글자를 새로 그려 두면 글꼴 그림(아틀라스)이 8192까지 커지고, 새 글자마다 그 그림을 통째로
/// GPU에 올려서 최대 950ms 끊겼다 (qa/perf-2026-11-14.md). 단계가 정해져 있으면 창 크기를 바꿔도 늘지 않는다.
pub const FONT_PIXEL_STEPS: [u16; 7] = [16, 24, 32, 48, 64, 96, 128];
/// 글자를 그릴 때 쓰는 화면 배율 상한. 큰 화면에서는 조금 부드러워질 뿐 크기는 맞다.
pub const MAX_TEXT_SCALE: f32 = 2.0;
/// 글 한 줄이 바닥선 위로, 아래로 차지하는 높이 (글자 크기에 대한 비율, 테두리·그림자 포함 넉넉히).
/// 큰 알림 글끼리, 정보판과 겹치지 않게 자리를 잡을 때 쓴다 (BUG-031).
pub const TEXT_ABOVE: f32 = 1.0;
pub const TEXT_BELOW: f32 = 0.22;
/// 원하는 픽셀보다 이만큼까지 작은 단계는 그대로 써서 키운다 (거의 티가 안 난다).
const STEP_STRETCH: f32 = 1.15;

/// 실제 픽셀 `pixels`를 그릴 글꼴 단계: `pixels`를 (조금 키워서라도) 덮는 가장 작은 단계. 아주 크면 가장 큰 단계.
pub fn font_step(pixels: f32) -> u16 {
    FONT_PIXEL_STEPS.iter().copied().find(|step| *step as f32 * STEP_STRETCH >= pixels).unwrap_or(FONT_PIXEL_STEPS[FONT_PIXEL_STEPS.len() - 1])
}

/// macroquad는 글꼴 크기에 화면 dpi를 곱해서 글자를 그려 둔다 (폰은 dpi 3 안팎). 그 곱이 이 값을 넘지 않게 글꼴 크기를
/// 줄이고 `font_scale`로 되키운다. 폰에서 글꼴 그림(아틀라스)이 8192×8192(256MB)까지 커지고, 커질 때 0.5초씩 멈추던 것을
/// 막는다 (qa/perf-2026-11-21.md). 대신 폰 글자가 아주 조금 부드럽다.
pub const MAX_RASTER_DPI: f32 = 2.0;

/// dpi가 `MAX_RASTER_DPI`보다 크면 단계 크기를 그만큼 줄인 글꼴 크기. 그 밖에는 단계 그대로.
pub fn raster_font_size(step: u16, dpi: f32) -> u16 {
    let shrink = (MAX_RASTER_DPI / dpi.max(0.1)).min(1.0);
    ((step as f32 * shrink).round() as u16).max(1)
}

impl<'a> TextPainter<'a> {
    pub fn new(font: &'a Font, letterbox_scale: f32) -> TextPainter<'a> {
        TextPainter { font, scale: letterbox_scale.clamp(0.1, MAX_TEXT_SCALE) }
    }

    /// 글꼴 단계는 `size`로 고르고, 보이는 크기는 `size × zoom`. 커졌다 작아지는 글자("쿵!", "부릉!")가
    /// 프레임마다 새 글꼴 크기를 만들지 않게 한다.
    fn params_zoomed(&self, size: f32, zoom: f32, color: Color) -> TextParams<'a> {
        let font_size = raster_font_size(font_step(size * self.scale), screen_dpi_scale());
        TextParams {
            font: Some(self.font),
            font_size,
            font_scale: size * zoom / font_size as f32,
            color,
            ..Default::default()
        }
    }

    /// BUG-013: 새 글자로 글꼴 그림(아틀라스)이 꽉 차면 macroquad가 더 큰 그림을 만들면서 옛 그림을 바로 지운다.
    /// 이번 프레임에 쌓아 둔 글자는 아직 옛 그림을 가리켜서 까만 네모가 된다.
    /// 그래서 처음 쓰는 글자가 있으면, 쌓인 그리기를 먼저 끝내고 넘어간다.
    fn flush_before_new_glyphs(&self, text: &str, font_size: u16) {
        let is_new = SEEN_GLYPHS.with_borrow_mut(|seen| {
            // 모두 넣어야 해서 `any`로 줄이지 않는다
            text.chars().fold(false, |new, c| seen.insert((c, font_size)) || new)
        });
        if is_new {
            // miniquad는 지운 그림을 "지금 묶인 그림"으로 기억했다가 다시 묶으려 한다 (콘솔의 glBindTexture 오류).
            // 안 보이는 네모를 마지막에 그려서 글꼴 그림이 묶인 채로 지워지지 않게 한다.
            draw_rectangle(0.0, 0.0, 0.0, 0.0, BLANK);
            // 안전: macroquad 창이 열린 뒤 그리는 중에만 불린다.
            unsafe { get_internal_gl() }.flush();
        }
    }

    pub fn width(&self, text: &str, size: f32) -> f32 {
        self.width_zoomed(text, size, 1.0)
    }

    fn width_zoomed(&self, text: &str, size: f32, zoom: f32) -> f32 {
        let p = self.params_zoomed(size, zoom, WHITE);
        self.flush_before_new_glyphs(text, p.font_size);
        measure_text(text, p.font, p.font_size, p.font_scale).width
    }

    /// `left_x`를 왼쪽 끝으로 그린다 (정보판 숫자처럼 왼쪽에 붙일 때).
    pub fn left(&self, text: &str, left_x: f32, baseline_y: f32, size: f32, color: Color) {
        self.centered(text, left_x + self.width(text, size) / 2.0, baseline_y, size, color);
    }

    /// `center_x`를 가운데로, `baseline_y`를 글자 바닥선으로 그린다.
    /// 그림자 → 진한 테두리 → 글자 순서. 테두리·그림자도 `color`의 투명도를 따른다.
    pub fn centered(&self, text: &str, center_x: f32, baseline_y: f32, size: f32, color: Color) {
        self.centered_zoomed(text, center_x, baseline_y, size, 1.0, color);
    }

    /// 테두리 없이 `color` 글자와 `shadow` 그림자만 그린다 (US-109 로고: 빨간 바탕에 흰 둥근 글자).
    /// 글꼴 단계는 `size`로 고르고 `zoom`만 바꿔서, 커졌다 작아져도 새 글꼴 크기를 만들지 않는다 (BUG-029).
    pub fn plain_zoomed(&self, text: &str, center_x: f32, baseline_y: f32, size: f32, zoom: f32, color: Color, shadow: Color) {
        if zoom <= 0.0 {
            return;
        }
        let shown = size * zoom;
        let x = center_x - self.width_zoomed(text, size, zoom) / 2.0;
        let d = (shown * 0.05).max(1.5);
        draw_text_ex(text, x + d, baseline_y + d, self.params_zoomed(size, zoom, shadow));
        draw_text_ex(text, x, baseline_y, self.params_zoomed(size, zoom, color));
    }

    /// `centered`와 같은데 `size × zoom` 크기로 보인다. 크기가 움직이는 글자는 `size`를 고정하고 `zoom`만 바꾼다.
    pub fn centered_zoomed(&self, text: &str, center_x: f32, baseline_y: f32, size: f32, zoom: f32, color: Color) {
        let shown = size * zoom;
        let x = center_x - self.width_zoomed(text, size, zoom) / 2.0;
        let shadow = (shown * 0.06).max(2.0);
        draw_text_ex(text, x + shadow, baseline_y + shadow, self.params_zoomed(size, zoom, Color::new(0.0, 0.0, 0.0, 0.35 * color.a)));
        let outline = Color { a: color.a, ..OUTLINE_COLOR };
        draw_outline_rings(shown * TITLE_OUTLINE_RATIO * TEXT_OUTLINE_SHARE, |d| {
            draw_text_ex(text, x + d.x, baseline_y + d.y, self.params_zoomed(size, zoom, outline));
        });
        draw_text_ex(text, x, baseline_y, self.params_zoomed(size, zoom, color));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn font_sizes_use_only_a_few_steps_on_any_screen() {
        // 화면 배율이 얼마든, 글자 크기가 얼마든 글꼴 단계는 정해진 7개 안이다 (아틀라스가 끝없이 크지 않게)
        let mut used = std::collections::BTreeSet::new();
        for scale_x100 in 30..=400 {
            let scale = (scale_x100 as f32 / 100.0).clamp(0.1, MAX_TEXT_SCALE);
            for size in 14..=120 {
                used.insert(font_step(size as f32 * scale));
            }
        }
        assert!(used.iter().all(|s| FONT_PIXEL_STEPS.contains(s)));
        assert!(used.len() <= FONT_PIXEL_STEPS.len());
    }

    #[test]
    fn raster_size_is_capped_on_high_dpi_but_not_on_low() {
        for step in FONT_PIXEL_STEPS {
            assert_eq!(raster_font_size(step, 1.0), step);
            assert_eq!(raster_font_size(step, 1.25), step);
            assert_eq!(raster_font_size(step, 2.0), step);
            for dpi in [2.5f32, 3.0, 4.0] {
                let drawn = raster_font_size(step, dpi) as f32 * dpi;
                assert!(drawn <= step as f32 * MAX_RASTER_DPI + dpi, "{step} dpi {dpi} -> {drawn}");
            }
        }
        assert_eq!(raster_font_size(48, 3.0), 32);
        assert_eq!(raster_font_size(16, 4.0), 8);
    }

    #[test]
    fn step_covers_the_wanted_pixels_so_text_stays_sharp() {
        for px in 1..=128 {
            let step = font_step(px as f32) as f32;
            assert!(step * STEP_STRETCH >= px as f32, "{px}px → {step}");
        }
        assert_eq!(font_step(30.0), 32);
        assert_eq!(font_step(50.0), 48, "조금 넘는 건 키워서 쓴다");
        assert_eq!(font_step(70.0), 64);
        assert_eq!(font_step(500.0), 128, "아주 크면 가장 큰 단계");
    }
}
