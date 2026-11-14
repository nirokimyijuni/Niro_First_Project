# REVIEW-019 M8 상점 (US-062, US-070, US-071, US-072) + 끝 화면 고른 캐릭터 (US-077)

- 리뷰어: 밥 (에이전트)
- 날짜: 2026-11-14
- 대상: 312ab00 뒤 커밋 안 된 변경 전부 (logic/shop.rs 새 파일, game.rs, combat.rs, monster.rs, skeleton.rs, player.rs, weapon.rs, world.rs, coins.rs, config.rs, view/shop.rs 새 파일, gameover.rs, screens.rs, effects.rs, gather.rs, hud.rs, main.rs, tests/m8_shop.rs)

## 이 변경이 하는 일
집 문(x=3,555) 앞에서 E 키나 들어가기 버튼으로 상점을 열어, 코인으로 칼·도끼·총·포션·독약·스케이트·갑옷 두 가지를 사고 금·은·동을 판다. 상점이 열린 동안 게임 속 시간이 모두 멈춘다.
산 물건은 포션 바꾸기·쓰기, 스케이트(다음 06:00까지 1.5배), 갑옷(게임 속 시간으로 무적), 독(200픽셀, 실제 1초마다 1)으로 게임에 들어간다.
끝 화면(클리어·쓰러짐)에 미스 대신 고른 캐릭터가 나온다 (US-077).

## 잘한 점
- 규칙과 화면이 잘 나뉘었다. 살 수 있는지, 이유 글, 값은 모두 logic의 `why_cant_buy`/`buy_rows`/`sell_rows`가 정하고, view/shop.rs는 받은 그대로 그린다. 수치와 상점 문구(가격, 판 값, 시간, 범위, "코인이 모자라요" 등)는 config.rs에 있다.
- 멈춤을 이미 있는 제작대 멈춤과 같은 한 줄 길(`SHOP_PAUSES_GAME`, 맨 앞 return)로 만들어서 시계·몬스터·코인·갑옷·독이 한꺼번에 멈춘다. `everything_stops_while_the_shop_is_open_even_fast_forward` 시험이 빨리감기까지 함께 확인한다.
- 판 코인이 `picked_total`(점수)에 안 들어가게 `wallet.add`만 쓰고, 시험(`gold_silver_copper_sell_for_their_prices`)이 점수 0까지 확인한다. 5,000 막기는 `SELL_BLOCKED_IF_COINS_OVERFLOW` 하나로 부모님 결정을 바로 바꿀 수 있고, 시험도 두 경우를 다 다룬다.
- 옛 시험 파일(tests/의 m1~m7)은 한 줄도 바뀌지 않았다. 시험을 약하게 만든 것은 없다.

## 시험·빌드 결과 (실제 출력)
- `CARGO_TARGET_DIR=target-bob timeout 300 cargo test`: 모두 통과. 본체 192 passed (3 ignored), d36 73, m1 80, m2_clock 67, m2_day_night 74, m3 101, m4 97, m5 128, m6_clear_score 81, m6_ellie 66 (14 ignored), m6_ellie_math 66, m6_map 86, m6_math 81, m6_measure 59 (5 ignored), m6_miss_weapons 130, m6_monsters 76, m6_ring 104 (1 ignored), m7_ellie_reach 61, m7_gathering 116, **m8_shop 99**. 0 failed.
  - 참고: 빈 빌드 폴더에서 처음 돌렸을 때는 컴파일만으로 300초를 넘겨 끊겼다 (exit 143). 컴파일이 끝난 뒤 다시 돌려 위 결과를 얻었다. 시험 자체가 멈춘 것은 아니다 (가장 긴 d36 19초, m8_shop 16초).
- `cargo build --target wasm32-unknown-unknown`: 성공 (`Finished dev profile`). 경고 2개: `method is_armored is never used` (combat.rs 75줄), `method switch is never used` (weapon.rs 188줄). 둘 다 이번 변경에서 생겼다.
- `cargo clippy --all-targets`: 에러 1개 `right-hand side of && operator has no effect` (view/gather.rs 580줄) — 이번 변경과 무관 (gather.rs는 273줄 한 줄만 바뀜). 그 밖에 `duplicated attribute`(tests에서 config.rs 포함), `is_multiple_of` 8개 등 옛 경고.

## 꼭 고칠 것

### 1. 포션을 든 채 미스의 무기를 주우면, 칼을 든 것처럼 보이는데 공격은 포션이 된다
- 무엇이 문제인지: `Weapons::update`(weapon.rs 179줄)는 새 무기를 주우면 `held`를 그 무기로 바꾼다. 그런데 `Game.held_potion`은 그대로 `Some`이다. `update_combat`은 `held_potion.is_some()`이면 공격을 포션 쓰기로 보낸다 (game.rs 241줄). 상점에서 사는 길(`buy`)은 `held_potion = None`을 하는데, 줍는 길은 하지 않는다.
- 어떤 경우에 틀리는지: 낮에 체력 포션을 들고(체력 가득) 미스를 찾아 칼을 줍는다 → "칼을 주웠어요"가 나오고 정보판은 칼을 든 것으로 보인다. Q를 누르면 칼을 휘두르지 않고, 체력이 가득이라 포션도 안 쓰여 아무 일이 없다. 화면은 손에 포션을 그린다 (screens.rs 467줄). 이때 W를 누르면 `(칼, Some(체력))`이 목록에 없어 `unwrap_or(0)`으로 처음부터 다시 세서 칼로 간다. 밤이면 좀비 앞에서 공격이 안 나간다.
- 가장 작은 고침: `update_miss_and_weapons`에서 `picked`가 비어 있지 않으면 `self.held_potion = None;` 한 줄. 시험 하나: 포션을 든 채 무기를 주우면 `held_potion() == None`이고 Q가 칼로 맞는다.
- 고칠 사람: 수산나

## 고치면 좋은 것

### 2. logic/shop.rs 맨 위 `#![allow(dead_code)]`가 남아 있다
- 무엇이 문제인지: 주석은 "로건이 상점 화면을 붙이기 전에는… 붙이면 지운다"인데 화면이 붙었다. 이 줄이 파일 전체의 안 쓰는 코드 경고를 가린다. 예: `Potion::name()`은 부르는 곳이 없고 (grep 결과 `.name()`은 `ShopItem`·`Material`만), `ShopEvent`의 `healed`, `monsters` 값은 main.rs가 `..`로 버린다.
- 어떤 경우에 틀리는지: 다음에 상점 코드가 안 쓰이게 되어도 경고가 안 나와 아무도 모른다.
- 가장 작은 고침: 그 두 줄(주석+allow)을 지우고, 나오는 경고대로 안 쓰는 것을 지운다 (시험에서만 쓰면 `#[cfg(test)]`).
- 고칠 사람: 수산나

### 3. 옛 `Weapons::switch`와 새 `PlayerHealth::is_armored`를 아무도 안 부른다 (컴파일 경고)
- 무엇이 문제인지: `Game::switch_weapon`이 새 순서(무기→포션)를 직접 만들면서 weapon.rs 188줄 `switch`는 쓰이지 않는다. 빌드 때 `warning: method switch is never used`가 나온다 (위 결과).
- 어떤 경우에 틀리는지: 누가 옛 `switch`를 부르면 포션을 건너뛰고 `held_potion`이 남아 1번과 같은 꼬임이 생긴다.
- 가장 작은 고침: `switch`와 `is_armored`(combat.rs 75줄, 새로 만들었지만 부르는 곳 없음)를 지운다. 그 함수만 보는 weapon.rs 안 시험이 있으면 `Game::switch_weapon` 시험으로 옮긴다.
- 고칠 사람: 수산나

### 4. 화면 글 두 개가 main.rs에, 세 개가 view/shop.rs에 있다
- 무엇이 문제인지: "코인이 모자라요", "가지고 있어요", "들어가기"는 config.rs에 있는데, 같은 상점 글인 `SKATES_OFF_TEXT`("아침이 되어 스케이트를 벗었어요"), `ARMOR_OFF_TEXT`("갑옷 시간이 끝났어요")는 main.rs 90~91줄, `BOUGHT_TEXT`·`SOLD_TEXT`·`HINT_TEXT`는 view/shop.rs에 있다.
- 어떤 경우에 틀리는지: 부모님이나 아이가 상점 문구를 바꾸려고 config.rs를 열면 다섯 개를 못 찾는다.
- 가장 작은 고침: 다섯 개를 config.rs의 "M8 상점" 묶음으로 옮긴다.
- 고칠 사람: 로건

## 작은 것

### 5. `level_up`의 설명 주석이 `spray_poison` 위로 붙었다
- combat.rs 308~311줄: `spray_poison`을 `level_up`의 `///` 두 줄과 `fn level_up` 사이에 끼워 넣어서, `spray_poison` 설명이 "레벨이 1 오른다…"로 시작하고 `level_up`은 설명이 없어졌다. 두 줄을 `fn level_up` 바로 위로 옮긴다. (수산나)

### 6. main.rs 한 줄이 붙어 버렸다
- main.rs 248줄 `}                            if ended.is_none() && (switch_weapon_pressed() ...` — 닫는 괄호 뒤에 줄바꿈이 빠졌다. 동작은 같지만 읽기 어렵다. 줄을 나눈다. (로건)

### 7. 독 깎기가 공격 코드와 거의 같다
- `hurt_poisoned_monsters`(combat.rs)의 좀비·스켈레톤 `retain_mut` 두 덩이가 `attack_with`와 거의 같다 ("맞으면 take_hit, 0이면 Defeated, 아니면 Hit"). 지금은 두 곳이라 괜찮지만, 셋째(주사기총 등 US-083)가 생기면 하나로 묶는다. 이번에는 고치지 않아도 된다. (수산나)

## 확인했고 문제없는 것 (요청받은 항목)
- 상점이 열리면 맨 앞에서 return → 시계, 몬스터, 화살·총알, 코인, 시민·미스, 갑옷, 독 모두 멈춘다. 걷기·공격 입력도 main.rs `window_open`과 logic `stand_still` 두 겹으로 막힌다. 열린 동안 `usable_in_reach`가 None이라 E로 다시 열리지 않는다.
- Esc: main.rs에서 `shop_open && menu.back`이 "고르기로 나가기"보다 먼저라 상점만 닫힌다.
- 사기: 모자라면 `why_cant_buy`가 먼저 막아 코인 그대로. 가진 무기·신은 스케이트는 "가지고 있어요". 쓴 코인은 `wallet`에서만 빠져 점수(`picked_total`)는 그대로.
- 팔기: 0개·철광석은 `NoneToSell`, 5,000 넘으면 지금은 막기(`CoinsFull`), 딱 5,000은 된다.
- 산 무기: `add_bought`가 `owned`에 넣으므로 미스 순서(`is_around`)가 건너뛰고, 총을 가지면 미스에게서 안 나온다 (시험 `bought_gun_never_comes_from_miss`).
- 스케이트: `DayStarted`(06:00)에서 벗는다. 05:00에 사면 1시간 (시험 있음). 빠르기는 매 프레임 `PLAYER_WALK_SPEED × 배수`라 벗으면 바로 220.
- 갑옷: `world_dt`(빨리감기 곱한 시간)로 줄고, 문제 창·제작대·상점·끝난 판에서는 그 줄에 오기 전에 return해서 멈춘다. `can_be_hit`에 `armored`가 들어가 좀비·화살·총알·펑이 모두 막힌다. 수학 틀림(`lose_from_quiz`)은 `can_be_hit`를 안 거쳐서 갑옷이어도 2 준다 (D71 (6)과 같음).
- 포션: 체력 가득이면 `heal`이 0 → 안 줄어든다. 0개가 되면 맨손. 포션을 들고 Q를 누르면 휘두르기 그림이 안 나온다 (`shows_swing`).
- 독: 몸 가운데끼리 거리 ≤ 200, 이음매 너머는 `nearest_copy`로 잰다. 실제 `dt`로 1초마다 1 (빨리감기와 상관없음). 동굴에서는 0마리. 06:00에 몬스터를 비울 때 같이 사라지고 경험치 없음. 독으로 쓰러지면 `gain_experience`로 경험치와 "해치운 몬스터"가 오른다.
- 끝 화면: `draw_end_panel`에 `assets.character(player.kind)`를 넘겨 고른 캐릭터가 춤추고(클리어), 쓰러진 화면에서는 어질어질 서 있다. 클리어 때 맵 위 캐릭터는 그리지 않아 둘이 겹치지 않는다. 글과 겹치지 않는지 시험(`hero_stays_below_the_title_and_above_the_tally`)이 있다.
- Wasm에서 안 되는 것(스레드, 파일, `Instant`) 없음. 무서운 글 없음 ("쓰러졌어요", "어질어질").

## 못 본 것
- 일부러 코드를 망가뜨려 보는 시험(뮤테이션)은 하지 않았다. 새 빌드 폴더 컴파일만 4분이 넘게 걸려 15분 안에 끝낼 수 없었다. 1번은 코드를 읽어 확인했다.
- 740×360 화면에서 상점 칸 크기, 실제 브라우저 모습은 엘리 몫으로 남긴다.

판정: 1번 고친 뒤 커밋
