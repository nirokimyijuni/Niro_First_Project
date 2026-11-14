//! 상자 부수기 화면 (US-089, D87, 로건): 상자의 금, 부서질 때 나뭇조각, 떨어진 깜짝 아이템, "와우!", "공격!", 도끼 안내.
//! 몇 번 쳤는지, 무엇이 나오는지, 언제 사라지는지는 모두 logic(`logic::boxes`)이 정한다. 여기서는 보여 주기만 한다.
//! 부서질 때는 글·소리 없이 나뭇조각만 흩어진다 (아이 답).

use super::assets::{ItemPicture, M6Art};
use super::effects::draw_bubble;
use super::items::draw_item_picture;
use super::places::near_view;
use super::shop::{draw_armor, draw_potion, draw_skates};
use super::sky::night;
use super::sprites::fit_size;
use super::text::TextPainter;
use super::touch::draw_star;
use crate::config::{BOX_MONSTER_TEXT, BOX_NEED_AXE_TEXT, BOX_WOW_TEXT};
use crate::logic::boxes::{BoxEvent, SurpriseItem};
use crate::logic::game::Game;
use crate::logic::shop::{Armor, Potion};
use crate::logic::weapon::Weapon;
use crate::logic::world::nearest_copy;
use macroquad::prelude::*;
use std::f32::consts::TAU;

/// 나뭇조각이 흩어지는 시간 (초). 금이 갈 때는 몇 조각만 튄다.
const CHIPS_SECONDS: f32 = 0.7;
const CRACK_CHIPS: u32 = 3;
const BREAK_CHIPS: u32 = 10;
/// 조각이 튀어 나가는 빠르기와 떨어지는 힘 (픽셀/초, 픽셀/초²)
const CHIP_SPEED: f32 = 170.0;
const CHIP_GRAVITY: f32 = 520.0;
const CHIP_SIZE: f32 = 9.0;
const CHIP_WOOD: Color = Color::new(0.80, 0.56, 0.30, 1.0);
const CHIP_DARK: Color = Color::new(0.52, 0.33, 0.15, 1.0);

/// 금 색과 굵기. 밤에는 상자처럼 조금 어두워진다.
const CRACK: Color = Color::new(0.24, 0.13, 0.05, 1.0);
const CRACK_WIDTH: f32 = 3.0;
const CRACK_NIGHT_AMOUNT: f32 = 0.7;

/// "도끼로 부술 수 있어요" 말풍선이 떠 있는 시간과, 상자 가운데에서 꼬리 끝까지의 높이
const NEED_AXE_SECONDS: f32 = 2.8;
const NEED_AXE_ABOVE: f32 = 38.0;

/// "와우!": 떠 있는 시간, 글자 크기, 처음 커지는 시간, 머리 위 틈
const WOW_SECONDS: f32 = 1.6;
const WOW_SIZE: f32 = 48.0;
const WOW_POP_SECONDS: f32 = 0.25;
const WOW_ABOVE: f32 = 14.0;
const WOW_TEXT: Color = Color::new(1.0, 0.86, 0.22, 1.0);

/// "공격!" 말풍선 꼬리 끝이 몬스터 머리에서 이만큼 위
const MONSTER_SHOUT_ABOVE: f32 = 6.0;

/// 땅의 깜짝 아이템: 그림 크기는 logic의 네모, 뒤의 빛, 사라지기 전 깜빡임
const ITEM_GLOW: Color = Color::new(1.0, 1.0, 0.85, 0.38);
const ITEM_GLOW_RADIUS: f32 = 24.0;
const ITEM_BLINK_FROM: f32 = 3.0;
const ITEM_BLINKS_PER_SECOND: f32 = 4.0;

/// 금 선들. 상자 네모를 0~1로 볼 때 꺾이는 점 4개짜리 선. 금 1 = 첫 줄, 금 2 = 세 줄 모두.
const CRACK_LINES: [[(f32, f32); 4]; 3] = [
    [(0.50, 0.0), (0.42, 0.30), (0.58, 0.52), (0.46, 0.80)],
    [(1.0, 0.30), (0.78, 0.40), (0.70, 0.62), (0.52, 0.66)],
    [(0.0, 0.58), (0.22, 0.52), (0.30, 0.74), (0.20, 1.0)],
];

/// 금 간 정도(0~2)에 그릴 금 선들.
pub fn crack_lines(cracks: u32) -> &'static [[(f32, f32); 4]] {
    match cracks {
        0 => &[],
        1 => &CRACK_LINES[..1],
        _ => &CRACK_LINES[..],
    }
}

/// 상자 위에 금을 그린다. `cracks`는 `Boxes::cracks_at`이 준 값 (0이면 아무것도 안 그린다).
pub fn draw_cracks(r: Rect, cracks: u32, d: f32) {
    let color = night(CRACK, d * CRACK_NIGHT_AMOUNT);
    let at = |(x, y): (f32, f32)| vec2(r.x + r.w * x, r.y + r.h * y);
    for line in crack_lines(cracks) {
        for pair in line.windows(2) {
            let (a, b) = (at(pair[0]), at(pair[1]));
            draw_line(a.x, a.y, b.x, b.y, CRACK_WIDTH, color);
        }
    }
}

/// 깜짝 아이템을 무슨 그림으로 그릴지. 모두 이미 있는 그림을 다시 쓴다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurpriseLook {
    Weapon(Weapon),
    Potion(Potion),
    Skates,
    Armor(Armor),
    Picture(ItemPicture),
}

pub fn surprise_look(item: SurpriseItem) -> SurpriseLook {
    match item {
        SurpriseItem::Sword => SurpriseLook::Weapon(Weapon::Sword),
        SurpriseItem::Axe => SurpriseLook::Weapon(Weapon::Axe),
        SurpriseItem::Gun => SurpriseLook::Weapon(Weapon::Gun),
        SurpriseItem::HealthPotion => SurpriseLook::Potion(Potion::Health),
        SurpriseItem::PoisonPotion => SurpriseLook::Potion(Potion::Poison),
        SurpriseItem::Skates => SurpriseLook::Skates,
        SurpriseItem::RainbowArmor => SurpriseLook::Armor(Armor::Rainbow),
        SurpriseItem::DiamondArmor => SurpriseLook::Armor(Armor::Diamond),
        SurpriseItem::SyringeGun => SurpriseLook::Picture(ItemPicture::SyringeGun),
        SurpriseItem::EnderFruit => SurpriseLook::Picture(ItemPicture::EnderFruit),
        SurpriseItem::Car => SurpriseLook::Picture(ItemPicture::Car),
        SurpriseItem::DiamondPickaxe => SurpriseLook::Picture(ItemPicture::DiamondPickaxe),
    }
}

/// 깜짝 아이템 그림 하나를 `c` 가운데 `size` 안에 그린다.
pub fn draw_surprise_item(art: &M6Art, item: SurpriseItem, c: Vec2, size: f32, t: f32) {
    match surprise_look(item) {
        SurpriseLook::Weapon(weapon) => {
            if let Some(texture) = art.weapon(weapon) {
                let s = fit_size(texture, size);
                draw_texture_ex(texture, c.x - s.x / 2.0, c.y - s.y / 2.0, WHITE, DrawTextureParams { dest_size: Some(s), ..Default::default() });
            }
        }
        SurpriseLook::Potion(potion) => draw_potion(potion, c, size),
        SurpriseLook::Skates => draw_skates(c, size),
        SurpriseLook::Armor(armor) => draw_armor(armor, c, size, t),
        SurpriseLook::Picture(picture) => draw_item_picture(art, picture, c, size),
    }
}

/// 사라지기 `ITEM_BLINK_FROM`초 전부터 깜빡인다. 이번 순간(`t`초)에 안 보일 차례면 true.
pub fn blinked_out(seconds_left: f32, t: f32) -> bool {
    seconds_left <= ITEM_BLINK_FROM && (t * ITEM_BLINKS_PER_SECOND).fract() < 0.5
}

/// 상자에서 나와 떨어지는 중이거나 땅에 놓인 깜짝 아이템. 놓이면 뒤에 빛이 깜빡여서 밤에도 찾기 쉽다 (땅의 무기와 같다).
pub fn draw_dropped_items(art: &M6Art, game: &Game, view_x: f32, t: f32) {
    for dropped in &game.boxes.items {
        if blinked_out(dropped.seconds_left(), t) {
            continue;
        }
        let r = nearest_copy(dropped.rect, view_x);
        let c = r.center();
        if !dropped.is_falling() {
            let glow = Color { a: ITEM_GLOW.a * (0.7 + 0.3 * (t * 4.0).sin()), ..ITEM_GLOW };
            draw_circle(c.x, c.y, ITEM_GLOW_RADIUS, glow);
        }
        draw_surprise_item(art, dropped.item, c, r.w.max(r.h), t);
    }
}

/// 상자에서 나와 가만히 있는 몬스터 머리 위 "공격!" (logic의 `is_waiting` 동안).
pub fn draw_monster_shouts(text: &TextPainter, game: &Game, view_x: f32, camera_left: f32) {
    let zombies = game.combat.zombies.list.iter().filter(|z| z.is_waiting()).map(|z| z.rect());
    let skeletons = game.combat.skeletons.list.iter().filter(|s| s.is_waiting()).map(|s| s.rect());
    for r in zombies.chain(skeletons) {
        draw_bubble(text, BOX_MONSTER_TEXT, near_view(view_x, r.center().x), r.y - MONSTER_SHOUT_ABOVE, camera_left, 1.0);
    }
}

/// 이번 프레임에 상자에서 나온 몬스터의 자리. 같은 몬스터의 `ZombieAppeared`·`SkeletonAppeared`가
/// 같은 `at`으로 `GameEvent::Combat`에도 오므로, 그 이벤트를 상자 몬스터로 알아보는 데 쓴다.
pub fn box_monster_spots(events: &[BoxEvent]) -> Vec<Vec2> {
    events.iter().filter_map(|e| if let BoxEvent::MonsterJumpedOut { at, .. } = e { Some(*at) } else { None }).collect()
}

/// 나뭇조각 한 묶음
#[derive(Debug, Clone, Copy)]
struct Chips {
    at: Vec2,
    count: u32,
    age: f32,
}

/// 조각 `i`(전체 `count`개)가 `age`초에 있는 자리. 위쪽 반원으로 튀었다가 떨어진다.
fn chip_position(at: Vec2, i: u32, count: u32, age: f32) -> Vec2 {
    let a = std::f32::consts::PI * (0.1 + 0.8 * (i as f32 + 0.5) / count as f32);
    let speed = CHIP_SPEED * (0.7 + 0.3 * ((i * 7 % 5) as f32 / 4.0));
    at + vec2(-a.cos() * speed * age, -a.sin() * speed * age + 0.5 * CHIP_GRAVITY * age * age)
}

/// 상자 효과 (화면 전용). `Effects`가 들고 있고 함께 흐르고 그려진다.
#[derive(Default)]
pub struct BoxView {
    chips: Vec<Chips>,
    /// 도끼 안내 말풍선을 띄울 상자 가운데와 지난 시간
    need_axe: Option<(Vec2, f32)>,
    /// "와우!" 지난 시간. 주인공을 따라다닌다.
    wow_age: Option<f32>,
}

impl BoxView {
    /// 상자에서 생긴 일을 받는다. 화면 가운데 큰 문구로 보여 줄 것이 있으면 돌려준다 ("○○을 얻었어요!").
    pub fn on_event(&mut self, event: BoxEvent) -> Option<&'static str> {
        match event {
            BoxEvent::Cracked { at, .. } => self.chips.push(Chips { at, count: CRACK_CHIPS, age: 0.0 }),
            BoxEvent::Broken { at, .. } => self.chips.push(Chips { at, count: BREAK_CHIPS, age: 0.0 }),
            BoxEvent::NeedAxe { at } => self.need_axe = Some((at, 0.0)),
            BoxEvent::Wow { .. } => self.wow_age = Some(0.0),
            BoxEvent::ItemPicked { item } => return Some(item.picked_text()),
            // 아이템은 땅 그림(`draw_dropped_items`)이, 몬스터의 "공격!"은 `draw_monster_shouts`가 보여 준다
            BoxEvent::ItemDropped { .. } | BoxEvent::MonsterJumpedOut { .. } | BoxEvent::CameBack { .. } => {}
        }
        None
    }

    pub fn update(&mut self, dt: f32) {
        for chips in &mut self.chips {
            chips.age += dt;
        }
        self.chips.retain(|c| c.age < CHIPS_SECONDS);
        self.need_axe = self.need_axe.map(|(at, age)| (at, age + dt)).filter(|(_, age)| *age < NEED_AXE_SECONDS);
        self.wow_age = self.wow_age.map(|age| age + dt).filter(|age| *age < WOW_SECONDS);
    }

    /// 맵 좌표로 그린다. `player`는 주인공 몸 네모, `view_x`는 화면 가운데의 맵 x.
    pub fn draw(&self, text: &TextPainter, player: Rect, view_x: f32, camera_left: f32) {
        for chips in &self.chips {
            let at = vec2(near_view(view_x, chips.at.x), chips.at.y);
            let fade = 1.0 - chips.age / CHIPS_SECONDS;
            for i in 0..chips.count {
                let p = chip_position(at, i, chips.count, chips.age);
                let color = if i % 2 == 0 { CHIP_WOOD } else { CHIP_DARK };
                let turn = chips.age * 9.0 + i as f32;
                let (dx, dy) = (turn.cos() * CHIP_SIZE / 2.0, turn.sin() * CHIP_SIZE / 2.0);
                draw_line(p.x - dx, p.y - dy, p.x + dx, p.y + dy, CHIP_SIZE * 0.6, Color { a: fade, ..color });
            }
        }
        if let Some((at, age)) = self.need_axe {
            let fade = ((1.0 - age / NEED_AXE_SECONDS) * 3.0).min(1.0);
            draw_bubble(text, BOX_NEED_AXE_TEXT, near_view(view_x, at.x), at.y - NEED_AXE_ABOVE, camera_left, fade);
        }
        if let Some(age) = self.wow_age {
            let fade = ((1.0 - age / WOW_SECONDS) * 3.0).min(1.0);
            let zoom = wow_zoom(age);
            let baseline = player.y - WOW_ABOVE;
            let x = player.center().x;
            for i in 0..5 {
                let a = i as f32 / 5.0 * TAU + age * 2.0;
                let p = vec2(x + a.cos() * 62.0, baseline - WOW_SIZE * 0.35 + a.sin() * 26.0);
                draw_star(p, 8.0 * zoom, age * 3.0, Color { a: fade, ..WOW_TEXT });
            }
            text.centered_zoomed(BOX_WOW_TEXT, x, baseline, WOW_SIZE, zoom, Color { a: fade, ..WOW_TEXT });
        }
    }

    /// 떠 있는 나뭇조각 묶음 수 (시험용)
    #[cfg(test)]
    fn chip_bursts(&self) -> usize {
        self.chips.len()
    }
}

/// "와우!"가 톡 커지는 배율: 0.6에서 시작해 `WOW_POP_SECONDS` 동안 1까지.
fn wow_zoom(age: f32) -> f32 {
    0.6 + 0.4 * (age / WOW_POP_SECONDS).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn more_hits_show_more_cracks() {
        assert!(crack_lines(0).is_empty(), "새 상자는 금이 없다");
        assert!(!crack_lines(1).is_empty());
        assert!(crack_lines(2).len() > crack_lines(1).len(), "금이 많이 = 금이 조금보다 많다");
        for line in crack_lines(2) {
            for (x, y) in line {
                assert!((0.0..=1.0).contains(x) && (0.0..=1.0).contains(y), "금은 상자 안에만");
            }
        }
    }

    #[test]
    fn every_surprise_item_has_its_own_picture() {
        let looks: Vec<SurpriseLook> = SurpriseItem::ALL.iter().map(|i| surprise_look(*i)).collect();
        for (i, a) in looks.iter().enumerate() {
            for b in &looks[i + 1..] {
                assert_ne!(a, b, "12가지가 모두 다르게 보인다");
            }
        }
        assert!(!looks.contains(&SurpriseLook::Weapon(Weapon::Hands)), "맨손은 그림이 없다");
    }

    #[test]
    fn items_blink_only_before_vanishing() {
        for i in 0..100 {
            let t = i as f32 * 0.03;
            assert!(!blinked_out(ITEM_BLINK_FROM + 0.1, t), "아직 시간이 많으면 늘 보인다");
        }
        let hidden = (0..100).filter(|i| blinked_out(1.0, *i as f32 * 0.03)).count();
        assert!(hidden > 20 && hidden < 80, "끝나 갈 때는 보였다 안 보였다 한다: {hidden}");
    }

    #[test]
    fn breaking_shows_only_wood_chips_and_picking_shows_text() {
        let mut view = BoxView::default();
        let at = vec2(300.0, 400.0);
        assert_eq!(view.on_event(BoxEvent::Broken { index: 0, at }), None, "부서질 때 글은 없다");
        assert_eq!(view.chip_bursts(), 1);
        assert_eq!(view.on_event(BoxEvent::Cracked { index: 0, at, cracks: 1 }), None);
        assert_eq!(view.chip_bursts(), 2);
        assert_eq!(view.on_event(BoxEvent::ItemPicked { item: SurpriseItem::Car }), Some(SurpriseItem::Car.picked_text()));
        for _ in 0..10 {
            view.update(0.1);
        }
        assert_eq!(view.chip_bursts(), 0, "조각은 잠깐 뒤 사라진다");
    }

    #[test]
    fn box_monster_spots_are_only_jumped_out_monsters() {
        let at = vec2(300.0, 400.0);
        let events = [
            BoxEvent::Broken { index: 0, at: vec2(1.0, 2.0) },
            BoxEvent::MonsterJumpedOut { monster: crate::logic::boxes::BoxMonster::Zombie, at },
        ];
        assert_eq!(box_monster_spots(&events), vec![at]);
        assert!(box_monster_spots(&[]).is_empty());
    }

    #[test]
    fn need_axe_and_wow_fade_away() {
        let mut view = BoxView::default();
        view.on_event(BoxEvent::NeedAxe { at: vec2(10.0, 20.0) });
        view.on_event(BoxEvent::Wow { at: vec2(10.0, 20.0) });
        assert!(view.need_axe.is_some() && view.wow_age.is_some());
        view.update(WOW_SECONDS + 0.01);
        assert!(view.wow_age.is_none());
        view.update(NEED_AXE_SECONDS);
        assert!(view.need_axe.is_none());
    }

    #[test]
    fn wow_pops_up_then_stays_full_size() {
        assert!((wow_zoom(0.0) - 0.6).abs() < 1e-6);
        assert!((wow_zoom(WOW_POP_SECONDS) - 1.0).abs() < 1e-6);
        assert!((wow_zoom(WOW_SECONDS) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn chips_fly_up_first_then_fall() {
        let at = vec2(0.0, 0.0);
        for i in 0..BREAK_CHIPS {
            assert!(chip_position(at, i, BREAK_CHIPS, 0.1).y < 0.0, "처음엔 위로 튄다");
            assert!(chip_position(at, i, BREAK_CHIPS, CHIPS_SECONDS).y > chip_position(at, i, BREAK_CHIPS, 0.3).y, "나중엔 떨어진다");
        }
    }
}
