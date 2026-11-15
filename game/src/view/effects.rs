//! 싸움 중 잠깐 보이는 효과: 공격 휘두름, 몬스터가 쓰러질 때·펑 터질 때, 화살·총알 번쩍, 스켈레톤 손, "레벨 업!", 아침에 하트 반짝, 코인 반짝.
//! 일어난 일은 logic의 `CombatEvent`가 알려 주고, 여기서는 보여 주기만 한다. 시간은 진짜 시간 `dt`로 흐른다.

use super::assets::M6Art;
use super::boxes::BoxView;
use super::coin::CoinPops;
use super::mission::MissionView;
use super::hud::draw_pill;
use super::items;
use super::places::near_view;
use super::screen::VIRTUAL_WIDTH;
use super::sky::{hash, lerp};
use super::sprites::{fit_size, flip_weapon};
use super::text::{TextPainter, TEXT_ABOVE, TEXT_BELOW};
use super::touch::draw_star;
use crate::config::{
    CAR_OFF_TEXT, CITIZEN_POP_TEXT, CITIZEN_TALK_SHARE, DIAMOND_PICKAXE_GONE_TEXT, ENDER_FRUIT_ALREADY_EATEN_TEXT, ENDER_FRUIT_EATEN_TEXT, MISS_FOUND_TEXT, POISON_RANGE,
    SKELETON_DODGE_TEXT, ZOMBIE_APPEAR_TEXT,
};
use crate::logic::game::{ItemEvent, SlamPower};
use crate::logic::combat::{attack_box, attack_box_with_range, CombatEvent};
use crate::logic::player::Facing;
use crate::logic::weapon::Weapon;
use macroquad::prelude::*;
use std::f32::consts::TAU;

/// 효과가 보이는 시간 (초)
const SWING_SECONDS: f32 = 0.22;
const BURST_SECONDS: f32 = 0.9;
const HEART_SPARKLE_SECONDS: f32 = 1.2;
const SPARK_SECONDS: f32 = 0.25;
const HAND_SECONDS: f32 = 0.4;
const LEVEL_UP_SECONDS: f32 = 1.8;
const MESSAGE_SECONDS: f32 = 2.2;
const CHEER_SECONDS: f32 = 1.0;
const SMOKE_SECONDS: f32 = 0.6;
const ZOMBIE_CALL_SECONDS: f32 = 2.0;
const POISON_CLOUD_SECONDS: f32 = 1.4;
const HEARTS_SECONDS: f32 = 1.2;
/// 준 체력 칸이 줄어드는 시간
const HURT_SHRINK_SECONDS: f32 = 1.0;
const POISON_CLOUD: Color = Color::new(0.45, 0.90, 0.35, 0.55);
const POTION_HEART: Color = Color::new(0.96, 0.32, 0.45, 1.0);
/// 손이 플레이어에게 닿는 때 (전체를 0~1로 볼 때)
const HAND_REACH_SHARE: f32 = 0.4;
/// "레벨 업!" 바라는 바닥선, 크기, 별이 도는 타원 폭·높이 (글자 크기에 대한 비율)
const LEVEL_UP_Y: f32 = 215.0;
const LEVEL_UP_SIZE: f32 = 84.0;
const LEVEL_UP_RING: Vec2 = Vec2::new(2.7, 0.62);

/// "미스를 찾았어요!" 같은 큰 문구 바라는 바닥선과 크기. "레벨 업!" 아래.
const MESSAGE_Y: f32 = 300.0;
const MESSAGE_SIZE: f32 = 56.0;
/// 두 줄 문구의 줄 간격 (글자 크기에 대한 비율)
const MESSAGE_LINE_STEP: f32 = 1.12;
/// 큰 글이 들어갈 화면 폭. 넘치면 두 줄로 나누고, 그래도 넘치면 줄인다 (BUG-031).
const MESSAGE_MAX_WIDTH: f32 = 920.0;
/// 머리 위 E 표시와의 틈
const MESSAGE_GAP: f32 = 8.0;
/// 큰 글 덩이 사이 틈, 정보판과의 틈
const STACK_GAP: f32 = 10.0;
const INFO_GAP: f32 = 8.0;
/// 큰 글 덩이들이 내려갈 수 있는 아래 끝 (땅 근처). E 표시 아래로 피할 때는 놀이 안내 줄 위까지.
const STACK_FLOOR: f32 = 430.0;
const STACK_FLOOR_BELOW_TAG: f32 = 500.0;
/// 자리가 모자라면 큰 글을 이만큼까지 줄인다
const STACK_MIN_SCALE: f32 = 0.6;
/// 말풍선: 글자 크기, 높이, 양옆 여백, 화면 끝과의 틈, 꼬리 크기
const BUBBLE_TEXT: f32 = 24.0;
const BUBBLE_HEIGHT: f32 = 38.0;
const BUBBLE_PAD: f32 = 16.0;
const BUBBLE_MARGIN: f32 = 8.0;
const BUBBLE_TAIL: f32 = 9.0;
/// "만세" 말풍선 아래 끝이 스켈레톤 자리에서 이만큼 위
const CHEER_ABOVE: f32 = 48.0;
/// "얍얍 공격하라" 말풍선 아래 끝 y. 폰의 바꾸기 버튼(y 240~)과 정보판(y ~182) 사이다.
const ZOMBIE_CALL_Y: f32 = 232.0;
/// 휘두를 때 손에 보이는 칼·도끼 크기
const SWING_WEAPON_SIZE: f32 = 46.0;
/// 칼·도끼 그림은 끝이 45도 위를 향해 있다
const WEAPON_ART_ANGLE: f32 = std::f32::consts::FRAC_PI_4;
/// 총 빛줄기 굵기
const GUN_LIGHT_WIDTH: f32 = 7.0;

/// 휘두름: 주먹이 위에서 앞으로 내려오는 각도 (라디안, 0 = 정면)
const SWING_START_ANGLE: f32 = -1.1;
const SWING_END_ANGLE: f32 = 0.7;
const SWING_TRAIL_STEPS: u32 = 8;

const FIST: Color = Color::new(1.0, 0.93, 0.78, 1.0);
const FIST_LINE: Color = Color::new(0.13, 0.16, 0.36, 1.0);
const TRAIL: Color = Color::new(1.0, 1.0, 1.0, 0.8);
const STAR: Color = Color::new(1.0, 0.86, 0.22, 1.0);
const PUFF: Color = Color::new(1.0, 1.0, 1.0, 0.85);
const POP_TEXT: Color = Color::new(1.0, 0.72, 0.20, 1.0);
const SPARK: Color = Color::new(1.0, 0.97, 0.75, 0.9);
const BONE: Color = Color::new(0.96, 0.92, 0.80, 1.0);
const LEVEL_UP_TEXT: Color = Color::new(1.0, 0.86, 0.22, 1.0);
const GUN_LIGHT: Color = Color::new(1.0, 0.95, 0.60, 0.9);
const BUBBLE: Color = Color::new(0.13, 0.16, 0.36, 0.85);
const SMOKE: Color = Color::new(0.92, 0.92, 0.95, 0.75);

#[derive(Clone, Copy, PartialEq)]
enum BurstKind {
    /// 공격으로 쓰러짐: 별이 퍼지고 "쓰러졌어요"
    Defeated,
    /// 팔을 내려 터짐: 구름과 별, "펑!"
    Popped,
    /// 미스를 찾은 자리: 별만 퍼진다
    Found,
    /// 스켈레톤이 피하며 "만세"
    Cheer,
    /// 폭발 시민이 공격에 쓰러짐: 작은 연기만
    Smoke,
    /// 화살·총알이 나가거나 튕길 때 작은 번쩍
    Spark,
    /// 스켈레톤 손이 플레이어에게 쑥
    Hand,
    /// 독약 포션을 뿌림: 초록 구름이 `POISON_RANGE`까지 퍼진다 (US-071)
    PoisonCloud,
    /// 체력 포션: 하트가 퐁퐁 올라간다 (US-071)
    Hearts,
    /// 주사기총: 침 총알이 양쪽으로 날아간다 (US-083)
    Needles,
    /// 엔더열매를 먹음: 금빛 고리 (US-081)
    GoldenPuff,
    /// 자동차를 타고 달리면 가끔 "부릉!" (아이)
    Vroom,
    /// 쿵: 땅에서 물결 고리가 퍼진다 (US-081/082)
    SlamWave,
}

impl BurstKind {
    fn seconds(self) -> f32 {
        match self {
            BurstKind::Defeated | BurstKind::Popped | BurstKind::Found => BURST_SECONDS,
            BurstKind::Spark => SPARK_SECONDS,
            BurstKind::Cheer => CHEER_SECONDS,
            BurstKind::Smoke => SMOKE_SECONDS,
            BurstKind::Hand => HAND_SECONDS,
            BurstKind::PoisonCloud => POISON_CLOUD_SECONDS,
            BurstKind::Hearts => HEARTS_SECONDS,
            BurstKind::Needles => items::NEEDLE_SECONDS,
            BurstKind::GoldenPuff => items::GOLDEN_PUFF_SECONDS,
            BurstKind::Vroom => items::VROOM_SECONDS,
            BurstKind::SlamWave => items::SHAKE_SECONDS * 1.5,
        }
    }
}

#[derive(Clone, Copy)]
struct Burst {
    kind: BurstKind,
    at: Vec2,
    age: f32,
}

#[derive(Default)]
pub struct Effects {
    /// 코인 주울 때 반짝
    pub pops: CoinPops,
    bursts: Vec<Burst>,
    swing_age: Option<f32>,
    /// 휘두를 때 든 것
    swing_weapon: Option<Weapon>,
    /// 큰 문구와 나온 지 지난 시간
    message: Option<(&'static str, f32)>,
    /// 좀비가 나온 맵 x와 지난 시간. 칸이 하나라 새 좀비가 나오면 바뀌어서 말풍선이 겹치지 않는다.
    zombie_call: Option<(f32, f32)>,
    /// 일반 시민마다 (지난 프레임에 걷고 있었는지, 이번 멈춤에 말을 하는지)
    citizen_talk: Vec<(bool, bool)>,
    /// 지금까지 멈춘 횟수. 말할지 뽑는 번호로 쓴다.
    citizen_stops: u32,
    heart_sparkle_age: Option<f32>,
    level_up_age: Option<f32>,
    /// 쿵 글자 아래 줄들(얻은 다이아몬드, 사라진 곡괭이)과 지난 시간. 흔들기도 이 시간으로 정한다.
    slam: Option<(Vec<&'static str>, f32)>,
    /// 자동차를 타고 움직이는 중이면 그 자리 (이번 프레임). `update`가 "부릉!"을 띄운다.
    car_moving_at: Option<Vec2>,
    /// 다음 "부릉!"까지 남은 초
    vroom_wait: f32,
    /// 상자 부수기 효과: 나뭇조각, 도끼 안내, "와우!" (US-089, `view::boxes`)
    pub boxes: BoxView,
    /// 깜짝 미션 말풍선·성공·실패 글 (US-086, `view::mission`)
    pub mission: MissionView,
    /// 방금 준 체력과 지난 시간: 체력 막대 끝 옅은 칸이 줄어든다 (D103)
    hurt: Option<(u32, f32)>,
}

impl Effects {
    pub fn new() -> Effects {
        Effects::default()
    }

    pub fn start_swing(&mut self, weapon: Weapon) {
        self.swing_age = Some(0.0);
        self.swing_weapon = Some(weapon);
    }

    /// 칼·도끼를 휘두르는 중이면 손의 무기는 안 그린다 (휘두르는 그림과 둘로 보인다).
    pub fn swinging_blade(&self) -> bool {
        self.swing_age.is_some() && matches!(self.swing_weapon, Some(Weapon::Sword | Weapon::Axe))
    }

    #[cfg(test)]
    pub fn message(&self) -> Option<&'static str> {
        self.message.map(|(m, _)| m)
    }

    /// 화면 가운데에 큰 문구를 잠깐 보여 준다. 새 문구가 오면 바뀐다.
    pub fn show_message(&mut self, message: &'static str) {
        self.message = Some((message, 0.0));
    }

    /// 일반 시민마다 걷는지를 프레임마다 받아서, 멈추는 순간에 이번에 말을 할지 정한다.
    /// logic에는 "말한다"가 없다. 그림일 뿐이라 여기서 정한다.
    pub fn watch_citizens(&mut self, walking: impl Iterator<Item = bool>) {
        for (i, is_walking) in walking.enumerate() {
            if i == self.citizen_talk.len() {
                self.citizen_talk.push((is_walking, false));
            }
            let (was_walking, talks) = &mut self.citizen_talk[i];
            if *was_walking && !is_walking {
                self.citizen_stops += 1;
                *talks = hash(self.citizen_stops) < CITIZEN_TALK_SHARE;
            }
            *was_walking = is_walking;
        }
    }

    /// `index`번 일반 시민이 이번 멈춤에 말을 하는지.
    pub fn citizen_talks(&self, index: usize) -> bool {
        self.citizen_talk.get(index).is_some_and(|(_, talks)| *talks)
    }

    pub fn on_miss_found(&mut self, at: Vec2) {
        self.burst(BurstKind::Found, at);
        self.show_message(MISS_FOUND_TEXT);
    }

    pub fn on_citizen_popped(&mut self, at: Vec2) {
        self.burst(BurstKind::Popped, at);
    }

    /// 상자 몬스터가 없는 프레임과 같다. 게임 루프는 `on_combat_near_boxes`를 쓴다.
    #[cfg(test)]
    pub fn on_combat(&mut self, event: CombatEvent) {
        self.on_combat_near_boxes(event, &[]);
    }

    /// `box_spots`(이번 프레임에 상자에서 몬스터가 나온 자리)에서 나타난 몬스터는 연기만 낸다.
    /// 그 몬스터는 머리 위 "공격!"만 보이고 "얍얍 공격하라" 말풍선은 띄우지 않는다 (부모님 결정 2026-11-14).
    pub fn on_combat_near_boxes(&mut self, event: CombatEvent, box_spots: &[Vec2]) {
        match event {
            CombatEvent::ZombieAppeared { at } if box_spots.contains(&at) => self.burst(BurstKind::Smoke, at),
            CombatEvent::ZombieDefeated { at } | CombatEvent::SkeletonDefeated { at } => self.burst(BurstKind::Defeated, at),
            CombatEvent::ZombiePopped { at } => self.burst(BurstKind::Popped, at),
            CombatEvent::ProjectileFired { at, .. } | CombatEvent::ProjectileBounced { at } => self.burst(BurstKind::Spark, at),
            CombatEvent::SkeletonHandHit { at } => self.burst(BurstKind::Hand, at),
            CombatEvent::SkeletonDodged { at } => self.burst(BurstKind::Cheer, at),
            CombatEvent::CitizenDefeated { at } => self.burst(BurstKind::Smoke, at),
            // 벽 안쪽에서는 화면 안에 나올 수 있어서 연기와 함께 나타난다
            CombatEvent::ZombieAppeared { at } => {
                self.zombie_call = Some((at.x, 0.0));
                self.burst(BurstKind::Smoke, at);
            }
            CombatEvent::SkeletonAppeared { at, .. } => self.burst(BurstKind::Smoke, at),
            CombatEvent::LevelUp { .. } => self.level_up_age = Some(0.0),
            CombatEvent::HealthRefilled => self.heart_sparkle_age = Some(0.0),
            CombatEvent::PlayerHurt { amount, .. } => self.hurt = Some((amount, 0.0)),
            _ => {}
        }
    }

    /// 체력 포션을 썼다: 정보판 하트가 반짝이고 주인공 위로 하트가 퐁퐁.
    pub fn on_health_potion(&mut self, at: Vec2) {
        self.heart_sparkle_age = Some(0.0);
        self.burst(BurstKind::Hearts, at);
    }

    /// 독약 포션을 뿌렸다: 주인공 둘레로 초록 구름.
    pub fn on_poison_sprayed(&mut self, at: Vec2) {
        self.burst(BurstKind::PoisonCloud, at);
    }

    /// 지금 퍼지는 독 구름 수 (시험용).
    #[cfg(test)]
    pub fn poison_clouds(&self) -> usize {
        self.bursts.iter().filter(|b| b.kind == BurstKind::PoisonCloud).count()
    }

    fn burst(&mut self, kind: BurstKind, at: Vec2) {
        self.bursts.push(Burst { kind, at, age: 0.0 });
    }

    /// 진짜 흐른 시간 `dt`만큼 진행하고, 끝난 효과는 지운다.
    pub fn update(&mut self, dt: f32) {
        self.pops.update(dt);
        self.boxes.update(dt);
        self.mission.update(dt);
        self.hurt = self.hurt.map(|(amount, age)| (amount, age + dt)).filter(|(_, age)| *age < HURT_SHRINK_SECONDS);
        for burst in &mut self.bursts {
            burst.age += dt;
        }
        self.bursts.retain(|b| b.age < b.kind.seconds());
        self.swing_age = self.swing_age.map(|age| age + dt).filter(|age| *age < SWING_SECONDS);
        self.heart_sparkle_age = self.heart_sparkle_age.map(|age| age + dt).filter(|age| *age < HEART_SPARKLE_SECONDS);
        self.level_up_age = self.level_up_age.map(|age| age + dt).filter(|age| *age < LEVEL_UP_SECONDS);
        self.message = self.message.map(|(message, age)| (message, age + dt)).filter(|(_, age)| *age < MESSAGE_SECONDS);
        self.zombie_call = self.zombie_call.map(|(x, age)| (x, age + dt)).filter(|(_, age)| *age < ZOMBIE_CALL_SECONDS);
        if let Some((_, age)) = self.slam.as_mut() {
            *age += dt;
        }
        self.slam = self.slam.take().filter(|(_, age)| *age < items::SLAM_SECONDS);
        match self.car_moving_at {
            Some(at) => {
                self.vroom_wait -= dt;
                if self.vroom_wait <= 0.0 {
                    self.vroom_wait = items::VROOM_EVERY_SECONDS;
                    self.burst(BurstKind::Vroom, at);
                }
            }
            // 서 있다가 다시 달리면 바로 "부릉!"
            None => self.vroom_wait = 0.0,
        }
    }

    /// 자동차를 타고 움직이는지 매 프레임 알려 준다 (`at`은 주인공 몸 가운데). "부릉!"은 `update`가 띄운다.
    pub fn watch_car(&mut self, moving_in_car: bool, at: Vec2) {
        self.car_moving_at = moving_in_car.then_some(at);
    }

    /// 새 조합 물건에 생긴 일을 효과로 보여 준다 (US-080~083).
    pub fn on_item_event(&mut self, event: ItemEvent) {
        match event {
            ItemEvent::CarOff => self.show_message(CAR_OFF_TEXT),
            ItemEvent::SyringeGunFired { at, .. } => self.burst(BurstKind::Needles, at),
            ItemEvent::EnderFruitEaten { at } => {
                self.burst(BurstKind::GoldenPuff, at);
                self.show_message(ENDER_FRUIT_EATEN_TEXT);
            }
            ItemEvent::EnderFruitAlreadyEaten => self.show_message(ENDER_FRUIT_ALREADY_EATEN_TEXT),
            ItemEvent::Slammed { power, at, .. } => {
                let mut lines = vec![power.diamonds_text()];
                if power == SlamPower::DiamondPickaxe {
                    lines.push(DIAMOND_PICKAXE_GONE_TEXT);
                }
                self.slam = Some((lines, 0.0));
                self.burst(BurstKind::SlamWave, at);
            }
        }
    }

    /// 지금 화면 흔들림 (쿵 뒤 잠깐). 맵 그림에만 쓴다.
    pub fn shake(&self) -> Vec2 {
        self.slam.as_ref().map_or(Vec2::ZERO, |(_, age)| items::shake_offset(*age))
    }

    /// 쿵 글 아래 줄들 (시험용)
    #[cfg(test)]
    pub fn slam_lines(&self) -> Option<&[&'static str]> {
        self.slam.as_ref().map(|(lines, _)| lines.as_slice())
    }

    /// 떠 있는 "부릉!" 수 (시험용)
    #[cfg(test)]
    pub fn vrooms(&self) -> usize {
        self.bursts.iter().filter(|b| b.kind == BurstKind::Vroom).count()
    }

    /// 날아가는 침 총알 묶음 수 (시험용)
    #[cfg(test)]
    pub fn needle_volleys(&self) -> usize {
        self.bursts.iter().filter(|b| b.kind == BurstKind::Needles).count()
    }

    /// 체력 막대 끝에 옅게 남길 준 체력 (막 줄었을 때 그만큼, 점점 0으로).
    pub fn lost_hp(&self) -> f32 {
        self.hurt.map_or(0.0, |(amount, age)| amount as f32 * (1.0 - age / HURT_SHRINK_SECONDS).max(0.0))
    }

    /// 하트 반짝임 세기: 1 막 시작 ~ 0 끝.
    pub fn heart_sparkle(&self) -> f32 {
        self.heart_sparkle_age.map_or(0.0, |age| 1.0 - age / HEART_SPARKLE_SECONDS)
    }

    /// 맵 좌표로 그린다. `camera_left`는 화면 왼쪽 끝의 맵 x (말풍선을 화면 안에 두는 데 쓴다).
    /// 효과의 자리는 맵 좌표로 저장되어 있어서, 이음매 너머면 화면에 가까운 쪽으로 옮겨 그린다 (D61).
    /// `attack_reach`는 logic이 준 공격 거리다. 총 빛줄기는 첫 벽에서 멈춘다 (US-073).
    /// `keep_words_off`: 머리 위 "쓰러졌어요"가 가리면 안 되는 칸들 (화면 좌표: 정보판, 큰 알림 글). 겹치면 그 글만 빼고 별은 그린다 (BUG-032).
    #[allow(clippy::too_many_arguments)]
    pub fn draw(&self, text: &TextPainter, player: Rect, facing: Facing, art: &M6Art, camera_left: f32, attack_reach: f32, keep_words_off: &[Rect]) {
        let view_x = camera_left + VIRTUAL_WIDTH / 2.0;
        if let Some(age) = self.swing_age {
            let t = age / SWING_SECONDS;
            match self.swing_weapon.unwrap_or(Weapon::Hands) {
                Weapon::Gun => draw_gun_light(player, facing, t, attack_reach),
                weapon => draw_swing(player, facing, t, weapon, art),
            }
        }
        for burst in &self.bursts {
            let burst = &Burst { at: vec2(near_view(view_x, burst.at.x), burst.at.y), ..*burst };
            match burst.kind {
                BurstKind::Spark => draw_spark(burst),
                BurstKind::Hand => draw_hand(burst, player.center()),
                BurstKind::Defeated => {
                    let words = defeated_words_rect(text, burst.at.x - camera_left, burst.at.y);
                    draw_burst(text, burst, !keep_words_off.iter().any(|r| r.overlaps(&words)));
                }
                BurstKind::Popped | BurstKind::Found => draw_burst(text, burst, true),
                BurstKind::Cheer => draw_bubble(text, SKELETON_DODGE_TEXT, burst.at.x, burst.at.y - CHEER_ABOVE, camera_left, 1.0 - burst.age / CHEER_SECONDS),
                BurstKind::Smoke => draw_smoke(burst),
                BurstKind::PoisonCloud => draw_poison_cloud(burst),
                BurstKind::Hearts => draw_potion_hearts(burst),
                BurstKind::Needles => items::draw_needles(burst.at, burst.age / items::NEEDLE_SECONDS),
                BurstKind::GoldenPuff => items::draw_golden_puff(burst.at, burst.age / items::GOLDEN_PUFF_SECONDS),
                BurstKind::Vroom => items::draw_vroom(text, burst.at + vec2(0.0, -player.h * 0.6), burst.age / items::VROOM_SECONDS),
                BurstKind::SlamWave => items::draw_slam_wave(burst.at, burst.age / (items::SHAKE_SECONDS * 1.5)),
            }
        }
        self.pops.draw(text, view_x);
        self.boxes.draw(text, player, view_x, camera_left);
        if let Some((x, age)) = self.zombie_call {
            // 화면 밖 x라도 `draw_bubble`이 그쪽 가장자리로 당긴다
            let fade = ((1.0 - age / ZOMBIE_CALL_SECONDS) * 3.0).min(1.0);
            draw_bubble(text, ZOMBIE_APPEAR_TEXT, near_view(view_x, x), ZOMBIE_CALL_Y, camera_left, fade);
        }
    }

    /// 지금 보이는 큰 글 덩이들 ("레벨 업!", 쿵 글, 큰 문구 순서)과 큰 문구의 줄·크기.
    fn big_blocks(&self, text: &TextPainter) -> (Vec<BigBlock>, Vec<&'static str>, f32) {
        let message_lines = self.message.map(|(m, _)| message_lines(m)).unwrap_or_default();
        let message_fit = message_lines.iter().map(|l| text.width(l, MESSAGE_SIZE)).fold(0.0, f32::max);
        let message_size = MESSAGE_SIZE * (MESSAGE_MAX_WIDTH / message_fit.max(1.0)).min(1.0);
        let mut blocks = Vec::new();
        if self.level_up_age.is_some() {
            blocks.push(BigBlock { want_top: LEVEL_UP_Y - LEVEL_UP_SIZE * TEXT_ABOVE, height: line_height(LEVEL_UP_SIZE), width: text.width("레벨 업!", LEVEL_UP_SIZE) });
        }
        if let Some((lines, _)) = &self.slam {
            blocks.push(BigBlock { want_top: SLAM_WANT_TOP, height: items::slam_block_height(lines.len()), width: items::slam_block_width(text, lines) });
        }
        if self.message.is_some() {
            blocks.push(BigBlock {
                want_top: MESSAGE_Y - MESSAGE_SIZE * TEXT_ABOVE,
                height: message_height(message_size, message_lines.len()),
                width: message_fit * message_size / MESSAGE_SIZE,
            });
        }
        (blocks, message_lines, message_size)
    }

    /// 큰 글들이 놓일 칸 (화면 좌표). `draw_on_screen`과 같은 자리. 머리 위 "쓰러졌어요"가 이 칸을 피한다 (BUG-032).
    pub fn big_text_rects(&self, text: &TextPainter, info_bottom: f32, keep_clear: Option<Rect>) -> Vec<Rect> {
        let (blocks, ..) = self.big_blocks(text);
        let (tops, scale) = stack_big_texts(&blocks, info_bottom + INFO_GAP, keep_clear);
        blocks.iter().zip(tops).map(|(b, t)| block_rect(b, t, scale)).collect()
    }

    /// 화면에 고정된 효과. 카메라를 푼 뒤에 부른다.
    /// 큰 글("레벨 업!", 쿵 글, 큰 문구)은 위에서부터 차례로 쌓아서 서로 덮지 않게 한다 (BUG-031).
    /// `info_bottom`: 정보판(재료 줄·물건 줄까지)의 아래 끝. 큰 글은 그 아래에만 놓인다.
    /// `keep_clear`: 큰 글이 가리면 안 되는 칸 (집 문 위 E 같은 머리 위 표시, 화면 좌표, BUG-030).
    pub fn draw_on_screen(&self, text: &TextPainter, info_bottom: f32, keep_clear: Option<Rect>) {
        let (blocks, message_lines, message_size) = self.big_blocks(text);
        let (tops, scale) = stack_big_texts(&blocks, info_bottom + INFO_GAP, keep_clear);
        let mut tops = tops.into_iter();
        if let Some(age) = self.level_up_age {
            draw_level_up(text, age / LEVEL_UP_SECONDS, tops.next().unwrap_or(0.0), scale);
        }
        if let Some((lines, age)) = &self.slam {
            items::draw_slam_shout(text, *age, lines, tops.next().unwrap_or(0.0), scale);
        }
        if let Some((_, age)) = self.message {
            let fade = ((1.0 - age / MESSAGE_SECONDS) * 3.0).min(1.0);
            let size = message_size * scale;
            let mut baseline = tops.next().unwrap_or(0.0) + size * TEXT_ABOVE;
            for line in &message_lines {
                text.centered(line, VIRTUAL_WIDTH / 2.0, baseline, size, Color { a: fade, ..LEVEL_UP_TEXT });
                baseline += size * MESSAGE_LINE_STEP;
            }
        }
    }
}

/// 쿵 글 덩이의 바라는 위 끝 (예전 큰 글 바닥선 250 − 글자 크기 52)
const SLAM_WANT_TOP: f32 = 198.0;

/// 글 한 줄 높이 (테두리 포함)
fn line_height(size: f32) -> f32 {
    size * (TEXT_ABOVE + TEXT_BELOW)
}

fn message_height(size: f32, lines: usize) -> f32 {
    line_height(size) + lines.saturating_sub(1) as f32 * size * MESSAGE_LINE_STEP
}

/// 큰 문구를 줄로 나눈다. 글자가 `MESSAGE_SPLIT_CHARS`보다 많으면 두 줄로:
/// "!"·"?"·"." 뒤 띄어쓰기가 있으면 거기서, 없으면 가운데에 가장 가까운 띄어쓰기에서 (BUG-031).
fn message_lines(message: &'static str) -> Vec<&'static str> {
    if message.chars().count() <= MESSAGE_SPLIT_CHARS {
        return vec![message];
    }
    let half = message.chars().count() as f32 / 2.0;
    let spaces = message.char_indices().filter(|(_, c)| *c == ' ').map(|(i, _)| i);
    let after_mark = spaces.clone().find(|i| message[..*i].ends_with(['!', '?', '.']));
    let middle = spaces.min_by(|a, b| {
        let d = |i: &usize| (message[..*i].chars().count() as f32 - half).abs();
        d(a).total_cmp(&d(b))
    });
    match after_mark.or(middle) {
        Some(i) => vec![message[..i].trim_end(), message[i..].trim_start()],
        None => vec![message],
    }
}

/// 이 글자 수(띄어쓰기 포함)까지는 한 줄 (56 크기로 약 850 폭). 그래도 넘치면 `MESSAGE_MAX_WIDTH`에 맞게 줄인다.
const MESSAGE_SPLIT_CHARS: usize = 18;

/// 큰 글 한 덩이: 바라는 위 끝, 높이, 폭 (화면 가운데, 배율 1)
#[derive(Clone, Copy, Debug)]
struct BigBlock {
    want_top: f32,
    height: f32,
    width: f32,
}

/// 큰 글 덩이들의 위 끝과 함께 쓸 배율. 위에서부터 순서대로, `top_min`(정보판 아래) 아래에, 서로 겹치지 않게.
/// `keep_clear`(E 표시)와 겹치면 그 위 칸에 줄여 넣고, 그래도 안 들어가면 그 아래로 보낸다.
fn stack_big_texts(blocks: &[BigBlock], top_min: f32, keep_clear: Option<Rect>) -> (Vec<f32>, f32) {
    let usual = place_in_band(blocks, top_min, STACK_FLOOR);
    let Some(clear) = keep_clear else { return usual };
    if !hits(blocks, &usual, clear) {
        return usual;
    }
    let above = clear.y - MESSAGE_GAP;
    let (tops, scale) = place_in_band(blocks, top_min, above);
    let fits = blocks.last().zip(tops.last()).is_none_or(|(b, t)| t + b.height * scale <= above + 0.01);
    if fits && !hits(blocks, &(tops.clone(), scale), clear) {
        return (tops, scale);
    }
    place_in_band(blocks, top_min.max(clear.bottom() + MESSAGE_GAP), STACK_FLOOR_BELOW_TAG)
}

/// `top`~`bottom` 칸에 덩이들을 쌓는다. 모자라면 `STACK_MIN_SCALE`까지 줄이고, 그래도 넘치면 아래로 넘친다.
fn place_in_band(blocks: &[BigBlock], top: f32, bottom: f32) -> (Vec<f32>, f32) {
    let gaps = STACK_GAP * blocks.len().saturating_sub(1) as f32;
    let total: f32 = blocks.iter().map(|b| b.height).sum();
    let scale = ((bottom - top - gaps) / total.max(1.0)).clamp(STACK_MIN_SCALE, 1.0);
    let mut tops = Vec::with_capacity(blocks.len());
    // 1) 바라는 자리, 정보판 아래, 앞 덩이 아래
    let mut y = top;
    for b in blocks {
        let t = b.want_top.max(y);
        tops.push(t);
        y = t + b.height * scale + STACK_GAP;
    }
    // 2) 아래 끝을 넘으면 위로 당긴다
    let mut limit = bottom;
    for (t, b) in tops.iter_mut().zip(blocks).rev() {
        *t = t.min(limit - b.height * scale);
        limit = *t - STACK_GAP;
    }
    // 3) 그래도 정보판 위·앞 덩이와 겹치면 아래로 (자리가 정말 모자랄 때만 아래 끝을 넘는다)
    let mut y = top;
    for (t, b) in tops.iter_mut().zip(blocks) {
        *t = t.max(y);
        y = *t + b.height * scale + STACK_GAP;
    }
    (tops, scale)
}

fn block_rect(b: &BigBlock, top: f32, scale: f32) -> Rect {
    let w = b.width * scale;
    Rect::new(VIRTUAL_WIDTH / 2.0 - w / 2.0, top, w, b.height * scale)
}

fn hits(blocks: &[BigBlock], placed: &(Vec<f32>, f32), clear: Rect) -> bool {
    blocks.iter().zip(&placed.0).any(|(b, t)| block_rect(b, *t, placed.1).overlaps(&clear))
}

/// 말풍선 하나. 꼬리 끝이 (`x`, `bottom_y`)를 가리키고, 몸은 화면 안으로 당겨진다. 맵 좌표로 그린다.
pub fn draw_bubble(text: &TextPainter, message: &str, x: f32, bottom_y: f32, camera_left: f32, alpha: f32) {
    let w = text.width(message, BUBBLE_TEXT) + BUBBLE_PAD * 2.0;
    let center = x.clamp(camera_left + BUBBLE_MARGIN + w / 2.0, camera_left + VIRTUAL_WIDTH - BUBBLE_MARGIN - w / 2.0);
    let (top, color) = (bottom_y - BUBBLE_TAIL - BUBBLE_HEIGHT, Color { a: BUBBLE.a * alpha, ..BUBBLE });
    draw_pill(center - w / 2.0, top, w, BUBBLE_HEIGHT, color);
    let tail_x = x.clamp(center - w / 2.0 + BUBBLE_PAD, center + w / 2.0 - BUBBLE_PAD);
    let base_y = top + BUBBLE_HEIGHT;
    draw_triangle(vec2(tail_x - BUBBLE_TAIL, base_y), vec2(tail_x + BUBBLE_TAIL, base_y), vec2(tail_x, bottom_y), color);
    text.centered(message, center, top + BUBBLE_HEIGHT - 11.0, BUBBLE_TEXT, Color { a: alpha, ..WHITE });
}

/// 독약 포션 구름: 동글동글 초록 구름이 주인공에서 `POISON_RANGE`까지 퍼졌다가 옅어진다.
fn draw_poison_cloud(burst: &Burst) {
    let t = (burst.age / POISON_CLOUD_SECONDS).clamp(0.0, 1.0);
    let spread = POISON_RANGE * (1.0 - (1.0 - t).powi(3));
    let fade = 1.0 - t * t;
    draw_circle(burst.at.x, burst.at.y, spread, Color { a: 0.12 * fade, ..POISON_CLOUD });
    for i in 0..14 {
        let a = i as f32 / 14.0 * TAU + t;
        let ring = if i % 2 == 0 { 1.0 } else { 0.6 };
        let p = burst.at + vec2(a.cos(), a.sin() * 0.6) * spread * ring;
        draw_circle(p.x, p.y, 16.0 + 14.0 * t, Color { a: POISON_CLOUD.a * fade, ..POISON_CLOUD });
    }
}

/// 체력 포션 하트: 주인공에서 하트 다섯 개가 퐁퐁 올라간다.
fn draw_potion_hearts(burst: &Burst) {
    let t = (burst.age / HEARTS_SECONDS).clamp(0.0, 1.0);
    for i in 0..5 {
        let dx = (i as f32 - 2.0) * 18.0 + (t * 8.0 + i as f32).sin() * 6.0;
        let p = burst.at + vec2(dx, -20.0 - 70.0 * t - (i % 2) as f32 * 14.0);
        super::hud::draw_heart(p, 20.0, Color { a: 1.0 - t, ..POTION_HEART });
        draw_star(p + vec2(10.0, -8.0), 5.0 * (1.0 - t), t * 4.0, Color { a: 1.0 - t, ..STAR });
    }
}

/// 폭발 시민이 공격에 쓰러질 때: 작은 연기가 위로 흩어진다.
fn draw_smoke(burst: &Burst) {
    let t = (burst.age / SMOKE_SECONDS).clamp(0.0, 1.0);
    for i in 0..5 {
        let a = i as f32 / 5.0 * TAU;
        let p = burst.at + vec2(a.cos() * 22.0 * t, a.sin() * 12.0 * t - 26.0 * t);
        draw_circle(p.x, p.y, 13.0 * (1.0 - t * 0.4), Color { a: SMOKE.a * (1.0 - t), ..SMOKE });
    }
}

/// 총: 앞쪽으로 빛줄기가 쭉 나갔다가 사라진다. 닿는 거리는 logic의 공격 네모를 그대로 쓴다.
/// 빛줄기가 시작하는 x와 끝나는 x. 벽에 바로 붙으면 둘이 같아서 줄은 없고 벽에 별만 남는다.
fn gun_light_ends(player: Rect, facing: Facing, attack_reach: f32) -> (f32, f32) {
    let reach = attack_box_with_range(player, facing, attack_reach);
    if facing == Facing::Right {
        (reach.x, reach.x + reach.w)
    } else {
        (reach.x + reach.w, reach.x)
    }
}

fn draw_gun_light(player: Rect, facing: Facing, t: f32, attack_reach: f32) {
    let (from, to) = gun_light_ends(player, facing, attack_reach);
    let y = player.center().y;
    let tip = lerp(from, to, (t * 2.0).min(1.0));
    let fade = 1.0 - t;
    draw_line(from, y, tip, y, GUN_LIGHT_WIDTH * fade + 2.0, Color { a: GUN_LIGHT.a * fade, ..GUN_LIGHT });
    draw_star(vec2(tip, y), 12.0, t * 6.0, Color { a: fade, ..STAR });
}

/// 플레이어 앞에서 주먹이나 칼·도끼가 휙. `t`: 0 시작 ~ 1 끝. 닿는 거리는 logic의 공격 네모를 그대로 쓴다.
fn draw_swing(player: Rect, facing: Facing, t: f32, weapon: Weapon, art: &M6Art) {
    let side = if facing == Facing::Right { 1.0 } else { -1.0 };
    let shoulder = player.center();
    let length = player.w / 2.0 + attack_box(player, facing).w * 0.8;
    let point = |u: f32| {
        let a = lerp(SWING_START_ANGLE, SWING_END_ANGLE, u);
        shoulder + vec2(side * a.cos(), a.sin()) * length
    };
    let fade = 1.0 - t * t;
    for i in 0..SWING_TRAIL_STEPS {
        let (u0, u1) = (t * i as f32 / SWING_TRAIL_STEPS as f32, t * (i + 1) as f32 / SWING_TRAIL_STEPS as f32);
        let (a, b) = (point(u0), point(u1));
        let strength = (i + 1) as f32 / SWING_TRAIL_STEPS as f32;
        draw_line(a.x, a.y, b.x, b.y, 4.0 + 8.0 * strength, Color { a: TRAIL.a * strength * fade, ..TRAIL });
    }
    let fist = point(t);
    if let Some(texture) = art.weapon(weapon) {
        let size = fit_size(texture, SWING_WEAPON_SIZE);
        let angle = lerp(SWING_START_ANGLE, SWING_END_ANGLE, t) + WEAPON_ART_ANGLE;
        draw_texture_ex(
            texture,
            fist.x - size.x / 2.0,
            fist.y - size.y / 2.0,
            Color { a: fade, ..WHITE },
            DrawTextureParams { dest_size: Some(size), rotation: angle * side, flip_x: flip_weapon(weapon, facing), ..Default::default() },
        );
        return;
    }
    draw_circle(fist.x, fist.y, 13.0, Color { a: fade, ..FIST_LINE });
    draw_circle(fist.x, fist.y, 10.5, Color { a: fade, ..FIST });
}

/// 화살·총알이 나갈 때·튕길 때: 빛이 퍼지며 사라진다.
fn draw_spark(burst: &Burst) {
    let t = burst.age / SPARK_SECONDS;
    draw_circle(burst.at.x, burst.at.y, 5.0 + 10.0 * t, Color { a: SPARK.a * (1.0 - t), ..SPARK });
    draw_star(burst.at, 9.0 * (1.0 - t) + 2.0, t * 3.0, Color { a: 1.0 - t, ..STAR });
}

/// 스켈레톤 손: 몸에서 플레이어 쪽으로 쑥 나가고, 닿으면 별이 튄다. 손가락 세 개 달린 뼈 손.
fn draw_hand(burst: &Burst, target: Vec2) {
    let t = burst.age / HAND_SECONDS;
    let fade = 1.0 - t * t;
    let p = burst.at.lerp(target, (t / HAND_REACH_SHARE).min(1.0));
    let forward = (target - burst.at).normalize_or_zero();
    for finger in [-0.6_f32, 0.0, 0.6] {
        let tip = p + Vec2::from_angle(finger).rotate(forward) * 15.0;
        draw_line(p.x, p.y, tip.x, tip.y, 5.0, Color { a: fade, ..FIST_LINE });
        draw_line(p.x, p.y, tip.x, tip.y, 3.0, Color { a: fade, ..BONE });
    }
    draw_circle(p.x, p.y, 9.0, Color { a: fade, ..FIST_LINE });
    draw_circle(p.x, p.y, 7.0, Color { a: fade, ..BONE });
    if t >= HAND_REACH_SHARE {
        for i in 0..5 {
            let a = i as f32 / 5.0 * TAU + 0.4;
            draw_star(p + vec2(a.cos(), a.sin()) * 34.0 * t, 8.0, t * 4.0 + i as f32, Color { a: fade, ..STAR });
        }
    }
}

/// "레벨 업!": 큰 글자 둘레로 별이 돌고, 끝날 때쯤 흐려진다. `t`: 0 시작 ~ 1 끝. `top`·`scale`은 쌓은 자리 (BUG-031).
fn draw_level_up(text: &TextPainter, t: f32, top: f32, scale: f32) {
    let fade = ((1.0 - t) * 3.0).min(1.0);
    let size = LEVEL_UP_SIZE * scale;
    let baseline = top + size * TEXT_ABOVE;
    let center = vec2(VIRTUAL_WIDTH / 2.0, baseline - size / 3.0);
    let open = (t * 4.0).min(1.0);
    for i in 0..12 {
        let a = i as f32 / 12.0 * TAU + t * 2.0;
        let p = center + vec2(a.cos(), a.sin()) * LEVEL_UP_RING * size * open;
        draw_star(p, 13.0 * scale, t * 6.0 + i as f32, Color { a: fade, ..STAR });
    }
    text.centered("레벨 업!", VIRTUAL_WIDTH / 2.0, baseline, size, Color { a: fade, ..LEVEL_UP_TEXT });
}

/// 쓰러진 몬스터 위 "쓰러졌어요" 글 크기와 처음 자리 (몬스터 가운데에서 위로), 떠오르는 높이
const DEFEATED_SIZE: f32 = 30.0;
const DEFEATED_ABOVE: f32 = 40.0;
const DEFEATED_RISE: f32 = 36.0;

/// "쓰러졌어요" 바닥선: 처음 자리에서 시간이 갈수록 위로 떠오른다.
fn defeated_baseline(y: f32, age: f32) -> f32 {
    y - DEFEATED_ABOVE - DEFEATED_RISE * (age / BURST_SECONDS).clamp(0.0, 1.0)
}

/// "쓰러졌어요"가 떠오르며 지나가는 칸 전체 (화면 좌표, 처음 자리부터 끝 자리까지). 이 칸이 정보판·큰 글과 겹치면 글을 뺀다 (BUG-032).
/// 처음부터 끝까지 한 칸으로 보아서, 떠오르다 중간에 사라지지 않고 아예 안 나온다.
fn defeated_words_rect(text: &TextPainter, screen_x: f32, y: f32) -> Rect {
    words_span(screen_x, y, text.width("쓰러졌어요", DEFEATED_SIZE))
}

fn words_span(screen_x: f32, y: f32, width: f32) -> Rect {
    let top = defeated_baseline(y, BURST_SECONDS) - DEFEATED_SIZE * TEXT_ABOVE;
    let bottom = defeated_baseline(y, 0.0) + DEFEATED_SIZE * TEXT_BELOW;
    Rect::new(screen_x - width / 2.0, top, width, bottom - top)
}

/// 별이 퍼지는 효과. `words`가 false면 "쓰러졌어요"·"펑!" 글은 빼고 별만.
fn draw_burst(text: &TextPainter, burst: &Burst, words: bool) {
    let t = (burst.age / BURST_SECONDS).clamp(0.0, 1.0);
    let fade = 1.0 - t;
    let spread = 10.0 + 46.0 * (1.0 - fade * fade);
    if burst.kind == BurstKind::Popped {
        for i in 0..6 {
            let a = i as f32 / 6.0 * TAU;
            let p = burst.at + vec2(a.cos(), a.sin()) * spread * 0.7;
            draw_circle(p.x, p.y, 20.0 * (1.0 - t * 0.5), Color { a: PUFF.a * fade, ..PUFF });
        }
    }
    for i in 0..8 {
        let a = i as f32 / 8.0 * TAU + 0.3;
        let p = burst.at + vec2(a.cos(), a.sin()) * spread;
        draw_star(p, 9.0 * fade + 3.0, t * 5.0 + i as f32, Color { a: fade, ..STAR });
    }
    if !words {
        return;
    }
    let rise = DEFEATED_RISE * t;
    match burst.kind {
        BurstKind::Defeated => text.centered("쓰러졌어요", burst.at.x, defeated_baseline(burst.at.y, burst.age), DEFEATED_SIZE, Color { a: fade, ..WHITE }),
        BurstKind::Popped => text.centered(CITIZEN_POP_TEXT, burst.at.x, burst.at.y - 30.0 - rise, 48.0, Color { a: fade, ..POP_TEXT }),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn potions_show_a_cloud_and_hearts_for_a_moment() {
        let mut effects = Effects::new();
        effects.on_poison_sprayed(vec2(100.0, 300.0));
        effects.on_health_potion(vec2(100.0, 300.0));
        assert_eq!(effects.poison_clouds(), 1);
        assert!(effects.heart_sparkle() > 0.99, "정보판 하트가 반짝인다");
        for _ in 0..120 {
            effects.update(1.0 / 60.0);
        }
        assert_eq!(effects.poison_clouds(), 0, "2초 뒤에는 사라진다");
        assert_eq!(effects.heart_sparkle(), 0.0);
    }

    #[test]
    fn gun_light_stops_at_the_reach_logic_gives() {
        let body = Rect::new(100.0, 300.0, 48.0, 64.0);
        assert_eq!(gun_light_ends(body, Facing::Right, 30.0), (148.0, 178.0));
        assert_eq!(gun_light_ends(body, Facing::Left, 30.0), (100.0, 70.0));
        assert_eq!(gun_light_ends(body, Facing::Right, 0.0), (148.0, 148.0), "벽에 붙으면 길이 0");
        assert_eq!(gun_light_ends(body, Facing::Left, 0.0), (100.0, 100.0));
    }

    #[test]
    fn bursts_and_sparkle_come_from_events_and_go_away() {
        let mut e = Effects::new();
        e.on_combat(CombatEvent::ZombieDefeated { at: Vec2::ZERO });
        e.on_combat(CombatEvent::ZombiePopped { at: Vec2::ZERO });
        e.on_combat(CombatEvent::ZombieHit { at: Vec2::ZERO, hp_left: 1 });
        e.on_combat(CombatEvent::HealthRefilled);
        assert_eq!(e.bursts.len(), 2, "맞기만 한 좀비는 번쩍이기만 한다");
        assert_eq!(e.heart_sparkle(), 1.0);
        e.update(BURST_SECONDS + 0.01);
        assert!(e.bursts.is_empty());
        e.update(HEART_SPARKLE_SECONDS);
        assert_eq!(e.heart_sparkle(), 0.0);
    }

    #[test]
    fn skeleton_events_show_effects_and_go_away() {
        let mut e = Effects::new();
        e.on_combat(CombatEvent::SkeletonDefeated { at: Vec2::ZERO });
        e.on_combat(CombatEvent::SkeletonHandHit { at: Vec2::ZERO });
        e.on_combat(CombatEvent::ProjectileBounced { at: Vec2::ZERO });
        e.on_combat(CombatEvent::SkeletonHit { at: Vec2::ZERO, hp_left: 2 });
        e.on_combat(CombatEvent::LevelUp { level: 2 });
        assert_eq!(e.bursts.len(), 3);
        e.update(SPARK_SECONDS);
        assert_eq!(e.bursts.len(), 2, "번쩍은 먼저 사라진다");
        e.update(BURST_SECONDS);
        assert!(e.bursts.is_empty());
        assert!(e.level_up_age.is_some());
        e.update(LEVEL_UP_SECONDS);
        assert!(e.level_up_age.is_none());
    }

    #[test]
    fn miss_and_weapon_messages_show_then_go_away() {
        let mut e = Effects::new();
        e.on_miss_found(Vec2::ZERO);
        assert_eq!(e.message.map(|m| m.0), Some(MISS_FOUND_TEXT));
        e.show_message(Weapon::Gun.picked_text());
        assert_eq!(e.message.map(|m| m.0), Some(Weapon::Gun.picked_text()), "새 문구로 바뀐다");
        e.on_citizen_popped(Vec2::ZERO);
        assert_eq!(e.bursts.len(), 2);
        e.update(MESSAGE_SECONDS);
        assert!(e.message.is_none() && e.bursts.is_empty());
    }

    #[test]
    fn only_one_zombie_call_at_a_time() {
        let mut e = Effects::new();
        e.on_combat(CombatEvent::ZombieAppeared { at: vec2(100.0, 0.0) });
        e.update(1.0);
        e.on_combat(CombatEvent::ZombieAppeared { at: vec2(900.0, 0.0) });
        assert_eq!(e.zombie_call, Some((900.0, 0.0)), "새 좀비로 바뀐다");
        e.update(ZOMBIE_CALL_SECONDS);
        assert_eq!(e.zombie_call, None);
    }

    fn message_block(width: f32) -> BigBlock {
        BigBlock { want_top: MESSAGE_Y - MESSAGE_SIZE * TEXT_ABOVE, height: message_height(MESSAGE_SIZE, 1), width }
    }

    fn rects(blocks: &[BigBlock], placed: &(Vec<f32>, f32)) -> Vec<Rect> {
        blocks.iter().zip(&placed.0).map(|(b, t)| block_rect(b, *t, placed.1)).collect()
    }

    /// 정보판이 짧을 때 (재료·물건 줄 없음): 체력·레벨 줄 아래 끝
    const SHORT_INFO: f32 = 122.0 + INFO_GAP;
    /// 정보판이 3줄일 때: 무기가 있고 물건 줄 아래 끝 238 (BUG-031)
    const TALL_INFO: f32 = 238.0 + INFO_GAP;

    #[test]
    fn big_message_moves_above_the_door_e_bug_030() {
        let blocks = [message_block(380.0)];
        let usual = stack_big_texts(&blocks, SHORT_INFO, None);
        assert_eq!(usual, (vec![MESSAGE_Y - MESSAGE_SIZE], 1.0), "아무것도 없으면 늘 자리");
        // 화면 가운데 집 문(위 328) 위의 E 칸: 표시 칸 아래 끝 319, 위 끝 253
        let door_tag = Rect::new(VIRTUAL_WIDTH / 2.0 - 75.0, 253.0, 150.0, 66.0);
        let placed = stack_big_texts(&blocks, SHORT_INFO, Some(door_tag));
        let r = rects(&blocks, &placed)[0];
        assert!(!r.overlaps(&door_tag), "글과 E가 안 겹친다 {r:?}");
        assert!(r.y >= SHORT_INFO && placed.1 == 1.0, "정보판 아래, 줄이지 않는다 {r:?}");
        // 화면 구석의 표시는 피할 필요가 없다
        let far = Rect { x: 10.0, ..door_tag };
        assert_eq!(stack_big_texts(&blocks, SHORT_INFO, Some(far)), usual);
    }

    #[test]
    fn big_texts_stay_below_a_tall_info_panel_and_apart_bug_031() {
        let level_up = BigBlock { want_top: LEVEL_UP_Y - LEVEL_UP_SIZE, height: line_height(LEVEL_UP_SIZE), width: 300.0 };
        let slam = BigBlock { want_top: SLAM_WANT_TOP, height: items::slam_block_height(2), width: 920.0 };
        let message = message_block(700.0);
        let door_tag = Rect::new(VIRTUAL_WIDTH / 2.0 - 75.0, 253.0, 150.0, 66.0);
        let sets: [&[BigBlock]; 5] = [&[level_up], &[slam], &[message], &[slam, message], &[level_up, slam, message]];
        for blocks in sets {
            for clear in [None, Some(door_tag)] {
                let placed = stack_big_texts(blocks, TALL_INFO, clear);
                let rs = rects(blocks, &placed);
                assert!(placed.1 >= STACK_MIN_SCALE && placed.1 <= 1.0);
                for (i, r) in rs.iter().enumerate() {
                    assert!(r.y >= TALL_INFO - 0.01, "정보판 아래 {r:?}");
                    assert!(r.x >= 0.0 && r.right() <= VIRTUAL_WIDTH, "화면 안 {r:?}");
                    assert!(rs[..i].iter().all(|o| !o.overlaps(r)), "알림끼리 안 덮는다 {rs:?}");
                    if blocks.len() < 3 {
                        assert!(r.bottom() <= STACK_FLOOR_BELOW_TAG + 0.01, "{r:?}");
                        if let Some(tag) = clear {
                            assert!(!r.overlaps(&tag), "E 표시를 안 가린다 {r:?}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn long_message_goes_on_two_lines_bug_031() {
        assert_eq!(message_lines(ENDER_FRUIT_EATEN_TEXT), vec!["애서열매를 먹었어요!", "해변 바닷가에서 쿵 해 봐요"]);
        assert_eq!(message_lines(ENDER_FRUIT_ALREADY_EATEN_TEXT), vec![ENDER_FRUIT_ALREADY_EATEN_TEXT]);
        assert_eq!(message_lines(CAR_OFF_TEXT), vec![CAR_OFF_TEXT], "18자 이하는 한 줄");
        assert_eq!(message_lines("가나다라 마바사아 자차카타 파하가나"), vec!["가나다라 마바사아", "자차카타 파하가나"], "가운데 띄어쓰기");
        for text in [ENDER_FRUIT_EATEN_TEXT, CAR_OFF_TEXT, MISS_FOUND_TEXT] {
            assert!(message_lines(text).iter().all(|l| l.chars().count() <= MESSAGE_SPLIT_CHARS), "{text}");
        }
    }

    #[test]
    fn defeated_words_skip_big_texts_and_the_info_panel_bug_032() {
        // 쓰러진 몬스터 위 "쓰러졌어요"가 떠오르는 칸 전체: 처음 자리와 끝 자리를 다 덮는다
        let y = 400.0;
        let span = words_span(480.0, y, 150.0);
        assert!(span.y <= defeated_baseline(y, BURST_SECONDS) - DEFEATED_SIZE && span.bottom() >= defeated_baseline(y, 0.0));
        // 쿵 글 덩이(작은 줄 포함)가 그 자리에 쌓이면 겹친다 → 글을 뺀다
        let blocks = [BigBlock { want_top: SLAM_WANT_TOP, height: items::slam_block_height(3), width: 900.0 }];
        let (tops, scale) = stack_big_texts(&blocks, SHORT_INFO, None);
        let slam = block_rect(&blocks[0], tops[0], scale);
        let low = words_span(480.0, slam.bottom() + 40.0, 150.0);
        assert!(low.overlaps(&slam), "쿵 작은 줄 바로 아래 몬스터의 글은 쿵 글과 겹친다 (그래서 뺀다)");
        let far = words_span(480.0, 440.0 + 200.0, 150.0);
        assert!(!far.overlaps(&slam), "멀리 있으면 그대로 보인다");
        // 정보판 칸과 겹치면 뺀다 (상자 위 높은 몬스터)
        let info = Rect::new(0.0, 0.0, VIRTUAL_WIDTH, SHORT_INFO);
        assert!(words_span(480.0, SHORT_INFO + 50.0, 150.0).overlaps(&info));
    }

    #[test]
    fn slam_and_car_off_at_the_same_time_both_show() {
        let mut e = Effects::new();
        e.on_item_event(ItemEvent::Slammed { power: SlamPower::EnderFruit, at: Vec2::ZERO, monsters: 0, diamonds: 10 });
        e.on_item_event(ItemEvent::CarOff);
        assert!(e.slam_lines().is_some() && e.message() == Some(CAR_OFF_TEXT), "둘 다 떠 있고 따로 쌓인다");
    }

    #[test]
    fn box_zombie_shows_only_smoke_not_the_call() {
        let mut e = Effects::new();
        let at = vec2(300.0, 400.0);
        e.on_combat_near_boxes(CombatEvent::ZombieAppeared { at }, &[at]);
        assert_eq!(e.zombie_call, None, "상자 좀비는 \"얍얍 공격하라\"가 없다");
        assert_eq!(e.bursts.len(), 1, "연기는 난다");
        e.on_combat_near_boxes(CombatEvent::ZombieAppeared { at: vec2(900.0, 400.0) }, &[at]);
        assert_eq!(e.zombie_call, Some((900.0, 0.0)), "밤 좀비는 그대로 말풍선");
    }

    #[test]
    fn cheer_and_smoke_go_away() {
        let mut e = Effects::new();
        e.on_combat(CombatEvent::SkeletonDodged { at: Vec2::ZERO });
        e.on_combat(CombatEvent::CitizenDefeated { at: Vec2::ZERO });
        assert_eq!(e.bursts.len(), 2);
        e.update(CHEER_SECONDS);
        assert!(e.bursts.is_empty());
    }

    #[test]
    fn about_half_of_the_stops_talk_and_it_stays_during_a_stop() {
        let mut e = Effects::new();
        let mut talked = 0;
        for _ in 0..200 {
            e.watch_citizens([true].into_iter());
            e.watch_citizens([false].into_iter());
            let talks = e.citizen_talks(0);
            e.watch_citizens([false].into_iter());
            assert_eq!(e.citizen_talks(0), talks, "멈춘 동안은 그대로다");
            talked += talks as u32;
        }
        assert!((60..=140).contains(&talked), "{talked}");
    }

    #[test]
    fn swing_ends_by_itself() {
        let mut e = Effects::new();
        e.start_swing(Weapon::Sword);
        e.update(SWING_SECONDS / 2.0);
        assert!(e.swing_age.is_some());
        e.update(SWING_SECONDS);
        assert!(e.swing_age.is_none());
    }

    #[test]
    fn slam_shows_shout_lines_and_shakes_for_a_moment() {
        let mut e = Effects::new();
        assert_eq!(e.shake(), Vec2::ZERO);
        e.on_item_event(ItemEvent::Slammed { power: SlamPower::DiamondPickaxe, at: vec2(300.0, 400.0), monsters: 2, diamonds: 1 });
        assert_eq!(e.slam_lines(), Some(&[crate::config::DIAMOND_PICKAXE_SLAM_DIAMONDS_TEXT, DIAMOND_PICKAXE_GONE_TEXT][..]), "곡괭이는 사라졌다고 알려 준다");
        e.update(0.05);
        assert!(e.shake().length() > 0.0, "쿵 하면 화면이 흔들린다");
        e.update(items::SHAKE_SECONDS);
        assert_eq!(e.shake(), Vec2::ZERO, "흔들기는 잠깐");
        assert!(e.slam_lines().is_some(), "글은 조금 더 남는다");
        e.update(items::SLAM_SECONDS);
        assert!(e.slam_lines().is_none());
        e.on_item_event(ItemEvent::Slammed { power: SlamPower::EnderFruit, at: Vec2::ZERO, monsters: 0, diamonds: 10 });
        assert_eq!(e.slam_lines(), Some(&[crate::config::ENDER_FRUIT_SLAM_DIAMONDS_TEXT][..]), "엔더열매 쿵은 다이아몬드 글만");
    }

    #[test]
    fn item_events_show_needles_and_messages() {
        let mut e = Effects::new();
        e.on_item_event(ItemEvent::SyringeGunFired { at: vec2(200.0, 400.0), monsters: 3 });
        assert_eq!(e.needle_volleys(), 1, "침 총알이 날아간다");
        e.update(items::NEEDLE_SECONDS + 0.01);
        assert_eq!(e.needle_volleys(), 0);
        e.on_item_event(ItemEvent::EnderFruitEaten { at: Vec2::ZERO });
        assert_eq!(e.message(), Some(ENDER_FRUIT_EATEN_TEXT));
        e.on_item_event(ItemEvent::EnderFruitAlreadyEaten);
        assert_eq!(e.message(), Some(ENDER_FRUIT_ALREADY_EATEN_TEXT));
        e.on_item_event(ItemEvent::CarOff);
        assert_eq!(e.message(), Some(CAR_OFF_TEXT));
    }

    #[test]
    fn vroom_pops_up_now_and_then_only_while_driving() {
        let mut e = Effects::new();
        let frame = 1.0 / 60.0;
        e.watch_car(true, vec2(100.0, 400.0));
        e.update(frame);
        assert_eq!(e.vrooms(), 1, "달리기 시작하면 바로 부릉!");
        let mut made = 1;
        let mut last = 1;
        for _ in 0..(items::VROOM_EVERY_SECONDS * 3.0 / frame) as usize {
            e.update(frame);
            let now = e.vrooms();
            if now > last {
                made += 1;
            }
            last = now;
            assert!(now <= 1, "한 번에 하나만 떠 있다");
        }
        assert!((3..=4).contains(&made), "가끔만: {made}");
        e.watch_car(false, Vec2::ZERO);
        for _ in 0..120 {
            e.update(frame);
        }
        assert_eq!(e.vrooms(), 0, "서 있거나 내리면 안 나온다");
    }
}
