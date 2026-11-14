# REVIEW-020 M8 네 아이템 + 상점 정리 + 멈춤 창 + 글꼴 끊김 + 빼기 범위

- 리뷰어: 밥 (에이전트)
- 날짜: 2026-11-14
- 대상: 312ab00 뒤 커밋 안 된 작업 트리. 상점 정리(US-062, D86), 네 아이템(US-078~083, D74~D80, D83~D85), Esc 멈춤 창(view/pause.rs, 부모님 요청), 글꼴 끊김(view/text.rs, BUG-029, qa/perf-2026-11-14.md), 빼기 범위(US-074, D88). REVIEW-019 고침 확인.
- **빼고 본 것**: 상자 부수기(US-089, `update_boxes`, `attack_box`, `break_box`, `take_box_events`, logic/boxes.rs, tests/m8_boxes.rs). 수산나가 만드는 중이라 다음에 따로 본다.

## 이 변경이 하는 일
제작대에서 곡괭이 말고도 자동차·엔더열매·다이아래더라이트 곡괭이·주사기총을 만들고(위쪽 그림 버튼으로 고르면 흐린 칸이 보인다), 동굴에서 루비·에메랄드·다이아몬드가 나온다.
포션·주사기총·엔더열매는 손에 들지 않고 PC는 R·T·Y·G, 폰은 정보판 그림을 눌러 쓴다. 해변에서 E(폰 "쿵!")로 쿵을 하면 화면 안 몬스터가 모두 쓰러진다.
상점에서 칼·도끼·총이 빠졌고, 게임 중 Esc·"멈춤" 버튼은 바로 나가지 않고 멈춤 창을 띄운다. 글꼴은 7단계 크기만 만들어 새 글자 때 끊기지 않게 했다. 빼기 문제는 두 자리끼리만 낸다.

## 잘한 점
- 규칙과 화면이 잘 나뉘었다. 쓸 수 있는지(체력 가득, 화면에 몬스터 없음, 창이 열림, 벌써 먹음)는 모두 logic(`use_potion`, `fire_syringe_gun`, `eat_ender_fruit`, `slam_power_here`)이 정하고, main.rs는 키·탭을 `Pocket`으로 바꿔 넘기기만 한다. 조합 모양·이름·문구·점수·다이아몬드 수·빠르기·해변 범위가 모두 config.rs에 있다. logic 새 줄에서 박힌 규칙 숫자는 찾지 못했다.
- 시험이 촘촘하다. tests/m8_items.rs 104개가 완료 조건을 거의 하나씩 짚는다 (몬스터 없으면 주사기총 안 쓰임, 몬스터 없는 쿵은 다이아몬드만·점수 없음, 엔더 힘이 곡괭이보다 먼저, 06:00 자동차 내림, 05:00에 만든 차는 한 시간, 500개 맵에서 빠른 차가 상자에 안 박힘). 빼기 시험(m6_ellie_math)은 두 수 모두 10~99로 **더 엄하게** 바뀌었고, math.rs의 `const assert`로 config 범위를 잘못 바꾸면 빌드가 멈춘다.
- 글꼴 고침이 원인(perf 문서 1·2·3번)을 정확히 겨눈다: 단계 7개(`FONT_PIXEL_STEPS`), 배율 상한 2.0, "부릉!"·"쿵!"은 글꼴 크기를 고정하고 `zoom`만 바꾼다. `font_sizes_use_only_a_few_steps_on_any_screen` 시험이 어떤 배율·크기에서도 7단계를 넘지 않음을 확인한다.
- 폰 누르는 자리 시험(`tap_spots_are_big_and_clear_of_the_buttons`)이 포션 네 칸 × 무기 있음/없음 × 두 폰 크기에서 버튼·조이스틱·멈춤 버튼과 안 겹치고 48 화면 픽셀이 넘는지 본다.

## 시험·빌드 결과 (실제 출력, `CARGO_TARGET_DIR=target-bob`, 끝난 뒤 지움)
- `timeout 300 cargo test`: 모두 통과, 0 failed. 본체 221 passed (3 ignored), d36 79, m1 86, m2_clock 73, m2_day_night 80, m3 108, m4 103, m5 134, m6_clear_score 87, m6_ellie 72 (14 ignored), m6_ellie_math 72, m6_map 92, m6_math 91, m6_measure 65 (5 ignored), m6_miss_weapons 136, m6_monsters 82, m6_ring 110 (1 ignored), m7_ellie_reach 67, m7_gathering 122, m8_boxes 88, **m8_items 104**, m8_shop 106. 가장 긴 것 d36 9.1초.
- `cargo build --target wasm32-unknown-unknown`: 성공 (`Finished dev profile`). 경고 2개: `methods pickaxe_is_ready and take_pickaxe are never used` (crafting.rs 214줄), `associated constant ALL is never used` (shop.rs 65줄 `Potion::ALL`).
- `cargo clippy`: 에러 0. 위 두 경고 말고는 옛 경고(assets.rs, screen.rs, screens.rs 인자 8개, sky.rs, main.rs `Scene` 크기 차이, text.rs `fold`, `is_multiple_of`).
- 바뀐 옛 시험: m6_ellie_math(빼기 범위, D88로 더 엄해짐), m6_miss_weapons(`weapons.switch()` → `g.switch_weapon()`, 옛 함수가 지워져서), m7_gathering(광석 일곱 가지·문구 늘어남). 셋 다 결정 사항대로 바뀐 것이고 약해진 것은 없다.

## REVIEW-019 고침 확인
- 1번(포션 든 채 무기 줍기): 포션을 손에 들지 않게 바뀌어 `held_potion`이 없어졌다. 문제 자체가 사라졌다. 고쳐짐.
- 2번(shop.rs `#![allow(dead_code)]`): 지워졌다. 고쳐짐. (그 대신 `Potion::ALL` 경고가 보이게 되었다 → 아래 4번)
- 3번(`Weapons::switch`, `is_armored`): 둘 다 지워졌다. 고쳐짐.
- 4번(상점 글 다섯 개): `SKATES_OFF_TEXT`, `ARMOR_OFF_TEXT`, `SHOP_BOUGHT_TEXT`, `SHOP_SOLD_TEXT`, `SHOP_HINT_TEXT` 모두 config.rs로 갔다. 고쳐짐.
- 5번(`level_up` 주석이 `spray_poison` 위로 붙음): **그대로다** (combat.rs 306~310줄). 아래 6번.
- 6번(main.rs 붙은 줄): 없어졌다. 고쳐짐.

## 꼭 고칠 것
없음.

## 고치면 좋은 것

### 1. 수학 문제 창에서 Esc(PC)를 누르면 멈춤 창 없이 바로 캐릭터 고르기로 나가 판이 사라진다
- 무엇이 문제인지: main.rs의 Esc 처리 순서에서 멈춤 창은 `!window_open`일 때만 뜬다. 문제 창이 열려 있으면 그 줄을 건너뛰고 맨 끝 `else if menu.back || retry || quiz_back`로 가서 `Scene::Select`가 된다. 제작대·상점은 Esc가 그 창만 닫는데, 문제 창만 판 전체를 끝낸다.
- 어떤 경우에 틀리는지: 3일째 18:00 문제가 떴을 때 아이가 이제 "Esc = 멈춤"인 줄 알고 Esc를 누른다 → 멈춤 창 없이 고르기 화면으로 가고 3일 동안 한 판이 없어진다. 옛날에도 그랬지만, 이번에 다른 곳의 Esc가 모두 "멈춤/닫기"로 바뀌어서 여기만 다르다. (폰 문제 창의 "◀ 고르기" 버튼도 같은 길이다.)
- 가장 작은 고침: 문제 창에서도 Esc는 `*paused = true`로 보내고(멈춤 창이 문제 창보다 위에 그려지게), 또는 문제 창에서는 Esc를 무시한다. 어느 쪽인지는 부모님께 물을 거리다. 시험은 `pause_choice`처럼 "문제 창 + Esc → 고르기로 가지 않는다"를 고르는 작은 함수 하나로.
- 고칠 사람: 로건 (어느 쪽으로 할지는 감독이 부모님께)

### 2. 멈춤 창에는 유저스토리와 완료 조건이 없다
- 무엇이 문제인지: stories/backlog.md를 "멈춤"으로 찾으면 문제 창·제작대·상점의 멈춤([임시-31], [임시-18], [임시-24])만 나오고, Esc 멈춤 창 스토리는 없다. 코드 주석은 "부모님 요청 2026-11-14"뿐이다.
- 어떤 경우에 틀리는지: 엘리가 무엇을 기준으로 통과를 줄지 없다. 예: 멈춘 동안 갑옷 시간이 줄어야 하는지, 폰 "◀ 고르기"가 "멈춤"으로 바뀐 것이 맞는지, 1번의 문제 창 Esc를 어떻게 할지.
- 가장 작은 고침: 니로가 스토리 하나(멈춤 창, 이어서 하기, 처음으로, 멈춘 동안 시계·몬스터·갑옷이 그대로)를 쓴다. 멈춤은 main이 `update`를 안 부르는 방식이라 `cargo test`로 확인할 수 없어 엘리 브라우저 확인이 필요하다.
- 고칠 사람: 니로(에이전트)

### 3. 멈춤 창 글 네 개와 "부릉!"이 config.rs 밖에 있다
- 무엇이 문제인지: view/pause.rs 14~17줄 `"잠깐 멈췄어요"`, `"이어서 하기"`, `"처음으로"`, `"(캐릭터 고르기)"`, view/items.rs 28줄 `VROOM_TEXT = "부릉!"`(아이가 정한 말, D84). REVIEW-019 4번에서 상점 글을 config.rs로 옮긴 것과 다르다. 같은 파일의 `SLAM_SHOUT_TEXT`는 config의 `SLAM_TEXT`를 가리키는데 "부릉!"만 view에 있다.
- 어떤 경우에 틀리는지: 아이가 "부릉부릉!"으로 바꾸자고 하면 부모님이 config.rs에서 찾지 못한다.
- 가장 작은 고침: 다섯 개를 config.rs로 옮기고 view는 가리키기만 한다.
- 고칠 사람: 로건

### 4. 안 쓰는 코드 경고 두 개 (이번 변경에서 생김)
- `Crafting::pickaxe_is_ready`, `take_pickaxe` (crafting.rs 214~220줄): 주석은 "예전 화면이 쓴다"인데 화면은 이제 `take_result`만 쓴다. 부르는 곳은 tests/m7_gathering.rs뿐이라 wasm 빌드에서 `never used` 경고가 난다. 주석이 코드와 안 맞는다.
- `Potion::ALL` (shop.rs 65줄): 부르는 곳이 없다 (`Pocket::ALL`이 대신한다).
- 어떤 경우에 틀리는지: 경고가 쌓이면 다음에 진짜 안 쓰이게 된 함수 경고를 놓친다 (REVIEW-019 2번과 같은 이유).
- 가장 작은 고침: `Potion::ALL`은 지운다. 곡괭이 두 함수는 옛 시험을 `take_result`로 바꾸고 지우거나, 주석을 "옛 시험만 쓴다"로 고치고 `#[cfg(test)]`가 맞는지 본다 (tests/가 src를 어떻게 넣는지에 따라 다르다).
- 고칠 사람: 수산나

## 작은 것

### 5. `pocket_tap_rects`에 넘기는 "무기 있음"을 두 가지로 쓴다
- main.rs는 `!game.weapons.owned().is_empty()`, 바로 위 줄은 `game.can_switch()`. 지금은 같은 값이지만 나중에 `can_switch` 규칙이 바뀌면 누르는 자리(main)와 그리는 자리(screens.rs 550줄 `!owned.is_empty()`)가 어긋날 수 있다. 셋 다 같은 것을 쓰면 좋다. (로건)

### 6. `level_up` 설명 주석이 아직 `spray_poison` 위에 붙어 있다 (REVIEW-019 5번)
- combat.rs 306~307줄의 "레벨이 1 오른다…" 두 줄을 `fn level_up` 바로 위로 옮긴다. (수산나)

## 범위 밖이지만 알림 (상자, 이번 판정과 무관)
- `box_events`는 `attack_box`·`update_boxes`가 계속 넣는데 main.rs가 아직 `take_box_events`를 부르지 않는다 (빌드 경고 `method take_box_events is never used`). 지금 상태로 커밋하면 상자를 칠 때마다 목록이 늘기만 한다. 수산나·로건이 상자를 붙일 때 main.rs에서 매 프레임 비우는지 다음 리뷰에서 본다.

## 확인했고 문제없는 것 (요청받은 항목)
- **이벤트 비우기**: `shop_events`, `item_events`, `gather_events`는 main이 `update` 뒤에 매 프레임 `take_*`로 비운다. 주사기총이 만든 쓰러짐은 `waiting_events`에 들어갔다가 다음 `update` 맨 앞(`std::mem::take`)에서 나간다. 상점·제작대가 열려 일찍 return해도, 판이 끝나도 같이 나가서 남지 않는다. (상자 이벤트는 위 "범위 밖".)
- **멈춤 중 움직이는 길**: 멈춤이면 main의 `else if *paused` 줄에서 끝나서 `game.update`, 포션 키, 빨리감기, 바꾸기, `update_screen_effects`, `watch_car`를 모두 부르지 않는다. 손가락은 `touch.idle`로 메뉴 탭만 된다. 이어서 하면 `walk_lock = true`라 누르던 ←/→로 저절로 걷지 않는다. 멈춤은 `ended.is_none() && !window_open`일 때만 켜지고, 멈춘 동안에는 창이 열릴 수 없다 (update를 안 부르니 18:00 문제도 안 온다). 멈춤 중에 판이 끝날 수도 없다.
- **창이 겹칠 때 입력**: 제작대·상점은 `usable_in_reach`가 서로 None이라 한 번에 하나만 열린다. 문제 창이 열린 동안 `can_use_potion_now`가 false라 R·T·Y·G와 그림 누르기가 안 쓰이고, `slam_power_here`도 None이다. 창이 열린 동안 `touch.update`를 부르지 않아 그림 탭도 들어가지 않는다. Esc 순서: 제작대 닫기 → 상점 닫기 → 멈춤 창 안 → 멈춤 열기 → 끝난 화면·문제 창은 고르기로 (문제 창은 1번).
- **폰 버튼 겹침**: 바꾸기 버튼이 아래로 옮겨졌고, 그림 누르는 자리는 버튼·조이스틱·멈춤 버튼과 안 겹친다 (시험 있음). 버튼이 먼저 검사되어 혹시 겹쳐도 버튼이 이긴다. 쓰는 물건 칸은 정보판 맨 뒤에 같은 폭으로 놓여 갑옷 시간 글 폭이 바뀌어도 누르는 자리가 안 움직인다.
- **글꼴 크기가 매 프레임 바뀌는 곳**: "부릉!"과 "쿵!"은 `centered_zoomed`로 바뀌었다. `fit`(craft.rs), 위치 안내(screens.rs 555줄), `text_fit`(items.rs)처럼 크기가 계산되는 곳은 남아 있지만 이제 모두 7단계 중 하나로 묶여서 아틀라스(글꼴 그림)가 늘지 않는다. `SEEN_GLYPHS`도 (글자, 단계)로만 세서 화면 배율이 바뀌어도 늘지 않는다.
- **config 밖 수치**: logic의 새 줄에는 규칙 숫자가 없다 (`SCREEN_WIDTH`, `SLAM_BEACH_*`, `CAR_SPEED_MULTIPLIER`, `*_SLAM_SCORE`, `MINE_SECONDS_DIAMOND_PICKAXE` 모두 config). view의 숫자(흔들기 0.7초, 부릉 1.6초 간격, 효과 크기)는 보이는 모습이라 view에 두어도 된다. 글은 3번.
- **결정 사항 대조**: D76 3배·06:00 내림·스케이트와 안 겹침, D77 동 9개·다이아몬드 10개, D78 0.5초·쿵 뒤 곡괭이 없어짐, D79 몬스터 없으면 안 쓰임, D80 점수는 한 번만(700/600/500)·몬스터 0이면 없음, D83 R·T·Y·G(F는 빨리감기 그대로, 시험 있음)·W 순서에서 빠짐, D85 쓸어 쓰러뜨리면 경험치 없음(`sweep_monsters_on_screen`은 `gain_experience`를 안 부른다), D86 상점에 무기 없음·가격 상수 삭제, D88 빼기 11~99 빼기 10~(앞 수-1)로 답 1 이상 — 모두 맞다. 동굴 안에서는 쓸기가 0마리라 주사기총이 안 쓰인다.
- **5,000 상한**: `Wallet::add`가 넘치는 몫을 버리고, 팔기는 막지 않는다 (config 557줄 부모님 결정과 같음). 판 코인은 점수에 안 들어간다.
- Wasm에서 안 되는 것(스레드, 파일, `Instant`) 없음. 화면 글에 무섭거나 잔인한 말 없음 ("쓰러졌어요", "쿵! 몬스터들을 다 없애버려!"는 아이 말 그대로, 주사기총에 피 표현 없음).

## 못 본 것
- 상자 부수기 전부 (요청대로 뺌). 상자 때문에 낮에도 몬스터가 움직이게 바뀐 `Combat::update`(combat.rs 214~227줄)도 상자 리뷰 때 본다.
- view/craft.rs(408줄 바뀜), view/gather.rs, view/screens.rs, view/gameover.rs, view/shop.rs의 그리기 부분은 줄 단위로 다 읽지 못했다. 입력·누르는 자리·글꼴에 닿는 곳만 봤다.
- 일부러 코드를 망가뜨려 보는 시험은 하지 않았다 (시간). m8_items가 완료 조건을 하나씩 확인하는 것은 읽어서 봤다.
- 브라우저 화면(740×360에서 그림 버튼 크기, 자동차·금빛 기운 모습, 멈춤 창)과 진짜 폰 끊김은 엘리 몫이다.

판정: 커밋해도 좋아요 (1~4번은 다음 고침에, 1번은 부모님께 물어서)
