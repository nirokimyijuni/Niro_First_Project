//! 그림·글꼴 불러오기.
//!
//! 캐릭터 그림은 아이 원화(`art/concept/게임 케릭터.png`)를 그대로 쓴다.
//! 원화 한 장(547×428)에 세 캐릭터가 같이 있어서, 게임이 시작할 때
//! 1) 캐릭터별 네모로 잘라 내고 2) 바깥 종이 배경을 투명하게 만든다.

use super::sky::SkyArt;
use super::title::TitleArt;
use crate::logic::character::CharacterKind;
use macroquad::prelude::*;
use std::collections::VecDeque;

pub const FONT_PATH: &str = "assets/fonts/Jua-Regular.ttf";
pub const CHARACTER_SHEET_PATH: &str = "assets/sprites/characters.png";

/// 원화에서 캐릭터별로 잘라 낼 네모 (x, y, 가로, 세로), 원화 픽셀 기준.
/// 위쪽 손글씨 이름과 이름에서 내려오는 선은 빼고, 모자·머리·왕관은 넣었다.
/// 아래쪽은 발끝에 딱 맞춰 잘라서, 그림 아래 끝 = 발 위치가 된다.
pub const NIRO_CUT: (u32, u32, u32, u32) = (34, 176, 182, 236);
pub const ELLIE_CUT: (u32, u32, u32, u32) = (232, 199, 132, 181);
pub const LOGAN_CUT: (u32, u32, u32, u32) = (382, 163, 130, 207);

/// 이 밝기(0~255) 이상이고
const PAPER_MIN_BRIGHTNESS: u8 = 148;
/// 색 진하기(가장 큰 색 - 가장 작은 색)가 이 값 이하면 "종이"로 본다.
const PAPER_MAX_SATURATION: u8 = 40;
/// 캐릭터 안쪽(얼굴·다리)의 종이색은 이 색으로 밝게 칠한다.
const INSIDE_PAPER_COLOR: [u8; 4] = [252, 250, 244, 255];
/// 사진이라 연필 선은 회색, 크레용은 흐릿하다. 밝기를 늘려서 또렷하게 한다:
/// 이 값 이하는 검정, 종이 밝기(LEVEL_WHITE) 이상은 흰색이 되도록 펼친다.
const LEVEL_BLACK: f32 = 50.0;
const LEVEL_WHITE: f32 = 165.0;

pub fn character_cut(kind: CharacterKind) -> (u32, u32, u32, u32) {
    match kind {
        CharacterKind::Niro => NIRO_CUT,
        CharacterKind::Ellie => ELLIE_CUT,
        CharacterKind::Logan => LOGAN_CUT,
    }
}

pub struct Assets {
    pub font: Font,
    /// 자모별 색 타이틀 제목
    pub title: TitleArt,
    /// `CharacterKind::ALL` 순서 (Niro, Ellie, Logan).
    characters: [Texture2D; 3],
    /// 코드로 만든 하늘 그림 (해·달·구름·빛). 시작할 때 한 번만 만든다.
    pub sky: SkyArt,
}

impl Assets {
    pub async fn load() -> Assets {
        let font_bytes = load_file(FONT_PATH)
            .await
            .expect("한글 글꼴(Jua)을 불러오지 못했어요");
        let font = load_ttf_font_from_bytes(&font_bytes).expect("한글 글꼴(Jua)을 읽지 못했어요");
        let title = TitleArt::new(&font_bytes);
        let bytes = load_file(CHARACTER_SHEET_PATH)
            .await
            .expect("캐릭터 그림을 불러오지 못했어요");
        let sheet = Image::from_file_with_format(&bytes, Some(ImageFormat::Png))
            .expect("캐릭터 그림(PNG)을 읽지 못했어요");
        let characters = CharacterKind::ALL.map(|kind| {
            let texture = Texture2D::from_image(&cut_character(&sheet, kind));
            texture.set_filter(FilterMode::Linear);
            texture
        });
        Assets { font, title, characters, sky: SkyArt::new() }
    }

    pub fn character(&self, kind: CharacterKind) -> &Texture2D {
        let index = CharacterKind::ALL.iter().position(|k| *k == kind).unwrap_or(0);
        &self.characters[index]
    }
}

/// 원화에서 한 캐릭터를 잘라 내고 배경을 투명하게 만든다.
pub fn cut_character(sheet: &Image, kind: CharacterKind) -> Image {
    let (x, y, w, h) = character_cut(kind);
    let mut image = sheet.sub_image(Rect::new(x as f32, y as f32, w as f32, h as f32));
    remove_paper_background(&mut image);
    image
}

fn is_paper(px: [u8; 4]) -> bool {
    let [r, g, b, _] = px;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let brightness = ((r as u16 + g as u16 + b as u16) / 3) as u8;
    brightness >= PAPER_MIN_BRIGHTNESS && max - min <= PAPER_MAX_SATURATION
}

/// 네모 테두리에서 시작해 이어진 종이색 픽셀을 모두 투명하게 만든다 (페인트 통 채우기).
/// 연필 선으로 둘러싸인 안쪽(얼굴, 다리)은 테두리와 이어지지 않으므로 남는다 → 흰색으로 정리.
pub fn remove_paper_background(image: &mut Image) {
    let w = image.width as usize;
    let h = image.height as usize;
    let data = image.get_image_data_mut();
    let mut outside = vec![false; w * h];
    let mut queue = VecDeque::new();

    let visit = |i: usize, outside: &mut Vec<bool>, queue: &mut VecDeque<usize>| {
        if !outside[i] && is_paper(data[i]) {
            outside[i] = true;
            queue.push_back(i);
        }
    };
    for x in 0..w {
        visit(x, &mut outside, &mut queue);
        visit((h - 1) * w + x, &mut outside, &mut queue);
    }
    for y in 0..h {
        visit(y * w, &mut outside, &mut queue);
        visit(y * w + w - 1, &mut outside, &mut queue);
    }
    while let Some(i) = queue.pop_front() {
        let (x, y) = (i % w, i / w);
        if x > 0 {
            visit(i - 1, &mut outside, &mut queue);
        }
        if x + 1 < w {
            visit(i + 1, &mut outside, &mut queue);
        }
        if y > 0 {
            visit(i - w, &mut outside, &mut queue);
        }
        if y + 1 < h {
            visit(i + w, &mut outside, &mut queue);
        }
    }

    for i in 0..w * h {
        if outside[i] {
            // 색은 남겨 두고 투명하게만 → Linear 필터로 확대해도 가장자리가 검게 번지지 않는다.
            data[i][3] = 0;
        } else if is_paper(data[i]) {
            data[i] = INSIDE_PAPER_COLOR;
        } else {
            for c in 0..3 {
                let v = (data[i][c] as f32 - LEVEL_BLACK) * 255.0 / (LEVEL_WHITE - LEVEL_BLACK);
                data[i][c] = v.clamp(0.0, 255.0) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sheet() -> Image {
        let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/", "assets/sprites/characters.png"))
            .expect("characters.png");
        Image::from_file_with_format(&bytes, Some(ImageFormat::Png)).unwrap()
    }

    #[test]
    fn cuts_fit_inside_sheet() {
        let s = sheet();
        for kind in CharacterKind::ALL {
            let (x, y, w, h) = character_cut(kind);
            assert!(x + w <= s.width as u32 && y + h <= s.height as u32, "{kind:?}");
        }
    }

    #[test]
    fn background_becomes_clear_but_character_stays() {
        let s = sheet();
        for kind in CharacterKind::ALL {
            let img = cut_character(&s, kind);
            let data = img.get_image_data();
            let (w, h) = (img.width as usize, img.height as usize);
            // 네 귀퉁이는 종이 → 투명
            for i in [0, w - 1, (h - 1) * w, h * w - 1] {
                assert_eq!(data[i][3], 0, "{kind:?} 귀퉁이가 투명해야 한다");
            }
            // 캐릭터 몸이 충분히 남아 있다 (전체의 1/4 이상 보임)
            let visible = data.iter().filter(|p| p[3] > 0).count();
            assert!(visible * 4 > w * h, "{kind:?} 남은 픽셀 {visible}/{}", w * h);
        }
    }

    /// 확인용: 잘라 낸 그림을 PNG로 저장한다. `CUT_PREVIEW_DIR=폴더 cargo test -- --ignored`
    #[test]
    #[ignore]
    fn export_cut_preview() {
        let dir = std::env::var("CUT_PREVIEW_DIR").expect("CUT_PREVIEW_DIR");
        let s = sheet();
        for kind in CharacterKind::ALL {
            let img = cut_character(&s, kind);
            // export_png는 위아래를 뒤집어 저장하므로 미리 한 번 뒤집는다.
            let (w, h) = (img.width as usize, img.height as usize);
            let mut flipped = img.clone();
            {
                let src = img.get_image_data();
                let dst = flipped.get_image_data_mut();
                for y in 0..h {
                    for x in 0..w {
                        // 투명한 곳은 분홍으로 보이게 해서 잘 보이도록.
                        let p = src[y * w + x];
                        dst[(h - 1 - y) * w + x] = if p[3] == 0 { [255, 0, 255, 255] } else { p };
                    }
                }
            }
            flipped.export_png(&format!("{dir}/{}.png", kind.name()));
        }
    }
}
