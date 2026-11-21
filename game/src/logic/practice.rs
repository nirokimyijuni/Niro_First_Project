//! 연습용 몬스터 (US-124). 첫 낮에 말풍선 ②(공격)가 나올 때 한 마리 나와서 공격을 연습하게 해 준다.
//!
//! - 약하다: 맞은 횟수 `PRACTICE_MONSTER_HP`번이면 쓰러진다 (무기 힘과 상관없이 한 번에 `PRACTICE_MONSTER_HIT_DAMAGE`).
//! - 안 아프다: 플레이어 체력을 건드리는 코드가 이 파일에 없다. 천천히 따라오기만 한다.
//! - 코인·경험치·킬 수·미션에 들지 않는다. 밤 몬스터 목록(`Combat`)에 들어가지 않는 따로 있는 목록이다.
//! - 안 때리고 두면 `PRACTICE_MONSTER_LIFE_SECONDS`초 뒤 저절로 사라진다. 밤이 오면 `Game`이 치운다.

use super::monster::{free_ground_x, walk_x};
use super::player::Facing;
use super::world::{really_overlaps, World};
use crate::config::{
    PRACTICE_MONSTER_FLASH_SECONDS, PRACTICE_MONSTER_HEIGHT, PRACTICE_MONSTER_HIT_DAMAGE, PRACTICE_MONSTER_HP,
    PRACTICE_MONSTER_LIFE_SECONDS, PRACTICE_MONSTER_SPAWN_DISTANCE, PRACTICE_MONSTER_STOP_DISTANCE,
    PRACTICE_MONSTER_WALK_SPEED, PRACTICE_MONSTER_WIDTH,
};
use macroquad::math::{Rect, Vec2};

/// 몬스터 종류 값. 화면은 이것을 보고 그림을 고른다 (밤 몬스터와 다르게 그린다).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterKind {
    /// 밤에 나오는 몬스터 (좀비·스켈레톤). 이 파일의 몬스터는 아니다.
    Night,
    /// 연습용 (임시 이름. 진짜 이름·생김새·색은 아이가 정한다)
    Practice,
}

/// 화면(로건)이 보고 표시·소리를 낼 일. `at`은 몸 가운데. `CombatEvent`와 따로 `Game::take_practice_events`로 받는다.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PracticeEvent {
    /// 연습용 몬스터가 나왔다.
    Appeared { at: Vec2 },
    /// 맞았고 아직 서 있다.
    Hit { at: Vec2, hp_left: u32 },
    /// 쓰러졌다 ("쓰러졌어요"). 코인·경험치는 없다.
    Defeated { at: Vec2 },
    /// 안 때려서 시간이 다 됐거나 밤이 와서 저절로 사라졌다.
    Vanished { at: Vec2 },
}

pub struct PracticeMonster {
    /// 몸 네모의 왼쪽 위 모서리.
    pub pos: Vec2,
    pub hp: u32,
    seconds_left: f32,
    flash_left: f32,
}

impl PracticeMonster {
    /// 땅 위 `x`(왼쪽 모서리)에 서 있는 연습용 몬스터.
    pub fn new(x: f32, ground_y: f32) -> PracticeMonster {
        PracticeMonster {
            pos: Vec2::new(x, ground_y - PRACTICE_MONSTER_HEIGHT),
            hp: PRACTICE_MONSTER_HP,
            seconds_left: PRACTICE_MONSTER_LIFE_SECONDS,
            flash_left: 0.0,
        }
    }

    pub fn kind(&self) -> MonsterKind {
        MonsterKind::Practice
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, PRACTICE_MONSTER_WIDTH, PRACTICE_MONSTER_HEIGHT)
    }

    /// 막 맞아서 반짝여야 하면 true.
    pub fn is_flashing(&self) -> bool {
        self.flash_left > 0.0
    }

    /// 사라지기까지 남은 초.
    pub fn seconds_left(&self) -> f32 {
        self.seconds_left
    }

    /// 한 번 맞는다. 쓰러졌으면 true.
    pub fn take_hit(&mut self) -> bool {
        self.hp = self.hp.saturating_sub(PRACTICE_MONSTER_HIT_DAMAGE);
        self.flash_left = PRACTICE_MONSTER_FLASH_SECONDS;
        self.hp == 0
    }

    /// 짧은 시간 `step`초 흐른다: 천천히 플레이어 쪽으로 가다가 가까워지면 멈춘다. 시간이 다 됐으면 true.
    pub fn step(&mut self, step: f32, target_x: f32, world: &World) -> bool {
        self.flash_left = (self.flash_left - step).max(0.0);
        self.seconds_left -= step;
        let near = super::world::shortest_dx(self.rect().center().x, target_x).abs() <= PRACTICE_MONSTER_STOP_DISTANCE;
        if !near {
            self.pos.x = walk_x(self.rect(), target_x, PRACTICE_MONSTER_WALK_SPEED * step, world);
        }
        self.seconds_left <= 0.0
    }
}

/// 말풍선 ②가 시작될 때 연습용 몬스터가 설 x (왼쪽 모서리). 플레이어가 바라보는 쪽으로
/// `PRACTICE_MONSTER_SPAWN_DISTANCE`만큼 떨어진 땅이고, 그쪽이 막혀 플레이어와 겹치면 반대쪽이다. 둘 다 안 되면 None.
pub fn practice_spawn_x(player_rect: Rect, facing: Facing, world: &World) -> Option<f32> {
    let forward = if facing == Facing::Left { -1.0 } else { 1.0 };
    for direction in [forward, -forward] {
        let wanted = player_rect.center().x + direction * PRACTICE_MONSTER_SPAWN_DISTANCE - PRACTICE_MONSTER_WIDTH / 2.0;
        let x = free_ground_x(world.keep_on_map(wanted, PRACTICE_MONSTER_WIDTH), direction, PRACTICE_MONSTER_WIDTH, world);
        let body = Rect::new(x, world.ground_y - PRACTICE_MONSTER_HEIGHT, PRACTICE_MONSTER_WIDTH, PRACTICE_MONSTER_HEIGHT);
        if !really_overlaps(body, player_rect) {
            return Some(x);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    #[test]
    fn practice_monster_is_weaker_than_night_zombie() {
        assert!(PRACTICE_MONSTER_HP < ZOMBIE_HP);
        assert!(PRACTICE_MONSTER_HP >= 1);
    }

    #[test]
    fn falls_after_configured_number_of_hits() {
        let mut m = PracticeMonster::new(300.0, GROUND_Y);
        for hit in 1..=PRACTICE_MONSTER_HP {
            let fell = m.take_hit();
            assert_eq!(fell, hit == PRACTICE_MONSTER_HP);
        }
        assert_eq!(m.hp, 0);
    }

    #[test]
    fn kind_is_practice_not_night() {
        let m = PracticeMonster::new(0.0, GROUND_Y);
        assert_eq!(m.kind(), MonsterKind::Practice);
        assert_ne!(m.kind(), MonsterKind::Night);
    }

    #[test]
    fn disappears_after_life_seconds() {
        let w = World::m1();
        let mut m = PracticeMonster::new(900.0, w.ground_y);
        let steps = (PRACTICE_MONSTER_LIFE_SECONDS / 0.5) as u32;
        let mut gone = false;
        for i in 0..steps + 4 {
            gone = m.step(0.5, 0.0, &w);
            if gone {
                assert!(i + 1 >= steps - 1, "too early at step {i}");
                break;
            }
        }
        assert!(gone);
    }

    #[test]
    fn follows_slowly_and_stops_near_player() {
        let w = World { width: WORLD_WIDTH, ground_y: GROUND_Y, obstacles: vec![] };
        let mut m = PracticeMonster::new(700.0, w.ground_y);
        let start = m.pos.x;
        m.step(1.0, 100.0, &w);
        assert!((start - m.pos.x - PRACTICE_MONSTER_WALK_SPEED).abs() < 0.01);
        assert!(PRACTICE_MONSTER_WALK_SPEED < ZOMBIE_WALK_SPEED);
        for _ in 0..200 {
            m.step(0.5, 100.0, &w);
        }
        let gap = (m.rect().center().x - 100.0).abs();
        assert!(gap <= PRACTICE_MONSTER_STOP_DISTANCE + 1.0, "gap {gap}");
        assert!(gap >= PRACTICE_MONSTER_STOP_DISTANCE - PRACTICE_MONSTER_WALK_SPEED, "too close {gap}");
    }

    #[test]
    fn spawns_in_front_on_screen_or_other_side() {
        let w = World::m1();
        let player = Rect::new(PLAYER_SPAWN_X, w.ground_y - PLAYER_HEIGHT, PLAYER_WIDTH, PLAYER_HEIGHT);
        let x = practice_spawn_x(player, Facing::Right, &w).unwrap();
        assert!(x > player.x);
        assert!(x + PRACTICE_MONSTER_WIDTH <= SCREEN_WIDTH);
        let left = practice_spawn_x(player, Facing::Left, &w).unwrap();
        assert!(left >= 0.0 && !really_overlaps(Rect::new(left, w.ground_y - 48.0, 40.0, 48.0), player));
    }
}
