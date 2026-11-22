//! 게임 수치를 한 곳에 모은다. 원본: docs/02-plan.md "결정 사항".
//! 숫자를 바꿀 때는 이 파일만 고치면 된다.

// 아직 logic/에서 쓰지 않는 상수가 있어서 경고를 끈다. 쓰기 시작하면 지워도 된다.
#![allow(dead_code)]

/// 낮 길이 (초) — 5분. 부모님 피드백의 임시 값 (D1은 10분). 시계는 낮 12시간에 맞춰 밤보다 2배 빨리 간다.
pub const DAY_SECONDS: f32 = 5.0 * 60.0;
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

/// M1 연습 맵 가로 길이 — 화면 폭(960)과 같게. 게임의 긴 맵은 `MAP_WIDTH`.
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
/// 전투가 한 프레임에 나누어 움직이는 최대 횟수. 한 프레임은 0.25초라 정상일 때 30번이면 끝난다.
/// 10000번이면 83초치라, 시험이 몇 초를 한 번에 넣어도(5초 = 600번) 잘리지 않는다.
/// 한도에 걸리면 남은 시간은 버린다 (다음 프레임으로 넘기지 않는다).
pub const COMBAT_STEP_LIMIT: u32 = 10_000;
/// 시계가 한 번에 넘기는 낮/밤 구간의 최대 수. 정상 프레임은 많아야 1번이고, 시험의 가장 큰 값도 20번대라 1000이면 충분하다.
/// 한도에 걸리면 남은 시간은 버리고 지금 구간의 처음(0초)으로 정리한다.
pub const CLOCK_PHASE_STEP_LIMIT: u32 = 1000;

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
pub const ZOMBIE_SPAWN_SECONDS: f32 = 10.0;
/// 부모님 피드백의 임시 값: 한 차례에 나오는 몬스터 수. 첫 마리 쪽은 랜덤이고, 다음은 번갈아 반대쪽이다.
pub const MONSTERS_PER_SPAWN: usize = 2;
/// US-039 화면에 함께 있을 수 있는 좀비 수
pub const ZOMBIE_MAX_ON_SCREEN: usize = 4;
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
/// US-041 맞은 뒤 다시 맞지 않는 시간 (초). 아이가 정한 임시 값이고 실제 시간이다. 시민의 펑도 같다.
pub const PLAYER_HURT_INVINCIBLE_SECONDS: f32 = 5.0;

// ---- M5 스켈레톤과 레벨업 (D35 말고는 수산나가 정한 처음 값. 해 보고 조절한다) ----

/// US-045 몬스터를 쓰러뜨리면 얻는 경험치
pub const EXPERIENCE_PER_DEFEAT: u32 = 1;
/// D35 레벨이 오르는 경험치. 오르면 0부터 다시 센다.
pub const EXPERIENCE_PER_LEVEL: u32 = 5;

/// M5 좀비와 스켈레톤을 합쳐 화면에 함께 있을 수 있는 수. 좀비만은 `ZOMBIE_MAX_ON_SCREEN`까지.
pub const MONSTER_MAX_ON_SCREEN: usize = 6;
/// M5 나올 차례에 좀비 대신 스켈레톤이 나올 확률. 나오는 시간은 `ZOMBIE_SPAWN_SECONDS`를 같이 쓴다.
pub const SKELETON_SPAWN_CHANCE: f32 = 0.5;

// ---- 날이 지날수록 몬스터가 많아진다 ([임시] 부모님 요청 2026-11-21, 해 보고 고친다) ----
// 1일째는 위의 값(`ZOMBIE_SPAWN_SECONDS`, `MONSTERS_PER_SPAWN`, `MONSTER_MAX_ON_SCREEN`, `ZOMBIE_MAX_ON_SCREEN`) 그대로다.
// 날 번호는 시계의 날(1일째 낮·밤 = 1)이다. 상자에서 나오는 몬스터도 같은 상한을 쓴다.
//
// 날  | 나오는 간격(초) | 한 번에 | 화면 최대 | 좀비 최대
//  1  | 10.0           | 2       | 6         | 4
//  2  |  9.3           | 2       | 7         | 4
//  3  |  8.6           | 2       | 8         | 5
//  4  |  7.9           | 3       | 9         | 6
//  5  |  7.2           | 3       | 10        | 6
//  6  |  6.5           | 3       | 11        | 7
//  7  |  6.0 (하한)    | 4       | 12 (상한) | 8

/// 하루마다 나오는 간격이 이만큼 짧아진다 (초).
pub const SPAWN_SECONDS_SHORTER_PER_DAY: f32 = 0.7;
/// 나오는 간격은 이보다 짧아지지 않는다 (초).
pub const SPAWN_SECONDS_SHORTEST: f32 = 6.0;
/// 이 날 수마다 한 번에 나오는 수가 1 는다 (1일째부터 센다: 4일째 +1, 7일째 +2).
pub const MONSTERS_PER_SPAWN_MORE_EVERY_DAYS: u32 = 3;
/// 한 번에 나오는 수는 이보다 많아지지 않는다.
pub const MONSTERS_PER_SPAWN_MOST: usize = 4;
/// 하루마다 화면 최대 수가 이만큼 는다.
pub const MONSTER_MAX_MORE_PER_DAY: usize = 1;
/// 화면 최대 수는 이보다 많아지지 않는다. 좀비 최대는 1일째 비율(4/6)대로 따라간다 (내림).
pub const MONSTER_MAX_MOST: usize = 12;

/// M5 스켈레톤 크기 (가로, 세로)
pub const SKELETON_WIDTH: f32 = 48.0;
pub const SKELETON_HEIGHT: f32 = 64.0;
/// D35 스켈레톤 체력. 세 단계 모두 같다.
pub const SKELETON_HP: u32 = 3;
/// US-135 (임시-166): 2일째부터 하루마다 새로 나오는 몬스터(좀비·스켈레톤) 체력이 이만큼 는다.
/// 1일째는 `ZOMBIE_HP`·`SKELETON_HP` 그대로다. 연습용 몬스터와 플레이어 체력은 상관없다.
pub const MONSTER_HP_MORE_PER_DAY: u32 = 1;
/// US-135: 몬스터 체력은 이보다 커지지 않는다 (7일째 9, 그 뒤로도 9).
pub const MONSTER_HP_MOST: u32 = 9;
/// D35 단계가 나오는 몬스터 레벨. 레벨마다 그 단계만 나온다.
pub const SKELETON_HALF_MIN_LEVEL: u32 = 2;
pub const SKELETON_FULL_MIN_LEVEL: u32 = 3;
/// D35 단계별 공격력 (일반 화살, 절반 손, 완전체 총)
pub const SKELETON_NORMAL_ATTACK: u32 = 2;
pub const SKELETON_HALF_ATTACK: u32 = 3;
pub const SKELETON_FULL_ATTACK: u32 = 4;
/// 절반 스켈레톤의 손이 몸 앞으로 닿는 거리 (픽셀). 부모님이 정한 임시 값.
pub const SKELETON_HAND_RANGE: f32 = 60.0;
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

// ---- M6 미스를 찾아라 (stories/backlog.md M6 맨 위 표. 임시-N은 아이·부모가 언제든 바꿀 수 있다) ----

/// M6 보이는 화면의 폭. 카메라와 몬스터 나오는 자리를 이 값으로 계산한다.
pub const SCREEN_WIDTH: f32 = 960.0;
/// D22 장소 이름. 왼쪽부터 화면 하나씩이다. 아이가 순서를 바꾸면 여기와 아래 x 값들을 같이 옮긴다.
pub const PLACE_NAMES: [&str; 4] = ["해변", "화산", "산", "집"];
/// US-050 긴 맵 가로 길이 (해변 → 화산 → 산 → 집). M1 연습 맵은 `WORLD_WIDTH`를 그대로 쓴다.
pub const MAP_WIDTH: f32 = SCREEN_WIDTH * PLACE_NAMES.len() as f32;

/// 임시-5 숨는 곳·시민·구조물 양옆으로 이만큼 안에는 상자를 놓지 않는다 (픽셀).
pub const NO_BOX_MARGIN: f32 = 64.0;
/// 부모님 결정 2026-11-15: 동굴 입구와 집 문 양옆으로 이만큼 안에는 상자를 놓지 않는다 (픽셀). `NO_BOX_MARGIN` 대신 쓴다.
/// 처음 값: 캐릭터 몸 2개 + 그곳의 들어가기 범위. 동굴은 `MINE_REACH`라서 화산에는 상자가 0개가 된다 (부모님 승인).
pub const BOX_CLEAR_AROUND_CAVE: f32 = PLAYER_WIDTH * 2.0 + MINE_REACH;
/// 집 문은 `HOUSE_DOOR_REACH`. 이보다 넓히면 집(시민·집 뒤 숨는 곳 사이)에 상자 자리가 없어진다.
pub const BOX_CLEAR_AROUND_DOOR: f32 = PLAYER_WIDTH * 2.0 + HOUSE_DOOR_REACH;

/// US-051 화산의 구멍: 왼쪽 x, 폭, 깊이. 깊이는 점프 최고 높이(약 97)보다 낮아야 나올 수 있다.
pub const HOLE_LEFT: f32 = 1620.0;
pub const HOLE_WIDTH: f32 = 160.0;
pub const HOLE_DEPTH: f32 = 64.0;

/// US-051 한 칸 높이. 점프 최고 높이(약 97)보다 낮아야 다음 칸에 오를 수 있다.
pub const CLIMB_STEP: f32 = 64.0;
/// US-051 나무 발판 (왼쪽 x, 땅에서 윗면까지 높이, 폭, 두께). 맨 아래는 땅까지 닿는 그루터기다.
/// 칸 사이를 벌린 까닭: 오른쪽에서 온 캐릭터가 그루터기 옆에서 머리를 박지 않고 뛰어넘게 하려고.
pub const TREE_BRANCHES: [(f32, f32, f32, f32); 3] = [
    (1920.0, CLIMB_STEP, 64.0, CLIMB_STEP),
    (2036.0, CLIMB_STEP * 2.0, 64.0, 16.0),
    (2152.0, CLIMB_STEP * 3.0, 64.0, 16.0),
];
/// US-051 바위 계단 (왼쪽 x, 땅에서 윗면까지 높이, 폭, 두께). 땅까지 닿는 산 모양이다.
/// 양쪽이 계단이어야 오른쪽에서 왼쪽으로도 넘어갈 수 있다 (US-050 왕복).
pub const ROCK_STEPS: [(f32, f32, f32, f32); 5] = [
    (2440.0, CLIMB_STEP, 64.0, CLIMB_STEP),
    (2504.0, CLIMB_STEP * 2.0, 64.0, CLIMB_STEP * 2.0),
    (2568.0, CLIMB_STEP * 3.0, 64.0, CLIMB_STEP * 3.0),
    (2632.0, CLIMB_STEP * 2.0, 64.0, CLIMB_STEP * 2.0),
    (2696.0, CLIMB_STEP, 64.0, CLIMB_STEP),
];

/// US-051 숨는 곳 (이름, 미스 몸 가운데 x, 땅에서 발까지 높이). 구멍 바닥은 땅보다 낮아서 음수다.
/// 줄을 더하면 그대로 새 숨는 곳이 된다 (갈 수 있는 자리여야 한다).
pub const HIDE_SPOTS: [(&str, f32, f32); 6] = [
    ("큰 돌 뒤", 700.0, 0.0),
    ("화산 뒤", 1300.0, 0.0),
    ("지하", 1700.0, -HOLE_DEPTH),
    ("나무 꼭대기", 2200.0, CLIMB_STEP * 3.0),
    ("산꼭대기", 2600.0, CLIMB_STEP * 3.0),
    ("집 뒤", 3700.0, 0.0),
];
/// US-051 미스 크기 (가로, 세로). 수산나가 정한 처음 값.
pub const MISS_WIDTH: f32 = 40.0;
pub const MISS_HEIGHT: f32 = 56.0;
/// US-051 찾았을 때 미스 대사. 아이가 정했다.
pub const MISS_FOUND_TEXT: &str = "아이템을 줄게!";
/// 찾은 미스가 다른 곳에 다시 숨을 때까지 (초). 아이가 정한 임시 값이고 실제 시간이다.
pub const MISS_RETURN_SECONDS: f32 = 20.0;

/// US-053 무기별 공격력. 아이가 정한 순서: 총 > 도끼 > 칼.
pub const HANDS_ATTACK: u32 = 1;
pub const SWORD_ATTACK: u32 = 2;
pub const AXE_ATTACK: u32 = 3;
pub const GUN_ATTACK: u32 = 4;
/// US-053 칼·도끼가 닿는 거리 (픽셀). 맨손과 같다.
pub const SWORD_RANGE: f32 = PLAYER_ATTACK_RANGE;
pub const AXE_RANGE: f32 = PLAYER_ATTACK_RANGE;
/// 임시-8 총이 닿는 거리 (픽셀)
pub const GUN_RANGE: f32 = 300.0;
/// US-052 무기 줍는 문구
pub const SWORD_PICKED_TEXT: &str = "칼을 주웠어요!";
pub const AXE_PICKED_TEXT: &str = "도끼를 주웠어요!";
pub const GUN_PICKED_TEXT: &str = "총을 주웠어요!";
/// US-052 땅에 놓인 무기 한 변 (픽셀). 수산나가 정한 처음 값.
pub const WEAPON_SIZE: f32 = 32.0;
/// 칼과 도끼가 다 있을 때 미스에게서 총이 나올 확률. 임시 값이다. 이미 가졌으면 나오지 않는다.
pub const GUN_DROP_CHANCE: f32 = 0.1;
/// 땅에 닿은 무기를 줍지 않으면 사라질 때까지 (초). 임시 값이고 실제 시간이다.
pub const WEAPON_GROUND_SECONDS: f32 = 15.0;
/// US-052 떨어지는 속도 (초당 픽셀). 실제 시간이다 (빨리감기로 빨라지지 않는다).
pub const WEAPON_FALL_SPEED: f32 = 300.0;

/// 폭발 시민 자리 (몸 가운데 x). 아이가 정한 임시 값: 화산과 집에 1명씩.
pub const CITIZEN_XS: [f32; 2] = [1500.0, 3200.0];
/// BUG-019 폭발 시민 몸 가운데에서 양옆으로 이만큼 안에는 상자를 놓지 않는다 (픽셀).
/// 상자에서 내려와 달리다가 시민을 뛰어넘을 자리다. 달려 뛰면 약 163픽셀을 날아간다.
pub const CITIZEN_RUN_UP_ROOM: f32 = 160.0;
/// US-054 시민 크기 (가로, 세로). 미스와 같다.
pub const CITIZEN_WIDTH: f32 = MISS_WIDTH;
pub const CITIZEN_HEIGHT: f32 = MISS_HEIGHT;
/// 임시-9 펑이 닿는 거리 (픽셀, 몸 가운데끼리)
pub const CITIZEN_BLAST_RANGE: f32 = 120.0;
/// US-054 펑에 줄어드는 플레이어 체력
pub const CITIZEN_BLAST_DAMAGE: u32 = 2;
/// US-054 터진 시민이 다시 나타날 때까지 (초). 실제 시간이다.
pub const CITIZEN_RESPAWN_SECONDS: f32 = 30.0;
/// US-054 터질 때 문구
pub const CITIZEN_POP_TEXT: &str = "펑!";
/// 일반 시민이 멈췄을 때 말을 할 확률. 임시 값이다.
pub const CITIZEN_TALK_SHARE: f32 = 0.5;
/// 아이가 정한 문구. 화면이 쓴다.
pub const ZOMBIE_APPEAR_TEXT: &str = "얍얍 공격하라";
pub const SKELETON_DODGE_TEXT: &str = "만세";
pub const CITIZEN_HELP_TEXT: &str = "도움이 필요해요";
/// 세로로 열렸을 때 "가로로 돌려 주세요" 아래 한 줄. 안드로이드는 "자동 회전", 아이폰은 "세로 방향 잠금"이라 한 문장에 둘 다 담는다.
pub const ROTATE_HINT_LOCK_TEXT: &str = "자동 회전(세로 잠금)이 켜져 있으면 꺼 주세요";

/// 임시 일반 시민: 장소마다 1명, 모두 `PLACE_NAMES` 수만큼. 크기는 미스와 같고, 걷는 속도는 초당 픽셀이다.
pub const WALKING_CITIZEN_WIDTH: f32 = MISS_WIDTH;
pub const WALKING_CITIZEN_HEIGHT: f32 = MISS_HEIGHT;
pub const WALKING_CITIZEN_SPEED: f32 = 40.0;
/// BUG-016 임시 값 (부모님 답을 기다린다): 이만큼 걷고, 이만큼 멈춘다 (초). 실제 시간이다.
pub const WALKING_CITIZEN_WALK_SECONDS: f32 = 6.0;
pub const WALKING_CITIZEN_PAUSE_SECONDS: f32 = 2.0;

/// 부모님이 정한 임시 값: 벽 안쪽에서 나오는 몬스터는 주인공에게서 이만큼은 떨어진다 (픽셀, 몸 가운데끼리).
pub const MONSTER_SPAWN_MIN_DISTANCE: f32 = 160.0;

/// D63 버틴 날이 이 값이 되는 아침에 게임을 클리어한다. 바꾸면 `GAME_CLEAR_TEXT`의 숫자도 같이 고친다.
pub const DAYS_TO_CLEAR: u32 = 7;
/// US-068 클리어 축하 문구
pub const GAME_CLEAR_TEXT: &str = "축하해요! 7일을 버텼어요!";

/// 점수 가중치. 주운 코인, 레벨업, 해치운 몬스터 하나마다 얻는 점수.
pub const SCORE_PER_COIN: u32 = 1;
pub const SCORE_PER_LEVEL_UP: u32 = 1;
pub const SCORE_PER_MONSTER: u32 = 1;

/// 임시-3, 임시-4 주인공에게서 이만큼 넘게 멀어진 몬스터와 화살·총알은 사라진다 (픽셀, 몸 가운데끼리)
pub const FORGET_DISTANCE: f32 = 1200.0;

/// D67 수학 문제 (US-074~076). 부모님 답(2026-11-08)으로 바꾼 임시 값이다. 부모님이 바꿀 수 있다.
/// 종류마다 켜고 끈다. false면 그 종류는 나오지 않는다.
pub const MATH_MULTIPLY_ON: bool = true;
pub const MATH_ADD_TWO_ON: bool = true;
pub const MATH_ADD_THREE_ON: bool = true;
pub const MATH_SUBTRACT_ON: bool = true;
/// 곱셈: 이 단들에 1 ~ `MATH_TIMES_MAX`를 곱한다 (한 자리 곱하기 전부)
pub const MATH_TIMES_TABLES: [u32; 8] = [2, 3, 4, 5, 6, 7, 8, 9];
pub const MATH_TIMES_MAX: u32 = 9;
/// 두 자리 수 더하기: 두 수 모두 이 범위 (예: 47 + 85). 부모님 말(2026-11-08): 두 자리 수끼리만.
pub const MATH_ADD_TWO_RANGE: (u32, u32) = (10, 99);
/// 세 수 연속 더하기: 세 수 모두 이 범위 (예: 7 + 15 + 3). 부모님 말(2026-11-08): 20 이하, 가장 작은 수 1은 임시.
pub const MATH_ADD_THREE_RANGE: (u32, u32) = (1, 20);
/// 뺄셈 (US-074 표): 빼지는 수는 이 범위, 빼는 수는 `MATH_SUBTRACT_TAKE_MIN` ~ (빼지는 수 - 1). 답은 1 이상이다.
/// D88 (부모님 결정 2026-11-14): 두 수 모두 두 자리(10~99)로만 뺀다 (예: 52 - 17). 빼는 수가 10 이상이고 답이 1 이상이라 빼지는 수는 11부터다.
pub const MATH_SUBTRACT_RANGE: (u32, u32) = (11, 99);
pub const MATH_SUBTRACT_TAKE_MIN: u32 = 10;
/// 답은 0 ~ 이 값. 숫자판으로 넣는 자리 수는 `MATH_ANSWER_DIGITS`까지다.
pub const MATH_ANSWER_MAX: u32 = 999;
pub const MATH_ANSWER_DIGITS: usize = 3;
/// 문제 주사위를 상자·무기 뽑기와 따로 굴리려고 게임 시드에 섞는 값
pub const MATH_SEED_MIX: u32 = 0x5EED_0067;
/// 아이템(지금은 미스가 준 무기)을 이만큼 얻을 때마다 한 번 문제가 나온다. 밤이 될 때(18:00)도 한 번 나온다.
pub const MATH_ITEMS_PER_QUESTION: u32 = 2;
/// 한 문제에 답할 기회. 다 틀리면 정답을 보여 주고 체력이 준다.
pub const MATH_TRIES: u32 = 3;
/// 결과를 보여 주는 시간 (실제 초). 이 동안도 게임은 멈춰 있다.
pub const MATH_RIGHT_SHOW_SECONDS: f32 = 2.0;
pub const MATH_WRONG_SHOW_SECONDS: f32 = 3.0;
/// 기회를 다 쓰고 틀리면 줄어드는 체력. 0이 되면 보통 쓰러짐과 같다. 무적 시간은 생기지 않는다.
pub const MATH_WRONG_HP_LOSS: u32 = 2;
/// 문제 창 글. 틀림 글의 "□이야"는 정답에 맞춰 "13이야", "2야"가 된다.
pub const MATH_TITLE_TEXT: &str = "잠깐 퀴즈!";
/// 미스가 문제를 낼 때 하는 말 (감독이 정한 임시 말)
pub const MATH_MISS_ASK_TEXT: &str = "나랑 같이 풀어 볼래?";
pub const MATH_RIGHT_TEXT: &str = "넌 최고야!";
pub const MATH_TRY_AGAIN_TEXT: &str = "너무 아쉽다! 다시 해 볼까?";
pub const MATH_WRONG_TEXT: &str = "너무 아쉽다! 정답은 □이야";
/// 틀린 뒤 남은 기회 글. □ 자리에 남은 기회 수가 들어간다 (예: "기회가 2번 남았어").
pub const MATH_TRIES_LEFT_TEXT: &str = "기회가 □번 남았어";
/// 문제 이야기에 나오는 이름 (부모님 답: 우리 팀과 미스, 카스테라). 아무거나 하나가 이름 자리에 들어간다.
pub const MATH_STORY_NAMES: &[&str] = &["니로", "엘리", "로건", "수산나", "헤일리", "밥", "케빈", "미스", "카스테라"];
/// 상황 문제 문장 틀. {a}, {b}, {c} 자리에 수가, {이름:이}·{이름:은}·{이름:을} 자리에 이름과
/// 받침에 맞는 말(이/가, 은/는, 을/를)이 들어간다. 수는 늘 a, b, c 순서로 쓴다.
/// 종류마다 3개 이상 둔다 (같은 문장이 연달아 나오지 않게).
/// 곱셈: {a}개씩 {b}묶음
pub const MATH_MULTIPLY_STORIES: &[&str] = &[
    "{이름:이} 보니 좀비 카스테라가 한 줄에 {a}마리씩 {b}줄 있어요. 모두 몇 마리일까요?",
    "{이름:이} 상자마다 코인을 {a}개씩 넣었어요. 상자가 {b}개예요. 코인은 모두 몇 개일까요?",
    "{이름:은} 하루에 코인을 {a}개씩 주웠어요. {b}일 동안 주우면 모두 몇 개일까요?",
    "{이름:이} 사탕을 {a}개씩 친구 {b}명에게 줬어요. 사탕은 모두 몇 개일까요?",
];
/// 두 수 더하기: {a} + {b}
pub const MATH_ADD_TWO_STORIES: &[&str] = &[
    "{이름:이} 코인 {a}개를 모았어요. 오늘 {b}개를 더 주웠어요. 모두 몇 개일까요?",
    "{이름:은} 어제 좀비 카스테라를 {a}마리, 오늘 {b}마리 만났어요. 모두 몇 마리일까요?",
    "가게에 사탕이 {a}개 있었어요. {이름:이} {b}개를 더 가져왔어요. 사탕은 모두 몇 개일까요?",
];
/// 세 수 더하기: {a} + {b} + {c}
pub const MATH_ADD_THREE_STORIES: &[&str] = &[
    "{이름:이} 첫째 날 코인 {a}개, 둘째 날 {b}개, 셋째 날 {c}개를 주웠어요. 모두 몇 개일까요?",
    "{이름:은} 꽃을 {a}송이, {b}송이, {c}송이 땄어요. 꽃은 모두 몇 송이일까요?",
    "{이름:을} 도우려고 친구들이 별을 {a}개, {b}개, {c}개 모았어요. 별은 모두 몇 개일까요?",
];
/// 뺄셈: {a} - {b}
pub const MATH_SUBTRACT_STORIES: &[&str] = &[
    "{이름:은} 화살을 {a}개 가지고 있었는데 {b}개를 쐈어요. 몇 개 남았을까요?",
    "{이름:이} 코인 {a}개를 가지고 있었어요. 가게에서 {b}개를 썼어요. 몇 개 남았을까요?",
    "{이름:이} 사탕 {a}개 중에서 {b}개를 친구에게 나눠 줬어요. 몇 개 남았을까요?",
];

// ---- D82·D106 잠깐 퀴즈 영어 낱말 문제 (US-087·088, [임시-56]~[임시-62]) ----
// 나오는 때, 멈춤, 맞힘·정답 글 시간(MATH_RIGHT_SHOW_SECONDS, MATH_WRONG_SHOW_SECONDS),
// 제목, 미스의 말, 남은 기회 글은 수학 문제와 같은 값을 쓴다.

/// 임시-56 잠깐 퀴즈가 열릴 때 영어 문제가 나올 확률 (%). 나머지는 수학. 0이면 수학만, 100이면 영어만.
pub const ENGLISH_QUIZ_PERCENT: u32 = 50;
/// 임시-58 영어 문제 한 문제의 기회 (부모님 확정)
pub const ENGLISH_TRIES: u32 = 2;
/// 임시-58 두 번 다 틀리면 줄어드는 체력. 갑옷이어도 준다 (수학과 같다)
pub const ENGLISH_WRONG_HP_LOSS: u32 = 2;
/// 임시-57 한 문제의 보기 수 (2×2 칸)
pub const ENGLISH_CHOICE_COUNT: usize = 4;
/// 수학·영어 고르기와 영어 문제 뽑기를 상자·미스·광석·수학·미션 뽑기와 따로 굴리려고 게임 시드에 섞는 값
pub const QUIZ_SUBJECT_SEED_MIX: u32 = 0x5EED_0056;
pub const ENGLISH_SEED_MIX: u32 = 0x5EED_0082;
/// 임시-61 묻는 글. (가) 그림 보고 낱말 고르기, (나) 낱말 보고 그림 고르기
pub const ENGLISH_PICTURE_TO_WORD_ASK_TEXT: &str = "이 그림은 영어로 뭘까?";
pub const ENGLISH_WORD_TO_PICTURE_ASK_TEXT: &str = "이 낱말에 맞는 그림은?";
/// 임시-58 두 번 다 틀렸을 때 글. □에 정답 낱말이 들어간다 ("너무 아쉽다! 정답은 Kite 야")
pub const ENGLISH_WRONG_TEXT: &str = "너무 아쉽다! 정답은 □ 야";

/// 임시-60 Tree는 그림 파일 대신 게임의 나무 그림(코드로 그린 것)을 쓴다. 로건은 이 이름을 보면 나무를 그린다.
pub const ENGLISH_GAME_TREE_PICTURE: &str = "game-tree";
/// 임시-59 영어 낱말 표 (48개, 부모님이 준 차례. Pole은 2026-11-20 부모님 결정으로 뺌). 한 줄에 (낱말, 그림 이름) 하나.
/// 낱말은 앞 글자만 대문자. 그림은 헤일리가 그린 `en-<소문자 낱말>.png` (D106).
/// 줄을 더하면 문제에 나오고, 빼면 안 나온다. 그림이 늦은 낱말은 잠시 빼 둔다.
/// 움직임 낱말(cry, fly 등)은 나중에 넣는다 (D104).
pub const ENGLISH_WORDS: &[(&str, &str)] = &[
    ("Acorn", "en-acorn.png"),
    ("Apron", "en-apron.png"),
    ("Baby", "en-baby.png"),
    ("Fan", "en-fan.png"),
    ("Jacket", "en-jacket.png"),
    ("Kangaroo", "en-kangaroo.png"),
    ("Lamp", "en-lamp.png"),
    ("Maple", "en-maple.png"),
    ("Paper", "en-paper.png"),
    ("Radio", "en-radio.png"),
    ("Table", "en-table.png"),
    ("Beehive", "en-beehive.png"),
    ("Cheek", "en-cheek.png"),
    ("Cheese", "en-cheese.png"),
    ("Elephant", "en-elephant.png"),
    ("Feet", "en-feet.png"),
    ("Hen", "en-hen.png"),
    ("Net", "en-net.png"),
    ("Seed", "en-seed.png"),
    ("Teeth", "en-teeth.png"),
    ("Tree", ENGLISH_GAME_TREE_PICTURE),
    ("Vest", "en-vest.png"),
    ("Dish", "en-dish.png"),
    ("Fire", "en-fire.png"),
    ("Kite", "en-kite.png"),
    ("Knife", "en-knife.png"),
    ("Lizard", "en-lizard.png"),
    ("Pine", "en-pine.png"),
    ("Slide", "en-slide.png"),
    ("Bib", "en-bib.png"),
    ("Bone", "en-bone.png"),
    ("Cone", "en-cone.png"),
    ("Doll", "en-doll.png"),
    ("Hole", "en-hole.png"),
    ("Hose", "en-hose.png"),
    ("Mole", "en-mole.png"),
    ("Mop", "en-mop.png"),
    ("Phone", "en-phone.png"),
    ("Rope", "en-rope.png"),
    ("Rose", "en-rose.png"),
    ("Smoke", "en-smoke.png"),
    ("Ice", "en-ice.png"),
    ("King", "en-king.png"),
    ("Sky", "en-sky.png"),
    ("Watch", "en-watch.png"),
    ("Candy", "en-candy.png"),
    ("Insect", "en-insect.png"),
    ("Tent", "en-tent.png"),
];
/// 임시-62 그림이 비슷해 헷갈리는 묶음 (D106). 한 묶음의 두 낱말은 한 문제의 보기 4개에 함께 나오지 않는다.
/// 한 낱말이 여러 묶음에 들 수 있다 (Pine). 줄을 더하면 묶음이 는다.
pub const ENGLISH_CONFUSING_GROUPS: &[&[&str]] = &[
    &["Pine", "Tree", "Maple"],
    &["Feet", "Teeth", "Cheek"],
    &["Hole", "Mole"],
    &["Apron", "Bib", "Vest", "Jacket"],
    &["Baby", "Doll", "King"],
    &["Fire", "Smoke", "Sky"],
    &["Hose", "Rope", "Mop"],
    &["Watch", "Phone"],
    &["Acorn", "Seed"],
    &["Cone", "Tent", "Pine"],
    &["Insect", "Beehive"],
    &["Ice", "Candy"],
];

// ---- M7 제작과 동굴 (stories/backlog.md M7 맨 위 표. 임시-N은 아이·부모가 언제든 바꿀 수 있다) ----
// x는 모두 몸 가운데다. 시간은 모두 실제 시간이라 빨리감기로 빨라지지 않는다.

/// 임시-16, 임시-36 캐기와 입구·출구·제작대 쓰기가 몸 양쪽(등 뒤 포함)으로 닿는 거리 (픽셀).
/// 닿는 것 중 가장 가까운 하나만 쓰고, 거리가 같으면 바라보는 쪽을 먼저 쓴다.
pub const MINE_REACH: f32 = 100.0;

/// 임시-14 나무 자리. 산의 나뭇가지 발판과는 다른 나무다. 줄을 더하면 그대로 새 나무가 된다.
pub const TREE_XS: [f32; 4] = [900.0, 1850.0, 2840.0, 3040.0];
/// M7 나무 크기 (가로, 세로). 수산나가 정한 처음 값.
pub const TREE_WIDTH: f32 = 48.0;
pub const TREE_HEIGHT: f32 = 96.0;
/// 임시-14 캔 나무가 다시 자랄 때까지 (초)
pub const TREE_REGROW_SECONDS: f32 = 30.0;

/// 임시-11 화산의 동굴 입구 자리와 크기. 크기는 수산나가 정한 처음 값.
pub const CAVE_ENTRANCE_X: f32 = 1060.0;
pub const CAVE_ENTRANCE_WIDTH: f32 = 96.0;
pub const CAVE_ENTRANCE_HEIGHT: f32 = 96.0;
/// 임시-12 동굴 안 출구 자리. 크기는 입구와 같다.
pub const CAVE_EXIT_X: f32 = 60.0;
/// 임시-12 동굴 안 광석 덩이 자리. 줄을 더하면 그대로 새 덩이가 된다.
pub const ORE_LUMP_XS: [f32; 6] = [300.0, 400.0, 500.0, 620.0, 740.0, 860.0];
/// M7 광석 덩이 크기 (가로, 세로). 수산나가 정한 처음 값.
pub const ORE_LUMP_WIDTH: f32 = 48.0;
pub const ORE_LUMP_HEIGHT: f32 = 48.0;
/// 임시-15 캔 덩이가 다시 생길 때까지 (초)
pub const ORE_REGROW_SECONDS: f32 = 30.0;
/// 임시-15, D75 광석이 나올 확률. 일곱을 더하면 1이어야 한다.
/// 희귀한 순서 다이아몬드 > 에메랄드 > 루비는 아이가 정했다 (US-079).
pub const ORE_IRON_CHANCE: f32 = 0.60;
pub const ORE_COPPER_CHANCE: f32 = 0.15;
pub const ORE_SILVER_CHANCE: f32 = 0.10;
pub const ORE_RUBY_CHANCE: f32 = 0.07;
pub const ORE_GOLD_CHANCE: f32 = 0.05;
pub const ORE_EMERALD_CHANCE: f32 = 0.025;
pub const ORE_DIAMOND_CHANCE: f32 = 0.005;
/// 임시-13 밤에도 동굴에 들어갈지. false면 18:00에 입구 앞으로 나온다.
/// true면 밤에도 있을 수 있고, 몬스터는 입구 앞에서 기다린다.
pub const CAVE_OPEN_AT_NIGHT: bool = false;
/// 동굴에 있는 동안 맵이 보는 주인공 몸의 y. 입구 위 먼 하늘이라서 코인·미스·시민·몬스터에 닿지 않는다.
pub const CAVE_AWAY_Y: f32 = -10000.0;

/// 임시-18 집의 제작대 자리와 크기. 크기는 수산나가 정한 처음 값.
pub const WORKBENCH_X: f32 = 2930.0;
pub const WORKBENCH_WIDTH: f32 = 64.0;
pub const WORKBENCH_HEIGHT: f32 = 64.0;
/// 임시-18 제작대를 여는 동안 시계·몬스터·코인이 멈추는지.
/// false면 게임은 흐르고 주인공만 서 있다.
pub const CRAFTING_PAUSES_GAME: bool = true;
/// 임시-19 곡괭이 모양. D5 그림이고 아이 그림이 오면 바꾼다 (D53). 칸이 어긋나도 안 나온다.
pub const CRAFT_WOOD_MARK: char = '나';
pub const CRAFT_IRON_MARK: char = '철';
pub const PICKAXE_SHAPE: [[char; CRAFT_GRID_WIDTH]; CRAFT_GRID_HEIGHT] = [
    ['철', '나', '철', ' '],
    [' ', '나', ' ', ' '],
    [' ', '나', '나', ' '],
];

/// M7 재료 이름. 정보판이 쓴다.
pub const WOOD_NAME: &str = "나무";
pub const IRON_NAME: &str = "철광석";
pub const COPPER_NAME: &str = "동";
pub const SILVER_NAME: &str = "은";
pub const GOLD_NAME: &str = "금";
/// D75 새 광물 이름 (US-079)
pub const RUBY_NAME: &str = "루비";
pub const EMERALD_NAME: &str = "에메랄드";
pub const DIAMOND_NAME: &str = "다이아몬드";
pub const RUBY_MINED_TEXT: &str = "루비를 캤어요!";
pub const EMERALD_MINED_TEXT: &str = "에메랄드를 캤어요!";
pub const DIAMOND_MINED_TEXT: &str = "영차! 다이아몬드를 캤어요!";
/// US-065 캤을 때 문구
pub const WOOD_MINED_TEXT: &str = "나무를 캤어요!";
pub const IRON_MINED_TEXT: &str = "철광석을 캤어요!";
pub const COPPER_MINED_TEXT: &str = "동을 캤어요!";
pub const SILVER_MINED_TEXT: &str = "은을 캤어요!";
pub const GOLD_MINED_TEXT: &str = "금을 캤어요!";
/// US-064 동굴 문구와 장소 이름
pub const CAVE_NAME: &str = "동굴";
pub const CAVE_NIGHT_OUT_TEXT: &str = "밤이 왔어요! 밖으로 나왔어요";
pub const CAVE_CLOSED_TEXT: &str = "동굴은 낮에 열려요";
/// 임시-36 폰 버튼 글. 그때 쓸 것에 맞게 바뀐다 (`Usable::button_text`).
pub const MINE_BUTTON_TEXT: &str = "캐기";
pub const CAVE_ENTER_BUTTON_TEXT: &str = "들어가기";
pub const CAVE_EXIT_BUTTON_TEXT: &str = "나가기";
pub const WORKBENCH_BUTTON_TEXT: &str = "만들기";

// ---- M8 상점 (stories/backlog.md M8 맨 위 표, docs/02-plan.md D65·D71. 임시-N은 아이·부모가 언제든 바꿀 수 있다) ----

/// 임시-22 집 문 가운데 x (집 그림 가운데). 크기는 집 그림의 문과 같다.
pub const HOUSE_DOOR_X: f32 = 3555.0;
pub const HOUSE_DOOR_WIDTH: f32 = 68.0;
pub const HOUSE_DOOR_HEIGHT: f32 = 112.0;
/// [임시] 부모님 결정 2026-11-15: 집 문은 바로 앞에서만 들어간다. 몸 가운데가 문 폭 안이거나 문 좌우로 이만큼(픽셀) 안일 때.
/// 다른 것(나무·제작대·동굴 입구)은 그대로 `MINE_REACH`를 쓴다.
pub const HOUSE_DOOR_REACH: f32 = 8.0;
/// 임시-36 폰 버튼 글
pub const HOUSE_DOOR_BUTTON_TEXT: &str = "들어가기";
/// 임시-24 상점 화면이 열려 있는 동안 게임 속 시간이 모두 멈춘다
pub const SHOP_PAUSES_GAME: bool = true;

/// 가격 (코인, D65, 아이가 정함). 칼·도끼·총은 상점에서 팔지 않는다 (부모님 결정 2026-11-14).
pub const HEALTH_POTION_PRICE: u32 = 100;
pub const POISON_POTION_PRICE: u32 = 100;
pub const SKATES_PRICE: u32 = 200;
pub const RAINBOW_ARMOR_PRICE: u32 = 500;
pub const DIAMOND_ARMOR_PRICE: u32 = 1000;

/// 상점 물건 이름
pub const HEALTH_POTION_NAME: &str = "체력 포션";
pub const POISON_POTION_NAME: &str = "독약 포션";
pub const SKATES_NAME: &str = "스케이트";
pub const RAINBOW_ARMOR_NAME: &str = "무지개 갑옷";
pub const DIAMOND_ARMOR_NAME: &str = "다이아몬드 갑옷";

/// 상점 문구
pub const SHOP_NOT_ENOUGH_COINS_TEXT: &str = "코인이 모자라요";
pub const SHOP_ALREADY_HAVE_TEXT: &str = "가지고 있어요";
pub const SHOP_NONE_TO_SELL_TEXT: &str = "팔 것이 없어요";
/// 상점 화면 안내 글
pub const SHOP_HINT_TEXT: &str = "물건을 눌러서 사요";
/// 산 뒤, 판 뒤 잠깐 나오는 글
pub const SHOP_BOUGHT_TEXT: &str = "샀어요!";
pub const SHOP_SOLD_TEXT: &str = "팔았어요!";
/// 06:00에 스케이트가 벗겨질 때 나오는 글 (`ShopEvent::SkatesOff`)
pub const SKATES_OFF_TEXT: &str = "아침이 되어 스케이트를 벗었어요";
/// 갑옷 시간이 끝날 때 나오는 글 (`ShopEvent::ArmorOff`)
pub const ARMOR_OFF_TEXT: &str = "갑옷 시간이 끝났어요";

/// 임시-25 스케이트를 신으면 걷는 빠르기가 이만큼 곱해진다.
pub const SKATES_SPEED_MULTIPLIER: f32 = 1.5;
/// 스케이트가 벗겨지는 아침 (부모님 결정 2026-11-21): 낮에 신으면 다음 06:00(1번째 아침),
/// 밤(18:00~06:00)에 신으면 하루 더 지난 06:00(2번째 아침)에 벗겨진다. 상자에서 얻어 신어도 같다.
pub const SKATES_MORNINGS_WHEN_WORN_BY_DAY: u32 = 1;
pub const SKATES_MORNINGS_WHEN_WORN_AT_NIGHT: u32 = 2;

/// 임시-26 갑옷 무적 시간 (게임 속 초 = 빨리감기 배수만큼 빨리 준다, D71). 다이아몬드 12분, 무지개 72초.
pub const DIAMOND_ARMOR_SECONDS: f32 = 12.0 * 60.0;
pub const RAINBOW_ARMOR_SECONDS: f32 = 72.0;

/// 임시-27 체력 포션이 채우는 체력
pub const HEALTH_POTION_HEAL: u32 = 5;
/// 임시-28 독약 포션: 주인공 몸 가운데에서 이 거리(픽셀) 안의 몬스터 몸 가운데에 뿌린다
pub const POISON_RANGE: f32 = 200.0;
/// 임시-28 독에 걸린 몬스터는 이 실제 초마다 체력이 `POISON_DAMAGE`씩 준다
pub const POISON_TICK_SECONDS: f32 = 1.0;
pub const POISON_DAMAGE: u32 = 1;

/// 임시-38 광물 파는 값 (코인). 철광석은 팔지 않는다.
pub const GOLD_SELL_PRICE: u32 = 1000;
pub const SILVER_SELL_PRICE: u32 = 500;
pub const COPPER_SELL_PRICE: u32 = 200;
// 팔아서 코인이 `COIN_MAX`를 넘어도 막지 않는다. 코인은 `COIN_MAX`에서 멈춘다 (부모님 결정, US-072).

// ---- M8 새 조합 네 가지 (D74~D80, stories/backlog.md US-078~US-083, 임시-39~48) ----

/// 임시-39 조합판 표시. 곡괭이의 '나'·'철'(`CRAFT_WOOD_MARK`, `CRAFT_IRON_MARK`)과 함께 쓴다. ' '는 빈칸이다.
pub const CRAFT_COPPER_MARK: char = '동';
pub const CRAFT_SILVER_MARK: char = '은';
pub const CRAFT_GOLD_MARK: char = '금';
pub const CRAFT_RUBY_MARK: char = '루';
pub const CRAFT_EMERALD_MARK: char = '에';
pub const CRAFT_DIAMOND_MARK: char = '다';
/// D76 자동차 (철광석 4, 나무 3). 모든 조합은 왼쪽 3칸이고 4번째 칸은 비운다 (D74).
pub const CAR_SHAPE: [[char; CRAFT_GRID_WIDTH]; CRAFT_GRID_HEIGHT] = [
    ['철', '나', '철', ' '],
    [' ', '나', ' ', ' '],
    ['철', '나', '철', ' '],
];
/// D77 엔더열매 (동 9)
pub const ENDER_FRUIT_SHAPE: [[char; CRAFT_GRID_WIDTH]; CRAFT_GRID_HEIGHT] = [
    ['동', '동', '동', ' '],
    ['동', '동', '동', ' '],
    ['동', '동', '동', ' '],
];
/// D78 다이아래더라이트 곡괭이 (철광석 2, 다이아몬드 1, 나무 3)
pub const DIAMOND_PICKAXE_SHAPE: [[char; CRAFT_GRID_WIDTH]; CRAFT_GRID_HEIGHT] = [
    ['철', '다', '철', ' '],
    [' ', '나', ' ', ' '],
    [' ', '나', '나', ' '],
];
/// D79 주사기총 (은 3, 나무 2, 에메랄드 3, 루비 1)
pub const SYRINGE_GUN_SHAPE: [[char; CRAFT_GRID_WIDTH]; CRAFT_GRID_HEIGHT] = [
    ['은', '나', '나', ' '],
    ['은', '에', '에', ' '],
    ['은', '에', '루', ' '],
];

/// 만들 것 이름 (아이가 지은 이름 그대로)
pub const PICKAXE_NAME: &str = "곡괭이";
pub const CAR_NAME: &str = "자동차";
pub const ENDER_FRUIT_NAME: &str = "애서열매";
pub const DIAMOND_PICKAXE_NAME: &str = "다이아레어라이트 곡괭이";
pub const SYRINGE_GUN_NAME: &str = "주사기총";
/// 만들었을 때 문구. 자동차는 만들면 바로 타서 `CAR_ON_TEXT`가 나온다.
pub const PICKAXE_CRAFTED_TEXT: &str = "곡괭이를 만들었어요!";
pub const ENDER_FRUIT_CRAFTED_TEXT: &str = "애서열매를 만들었어요!";
pub const DIAMOND_PICKAXE_CRAFTED_TEXT: &str = "다이아레어라이트 곡괭이를 만들었어요!";
pub const SYRINGE_GUN_CRAFTED_TEXT: &str = "주사기총을 만들었어요!";
/// 재료가 모자랄 때. {모자란 것}은 "에메랄드가 2개", 여러 개면 "철광석이 1개, 나무가 2개"가 된다.
pub const CRAFT_NEED_MORE_TEXT: &str = "{모자란 것} 더 필요해요";
/// 이미 가진 것 (결과 칸에 나오지 않는다)
pub const CRAFT_ALREADY_HAVE_TEXT: &str = "벌써 가지고 있어요";
/// 임시-39 순서 안내
pub const CRAFT_STEPS_TEXT: &str = "① 만들 것을 골라요 → ② 재료를 골라요 → ③ 흐린 칸을 눌러 채워요 → ④ 결과를 눌러 가져가요";

/// 임시-41 자동차를 타면 걷는 빠르기가 이만큼 곱해진다 (부모님: 3배, 220 → 660). 스케이트와 겹쳐 곱하지 않는다.
pub const CAR_SPEED_MULTIPLIER: f32 = 3.0;
pub const CAR_ON_TEXT: &str = "자동차를 탔어요! 슝!";
pub const CAR_OFF_TEXT: &str = "아쉽다! 아침이 와서 자동차에서 내렸어요";
/// 자동차를 타고 움직이는 동안 떠오르는 글 (아이: 자동차 소리).
pub const CAR_VROOM_TEXT: &str = "부릉!";

/// Esc(PC)·멈춤 버튼(폰)으로 여는 멈춤 창 글 (부모님 요청).
pub const PAUSE_TITLE_TEXT: &str = "잠깐 멈췄어요";
pub const PAUSE_RESUME_TEXT: &str = "이어서 하기";
pub const PAUSE_QUIT_TEXT: &str = "처음으로";
pub const PAUSE_QUIT_SUB_TEXT: &str = "(캐릭터 고르기)";

/// 임시-45 다이아래더라이트 곡괭이로 하나 캐는 시간 (초, 실제 시간)
pub const MINE_SECONDS_DIAMOND_PICKAXE: f32 = 0.5;

/// 임시-42 바다에 쿵 하는 자리: 긴 맵 해변 모래밭, 몸 가운데 x가 이 안이면 된다.
pub const SLAM_BEACH_LEFT: f32 = 40.0;
pub const SLAM_BEACH_RIGHT: f32 = 780.0;
/// 임시-42 폰 쿵 버튼 글. 캐기 버튼 자리에 나온다.
pub const SLAM_BUTTON_TEXT: &str = "쿵!";
pub const SLAM_TEXT: &str = "쿵! 몬스터들을 다 없애버려!";
/// 임시-44, 임시-45 쿵 하면 얻는 다이아몬드 (몬스터가 없어도 얻는다)
pub const ENDER_FRUIT_SLAM_DIAMONDS: u32 = 10;
pub const DIAMOND_PICKAXE_SLAM_DIAMONDS: u32 = 1;
pub const ENDER_FRUIT_SLAM_DIAMONDS_TEXT: &str = "다이아몬드를 10개 얻었어요!";
pub const DIAMOND_PICKAXE_SLAM_DIAMONDS_TEXT: &str = "다이아몬드를 1개 얻었어요!";
pub const DIAMOND_PICKAXE_GONE_TEXT: &str = "곡괭이가 쿵 하고 사라졌어요";
/// 임시-44 엔더열매 먹기
pub const ENDER_FRUIT_EATEN_TEXT: &str = "애서열매를 먹었어요! 해변 바닷가에서 쿵 해 봐요";
pub const ENDER_FRUIT_ALREADY_EATEN_TEXT: &str = "벌써 먹었어요";

/// 부모님 결정 2026-11-15: 쓰는 물건을 쓸 수 없을 때 안내 글 (`CantUse::text`)
pub const USE_HEALTH_FULL_TEXT: &str = "체력이 가득해요";
/// 독약 포션: `POISON_RANGE` 안에 몬스터가 없을 때
pub const USE_NO_MONSTER_NEAR_TEXT: &str = "가까이에 몬스터가 없어요";
/// 주사기총: 화면에 몬스터가 없을 때 (동굴 안 포함)
pub const USE_NO_MONSTER_ON_SCREEN_TEXT: &str = "화면에 몬스터가 없어요";
/// 가진 개수가 0일 때 (문구는 부모님 확정 2026-11-15)
pub const USE_NONE_LEFT_TEXT: &str = "남은 게 없어요";

/// D97 (US-091) 가방에서 곡괭이 칸을 누르면 보이는 쓰는 법. 폰은 E 대신 캐기·쿵! 버튼이라 _TOUCH 글을 쓴다.
pub const PICKAXE_HOW_TEXT: &str = "나무나 광석 가까이에서 E를 누르고 있으면 캐요";
pub const PICKAXE_HOW_TOUCH_TEXT: &str = "나무나 광석 가까이에서 캐기 버튼을 누르고 있으면 캐요";
pub const DIAMOND_PICKAXE_HOW_TEXT: &str = "더 빨리 캐요. 해변 바닷가에서 E를 누르면 쿵!";
pub const DIAMOND_PICKAXE_HOW_TOUCH_TEXT: &str = "더 빨리 캐요. 해변 바닷가에서 쿵! 버튼을 누르면 쿵!";

/// US-091 가방 (REVIEW-022 2번)
pub const BAG_NAME_TEXT: &str = "가방";
/// [임시-76] 화면 아래 키 안내 한 줄이 보이는 실제 초 (끝에 흐려진다)
pub const KEY_HINT_SECONDS: f32 = 8.0;
/// [임시-77] 새 물건을 얻으면 가방 상자가 반짝이는 실제 초
pub const BAG_FLASH_SECONDS: f32 = 2.0;

/// 임시-47, D80 한 번 쓸 때 점수의 "해치운 몬스터 수"에 더하는 수. 쓰러진 몬스터가 0마리면 더하지 않는다.
pub const ENDER_FRUIT_SLAM_SCORE: u32 = 700;
pub const DIAMOND_PICKAXE_SLAM_SCORE: u32 = 600;
pub const SYRINGE_GUN_SCORE: u32 = 500;

/// D87 상자 부수기 (US-089). 도끼를 들었을 때만 공격으로 부순다. 처음 값은 임시다.
/// [임시-63] 이만큼 치면 부서진다 (아이 확정: 3번)
pub const BOX_HITS_TO_BREAK: u32 = 3;
/// [임시-65] [임시-69] 부서질 때 나오는 것: 아이템 15%, 몬스터 15%, 나머지 70%는 아무것도 없음 (부모님 확정, 2026-11-15에 아이템 30%→15%)
pub const BOX_ITEM_CHANCE: f32 = 0.15;
pub const BOX_MONSTER_CHANCE: f32 = 0.15;
/// [임시-65] 아이템 안의 비율 (감독 임시): 총 10%, 아주 센 것(다이아몬드 갑옷, 다이아래더라이트 곡괭이, 엔더열매, 자동차) 각 1%.
/// 나머지 7개(칼, 도끼, 체력 포션, 독약 포션, 스케이트, 무지개 갑옷, 주사기총)가 남은 몫을 똑같이 나눈다.
pub const BOX_ITEM_GUN_SHARE: f32 = 0.10;
pub const BOX_ITEM_VERY_STRONG_SHARE: f32 = 0.01;
/// [임시-69] 몬스터가 나올 때 스켈레톤일 확률 (나머지는 좀비, 감독 임시 반반)
pub const BOX_SKELETON_CHANCE: f32 = 0.5;
/// [임시-69] 상자에서 나온 몬스터가 가만히 있는 시간 (실제 초). 이동안 공격하지 않는다.
pub const BOX_MONSTER_WAIT_SECONDS: f32 = 1.0;
/// [임시-65] 깜짝 아이템 크기, 떨어지는 빠르기, 땅에 닿은 뒤 사라질 때까지 (실제 초)
pub const BOX_ITEM_SIZE: f32 = 32.0;
pub const BOX_ITEM_FALL_SPEED: f32 = 300.0;
pub const BOX_ITEM_GROUND_SECONDS: f32 = 15.0;
/// [임시-68] 도끼가 아닌 것으로 상자를 처음 쳤을 때 한 번 나오는 안내
pub const BOX_NEED_AXE_TEXT: &str = "도끼로 부술 수 있어요";
/// 아주 센 것이 나오면 캐릭터 머리 위에 (아이)
pub const BOX_WOW_TEXT: &str = "와우!";
/// [임시-69] 상자에서 몬스터가 나오면 몬스터 머리 위에 (아이)
pub const BOX_MONSTER_TEXT: &str = "공격!";
/// [임시-65] 깜짝 아이템을 주웠을 때 ("○○을 얻었어요!")
pub const BOX_GOT_SWORD_TEXT: &str = "칼을 얻었어요!";
pub const BOX_GOT_AXE_TEXT: &str = "도끼를 얻었어요!";
pub const BOX_GOT_GUN_TEXT: &str = "총을 얻었어요!";
pub const BOX_GOT_HEALTH_POTION_TEXT: &str = "체력 포션을 얻었어요!";
pub const BOX_GOT_POISON_POTION_TEXT: &str = "독약 포션을 얻었어요!";
pub const BOX_GOT_SKATES_TEXT: &str = "스케이트를 얻었어요!";
pub const BOX_GOT_RAINBOW_ARMOR_TEXT: &str = "무지개 갑옷을 얻었어요!";
pub const BOX_GOT_DIAMOND_ARMOR_TEXT: &str = "다이아몬드 갑옷을 얻었어요!";
pub const BOX_GOT_SYRINGE_GUN_TEXT: &str = "주사기총을 얻었어요!";
pub const BOX_GOT_ENDER_FRUIT_TEXT: &str = "애서열매를 얻었어요!";
pub const BOX_GOT_CAR_TEXT: &str = "자동차를 얻었어요!";
pub const BOX_GOT_DIAMOND_PICKAXE_TEXT: &str = "다이아레어라이트 곡괭이를 얻었어요!";

/// D81, D103 깜짝 미션 (US-084~086). 니로(에이전트)의 임시 값 [임시-49]~[임시-55]. 부모님·아이가 바꿀 수 있다.
/// 미션 수 N = 처음 수 + 하루마다 늘어나는 수 × (날 − 1), 가장 큰 수를 넘지 않는다 ([임시-50]).
/// 종류마다 `_ON`을 false로 하면 그 종류는 나오지 않는다 ([임시-51]).
pub const MISSION_COINS_ON: bool = true;
pub const MISSION_COINS_START: u32 = 10;
pub const MISSION_COINS_PER_DAY: u32 = 2;
pub const MISSION_COINS_MAX: u32 = 20;
pub const MISSION_TREES_ON: bool = true;
pub const MISSION_TREES_START: u32 = 3;
pub const MISSION_TREES_PER_DAY: u32 = 1;
pub const MISSION_TREES_MAX: u32 = 6;
pub const MISSION_ORES_ON: bool = true;
pub const MISSION_ORES_START: u32 = 2;
pub const MISSION_ORES_PER_DAY: u32 = 1;
pub const MISSION_ORES_MAX: u32 = 5;
pub const MISSION_FIND_MISS_ON: bool = true;
pub const MISSION_FIND_MISS_START: u32 = 1;
pub const MISSION_FIND_MISS_PER_DAY: u32 = 0;
pub const MISSION_FIND_MISS_MAX: u32 = 1;
pub const MISSION_MONSTERS_ON: bool = true;
pub const MISSION_MONSTERS_START: u32 = 5;
pub const MISSION_MONSTERS_PER_DAY: u32 = 1;
pub const MISSION_MONSTERS_MAX: u32 = 10;
pub const MISSION_KEEP_HEALTH_ON: bool = true;
pub const MISSION_KEEP_HEALTH_START: u32 = 4;
pub const MISSION_KEEP_HEALTH_PER_DAY: u32 = 0;
pub const MISSION_KEEP_HEALTH_MAX: u32 = 4;
/// [임시-53] 성공하면 받는 코인 (부모님 결정). 지갑은 `COIN_MAX`를 넘지 않지만 점수의 "획득한 동전 수"에는 다 센다 (D103).
pub const MISSION_REWARD_COINS: u32 = 100;
/// [임시-53] 실패하면 줄어드는 체력 (부모님 확정 D103). 갑옷이어도 준다. 7일째 클리어 아침에는 줄이지 않는다 (니로 임시).
pub const MISSION_FAIL_HP_LOSS: u32 = 1;
/// [임시-51] 미션 뽑기는 상자·미스·광석·수학 문제와 따로 시드를 쓴다. 게임 seed에 이 수를 섞는다.
pub const MISSION_SEED_MIX: u32 = 0x5EED_0081;
/// [임시-55] 미스가 알려 주는 말. "{n}"은 그날 미션 수(N)로 바뀐다.
pub const MISSION_COINS_MISS_SAYS: &str = "깜짝 미션이야! 밤이 되기 전에 코인 {n}개를 모아 줘!";
pub const MISSION_TREES_MISS_SAYS: &str = "깜짝 미션이야! 밤이 되기 전에 나무 {n}개를 캐 줘!";
pub const MISSION_ORES_MISS_SAYS: &str = "깜짝 미션이야! 밤이 되기 전에 동굴에서 광석 {n}개를 캐 줘!";
pub const MISSION_FIND_MISS_MISS_SAYS: &str = "깜짝 미션이야! 밤이 되기 전에 나를 찾아 줘!";
pub const MISSION_MONSTERS_MISS_SAYS: &str = "깜짝 미션이야! 아침이 되기 전에 몬스터 {n}마리를 물리쳐 줘!";
pub const MISSION_KEEP_HEALTH_MISS_SAYS: &str = "깜짝 미션이야! 아침까지 체력을 {n} 이상 지켜 줘!";
/// [임시-55] 화면 위쪽 진행 글. "{now}"는 지금 수(체력 지키기는 지금 체력), "{n}"은 미션 수.
pub const MISSION_COINS_PROGRESS: &str = "깜짝 미션! 코인 {now}/{n} (밤 전까지)";
pub const MISSION_TREES_PROGRESS: &str = "깜짝 미션! 나무 {now}/{n} (밤 전까지)";
pub const MISSION_ORES_PROGRESS: &str = "깜짝 미션! 광석 {now}/{n} (밤 전까지)";
pub const MISSION_FIND_MISS_PROGRESS: &str = "깜짝 미션! 미스 찾기 {now}/{n} (밤 전까지)";
pub const MISSION_MONSTERS_PROGRESS: &str = "깜짝 미션! 몬스터 {now}/{n} (아침 전까지)";
pub const MISSION_KEEP_HEALTH_PROGRESS: &str = "깜짝 미션! 체력 {n} 이상 지키기, 지금 {now} (아침까지)";
/// [임시-55] 성공·실패 글. 꾸짖는 말은 쓰지 않는다. 성공 글의 "{reward}"는 `MISSION_REWARD_COINS`로 바뀐다.
pub const MISSION_SUCCESS_TEXT: &str = "잘했어! 코인 {reward}개를 줄게!";
pub const MISSION_FAIL_TEXT: &str = "아쉽다! 다음 미션도 해 보자";
/// [임시-55] 보여 주는 시간 (실제 초). 미스 말풍선과 글이 보이는 동안에도 게임은 멈추지 않는다.
pub const MISSION_MISS_SAYS_SECONDS: f32 = 3.0;
pub const MISSION_SUCCESS_SHOW_SECONDS: f32 = 3.0;
pub const MISSION_FAIL_SHOW_SECONDS: f32 = 2.0;
/// [BUG-034, 부모님 결정 2026-11-15] 18:00에 낮 미션이 실패했으면 실패 글을 먼저 이만큼(실제 초) 보여 주고
/// 그다음 잠깐 퀴즈를 연다. 기다리는 동안 퀴즈가 열린 것처럼 시계·몬스터·주인공이 멈춘다 (BUG-025: 퀴즈보다 몬스터가 먼저 나오지 않게).
pub const QUIZ_WAIT_AFTER_DAY_MISSION_FAIL_SECONDS: f32 = MISSION_FAIL_SHOW_SECONDS;

// ---- 튜토리얼 말풍선 규칙 (US-119·120, 수산나: 숫자와 임시 문구. 그림은 로건) ----
/// ① 이동: 이만큼 걸으면 넘어간다 (픽셀, 임시)
pub const TUTORIAL_WALK_DISTANCE: f32 = 200.0;
/// 한 단계를 해내면 "잘했어요!"를 이만큼 보여 준 뒤 다음 말풍선 (실제 초, 임시)
pub const TUTORIAL_CHEER_SECONDS: f32 = 1.5;
/// ⑤ 미스 찾기: 말만 하고 이만큼 지나면 저절로 사라진다 (실제 초, 임시)
pub const TUTORIAL_FIND_MISS_SECONDS: f32 = 6.0;
/// 말풍선 글 (임시 문구. 캐릭터 Niro가 귀엽게 말한다. 아이와 같이 "이 말 쉬워?" 보고 고친다).
/// 순서: ① 이동 ② 공격 ③ 캐기 ④ 점프 ⑤ 미스 찾기. PC(키보드)와 폰(터치)은 따로 둔다 (D16).
pub const TUTORIAL_TEXT_PC: [&str; 5] = [
    "왼쪽, 오른쪽 화살표로 걸어 봐요~",
    "Q 키로 친구를 톡톡 쳐 봐요~",
    "나무나 돌 옆에서 E 키를 꾹 눌러 봐요~",
    "스페이스 키로 폴짝 뛰어 봐요~",
    "하얀 미스를 찾아 봐요~ 어디 있을까요?",
];
pub const TUTORIAL_TEXT_TOUCH: [&str; 5] = [
    "왼쪽 동그라미를 밀어서 걸어 봐요~",
    "공격 버튼을 눌러 친구를 쳐 봐요~",
    "나무나 돌 옆에서 캐기 버튼을 꾹 눌러 봐요~",
    // 폰에는 점프 버튼이 없다: 왼쪽 동그라미를 위로 밀면 뛴다 (US-063)
    "왼쪽 동그라미를 위로 밀어서 폴짝 뛰어 봐요~",
    "하얀 미스를 찾아 봐요~ 어디 있을까요?",
];
/// 단계를 해냈을 때 한 줄 (임시)
pub const TUTORIAL_CHEER_TEXT: &str = "와, 잘했어요! 짝짝!";
/// 건너뛰기 버튼 글 (임시. 다른 이름은 아이에게 물어볼 카드)
pub const TUTORIAL_SKIP_TEXT: &str = "건너뛰기";

// ---- 창 한 줄 안내 (US-121, 수산나: 임시 문구와 초. 그림은 로건) ----
/// 한 줄 안내가 보이는 초 (실제 초, 임시). 창을 닫으면 그 전에도 사라진다.
pub const WINDOW_TIP_SECONDS: f32 = 5.0;
/// 창 한 줄 안내 글 (임시. 캐릭터 Niro가 귀엽게, 25자 안쪽, 정답은 알려 주지 않는다).
/// 순서: 수학 퀴즈, 영어 퀴즈, 제작대, 상점, 가방, 깜짝 미션 (`TipWindow::index`).
pub const WINDOW_TIP_TEXT_PC: [&str; 6] = [
    "숫자 키로 답을 쓰고 Enter를 눌러요~",
    "그림이나 낱말을 보고 1~4 키로 골라요~",
    "칸을 클릭해서 재료를 놓아 봐요~",
    "코인으로 물건을 살 수 있어요~",
    "가진 물건을 보고 쓸 수 있어요~",
    "미스가 낸 숙제예요~ 위쪽에 진행이 보여요",
];
pub const WINDOW_TIP_TEXT_TOUCH: [&str; 6] = [
    "숫자판을 눌러 답을 넣어요~ 천천히 해요!",
    "그림이나 낱말을 보고 맞는 걸 눌러요~",
    "칸을 눌러서 재료를 놓아 봐요~",
    "코인으로 물건을 살 수 있어요~",
    "가진 물건을 보고 쓸 수 있어요~",
    "미스가 낸 숙제예요~ 위쪽에 진행이 보여요",
];

// ---- 물음표(?) 버튼과 조작 안내 한 장 (US-122, 수산나: 초와 임시 문구. 그림은 로건) ----
/// 물음표 버튼이 보이는 초 (실제 초, 부모님 값 2026-11-21). 플레이를 시작하고 이 시간 동안만 보인다.
pub const HELP_BUTTON_SECONDS: f32 = 30.0;
/// 조작 안내 한 장에 들어가는 줄 수 (넘지 않게)
pub const HELP_LINES_MAX: usize = 8;
/// 조작 안내 (무엇을 하는지, 어떻게 하는지). 지금 게임의 실제 키·버튼이다 (임시 문구).
pub const HELP_LINES_PC: [(&str, &str); 6] = [
    ("걷기", "← → 화살표 키"),
    ("점프", "Space 키 또는 ↑ 키"),
    ("공격", "Q 키"),
    ("캐기", "E 키를 꾹 눌러요"),
    ("가방", "I 키"),
    ("멈춤", "Esc 키"),
];
pub const HELP_LINES_TOUCH: [(&str, &str); 6] = [
    ("걷기", "왼쪽 동그라미를 밀어요"),
    ("점프", "동그라미를 위로 밀어요"),
    ("공격", "오른쪽 아래 공격 버튼"),
    ("캐기", "캐기 버튼을 꾹 눌러요"),
    ("가방", "가방 버튼을 눌러요"),
    ("멈춤", "왼쪽 위 멈춤 버튼"),
];

// ---- 튜토리얼 말풍선 화면 (US-119·120, 로건). 자리는 가상 화면 960×540 기준 ----
/// 말풍선 카드: 가로·세로, 위 끝. 미션 진행 줄(아래 끝 159)·성공/실패 글(아래 끝 165) 바로 아래, 캔 것 팝업(위 끝 228) 바로 위에 들어간다. 가운데 정렬.
/// 폰은 왼쪽 멈춤 버튼(오른쪽 끝 168)을 가로로 비켜서 PC와 같은 높이다.
pub const TUTORIAL_BUBBLE_WIDTH: f32 = 616.0;
pub const TUTORIAL_BUBBLE_HEIGHT: f32 = 62.0;
pub const TUTORIAL_BUBBLE_TOP_PC: f32 = 166.0;
pub const TUTORIAL_BUBBLE_TOP_TOUCH: f32 = 166.0;
/// 큰 알림 글이 말풍선 아래로 비켜 쌓이도록 말풍선 아래에 두는 틈
pub const TUTORIAL_BUBBLE_GAP_BELOW: f32 = 6.0;
/// 글자 크기와 줄 간격(글자 크기에 대한 비율). 폰 667×375에서 16 화면 픽셀 안팎
pub const TUTORIAL_TEXT_SIZE: f32 = 24.0;
pub const TUTORIAL_LINE_STEP: f32 = 1.1;
/// 말풍선 안 여백, Niro 얼굴 동그라미 반지름, 얼굴과 풍선 틈, 풍선 모서리
pub const TUTORIAL_PAD: f32 = 12.0;
pub const TUTORIAL_FACE_RADIUS: f32 = 28.0;
pub const TUTORIAL_FACE_GAP: f32 = 10.0;
pub const TUTORIAL_CORNER: f32 = 14.0;
/// 얼굴이 살랑 위아래로 움직이는 폭(픽셀)과 빠르기
pub const TUTORIAL_FACE_BOB: f32 = 1.5;
pub const TUTORIAL_FACE_BOB_SPEED: f32 = 4.0;
/// 건너뛰기 버튼 크기 (폰 667 폭에서도 가로 48 화면 픽셀 이상)
pub const TUTORIAL_SKIP_WIDTH: f32 = 132.0;
pub const TUTORIAL_SKIP_HEIGHT: f32 = 56.0;
pub const TUTORIAL_SKIP_TEXT_SIZE: f32 = 26.0;
/// 색 (r, g, b, a)
pub const TUTORIAL_BUBBLE_COLOR: [f32; 4] = [0.13, 0.16, 0.36, 0.88];
pub const TUTORIAL_TEXT_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const TUTORIAL_CHEER_COLOR: [f32; 4] = [1.0, 0.85, 0.25, 1.0];
pub const TUTORIAL_FACE_BACK_COLOR: [f32; 4] = [0.68, 0.85, 1.0, 1.0];
/// Niro 얼굴 둘레 (Niro 노란 상의)
pub const TUTORIAL_FACE_RING_COLOR: [f32; 4] = [1.0, 0.82, 0.20, 1.0];
pub const TUTORIAL_SKIP_FILL_COLOR: [f32; 4] = [1.0, 0.82, 0.25, 1.0];
pub const TUTORIAL_SKIP_TEXT_COLOR: [f32; 4] = [0.13, 0.16, 0.36, 1.0];

// ---- 창 한 줄 안내 그림 (US-121, 로건). 자리는 가상 화면 960×540 기준 ----
/// 한 줄 카드: 글자 크기, 높이, 글 양옆 여백, 왼쪽 별 그림 폭
pub const WINDOW_TIP_TEXT_SIZE: f32 = 21.0;
pub const WINDOW_TIP_HEIGHT: f32 = 34.0;
pub const WINDOW_TIP_PAD: f32 = 16.0;
pub const WINDOW_TIP_ICON_W: f32 = 30.0;
/// 창마다 카드 자리 (가운데 x, 위 y, 가장 넓을 때 폭). 순서는 `TipWindow::index`: 수학, 영어, 제작대, 상점, 가방, 깜짝 미션.
/// 창 글자·칸·버튼이 없는 빈 곳에 놓는다 (창 모양이 바뀌면 `view::window_tip`의 시험이 겹침을 잡는다).
pub const WINDOW_TIP_SPOTS: [(f32, f32, f32); 6] = [
    (640.0, 495.0, 600.0),
    (565.0, 16.0, 640.0),
    (632.0, 422.0, 576.0),
    (480.0, 482.0, 700.0),
    (520.0, 26.0, 440.0),
    (480.0, 236.0, 616.0),
];

// ---- 물음표(?) 버튼과 조작 안내 한 장 그림 (US-122, 로건). 자리는 가상 화면 960×540 기준 ----
/// 물음표 버튼 (왼쪽 위 얼굴·물건 칸 아래). PC는 물건 칸(아래 끝 117) 바로 아래, 폰은 "멈춤" 버튼(아래 끝 178) 바로 아래.
/// 폰 667×375에서 지름 56 화면 픽셀 (손가락으로 누르기 쉽게). 부모님이 고를 시안 값.
pub const HELP_BUTTON_LEFT: f32 = 14.0;
pub const HELP_BUTTON_TOP_PC: f32 = 126.0;
pub const HELP_BUTTON_TOP_TOUCH: f32 = 188.0;
pub const HELP_BUTTON_SIZE_PC: f32 = 64.0;
pub const HELP_BUTTON_SIZE_TOUCH: f32 = 80.0;
pub const HELP_BUTTON_COLOR: [f32; 4] = [1.0, 0.80, 0.22, 1.0];
pub const HELP_BUTTON_MARK_COLOR: [f32; 4] = [0.13, 0.16, 0.36, 1.0];
/// 조작 안내 한 장: 카드 자리, 제목, 닫기 버튼
pub const HELP_SHEET: [f32; 4] = [70.0, 22.0, 820.0, 496.0];
pub const HELP_TITLE_TEXT: &str = "이렇게 해요!";
pub const HELP_CLOSE_TEXT: &str = "알았어요!";
pub const HELP_CLOSE: [f32; 4] = [340.0, 436.0, 280.0, 70.0];

// ---- 연습용 몬스터 그림 (US-124, 로건). 이름·생김새·색은 아이 질문 카드라서 임시 모양 ----
/// 밤 좀비 그림에 곱하는 부드러운 민트색 (밤 몬스터와 한눈에 다르게)
pub const PRACTICE_TINT: [f32; 4] = [0.72, 1.0, 0.86, 1.0];
/// 머리 위 별 표시: 머리 위로 띄우는 높이, 반지름, 위아래 움직임 폭, 돌아가는 빠르기, 색
pub const PRACTICE_MARK_ABOVE: f32 = 12.0;
pub const PRACTICE_MARK_RADIUS: f32 = 9.0;
pub const PRACTICE_MARK_BOB: f32 = 3.0;
pub const PRACTICE_MARK_TURN_SPEED: f32 = 1.5;
pub const PRACTICE_MARK_COLOR: [f32; 4] = [1.0, 0.86, 0.30, 1.0];

// ---- 연습용 몬스터 (US-124, 수산나). 이름·생김새·색은 아이가 정한다: 지금은 "연습용" 임시 하나 ----
/// 연습용 몬스터를 맞춰서 쓰러뜨릴 때까지 맞는 횟수 (무기와 상관없이 한 번에 1, 임시)
pub const PRACTICE_MONSTER_HP: u32 = 2;
/// 한 번 맞을 때 주는 피해 (무기 힘과 상관없이 같다)
pub const PRACTICE_MONSTER_HIT_DAMAGE: u32 = 1;
/// 안 때리고 두면 저절로 사라지는 시간 (게임 속 시간 아닌 실제 초, 임시)
pub const PRACTICE_MONSTER_LIFE_SECONDS: f32 = 60.0;
/// 걷는 빠르기 (밤 좀비보다 느리다)
pub const PRACTICE_MONSTER_WALK_SPEED: f32 = 40.0;
/// 플레이어와 몸 가운데끼리 이만큼 가까워지면 더 안 따라온다 (공격이 닿는 거리 안쪽)
pub const PRACTICE_MONSTER_STOP_DISTANCE: f32 = 70.0;
/// 말풍선 ②가 시작될 때 플레이어 바라보는 쪽으로 이만큼 떨어져 나온다 (화면 안, 폭 960 안쪽)
pub const PRACTICE_MONSTER_SPAWN_DISTANCE: f32 = 220.0;
pub const PRACTICE_MONSTER_WIDTH: f32 = 40.0;
pub const PRACTICE_MONSTER_HEIGHT: f32 = 48.0;
/// 맞았을 때 반짝이는 시간
pub const PRACTICE_MONSTER_FLASH_SECONDS: f32 = ZOMBIE_HIT_FLASH_SECONDS;

/// 연습용 몬스터가 한 프레임에 나누어 움직이는 최대 횟수 (아주 긴 프레임에서도 끝나게)
pub const PRACTICE_STEP_LIMIT: u32 = 200;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decided_values_match_plan() {
        assert_eq!(DAY_SECONDS, 300.0);
        assert_eq!(NIGHT_SECONDS, 600.0);
        assert_eq!(PLAYER_START_HP, 7);
        assert_eq!(PLAYER_HP_PER_LEVEL, 5);
        assert_eq!(ZOMBIE_HP, 3);
        assert_eq!(ZOMBIE_ATTACK, 2);
        assert_eq!(EXPERIENCE_PER_LEVEL, 5);
        assert_eq!(SKELETON_HP, 3);
        assert_eq!((ZOMBIE_SPAWN_SECONDS, MONSTERS_PER_SPAWN, MONSTER_MAX_ON_SCREEN, ZOMBIE_MAX_ON_SCREEN), (10.0, 2, 6, 4));
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
        assert_eq!(MAP_WIDTH, 3840.0);
        assert_eq!((HANDS_ATTACK, SWORD_ATTACK, AXE_ATTACK, GUN_ATTACK), (1, 2, 3, 4));
        assert_eq!((CITIZEN_BLAST_DAMAGE, CITIZEN_RESPAWN_SECONDS), (2, 30.0));
        // 낮은 실제 1초 = 게임 2.4분, 밤은 D14대로 1.2분
        assert_eq!(DAY_GAME_MINUTES as f32 / DAY_SECONDS, 2.4);
        assert_eq!(NIGHT_GAME_MINUTES as f32 / NIGHT_SECONDS, 1.2);
    }
}

// ---- 로고·시작 화면 (US-109, 로건: 화면 전용 시간 값. 그림 값은 view/logo.rs) ----
/// 로고 전체 길이 (초, [임시-105] 5~6초)
pub const LOGO_SECONDS: f32 = 5.8;
/// 1·2·3이 끝나는 때 (숫자 하나에 1/3씩)
pub const LOGO_COUNT_END: f32 = 2.5;
/// "Hey"가 끝나고 "Lucky Mis!"가 나오는 때
pub const LOGO_HEY_END: f32 = 3.5;
/// 글자가 통통 튀어나오는 시간
pub const LOGO_POP_SECONDS: f32 = 0.35;
/// 글자가 작아지며 사라지는 시간 (다음 글자 바로 전)
pub const LOGO_OUT_SECONDS: f32 = 0.18;
/// 미스가 아래에서 올라오는 때 (시작, 끝)
pub const LOGO_MIS_RISE: (f32, f32) = (3.65, 4.25);
/// 미스 눈 깜빡이는 때 (두 번)과 길이
pub const LOGO_BLINKS: [f32; 2] = [4.5, 4.85];
pub const LOGO_BLINK_SECONDS: f32 = 0.12;
/// 미스가 살짝 뿅 뛰는 때와 길이
pub const LOGO_HOP_START: f32 = 5.1;
pub const LOGO_HOP_SECONDS: f32 = 0.35;
/// "NIRO STUDIO"가 나타나는 때 (시작, 끝)
pub const LOGO_STUDIO_FADE: (f32, f32) = (4.0, 4.4);
/// "Now Loading..." 도는 점이 한 바퀴 도는 시간 (web/index.html의 CSS 1s와 같아야 한다)
pub const LOADING_SPINNER_TURN_SECONDS: f32 = 1.0;
/// "눌러서 시작" 글자가 천천히 깜빡이는 한 번 (진해졌다 흐려졌다 다시 진해지기까지)
pub const TAP_TO_START_BLINK_SECONDS: f32 = 1.6;

// ---- 설정 저장 (US-125, 수산나: 값 모양과 이름. 브라우저 읽기·쓰기는 로건) ----
/// 브라우저 저장소에 쓰는 키 이름 (새 틀 하나. 옛 best-days·best-score 키는 따로 그대로)
pub const SETTINGS_STORAGE_KEY: &str = "miss-game.settings";
/// 저장 글자의 모양 버전 (맨 앞에 쓴다. 모양이 바뀌면 올린다)
pub const SETTINGS_VERSION: u32 = 1;
/// 저장 글자가 이보다 길면 이상한 값으로 보고 새 게임 값으로 읽는다
pub const SETTINGS_MAX_CHARS: usize = 200;
/// 항목 사이를 가르는 글자와, 이름과 값 사이를 가르는 글자
pub const SETTINGS_ITEM_SEPARATOR: char = ';';
pub const SETTINGS_VALUE_SEPARATOR: char = '=';
/// 항목 이름 (글자에 쓰는 이름)
pub const SETTINGS_KEY_TUTORIAL_DONE: &str = "tutorial";
/// 창 한 줄 안내를 본 창들 (여섯 칸 숫자 0~63). 옛 `tip=`는 쓰지 않고, 있어도 읽을 때 무시한다.
pub const SETTINGS_KEY_WINDOW_TIPS_SEEN: &str = "tips";
pub const SETTINGS_KEY_SOUND_ON: &str = "sound";
pub const SETTINGS_KEY_MUSIC_PERCENT: &str = "music";
pub const SETTINGS_KEY_EFFECTS_PERCENT: &str = "effects";
/// 소리 크기는 0~100 정수(퍼센트)로 저장한다
pub const SETTINGS_VOLUME_MAX_PERCENT: u8 = 100;
/// 처음 값 (새 게임)
pub const SETTINGS_DEFAULT_SOUND_ON: bool = true;
pub const SETTINGS_DEFAULT_MUSIC_PERCENT: u8 = 70;
pub const SETTINGS_DEFAULT_EFFECTS_PERCENT: u8 = 80;

// ---- 소리 규칙 (US-112~117, 수산나: 이름과 수치. 소리 내기는 로건) ----
/// 소리 파일 폴더 (`game/assets/` 아래)
pub const SOUNDS_SFX_DIR: &str = "sounds/sfx";
pub const SOUNDS_MUSIC_DIR: &str = "sounds/music";
/// 소리 파일을 모두 합친 크기 한도 (바이트, 5MB). 시험이 폴더 크기를 잰다.
pub const SOUNDS_TOTAL_MAX_BYTES: u64 = 5 * 1024 * 1024;
/// 같은 효과음은 이 시간(초) 안에 또 나지 않는다 (임시-117)
pub const SFX_MIN_GAP_SECONDS: f32 = 0.05;
/// 효과음이 한꺼번에 나는 수 한도: `SFX_BURST_WINDOW_SECONDS` 안에 시작한 소리가 이 수를 넘지 않는다
pub const SFX_MAX_AT_ONCE: usize = 4;
pub const SFX_BURST_WINDOW_SECONDS: f32 = 0.25;
/// 중요한 소리(캐는 "톡톡" 포함)는 한도가 꽉 차도 이 수만큼 더 낼 수 있다 (밤에 몬스터 소리가 몰려도 안 묻히게)
pub const SFX_IMPORTANT_EXTRA_SLOTS: usize = 2;
/// 한 프레임에 모아 두는 효과음 요청 수 한도 (넘으면 버린다)
pub const SFX_PENDING_LIMIT: usize = 16;
/// 캐는 동안 "톡톡" 소리 사이 시간 (초)
pub const MINING_TICK_SECONDS: f32 = 0.35;
/// 소리 크기 4칸: 끔·작게·보통·크게 (퍼센트)
pub const SOUND_VOLUME_STEPS: [u8; 4] = [0, 25, 50, 80];
/// 멈춤 창이 열려 있을 때 음악 크기에 곱하는 수 ("작게 계속", 임시-115)
pub const SOUND_PAUSED_MUSIC_FACTOR: f32 = 0.4;
/// 음악이 바뀔 때 앞 곡이 작아지고 새 곡이 커지는 시간 (초, 임시-116). 실제 섞기는 로건.
pub const MUSIC_FADE_SECONDS: f32 = 2.0;

/// 음악 파일 이름 (`SOUNDS_MUSIC_DIR` 아래). 밤과 동굴은 같은 곡이다.
pub const MUSIC_TITLE_FILE: &str = "title_children_march.ogg";
pub const MUSIC_DAY_FILE: &str = "day_overworld.ogg";
pub const MUSIC_NIGHT_FILE: &str = "night_musicbox.ogg";
pub const MUSIC_CAVE_FILE: &str = MUSIC_NIGHT_FILE;
pub const MUSIC_SHOP_FILE: &str = "shop_urban.ogg";

/// 아이가 고르는 소리 (파일 이름들 중 `*_CHOICE` 번째를 쓴다. 바꾸려면 번호만 고친다)
/// 몬스터 쓰러짐: 0 뿅, 1 퐁, 2 띠용
pub const MONSTER_DEFEATED_FILES: [&str; 3] = ["monster_down_pop.wav", "monster_down_bloop.wav", "monster_down_boing.wav"];
pub const MONSTER_DEFEATED_CHOICE: usize = 0;
/// 레벨업: 0 빰빠밤 나팔, 1 반짝 별, 2 뿅뿅 게임기
pub const LEVEL_UP_FILES: [&str; 3] = ["level_up_fanfare.wav", "level_up_star.wav", "level_up_chip.wav"];
pub const LEVEL_UP_CHOICE: usize = 0;
/// 내가 쓰러짐: 0 웃긴 "띠로리", 1 조용한 소리
pub const PLAYER_DOWN_FILES: [&str; 2] = ["fall_down_funny.wav", "wrong.wav"];
pub const PLAYER_DOWN_CHOICE: usize = 0;

// ---- 소리 화면 (US-112, 로건: 글과 크기. 소리 규칙 수치는 위 "소리 규칙") ----
/// M 키를 누르면 잠깐 보이는 글과 시간 (초)
pub const SOUND_TOAST_ON_TEXT: &str = "소리를 켰어요";
pub const SOUND_TOAST_OFF_TEXT: &str = "소리를 껐어요";
pub const SOUND_TOAST_SECONDS: f32 = 1.0;
/// 글이 뜨는 자리: 화면 아래 오른쪽 (손에 든 칸 오른쪽의 빈 모래밭). 위쪽 정보판·제목·로고·캐릭터와 안 겹친다.
/// 멈춤 창이 열려 있으면 글은 안 띄운다 (창 안의 켜짐/꺼짐 버튼이 바뀌어 보인다).
pub const SOUND_TOAST_CENTER_X: f32 = 750.0;
pub const SOUND_TOAST_Y: f32 = 468.0;
pub const SOUND_TOAST_H: f32 = 56.0;
/// 멈춤 창 소리 줄 글
pub const PAUSE_SOUND_ON_TEXT: &str = "소리 켜짐";
pub const PAUSE_SOUND_OFF_TEXT: &str = "소리 꺼짐";
pub const PAUSE_MUSIC_LABEL: &str = "음악";
pub const PAUSE_EFFECTS_LABEL: &str = "효과음";
/// 크기 4칸 이름 (`SOUND_VOLUME_STEPS` 순서)
pub const SOUND_STEP_LABELS: [&str; 4] = ["끔", "작게", "보통", "크게"];
/// 소리 파일 하나를 받는 데, 그리고 푸는 데 각각 기다리는 최대 시간 (초). 넘으면 그 소리는 건너뛴다.
pub const SOUND_LOAD_TIMEOUT_SECONDS: f64 = 15.0;
