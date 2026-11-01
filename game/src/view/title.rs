//! 타이틀 제목 "미스 찾기" — 자모마다 다른 색 (D7, 아이가 정한 색).
//!
//! 완성형 글자(예: "찾")는 글꼴 안에서 그림 하나라서 부분마다 색을 줄 수 없다.
//! 그래서 1) 음절마다 Jua 글리프를 흰색 그림(텍스처)으로 한 번 만들어 두고
//! 2) 그 그림을 자모 영역(네모)마다 잘라서 색을 입혀 여러 번 그린다.
//! 영역은 글리프 네모에 대한 비율(0~1)이라 글자 크기가 달라도 그대로 맞는다.

use macroquad::prelude::*;

// ---- 아이가 정한 색 ----
/// 초록
const GREEN: Color = Color::new(0.20, 0.78, 0.25, 1.0);
/// 핑크
const PINK: Color = Color::new(1.0, 0.45, 0.75, 1.0);
/// 파랑
const BLUE: Color = Color::new(0.15, 0.40, 1.0, 1.0);
/// 주황
const ORANGE: Color = Color::new(1.0, 0.55, 0.10, 1.0);
/// 빨강
const RED: Color = Color::new(0.95, 0.15, 0.15, 1.0);
/// 노랑
const YELLOW: Color = Color::new(1.0, 0.90, 0.10, 1.0);
/// 하양
const WHITE_JAMO: Color = Color::new(1.0, 1.0, 1.0, 1.0);
/// 검정
const BLACK_JAMO: Color = Color::new(0.05, 0.05, 0.05, 1.0);
/// 회색
const GRAY: Color = Color::new(0.62, 0.62, 0.66, 1.0);

// 테두리 색·두께는 다른 글자와 같이 text.rs에서 정한다.
use super::text::{draw_outline_rings, OUTLINE_COLOR, TITLE_OUTLINE_RATIO};
/// 그림자
const SHADOW: Color = Color::new(0.0, 0.0, 0.0, 0.30);
const SHADOW_RATIO: f32 = 0.07;

/// 글리프를 이 크기(px)로 한 번 그려 두고 줄여서 쓴다.
const RASTER_PX: f32 = 256.0;

/// 음절 하나와 그 자모 영역들: (x0, y0, x1, y1) 글리프 네모 비율 (왼쪽 위가 0,0), 색.
type Part = ((f32, f32, f32, f32), Color);

/// 자모 영역. 경계선은 Jua 글리프를 그려 보고 획 사이 빈틈에 맞췄다
/// (`TITLE_PREVIEW_DIR=폴더 cargo test export_title_glyphs -- --ignored`로 확인).
fn syllable_parts(ch: char) -> Vec<Part> {
    match ch {
        // 미: 왼쪽 ㅁ | 오른쪽 ㅣ
        '미' => vec![((0.0, 0.0, MI_SPLIT, 1.0), GREEN), ((MI_SPLIT, 0.0, 1.0, 1.0), PINK)],
        // 스: 위 ㅅ / 아래 ㅡ
        '스' => vec![((0.0, 0.0, 1.0, SEU_SPLIT), BLUE), ((0.0, SEU_SPLIT, 1.0, 1.0), ORANGE)],
        // 찾: 위 왼쪽 ㅊ, 위 오른쪽 ㅏ, 아래 받침 ㅈ
        '찾' => vec![
            ((0.0, 0.0, CHAT_SPLIT_X, CHAT_SPLIT_Y), RED),
            ((CHAT_SPLIT_X, 0.0, 1.0, CHAT_SPLIT_Y), YELLOW),
            ((0.0, CHAT_SPLIT_Y, 1.0, 1.0), WHITE_JAMO),
        ],
        // 기: 왼쪽 ㄱ | 오른쪽 ㅣ
        '기' => vec![((0.0, 0.0, GI_SPLIT, 1.0), BLACK_JAMO), ((GI_SPLIT, 0.0, 1.0, 1.0), GRAY)],
        _ => vec![((0.0, 0.0, 1.0, 1.0), WHITE)],
    }
}

const MI_SPLIT: f32 = 0.73;
const SEU_SPLIT: f32 = 0.765;
const CHAT_SPLIT_X: f32 = 0.617;
const CHAT_SPLIT_Y: f32 = 0.593;
const GI_SPLIT: f32 = 0.687;

pub const TITLE_TEXT: &str = "미스 찾기";

/// 흰색 글리프 그림 (RGBA, 알파 = 글자 진하기).
struct GlyphBitmap {
    /// 펜 위치에서 그림 왼쪽까지 (px, RASTER_PX 기준)
    xmin: f32,
    /// 바닥선에서 그림 아래쪽까지 (위쪽이 +)
    ymin: f32,
    width: u16,
    height: u16,
    advance: f32,
    rgba: Vec<u8>,
}

fn rasterize(font: &fontdue::Font, ch: char) -> GlyphBitmap {
    let (m, coverage) = font.rasterize(ch, RASTER_PX);
    let rgba = coverage.iter().flat_map(|&a| [255, 255, 255, a]).collect();
    GlyphBitmap {
        xmin: m.xmin as f32,
        ymin: m.ymin as f32,
        width: m.width as u16,
        height: m.height as u16,
        advance: m.advance_width,
        rgba,
    }
}

struct Glyph {
    xmin: f32,
    ymin: f32,
    advance: f32,
    texture: Option<Texture2D>,
    parts: Vec<Part>,
}

pub struct TitleArt {
    glyphs: Vec<Glyph>,
}

impl TitleArt {
    /// `font_bytes`: Jua 글꼴 파일 내용.
    pub fn new(font_bytes: &[u8]) -> TitleArt {
        let font = fontdue::Font::from_bytes(font_bytes, fontdue::FontSettings::default())
            .expect("한글 글꼴(Jua)을 읽지 못했어요");
        let glyphs = TITLE_TEXT
            .chars()
            .map(|ch| {
                let g = rasterize(&font, ch);
                let texture = (g.width > 0 && g.height > 0).then(|| {
                    let t = Texture2D::from_rgba8(g.width, g.height, &g.rgba);
                    t.set_filter(FilterMode::Linear);
                    t
                });
                Glyph { xmin: g.xmin, ymin: g.ymin, advance: g.advance, texture, parts: syllable_parts(ch) }
            })
            .collect();
        TitleArt { glyphs }
    }

    /// `center_x`를 가운데로, `baseline_y`를 바닥선으로, 글자 크기 `size`(가상 px)로 그린다.
    pub fn draw(&self, center_x: f32, baseline_y: f32, size: f32) {
        let s = size / RASTER_PX;
        let total: f32 = self.glyphs.iter().map(|g| g.advance).sum::<f32>() * s;
        let outline = size * TITLE_OUTLINE_RATIO;
        let shadow = size * SHADOW_RATIO;

        // 글자마다 (텍스처, 왼쪽 위 위치, 크기)
        let mut pen = center_x - total / 2.0;
        let mut placed = Vec::new();
        for g in &self.glyphs {
            if let Some(t) = &g.texture {
                let size = vec2(t.width(), t.height()) * s;
                let top_left = vec2(pen + g.xmin * s, baseline_y - (g.ymin * s) - size.y);
                placed.push((g, t, top_left, size));
            }
            pen += g.advance * s;
        }

        let whole = |t: &Texture2D, pos: Vec2, size: Vec2, color: Color| {
            draw_texture_ex(t, pos.x, pos.y, color, DrawTextureParams { dest_size: Some(size), ..Default::default() });
        };
        // 1) 그림자 2) 테두리(여러 방향으로 조금씩 밀어서 진하게) 3) 자모별 색
        for (_, t, pos, size) in &placed {
            whole(t, *pos + vec2(shadow, shadow), *size, SHADOW);
        }
        draw_outline_rings(outline, |d| {
            for (_, t, pos, size) in &placed {
                whole(t, *pos + d, *size, OUTLINE_COLOR);
            }
        });
        for (g, t, pos, size) in &placed {
            for &((x0, y0, x1, y1), color) in &g.parts {
                let (tw, th) = (t.width(), t.height());
                draw_texture_ex(
                    t,
                    pos.x + x0 * size.x,
                    pos.y + y0 * size.y,
                    color,
                    DrawTextureParams {
                        dest_size: Some(vec2((x1 - x0) * size.x, (y1 - y0) * size.y)),
                        source: Some(Rect::new(x0 * tw, y0 * th, (x1 - x0) * tw, (y1 - y0) * th)),
                        ..Default::default()
                    },
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font() -> fontdue::Font {
        let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/fonts/Jua-Regular.ttf")).unwrap();
        fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).unwrap()
    }

    /// 경계선이 글자 획을 자르지 않는지: 경계선 위 픽셀이 거의 비어 있어야 한다.
    #[test]
    fn jamo_splits_fall_in_gaps() {
        let f = font();
        for ch in ['미', '스', '찾', '기'] {
            let g = rasterize(&f, ch);
            let (w, h) = (g.width as usize, g.height as usize);
            let alpha = |x: usize, y: usize| g.rgba[(y * w + x) * 4 + 3];
            for ((x0, y0, x1, y1), _) in syllable_parts(ch) {
                // 각 영역의 안쪽 경계(0이나 1이 아닌 쪽)를 따라 픽셀을 본다.
                for fx in [x0, x1].into_iter().filter(|v| *v > 0.0 && *v < 1.0) {
                    let x = ((fx * w as f32) as usize).min(w - 1);
                    let ink = (((y0 * h as f32) as usize)..((y1 * h as f32) as usize).min(h))
                        .filter(|&y| alpha(x, y) > 128)
                        .count();
                    assert!(ink <= 2, "{ch} 세로 경계 {fx}에 획이 {ink}픽셀 걸림");
                }
                for fy in [y0, y1].into_iter().filter(|v| *v > 0.0 && *v < 1.0) {
                    let y = ((fy * h as f32) as usize).min(h - 1);
                    let ink = (((x0 * w as f32) as usize)..((x1 * w as f32) as usize).min(w))
                        .filter(|&x| alpha(x, y) > 128)
                        .count();
                    assert!(ink <= 2, "{ch} 가로 경계 {fy}에 획이 {ink}픽셀 걸림");
                }
            }
        }
    }

    /// 확인용: 음절별로 자모 색을 칠한 PNG를 저장한다 (경계선은 빨간 줄).
    /// `TITLE_PREVIEW_DIR=폴더 cargo test export_title_glyphs -- --ignored`
    #[test]
    #[ignore]
    fn export_title_glyphs() {
        let dir = std::env::var("TITLE_PREVIEW_DIR").expect("TITLE_PREVIEW_DIR");
        let f = font();
        for (n, ch) in ['미', '스', '찾', '기'].into_iter().enumerate() {
            let g = rasterize(&f, ch);
            let (w, h) = (g.width as usize, g.height as usize);
            let mut img = Image::gen_image_color(w as u16, h as u16, Color::new(0.55, 0.8, 1.0, 1.0));
            let parts = syllable_parts(ch);
            // 획이 하나도 없는 줄(경계선을 둘 수 있는 곳)을 비율로 알려 준다.
            let ink = |x: usize, y: usize| g.rgba[(y * w + x) * 4 + 3] > 128;
            let empty_rows: Vec<String> =
                (0..h).filter(|&y| !(0..w).any(|x| ink(x, y))).map(|y| format!("{:.3}", y as f32 / h as f32)).collect();
            let empty_cols: Vec<String> =
                (0..w).filter(|&x| !(0..h).any(|y| ink(x, y))).map(|x| format!("{:.3}", x as f32 / w as f32)).collect();
            let upper = (h as f32 * CHAT_SPLIT_Y) as usize;
            let empty_cols_top: Vec<String> = (0..w)
                .filter(|&x| !(0..upper).any(|y| ink(x, y)))
                .map(|x| format!("{:.3}", x as f32 / w as f32))
                .collect();
            println!("{ch}: 빈 가로줄 {empty_rows:?}\n   빈 세로줄 {empty_cols:?}\n   위쪽만 빈 세로줄 {empty_cols_top:?}");
            for y in 0..h {
                for x in 0..w {
                    let a = g.rgba[(y * w + x) * 4 + 3] as f32 / 255.0;
                    let (fx, fy) = (x as f32 / w as f32, y as f32 / h as f32);
                    let c = parts
                        .iter()
                        .find(|((x0, y0, x1, y1), _)| fx >= *x0 && fx < *x1 && fy >= *y0 && fy < *y1)
                        .map(|p| p.1)
                        .unwrap_or(WHITE);
                    let bg = Color::new(0.55, 0.8, 1.0, 1.0);
                    let mut px = Color::new(bg.r + (c.r - bg.r) * a, bg.g + (c.g - bg.g) * a, bg.b + (c.b - bg.b) * a, 1.0);
                    // 10% 눈금(옅은 회색)
                    if (x * 10) % w < 10 || (y * 10) % h < 10 {
                        px = Color::new(px.r * 0.8, px.g * 0.8, px.b * 0.8, 1.0);
                    }
                    let on_split = parts.iter().any(|((x0, y0, x1, y1), _)| {
                        [*x0, *x1].iter().any(|v| *v > 0.0 && *v < 1.0 && (v * w as f32) as usize == x)
                            || [*y0, *y1].iter().any(|v| *v > 0.0 && *v < 1.0 && (v * h as f32) as usize == y)
                    });
                    if on_split {
                        px = RED;
                    }
                    // export_png는 위아래를 뒤집으므로 미리 뒤집는다.
                    img.set_pixel(x as u32, (h - 1 - y) as u32, px);
                }
            }
            img.export_png(&format!("{dir}/title_{n}.png"));
        }
    }
}
