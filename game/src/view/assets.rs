//! 그림·글꼴 불러오기.
//!
//! 캐릭터 그림은 아이 원화(`art/concept/게임 케릭터.png`)를 그대로 쓴다.
//! 원화 한 장(547×428)에 세 캐릭터가 같이 있어서, 게임이 시작할 때
//! 1) 캐릭터별 네모로 잘라 내고 2) 바깥 종이 배경을 투명하게 만든다.

use super::english::{picture_files, picture_path, EnglishArt};
use super::sky::SkyArt;
use super::title::TitleArt;
use crate::logic::character::CharacterKind;
use crate::logic::skeleton::SkeletonStage;
use crate::logic::weapon::Weapon;
use macroquad::prelude::*;
use std::collections::VecDeque;

pub const FONT_PATH: &str = "assets/fonts/Jua-Regular.ttf";
pub const CHARACTER_SHEET_PATH: &str = "assets/sprites/characters.png";
/// 좀비 카스테라 픽셀 그림 (D19, D23, D29). 맨 아래 한 줄이 비어 있어서 잘라 내면 그림 아래 끝 = 발이다.
pub const ZOMBIE_PATH: &str = "assets/sprites/zombie-round.png";
const ZOMBIE_EMPTY_ROWS_BELOW: u16 = 1;
/// 스켈레톤 그림 (D23, D35). `SkeletonStage` 순서 (일반, 절반, 완전체). 크기가 제각각이라 여백을 잘라 내고 쓴다.
pub const SKELETON_PATHS: [&str; 3] = ["assets/sprites/skeleton-normal.png", "assets/sprites/skeleton-half.png", "assets/sprites/skeleton-full.png"];
/// M6 그림 (헤일리, `art/spritecook/`).
pub const MISS_PATH: &str = "assets/sprites/miss.png";
pub const MISS_CITIZEN_BOMB_PATH: &str = "assets/sprites/miss-citizen-bomb.png";
pub const WEAPON_SWORD_PATH: &str = "assets/sprites/weapon-sword.png";
pub const WEAPON_AXE_PATH: &str = "assets/sprites/weapon-axe.png";
pub const WEAPON_GUN_PATH: &str = "assets/sprites/weapon-gun.png";
/// 새 아이템 그림 4개 (헤일리, `art/spritecook/items/`). 옛 `assets/sprites/item-*.png`는 쓰지 않는다.
/// `ItemPicture::ALL` 순서.
pub const ITEM_PATHS: [&str; 4] = [
    "assets/sprites/items/item-diamond-pickaxe.png",
    "assets/sprites/items/item-syringe-gun.png",
    "assets/sprites/items/item-ender-fruit.png",
    "assets/sprites/items/item-car.png",
];

/// 새 아이템 그림 (제작대 버튼, 정보판, 맵의 자동차·곡괭이). `view::items`가 그린다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemPicture {
    DiamondPickaxe,
    SyringeGun,
    EnderFruit,
    Car,
}

#[allow(dead_code)]
impl ItemPicture {
    pub const ALL: [ItemPicture; 4] = [ItemPicture::DiamondPickaxe, ItemPicture::SyringeGun, ItemPicture::EnderFruit, ItemPicture::Car];
}

/// 원화에서 캐릭터별로 잘라 낼 네모 (x, y, 가로, 세로), 원화 픽셀 기준.
/// 위쪽 손글씨 이름은 빼고, 머리 위 안테나(T자 선, docs/04-scenario.md)와 모자·머리·왕관은 넣었다.
/// 아래쪽은 발끝에 딱 맞춰 잘라서, 그림 아래 끝 = 발 위치가 된다.
pub const NIRO_CUT: (u32, u32, u32, u32) = (34, 147, 182, 265);
pub const ELLIE_CUT: (u32, u32, u32, u32) = (232, 150, 132, 230);
pub const LOGAN_CUT: (u32, u32, u32, u32) = (382, 115, 130, 255);

/// 안테나를 뺀 몸(모자·머리·왕관) 맨 위 y, 원화 픽셀 기준. 안테나가 있어도 몸 크기가 그대로이게 키를 늘릴 때 쓴다.
const NIRO_BODY_TOP: u32 = 176;
const ELLIE_BODY_TOP: u32 = 199;
const LOGAN_BODY_TOP: u32 = 163;

/// 머리와 몸을 나누는 y (옷 위 끝쯤), 원화 픽셀 기준. 예전에는 쓰러질 때 머리만 튀어 올랐다 (D30).
/// 부모님 결정 2026-11-15로 머리는 붙어 있고, 지금은 정보판 얼굴 자리 시험에만 쓴다.
#[cfg(test)]
const NIRO_NECK: u32 = 262;
#[cfg(test)]
const ELLIE_NECK: u32 = 268;
#[cfg(test)]
const LOGAN_NECK: u32 = 256;

/// 자르는 네모 안에 걸린 이름 글자 조각 (x, y, 가로, 세로), 원화 픽셀 기준. 종이색으로 덮는다.
const ELLIE_NAME_BITS: &[(u32, u32, u32, u32)] = &[(232, 150, 35, 14), (296, 150, 10, 5)];
const LOGAN_NAME_BITS: &[(u32, u32, u32, u32)] = &[(468, 115, 7, 4)];
/// 이름 조각을 덮는 색. [`is_paper`]가 종이로 보는 색이라 배경과 함께 투명해진다.
const COVER_PAPER_COLOR: [u8; 4] = [200, 200, 195, 255];

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

/// 정보판 얼굴 (US-091): 원화에서 모자·머리·왕관과 얼굴만 담은 네모 (x, y, 가로, 세로), 원화 픽셀 기준.
/// 안테나와 몸은 뺀다. 캐릭터 그림(`character_cut`으로 자른 것) 안에 들어 있어야 한다.
pub const NIRO_FACE: (u32, u32, u32, u32) = (70, 188, 110, 80);
pub const ELLIE_FACE: (u32, u32, u32, u32) = (236, 204, 100, 76);
pub const LOGAN_FACE: (u32, u32, u32, u32) = (418, 168, 92, 96);

/// 잘라 낸 캐릭터 그림 안에서 얼굴 자리 (그림 픽셀). `draw_texture_ex`의 `source`로 쓴다.
pub fn face_source(kind: CharacterKind) -> Rect {
    let (cx, cy, _, _) = character_cut(kind);
    let (x, y, w, h) = match kind {
        CharacterKind::Niro => NIRO_FACE,
        CharacterKind::Ellie => ELLIE_FACE,
        CharacterKind::Logan => LOGAN_FACE,
    };
    Rect::new((x - cx) as f32, (y - cy) as f32, w as f32, h as f32)
}

pub fn character_cut(kind: CharacterKind) -> (u32, u32, u32, u32) {
    match kind {
        CharacterKind::Niro => NIRO_CUT,
        CharacterKind::Ellie => ELLIE_CUT,
        CharacterKind::Logan => LOGAN_CUT,
    }
}

/// 몸 높이가 `body_height`일 때, 안테나까지 넣은 그림 높이.
pub fn height_with_antenna(kind: CharacterKind, body_height: f32) -> f32 {
    let (_, y, _, h) = character_cut(kind);
    let body_top = match kind {
        CharacterKind::Niro => NIRO_BODY_TOP,
        CharacterKind::Ellie => ELLIE_BODY_TOP,
        CharacterKind::Logan => LOGAN_BODY_TOP,
    };
    body_height * h as f32 / (y + h - body_top) as f32
}

/// 잘라 낸 그림에서 머리 쪽이 차지하는 비율 (위에서부터, 0~1).
#[cfg(test)]
pub fn head_share(kind: CharacterKind) -> f32 {
    let (_, y, _, h) = character_cut(kind);
    let neck = match kind {
        CharacterKind::Niro => NIRO_NECK,
        CharacterKind::Ellie => ELLIE_NECK,
        CharacterKind::Logan => LOGAN_NECK,
    };
    (neck - y) as f32 / h as f32
}

fn name_bits(kind: CharacterKind) -> &'static [(u32, u32, u32, u32)] {
    match kind {
        CharacterKind::Niro => &[],
        CharacterKind::Ellie => ELLIE_NAME_BITS,
        CharacterKind::Logan => LOGAN_NAME_BITS,
    }
}

pub struct Assets {
    pub font: Font,
    /// 자모별 색 타이틀 제목
    pub title: TitleArt,
    /// `CharacterKind::ALL` 순서 (Niro, Ellie, Logan).
    characters: [Texture2D; 3],
    pub zombie: Texture2D,
    /// 좀비와 같은 모양의 흰 그림. 색을 곱해 맞은 좀비를 번쩍이게 한다.
    pub zombie_flash: Texture2D,
    /// `SkeletonStage` 순서로 (그림, 같은 모양의 흰 그림).
    skeletons: Vec<(Texture2D, Texture2D)>,
    /// 코드로 만든 하늘 그림 (해·달·구름·빛). 시작할 때 한 번만 만든다.
    pub sky: SkyArt,
    pub m6: M6Art,
    /// 영어 퀴즈 그림 (US-088, config 낱말 표의 그림). 없는 그림은 빠져 있다.
    pub english: EnglishArt,
    /// 새 기록 폭죽 그림 (US-132)
    pub fireworks: FireworkArt,
}

/// 미스·시민·무기 그림. 둘레 여백을 잘라서 그림 아래 끝 = 발이다.
pub struct M6Art {
    pub miss: Texture2D,
    pub miss_citizen_bomb: Texture2D,
    pub weapon_sword: Texture2D,
    pub weapon_axe: Texture2D,
    pub weapon_gun: Texture2D,
    /// `ItemPicture::ALL` 순서
    items: Vec<Texture2D>,
}

impl M6Art {
    /// 새 아이템 그림.
    pub fn item(&self, picture: ItemPicture) -> &Texture2D {
        &self.items[picture as usize]
    }

    /// 무기 그림. 맨손은 없다.
    pub fn weapon(&self, kind: Weapon) -> Option<&Texture2D> {
        match kind {
            Weapon::Hands => None,
            Weapon::Sword => Some(&self.weapon_sword),
            Weapon::Axe => Some(&self.weapon_axe),
            Weapon::Gun => Some(&self.weapon_gun),
        }
    }

    /// M6 그림 파일 경로들 (순서: 미스, 시민 폭탄 미스, 칼, 도끼, 총, 아이템들).
    fn paths() -> Vec<String> {
        [MISS_PATH, MISS_CITIZEN_BOMB_PATH, WEAPON_SWORD_PATH, WEAPON_AXE_PATH, WEAPON_GUN_PATH]
            .into_iter()
            .chain(ITEM_PATHS)
            .map(String::from)
            .collect()
    }

    /// `paths()` 차례로 미리 받아 둔 파일 내용으로 그림을 만든다.
    fn from_files(files: Vec<FileResult>) -> M6Art {
        // 미스·시민은 손그림이라 부드럽게, 무기는 픽셀 그림이라 또렷하게 키운다.
        let mut files = files.into_iter();
        let mut next = |filter: FilterMode| {
            let bytes = files.next().and_then(Result::ok).expect("M6 그림을 불러오지 못했어요");
            let texture = Texture2D::from_image(&cut_to_ink(&bytes));
            texture.set_filter(filter);
            texture
        };
        M6Art {
            miss: next(FilterMode::Linear),
            miss_citizen_bomb: next(FilterMode::Linear),
            weapon_sword: next(FilterMode::Nearest),
            weapon_axe: next(FilterMode::Nearest),
            weapon_gun: next(FilterMode::Nearest),
            items: ITEM_PATHS.iter().map(|_| next(FilterMode::Nearest)).collect(),
        }
    }
}

/// 새 기록 폭죽 그림 9장 (`config::FIREWORK_FILES` 차례). 못 받은 그림은 비어 있고, 그 폭죽만 안 그린다.
pub struct FireworkArt {
    pictures: Vec<Option<Texture2D>>,
}

impl FireworkArt {
    fn paths() -> Vec<String> {
        crate::config::FIREWORK_FILES.iter().map(|name| format!("{}/{}", crate::config::FIREWORK_DIR, name)).collect()
    }

    fn from_files(files: Vec<FileResult>) -> FireworkArt {
        let pictures = files
            .into_iter()
            .map(|file| {
                let image = Image::from_file_with_format(&file.ok()?, Some(ImageFormat::Png)).ok()?;
                let texture = Texture2D::from_image(&image);
                texture.set_filter(FilterMode::Linear); // 부드럽게 줄인 그림이라 부드럽게 키운다
                Some(texture)
            })
            .collect();
        FireworkArt { pictures }
    }

    pub fn picture(&self, index: usize) -> Option<&Texture2D> {
        self.pictures.get(index)?.as_ref()
    }
}

/// 파일 하나를 받은 결과.
pub type FileResult = Result<Vec<u8>, macroquad::Error>;

/// 파일을 모두 동시에 요청하고 다 올 때까지 기다린다. 결과는 `paths` 차례 그대로 (실패도 그 자리에).
/// 한 장씩 차례로 받으면 서버 왕복이 쌓여 폰에서 오래 걸린다.
pub async fn fetch_all(paths: &[String]) -> Vec<FileResult> {
    join_in_order(paths.iter().map(|p| Box::pin(load_file(p))).collect()).await
}

/// 미래(future)들을 모두 같이 진행시키고, 다 끝나면 넣은 차례대로 결과를 돌려준다.
async fn join_in_order<F: std::future::Future + Unpin>(mut pending: Vec<F>) -> Vec<F::Output> {
    use std::pin::Pin;
    use std::task::Poll;
    let mut done: Vec<Option<F::Output>> = pending.iter().map(|_| None).collect();
    std::future::poll_fn(|cx| {
        let mut left = 0;
        for (future, result) in pending.iter_mut().zip(done.iter_mut()) {
            if result.is_none() {
                match Pin::new(future).poll(cx) {
                    Poll::Ready(value) => *result = Some(value),
                    Poll::Pending => left += 1,
                }
            }
        }
        if left == 0 { Poll::Ready(()) } else { Poll::Pending }
    })
    .await;
    done.into_iter().map(|r| r.expect("모든 요청이 끝났어요")).collect()
}

impl Assets {
    /// 글꼴만 먼저 불러온다 (US-109: 로고는 글꼴만 있으면 그릴 수 있다). (글꼴, 파일 내용)
    pub async fn load_font() -> (Font, Vec<u8>) {
        let font_bytes = load_file(FONT_PATH)
            .await
            .expect("한글 글꼴(Jua)을 불러오지 못했어요");
        let font = load_ttf_font_from_bytes(&font_bytes).expect("한글 글꼴(Jua)을 읽지 못했어요");
        (font, font_bytes)
    }

    /// 글꼴을 뺀 나머지 그림을 불러온다. 로고가 나오는 동안 같이 불러온다 (US-109).
    pub async fn load_with_font(font: Font, font_bytes: Vec<u8>) -> Assets {
        let title = TitleArt::new(&font_bytes);
        // 모든 그림 파일을 한꺼번에 요청한다: 캐릭터, 좀비, 스켈레톤 3, M6 그림, 영어 그림.
        let m6_paths = M6Art::paths();
        let english_names: Vec<&'static str> = picture_files().collect();
        let paths: Vec<String> = [CHARACTER_SHEET_PATH, ZOMBIE_PATH]
            .into_iter()
            .chain(SKELETON_PATHS)
            .map(String::from)
            .chain(m6_paths.iter().cloned())
            .chain(english_names.iter().map(|name| picture_path(name)))
            .chain(FireworkArt::paths())
            .collect();
        let mut files = fetch_all(&paths).await;
        let firework_files = files.split_off(2 + SKELETON_PATHS.len() + m6_paths.len() + english_names.len());
        let english_files = files.split_off(2 + SKELETON_PATHS.len() + m6_paths.len());
        let m6_files = files.split_off(2 + SKELETON_PATHS.len());
        let skeleton_files = files.split_off(2);
        let mut head = files.into_iter();
        let bytes = head.next().and_then(Result::ok).expect("캐릭터 그림을 불러오지 못했어요");
        let sheet = Image::from_file_with_format(&bytes, Some(ImageFormat::Png))
            .expect("캐릭터 그림(PNG)을 읽지 못했어요");
        let characters = CharacterKind::ALL.map(|kind| {
            let texture = Texture2D::from_image(&cut_character(&sheet, kind));
            texture.set_filter(FilterMode::Linear);
            texture
        });
        let zombie_bytes = head.next().and_then(Result::ok).expect("좀비 그림을 불러오지 못했어요");
        let zombie_image = cut_zombie(&zombie_bytes);
        let pixel_texture = |image: &Image| {
            let texture = Texture2D::from_image(image);
            texture.set_filter(FilterMode::Nearest);
            texture
        };
        let zombie = pixel_texture(&zombie_image);
        let zombie_flash = pixel_texture(&white_silhouette(&zombie_image));
        let mut skeletons = Vec::new();
        for file in skeleton_files {
            let bytes = file.expect("스켈레톤 그림을 불러오지 못했어요");
            let image = cut_to_ink(&bytes);
            skeletons.push((pixel_texture(&image), pixel_texture(&white_silhouette(&image))));
        }
        Assets {
            font,
            title,
            characters,
            zombie,
            zombie_flash,
            skeletons,
            sky: SkyArt::new(),
            m6: M6Art::from_files(m6_files),
            english: EnglishArt::from_files(&english_names, english_files),
            fireworks: FireworkArt::from_files(firework_files),
        }
    }

    pub fn character(&self, kind: CharacterKind) -> &Texture2D {
        let index = CharacterKind::ALL.iter().position(|k| *k == kind).unwrap_or(0);
        &self.characters[index]
    }

    /// 단계에 맞는 (그림, 번쩍일 때 덧그리는 흰 그림).
    pub fn skeleton(&self, stage: SkeletonStage) -> (&Texture2D, &Texture2D) {
        let (picture, flash) = &self.skeletons[stage as usize];
        (picture, flash)
    }
}

/// 좀비 PNG를 읽고 발 아래 빈 줄을 잘라 낸다.
fn cut_zombie(png: &[u8]) -> Image {
    let image = Image::from_file_with_format(png, Some(ImageFormat::Png)).expect("좀비 그림(PNG)을 읽지 못했어요");
    image.sub_image(Rect::new(0.0, 0.0, image.width as f32, (image.height - ZOMBIE_EMPTY_ROWS_BELOW) as f32))
}

/// PNG를 읽고 둘레의 투명 여백을 잘라 낸다 → 그림 아래 끝 = 발, 그림 키 = 몸 키.
fn cut_to_ink(png: &[u8]) -> Image {
    let image = Image::from_file_with_format(png, Some(ImageFormat::Png)).expect("그림(PNG)을 읽지 못했어요");
    let w = image.width as usize;
    let inked = || image.get_image_data().iter().enumerate().filter(|(_, p)| p[3] > 0).map(|(i, _)| (i % w, i / w));
    let (left, right) = (inked().map(|p| p.0).min().unwrap_or(0), inked().map(|p| p.0).max().unwrap_or(0));
    let (top, bottom) = (inked().map(|p| p.1).min().unwrap_or(0), inked().map(|p| p.1).max().unwrap_or(0));
    image.sub_image(Rect::new(left as f32, top as f32, (right - left + 1) as f32, (bottom - top + 1) as f32))
}

/// 모양(투명도)은 그대로 두고 색만 하얗게.
fn white_silhouette(image: &Image) -> Image {
    let mut white = image.clone();
    for px in white.get_image_data_mut() {
        *px = [255, 255, 255, px[3]];
    }
    white
}

/// 원화에서 한 캐릭터를 잘라 내고 배경을 투명하게 만든다.
pub fn cut_character(sheet: &Image, kind: CharacterKind) -> Image {
    let (x, y, w, h) = character_cut(kind);
    let mut image = sheet.sub_image(Rect::new(x as f32, y as f32, w as f32, h as f32));
    let width = w as usize;
    let data = image.get_image_data_mut();
    for &(bx, by, bw, bh) in name_bits(kind) {
        for py in by - y..by - y + bh {
            for px in bx - x..bx - x + bw {
                data[py as usize * width + px as usize] = COVER_PAPER_COLOR;
            }
        }
    }
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
    #[test]
    fn join_in_order_starts_all_together_and_keeps_order() {
        use std::future::Future;
        use std::pin::Pin;
        use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
        // 처음 몇 번은 Pending, 그 뒤에 값을 주는 미래. 첫 poll 때 시작 표시를 남긴다.
        struct Slow { waits: u32, value: u32, started: std::rc::Rc<std::cell::Cell<u32>>, first: bool }
        impl Future for Slow {
            type Output = u32;
            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u32> {
                if self.first {
                    self.first = false;
                    self.started.set(self.started.get() + 1);
                }
                if self.waits == 0 { return Poll::Ready(self.value) }
                self.waits -= 1;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
        fn noop_raw() -> RawWaker {
            fn clone(_: *const ()) -> RawWaker { noop_raw() }
            fn noop(_: *const ()) {}
            static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, noop, noop, noop);
            RawWaker::new(std::ptr::null(), &VTABLE)
        }
        let started = std::rc::Rc::new(std::cell::Cell::new(0));
        let make = |waits, value| Slow { waits, value, started: started.clone(), first: true };
        let mut future = Box::pin(super::join_in_order(vec![make(3, 10), make(0, 20), make(1, 30)]));
        let waker = unsafe { Waker::from_raw(noop_raw()) };
        let mut cx = Context::from_waker(&waker);
        assert!(future.as_mut().poll(&mut cx).is_pending());
        assert_eq!(started.get(), 3, "첫 번째 확인에서 세 요청이 모두 시작돼야 한다");
        let mut result = None;
        for _ in 0..10 {
            if let Poll::Ready(v) = future.as_mut().poll(&mut cx) { result = Some(v); break }
        }
        assert_eq!(result, Some(vec![10, 20, 30]), "결과는 넣은 차례 그대로");
    }

    use super::*;

    fn sheet() -> Image {
        let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/", "assets/sprites/characters.png"))
            .expect("characters.png");
        Image::from_file_with_format(&bytes, Some(ImageFormat::Png)).unwrap()
    }

    #[test]
    fn face_is_inside_the_character_picture_and_below_the_antenna() {
        // US-091 정보판 얼굴: 잘라 낸 캐릭터 그림 안, 몸 맨 위(안테나 아래)쯤부터 목 근처까지
        for kind in CharacterKind::ALL {
            let (_, _, w, h) = character_cut(kind);
            let f = face_source(kind);
            assert!(f.x >= 0.0 && f.y >= 0.0 && f.right() <= w as f32 && f.bottom() <= h as f32, "{kind:?} {f:?}");
            assert!(f.w >= 60.0 && f.h >= 60.0, "{kind:?} 얼굴이 너무 작다");
            let neck = head_share(kind) * h as f32;
            assert!(f.bottom() <= neck + 12.0, "{kind:?} 몸까지 많이 들어가지 않는다");
        }
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

    /// 몸 위쪽 줄에서 안테나가 있는 칸 (잘라 낸 그림의 x 범위).
    fn antenna_columns(kind: CharacterKind) -> std::ops::RangeInclusive<usize> {
        match kind {
            // Niro는 안테나(x 67~113) 오른쪽 위에 종이 얼룩 점이 하나 있어서 그 칸까지 넣었다.
            CharacterKind::Niro => 67..=147,
            CharacterKind::Ellie => 53..=91,
            CharacterKind::Logan => 80..=113,
        }
    }

    #[test]
    fn only_antenna_is_left_above_the_body() {
        // REVIEW-005 2번: 덮은 네모가 아니라 몸 위쪽 줄 전체를 봐서, 네모 밖에 남은 이름 조각도 잡는다.
        let s = sheet();
        for kind in CharacterKind::ALL {
            let (_, y, ..) = character_cut(kind);
            let body_top = match kind {
                CharacterKind::Niro => NIRO_BODY_TOP,
                CharacterKind::Ellie => ELLIE_BODY_TOP,
                CharacterKind::Logan => LOGAN_BODY_TOP,
            };
            let img = cut_character(&s, kind);
            let w = img.width as usize;
            let data = img.get_image_data();
            let columns = antenna_columns(kind);
            let inked: Vec<usize> = (0..(body_top - y) as usize * w).filter(|i| data[*i][3] > 0).map(|i| i % w).collect();
            assert!(inked.iter().all(|x| columns.contains(x)), "{kind:?} 안테나 밖에 잉크가 있다");
        }
    }

    #[test]
    fn antenna_stays_joined_to_body_and_name_is_gone() {
        let s = sheet();
        for kind in CharacterKind::ALL {
            let (x, y, ..) = character_cut(kind);
            let img = cut_character(&s, kind);
            let data = img.get_image_data();
            let (w, h) = (img.width as usize, img.height as usize);
            let row_has_ink = |row: usize| data[row * w..(row + 1) * w].iter().any(|p| p[3] > 0);
            // 안테나 위 끝이 위쪽에 있고, 거기서 발까지 끊긴 줄이 없다 (연한 선이 종이로 지워지지 않았다)
            let top = (0..h).find(|&row| row_has_ink(row)).unwrap();
            assert!(top < 10, "{kind:?} 안테나 맨 위 {top}");
            let bottom = (0..h).rfind(|&row| row_has_ink(row)).unwrap();
            assert!((top..bottom).all(row_has_ink), "{kind:?} 안테나가 끊겼다");
            for &(bx, by, bw, bh) in name_bits(kind) {
                for py in by - y..by - y + bh {
                    for px in bx - x..bx - x + bw {
                        assert_eq!(data[py as usize * w + px as usize][3], 0, "{kind:?} 이름이 남았다");
                    }
                }
            }
            assert!(height_with_antenna(kind, 84.0) > 84.0);
        }
    }

    #[test]
    fn zombie_feet_are_at_the_bottom_of_the_picture() {
        let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/", "assets/sprites/zombie-round.png")).expect("zombie-round.png");
        let img = cut_zombie(&bytes);
        let (w, h) = (img.width as usize, img.height as usize);
        assert!(img.get_image_data()[(h - 1) * w..].iter().any(|p| p[3] > 0), "맨 아래 줄에 발이 있다");
        let white = white_silhouette(&img);
        for (a, b) in img.get_image_data().iter().zip(white.get_image_data()) {
            assert_eq!((a[3], b[0]), (b[3], 255));
        }
    }

    #[test]
    fn cut_pictures_have_no_empty_edges() {
        for path in SKELETON_PATHS.into_iter().chain([MISS_PATH, MISS_CITIZEN_BOMB_PATH, WEAPON_SWORD_PATH, WEAPON_AXE_PATH, WEAPON_GUN_PATH]).chain(ITEM_PATHS) {
            let bytes = std::fs::read(format!("{}/{path}", env!("CARGO_MANIFEST_DIR"))).expect("그림");
            let img = cut_to_ink(&bytes);
            let (w, h) = (img.width as usize, img.height as usize);
            println!("{path}: {w}x{h}");
            let data = img.get_image_data();
            assert!(data[(h - 1) * w..].iter().any(|p| p[3] > 0), "{path} 맨 아래 줄에 발이 있다");
            assert!(data[..w].iter().any(|p| p[3] > 0), "{path} 맨 위");
            assert!((0..h).any(|y| data[y * w][3] > 0) && (0..h).any(|y| data[y * w + w - 1][3] > 0), "{path} 양옆");
        }
    }

    #[test]
    fn every_item_picture_has_a_file_in_the_new_folder() {
        assert_eq!(ITEM_PATHS.len(), ItemPicture::ALL.len());
        for (i, picture) in ItemPicture::ALL.iter().enumerate() {
            assert_eq!(*picture as usize, i, "그림 순서와 파일 순서가 같다");
            assert!(ITEM_PATHS[i].starts_with("assets/sprites/items/"), "옛 item-*.png가 아니라 새 폴더 그림");
        }
    }

    #[test]
    fn head_and_body_both_have_ink() {
        let s = sheet();
        for kind in CharacterKind::ALL {
            let img = cut_character(&s, kind);
            let (w, h) = (img.width as usize, img.height as usize);
            let neck = (head_share(kind) * h as f32) as usize;
            assert!(neck > h / 4 && neck < h * 3 / 4, "{kind:?} 목 {neck}/{h}");
            let inked = |rows: std::ops::Range<usize>| img.get_image_data()[rows.start * w..rows.end * w].iter().any(|p| p[3] > 0);
            assert!(inked(0..neck) && inked(neck..h), "{kind:?}");
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
            // 정보판 얼굴 자리 (US-091). 이것도 위아래를 뒤집어 저장한다.
            let f = face_source(kind);
            let face = flipped.sub_image(Rect::new(f.x, h as f32 - f.y - f.h, f.w, f.h));
            face.export_png(&format!("{dir}/{}-face.png", kind.name()));
        }
    }
}
