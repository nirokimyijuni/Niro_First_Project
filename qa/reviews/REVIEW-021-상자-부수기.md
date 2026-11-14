# REVIEW-021 상자 부수기 (US-089, D87)

리뷰어: 밥 / 날짜: 2026-11-14 / 대상: 커밋 전 작업 트리 (상자 관련 부분만)

## 이 변경이 하는 일
도끼로 밖 맵의 나무 상자를 3번 치면 부서지고, 30%는 깜짝 아이템, 15%는 좀비·스켈레톤이 나온다.
부서진 상자는 `world.obstacles`에서 빠졌다가 다음 06:00에 같은 자리로 돌아오고, 그 자리에 누가 있으면 비킬 때까지 기다린다.
상자에서 나온 몬스터가 낮에도 움직이도록 `Combat::update`에서 몬스터 걸음을 밤 조건 밖으로 꺼냈다.

## 잘한 점
- 상자 순서 문제를 잘 피했다. `Boxes::list`와 `Coins::list`는 새 게임 때 한 번 만들고 그 뒤로 순서가 바뀌지 않는다. `world.obstacles`는 빼고(retain) 넣을(push) 때 순서가 바뀌지만, 화면은 `cracks_at(rect)`처럼 네모로 찾아서 순서가 바뀌어도 맞다.
- 확률 시험은 모두 정해진 seed(`SEED`, `0..400`, `0..3000`, `0..500`)와 정해진 횟수만 쓴다. `while`·`loop`가 없다. 06:00 겹침 시험도 240프레임 안에 끝난다.
- 일부러 망가뜨려 본 3가지가 모두 시험에서 잡혔다 (아래 "돌려 본 것").

## 꼭 고칠 것
1. **입고 있는 갑옷이 상자에서 또 나온다 (임시-65와 다름)**
   - 문제: `game.rs` `has_the_only_one`에서 `RainbowArmor`, `DiamondArmor`가 늘 `false`다. 임시-65는 "하나만 가질 수 있는 것을 이미 가졌으면 (… **입고 있는 갑옷** …) 빼고 다시 뽑는다"이고, 늘 나오는 것은 "포션과 엔더열매"뿐이다.
   - 틀리는 예: 다이아몬드 갑옷을 입고 상자를 부숴 무지개 갑옷이 나와 주우면 `put_on_armor(Rainbow)`가 다이아몬드 갑옷을 무지개 갑옷으로 바꿔 버린다. 입은 갑옷과 같은 것이 나와도 그 갑옷 몫이 빠지지 않는다.
   - 시험도 이 다름을 굳혀 놓았다: `tests/m8_boxes.rs` 263줄 주석 "포션·갑옷·엔더열매만 나온다", 284줄 `only_ones`에 갑옷이 없다.
   - 가장 작은 고침: `RainbowArmor => self.shop.armor() == Some(Armor::Rainbow)`, `DiamondArmor => self.shop.armor() == Some(Armor::Diamond)`로 바꾸고, `potions_and_ender_fruit_still_come_when_owned_in_the_game`에서 갑옷을 입혀 두고 그 갑옷이 안 나오는지 보게 한다. 879~880줄 주석도 맞춘다. (갑옷이 또 나와도 된다는 뜻이었다면 니로(에이전트)가 임시-65 글을 고친다.)
   - 고칠 사람: 수산나

## 고치면 좋은 것
2. **낮에 동굴에 있는 동안 상자 몬스터가 동굴 입구에 모인다 (질문)**
   - 이번 변경으로 낮에도 몬스터가 움직인다(`combat.rs` 222~227줄). 동굴 안에서는 `body_on_map()`이 입구 위 먼 하늘(`CAVE_ENTRANCE_X`, `CAVE_AWAY_Y`)이라, 상자 몬스터가 입구 x로 걸어가 기다린다. 나올 때 `leave_cave`는 주인공을 바로 그 자리에 세운다.
   - 예: 낮에 상자를 부숴 좀비가 나오자마자 동굴에 들어가 캐고 나오면, 나오는 순간 좀비와 겹쳐 맞는다. 전에는 낮에 몬스터가 없고 밤에는 동굴이 닫혀서(`CAVE_OPEN_AT_NIGHT = false`) 생기지 않던 일이다.
   - 이대로 괜찮은지 부모님께 물어볼 거리다. 막으려면 가장 작은 고침은 `in_cave`인 동안 `combat.update`에 쉬는 것을 알려 주거나(몬스터가 멈춤), 나올 때 1초 무적을 주는 것이다. 고칠 사람: 수산나 (정하는 사람: 니로(에이전트)·부모님)
   - 기존 낮 규칙은 깨지지 않았다: 06:00에 `was_night && !is_night`이면 몬스터를 모두 지우고 체력을 채우는 것은 그대로이고, 낮에 화면 밖에서 새로 나오는 것(`spawn_monster_when_ready`)도 여전히 밤에만 한다. 낮에 움직이는 몬스터는 상자 몬스터뿐이다. 전체 `cargo test`도 통과한다.

## 작은 것
3. 도끼 안내가 몬스터를 노린 총 쏘기에도 나올 수 있다: `hit_a_monster`는 바로 맞은 것(`ZombieHit` 등)만 본다. 총을 들고 앞에 상자가 있고 멀리 몬스터를 쏘면, 몬스터를 노렸어도 "도끼로 부술 수 있어요"가 한 번 나올 수 있다. 한 판에 한 번뿐이라 작은 것. (수산나, 고치지 않아도 됨)

## 확인한 것 (문제 없음)
- **06:00 겹침 기다림**: 상자 몬스터는 06:00에 다른 몬스터와 함께 지워지고, 낮에 부순 상자는 다음 06:00에 돌아오므로 몬스터 때문에 끝없이 기다리는 일은 없다. 주인공이 그 자리에 계속 서 있으면 계속 기다리는데, 그건 규칙대로다. 상자 몬스터(폭 48)는 상자(48·64) 가운데에 서므로 옆 상자와 겹쳐 나오지 않는다.
- **부서진 상자 빼고 넣기**: `break_box`는 같은 네모만 `retain`으로 빼고, 돌아올 때 `push`한다. 코인은 `coins.list[index]`(처음 순서)로 찾아서 맞다. 돌아오면 떨어진 무기·깜짝 아이템은 `land_things_again`으로 상자 위로 올라간다.
- **떨어진 아이템 목록**: 땅에 닿고 15초 뒤 `retain`으로 빠지고, 주우면 빠진다. 새 게임은 `Boxes::on_world`로 새로 만든다. 판이 끝나면 `update_boxes`가 멈춘다.
- **몬스터 한도**: `spawn_from_box`가 `is_map_full()`(6마리)이면 `None`, 좀비가 4마리면 스켈레톤을 낸다.
- **이벤트 비우기**: `main.rs` 371줄이 `game.update` 바로 뒤에서 매 프레임 `take_box_events()`(`std::mem::take`)를 부른다. 상자 이벤트는 `update` 안에서만 생기므로 멈춤 창 동안 쌓이지 않는다. `BoxView::update`는 나뭇조각·안내·와우를 시간이 지나면 지운다.
- **프로젝트 규칙**: `logic/boxes.rs`에 macroquad 그리기·입력·사운드 없음(`Rect`, `Vec2`만). 규칙 수치는 `config.rs`의 `BOX_*`에 있다. `view/boxes.rs`의 숫자는 그림 모양(나뭇조각 속도, 글 위치)뿐이다. 화면 글 "도끼로 부술 수 있어요", "와우!", "공격!", "○○을 얻었어요!"는 쉽고 무섭지 않다. 부서질 때 글·소리가 없다(임시-68).

## 돌려 본 것 (실제 출력)
- `timeout 300 cargo test` (CARGO_TARGET_DIR=target-bob): 모두 통과. `m8_boxes` 88 passed, 0 failed.
- `cargo build --target wasm32-unknown-unknown`: `Finished`, 경고 2개(`crafting.rs` `pickaxe_is_ready`/`take_pickaxe`, `shop.rs` `Potion::ALL` 안 씀) — 이번 변경과 무관.
- `cargo clippy --all-targets`: 테스트 빌드에서 에러 1개 `src\view\gather.rs:619` "right-hand side of `&&` operator has no effect" — 이번 변경과 무관(캐기 화면 시험). 상자 파일에서 나온 clippy 지적은 없다.
- 망가뜨려 보기 (작업 트리를 건드리지 않게 스크래치 복사본에서, `timeout 300`):
  1. 06:00 자리 확인을 끔(`someone_there = false && …`) → `box_waits_while_someone_stands_there_and_never_traps_anyone_for_500_seeds` 실패 (잡힘)
  2. 몬스터 걸음을 다시 밤에만(`if is_night { zombies.step }`) → `box_monster_waits_one_second_without_attacking_even_in_the_day` 실패 (잡힘)
  3. `spawn_from_box`에서 6마리 한도 확인을 뺌 → `no_box_monster_when_full_and_skeleton_when_zombies_are_full` 실패 (잡힘)
- 다 쓴 `target-bob`, `target-bob-mut`, 스크래치 복사본은 지웠다.

## 이번 변경과 무관 (따로 적음)
- clippy 에러 `view/gather.rs:619`, wasm 경고 2개 (`crafting.rs:214`, `shop.rs:65`).

## 못 본 것
- `view/boxes.rs`는 이벤트 처리·지우기·시험만 읽었고, 그리기 좌표(금 모양, 와우 별)는 자세히 보지 않았다. 브라우저 확인은 엘리 몫.
- 시민 `walkers`(걷는 시민)가 06:00에 상자 자리에 있을 때: `update_boxes`는 `citizens.list`만 보고 `citizens.walkers`는 보지 않는다. 걷는 시민이 상자 자리를 지나갈 수 있는지 끝까지 확인하지 못했다. 엘리가 브라우저에서 한 번 봐 주면 좋다.
- 미스가 상자 위에 있다가 떨어지는 경우: 숨는 곳 옆에는 상자가 없어서(`no_box_zones`) 생기지 않는다고 보았지만 따로 시험하지는 않았다.
- `update_with_time_scale`(빨리감기)에서의 상자 동작.

판정: 1번 고친 뒤 커밋
