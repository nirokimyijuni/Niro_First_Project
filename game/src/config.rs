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
/// D32 좀비 공격력. 몬스터 레벨이 올라도 그대로.
pub const ZOMBIE_ATTACK: u32 = 2;

/// M8 상점에서도 쓰는 코인 최대 개수
pub const COIN_MAX: u32 = 5000;

/// D5 광석 하나 캐는 시간 (초). 맨손은 곡괭이보다 느리다. M7(제작과 동굴)에서 해 보고 조절한다.
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
/// BUG-006 몸 가운데가 상자 밖일 때 상자 바깥쪽으로 미끄러지는 속도 (초당 픽셀).
/// 걷는 속도보다 느려야 상자 쪽으로 걸어서 다시 올라설 수 있다 (같으면 공중에 멈춘다).
pub const PLAYER_EDGE_SLIDE_SPEED: f32 = 120.0;
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
/// 해가 지는 18:00의 하늘 어둠 (0 한낮 ~ 1 한밤). 노을빛이 가장 센 때쯤이다 (sky.rs `dusk_glow`).
pub const SUNSET_DARKNESS: f32 = 0.6;
/// 해가 진 뒤 한밤처럼 어두워질 때까지 (초). 20초 = 게임 시계 24분.
pub const NIGHTFALL_SECONDS: f32 = 20.0;
/// D12 빨리감기(F 키)를 켰을 때 시간이 몇 배 빨리 가는지. 호출하는 쪽이 dt에 곱한다.
pub const DEV_FAST_FORWARD_SCALE: f32 = 10.0;
/// D33 새 게임을 빨리감기가 켜진 채로 시작하는지. 정식 릴리즈 전에 다시 정한다.
pub const DEV_FAST_FORWARD_AT_START: bool = true;

/// M2 한 프레임에 흘려 보내는 가장 긴 시간 (초). 다른 탭에 다녀오면 프레임 시간이 수십 초가 될 수 있는데,
/// 그만큼 한꺼번에 흘리지 않고 여기서 자른다 → 탭을 떠나 있는 동안은 게임이 멈춘 것처럼 된다 (US-009).
pub const MAX_FRAME_SECONDS: f32 = 0.25;

/// D14 하루가 몇 분인지 (게임 시계). 24시간 × 60분.
pub const MINUTES_PER_GAME_DAY: u32 = 24 * 60;
/// D14 낮이 시작되는 시각 (시). 06:00에 해가 뜨고 N일째·버틴 날이 1 는다.
pub const DAY_START_HOUR: u32 = 6;
/// D14 밤이 시작되는 시각 (시). 낮은 06:00~18:00, 밤은 18:00~06:00.
pub const NIGHT_START_HOUR: u32 = 18;
/// D14 게임 시계에서 낮이 차지하는 분 (06:00~18:00 = 12시간).
/// 실제 낮 길이(DAY_SECONDS)가 바뀌어도 이 12시간에 맞춰 시계가 빨라지거나 느려진다.
pub const DAY_GAME_MINUTES: u32 = (NIGHT_START_HOUR - DAY_START_HOUR) * 60;
/// D14 게임 시계에서 밤이 차지하는 분 (18:00~다음 날 06:00 = 12시간).
pub const NIGHT_GAME_MINUTES: u32 = MINUTES_PER_GAME_DAY - DAY_GAME_MINUTES;
/// D31 정보판 시간대 이름: (시작 시각, 이름). 시작 시각이 커지는 순서로 둔다.
pub const DAY_PERIODS: [(u32, &str); 6] = [
    (0, "새벽"),
    (DAY_START_HOUR, "아침"),
    (10, "낮"),
    (14, "오후"),
    (NIGHT_START_HOUR, "저녁"),
    (21, "밤"),
];

/// M2 게임을 시작할 때 캐릭터 왼쪽 위 x 위치 (상자가 없는 왼쪽).
pub const PLAYER_SPAWN_X: f32 = 120.0;

/// D11 코인 크기 (가로 = 세로, 픽셀)
pub const COIN_SIZE: f32 = 24.0;
/// D11 코인 아래쪽과 상자 윗면 사이 틈 (픽셀). 상자 위에 서면 몸이 코인에 닿는 높이.
pub const COIN_GAP_ABOVE_BOX: f32 = 8.0;
/// D11 코인을 주운 뒤 같은 자리에 다시 생길 때까지 (초)
pub const COIN_RESPAWN_SECONDS: f32 = 10.0;

// ---- M4 좀비 카스테라 (처음 값은 니로가 정했다. 해 보고 조절한다) ----

/// US-039 좀비 크기 (가로, 세로). 수산나가 정한 처음 값: 플레이어와 같다.
pub const ZOMBIE_WIDTH: f32 = 48.0;
pub const ZOMBIE_HEIGHT: f32 = 64.0;
/// US-039 좀비가 걷는 속도 (초당 픽셀). 플레이어보다 느려야 한다. 수산나가 정한 처음 값.
pub const ZOMBIE_WALK_SPEED: f32 = 80.0;
/// US-039 다음 좀비가 나올 때까지 (초). 3마리가 꽉 차 있는 동안은 세지 않는다.
pub const ZOMBIE_SPAWN_SECONDS: f32 = 20.0;
/// US-039 화면에 함께 있을 수 있는 좀비 수
pub const ZOMBIE_MAX_ON_SCREEN: usize = 3;
/// 좀비가 왼쪽 끝에서 나올 확률. 나머지는 오른쪽 끝.
pub const ZOMBIE_SPAWN_LEFT_CHANCE: f32 = 0.5;
/// 좀비가 나오는 화면 끝이 플레이어와 이만큼 가까우면 반대쪽 끝에서 나온다 (픽셀).
/// 나오자마자 맞는 일이 없게 수산나가 정한 처음 값.
pub const ZOMBIE_SPAWN_MIN_DISTANCE: f32 = 200.0;
/// US-042 나타난 좀비가 날아오르는 좀비일 확률 (3마리 중 1마리쯤)
pub const ZOMBIE_FLOAT_CHANCE: f32 = 1.0 / 3.0;
/// US-042 나타나고 몇 초 뒤 떠오르는지
pub const ZOMBIE_FLOAT_START_SECONDS: f32 = 10.0;
/// US-042 떠오르고 몇 초 뒤 팔을 내려 펑 터지는지
pub const ZOMBIE_FLOAT_SECONDS: f32 = 5.0;
/// US-042 떠오르는 속도 (초당 픽셀). 수산나가 정한 처음 값.
pub const ZOMBIE_FLOAT_RISE_SPEED: f32 = 30.0;
/// US-040 맞은 좀비가 반짝이는 시간 (초). 수산나가 정한 처음 값.
pub const ZOMBIE_HIT_FLASH_SECONDS: f32 = 0.2;

/// US-040 플레이어 앞쪽으로 공격이 닿는 거리 (픽셀). 수산나가 정한 처음 값.
pub const PLAYER_ATTACK_RANGE: f32 = 60.0;
/// US-041 맞은 뒤 다시 맞지 않는 시간 (초)
pub const PLAYER_HURT_INVINCIBLE_SECONDS: f32 = 1.0;

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
        assert_eq!(ZOMBIE_ATTACK, 2);
        assert_eq!(COIN_MAX, 5000);
        assert_eq!(COIN_RESPAWN_SECONDS, 10.0);
        assert_eq!(DEV_FAST_FORWARD_SCALE, 10.0);
        assert!(DEV_FAST_FORWARD_AT_START, "D33");
        assert_eq!((DUSK_SECONDS, DAWN_SECONDS), (60.0, 60.0));
        assert!(MINE_SECONDS_BARE_HAND > MINE_SECONDS_PICKAXE);
        assert_eq!((CRAFT_GRID_WIDTH, CRAFT_GRID_HEIGHT), (4, 3));
        assert_eq!((DAY_START_HOUR, NIGHT_START_HOUR), (6, 18));
        assert_eq!((DAY_GAME_MINUTES, NIGHT_GAME_MINUTES), (720, 720));
        // D14: 실제 1초 = 게임 1.2분
        assert_eq!(DAY_GAME_MINUTES as f32 / DAY_SECONDS, 1.2);
        assert_eq!(NIGHT_GAME_MINUTES as f32 / NIGHT_SECONDS, 1.2);
    }
}
