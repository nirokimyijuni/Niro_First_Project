//! 시험용 단축 (로건, 부모님 결정 2026-11-14 "시험 단축 만들기"): 엘리가 7일을 기다리지 않고 확인하게.
//! 주소에 `?qa`가 있을 때만 켜진다 (PC 실행은 `cargo run -- qa`). 보통 주소에서는 단축도 "QA" 표시도 없다.
//! PC는 숫자 키 1~9·0, 폰과 마우스는 오른쪽 위 "QA" 표를 눌러 큰 칸 창에서 고른다.
//! 규칙을 새로 만들지 않는다: logic의 pub 필드·함수(지갑, 가방, 상점, 무기, 시계, 상자 주사위)로 상태만 바꾼다.

use super::hud::draw_pill;
use super::screen::{VIRTUAL_HEIGHT, VIRTUAL_WIDTH};
use super::text::TextPainter;
use crate::config::DAYS_TO_CLEAR;
use crate::logic::boxes::{BoxMonster, Surprise, SurpriseDice};
use crate::logic::game::Game;
use crate::logic::gathering::Material;
use crate::logic::player::Facing;
use crate::logic::shop::{Armor, Potion};
use crate::logic::weapon::Weapon;
use macroquad::prelude::*;

/// 코인 채우기는 이만큼까지
pub const QA_COINS: u32 = 3000;
/// 재료 채우기는 종류마다 이만큼까지
pub const QA_MATERIALS: u32 = 20;
/// 포션·엔더열매는 이만큼까지
pub const QA_POTIONS: u32 = 3;
/// 7일째 저녁: 밤까지 남는 초. 클리어 직전: 마지막 밤이 끝날 때까지 남는 초.
pub const QA_EVENING_SECONDS_LEFT: f32 = 20.0;
pub const QA_CLEAR_SECONDS_LEFT: f32 = 10.0;
/// 상자 주사위를 바꿀 때 찾아볼 시드 수 (끝없이 돌지 않게)
const DICE_TRIES: u32 = 2000;

/// 오른쪽 "QA" 표: 정보판(물건 줄까지, 아래 끝 238)보다 아래, 폰 버튼·조이스틱과 먼 곳 (BUG-031).
/// 상점·문제·제작대·멈춤 창이 열려 있으면 그리지도 받지도 않는다 (창의 칸·버튼을 가리지 않게).
pub const QA_BADGE: Rect = Rect::new(VIRTUAL_WIDTH - 104.0, 248.0, 92.0, 52.0);
const PANEL: Rect = Rect::new(80.0, 40.0, 800.0, 460.0);
const CELL_W: f32 = 370.0;
const CELL_H: f32 = 70.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QaAction {
    Coins,
    Materials,
    Items,
    DaySevenEvening,
    BeforeClear,
    NightNow,
    ToCave,
    ToBox,
    BoxMonsterNext,
    BoxItemNext,
}

impl QaAction {
    /// 숫자 키 순서 (1~9, 0)
    pub const ALL: [QaAction; 10] = [
        QaAction::Coins,
        QaAction::Materials,
        QaAction::Items,
        QaAction::DaySevenEvening,
        QaAction::BeforeClear,
        QaAction::NightNow,
        QaAction::ToCave,
        QaAction::ToBox,
        QaAction::BoxMonsterNext,
        QaAction::BoxItemNext,
    ];

    pub fn label(self) -> &'static str {
        match self {
            QaAction::Coins => "1 코인 3000",
            QaAction::Materials => "2 재료 채우기",
            QaAction::Items => "3 아이템·무기 다",
            QaAction::DaySevenEvening => "4 7일째 저녁",
            QaAction::BeforeClear => "5 클리어 직전",
            QaAction::NightNow => "6 밤으로",
            QaAction::ToCave => "7 동굴 앞",
            QaAction::ToBox => "8 상자 앞 (도끼)",
            QaAction::BoxMonsterNext => "9 다음 상자=몬스터",
            QaAction::BoxItemNext => "0 다음 상자=아이템",
        }
    }

    fn key(self) -> KeyCode {
        const KEYS: [KeyCode; 10] = [
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Key5,
            KeyCode::Key6,
            KeyCode::Key7,
            KeyCode::Key8,
            KeyCode::Key9,
            KeyCode::Key0,
        ];
        KEYS[QaAction::ALL.iter().position(|a| *a == self).unwrap_or(0)]
    }
}

/// QA 창의 칸 자리 (가상 좌표): 두 줄로 다섯 칸씩
pub fn cell_rect(i: usize) -> Rect {
    let (col, row) = ((i / 5) as f32, (i % 5) as f32);
    Rect::new(PANEL.x + 20.0 + col * (CELL_W + 20.0), PANEL.y + 64.0 + row * (CELL_H + 8.0), CELL_W, CELL_H)
}

pub fn qa_tap_at(point: Vec2) -> Option<QaAction> {
    QaAction::ALL.into_iter().enumerate().find(|(i, _)| cell_rect(*i).contains(point)).map(|(_, a)| a)
}

/// 시험용 단축 상태. `enabled`가 false이면(보통 주소) 아무것도 하지 않는다.
#[derive(Debug, Default)]
pub struct Qa {
    pub enabled: bool,
    pub open: bool,
}

impl Qa {
    pub fn new(enabled: bool) -> Qa {
        Qa { enabled, open: false }
    }

    /// 이번 프레임에 누른 숫자 키의 단축. 꺼져 있으면 늘 None.
    pub fn key_action(&self) -> Option<QaAction> {
        if !self.enabled {
            return None;
        }
        QaAction::ALL.into_iter().find(|a| is_key_pressed(a.key()))
    }

    /// "QA" 표를 눌렀는지. 꺼져 있으면 늘 false.
    pub fn badge_tapped(&self, tap: Option<Vec2>) -> bool {
        self.enabled && tap.is_some_and(|p| QA_BADGE.contains(p))
    }
}

/// 단축 하나를 게임에 적용한다. 상태만 바꾸고 규칙(쓰기·사기·싸움)은 그대로 logic이 한다.
pub fn apply(game: &mut Game, action: QaAction) {
    match action {
        QaAction::Coins => {
            let have = game.wallet.coins();
            game.wallet.add(QA_COINS.saturating_sub(have));
        }
        QaAction::Materials => {
            for m in Material::ALL {
                for _ in game.gathering.bag.count(m)..QA_MATERIALS {
                    game.gathering.bag.add(m);
                }
            }
        }
        QaAction::Items => {
            let bag = &mut game.gathering.bag;
            bag.has_pickaxe = true;
            bag.has_diamond_pickaxe = true;
            bag.has_syringe_gun = true;
            bag.ender_fruits = bag.ender_fruits.max(QA_POTIONS);
            bag.riding_car = true;
            for p in [Potion::Health, Potion::Poison] {
                for _ in game.shop.potions(p)..QA_POTIONS {
                    game.shop.add_potion(p);
                }
            }
            game.shop.put_on_skates();
            game.shop.put_on_armor(Armor::Diamond);
            for w in [Weapon::Sword, Weapon::Axe, Weapon::Gun] {
                if !game.weapons.has(w) {
                    game.weapons.add_bought(w);
                }
            }
        }
        QaAction::DaySevenEvening => skip_clock_to(game, DAYS_TO_CLEAR, false, QA_EVENING_SECONDS_LEFT),
        QaAction::BeforeClear => skip_clock_to(game, DAYS_TO_CLEAR, true, QA_CLEAR_SECONDS_LEFT),
        QaAction::NightNow => {
            if !game.clock.is_night() {
                game.clock.update(game.clock.seconds_left_in_phase() + 0.01);
            }
        }
        QaAction::ToCave => {
            if let Some(r) = game.world.cave_entrance() {
                stand_left_of(game, r);
            }
        }
        QaAction::ToBox => {
            let x = game.player.rect().center().x;
            let nearest = game
                .boxes
                .list
                .iter()
                .filter(|b| b.is_standing())
                .map(|b| b.rect)
                .min_by(|a, b| (a.center().x - x).abs().total_cmp(&(b.center().x - x).abs()));
            if let Some(r) = nearest {
                stand_left_of(game, r);
                if !game.weapons.has(Weapon::Axe) {
                    game.weapons.add_bought(Weapon::Axe);
                }
                game.weapons.hold(Weapon::Axe);
            }
        }
        QaAction::BoxMonsterNext => set_next_surprise(game, Surprise::Monster),
        QaAction::BoxItemNext => set_next_surprise(game, Surprise::Item),
    }
}

/// 시계를 앞으로만 돌려 `day`일째 `night`(낮/밤) 구간의 끝 `seconds_left`초 전에 둔다. 이미 지났으면 그대로.
fn skip_clock_to(game: &mut Game, day: u32, night: bool, seconds_left: f32) {
    let past = |g: &Game| g.clock.day_number() > day || (g.clock.day_number() == day && g.clock.is_night() && !night);
    if past(game) {
        return;
    }
    // 한 번에 한 구간씩. 7일이면 14번이면 된다.
    for _ in 0..64 {
        if game.clock.day_number() == day && game.clock.is_night() == night {
            break;
        }
        game.clock.update(game.clock.seconds_left_in_phase() + 0.01);
    }
    let left = game.clock.seconds_left_in_phase();
    if left > seconds_left {
        game.clock.update(left - seconds_left);
    }
}

/// 맵 밖(동굴)에 있으면 맵으로 나와서 `target` 왼쪽 땅에 오른쪽을 보고 선다.
fn stand_left_of(game: &mut Game, target: Rect) {
    game.in_cave = false;
    game.player.pos = vec2(target.x - game.player.size.x - 8.0, game.world.ground_y - game.player.size.y);
    game.player.vel = Vec2::ZERO;
    game.player.facing = Facing::Right;
}

/// 다음에 부서지는 상자가 `wanted`(몬스터면 좀비/스켈레톤 어느 쪽이든)가 되도록 상자 주사위를 새 시드로 바꾼다.
/// 주사위의 다른 수는 그대로 logic이 굴린다.
fn set_next_surprise(game: &mut Game, wanted: Surprise) {
    if let Some(seed) = seed_for(wanted) {
        game.boxes.dice = SurpriseDice::new(seed);
    }
}

fn seed_for(wanted: Surprise) -> Option<u32> {
    (1..DICE_TRIES).find(|s| {
        let mut dice = SurpriseDice::new(*s);
        let got = dice.roll_surprise();
        // 몬스터는 좀비가 나오게 (상자 좀비 "공격!"을 보기 쉽게)
        got == wanted && (wanted != Surprise::Monster || dice.roll_monster() == BoxMonster::Zombie)
    })
}

/// 오른쪽 위 작은 "QA" 표. 꺼져 있으면 아무것도 안 그린다.
pub fn draw_badge(text: &TextPainter, qa: &Qa) {
    if !qa.enabled {
        return;
    }
    let b = QA_BADGE;
    draw_pill(b.x, b.y, b.w, b.h, Color::new(0.75, 0.10, 0.55, 0.9));
    text.centered("QA", b.center().x, b.center().y + 12.0, 34.0, WHITE);
}

/// QA 창 (열려 있을 때만)
pub fn draw_menu(text: &TextPainter, qa: &Qa) {
    if !(qa.enabled && qa.open) {
        return;
    }
    draw_rectangle(0.0, 0.0, VIRTUAL_WIDTH, VIRTUAL_HEIGHT, Color::new(0.0, 0.0, 0.0, 0.4));
    draw_rectangle(PANEL.x, PANEL.y, PANEL.w, PANEL.h, Color::new(0.20, 0.05, 0.22, 0.95));
    text.centered("QA 단축 (시험용) — 밖을 누르면 닫기", PANEL.center().x, PANEL.y + 44.0, 28.0, WHITE);
    for (i, a) in QaAction::ALL.into_iter().enumerate() {
        let r = cell_rect(i);
        draw_pill(r.x, r.y, r.w, r.h, Color::new(0.75, 0.10, 0.55, 1.0));
        text.centered(a.label(), r.center().x, r.center().y + 11.0, 30.0, WHITE);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic::character::CharacterKind;

    fn game() -> Game {
        Game::new(CharacterKind::Niro, 7)
    }

    #[test]
    fn off_without_qa_in_the_address() {
        let qa = Qa::new(false);
        assert_eq!(qa.key_action(), None);
        assert!(!qa.badge_tapped(Some(QA_BADGE.center())), "보통 주소에서는 QA 표를 눌러도 아무 일 없다");
        assert!(Qa::new(true).badge_tapped(Some(QA_BADGE.center())));
        assert!(!Qa::new(true).badge_tapped(Some(vec2(5.0, 5.0))));
    }

    #[test]
    fn badge_is_below_the_info_panel_and_away_from_buttons_bug_031() {
        use crate::view::touch::{play_buttons_with_mine, BACK_BUTTON};
        // 정보판은 물건 줄(무기 있을 때)까지 238에서 끝난다
        let info_bottom = crate::view::hud::hud_bottom(true).max(238.0);
        assert!(QA_BADGE.y > info_bottom, "정보판 아래");
        assert!(QA_BADGE.right() <= VIRTUAL_WIDTH && QA_BADGE.bottom() <= VIRTUAL_HEIGHT);
        assert!(!QA_BADGE.overlaps(&BACK_BUTTON));
        for b in play_buttons_with_mine(true, true) {
            let nearest = vec2(b.center.x.clamp(QA_BADGE.x, QA_BADGE.right()), b.center.y.clamp(QA_BADGE.y, QA_BADGE.bottom()));
            assert!(nearest.distance(b.center) > b.radius + 8.0, "{:?} 버튼과 떨어져 있다", b.kind);
        }
    }

    #[test]
    fn cells_are_big_inside_the_panel_and_apart() {
        for i in 0..QaAction::ALL.len() {
            let r = cell_rect(i);
            assert!(PANEL.contains(r.point()) && PANEL.contains(vec2(r.right(), r.bottom())), "{i}");
            assert_eq!(qa_tap_at(r.center()), Some(QaAction::ALL[i]));
            for j in 0..i {
                assert!(r.intersect(cell_rect(j)).is_none());
            }
        }
        assert_eq!(qa_tap_at(vec2(5.0, 5.0)), None);
    }

    #[test]
    fn fills_coins_materials_and_items() {
        let mut g = game();
        apply(&mut g, QaAction::Coins);
        assert_eq!(g.wallet.coins(), QA_COINS);
        apply(&mut g, QaAction::Materials);
        assert!(Material::ALL.iter().all(|m| g.gathering.bag.count(*m) == QA_MATERIALS));
        apply(&mut g, QaAction::Materials);
        assert!(Material::ALL.iter().all(|m| g.gathering.bag.count(*m) == QA_MATERIALS), "두 번 눌러도 그만큼");
        apply(&mut g, QaAction::Items);
        let bag = &g.gathering.bag;
        assert!(bag.has_syringe_gun && bag.has_diamond_pickaxe && bag.riding_car && bag.ender_fruits == QA_POTIONS);
        assert_eq!(g.shop.potions(Potion::Health), QA_POTIONS);
        assert!(g.shop.has_skates() && g.shop.armor() == Some(Armor::Diamond));
        assert!(g.can_switch() && g.weapons.has(Weapon::Gun));
    }

    #[test]
    fn skips_to_day_seven_evening_and_before_clear() {
        let mut g = game();
        apply(&mut g, QaAction::DaySevenEvening);
        assert_eq!((g.clock.day_number(), g.clock.is_night()), (DAYS_TO_CLEAR, false));
        assert!((g.clock.seconds_left_in_phase() - QA_EVENING_SECONDS_LEFT).abs() < 0.1);
        apply(&mut g, QaAction::BeforeClear);
        assert_eq!((g.clock.day_number(), g.clock.is_night()), (DAYS_TO_CLEAR, true));
        assert!((g.clock.seconds_left_in_phase() - QA_CLEAR_SECONDS_LEFT).abs() < 0.1);
        assert!(!g.is_cleared());
        apply(&mut g, QaAction::DaySevenEvening);
        assert!(g.clock.is_night(), "시계는 뒤로 가지 않는다");
        let mut g = game();
        apply(&mut g, QaAction::NightNow);
        assert!(g.clock.is_night() && g.clock.day_number() == 1);
    }

    #[test]
    fn moves_to_the_cave_and_a_box_with_an_axe() {
        let mut g = game();
        apply(&mut g, QaAction::ToCave);
        let cave = g.world.cave_entrance().expect("긴 맵에는 동굴이 있다");
        assert!(g.player.rect().right() <= cave.x && cave.x - g.player.rect().right() < 20.0);
        apply(&mut g, QaAction::ToBox);
        assert_eq!(g.weapons.held(), Weapon::Axe);
        let body = g.player.rect();
        assert!(g.boxes.list.iter().any(|b| b.is_standing() && b.rect.x >= body.right() && b.rect.x - body.right() < 20.0));
    }

    #[test]
    fn next_box_gives_a_monster_or_an_item() {
        for wanted in [Surprise::Monster, Surprise::Item] {
            let mut g = game();
            apply(&mut g, if wanted == Surprise::Monster { QaAction::BoxMonsterNext } else { QaAction::BoxItemNext });
            assert_eq!(g.boxes.dice.roll_surprise(), wanted);
        }
    }
}
