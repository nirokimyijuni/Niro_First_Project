//! 스켈레톤 (D20, D35): 밤에 화면 끝에서 나타나 걸어오고, 단계마다 공격이 다르다.
//!
//! - 일반은 화살, 절반은 손(닿으면 맞음), 완전체는 총. 레벨마다 그 단계만 나온다.
//! - 화살·총알은 땅에서 일정한 높이로 곧게 난다. 그래서 작은 상자는 넘어가고 큰 상자에서 튕긴다 (US-049).
//! - 튕겨 온 공격은 스켈레톤도 맞는다. 가까이 오면 점프해서 피한다.
//! - 플레이어 체력과 경험치는 `combat.rs`가 맡는다.

use super::combat::CombatEvent;
use super::monster::walk_x;
use super::world::{really_overlaps, World};
use crate::config::{
    ARROW_INTERVAL_SECONDS, ARROW_SPEED, ARROW_WIDTH, BULLET_INTERVAL_SECONDS, BULLET_SPEED, BULLET_WIDTH,
    FORGET_DISTANCE, GRAVITY, PROJECTILE_BOUNCED_RANGE, PROJECTILE_FLIGHT_HEIGHT, PROJECTILE_HEIGHT, SKELETON_DODGE_JUMP_SPEED,
    SKELETON_DODGE_SENSE_DISTANCE, SKELETON_FULL_ATTACK, SKELETON_FULL_MIN_LEVEL, SKELETON_HALF_ATTACK,
    SKELETON_HALF_MIN_LEVEL, SKELETON_HEIGHT, SKELETON_HIT_FLASH_SECONDS, SKELETON_HP, SKELETON_NORMAL_ATTACK,
    SKELETON_SHOOT_DISTANCE, SKELETON_WALK_SPEED, SKELETON_WIDTH,
};
use macroquad::math::{Rect, Vec2};

/// 스켈레톤 단계. 화면은 이것을 보고 그림을 고른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkeletonStage {
    /// 일반: 화살을 쏜다.
    Normal,
    /// 절반: 걸어와서 손으로 때린다.
    Half,
    /// 완전체: 총을 쏜다.
    Full,
}

impl SkeletonStage {
    /// 몬스터 레벨에 나오는 단계.
    pub fn for_level(level: u32) -> SkeletonStage {
        if level >= SKELETON_FULL_MIN_LEVEL {
            SkeletonStage::Full
        } else if level >= SKELETON_HALF_MIN_LEVEL {
            SkeletonStage::Half
        } else {
            SkeletonStage::Normal
        }
    }

    pub fn attack(self) -> u32 {
        match self {
            SkeletonStage::Normal => SKELETON_NORMAL_ATTACK,
            SkeletonStage::Half => SKELETON_HALF_ATTACK,
            SkeletonStage::Full => SKELETON_FULL_ATTACK,
        }
    }

    /// 쏘는 것. 손으로 때리는 절반은 없다.
    fn weapon(self) -> Option<ProjectileKind> {
        match self {
            SkeletonStage::Normal => Some(ProjectileKind::Arrow),
            SkeletonStage::Half => None,
            SkeletonStage::Full => Some(ProjectileKind::Bullet),
        }
    }
}

/// 스켈레톤이 지금 무엇을 하는지. 화면은 이것을 보고 움직임을 고른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkeletonState {
    Walking,
    /// 쏘기 좋은 거리에 왔거나 상자에 막혀 서 있다.
    Standing,
    /// 튕겨 온 공격을 피해 점프하는 중. 이때는 걷지도 쏘지도 않는다.
    Dodging,
}

pub struct Skeleton {
    /// 스켈레톤 네모의 왼쪽 위 모서리.
    pub pos: Vec2,
    pub hp: u32,
    pub stage: SkeletonStage,
    pub state: SkeletonState,
    /// 위아래 속도 (초당 픽셀). 피해서 점프하는 동안만 쓴다.
    fall_speed: f32,
    seconds_to_next_shot: f32,
    hit_flash_seconds_left: f32,
}

impl Skeleton {
    /// 땅 위 `x`에 서 있는 스켈레톤. 첫 공격은 쏘는 간격만큼 지나서 나온다.
    pub fn new(x: f32, ground_y: f32, stage: SkeletonStage) -> Skeleton {
        Skeleton {
            pos: Vec2::new(x, ground_y - SKELETON_HEIGHT),
            hp: SKELETON_HP,
            stage,
            state: SkeletonState::Walking,
            fall_speed: 0.0,
            seconds_to_next_shot: stage.weapon().map_or(0.0, ProjectileKind::interval),
            hit_flash_seconds_left: 0.0,
        }
    }

    /// 스켈레톤이 차지하는 네모 (왼쪽 위 기준).
    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, SKELETON_WIDTH, SKELETON_HEIGHT)
    }

    /// 막 맞아서 반짝여야 하면 true.
    pub fn is_flashing(&self) -> bool {
        self.hit_flash_seconds_left > 0.0
    }

    /// 공격을 맞아 체력이 `attack`만큼 줄고, 화면에 알릴 일을 돌려준다.
    pub fn take_hit(&mut self, attack: u32) -> CombatEvent {
        self.hp = self.hp.saturating_sub(attack);
        self.hit_flash_seconds_left = SKELETON_HIT_FLASH_SECONDS;
        let at = self.rect().center();
        if self.hp == 0 {
            CombatEvent::SkeletonDefeated { at }
        } else {
            CombatEvent::SkeletonHit { at, hp_left: self.hp }
        }
    }

    fn step(&mut self, step: f32, player_x: f32, world: &World, shots: &mut Vec<Projectile>, events: &mut Vec<CombatEvent>) {
        self.hit_flash_seconds_left = (self.hit_flash_seconds_left - step).max(0.0);
        if self.state == SkeletonState::Dodging {
            self.jump(step, world);
            return;
        }

        let weapon = self.stage.weapon();
        let distance = (player_x - self.rect().center().x).abs();
        let old_x = self.pos.x;
        if weapon.is_none() || distance > SKELETON_SHOOT_DISTANCE {
            self.pos.x = walk_x(self.rect(), player_x, SKELETON_WALK_SPEED * step, world);
        }
        self.state = if self.pos.x == old_x { SkeletonState::Standing } else { SkeletonState::Walking };

        if let Some(kind) = weapon {
            self.seconds_to_next_shot -= step;
            // 임시-4: 화면 밖에서는 쏘지 않고, 들어오는 대로 쏜다.
            if self.seconds_to_next_shot <= 0.0 && !world.is_on_screen(self.rect().center().x, player_x) {
                self.seconds_to_next_shot = 0.0;
            } else if self.seconds_to_next_shot <= 0.0 {
                self.seconds_to_next_shot += kind.interval();
                let shot = Projectile::fired_by(self, kind, player_x, world);
                events.push(CombatEvent::ProjectileFired { at: shot.rect().center(), kind });
                shots.push(shot);
            }
        }
    }

    fn start_dodge(&mut self) {
        self.state = SkeletonState::Dodging;
        self.fall_speed = -SKELETON_DODGE_JUMP_SPEED;
    }

    fn jump(&mut self, step: f32, world: &World) {
        self.fall_speed += GRAVITY * step;
        self.pos.y += self.fall_speed * step;
        let standing_y = world.ground_y - SKELETON_HEIGHT;
        if self.pos.y >= standing_y {
            self.pos.y = standing_y;
            self.state = SkeletonState::Standing;
        }
    }
}

/// 날아가는 공격의 종류. 화면은 이것을 보고 그림을 고른다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileKind {
    Arrow,
    Bullet,
}

impl ProjectileKind {
    fn interval(self) -> f32 {
        match self {
            ProjectileKind::Arrow => ARROW_INTERVAL_SECONDS,
            ProjectileKind::Bullet => BULLET_INTERVAL_SECONDS,
        }
    }

    fn speed(self) -> f32 {
        match self {
            ProjectileKind::Arrow => ARROW_SPEED,
            ProjectileKind::Bullet => BULLET_SPEED,
        }
    }

    fn width(self) -> f32 {
        match self {
            ProjectileKind::Arrow => ARROW_WIDTH,
            ProjectileKind::Bullet => BULLET_WIDTH,
        }
    }
}

/// 날아가는 화살이나 총알.
pub struct Projectile {
    /// 네모의 왼쪽 위 모서리.
    pub pos: Vec2,
    pub kind: ProjectileKind,
    /// 맞으면 줄어드는 플레이어 체력 (쏜 스켈레톤의 공격력).
    pub attack: u32,
    /// 날아가는 쪽. 1이면 오른쪽, -1이면 왼쪽. 화면은 이것으로 그림을 뒤집는다.
    pub direction: f32,
    /// 상자에 튕겼으면 true. 튕긴 공격만 스켈레톤이 맞는다 (US-049).
    pub bounced: bool,
    distance_since_bounce: f32,
}

impl Projectile {
    /// `y`는 땅에서 정해진 높이다 → 작은 상자는 넘어가고 큰 상자는 맞는다.
    pub fn new(x: f32, direction: f32, kind: ProjectileKind, attack: u32, world: &World) -> Projectile {
        let y = world.ground_y - PROJECTILE_FLIGHT_HEIGHT - PROJECTILE_HEIGHT / 2.0;
        Projectile { pos: Vec2::new(x, y), kind, attack, direction, bounced: false, distance_since_bounce: 0.0 }
    }

    /// 스켈레톤 몸 앞에서 플레이어 쪽으로 나간다. 몸이 거의 겹치면 플레이어 자리에서 나간다:
    /// 몸 앞은 플레이어보다 멀어서 맞지 않고, 맵 끝에서는 나가자마자 사라진다 (엘리 관찰: 왼쪽 끝).
    fn fired_by(skeleton: &Skeleton, kind: ProjectileKind, player_x: f32, world: &World) -> Projectile {
        let body = skeleton.rect();
        let (front_x, direction) = if player_x < body.center().x {
            (body.x - kind.width(), -1.0)
        } else {
            (body.x + body.w, 1.0)
        };
        let is_touching = (player_x - body.center().x).abs() < body.w / 2.0;
        let x = if is_touching { player_x - kind.width() / 2.0 } else { front_x };
        Projectile::new(x, direction, kind, skeleton.stage.attack(), world)
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.kind.width(), PROJECTILE_HEIGHT)
    }

    /// 짧은 시간 `step` 동안 날아간다. 아직 날고 있으면 true.
    fn step(&mut self, step: f32, world: &World, events: &mut Vec<CombatEvent>) -> bool {
        let moved = self.kind.speed() * step;
        self.pos.x += self.direction * moved;
        if self.bounced {
            self.distance_since_bounce += moved;
        }

        let body = self.rect();
        if let Some(wall) = world.obstacles.iter().find(|wall| really_overlaps(body, **wall)) {
            if self.bounced {
                return false; // 튕기는 건 한 번뿐이다.
            }
            self.pos.x = if self.direction > 0.0 { wall.x - body.w } else { wall.x + wall.w };
            self.direction = -self.direction;
            self.bounced = true;
            events.push(CombatEvent::ProjectileBounced { at: self.rect().center() });
        }

        let on_screen = body.x + body.w > 0.0 && body.x < world.width;
        on_screen && self.distance_since_bounce < PROJECTILE_BOUNCED_RANGE
    }

    /// 튕겨서 `body` 쪽으로 오고 있고, 피해야 할 만큼 가까우면 true.
    fn is_about_to_hit(&self, body: Rect) -> bool {
        let me = self.rect();
        let gap = if self.direction > 0.0 { body.x - (me.x + me.w) } else { me.x - (body.x + body.w) };
        self.bounced && (0.0..=SKELETON_DODGE_SENSE_DISTANCE).contains(&gap)
    }
}

/// 맵에 있는 스켈레톤과 날아가는 공격 전부.
pub struct Skeletons {
    pub list: Vec<Skeleton>,
    pub projectiles: Vec<Projectile>,
}

impl Skeletons {
    pub fn new() -> Skeletons {
        Skeletons { list: Vec::new(), projectiles: Vec::new() }
    }

    /// 아침: 스켈레톤과 날아가던 공격이 모두 사라진다.
    pub fn clear(&mut self) {
        self.list.clear();
        self.projectiles.clear();
    }

    /// 임시-3, 임시-4: 주인공에게서 너무 멀어진 스켈레톤과 화살·총알이 조용히 사라진다.
    pub fn forget_far_away(&mut self, player_x: f32) {
        self.list.retain(|s| (s.rect().center().x - player_x).abs() <= FORGET_DISTANCE);
        self.projectiles.retain(|shot| (shot.rect().center().x - player_x).abs() <= FORGET_DISTANCE);
    }

    /// 밤에 짧은 시간 `step` 동안: 피하고, 움직이고, 쏘고, 날린다. 튕긴 공격에 쓰러진 수를 돌려준다.
    pub fn step(&mut self, step: f32, player_rect: Rect, world: &World, events: &mut Vec<CombatEvent>) -> u32 {
        self.dodge_bounced_attacks(events);
        let player_x = player_rect.center().x;
        for skeleton in &mut self.list {
            skeleton.step(step, player_x, world, &mut self.projectiles, events);
        }
        self.projectiles.retain_mut(|shot| shot.step(step, world, events));
        self.get_hit_by_bounced_attacks(events)
    }

    /// 몸에 닿은 공격은 모두 사라진다 (무적이어도). 그중 가장 센 공격력을 돌려준다.
    pub fn take_shots_hitting(&mut self, player_rect: Rect) -> Option<u32> {
        let mut strongest = None;
        self.projectiles.retain(|shot| {
            let hit = really_overlaps(shot.rect(), player_rect);
            if hit {
                strongest = strongest.max(Some(shot.attack));
            }
            !hit
        });
        strongest
    }

    /// 플레이어 몸에 닿은 절반 스켈레톤. 손은 몸이 닿을 만큼 가까워야 맞는다 (US-047).
    pub fn half_skeleton_touching(&self, player_rect: Rect) -> Option<&Skeleton> {
        self.list.iter().find(|s| s.stage == SkeletonStage::Half && really_overlaps(s.rect(), player_rect))
    }

    fn dodge_bounced_attacks(&mut self, events: &mut Vec<CombatEvent>) {
        for skeleton in &mut self.list {
            let body = skeleton.rect();
            if skeleton.state != SkeletonState::Dodging && self.projectiles.iter().any(|shot| shot.is_about_to_hit(body)) {
                skeleton.start_dodge();
                events.push(CombatEvent::SkeletonDodged { at: body.center() });
            }
        }
    }

    fn get_hit_by_bounced_attacks(&mut self, events: &mut Vec<CombatEvent>) -> u32 {
        let mut defeated = 0;
        let skeletons = &mut self.list;
        self.projectiles.retain(|shot| {
            let touched = skeletons.iter().position(|s| really_overlaps(s.rect(), shot.rect()));
            let Some(index) = touched.filter(|_| shot.bounced) else {
                return true;
            };
            events.push(skeletons[index].take_hit(1));
            if skeletons[index].hp == 0 {
                skeletons.remove(index);
                defeated += 1;
            }
            false
        });
        defeated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_stage_per_level() {
        assert_eq!(SkeletonStage::for_level(1), SkeletonStage::Normal);
        assert_eq!(SkeletonStage::for_level(2), SkeletonStage::Half);
        assert_eq!(SkeletonStage::for_level(3), SkeletonStage::Full);
        assert_eq!(SkeletonStage::for_level(30), SkeletonStage::Full);
    }

    #[test]
    fn only_the_half_skeleton_has_no_weapon() {
        assert_eq!(SkeletonStage::Normal.weapon(), Some(ProjectileKind::Arrow));
        assert_eq!(SkeletonStage::Half.weapon(), None);
        assert_eq!(SkeletonStage::Full.weapon(), Some(ProjectileKind::Bullet));
    }

    #[test]
    fn bullet_is_faster_and_more_often_than_arrow() {
        assert!(ProjectileKind::Bullet.speed() > ProjectileKind::Arrow.speed());
        assert!(ProjectileKind::Bullet.interval() < ProjectileKind::Arrow.interval());
    }

    #[test]
    fn third_hit_reports_defeat() {
        let mut s = Skeleton::new(0.0, 440.0, SkeletonStage::Normal);
        assert!(matches!(s.take_hit(1), CombatEvent::SkeletonHit { hp_left: 2, .. }));
        assert!(matches!(s.take_hit(1), CombatEvent::SkeletonHit { hp_left: 1, .. }));
        assert!(matches!(s.take_hit(1), CombatEvent::SkeletonDefeated { .. }));
    }
}
