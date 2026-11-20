# REVIEW-026 영어 퀴즈 (US-087·088) + 작은 고침 + 영어 그림

- 리뷰어: 밥 / 날짜: 2026-11-20 / 대상: 커밋 안 된 작업 트리 (b1a06c8 다음)
- 걸린 시간: 약 20분

## 이 변경이 하는 일
잠깐 퀴즈가 열릴 때 수학이나 영어 낱말 문제를 반반으로 고른다(D82, [임시-56]). 영어 문제는 그림 보고 낱말 고르기 / 낱말 보고 그림 고르기 두 종류, 보기 4개, 기회 2번, 두 번 다 틀리면 체력 -2(갑옷이어도)이다. 헷갈리는 묶음(D106 12묶음)은 한 문제에 같이 나오지 않는다. 그 밖에 멈춤 창 친구, 노란 별 5개, 은색 바늘, 문구 두 개를 바꿨다.

## 잘한 점
- **설계가 깔끔해요.** 영어 문제는 `logic/english.rs`가 만들기만 하고, 창·기회·결과 글·체력은 기존 `MathQuiz` 한 곳이 맡아요. 그래서 "문제가 열리면 시계·몬스터·미션이 멈춘다", "BUG-034 2초 기다림 뒤 연다", "Esc는 멈춤 창" 같은 규칙이 모두 `is_open()` 하나로 그대로 이어져요. `game.rs`에서 바뀐 곳은 `wrong_hp_loss()` 한 줄뿐이고, 창을 닫기 **전에** 값을 읽어서 순서도 맞아요.
- **난수를 따로 굴려요.** 수학·영어 고르기(`QUIZ_SUBJECT_SEED_MIX`)와 영어 문제(`ENGLISH_SEED_MIX`)는 다른 시드에서 굴러서 상자·미스 무기·광석·미션 뽑기를 건드리지 않아요. 수학 문제도 영어가 끼어든 만큼만 건너뛰고 차례는 그대로예요 (`same_seed_same_order_and_math_sentences_do_not_change`, `mission_draws_do_not_change_...`가 확인).
- 한 프레임에 두 키를 눌러도 하나만 고르고(`apply_quiz_input`의 `break`), 영어일 때 숫자판·숫자 입력은 `can_type()`에서 막혀요. 그림 파일이 없으면 "?"로 그려서 안 깨지고, 모든 그림 파일이 있는지 시험도 있어요.

## 돌려 본 결과 (실제 출력)
- `timeout 300 cargo test` (CARGO_TARGET_DIR=target-bob): **모두 통과**, 실패 0. 경고 2개:
  - `warning: associated function 'with_maker' is never used --> src\logic\math.rs:209` (게임 빌드)
  - `warning: methods 'answer_right' and 'answer_wrong' are never used --> src\logic\math.rs:411` (bin 시험 빌드)
- `cargo build --release --target wasm32-unknown-unknown`: **통과**, 경고 1개 (`with_maker` 위와 같음).
- `cargo clippy --all-targets`: 이번 변경에서 새로 생긴 것은 위 두 경고와 "상수 assert" 알림(items.rs:257, pause.rs:246)뿐. 나머지는 예전부터 있던 것.
- **망가뜨려 보는 시험** (작업 트리를 복사한 곳에서만, 원본은 안 건드림):
  - 흐려진 보기 다시 고르기 막는 줄 지우기 → `first_miss_keeps_health_...` 실패 (잡힘)
  - 헷갈리는 묶음 거르기 지우기 → `confusing_words_never_share_one_question` 실패 (잡힘)
  - 한 프레임 하나만 고르는 `break` 지우기 → `english_crossed_out_choice_does_nothing_and_one_pick_per_frame` 실패 (잡힘)
  - `ENGLISH_QUIZ_PERCENT`를 100으로 → 영어와 상관없는 시험 7개가 실패 (아래 1번)

## 꼭 고칠 것

**1. 첫 퀴즈가 우연히 수학이라서 통과하는 시험이 7개 있어요.**
- 무엇이: 이 시험들은 밤 문제를 `g.math.question().expect("밤 문제").answer` + `type_and_submit`으로 풀어요. 영어 문제가 열리면 `question()`이 None이라 멈추거나(expect), 문제를 안 풀고 넘어가서 시간이 안 흘러요.
- 어떤 경우에: `ENGLISH_QUIZ_PERCENT`를 100으로 바꾸면 아래가 모두 실패했어요. 비율은 [임시-56]이라 부모님이 바꿀 수 있는 값이고, 50인 지금도 시드가 바뀌면 같은 일이 생겨요.
  - `src/logic/game.rs:1276` `clock_events_come_through_game` (bin 단위 시험 + 모든 시험 파일에 같이 들어가요)
  - `tests/m2_day_night.rs:243` `us013_coins_can_be_picked_at_night`
  - `tests/m4_zombie.rs:555` `zombie_appears_in_the_same_frame_night_starts`
  - `tests/m7_gathering.rs:554` `six_pm_pushes_the_player_out`
  - `tests/m8_boxes.rs:529` `broken_box_comes_back_new_at_the_next_6_oclock`
  - `tests/m8_items.rs:134` `run_until_morning` → `car_is_left_at_the_first_six_o_clock`, `ender_power_lasts_through_morning_and_the_cave`
  - `tests/m8_shop.rs:72` `answer_quiz_if_open` → `skates_stay_on_until_the_next_six_am` (영어면 조용히 안 풀고 넘어가서, 날짜가 안 바뀌었다고 엉뚱한 이유로 실패)
- 가장 작은 고침: 각 자리를 `assert!(g.math.is_open()); g.math.answer_right();` (m8_items·m8_shop은 `if g.math.is_open() && g.math.result().is_none() { g.math.answer_right(); ... }`)로 바꾼다. m6_clear_score·m8_missions에서 이미 한 방법과 같아요. game.rs:1276이 `answer_right`를 쓰면 2번의 bin 시험 경고 하나도 같이 사라져요.
- 고칠 사람: 수산나 (시험 파일은 엘리와 나눠도 됨)

**2. 안 쓰이는 코드 경고 2개 (wasm 빌드에도 1개 나와요).**
- 무엇이: `MathQuiz::with_maker`는 이제 시험에서만 써서 게임 빌드에서 "never used" 경고가 나요. `answer_wrong`은 어디에서도 안 써요 (grep 0건). `answer_right`는 시험 파일에서만 써서 bin 시험 빌드에서 경고가 나요.
- 어떤 경우에: `cargo build --target wasm32-unknown-unknown` 출력에 `with_maker is never used`가 매번 나와요. 경고가 쌓이면 진짜 문제 경고를 놓쳐요.
- 가장 작은 고침: `with_maker`에 `#[cfg(test)]`를 붙인다 (쓰는 곳이 모두 시험: logic/math.rs 시험, view/math.rs 시험, tests/m6_math.rs). `answer_wrong`은 지운다. `answer_right`는 1번 고침으로 bin 시험에서 쓰이게 된다. `#[allow(dead_code)]`로 덮지 않는다.
- 고칠 사람: 수산나

## 고치면 좋은 것

**3. 쓰지 않는 Pole 그림이 게임 폴더에 들어가 있어요.**
- 무엇이: `game/assets/sprites/english/en-pole.png`가 있어요 (파일 48개 = 쓰는 그림 47 + Pole). D106에서 Pole을 뺐고 게임은 이 파일을 불러오지 않아요.
- 어떤 경우에: 배포 때 안 쓰는 그림이 같이 올라가고, 나중에 "그림 47개"를 세는 사람이 헷갈려요.
- 가장 작은 고침: `game/assets/sprites/english/en-pole.png`만 지운다 (art/spritecook 쪽 원본과 json 기록은 남겨도 됨).
- 고칠 사람: 헤일리 (또는 로건)

**4. backlog [임시-62] 줄에 Pole이 남아 있어요.**
- 무엇이: `stories/backlog.md` 1851줄 [임시-62]에 "**Hose·Rope·Pole·Mop**"이라고 써 있어요. 코드·D106·완료 조건(1937줄)은 "Hose·Rope·Mop"이에요.
- 가장 작은 고침: 그 줄에서 "Pole·"을 지운다.
- 고칠 사람: 니로(에이전트)

**5. 영어 체력 값과 수학 체력 값이 같아서, 순서가 틀려도 시험이 못 잡아요.**
- 무엇이: `wait_after_math_answer`가 `wrong_hp_loss()`를 창을 닫기 **전에** 읽어야 영어 값이 나와요. 지금은 맞게 되어 있어요. 그런데 `ENGLISH_WRONG_HP_LOSS`와 `MATH_WRONG_HP_LOSS`가 둘 다 2라서, 누가 이 줄을 닫은 뒤로 옮겨도 시험은 통과해요.
- 어떤 경우에: 나중에 부모님이 영어만 체력 -1로 바꾸면, 줄이 옮겨져 있을 때 영어도 -2가 빠져요.
- 가장 작은 고침: 지금은 고칠 것 없음. 두 값 중 하나를 바꿀 때 `two_misses_show_the_word_then_cost_2_hp_even_with_armor`가 그 값을 보도록 이미 상수로 적혀 있으니, 그때 같이 확인하면 돼요. (기록만)
- 고칠 사람: — (기록)

**6. 시작할 때 영어 그림 47장을 하나씩 차례로 불러와요 (질문).**
- `EnglishArt::load`가 `load_file`을 47번 차례로 `await`해요. 브라우저에서는 파일 47개를 하나씩 받아요 (모두 합쳐 약 600KB). 폰에서 첫 화면이 뜨는 시간이 얼마나 늘었나요? 엘리 브라우저 확인 때 첫 화면까지 걸린 시간을 한 번 봐 주면 좋겠어요. 눈에 띄게 느리면 그때 고쳐요.
- 고칠 사람: 로건(확인은 엘리)

## 작은 것

**7.** `tests/m8_english.rs`의 `all_48_words_are_answers_...`는 "Pole은 정답에도 보기에도 나오지 않는다"(완료 조건 1936줄)를 직접 보지 않아요. 지금은 48개 목록 검사로 사실상 막혀 있지만, `assert!(ENGLISH_WORDS.iter().all(|(w, _)| *w != "Pole"))` 한 줄이면 완료 조건과 바로 맞아요. (수산나)

**8.** `logic/english.rs` 단위 시험 `words_are_all_different_and_pictures_follow_the_name_rule`과 `tests/m8_english.rs`의 `pictures_are_named_after_the_word_and_tree_uses_the_game_tree`가 거의 같은 검사예요. 하나만 남겨도 돼요. (수산나)

## 바뀐 시험 따로 보기
- `m6_clear_score`, `m8_missions::answer_right_and_close`: 수학 답 넣기 → `answer_right()`(수학이든 영어든). 약해지지 않았어요.
- `m8_missions::mission_draws_do_not_change_...`: 수학 문제 비교를 "두 판이 같은 과목·같은 영어 문제, 수학이면 미션 없는 문제 만들기와 같은 차례"로 바꿨어요. 영어까지 보게 되어 오히려 넓어졌어요.
- `m6_math`: 수학 규칙 시험 두 곳을 `with_maker`(수학만)로 바꿨어요. 수학 규칙만 보는 시험이라 맞는 선택이에요.
- `effects.rs`: CAR_OFF_TEXT가 22자로 길어져 두 줄 기대값으로 바꾸고, 18자 한 줄 확인은 다른 글로 옮겼어요. 확인 내용은 그대로예요.
- `mission.rs`: 성공 문구 기대값만 바뀜.

## 작은 고침 (2번 묶음) 확인
- 노란 별 5개(gameover.rs), 은색 바늘(items.rs), 멈춤 창 친구(pause.rs, main.rs): 화면만 바뀌고 규칙은 안 건드려요. 친구 칸(x 730~940)은 창 오른쪽 바깥, 960 안이에요. 멈춤 창을 열 때마다 `date::now()`로 고르는데 화면 쪽 난수라 게임 시드와 상관없어요. 문제 없음.
- 문구 "아쉽다! 아침이 와서 자동차에서 내렸어요", "잘했어! 코인 100개를 줄게!": 쉬운 한글, 무섭거나 꾸짖는 말 없음.
- 영어 화면 문구("이 그림은 영어로 뭘까?", "너무 아쉽다! 정답은 Kite 야")도 쉽고 부드러워요.

## 못 본 것
- `art/spritecook/english/draw_english.py`(1,197줄)는 머리 부분과 Pole 부분만 봤어요. 그림 47장을 눈으로 하나씩 보지는 않았어요 (헤일리·엘리·부모님 확인 몫).
- 브라우저에서 실제로 눌러 보지는 않았어요 (엘리 몫). 폰 터치 모드에서 보기 칸과 "◀ 고르기" 버튼이 안 겹치는 것은 시험(`choices_are_big_apart_and_inside_the_panel`)으로만 확인했어요.
- `docs/02-plan.md`, `.claude/agents/*.md`, `qa/gates.md`, `stories/child-decisions.md`의 바뀐 글은 읽지 않았어요.

판정: 1, 2번 고친 뒤 커밋
