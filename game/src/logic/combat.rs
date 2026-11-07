//! 싸움 규칙: 플레이어 체력(D2), 공격, 몬스터 내보내기, 맞기, 경험치와 레벨(D35), 아침 회복(D27).
//!
//! `Combat` 하나가 체력과 좀비·스켈레톤을 함께 들고 있다. 결과는 `CombatEvent`로 돌려주고,
//! 화면(로건)이 그걸 보고 "쓰러졌어요", "펑!" 같은 표시와 소리를 낸다.

use super::monster::{pick_left_side, spawn_x_on_side, SeededRandom, Zombie, Zombies};
use super::player::Facing;
use super::skeleton::{ProjectileKind, Skeleton, SkeletonStage, Skeletons};
use super::world::{really_overlaps, World};
use crate::config::{
    CITIZEN_BLAST_DAMAGE, CITIZEN_BLAST_RANGE, EXPERIENCE_PER_DEFEAT, EXPERIENCE_PER_LEVEL, FORGET_DISTANCE,
    MONSTERS_PER_SPAWN, MONSTER_MAX_ON_SCREEN, PHYSICS_STEP_SECONDS,
    PLAYER_ATTACK_RANGE, PLAYER_HP_PER_LEVEL, PLAYER_HURT_INVINCIBLE_SECONDS, PLAYER_START_HP,
    SKELETON_SPAWN_CHANCE, SKELETON_WIDTH, ZOMBIE_ATTACK, ZOMBIE_FLOAT_CHANCE, ZOMBIE_MAX_ON_SCREEN,
    ZOMBIE_SPAWN_SECONDS, ZOMBIE_WIDTH,
};
use macroquad::math::{Rect, Vec2};

/// 화면(로건)이 보고 표시·소리를 낼 일. `at`은 몬스터(또는 화살·총알) 몸 가운데.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CombatEvent {
    /// 화면 끝에 좀비가 나타났다.
    ZombieAppeared { at: Vec2 },
    /// 공격이 맞았고 좀비는 아직 서 있다.
    ZombieHit { at: Vec2, hp_left: u32 },
    /// 공격으로 좀비가 쓰러졌다 ("쓰러졌어요"). 경험치를 준다.
    ZombieDefeated { at: Vec2 },
    /// 날아오른 좀비가 팔을 내려 펑 터졌다 ("펑!"). 경험치는 없다.
    ZombiePopped { at: Vec2 },
    /// 화면 끝에 스켈레톤이 나타났다.
    SkeletonAppeared { at: Vec2, stage: SkeletonStage },
    /// 공격이나 튕겨 온 화살·총알에 맞았고 스켈레톤은 아직 서 있다.
    SkeletonHit { at: Vec2, hp_left: u32 },
    /// 스켈레톤이 쓰러졌다 ("쓰러졌어요"). 경험치를 준다.
    SkeletonDefeated { at: Vec2 },
    /// 튕겨 오는 공격을 피해 스켈레톤이 뛰어올랐다.
    SkeletonDodged { at: Vec2 },
    /// 절반 스켈레톤의 손이 플레이어에게 맞았다. 바로 뒤에 `PlayerHurt`가 온다.
    SkeletonHandHit { at: Vec2 },
    /// 스켈레톤이 화살이나 총알을 쐈다.
    ProjectileFired { at: Vec2, kind: ProjectileKind },
    /// 화살이나 총알이 상자에 맞아 튕겼다.
    ProjectileBounced { at: Vec2 },
    /// 경험치가 차서 레벨이 올랐다 ("레벨 업!"). `level`은 새 레벨.
    LevelUp { level: u32 },
    /// 플레이어가 몬스터에게 맞아 체력이 줄었다.
    PlayerHurt { amount: u32, hp_left: u32 },
    /// 체력이 0이 되어 쓰러졌다 (게임오버). 한 판에 한 번만 온다.
    PlayerDown,
    /// 아침이 오거나 (D27) 레벨이 올라서 체력이 다시 찼다.
    HealthRefilled,
}

/// 플레이어 체력.
pub struct PlayerHealth {
    hp: u32,
    max_hp: u32,
    invincible_seconds_left: f32,
}

impl PlayerHealth {
    pub fn new() -> PlayerHealth {
        PlayerHealth { hp: PLAYER_START_HP, max_hp: PLAYER_START_HP, invincible_seconds_left: 0.0 }
    }

    pub fn hp(&self) -> u32 {
        self.hp
    }

    pub fn max_hp(&self) -> u32 {
        self.max_hp
    }

    pub fn is_down(&self) -> bool {
        self.hp == 0
    }

    /// 맞은 뒤 다시 안 맞는 시간이면 true. 이 동안 캐릭터가 깜빡인다.
    pub fn is_invincible(&self) -> bool {
        self.invincible_seconds_left > 0.0
    }

    fn can_be_hit(&self) -> bool {
        !self.is_invincible() && !self.is_down()
    }

    /// `amount`만큼 맞는다. `can_be_hit`일 때만 부른다.
    fn take_hit(&mut self, amount: u32) {
        self.hp = self.hp.saturating_sub(amount);
        self.invincible_seconds_left = PLAYER_HURT_INVINCIBLE_SECONDS;
    }
}

impl Default for PlayerHealth {
    fn default() -> PlayerHealth {
        PlayerHealth::new()
    }
}

/// 플레이어 앞쪽의 맨손 공격이 닿는 네모. 높이는 플레이어 키만큼.
pub fn attack_box(player_rect: Rect, facing: Facing) -> Rect {
    attack_box_with_range(player_rect, facing, PLAYER_ATTACK_RANGE)
}

/// 플레이어 앞쪽 `range`까지 닿는 네모 (US-053: 무기마다 다르다).
pub fn attack_box_with_range(player_rect: Rect, facing: Facing, range: f32) -> Rect {
    let x = match facing {
        Facing::Right => player_rect.x + player_rect.w,
        Facing::Left => player_rect.x - range,
    };
    Rect::new(x, player_rect.y, range, player_rect.h)
}

/// 몸이 겹쳐 있어도 등 뒤는 맞지 않게, 가운데가 바라보는 쪽에 있는지 본다.
fn is_in_front(player_rect: Rect, facing: Facing, target: Rect) -> bool {
    match facing {
        Facing::Right => target.center().x >= player_rect.center().x,
        Facing::Left => target.center().x <= player_rect.center().x,
    }
}

/// 공격 범위 안이고 바라보는 쪽에 있으면 맞는다.
fn is_reached(player_rect: Rect, facing: Facing, range: f32, target: Rect) -> bool {
    attack_box_with_range(player_rect, facing, range).overlaps(&target) && is_in_front(player_rect, facing, target)
}

/// 한 판의 싸움 상태. 새 게임을 하면 새로 만든다.
pub struct Combat {
    pub health: PlayerHealth,
    pub zombies: Zombies,
    pub skeletons: Skeletons,
    /// 플레이어 레벨이자 몬스터 레벨 (D3: 같이 오른다). 1부터.
    pub level: u32,
    /// 지금 레벨에서 모은 경험치. `EXPERIENCE_PER_LEVEL`이 되면 레벨이 오르고 0으로 돌아간다.
    pub experience: u32,
    /// 나올 차례에 스켈레톤이 나올 확률. 테스트는 0으로 두어 좀비만 있는 밤을 만든다.
    pub skeleton_chance: f32,
    seconds_to_next_spawn: f32,
    random: SeededRandom,
    was_night: bool,
}

impl Combat {
    /// `seed`가 같으면 나오는 몬스터와 나오는 쪽, 날아오르는 좀비가 늘 똑같다.
    /// 처음에는 몬스터가 없고, 밤이 되면 첫 몬스터가 바로 나온다.
    pub fn new(seed: u32) -> Combat {
        Combat {
            health: PlayerHealth::new(),
            zombies: Zombies::new(),
            skeletons: Skeletons::new(),
            level: 1,
            experience: 0,
            skeleton_chance: SKELETON_SPAWN_CHANCE,
            seconds_to_next_spawn: 0.0,
            random: SeededRandom::new(seed),
            was_night: false,
        }
    }

    /// 한 프레임 진행. 쓰러진 뒤에는 아무것도 움직이지 않는다 (US-043).
    pub fn update(&mut self, dt: f32, is_night: bool, player_rect: Rect, world: &World) -> Vec<CombatEvent> {
        let mut events = Vec::new();
        if self.health.is_down() {
            return events;
        }

        if self.was_night && !is_night {
            self.zombies.list.clear();
            self.skeletons.clear();
            self.seconds_to_next_spawn = 0.0;
            self.health.hp = self.health.max_hp;
            events.push(CombatEvent::HealthRefilled);
        }
        self.was_night = is_night;

        // 시간을 잘게 나눠서 움직인다 → 프레임이 길어도 몬스터나 화살이 상자를 뚫지 않는다.
        let mut time_left = dt.max(0.0);
        while time_left > 0.0 && !self.health.is_down() {
            let step = time_left.min(PHYSICS_STEP_SECONDS);
            self.health.invincible_seconds_left = (self.health.invincible_seconds_left - step).max(0.0);
            if is_night {
                self.spawn_monster_when_ready(step, player_rect, world, &mut events);
                self.zombies.step(step, player_rect, world, &mut events);
                let defeated = self.skeletons.step(step, player_rect, world, &mut events);
                self.gain_experience(defeated, &mut events);
                self.get_hurt_by_touching_zombie(player_rect, &mut events);
                self.get_hurt_by_skeletons(player_rect, &mut events);
            }
            time_left -= step;
        }
        self.forget_far_monsters(player_rect.center().x);
        events
    }

    /// 임시-3: 뒤에 남은 몬스터가 마릿수를 채우지 않게 조용히 치운다. 경험치는 없다.
    fn forget_far_monsters(&mut self, player_x: f32) {
        self.zombies.list.retain(|z| (z.rect().center().x - player_x).abs() <= FORGET_DISTANCE);
        self.skeletons.forget_far_away(player_x);
    }

    /// 공격 버튼을 "새로" 눌렀을 때 한 번 부른다. `range` 안의 몬스터가 모두 체력 `attack`씩 줄어든다.
    pub fn attack_with(&mut self, player_rect: Rect, facing: Facing, attack: u32, range: f32) -> Vec<CombatEvent> {
        let mut events = Vec::new();
        if self.health.is_down() {
            return events;
        }
        let mut defeated = 0;
        self.zombies.list.retain_mut(|zombie| {
            let body = zombie.rect();
            if !is_reached(player_rect, facing, range, body) {
                return true;
            }
            zombie.take_hit(attack);
            if zombie.hp == 0 {
                events.push(CombatEvent::ZombieDefeated { at: body.center() });
                defeated += 1;
                return false;
            }
            events.push(CombatEvent::ZombieHit { at: body.center(), hp_left: zombie.hp });
            true
        });
        self.skeletons.list.retain_mut(|skeleton| {
            if !is_reached(player_rect, facing, range, skeleton.rect()) {
                return true;
            }
            events.push(skeleton.take_hit(attack));
            if skeleton.hp == 0 {
                defeated += 1;
            }
            skeleton.hp > 0
        });
        self.gain_experience(defeated, &mut events);
        events
    }

    /// 몬스터 몸이 `area`에 닿았는지. 떠오른 좀비도 몸이 닿으면 닿은 것이다 (US-054).
    pub fn any_monster_touches(&self, area: Rect) -> bool {
        self.zombies.list.iter().any(|z| really_overlaps(z.rect(), area))
            || self.skeletons.list.iter().any(|s| really_overlaps(s.rect(), area))
    }

    /// 시민이 `at`에서 펑 터진다: 가까운 플레이어는 체력이 줄고, 가까운 몬스터는 쓰러진다.
    /// 경험치는 없다 (임시-9).
    pub fn citizen_blast(&mut self, at: Vec2, player_rect: Rect) -> Vec<CombatEvent> {
        let mut events = Vec::new();
        let is_near = |body: Rect| body.center().distance(at) <= CITIZEN_BLAST_RANGE;
        if is_near(player_rect) {
            self.hurt_player(CITIZEN_BLAST_DAMAGE, &mut events);
        }
        self.zombies.list.retain(|zombie| {
            let near = is_near(zombie.rect());
            if near {
                events.push(CombatEvent::ZombieDefeated { at: zombie.rect().center() });
            }
            !near
        });
        self.skeletons.list.retain(|skeleton| {
            let near = is_near(skeleton.rect());
            if near {
                events.push(CombatEvent::SkeletonDefeated { at: skeleton.rect().center() });
            }
            !near
        });
        events
    }

    /// 레벨이 1 오른다: 최대 체력 +5 (D2), 몬스터도 같이 (D3), 체력은 바로 다 찬다 (아이가 정함).
    /// 쓰러진 뒤에는 오르지 않는다: 체력이 차면 일어나 버린다.
    pub fn level_up(&mut self) {
        if self.health.is_down() {
            return;
        }
        self.level += 1;
        self.health.max_hp += PLAYER_HP_PER_LEVEL;
        self.health.hp = self.health.max_hp;
    }

    /// 몬스터 `defeated`마리를 쓰러뜨린 경험치. 펑 터진 좀비는 여기 오지 않는다 (US-045).
    fn gain_experience(&mut self, defeated: u32, events: &mut Vec<CombatEvent>) {
        if self.health.is_down() {
            return;
        }
        for _ in 0..defeated {
            self.experience += EXPERIENCE_PER_DEFEAT;
            if self.experience >= EXPERIENCE_PER_LEVEL {
                self.experience = 0;
                self.level_up();
                events.push(CombatEvent::LevelUp { level: self.level });
                events.push(CombatEvent::HealthRefilled);
            }
        }
    }

    /// 좀비와 스켈레톤이 나오는 시간을 함께 센다. 화면이 꽉 차 있는 동안은 세지 않는다.
    fn spawn_monster_when_ready(&mut self, step: f32, player_rect: Rect, world: &World, events: &mut Vec<CombatEvent>) {
        if self.is_map_full() {
            return;
        }
        self.seconds_to_next_spawn -= step;
        if self.seconds_to_next_spawn > 0.0 {
            return;
        }
        self.seconds_to_next_spawn += ZOMBIE_SPAWN_SECONDS;

        // 왼쪽·오른쪽 번갈아 나온다. 마릿수 상한에 닿으면 이번 차례는 거기까지다.
        let first_from_left = pick_left_side(&mut self.random);
        let mut taken = Vec::new();
        for turn in 0..MONSTERS_PER_SPAWN {
            if self.is_map_full() {
                break;
            }
            let from_left = first_from_left == (turn % 2 == 0);
            let x = self.spawn_one_monster(from_left, player_rect, world, &taken, events);
            taken.push(x);
        }
    }

    fn is_map_full(&self) -> bool {
        self.zombies.list.len() + self.skeletons.list.len() >= MONSTER_MAX_ON_SCREEN
    }

    /// 좀비나 스켈레톤을 하나 뽑아 내보내고 나온 x를 돌려준다.
    fn spawn_one_monster(
        &mut self,
        from_left: bool,
        player_rect: Rect,
        world: &World,
        taken: &[f32],
        events: &mut Vec<CombatEvent>,
    ) -> f32 {
        if self.zombies.list.len() >= ZOMBIE_MAX_ON_SCREEN || self.random.next_unit() < self.skeleton_chance {
            let stage = SkeletonStage::for_level(self.level);
            let x = spawn_x_on_side(from_left, player_rect, SKELETON_WIDTH, world, taken);
            let skeleton = Skeleton::new(x, world.ground_y, stage);
            events.push(CombatEvent::SkeletonAppeared { at: skeleton.rect().center(), stage });
            self.skeletons.list.push(skeleton);
            x
        } else {
            let x = spawn_x_on_side(from_left, player_rect, ZOMBIE_WIDTH, world, taken);
            let will_float = self.random.next_unit() < ZOMBIE_FLOAT_CHANCE;
            let zombie = Zombie::new(x, world.ground_y, will_float);
            events.push(CombatEvent::ZombieAppeared { at: zombie.rect().center() });
            self.zombies.list.push(zombie);
            x
        }
    }

    fn hurt_player(&mut self, amount: u32, events: &mut Vec<CombatEvent>) {
        if !self.health.can_be_hit() {
            return;
        }
        self.health.take_hit(amount);
        events.push(CombatEvent::PlayerHurt { amount, hp_left: self.health.hp });
        if self.health.is_down() {
            events.push(CombatEvent::PlayerDown);
        }
    }

    fn get_hurt_by_touching_zombie(&mut self, player_rect: Rect, events: &mut Vec<CombatEvent>) {
        if self.zombies.any_walking_zombie_touches(player_rect) {
            self.hurt_player(ZOMBIE_ATTACK, events);
        }
    }

    fn get_hurt_by_skeletons(&mut self, player_rect: Rect, events: &mut Vec<CombatEvent>) {
        if let Some(attack) = self.skeletons.take_shots_hitting(player_rect) {
            self.hurt_player(attack, events);
        }
        if !self.health.can_be_hit() {
            return;
        }
        if let Some(skeleton) = self.skeletons.half_skeleton_touching(player_rect) {
            let attack = skeleton.stage.attack();
            events.push(CombatEvent::SkeletonHandHit { at: skeleton.rect().center() });
            self.hurt_player(attack, events);
        }
    }
}
