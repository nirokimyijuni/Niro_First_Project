# REVIEW-023 가방 고침 다시 보기 (REVIEW-022 뒤, US-091·US-062, D97·D98)

- 리뷰어: 밥 / 날짜: 2026-11-15 / 대상: 커밋 전 작업 트리 (fb4d8db 다음)
- 기준: stories/backlog.md US-091·US-062, docs/02-plan.md D94·D97·D98, REVIEW-022

## 이 변경이 하는 일
REVIEW-022 꼭 고칠 것 1번(↑를 누른 채 문에 닿거나 상점을 닫으면 저절로 들어감)을 고쳤어요. 물건을 쓸 수 없을 때 logic이 까닭을 알려 주고(`why_cant_use`, "체력이 가득해요" 같은 글), 가방에서 곡괭이 칸을 누르면 쓰는 법을 보여 줘요(D97). 집 문은 문 바로 앞(문 폭 + 좌우 8픽셀)에서만 들어가요(D98). 화면과 logic에 따로 있던 `Pocket`을 logic 하나로 합쳤어요.

## 잘한 점
- **까닭과 실제 쓰기가 같은 판단을 써요.** 독약 거리(`is_body_in_poison_range`)와 화면 안(`is_body_on_screen`)을 함수로 빼서 `spray_poison`·`sweep_monsters_on_screen`과 새 `count_monsters_*`가 똑같이 불러요. 그리고 `reason_always_agrees_with_using` 시험이 "까닭이 없으면 쓰이고, 있으면 안 쓰인다"를 네 상황 × 네 물건으로 직접 맞춰 봐요. 둘이 어긋나면 바로 깨지는 좋은 시험이에요.
- 1번 고침이 아주 작고 정확해요: ↑만 `is_key_pressed`, E는 그대로 `is_key_down`. `keys_to_mine`으로 빼서 단위 시험을 넣었고, main.rs에는 REVIEW-022에서 말한 두 경우((가) ↑ 누른 채 문에 닿기, (나) ↑ 누른 채 상점 닫기)를 그대로 따라 하는 시험이 있어요.

## REVIEW-022 다시 보기
| 번호 | 내용 | 결과 |
|---|---|---|
| 1 (꼭) | ↑ 꾹 누름으로 저절로 들어감 | **해결**. `keys_to_mine(e_down, up_pressed, up_enters)`, 시험 `up_enters_only_on_a_new_press`, `holding_up_into_the_door_does_not_enter`, `closing_the_shop_with_up_held_does_not_reopen_it` |
| 2 | 가방 시간 값이 config 밖 | **해결**. `KEY_HINT_SECONDS`·`BAG_FLASH_SECONDS`가 config.rs에 [임시-76]·[임시-77]로 있음 |
| 3 | 가방 입력 시험 없음 | **해결**. main.rs에 `bag_input`·`bag_opens` 시험 4개 |
| 4 | 문 끝 한 프레임 ↑ | 그대로 (작은 것, 그대로 두어도 됨) |
| 5 | 니로 메모 | 이번에 안 봄 |

## 특히 봐 달라고 한 것
- **why_cant_use와 실제 사용이 늘 같은지**: 같은 조건 함수를 써요. 순서도 같아요: 창이 열림·판이 끝남(→ 둘 다 아무 일 없음) → 0개 → 물건별 조건. 체력 포션은 `hp >= max_hp`(까닭)와 `heal(..) > 0`(쓰기)인데, 쓰러진 경우는 `is_over`가 먼저 막아서 둘이 같아요. 독약·주사기총의 동굴 안 처리(`in_cave`)도 양쪽 같아요.
- **spray_poison/sweep 기존 동작**: 클로저 몸통을 그대로 함수로 옮긴 것이라 계산이 한 글자도 안 바뀌었어요 (`nearest_copy(body, at.x).center().distance(at) <= POISON_RANGE`, 화면 가운데 기준 `nearest_copy` 뒤 좌우 비교).
- **집 문 범위 = 머리 위 표시 = 폰 버튼**: 셋 다 `usable_in_reach()` 하나를 봐요 (gather.rs `draw_usable_tag`·`up_enters`, main.rs `mine_button_label`, 들어가기 실행 game.rs 824줄). 그래서 늘 같아요.
- **동굴·제작대·나무 범위 그대로**: `usable_in_reach`에서 `HouseDoor`만 `is_in_front_of_door`이고 나머지는 예전처럼 `is_reached_on_either_side(body, MINE_REACH, ..)`예요. 광석 캐기(gathering.rs 247줄)도 그대로예요. m7_gathering·m7_ellie_reach 시험 통과.

## 시험 결과 (직접 돌림, CARGO_TARGET_DIR=target-bob, 끝나고 지움)
- `timeout 300 cargo test` 첫 번째: 새 폴더라 컴파일이 길어서 **300초를 넘겨 멈췄어요** (EXIT 124, 시험이 멈춘 것이 아니라 시간 부족). 두 번째부터는 제시간에 끝났어요.
- `timeout 300 cargo test --no-fail-fast`: **실패 4개** (아래 꼭 고칠 것 1번). 이번에 봐 달라고 한 부분의 시험은 모두 통과: main 단위 시험 271, m7_gathering 122, m7_ellie_reach 67, m8_bag_use 76, m8_boxes 90, m8_items 104, m8_shop 108/109 (실패 1개는 아래 1번).
- `cargo build --target wasm32-unknown-unknown`: 통과
- `cargo clippy`: 경고 8개, REVIEW-022와 같은 자리. 새 경고 없음.

## 꼭 고칠 것

### 1. 리뷰 중에 들어온 "문 둘레 상자 비우기" 변경 때문에 시험 4개가 깨져요 (이번 리뷰 범위 밖이지만 커밋을 막아요)
- **무엇이 문제인지**: 리뷰를 시작한 뒤(08:21) `logic/world.rs`·`config.rs`·`tests/m8_shop.rs`가 바뀌었어요. 동굴 입구와 집 문 양옆 `BOX_CLEAR_AROUND_DOOR`(= 몸 2개 + `MINE_REACH`) 안에 상자를 안 놓게 했는데, 그러면 화산 장소에 상자가 한 개도 안 생기는 시드가 있어요.
- **어떤 경우에 틀리는지** (실제 출력):
  - `m6_map every_place_has_boxes_and_never_more_than_ten`: "seed 0, 화산 상자 0개"
  - `m8_shop no_box_around_cave_entrance_and_house_door_on_any_seed`(새 시험): "시드 0: 장소 1에 상자가 없다"
  - `m6_miss_weapons walking_off_a_box_there_is_room_to_jump_the_citizen`: "상자 뒤 시민 0곳"
  - `m6_miss_weapons jumping_a_box_then_the_citizen_works_from_most_take_off_spots`: `tries > 100` 실패
- **가장 작은 고침**: 이 변경을 끝까지 마치거나(화산에 상자가 놓일 자리를 남기도록 비우는 폭을 줄이거나 화산의 상자 자리 규칙 조정), 마치기 전이면 world.rs·config.rs의 이 부분을 되돌리고 커밋해 주세요. 이 변경은 다 된 뒤 따로 리뷰해야 해요 (규칙 변경이라 밥 리뷰 대상).
- **고칠 사람**: 수산나
- 질문: 이 변경을 이번 커밋에 넣을 건가요? 넣는다면 D 번호(결정 사항)와 스토리 완료 조건도 있어야 해요. 지금은 config 주석에 "부모님 결정 2026-11-15"만 있어요.

## 고치면 좋은 것
없음.

## 작은 것

### 2. "벌써 먹었어요" 이벤트 길이 이제 게임에서 안 쓰여요
- 키·정보판·가방 모두 `use_pocket_or_why`를 지나서, 힘이 있으면 `why_cant_use`가 먼저 `AlreadyEaten`을 돌려주고 `eat_ender_fruit`는 불리지 않아요. 그래서 `ItemEvent::EnderFruitAlreadyEaten`(game.rs 478줄)과 effects.rs 332줄 처리는 시험에서만 돌아요. 글은 똑같이 나오니 아이에게 보이는 차이는 없어요. 나중에 정리할 때 지우면 돼요. (수산나·로건)

### 3. config 주석 두 개가 문서와 달라요
- `HOUSE_DOOR_REACH`: config 주석은 "부모님 결정"이라고만 쓰고, D98·US-062는 "8픽셀은 임시"라고 써요. `[임시]` 표시를 달아 주세요.
- `USE_NONE_LEFT_TEXT`: config 주석은 "[임시] 부모님 확인 필요"인데 US-091 완료 조건은 "부모님 확정 2026-11-15"예요. 주석을 확정으로 고쳐 주세요. (수산나)

## 확인한 것 (문제 없음)
- ↑ 들어가기: logic은 `mine_held && !mine_was_held`로 새 누름을 봐요. ↑는 누른 프레임에만 true라서 한 번 들어가고, 꾹 누른 E 캐기는 그대로예요.
- `Pocket` 합치기: view/shop.rs는 `pub use crate::logic::shop::Pocket`으로 다시 내보내고, `Pocket::count`는 `Game::pocket_count`를 그대로 불러요. 두 군데서 개수를 따로 세지 않아요.
- 곡괭이 설명: PC는 E, 폰은 버튼 이름(캐기·쿵!) 글. 폰 글에 E가 없는지 시험 있음. 문구는 쉬운 한글이고 무서운 표현 없음.
- 안내 글("체력이 가득해요", "가까이에 몬스터가 없어요", "화면에 몬스터가 없어요", "남은 게 없어요")은 모두 config.rs에 있고 US-091 완료 조건과 글자가 같아요.
- 바뀐 시험: m8_shop에 문 앞·벽 앞 시험 2개가 새로 생겼어요 (벽 앞에서 E 안 됨, ↑는 점프). 약해진 시험은 없어요.

## 못 본 것
- 브라우저·진짜 폰 (엘리 몫). 가방 창에서 안내 글이 길 때 창 밖으로 넘치는지는 그림을 안 봤어요.
- 일부러 코드를 망가뜨려 보는 시험은 안 했어요. 대신 `reason_always_agrees_with_using`·`in_the_cave_...`·`house_wall_beside_the_door_...` 시험이 각 조건을 빼면 깨지는 것을 읽어서 확인했어요.
- 꼭 고칠 것 1번의 "문 둘레 상자 비우기" 변경 자체는 리뷰하지 않았어요 (도중에 들어왔고 아직 시험이 깨져 있어요).
- REVIEW-022 5번(니로 메모)은 안 봤어요.

판정: 1번 고친 뒤 커밋 (이번에 봐 달라고 한 가방 고침 자체는 커밋해도 좋아요)
