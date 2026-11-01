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

/// D5 광석 하나 캐는 시간 (초). 맨손은 곡괭이보다 느리다. M5에서 해 보고 조절한다.
pub const MINE_SECONDS_BARE_HAND: f32 = 3.0;
pub const MINE_SECONDS_PICKAXE: f32 = 1.0;

/// D5 제작대 크기: 가로 4칸 × 세로 3칸
pub const CRAFT_GRID_WIDTH: usize = 4;
pub const CRAFT_GRID_HEIGHT: usize = 3;


// ---- M1 이동 ----
// 좌표: 화면 픽셀 단위, 왼쪽 위가 (0, 0), 아래로 갈수록 y가 커진다.

/// M1 월드(땅) 가로 길이 — 화면 폭(960)과 같게.
pub const WORLD_WIDTH: f32 = 960.0;
/// M1 땅 윗면의 y 위치. 캐릭터 발이 여기 닿으면 땅 위에 서 있다.
pub const GROUND_Y: f32 = 440.0;

/// M1 캐릭터 크기 (가로, 세로)
pub const PLAYER_WIDTH: f32 = 48.0;
pub const PLAYER_HEIGHT: f32 = 64.0;

/// M1 좌우로 걷는 속도 (초당 픽셀)
pub const PLAYER_WALK_SPEED: f32 = 220.0;
/// M1 점프할 때 위로 튀어 오르는 속도 (초당 픽셀). 최고 높이 ≈ 속도² / (2 × 중력) ≈ 97픽셀.
pub const PLAYER_JUMP_SPEED: f32 = 520.0;
/// M1 중력: 매초 아래쪽 속도가 이만큼 커진다 (초당 픽셀²)
pub const GRAVITY: f32 = 1400.0;
/// M1 떨어지는 속도의 최대값 (초당 픽셀). 너무 빨라 상자를 뚫지 않게.
pub const MAX_FALL_SPEED: f32 = 900.0;
/// M1 한 번에 계산하는 가장 긴 시간 (초). 프레임이 길면 이만큼씩 잘게 나눠 계산한다.
pub const PHYSICS_STEP_SECONDS: f32 = 1.0 / 120.0;

// ---- M2 낮과 밤 ----

/// M2 노을 길이 (초). US-011: 낮의 마지막 1분. 낮의 마지막 이만큼 동안 하늘이 천천히 어두워진다.
pub const DUSK_SECONDS: f32 = 60.0;
/// M2 새벽 길이 (초). US-011: 밤의 마지막 1분. 밤의 마지막 이만큼 동안 하늘이 천천히 밝아진다.
pub const DAWN_SECONDS: f32 = 60.0;
/// D12 빨리감기(F 키)를 켰을 때 시간이 몇 배 빨리 가는지. 호출하는 쪽이 dt에 곱한다.
pub const DEV_FAST_FORWARD_SCALE: f32 = 10.0;

/// M2 한 프레임에 흘려 보내는 가장 긴 시간 (초). 다른 탭에 다녀오면 프레임 시간이 수십 초가 될 수 있는데,
/// 그만큼 한꺼번에 흘리지 않고 여기서 자른다 → 탭을 떠나 있는 동안은 게임이 멈춘 것처럼 된다 (US-009).
pub const MAX_FRAME_SECONDS: f32 = 0.25;

/// M2 게임을 시작할 때 캐릭터 왼쪽 위 x 위치 (상자가 없는 왼쪽).
pub const PLAYER_SPAWN_X: f32 = 120.0;

/// D11 코인 크기 (가로 = 세로, 픽셀)
pub const COIN_SIZE: f32 = 24.0;
/// D11 코인 아래쪽과 상자 윗면 사이 틈 (픽셀). 상자 위에 서면 몸이 코인에 닿는 높이.
pub const COIN_GAP_ABOVE_BOX: f32 = 8.0;
/// D11 코인을 주운 뒤 같은 자리에 다시 생길 때까지 (초)
pub const COIN_RESPAWN_SECONDS: f32 = 10.0;

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
        assert_eq!(COIN_RESPAWN_SECONDS, 10.0);
        assert_eq!(DEV_FAST_FORWARD_SCALE, 10.0);
        assert_eq!((DUSK_SECONDS, DAWN_SECONDS), (60.0, 60.0));
        assert!(MINE_SECONDS_BARE_HAND > MINE_SECONDS_PICKAXE);
        assert_eq!((CRAFT_GRID_WIDTH, CRAFT_GRID_HEIGHT), (4, 3));
    }
}
