//! 상점 화면 (US-062, US-070, US-071, US-072, 로건): 집 안 느낌의 밝은 판에 물건 5개(칼·도끼·총은 부모님 결정으로 뺐다), 금·은·동 팔기, 지금 코인, "닫기".
//! 마우스 클릭과 손가락 탭이 같은 `shop_tap_at` → `apply_shop_tap` 한 길로 간다. Esc는 main이 `close_shop`으로 닫는다.
//! 살 수 있는지, 값, 이유 글은 모두 logic(`buy_rows`, `sell_rows`)이 준 그대로 보여 준다.
//! 산 물건 그림(포션·스케이트·갑옷)과 쓰는 물건(`Pocket`)도 여기 있다. 정보판 물건 칸은 view::hud, 가방은 view::bag이 그린다.
//! 포션·주사기총·엔더열매는 손에 들지 않는다 (부모님 결정): PC는 1·2·R·3 키, 폰은 정보판의 그림을 누르면 바로 쓴다 (`hud::pocket_tap_rects`).

use super::assets::{ItemPicture, M6Art};
use super::items::draw_item_picture;
use super::coin::draw_coin;
use super::gather::draw_material_icon;
use super::hud::{draw_heart, draw_pill};
use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::text::TextPainter;
use super::touch::draw_star;
use crate::config::{SHOP_BOUGHT_TEXT, SHOP_HINT_TEXT, SHOP_SOLD_TEXT};
use crate::logic::game::Game;
use crate::logic::gathering::Material;
use crate::logic::shop::{Armor, Potion, ShopItem, SELLABLE};
use crate::logic::sound::{sfx_for_buy, sfx_for_sell, Sfx};
use macroquad::prelude::*;

// ---- 자리 (가상 960×540 안) ----
const PANEL: Rect = Rect::new(20.0, 14.0, 920.0, 512.0);
const TITLE_BASELINE: f32 = 60.0;
/// 팔기 줄은 네 칸: 왼쪽 끝, 폭, 사이
const COLUMN_LEFT: f32 = 36.0;
const CARD_W: f32 = 212.0;
const CARD_GAP: f32 = 10.0;
/// 사기 칸 한 줄에 다섯 개 (칼·도끼·총을 뺀 뒤). 키가 커서 이름을 두 줄로 쓸 수 있다.
const BUY_TOP: f32 = 82.0;
const BUY_H: f32 = 230.0;
const BUY_W: f32 = 168.0;
const BUY_GAP: f32 = 12.0;
/// 팔기 칸 한 줄 (금·은·동) + 닫기
const SELL_TOP: f32 = 340.0;
const SELL_H: f32 = 136.0;
pub const CLOSE: Rect = Rect::new(COLUMN_LEFT + (CARD_W + CARD_GAP) * 3.0, 356.0, CARD_W, 104.0);

// ---- 색: 밝은 집 안 (크림색 벽지 + 나무 테두리) ----
const DIM: Color = Color::new(0.0, 0.0, 0.0, 0.35);
const WALL: Color = Color::new(1.0, 0.95, 0.84, 0.98);
const WALL_STRIPE: Color = Color::new(0.99, 0.89, 0.74, 0.98);
const WOOD: Color = Color::new(0.62, 0.40, 0.20, 1.0);
const ROOF: Color = Color::new(0.90, 0.36, 0.30, 1.0);
const CARD_FILL: Color = Color::new(1.0, 1.0, 1.0, 0.92);
const CARD_LINE: Color = Color::new(0.95, 0.62, 0.15, 1.0);
const CARD_OFF_FILL: Color = Color::new(0.86, 0.83, 0.79, 0.9);
const CARD_OFF_LINE: Color = Color::new(0.62, 0.58, 0.54, 1.0);
/// 못 사는 칸 위에 덮어서 흐리게
const FADE: Color = Color::new(0.93, 0.90, 0.85, 0.6);
const REASON: Color = Color::new(1.0, 0.62, 0.22, 1.0);
const HAPPY: Color = Color::new(1.0, 0.86, 0.22, 1.0);
const CLOSE_FILL: Color = Color::new(0.30, 0.62, 0.90, 1.0);

// ---- 글자 크기 ----
const TITLE_SIZE: f32 = 46.0;
const NAME_SIZE: f32 = 30.0;
const PRICE_SIZE: f32 = 32.0;
const REASON_SIZE: f32 = 25.0;
const CLOSE_SIZE: f32 = 46.0;

/// 사거나 판 뒤 칸이 통통 튀고 위에 글이 보이는 초
const FEEDBACK_SECONDS: f32 = 1.2;

/// 상점 화면에서 누른 곳.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopTap {
    Buy(ShopItem),
    Sell(Material),
    Close,
}

pub fn buy_rect(index: usize) -> Rect {
    Rect::new(COLUMN_LEFT + index as f32 * (BUY_W + BUY_GAP), BUY_TOP, BUY_W, BUY_H)
}

/// 이름이 칸 폭에 안 들어가면 띄어쓰기에서 두 줄로 나눈다 ("다이아몬드 갑옷" → "다이아몬드" / "갑옷").
fn name_lines(text: &TextPainter, name: &str, size: f32, max_w: f32) -> Vec<String> {
    match name.rsplit_once(' ') {
        Some((a, b)) if text.width(name, size) > max_w => vec![a.to_string(), b.to_string()],
        _ => vec![name.to_string()],
    }
}

pub fn sell_rect(index: usize) -> Rect {
    Rect::new(COLUMN_LEFT + index as f32 * (CARD_W + CARD_GAP), SELL_TOP, CARD_W, SELL_H)
}

/// 누른 곳(가상 좌표)이 무엇인지. 판 밖이나 빈 곳은 None.
pub fn shop_tap_at(point: Vec2) -> Option<ShopTap> {
    if CLOSE.contains(point) {
        return Some(ShopTap::Close);
    }
    if let Some(i) = (0..ShopItem::ALL.len()).find(|i| buy_rect(*i).contains(point)) {
        return Some(ShopTap::Buy(ShopItem::ALL[i]));
    }
    (0..SELLABLE.len()).find(|i| sell_rect(*i).contains(point)).map(|i| ShopTap::Sell(SELLABLE[i]))
}

/// 화면 쪽 상점 상태: 막 사거나 판 칸과 지난 시간 (진짜 시간, 게임이 멈춰도 흐른다).
#[derive(Debug, Default)]
pub struct ShopView {
    feedback: Option<(ShopTap, f32)>,
    /// 누른 결과로 날 효과음 (사면 짤랑 …). `take_sfx`로 한 번 꺼낸다.
    sfx: Option<Sfx>,
}

impl ShopView {
    pub fn new() -> ShopView {
        ShopView::default()
    }

    pub fn update(&mut self, dt: f32) {
        self.feedback = self.feedback.map(|(tap, age)| (tap, age + dt)).filter(|(_, age)| *age < FEEDBACK_SECONDS);
    }

    /// 누른 결과로 날 효과음을 한 번 꺼낸다 (없으면 None).
    pub fn take_sfx(&mut self) -> Option<Sfx> {
        self.sfx.take()
    }

    /// 누른 것을 logic에 넘긴다. 사거나 팔았으면 true.
    pub fn apply(&mut self, game: &mut Game, tap: ShopTap) -> bool {
        let done = match tap {
            ShopTap::Buy(item) => {
                let result = game.buy(item);
                self.sfx = sfx_for_buy(&result);
                result.is_ok()
            }
            ShopTap::Sell(material) => {
                let result = game.sell(material);
                self.sfx = sfx_for_sell(&result);
                result.is_ok()
            }
            ShopTap::Close => {
                game.close_shop();
                self.feedback = None;
                false
            }
        };
        if done {
            self.feedback = Some((tap, 0.0));
        }
        done
    }

    /// 지금 보이는 기쁜 글 ("샀어요!", "팔았어요!").
    pub fn feedback_text(&self) -> Option<&'static str> {
        match self.feedback?.0 {
            ShopTap::Buy(_) => Some(SHOP_BOUGHT_TEXT),
            ShopTap::Sell(_) => Some(SHOP_SOLD_TEXT),
            ShopTap::Close => None,
        }
    }

    /// 막 사거나 판 칸이 통통 튀는 높이 (0이면 그대로).
    fn bounce(&self, tap: ShopTap) -> f32 {
        match self.feedback {
            Some((t, age)) if t == tap => 8.0 * (age / FEEDBACK_SECONDS * std::f32::consts::PI * 3.0).sin().abs() * (1.0 - age / FEEDBACK_SECONDS),
            _ => 0.0,
        }
    }
}

/// 글이 `max_w` 안에 들어가는 크기 (넘치면 줄인다).
fn fit_text(text: &TextPainter, words: &str, size: f32, max_w: f32) -> f32 {
    let w = text.width(words, size);
    if w > max_w {
        size * max_w / w
    } else {
        size
    }
}

// ---- 물건 그림 (임시, 코드로 그림) ----

/// 둥근 병: 몸통 동그라미 + 목 + 코르크. 체력 포션은 분홍빛 빨강에 하트, 독약 포션은 초록에 거품.
pub fn draw_potion(potion: Potion, c: Vec2, size: f32) {
    let (liquid, glass) = match potion {
        Potion::Health => (Color::new(0.96, 0.32, 0.45, 1.0), Color::new(1.0, 0.85, 0.88, 1.0)),
        Potion::Poison => (Color::new(0.35, 0.80, 0.30, 1.0), Color::new(0.85, 1.0, 0.82, 1.0)),
    };
    let r = size * 0.32;
    let body = c + vec2(0.0, size * 0.12);
    draw_rectangle(c.x - size * 0.1, c.y - size * 0.4, size * 0.2, size * 0.28, glass);
    draw_rectangle(c.x - size * 0.13, c.y - size * 0.48, size * 0.26, size * 0.1, Color::new(0.70, 0.48, 0.28, 1.0));
    draw_circle(body.x, body.y, r, glass);
    draw_circle(body.x, body.y + r * 0.12, r * 0.86, liquid);
    draw_circle_lines(body.x, body.y, r, 2.0, Color::new(0.25, 0.20, 0.35, 1.0));
    match potion {
        Potion::Health => draw_heart(body + vec2(0.0, r * 0.05), r * 0.9, WHITE),
        Potion::Poison => {
            for (dx, dy, k) in [(-0.3, 0.1, 0.18), (0.25, -0.15, 0.13), (0.05, 0.35, 0.1)] {
                draw_circle(body.x + dx * r, body.y + dy * r, k * r * 1.4, Color::new(0.85, 1.0, 0.7, 0.9));
            }
        }
    }
}

/// 스케이트: 하늘색 신발 + 바퀴 두 개.
pub fn draw_skates(c: Vec2, size: f32) {
    let boot = Color::new(0.45, 0.75, 1.0, 1.0);
    let line = Color::new(0.20, 0.25, 0.45, 1.0);
    draw_rectangle(c.x - size * 0.28, c.y - size * 0.4, size * 0.26, size * 0.5, boot);
    draw_rectangle(c.x - size * 0.28, c.y - size * 0.02, size * 0.62, size * 0.2, boot);
    draw_rectangle_lines(c.x - size * 0.28, c.y - size * 0.4, size * 0.26, size * 0.5, 2.0, line);
    draw_rectangle(c.x - size * 0.32, c.y + size * 0.18, size * 0.7, size * 0.06, line);
    for dx in [-0.18, 0.22] {
        draw_circle(c.x + dx * size, c.y + size * 0.32, size * 0.1, Color::new(1.0, 0.85, 0.25, 1.0));
        draw_circle_lines(c.x + dx * size, c.y + size * 0.32, size * 0.1, 2.0, line);
    }
}

/// 무지개 색 하나: `t`(0~1)를 무지개 고리로.
pub fn rainbow(t: f32) -> Color {
    let h = t.rem_euclid(1.0) * 6.0;
    let x = 1.0 - (h % 2.0 - 1.0).abs();
    let (r, g, b) = match h as u32 {
        0 => (1.0, x, 0.0),
        1 => (x, 1.0, 0.0),
        2 => (0.0, 1.0, x),
        3 => (0.0, x, 1.0),
        4 => (x, 0.0, 1.0),
        _ => (1.0, 0.0, x),
    };
    // 아이 눈에 편한 밝은 색으로
    Color::new(0.35 + 0.65 * r, 0.35 + 0.65 * g, 0.35 + 0.65 * b, 1.0)
}

/// 갑옷 (가슴판): 몸통 + 어깨. 무지개는 줄무늬, 다이아몬드는 하늘색에 반짝 별.
pub fn draw_armor(armor: Armor, c: Vec2, size: f32, t: f32) {
    let line = Color::new(0.20, 0.25, 0.45, 1.0);
    let body = Rect::new(c.x - size * 0.3, c.y - size * 0.3, size * 0.6, size * 0.66);
    let shoulders = Rect::new(c.x - size * 0.44, c.y - size * 0.36, size * 0.88, size * 0.2);
    match armor {
        Armor::Rainbow => {
            draw_rectangle(shoulders.x, shoulders.y, shoulders.w, shoulders.h, rainbow(t * 0.3));
            let stripes = 6;
            for i in 0..stripes {
                let h = body.h / stripes as f32;
                draw_rectangle(body.x, body.y + h * i as f32, body.w, h + 0.5, rainbow(i as f32 / stripes as f32 + t * 0.3));
            }
        }
        Armor::Diamond => {
            let ice = Color::new(0.60, 0.92, 1.0, 1.0);
            draw_rectangle(shoulders.x, shoulders.y, shoulders.w, shoulders.h, ice);
            draw_rectangle(body.x, body.y, body.w, body.h, ice);
            draw_triangle(vec2(body.x + 4.0, body.y + 4.0), vec2(body.center().x, body.y + 4.0), vec2(body.x + 4.0, body.center().y), Color::new(1.0, 1.0, 1.0, 0.6));
            let p = 0.5 + 0.5 * (t * 5.0).sin();
            draw_star(vec2(body.center().x + size * 0.1, body.center().y), size * (0.08 + 0.06 * p), t, WHITE);
        }
    }
    draw_rectangle_lines(shoulders.x, shoulders.y, shoulders.w, shoulders.h, 2.0, line);
    draw_rectangle_lines(body.x, body.y, body.w, body.h, 2.0, line);
    // 목 자리
    draw_triangle(vec2(c.x - size * 0.12, shoulders.y), vec2(c.x + size * 0.12, shoulders.y), vec2(c.x, shoulders.y + size * 0.16), line);
}

/// 상점 물건 그림.
pub fn draw_item_icon(item: ShopItem, c: Vec2, size: f32, t: f32) {
    match item {
        ShopItem::HealthPotion => draw_potion(Potion::Health, c, size),
        ShopItem::PoisonPotion => draw_potion(Potion::Poison, c, size),
        ShopItem::Skates => draw_skates(c, size),
        ShopItem::RainbowArmor => draw_armor(Armor::Rainbow, c, size, t),
        ShopItem::DiamondArmor => draw_armor(Armor::Diamond, c, size, t),
    }
}

/// 집 아이콘: 빨간 지붕 + 벽 + 문.
fn draw_house_icon(c: Vec2, size: f32) {
    draw_triangle(c + vec2(-size * 0.55, -size * 0.05), c + vec2(size * 0.55, -size * 0.05), c + vec2(0.0, -size * 0.5), ROOF);
    draw_rectangle(c.x - size * 0.4, c.y - size * 0.05, size * 0.8, size * 0.5, WALL_STRIPE);
    draw_rectangle_lines(c.x - size * 0.4, c.y - size * 0.05, size * 0.8, size * 0.5, 2.0, WOOD);
    draw_rectangle(c.x - size * 0.1, c.y + size * 0.15, size * 0.2, size * 0.3, WOOD);
}

fn draw_card(r: Rect, can: bool) {
    let (fill, line) = if can { (CARD_FILL, CARD_LINE) } else { (CARD_OFF_FILL, CARD_OFF_LINE) };
    draw_rectangle(r.x, r.y, r.w, r.h, fill);
    draw_rectangle_lines(r.x, r.y, r.w, r.h, if can { 5.0 } else { 3.0 }, line);
}

/// 코인 그림 + 숫자 (`prefix`는 "+" 같은 앞 글자).
fn draw_price(text: &TextPainter, prefix: &str, price: u32, coin: Vec2, size: f32, t: f32) {
    draw_coin(coin, size * 0.42, 0.0, t);
    text.left(&format!("{prefix}{price}"), coin.x + size * 0.55, coin.y + size * 0.34, size, WHITE);
}

/// 상점 화면 전체. 화면 좌표. `touch_mode`가 아니면 닫기 아래에 "Esc"를 작게 쓴다.
pub fn draw_shop(text: &TextPainter, game: &Game, view: &ShopView, touch_mode: bool) {
    let t = get_time() as f32;
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, DIM);
    // 벽지: 크림색에 세로 줄무늬, 둘레는 나무 테두리
    draw_rectangle(PANEL.x, PANEL.y, PANEL.w, PANEL.h, WALL);
    let mut x = PANEL.x + 20.0;
    while x < PANEL.right() - 10.0 {
        draw_rectangle(x, PANEL.y, 18.0, PANEL.h, WALL_STRIPE);
        x += 46.0;
    }
    draw_rectangle_lines(PANEL.x, PANEL.y, PANEL.w, PANEL.h, 8.0, WOOD);

    // 제목 줄: 집 + "상점", 가운데 안내(또는 "샀어요!"), 오른쪽 지금 코인
    draw_house_icon(vec2(COLUMN_LEFT + 26.0, 44.0), 48.0);
    text.left("상점", COLUMN_LEFT + 60.0, TITLE_BASELINE, TITLE_SIZE, HAPPY);
    match view.feedback_text() {
        Some(words) => text.centered(words, VIRTUAL_WIDTH / 2.0 - 20.0, TITLE_BASELINE, 40.0, HAPPY),
        None => text.centered(SHOP_HINT_TEXT,VIRTUAL_WIDTH / 2.0 - 20.0, TITLE_BASELINE - 4.0, 28.0, WHITE),
    }
    let coins = game.wallet.coins().to_string();
    let coin_w = 52.0 + text.width(&coins, 38.0) + 20.0;
    let coin_x = PANEL.right() - 16.0 - coin_w;
    draw_pill(coin_x, 22.0, coin_w, 50.0, Color::new(0.10, 0.13, 0.32, 0.7));
    draw_coin(vec2(coin_x + 28.0, 47.0), 15.0, 0.0, t);
    text.left(&coins, coin_x + 52.0, 60.0, 38.0, WHITE);

    // 사기 칸 5개 (한 줄): 큰 그림, 값, 이름(길면 두 줄), 못 사는 이유
    for (i, row) in game.buy_rows().iter().enumerate() {
        let mut r = buy_rect(i);
        r.y -= view.bounce(ShopTap::Buy(row.item));
        draw_card(r, row.can_buy);
        draw_item_icon(row.item, vec2(r.center().x, r.y + 58.0), 84.0, t);
        let price_w = PRICE_SIZE * 0.55 + text.width(&row.price.to_string(), PRICE_SIZE);
        draw_price(text, "", row.price, vec2(r.center().x - price_w / 2.0, r.y + 124.0), PRICE_SIZE, t);
        let lines = name_lines(text, row.name, NAME_SIZE, r.w - 16.0);
        for (k, line) in lines.iter().enumerate() {
            let y = if lines.len() == 1 { r.y + 180.0 } else { r.y + 166.0 + k as f32 * 30.0 };
            text.centered(line, r.center().x, y, fit_text(text, line, NAME_SIZE, r.w - 16.0), WHITE);
        }
        if let Some(reason) = row.reason {
            draw_rectangle(r.x + 2.0, r.y + 2.0, r.w - 4.0, r.h - 4.0, FADE);
            let words = reason.text();
            text.centered(words, r.center().x, r.y + 220.0, fit_text(text, words, REASON_SIZE, r.w - 12.0), REASON);
        }
    }

    // 팔기 칸: 금·은·동 (가진 개수, 받는 코인)
    for (i, row) in game.sell_rows().iter().enumerate() {
        let mut r = sell_rect(i);
        r.y -= view.bounce(ShopTap::Sell(row.material));
        draw_card(r, row.can_sell);
        draw_material_icon(row.material, vec2(r.x + 44.0, r.y + 44.0), 52.0);
        text.centered(&format!("{}개", row.count), r.x + 44.0, r.y + 98.0, 28.0, WHITE);
        let title = format!("{} 팔기", row.name);
        text.left(&title, r.x + 88.0, r.y + 46.0, fit_text(text, &title, NAME_SIZE, r.w - 96.0), WHITE);
        draw_price(text, "+", row.price, vec2(r.x + 102.0, r.y + 76.0), 28.0, t);
        if let Some(reason) = row.reason {
            draw_rectangle(r.x + 2.0, r.y + 2.0, r.w - 4.0, r.h - 4.0, FADE);
            let words = reason.text();
            text.centered(words, r.center().x, r.y + 126.0, fit_text(text, words, REASON_SIZE, r.w - 12.0), REASON);
        }
    }

    // 닫기: 크게
    draw_pill(CLOSE.x, CLOSE.y, CLOSE.w, CLOSE.h, CLOSE_FILL);
    let close_y = if touch_mode { CLOSE.center().y + 16.0 } else { CLOSE.center().y + 6.0 };
    text.centered("닫기", CLOSE.center().x, close_y, CLOSE_SIZE, WHITE);
    if !touch_mode {
        text.centered("Esc", CLOSE.center().x, CLOSE.bottom() - 10.0, 24.0, WHITE);
    }
}

// ---- 쓰는 물건 (정보판 물건 칸·가방, view::hud·view::bag이 그린다) ----
/// 손에 들지 않고 바로 쓰는 물건 (부모님 결정): 포션 둘, 주사기총, 엔더열매.
/// PC는 키(`input::pocket_key_label`), 폰은 표시판의 그림을 누른다. 쓸 수 있는지는 logic이 정한다 (`Game::why_cant_use`).
pub use crate::logic::shop::Pocket;

impl Pocket {
    /// 가진 개수 (`Game::pocket_count`). 0이면 표시판에 안 나온다.
    pub fn count(self, game: &Game) -> u32 {
        game.pocket_count(self)
    }
}

/// 그림 하나 (표시판·안내).
pub fn draw_pocket(pocket: Pocket, art: &M6Art, c: Vec2, size: f32) {
    match pocket {
        Pocket::Potion(potion) => draw_potion(potion, c, size),
        Pocket::SyringeGun => draw_item_picture(art, ItemPicture::SyringeGun, c, size * 1.15),
        Pocket::EnderFruit => draw_item_picture(art, ItemPicture::EnderFruit, c, size),
    }
}

/// 갑옷 남은 시간 글. 게임 속 초를 지금 빨리감기 배수로 나눠서, 실제로 기다릴 시간으로 보여 준다 (임시-26).
/// 예: 72초 ×10 → "8초", 720초 ×1 → "12분". 1분이 넘으면 "1분 5초".
pub fn armor_time_text(game_seconds_left: f32, time_scale: f32) -> String {
    let real = (game_seconds_left / time_scale.max(1.0)).ceil().max(0.0) as u32;
    match (real / 60, real % 60) {
        (0, s) => format!("{s}초"),
        (m, 0) => format!("{m}분"),
        (m, s) => format!("{m}분 {s}초"),
    }
}

/// 갑옷을 입은 주인공 둘레 빛 (아이가 정함): 다이아몬드는 하얀 하늘빛으로 반짝반짝, 무지개는 색이 바뀐다.
/// `body`는 몸 네모(그리는 자리), 캐릭터보다 먼저 그린다.
pub fn draw_armor_glow(armor: Armor, body: Rect, t: f32) {
    let c = body.center();
    let p = 0.5 + 0.5 * (t * 6.0).sin();
    let (rx, ry) = (body.w * 0.9 + 10.0 + 6.0 * p, body.h * 0.62 + 10.0 + 6.0 * p);
    let color = match armor {
        Armor::Diamond => Color::new(0.75, 0.95, 1.0, 1.0),
        Armor::Rainbow => rainbow(t * 0.8),
    };
    draw_ellipse(c.x, c.y, rx, ry, 0.0, Color { a: 0.22 + 0.12 * p, ..color });
    draw_ellipse_lines(c.x, c.y, rx, ry, 0.0, 4.0, Color { a: 0.6 + 0.3 * p, ..color });
}

/// 갑옷 위에 덧그리는 반짝이 (캐릭터 다음에 그린다). 다이아몬드는 별이 깜빡깜빡.
pub fn draw_armor_sparkles(armor: Armor, body: Rect, t: f32) {
    let c = body.center();
    for i in 0..4 {
        let a = t * 1.5 + i as f32 * std::f32::consts::FRAC_PI_2;
        let twinkle = (t * 7.0 + i as f32 * 1.7).sin().max(0.0);
        let p = c + vec2(a.cos() * (body.w * 0.8 + 8.0), a.sin() * (body.h * 0.55 + 6.0));
        let color = match armor {
            Armor::Diamond => Color::new(1.0, 1.0, 1.0, twinkle),
            Armor::Rainbow => Color { a: twinkle, ..rainbow(t * 0.8 + i as f32 * 0.25) },
        };
        draw_star(p, 5.0 + 6.0 * twinkle, t * 2.0, color);
    }
}

/// 무지개 갑옷이면 캐릭터 색이 반짝반짝 바뀐다: 원래 색에 무지개 색을 살짝 곱한다.
pub fn armor_tint(armor: Option<Armor>, tint: Color, t: f32) -> Color {
    match armor {
        Some(Armor::Rainbow) => {
            let r = rainbow(t * 0.8);
            let k = 0.45;
            Color::new(tint.r * (1.0 - k + k * r.r), tint.g * (1.0 - k + k * r.g), tint.b * (1.0 - k + k * r.b), tint.a)
        }
        _ => tint,
    }
}

/// 독에 걸린 몬스터 둘레 초록 기운 + 위로 올라가는 작은 거품 (무섭지 않게 동글동글).
pub fn draw_poison_aura(body: Rect, t: f32) {
    let c = body.center();
    let p = 0.5 + 0.5 * (t * 4.0).sin();
    draw_ellipse(c.x, c.y, body.w * 0.75 + 6.0 * p, body.h * 0.6 + 4.0 * p, 0.0, Color::new(0.45, 0.95, 0.35, 0.25 + 0.1 * p));
    for i in 0..3 {
        let rise = ((t * 0.9 + i as f32 / 3.0) % 1.0) * body.h * 0.8;
        let x = c.x + (i as f32 - 1.0) * body.w * 0.35;
        draw_circle(x, body.y + body.h * 0.4 - rise, 5.0, Color::new(0.55, 1.0, 0.45, 0.8 * (1.0 - rise / (body.h * 0.8))));
    }
}

/// 독에 걸린 몬스터 색: 살짝 초록빛.
pub fn poison_tint(tint: Color) -> Color {
    Color::new(tint.r * 0.75, tint.g, tint.b * 0.75, tint.a)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::character::CharacterKind;
    use crate::logic::shop::CantBuy;
    use crate::logic::world::World;

    fn all_targets() -> Vec<Rect> {
        let mut list: Vec<Rect> = (0..ShopItem::ALL.len()).map(buy_rect).collect();
        list.extend((0..SELLABLE.len()).map(sell_rect));
        list.push(CLOSE);
        list
    }

    #[test]
    fn everything_fits_in_the_panel_without_overlap_and_is_big() {
        let targets = all_targets();
        assert_eq!(targets.len(), 9, "물건 5개(칼·도끼·총은 뺐다) + 금·은·동 + 닫기");
        for (i, a) in targets.iter().enumerate() {
            assert!(PANEL.contains(a.point()) && PANEL.contains(vec2(a.right(), a.bottom())), "{a:?} 판 밖");
            // 740×360 화면은 0.667배라서 가장 작은 칸도 약 69 화면 픽셀이다
            assert!(a.w.min(a.h) >= 100.0, "{a:?} 작다");
            for b in &targets[i + 1..] {
                assert!(a.intersect(*b).is_none(), "{a:?} {b:?} 겹친다");
            }
        }
        // 사기 다섯 칸이 판 폭을 거의 다 쓴다 (한쪽에 빈자리가 크게 남지 않는다)
        let last = buy_rect(ShopItem::ALL.len() - 1);
        assert!(PANEL.right() - last.right() < 40.0 && buy_rect(0).x - PANEL.x < 40.0);
        // 칸이 통통 튀어도(최대 8) 제목 줄·판 밖으로 나가지 않는다
        assert!(BUY_TOP - 8.0 > TITLE_BASELINE + 4.0);
        assert!(PANEL.bottom() <= VIRTUAL_HEIGHT && PANEL.right() <= VIRTUAL_WIDTH);
    }

    #[test]
    fn text_is_big_enough_on_a_small_phone() {
        // 740×360 폰은 0.667배: 가장 작은 글(이유 글)도 약 16 화면 픽셀
        // (키 글자 1·2·R·3는 PC에서만 보여서 여기 넣지 않는다)
        for size in [NAME_SIZE, PRICE_SIZE, REASON_SIZE, CLOSE_SIZE] {
            assert!(size * 360.0 / 540.0 >= 16.0, "{size}");
        }
    }

    #[test]
    fn taps_find_the_right_target() {
        assert_eq!(shop_tap_at(CLOSE.center()), Some(ShopTap::Close));
        assert_eq!(shop_tap_at(buy_rect(0).center()), Some(ShopTap::Buy(ShopItem::HealthPotion)));
        assert_eq!(shop_tap_at(buy_rect(4).center()), Some(ShopTap::Buy(ShopItem::DiamondArmor)));
        assert_eq!(shop_tap_at(sell_rect(0).center()), Some(ShopTap::Sell(Material::Gold)));
        assert_eq!(shop_tap_at(sell_rect(2).center()), Some(ShopTap::Sell(Material::Copper)));
        assert_eq!(shop_tap_at(vec2(5.0, 5.0)), None);
    }

    #[test]
    fn buy_sell_and_close_go_to_logic() {
        let mut game = Game::on_world(CharacterKind::Niro, 7, World::m1());
        let mut view = ShopView::new();
        game.shop.open();
        game.wallet.add(250);
        assert!(view.apply(&mut game, shop_tap_at(buy_rect(2).center()).unwrap()), "스케이트 200코인");
        assert_eq!(game.wallet.coins(), 50);
        assert_eq!(view.feedback_text(), Some(SHOP_BOUGHT_TEXT));
        assert!(!view.apply(&mut game, ShopTap::Buy(ShopItem::Skates)), "신고 있는 스케이트는 다시 안 산다");
        assert_eq!(game.why_cant_buy(ShopItem::Skates), Some(CantBuy::AlreadyHave));
        game.gathering.bag.add(Material::Gold);
        assert!(view.apply(&mut game, shop_tap_at(sell_rect(0).center()).unwrap()));
        assert_eq!(game.wallet.coins(), 1050);
        assert_eq!(view.feedback_text(), Some(SHOP_SOLD_TEXT));
        assert!(!view.apply(&mut game, ShopTap::Sell(Material::Gold)), "금이 없으면 못 판다");
        for _ in 0..120 {
            view.update(1.0 / 60.0);
        }
        assert_eq!(view.feedback_text(), None, "기쁜 글은 잠깐만");
        view.apply(&mut game, ShopTap::Close);
        assert!(!game.shop.is_open());
    }

    #[test]
    fn armor_time_is_shown_as_real_waiting_time() {
        assert_eq!(armor_time_text(72.0, 10.0), "8초");
        assert_eq!(armor_time_text(72.0, 1.0), "1분 12초");
        assert_eq!(armor_time_text(720.0, 1.0), "12분");
        assert_eq!(armor_time_text(720.0, 10.0), "1분 12초");
        assert_eq!(armor_time_text(0.0, 10.0), "0초");
    }

    #[test]
    fn rainbow_goes_round_and_stays_bright() {
        for i in 0..12 {
            let c = rainbow(i as f32 / 12.0);
            assert!(c.r >= 0.35 && c.g >= 0.35 && c.b >= 0.35);
        }
        assert_ne!(rainbow(0.0), rainbow(0.5));
        assert_eq!(rainbow(0.0), rainbow(1.0));
    }
}
