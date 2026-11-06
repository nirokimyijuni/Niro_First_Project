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

// ---- D36 랜덤 상자 (개수 말고는 수산나가 정한 처음 값. 해 보고 조절한다) ----

/// D36 새 게임의 상자 수
pub const BOX_COUNT_MIN: usize = 3;
pub const BOX_COUNT_MAX: usize = 10;
/// D36 상자 한 변 (픽셀). 큰 상자도 점프 최고 높이(약 97)보다 낮아야 넘을 수 있다.
pub const BOX_SMALL_SIZE: f32 = 48.0;
pub const BOX_BIG_SIZE: f32 = 64.0;
/// D36 상자가 큰 상자일 확률
pub const BOX_BIG_CHANCE: f32 = 0.5;
/// D36 상자를 놓을 수 있는 곳의 왼쪽·오른쪽 끝 (x). 바깥은 시작 자리와 몬스터가 나오는 곳이라 비운다.
/// 큰 상자 10개를 붙이면 640이라 이 안(648)에 꼭 들어간다.
pub const BOX_AREA_LEFT: f32 = 216.0;
pub const BOX_AREA_RIGHT: f32 = 864.0;
/// D36 상자 사이 틈은 0(붙음)이거나 이 폭 이상이다. 몸 폭과 딱 맞으면 끼어서 못 내려간다.
pub const BOX_MIN_GAP: f32 = PLAYER_WIDTH + 16.0;
/// D36 이웃한 상자 사이에 틈을 둘 확률. 나머지는 붙는다. 자리가 모자라면 더 붙는다.
pub const BOX_GAP_CHANCE: f32 = 0.5;

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
/// US-039 다음 몬스터가 나올 때까지 (초). 스켈레톤도 같이 쓴다. 화면이 꽉 차 있는 동안은 세지 않는다.
pub const ZOMBIE_SPAWN_SECONDS: f32 = 20.0;
/// US-039 화면에 함께 있을 수 있는 좀비 수
pub const ZOMBIE_MAX_ON_SCREEN: usize = 3;
/// 몬스터가 왼쪽 끝에서 나올 확률. 나머지는 오른쪽 끝 (D36: 플레이어 위치와 상관없다).
/// 플레이어가 그 끝 자리에 서 있을 때만 겹치지 않게 반대쪽 끝에서 나온다.
pub const ZOMBIE_SPAWN_LEFT_CHANCE: f32 = 0.5;
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

// ---- M5 스켈레톤과 레벨업 (D35 말고는 수산나가 정한 처음 값. 해 보고 조절한다) ----

/// US-045 몬스터를 쓰러뜨리면 얻는 경험치
pub const EXPERIENCE_PER_DEFEAT: u32 = 1;
/// D35 레벨이 오르는 경험치. 오르면 0부터 다시 센다.
pub const EXPERIENCE_PER_LEVEL: u32 = 5;

/// M5 좀비와 스켈레톤을 합쳐 화면에 함께 있을 수 있는 수. 좀비만은 `ZOMBIE_MAX_ON_SCREEN`까지.
pub const MONSTER_MAX_ON_SCREEN: usize = 4;
/// M5 나올 차례에 좀비 대신 스켈레톤이 나올 확률. 나오는 시간은 `ZOMBIE_SPAWN_SECONDS`를 같이 쓴다.
pub const SKELETON_SPAWN_CHANCE: f32 = 0.5;

/// M5 스켈레톤 크기 (가로, 세로)
pub const SKELETON_WIDTH: f32 = 48.0;
pub const SKELETON_HEIGHT: f32 = 64.0;
/// D35 스켈레톤 체력. 세 단계 모두 같다.
pub const SKELETON_HP: u32 = 3;
/// D35 단계가 나오는 몬스터 레벨. 레벨마다 그 단계만 나온다.
pub const SKELETON_HALF_MIN_LEVEL: u32 = 2;
pub const SKELETON_FULL_MIN_LEVEL: u32 = 3;
/// D35 단계별 공격력 (일반 화살, 절반 손, 완전체 총)
pub const SKELETON_NORMAL_ATTACK: u32 = 2;
pub const SKELETON_HALF_ATTACK: u32 = 3;
pub const SKELETON_FULL_ATTACK: u32 = 4;
/// M5 스켈레톤이 걷는 속도 (초당 픽셀). 플레이어보다 느려야 한다.
pub const SKELETON_WALK_SPEED: f32 = 70.0;
/// M5 활·총을 든 스켈레톤이 플레이어와 이만큼 가까워지면 걸음을 멈춘다 (픽셀, 몸 가운데끼리).
pub const SKELETON_SHOOT_DISTANCE: f32 = 240.0;
/// US-040 맞은 스켈레톤이 반짝이는 시간 (초)
pub const SKELETON_HIT_FLASH_SECONDS: f32 = ZOMBIE_HIT_FLASH_SECONDS;
/// US-049 튕겨 오는 공격이 이만큼 가까워지면 점프한다 (픽셀).
/// D36 둔하게: 52보다 작으면 총알은 늘 맞고, 30보다 크면 화살은 멀리서 튕겨 오면 피한다.
/// 상자 바로 옆에 선 스켈레톤은 화살도 맞는다.
pub const SKELETON_DODGE_SENSE_DISTANCE: f32 = 40.0;
/// US-049 피할 때 뛰어오르는 속도 (초당 픽셀). 발이 화살 높이보다 높이 올라가야 한다.
pub const SKELETON_DODGE_JUMP_SPEED: f32 = 560.0;

/// US-046 화살 쏘는 간격(초), 속도(초당 픽셀), 길이(픽셀)
pub const ARROW_INTERVAL_SECONDS: f32 = 3.0;
pub const ARROW_SPEED: f32 = 240.0;
pub const ARROW_WIDTH: f32 = 24.0;
/// US-048 총알 쏘는 간격(초), 속도(초당 픽셀), 길이(픽셀). 화살보다 빠르다.
pub const BULLET_INTERVAL_SECONDS: f32 = 2.0;
pub const BULLET_SPEED: f32 = 420.0;
pub const BULLET_WIDTH: f32 = 12.0;
/// M5 화살·총알의 두께 (픽셀)
pub const PROJECTILE_HEIGHT: f32 = 6.0;
/// US-049 화살·총알 가운데가 땅에서 떠 있는 높이 (픽셀).
/// 작은 상자(48)보다 높고 큰 상자(64)보다 낮아서, 작은 상자는 넘어가고 큰 상자에서는 튕긴다.
pub const PROJECTILE_FLIGHT_HEIGHT: f32 = 56.0;
/// US-049 튕긴 공격이 사라질 때까지 날아가는 거리 (픽셀)
pub const PROJECTILE_BOUNCED_RANGE: f32 = 400.0;

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
        assert_eq!(EXPERIENCE_PER_LEVEL, 5);
        assert_eq!(SKELETON_HP, 3);
        assert_eq!((SKELETON_HALF_MIN_LEVEL, SKELETON_FULL_MIN_LEVEL), (2, 3));
        assert_eq!((SKELETON_NORMAL_ATTACK, SKELETON_HALF_ATTACK, SKELETON_FULL_ATTACK), (2, 3, 4));
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
