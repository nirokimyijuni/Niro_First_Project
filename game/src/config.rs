//! 게임 수치를 한 곳에 모은다. 원본: docs/02-plan.md "결정 사항".
//! 숫자를 바꿀 때는 이 파일만 고치면 된다.

// 아직 logic/에서 쓰지 않는 상수가 있어서 경고를 끈다. 쓰기 시작하면 지워도 된다.
#![allow(dead_code)]

/// D1 낮 길이 (초) — 10분
pub const DAY_SECONDS: f32 = 10.0 * 60.0;
/// D1 밤 길이 (초) — 10분
pub const NIGHT_SECONDS: f32 = 10.0 * 60.0;

/// D2 시작 체력(하트)
pub const PLAYER_START_HP: u32 = 7;
/// D2 레벨이 오를 때마다 늘어나는 체력
pub const PLAYER_HP_PER_LEVEL: u32 = 5;

/// D3 좀비 체력 (3번 때리면 쓰러짐). 몬스터 레벨이 올라도 그대로.
pub const ZOMBIE_HP: u32 = 3;
/// D3 레벨 1 좀비 공격력
pub const ZOMBIE_START_ATTACK: u32 = 5;
/// D3 몬스터 레벨이 오를 때마다 줄어드는 좀비 공격력
pub const ZOMBIE_ATTACK_DROP_PER_LEVEL: u32 = 2;
/// D3 좀비 공격력은 이보다 낮아지지 않는다
pub const ZOMBIE_MIN_ATTACK: u32 = 1;

/// M6 코인 최대 개수
pub const COIN_MAX: u32 = 5000;

/// D5 제작대 크기: 가로 4칸 × 세로 3칸
pub const CRAFT_GRID_WIDTH: usize = 4;
pub const CRAFT_GRID_HEIGHT: usize = 3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decided_values_match_plan() {
        assert_eq!(DAY_SECONDS, 600.0);
        assert_eq!(NIGHT_SECONDS, 600.0);
        assert_eq!(PLAYER_START_HP, 7);
        assert_eq!(PLAYER_HP_PER_LEVEL, 5);
        assert_eq!(ZOMBIE_HP, 3);
        assert_eq!(ZOMBIE_START_ATTACK, 5);
        assert_eq!(ZOMBIE_ATTACK_DROP_PER_LEVEL, 2);
        assert_eq!(ZOMBIE_MIN_ATTACK, 1);
        assert_eq!(COIN_MAX, 5000);
        assert_eq!((CRAFT_GRID_WIDTH, CRAFT_GRID_HEIGHT), (4, 3));
    }
}
