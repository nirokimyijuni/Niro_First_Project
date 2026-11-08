# REVIEW-016 수학 문제 (US-074~076, D67·D68)

리뷰: 밥(에이전트) / 2026-11-08 / 대상: 5ba2622 뒤 커밋 안 된 변경

## 이 변경이 하는 일
미스가 게임 이야기로 된 수학 문제(곱하기, 세 자리 더하기, 세 수 더하기, 뺄셈)를 낸다. 미스가 준 무기를 2개 주울 때마다, 그리고 밤이 될 때(18:00) 문제 창이 열린다. 창이 열린 동안 게임 시간은 모두 멈춘다. 숫자판이나 숫자 키로 답을 넣고, 기회 3번을 다 틀리면 정답을 보여 준 뒤 체력이 2 준다.

## 잘한 점
- 빨리감기 길을 `Game::update_with_time_scale` 하나로 모았다. 전에는 main.rs와 시험(`fast_frame`)이 같은 순서를 따로 베껴 썼는데, 이제 둘 다 같은 logic 함수를 부른다. 그래서 "문제 창이 열리면 멈춤"이 보통 속도와 빨리감기에서 똑같이 지켜진다. main.rs의 `update_with_time_scale`은 새 함수만 부른다.
- 문제 주사위를 따로 굴려서(`MATH_SEED_MIX`) 상자·미스 뽑기가 바뀌지 않는다. 시험(`math_dice_do_not_change_the_other_dice`)도 있다.
- 받침에 맞는 조사(이/가, 은/는, 을/를)와 "정답은 13이야 / 2야"를 계산으로 만들고, 사람이 손으로 적은 표와 맞춰 보는 시험이 있다. 9개 이름 모두 맞다.
- 문제 창이 열린 동안 손가락은 `touch.idle`로, ←/→는 `hold_walk_until_released`로 막아서, 창이 닫힌 뒤 누르던 키·손가락이 저절로 걷기·점프가 되지 않는다.

## 돌려 본 결과 (실제 출력)
- `timeout 300 cargo test --no-fail-fast` (CARGO_TARGET_DIR=target-bob): 내 범위의 시험은 모두 통과 (`m6_math` 74개, 앱 안 시험 160개, `m6_clear_score` 77개, `m2_day_night` 70개 등).
  다만 엘리가 새로 만든 `tests/m6_ellie_math.rs`(이번 리뷰 대상 아님)가 2개 실패한다. 둘 다 아래 1번, 2번과 같은 문제다.
  - `us075_night_question_comes_before_the_first_monster`: "문제보다 먼저 몬스터 2마리가 나왔다"
  - `us074_subtract_follows_story_range`: 표 범위를 벗어난 뺄셈
- `cargo build --release --target wasm32-unknown-unknown`: 성공.
- `cargo clippy --all-targets`: 새 코드에서 경고 2개 (아래 7번).
- 망가뜨리는 시험(뮤테이션 시험)은 시간이 모자라 돌리지 않았다.
- 끝없는 반복: 새 시험의 `for` 반복은 모두 횟수가 정해져 있다 (`0..SEEDS`, `0..(70.0 / FRAME)`, `0..limit`). `while`·`loop`는 없다.

## 꼭 고칠 것

**1. 밤 문제보다 몬스터가 먼저 나온다** (US-075 "밤 문제는 첫 몬스터보다 먼저 나온다")
- 문제: `update_combat`에서 `combat.update(dt, is_night=true …)`가 먼저 돌고, 밤 문제는 함수 맨 끝 `queue_math_questions`에서 연다. 낮이 끝날 때 `seconds_to_next_spawn`이 0이라서, 18:00이 된 바로 그 프레임에 몬스터가 한 번 나온 뒤에 창이 열린다.
- 틀리는 예: 보통 속도로 18:00을 넘기는 프레임에 몬스터 2마리(`MONSTERS_PER_SPAWN`)가 생기고, 그 위에 문제 창이 뜬다. 엘리의 `us075_night_question_comes_before_the_first_monster`가 이것 때문에 실패한다.
- 가장 작은 고침: `update_combat`에서 `combat.update`를 부르기 **전에** 이번 프레임 `events`에 `NightStarted`가 있는지 보고, 있으면 `add_waiting()` + `open_next_waiting()` 뒤 창이 열렸으면 `return`. 끝의 `queue_math_questions`에서 밤 부분은 뺀다. `m6_math.rs`에도 "18:00 프레임에 몬스터 0마리" 시험 하나를 넣는다.
- 고칠 사람: 수산나

**2. 뺄셈 수가 스토리 표 범위를 지키지 않는다** (US-074 표, [임시-34])
- 문제: 표는 "빼지는 수 100~999, 빼는 수 10 이상, 답 1 이상"인데, 코드는 `MATH_SUBTRACT_RANGE = (20, 999)`, 빼는 수 `pick_between(1, a)`이다. config 주석도 "답은 0 이상"이라 스토리와 다르다.
- 틀리는 예: "밥은 화살을 20개 가지고 있었는데 20개를 쐈어요. 몇 개 남았을까요?"(답 0)나 "… 45개 중에서 3개를 …"(빼지는 수 두 자리, 빼는 수 한 자리)가 나온다. 엘리의 `us074_subtract_follows_story_range`가 실패한다. `m6_math.rs`의 시험은 `n[1] >= 1 && n[1] <= n[0]`만 봐서 이것을 놓친다.
- 가장 작은 고침: config에 `MATH_SUBTRACT_RANGE = (100, 999)`와 빼는 수 최소값(예: `MATH_SUBTRACT_TAKE_MIN = 10`)을 두고, 빼는 수를 `pick_between(MATH_SUBTRACT_TAKE_MIN, a - 1)`로 고른다(답 1 이상). `answers_are_right_and_in_range_for_1000_seeds`의 뺄셈 검사를 표 범위로 바꾼다.
- 고칠 사람: 수산나

## 고치면 좋은 것

**3. 플레이 중에 누른 숫자가 다음 문제 답 칸에 한꺼번에 들어간다**
- 문제: `math_keys_pressed`는 `get_char_pressed`로 글자 줄을 읽는다. macroquad 0.4.16에서 이 글자 줄은 프레임이 끝나도 비워지지 않는다(`end_frame`은 `keys_pressed`만 비우고 `chars_pressed_queue`는 안 비운다). 문제 창이 닫혀 있을 때는 아무도 이 줄을 읽지 않는다.
- 틀리는 예: 아이가 놀다가 숫자 키 `5`, `3`을 눌렀다(게임에서는 아무 일 없음). 몇 분 뒤 밤 문제가 열리는 첫 프레임에 답 칸이 "53"으로 차 있다. 문제 창이 열리기 전 마지막 결과 글(2초·3초) 동안 누른 숫자는 그때 읽고 버려지니 괜찮다.
- 가장 작은 고침: main.rs에서 문제 창이 닫혀 있을 때도 매 프레임 `math_keys_pressed()`(또는 `while get_char_pressed().is_some()`에 횟수 제한)로 줄을 비운다. 또는 문제가 열린 첫 프레임에 `clear_input_queue()`를 부른다.
- 고칠 사람: 로건

**4. 시험이 스토리 기준보다 느슨하다**
- `every_kind_shows_up_and_none_is_rare`는 10% 이상만 본다. 스토리는 15% 이상이다.
- `every_story_has_its_numbers_and_a_name_slot`은 틀 2개 이상만 본다. 스토리는 3개 이상이다(지금 config는 4·3·3·3개라 맞지만, 틀 하나를 지워 2개가 돼도 시험이 통과한다). config 주석도 "2개 이상"이다.
- 틀리는 예: 뺄셈 틀을 2개로 줄여도 `cargo test`가 통과한다.
- 가장 작은 고침: 숫자를 15%, 3개로 바꾸고 config 주석도 "3개 이상"으로.
- 고칠 사람: 수산나

## 작은 것

**5.** `math_key`의 숫자 키 → `MathKey::Digit` 짝짓기는 시험만 쓴다. 실제 게임은 숫자를 글자 줄(`get_char_pressed`)에서 받고 `math_key`의 숫자 결과는 `filter`로 버린다. 그래서 `number_keys_backspace_and_enter_work_in_the_quiz`의 숫자 부분은 실제 길을 확인하지 않는다. 고칠 사람: 로건 (숫자 부분을 빼거나, 주석으로 "숫자는 글자 줄에서 받는다"를 적기).

**6.** main.rs의 `update_with_time_scale`은 이제 `game.update_with_time_scale`을 그대로 부르기만 한다. 부르는 곳(main.rs 194, 306, 324줄)에서 바로 `game.update_with_time_scale`을 쓰면 함수 하나가 준다. 고칠 사람: 로건.

**7.** clippy 경고: `logic/game.rs` 171줄 `queue_math_questions`의 팔 하나짜리 `match`(1번을 고치면 사라질 수 있다), `logic/math.rs` 333줄 `% 28 != 0`(`is_multiple_of`). `game.rs`의 `wait_after_math_answer` 뒤 빈 줄 2개. 고칠 사람: 수산나.

## 확인했고 문제없는 것
- 멈춤: 열린 동안 `update_with_time_scale`은 `wait_after_math_answer`만 부른다. 시계, 코인(10초), 몬스터와 화살, 무적, 미스(20초)·시민(30초)·떨어진 무기(15초), 걷기 모두 안 흐른다. 빨리감기도 같다. `everything_waits_while_the_question_is_open`이 70초 동안 확인한다.
- 세 번 틀림: 정답 글 3초가 끝날 때 체력 -2, 무적 없음, 0이면 `PlayerDown` 이벤트가 나가 main.rs가 쓰러진 화면을 연다. 시험 있음.
- 겹칠 때: 무기와 밤이 같은 프레임이면 하나씩 차례로 열린다(시험 있음). 쓰러짐·클리어 프레임에는 열지 않는다. 클리어는 아침(DayStarted)에 오고 문제는 밤에 와서 겹치지 않는다.
- 아이템 세기: 한 프레임에 2개를 주워도 `count_before + 1..=picked_up_count` 반복으로 빠짐없이 센다. 가진 무기를 또 주워도 1개로 센다. 새 게임이면 0부터.
- 문장: 1,000개 시드에서 빈칸(`{`, `}`, `□`)이 남지 않고, 답은 999 이하, 곱셈 2~9단, 더하기 범위(100~499, 10~99)는 스토리 범위 안이다. 무서운 말은 없다("좀비 카스테라"는 게임 몬스터 이름). 60자 이하는 엘리 시험이 통과한다. "정답은 □이야"의 받침 처리(0 영이야, 2 이야, 10 십이야, 100 백이야)도 맞다.
- 입력: 같은 프레임 숫자는 글자 줄 순서대로 들어간다(`VecDeque::pop_front`). Esc(`menu.back`)와 터치 "◀ 고르기"는 창이 열려도 나가기다(D35). 숫자판을 누른 터치는 `touch.update`를 거치지 않아 공격·점프가 되지 않는다.
- 옛 시험: `fast_frame`이 베낀 순서 대신 진짜 함수를 부르게 바뀌어 더 튼튼해졌다. `fast_forward_clears_after_about_630_seconds`의 기댓값에 더한 `7 × 2초`는 밤 문제 7번의 칭찬 글 시간과 정확히 맞고, 허용 오차(1초)는 그대로다. `m2_day_night`, `logic/game.rs`의 바뀐 시험은 문제를 맞혀 닫는 줄만 더했고 원래 확인은 그대로다. 약해진 곳은 없다.
- 규칙 위치: 수치는 모두 config.rs의 `MATH_*`에 있다. view/math.rs는 그리기와 버튼 자리만 갖고 규칙은 `MathQuiz`를 부른다. logic/math.rs는 macroquad 그리기·입력을 쓰지 않는다. Wasm에서 안 되는 것은 쓰지 않았다.

판정: 1, 2번 고친 뒤 커밋
