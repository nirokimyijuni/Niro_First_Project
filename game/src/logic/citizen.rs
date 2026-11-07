//! 폭발형 미스 시민 (US-054, D21): 자기 자리에 서 있다가 플레이어나 몬스터 몸이 닿으면 펑 터진다.
//! 터진 뒤 일은 `Combat::citizen_blast`가 맡는다. 화살·총알에는 터지지 않는다.
//! 주인공이 공격하면 터지지 않고 그냥 쓰러진다 (아이가 정한 임시 규칙).
//!
//! 일반 시민(`WalkingCitizen`)은 장소 안을 천천히 오가는 구경꾼이다. 아무와도 부딪히지 않고 맞지도 않는다.
//! 상자·구조물·구멍과 장소 끝에 닿으면 돌아선다. 게임 규칙에는 끼어들지 않는다.

use super::monster::SeededRandom;
use super::player::Facing;
use super::world::{really_overlaps, World};
use crate::config::{
    CITIZEN_HEIGHT, CITIZEN_RESPAWN_SECONDS, CITIZEN_WIDTH, CITIZEN_XS, PLACE_NAMES, SCREEN_WIDTH,
    WALKING_CITIZEN_HEIGHT, WALKING_CITIZEN_PAUSE_SECONDS, WALKING_CITIZEN_SPEED, WALKING_CITIZEN_WALK_SECONDS,
    WALKING_CITIZEN_WIDTH,
};
use macroquad::math::{Rect, Vec2};

pub struct Citizen {
    /// 시민 몸 네모 (왼쪽 위 기준). 터져도 다시 나올 자리로 그대로 둔다.
    pub rect: Rect,
    /// 다시 나타날 때까지 남은 초.
    respawn_seconds_left: f32,
    standing: bool,
}

impl Citizen {
    fn at(center_x: f32, ground_y: f32) -> Citizen {
        let rect = Rect::new(center_x - CITIZEN_WIDTH / 2.0, ground_y - CITIZEN_HEIGHT, CITIZEN_WIDTH, CITIZEN_HEIGHT);
        Citizen { rect, respawn_seconds_left: 0.0, standing: true }
    }

    /// 지금 서 있으면 true. 화면은 이때만 그린다.
    pub fn is_standing(&self) -> bool {
        self.standing
    }

    /// 공격을 맞아 조용히 쓰러진다. 다시 나오는 시간은 터질 때와 같다.
    fn knock_down(&mut self) {
        self.standing = false;
        self.respawn_seconds_left = CITIZEN_RESPAWN_SECONDS;
    }

    /// 시간을 흘린다. `touched`는 플레이어나 몬스터 몸이 이 자리에 닿았는지. 지금 터졌으면 true.
    /// 사라진 동안 누가 자리에 있으면 비켜날 때까지 기다렸다가 나타난다.
    pub fn update(&mut self, dt: f32, touched: bool) -> bool {
        if self.standing {
            if touched {
                self.knock_down();
            }
            return touched;
        }
        self.respawn_seconds_left = (self.respawn_seconds_left - dt.max(0.0)).max(0.0);
        self.standing = self.respawn_seconds_left == 0.0 && !touched;
        false
    }
}

/// 안테나 1개 일반 시민 하나.
pub struct WalkingCitizen {
    /// 몸 네모 (왼쪽 위 기준).
    pub rect: Rect,
    /// 걷는 쪽. 화면은 이것으로 그림을 뒤집는다.
    pub facing: Facing,
    /// 머무는 장소 번호 (`PLACE_NAMES` 순서).
    place: usize,
    walking: bool,
    /// 지금 걷기나 멈춤이 끝날 때까지 남은 초.
    seconds_left: f32,
}

impl WalkingCitizen {
    /// `place` 장소 가운데쯤에 선다. 막혀 있으면 가장 가까운 빈 땅이다.
    /// 걷는 쪽과 첫 걷기 시간은 시드로 정한다 (네 명이 한꺼번에 멈추지 않게).
    fn in_place(place: usize, random: &mut SeededRandom, world: &World) -> WalkingCitizen {
        const SEARCH_STEP: usize = 8;
        let room = SCREEN_WIDTH - WALKING_CITIZEN_WIDTH;
        let middle = place as f32 * SCREEN_WIDTH + room / 2.0;
        let body_at = |x: f32| {
            Rect::new(x, world.ground_y - WALKING_CITIZEN_HEIGHT, WALKING_CITIZEN_WIDTH, WALKING_CITIZEN_HEIGHT)
        };
        let rect = (0..(room / 2.0) as usize)
            .step_by(SEARCH_STEP)
            .flat_map(|offset| [middle - offset as f32, middle + offset as f32])
            .map(body_at)
            .find(|body| !is_blocked(*body, world))
            .unwrap_or(body_at(middle));
        let facing = if random.next_unit() < 0.5 { Facing::Left } else { Facing::Right };
        let seconds_left = random.next_unit() * WALKING_CITIZEN_WALK_SECONDS;
        WalkingCitizen { rect, facing, place, walking: true, seconds_left }
    }

    /// 지금 걷는 중이면 true. 멈춘 동안과 쓰러진 화면에서는 false다.
    pub fn is_walking(&self) -> bool {
        self.walking
    }

    /// `dt`초 지낸다: 걷기와 멈춤을 번갈아 하고, 걷다가 앞이 막히면 그 자리에서 돌아선다.
    fn walk(&mut self, dt: f32, world: &World) {
        self.seconds_left -= dt.max(0.0);
        if self.seconds_left <= 0.0 {
            self.walking = !self.walking;
            self.seconds_left = if self.walking { WALKING_CITIZEN_WALK_SECONDS } else { WALKING_CITIZEN_PAUSE_SECONDS };
        }
        if !self.walking {
            return;
        }
        let step = WALKING_CITIZEN_SPEED * dt.max(0.0);
        let moved_x = self.rect.x + if self.facing == Facing::Right { step } else { -step };
        let place_left = self.place as f32 * SCREEN_WIDTH;
        let in_place = place_left <= moved_x && moved_x + self.rect.w <= place_left + SCREEN_WIDTH;
        let moved = Rect { x: moved_x, ..self.rect };
        if in_place && !is_blocked(moved, world) {
            self.rect = moved;
        } else {
            self.facing = if self.facing == Facing::Right { Facing::Left } else { Facing::Right };
        }
    }
}

fn is_blocked(body: Rect, world: &World) -> bool {
    world.monster_walls().any(|wall| really_overlaps(wall, body))
}

/// 맵의 시민 전부. 긴 맵에만 있다.
pub struct Citizens {
    /// 폭발형 시민.
    pub list: Vec<Citizen>,
    /// 일반 시민.
    pub walkers: Vec<WalkingCitizen>,
}

impl Citizens {
    /// `seed`가 같으면 일반 시민의 자리와 걷는 쪽이 늘 같다.
    pub fn on_world(world: &World, seed: u32) -> Citizens {
        if !world.is_long_map() {
            return Citizens { list: Vec::new(), walkers: Vec::new() };
        }
        // 맵·싸움·미스와 같은 seed를 쓰므로 섞어야 서로 따라가지 않는다.
        let mut random = SeededRandom::new(seed.wrapping_mul(37).wrapping_add(29));
        Citizens {
            list: CITIZEN_XS.iter().map(|x| Citizen::at(*x, world.ground_y)).collect(),
            walkers: (0..PLACE_NAMES.len()).map(|place| WalkingCitizen::in_place(place, &mut random, world)).collect(),
        }
    }

    /// 공격이 닿은 시민들이 쓰러진다. 돌려주는 값: 쓰러진 자리들 (몸 가운데).
    pub fn knock_down_reached(&mut self, is_reached: impl Fn(Rect) -> bool) -> Vec<Vec2> {
        let mut knocked_down = Vec::new();
        for citizen in self.list.iter_mut().filter(|c| c.standing && is_reached(c.rect)) {
            citizen.knock_down();
            knocked_down.push(citizen.rect.center());
        }
        knocked_down
    }

    /// 쓰러진 화면: 일반 시민이 그 자리에 멈춘다.
    pub fn stop_walkers(&mut self) {
        for walker in &mut self.walkers {
            walker.walking = false;
        }
    }

    /// 일반 시민들을 `dt`초 걷게 한다.
    pub fn walk(&mut self, dt: f32, world: &World) {
        for walker in &mut self.walkers {
            walker.walk(dt, world);
        }
    }
}
