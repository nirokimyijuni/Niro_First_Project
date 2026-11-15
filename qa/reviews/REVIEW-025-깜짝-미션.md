# REVIEW-025 깜짝 미션 (US-084·085·086, D81·D103, 임시-49~55)

- 리뷰어: 밥 / 날짜: 2026-11-15 / 대상: 커밋 전 작업 트리 (563717f 다음)
- 기준: docs/02-plan.md D81·D103, stories/backlog.md 임시-49~55와 US-084~086 완료 조건

## 이 변경이 하는 일
미스가 하루에 낮 미션 하나(06:00~18:00), 밤 미션 하나(18:00 퀴즈가 닫힌 뒤~06:00)를 알려 줘요. 수가 N이 되는 순간 성공해서 코인 100을 받고(점수의 "획득한 동전 수"에도 들어가요), 시간이 끝날 때까지 못 하면 체력이 1 줄어요. 화면 위쪽 가운데에 진행 글이, 나올 때는 미스 얼굴과 말풍선이, 끝날 때는 성공·실패 글이 나와요.

## 잘한 점
- **자리가 잘 나뉘었어요.** 미션 수와 상태는 `logic/mission.rs`, 코인 주기·체력 줄이기·때(18:00, 06:00)는 `Game`, 보여 주기는 `view/mission.rs`예요. 수치와 글은 모두 `config.rs`에 있어요. 미션 뽑기는 `seed ^ MISSION_SEED_MIX`로 **따로 만든 난수**라서 상자·미스 무기·광석·수학 문제 난수와 상태를 나누어 쓰지 않아요 (구조상 바뀔 수가 없어요).
- **시험이 아주 꼼꼼해요.** `m8_missions.rs`가 완료 조건마다 하나씩 있고(18:00 순서, 체력 1에서 쓰러지면 퀴즈 안 열림, 채우기 전 체력, 클리어 아침, 갑옷, 창 멈춤, 빨리감기, 시드 1,000개), 반복은 모두 횟수가 정해져 있어요.

## 확인한 것 (문제 없음)
- **18:00 순서**: 시계 이벤트 안에서 `end_day_mission` → 실패면 체력 -1(`PlayerHurt`가 `NightStarted`보다 먼저) → `update_combat`에서 퀴즈. 쓰러졌으면 `&& !self.is_over()`로 퀴즈를 안 열어요.
- **06:00 순서**: 밤 미션 판정(채우기 **전** 체력) → 실패면 체력 -1 → 새 낮 미션 → 그 뒤 `combat.update`에서 체력 채우기. 클리어 아침(`day-1 >= DAYS_TO_CLEAR`, `nights_survived`와 같은 셈)에는 체력을 안 줄이고 새 미션도 안 내요. 쓰러졌으면 새 미션이 없어요.
- **밤 미션은 퀴즈가 닫힌 뒤**: `give_night_mission_after_quiz`가 `update` 끝에서 `math.is_open()`을 봐요. 퀴즈가 열린 프레임은 `update`가 일찍 돌아가서 아예 안 불려요. 같은 밤 두 번은 `night_given_on_day`가 막아요.
- **한 프레임에 18:00과 06:00이 같이 오는 일**: `MAX_FRAME_SECONDS` 0.25 × 빨리감기 10 = 2.5초, 밤은 600초라 진짜 게임에서는 생기지 않아요.
- **창 열린 동안 멈춤**: 상점·제작대·퀴즈가 열리면 `update`가 시계 전에 돌아가요. 화면 글 시간도 `update_screen_effects`가 창이 열린 동안 멈춰서, 18:00 실패 글은 퀴즈를 닫은 뒤에 보여요.
- **세기**: 코인은 `coins.update`로 주운 수만(팔기·보상 제외, 5,000이어도 셈). 나무/광석은 `keep_mining`이 다 캔 것만, 밖은 나무뿐이라 `in_cave`로 나눈 게 맞아요. 미스는 이번 `update_combat`에서 나온 `MissFound`만(앞 프레임의 `waiting_events`는 `events_before`로 빼요). 몬스터는 `gain_experience`만 늘리는 새 수라 시민 펑·쿵·주사기총은 안 들어가요.
- **이벤트 비우기**: `take_mission_events`가 `mem::take`로 비워요. 새 게임은 `Effects::new()`도 새로 만들어 옛 말풍선이 남지 않아요.
- **아이 눈높이**: "깜짝 미션이야! … 물리쳐 줘!", "아쉽다! 다음 미션도 해 보자" — 쉽고 무섭지 않아요. Jua에 없는 글자 시험도 있어요.

## 바뀐 기존 시험 3개 (따로 적음)
`turn_off_all()`을 빼고 직접 돌려 봤어요. 실패한 이유가 모두 **새 규칙이 맞게 끼어든 것**이었어요. 숨긴 버그는 없어요.
- `m4_zombie zombie_appears_in_the_same_frame_night_starts`: `[PlayerHurt{1}, NightStarted]` ≠ `[NightStarted]` → 18:00 낮 미션 실패 체력 -1 (D103 그대로).
- `m6_clear_score five_zombies_and_three_coins_make_nine`: 점수 103 ≠ 3 → 몬스터 5마리 밤 미션 보상 100 (D103 그대로).
- `m6_math quiet_game` 4개: 체력 6 ≠ 7 (18:00 실패 -1). `every_nightfall_opens_one_question_for_7_days`는 4일째에 쓰러져 퀴즈가 안 열렸는데, 이 시험은 밤 전체를 한 번에 건너뛰어서 아침 체력 채우기가 안 일어나요 (아래 5번). 진짜 게임에서는 생기지 않아요.
- 18:00·06:00 순서와 미션이 켜진 채 퀴즈가 어떻게 되는지는 `m8_missions`가 따로 봐요. 그래서 기존 시험에서 미션을 끈 것이 시험을 약하게 만들지 않아요.

## 돌려 본 결과 (실제 출력)
- `timeout 300 cargo test` (CARGO_TARGET_DIR=target-bob): **모두 통과**, 실패 0. m8_missions 102 통과 (logic 단위 시험 포함), main 291 통과·3 무시.
- `cargo build --target wasm32-unknown-unknown`: 통과, 경고 1개 `method miss_says_text is never used --> src\logic\mission.rs:137:12` (아래 1번).
- `cargo clippy --all-targets`: 새 코드에서 나온 것은 위 경고 하나뿐이에요 (나머지는 예전 파일).
- **망가뜨려 보기** (복사본에서, 하나씩):
  | 망가뜨린 것 | 결과 |
  |---|---|
  | 쓰러져도 18:00 퀴즈를 연다 | `failing_with_1_hp_goes_down_and_no_quiz_opens` 실패 (잡음) |
  | 클리어 아침에도 체력 -1 | `clear_morning_failure_does_not_cost_hp` 실패 (잡음) |
  | 퀴즈가 열려 있어도 밤 미션을 낸다 | `night_mission_comes_after_the_quiz_closes_once_per_night` 실패 (잡음) |
  | 몬스터 세기를 `monsters_given_experience` 대신 `monsters_defeated`(쿵·주사기총 점수 포함)로 | **모두 통과 (못 잡음)** → 아래 2번 |

## 꼭 고칠 것
없어요.

## 고치면 좋은 것
1. **`Mission::miss_says_text`가 게임에서 안 쓰여 빌드마다 경고가 나요.**
   - 어떤 경우: `cargo build`, wasm 빌드에서 매번 `never used` 경고. 화면은 `MissionEvent::Appeared { kind, goal }`로 `kind.miss_says_text(goal)`을 써서 이 메서드가 필요 없어요. 쓰는 곳은 시험 두 줄뿐이에요 (`view/mission.rs` 392줄, `tests/m8_missions.rs` 762줄).
   - 가장 작은 고침: `logic/mission.rs`의 `Mission::miss_says_text`를 지우고, 시험 두 줄을 `m.kind.miss_says_text(m.goal)`로 바꿔요. `Game::mission_now` 주석의 "미스 말은 `miss_says_text()`"도 같이 지워요.
   - 고칠 사람: 수산나 (view 시험 한 줄 포함, 아주 작음)
2. **"쿵·주사기총은 안 센다" 시험이 새 수(`monsters_given_experience`)를 지켜 주지 못해요.**
   - 어떤 경우: 위 망가뜨려 보기 4번째처럼 `monsters_defeated`로 바꿔도 통과해요. 주사기총(`fire_syringe_gun`)은 `update` 밖에서, 쿵(`slam`)은 `update_gathering`에서 일어나서, 둘 다 `update_combat` 앞뒤로 재는 창 **밖**이라 지금은 어느 수를 써도 같기 때문이에요. 나중에 쿵을 싸움 쪽으로 옮기면 쿵 점수가 몬스터 미션에 몰래 들어가도 시험이 모를 거예요.
   - 가장 작은 고침: `citizen_blast_and_sweeps_do_not_count`에서 주사기총을 쏜 뒤 `g.combat.monsters_given_experience`가 그대로인지 한 줄 확인해요 (`monsters_defeated`는 늘고 이것은 안 늘어야 함).
   - 고칠 사람: 수산나

## 작은 것
3. `Missions::turn_off_all`은 시험에서만 쓰는데 `#[allow(dead_code)]`로 게임 코드에 남아 있어요. 통합 시험도 `cargo test` 때 `cfg(test)`가 켜지니 `#[cfg(test)]`로 바꾸면 allow 없이 게임에서는 아예 없어져요. (수산나)
4. 성공 반짝이의 코인 자리 `COIN_SPARKLE_AT = (800, 28)`이 고정이에요. 코인 판 왼쪽 끝은 `right_row1_left`가 코인 자릿수·날짜에 따라 계산하니, 코인 5,000·"7일째"일 때 반짝이가 코인 판에 잘 걸리나요? 브라우저에서 한 번 봐 주세요. (로건, 엘리 확인 때)
5. (이번 변경과 무관) 06:00 체력 채우기는 `combat.update`가 밤을 한 번이라도 본 뒤(`was_night`)에만 일어나요. 시험에서 밤 전체를 한 프레임에 건너뛰면 채우기가 없어요. 진짜 게임은 밤이 수백 프레임이라 문제없지만, 미션 -1이 생겨서 이런 시험이 더 쉽게 쓰러져요. 시험을 쓸 때 알아 두면 좋아요.

## 못 본 것
- 실제 Jua 글꼴 폭과 740×360 폰에서의 겹침 (시험은 글 폭을 어림해서 봐요). 화면이 바뀌었으니 엘리의 브라우저 확인 몫이에요. 특히 진행 글이 정보판 사이에 들어가는지/아래로 내려가는지, 말풍선이 큰 알림 글과 겹치지 않는지.
- `hud::right_row1_left`가 `draw_coins_and_clock`의 실제 그리기 자리와 한 픽셀까지 같은지는 줄 단위로 맞대 보지 못했어요.
- `book/`, `.claude/agents/`, `qa/gates.md` 변경은 이번 리뷰 범위 밖이라 보지 않았어요 (개인정보 grep만 함, 이상 없음).

판정: 커밋해도 좋아요 (1, 2번은 같이 고치면 더 좋아요)
