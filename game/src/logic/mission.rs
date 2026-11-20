//! 깜짝 미션 (D81, D103, US-084~086). 미스가 아침에 낮 미션 하나, 밤에 밤 미션 하나를 알려 준다.
//!
//! - 낮 미션: 06:00에 나와서 18:00에 끝난다. 새 게임을 시작할 때도 1일째 낮 미션이 바로 나온다.
//! - 밤 미션: 18:00 잠깐 퀴즈가 닫힌 뒤 나와서 06:00에 끝난다.
//! - 미션이 나온 뒤에 한 것만 센다. 수가 N이 되는 순간 바로 성공이다. 체력 지키기만 06:00에 본다.
//! - 뽑기는 따로 시드를 써서 상자·미스·광석·수학 문제 뽑기가 바뀌지 않는다.
//!
//! 이 파일은 미션의 수와 상태만 들고 있다. 코인 주기와 체력 줄이기는 `Game`이 한다.

use super::monster::SeededRandom;
use crate::config::*;

/// 미션 종류 ([임시-50] 미션 표).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionKind {
    /// 낮: 상자 위 코인 줍기.
    Coins,
    /// 낮: 나무 다 캐기.
    Trees,
    /// 낮: 동굴에서 광석 다 캐기.
    Ores,
    /// 낮: 미스 찾기.
    FindMiss,
    /// 밤: 경험치를 주는 몬스터 물리치기.
    Monsters,
    /// 밤: 06:00에 체력이 N 이상이면 성공.
    KeepHealth,
}

impl MissionKind {
    /// 낮 미션 종류 (뽑는 차례).
    pub const DAY_KINDS: [MissionKind; 4] = [MissionKind::Coins, MissionKind::Trees, MissionKind::Ores, MissionKind::FindMiss];
    /// 밤 미션 종류 (뽑는 차례).
    pub const NIGHT_KINDS: [MissionKind; 2] = [MissionKind::Monsters, MissionKind::KeepHealth];

    /// 낮 미션이면 true, 밤 미션이면 false.
    pub fn is_day_mission(self) -> bool {
        !matches!(self, MissionKind::Monsters | MissionKind::KeepHealth)
    }

    /// `config.rs`에서 켜 두었는지.
    pub fn is_turned_on_in_config(self) -> bool {
        match self {
            MissionKind::Coins => MISSION_COINS_ON,
            MissionKind::Trees => MISSION_TREES_ON,
            MissionKind::Ores => MISSION_ORES_ON,
            MissionKind::FindMiss => MISSION_FIND_MISS_ON,
            MissionKind::Monsters => MISSION_MONSTERS_ON,
            MissionKind::KeepHealth => MISSION_KEEP_HEALTH_ON,
        }
    }

    /// (처음 수, 하루마다 늘어나는 수, 가장 큰 수).
    fn numbers(self) -> (u32, u32, u32) {
        match self {
            MissionKind::Coins => (MISSION_COINS_START, MISSION_COINS_PER_DAY, MISSION_COINS_MAX),
            MissionKind::Trees => (MISSION_TREES_START, MISSION_TREES_PER_DAY, MISSION_TREES_MAX),
            MissionKind::Ores => (MISSION_ORES_START, MISSION_ORES_PER_DAY, MISSION_ORES_MAX),
            MissionKind::FindMiss => (MISSION_FIND_MISS_START, MISSION_FIND_MISS_PER_DAY, MISSION_FIND_MISS_MAX),
            MissionKind::Monsters => (MISSION_MONSTERS_START, MISSION_MONSTERS_PER_DAY, MISSION_MONSTERS_MAX),
            MissionKind::KeepHealth => (MISSION_KEEP_HEALTH_START, MISSION_KEEP_HEALTH_PER_DAY, MISSION_KEEP_HEALTH_MAX),
        }
    }

    /// `day`일째 미션 수 N = 처음 수 + 늘어나는 수 × (날 − 1), 가장 큰 수를 넘지 않는다 ([임시-50]).
    pub fn goal_on_day(self, day: u32) -> u32 {
        let (start, per_day, max) = self.numbers();
        (start + per_day * day.saturating_sub(1)).min(max)
    }

    /// 미스가 알려 주는 말 (예: "깜짝 미션이야! 밤이 되기 전에 코인 12개를 모아 줘!").
    pub fn miss_says_text(self, goal: u32) -> String {
        let text = match self {
            MissionKind::Coins => MISSION_COINS_MISS_SAYS,
            MissionKind::Trees => MISSION_TREES_MISS_SAYS,
            MissionKind::Ores => MISSION_ORES_MISS_SAYS,
            MissionKind::FindMiss => MISSION_FIND_MISS_MISS_SAYS,
            MissionKind::Monsters => MISSION_MONSTERS_MISS_SAYS,
            MissionKind::KeepHealth => MISSION_KEEP_HEALTH_MISS_SAYS,
        };
        text.replace("{n}", &goal.to_string())
    }

    /// 화면 위쪽 진행 글 (예: "깜짝 미션! 코인 3/10 (밤 전까지)"). 체력 지키기는 `now`가 지금 체력이다.
    pub fn progress_text(self, now: u32, goal: u32) -> String {
        let text = match self {
            MissionKind::Coins => MISSION_COINS_PROGRESS,
            MissionKind::Trees => MISSION_TREES_PROGRESS,
            MissionKind::Ores => MISSION_ORES_PROGRESS,
            MissionKind::FindMiss => MISSION_FIND_MISS_PROGRESS,
            MissionKind::Monsters => MISSION_MONSTERS_PROGRESS,
            MissionKind::KeepHealth => MISSION_KEEP_HEALTH_PROGRESS,
        };
        text.replace("{now}", &now.to_string()).replace("{n}", &goal.to_string())
    }
}

/// 성공 글: "잘했어! 코인 100개를 줄게!".
pub fn mission_success_text() -> String {
    MISSION_SUCCESS_TEXT.replace("{reward}", &MISSION_REWARD_COINS.to_string())
}

/// 실패 글: "아쉽다! 다음 미션도 해 보자".
pub fn mission_fail_text() -> &'static str {
    MISSION_FAIL_TEXT
}

/// 미션이 어떻게 되었는지.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionState {
    /// 하는 중 (진행 글이 보인다).
    Going,
    Succeeded,
    Failed,
}

/// 미션 하나.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mission {
    pub kind: MissionKind,
    /// 미션 수 N.
    pub goal: u32,
    /// 나온 뒤 한 수. 체력 지키기는 쓰지 않는다 (지금 체력은 `Game::mission_now`가 채운다).
    pub count: u32,
    pub state: MissionState,
    /// 몇째 날의 미션인지 (밤 미션은 그 밤이 시작된 날).
    pub day: u32,
}

impl Mission {
    /// 진행 글. 체력 지키기는 `count` 자리에 지금 체력이 들어간 미션으로 부른다 (`Game::mission_now`).
    pub fn progress_text(&self) -> String {
        self.kind.progress_text(self.count, self.goal)
    }
}

/// 화면(로건)이 보고 미스 말풍선·성공 글·실패 글을 띄울 일. `Game::take_mission_events`로 받는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MissionEvent {
    /// 미션이 나왔다: 미스 얼굴과 말풍선 `MISSION_MISS_SAYS_SECONDS`초 (`MissionKind::miss_says_text(goal)`).
    Appeared { kind: MissionKind, goal: u32 },
    /// 성공했다: `mission_success_text()` `MISSION_SUCCESS_SHOW_SECONDS`초와 반짝이. 지갑에 코인이 이미 들어갔다.
    Succeeded { kind: MissionKind },
    /// 실패했다: `mission_fail_text()` `MISSION_FAIL_SHOW_SECONDS`초. 체력이 줄었으면 `GameEvent::Combat(PlayerHurt)`도 온다.
    Failed { kind: MissionKind },
}

/// 한 판의 미션 뽑기와 지금 미션.
pub struct Missions {
    random: SeededRandom,
    /// 뽑을 수 있는 종류. 처음에는 `config.rs`에서 켠 것이다. 테스트는 빼서 "끄기"를 본다.
    pub turned_on: Vec<MissionKind>,
    /// 지금 미션 (끝난 미션도 다음 미션이 나올 때까지 남아 있다).
    current: Option<Mission>,
    last_day_kind: Option<MissionKind>,
    last_night_kind: Option<MissionKind>,
    /// 밤 미션을 이미 낸 날 (그 밤이 시작된 날). 0이면 아직 없다.
    night_given_on_day: u32,
    events: Vec<MissionEvent>,
}

impl Missions {
    /// `seed`가 같으면 같은 미션이 같은 순서로 나온다. 다른 뽑기와 따로 섞는다 (`MISSION_SEED_MIX`).
    pub fn new(seed: u32) -> Missions {
        Missions {
            random: SeededRandom::new(seed ^ MISSION_SEED_MIX),
            turned_on: MissionKind::DAY_KINDS
                .into_iter()
                .chain(MissionKind::NIGHT_KINDS)
                .filter(|kind| kind.is_turned_on_in_config())
                .collect(),
            current: None,
            last_day_kind: None,
            last_night_kind: None,
            night_given_on_day: 0,
            events: Vec::new(),
        }
    }

    /// 미션을 모두 끈다: 지금 미션도 없어지고 새 미션도 나오지 않는다.
    /// 미션과 상관없는 규칙(밤 문제, 점수 등)을 따로 보는 시험에서 쓴다.
    #[cfg(test)] // 시험에서만 쓴다. 게임 빌드에는 들어가지 않는다
    pub fn turn_off_all(&mut self) {
        self.turned_on.clear();
        self.current = None;
        self.events.clear();
    }

    /// 지금 미션 (끝난 것 포함). 없으면 None.
    pub fn current(&self) -> Option<&Mission> {
        self.current.as_ref()
    }

    /// 지금 하는 중인 미션의 종류. 없거나 끝났으면 None.
    pub fn going_kind(&self) -> Option<MissionKind> {
        self.current.filter(|m| m.state == MissionState::Going).map(|m| m.kind)
    }

    /// `day`일째 낮 미션을 낸다. 켠 낮 종류가 없으면 아무 일도 없다.
    pub fn give_day_mission(&mut self, day: u32) {
        let last = self.last_day_kind;
        if let Some(kind) = self.pick(true, last) {
            self.last_day_kind = Some(kind);
            self.start(kind, day);
        }
    }

    /// `day`일째 밤에 밤 미션을 아직 내지 않았으면 true.
    pub fn night_mission_is_waiting(&self, day: u32) -> bool {
        self.night_given_on_day != day
    }

    /// `day`일째 밤 미션을 낸다. 같은 밤에 두 번 내지 않는다.
    pub fn give_night_mission(&mut self, day: u32) {
        if !self.night_mission_is_waiting(day) {
            return;
        }
        self.night_given_on_day = day;
        let last = self.last_night_kind;
        if let Some(kind) = self.pick(false, last) {
            self.last_night_kind = Some(kind);
            self.start(kind, day);
        }
    }

    /// 미션 하나를 새로 시작한다. 앞 미션은 이미 끝났다.
    fn start(&mut self, kind: MissionKind, day: u32) {
        let goal = kind.goal_on_day(day);
        self.current = Some(Mission { kind, goal, count: 0, state: MissionState::Going, day });
        self.events.push(MissionEvent::Appeared { kind, goal });
    }

    /// 낮(또는 밤) 종류 중 똑같은 확률로 하나. 어제 같은 때의 종류(`last`)는 빼되, 그것밖에 없으면 낸다.
    fn pick(&mut self, day_time: bool, last: Option<MissionKind>) -> Option<MissionKind> {
        let all: Vec<MissionKind> = self.turned_on.iter().copied().filter(|k| k.is_day_mission() == day_time).collect();
        let not_yesterday: Vec<MissionKind> = all.iter().copied().filter(|k| Some(*k) != last).collect();
        let choices = if not_yesterday.is_empty() { all } else { not_yesterday };
        if choices.is_empty() {
            return None;
        }
        let index = ((self.random.next_unit() * choices.len() as f32) as usize).min(choices.len() - 1);
        Some(choices[index])
    }

    /// `kind`를 `amount`만큼 했다. 그 종류 미션을 하는 중일 때만 센다.
    /// N이 되는 순간 성공하고 true를 돌려준다 (코인은 `Game`이 준다). 성공한 뒤에는 더 세지 않는다.
    pub fn add(&mut self, kind: MissionKind, amount: u32) -> bool {
        let Some(mission) = self.current.as_mut() else {
            return false;
        };
        if amount == 0 || mission.kind != kind || mission.state != MissionState::Going || kind == MissionKind::KeepHealth {
            return false;
        }
        mission.count = (mission.count + amount).min(mission.goal);
        if mission.count < mission.goal {
            return false;
        }
        mission.state = MissionState::Succeeded;
        self.events.push(MissionEvent::Succeeded { kind });
        true
    }

    /// 18:00: 하는 중인 낮 미션이 실패로 끝난다. 실패했으면 true (체력은 `Game`이 줄인다).
    pub fn end_day_mission(&mut self) -> bool {
        self.fail_going_mission(true)
    }

    /// 06:00: 하는 중인 밤 미션이 끝난다. 체력 지키기는 `hp_before_refill`이 N 이상이면 성공이다.
    /// 돌려주는 값: Some(true) 성공, Some(false) 실패, None 끝낼 미션이 없었다.
    pub fn end_night_mission(&mut self, hp_before_refill: u32) -> Option<bool> {
        let going = self.current.filter(|m| m.state == MissionState::Going && !m.kind.is_day_mission())?;
        if going.kind == MissionKind::KeepHealth && hp_before_refill >= going.goal {
            let mission = self.current.as_mut()?;
            mission.state = MissionState::Succeeded;
            self.events.push(MissionEvent::Succeeded { kind: going.kind });
            return Some(true);
        }
        self.fail_going_mission(false);
        Some(false)
    }

    /// 하는 중인 낮(또는 밤) 미션을 실패로 끝낸다. 실패했으면 true.
    fn fail_going_mission(&mut self, day_time: bool) -> bool {
        let Some(mission) = self.current.as_mut() else {
            return false;
        };
        if mission.state != MissionState::Going || mission.kind.is_day_mission() != day_time {
            return false;
        }
        mission.state = MissionState::Failed;
        self.events.push(MissionEvent::Failed { kind: mission.kind });
        true
    }

    /// 지난번 뒤로 생긴 미션 일을 모두 받는다.
    pub fn take_events(&mut self) -> Vec<MissionEvent> {
        std::mem::take(&mut self.events)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn goal_grows_each_day_and_stops_at_max() {
        assert_eq!(MissionKind::Coins.goal_on_day(1), MISSION_COINS_START);
        assert_eq!(MissionKind::Coins.goal_on_day(2), MISSION_COINS_START + MISSION_COINS_PER_DAY);
        assert_eq!(MissionKind::Coins.goal_on_day(100), MISSION_COINS_MAX);
    }

    #[test]
    fn texts_get_numbers() {
        assert_eq!(MissionKind::Coins.miss_says_text(12), "깜짝 미션이야! 밤이 되기 전에 코인 12개를 모아 줘!");
        assert_eq!(MissionKind::Monsters.progress_text(2, 5), "깜짝 미션! 몬스터 2/5 (아침 전까지)");
        assert_eq!(MissionKind::KeepHealth.progress_text(6, 4), "깜짝 미션! 체력 4 이상 지키기, 지금 6 (아침까지)");
        assert_eq!(mission_success_text(), "잘했어! 코인 100개를 줄게!");
    }

    #[test]
    fn turn_off_all_removes_mission_and_stops_new_ones() {
        let mut missions = Missions::new(1);
        missions.turned_on = vec![MissionKind::Coins];
        missions.give_day_mission(1);
        assert!(missions.current().is_some());
        missions.turn_off_all();
        assert!(missions.current().is_none());
        assert!(missions.take_events().is_empty());
        missions.give_day_mission(2);
        assert!(missions.current().is_none());
    }

    #[test]
    fn counting_other_kind_does_nothing() {
        let mut missions = Missions::new(1);
        missions.turned_on = vec![MissionKind::Coins];
        missions.give_day_mission(1);
        assert!(!missions.add(MissionKind::Trees, 5));
        assert_eq!(missions.current().unwrap().count, 0);
    }
}
