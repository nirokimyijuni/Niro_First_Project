# REVIEW-003 폰 조작과 GitHub Pages 배포 (D16, D17)

- 리뷰한 사람: 밥 (코드 리뷰어)
- 날짜: 2026-11-02
- 대상: 아직 커밋하지 않은 변경 전부 (마지막 커밋 71c7184 다음). 루트의 `2026-11-01 20 07 21.png`는 대상 아님
  - 로건: 새 파일 `game/src/view/touch.rs`, 바뀐 파일 `game/src/main.rs`, `view/input.rs`, `view/screens.rs`, `view/hud.rs`, `view/mod.rs`
  - 케빈: 새 파일 `.github/workflows/deploy.yml`, 바뀐 파일 `deploy/build.sh`, `deploy/RELEASES.md`
  - 니로: `stories/backlog.md` US-031~036, `docs/02-plan.md` D16·D17

## 이 변경이 하는 일

폰에서 놀 수 있게 플레이 화면에 떠다니는 조이스틱(왼쪽 절반), 점프 버튼(오른쪽 아래), "◀ 고르기" 버튼을 붙였어요. 손가락이 화면에 닿으면 "터치 모드"가 켜져 버튼이 보이고, 폰을 세로로 들면 "가로로 돌려 주세요" 안내가 나오며 게임이 멈춰요. 그리고 main에 올리면(push) GitHub가 알아서 테스트 → 웹 빌드 → GitHub Pages 배포를 하고, 브라우저가 옛 파일을 쓰지 않게 `.js`/`.wasm` 주소 끝에 `?v=버전`을 붙여요.

## 잘한 점

- **손가락 판정을 그림과 떼어 놓은 것.** `TouchControls::update`는 손가락 목록만 받아 결과(`TouchFrame`)를 돌려주는 함수라서, "같은 프레임에 대고 뗀 톡", "꾹 눌러도 점프 한 번", "점프 손가락이 조이스틱을 뺏지 않음", "메뉴에서 누른 손가락은 플레이에서 무시" 같은 어려운 경우를 GPU 없이 테스트 13개로 지키고 있어요. 결과가 키보드와 같은 `PlayerInput` 모양이라 logic은 하나도 안 바뀌었어요.
- **메뉴 손가락이 플레이로 새지 않게 한 방법이 깔끔해요.** 플레이가 아닐 때는 `idle()`로 "지금 닿아 있는 손가락"만 기억해 두고, 플레이에서는 처음 보는 손가락만 새 손가락으로 쳐요. 세로 안내 중에도 같은 방법을 써서, 돌아왔을 때 저절로 걷지 않아요.
- **배포가 실패하면 멈춰요.** `cargo test`가 실패하면 그 단계에서 끝나고, `build.sh`는 `set -euo pipefail`에 더해 `?v=`를 못 붙였으면 스스로 `exit 1` 해요. `deploy` 작업은 `needs: build`라서 빌드가 실패하면 배포가 안 돼요.

## 돌려 본 것

`deploy/build.sh`는 실행하지 않았어요(부탁받은 대로). 다른 빌드와 부딪히지 않게 별도 target 폴더로 돌렸어요.

| 무엇 | 결과 |
|---|---|
| `cargo test` | 통과 (82 + 59 + 51 + 58개, 3개는 꺼 둔 것). `touch::tests` 13개 포함 |
| `cargo build --target wasm32-unknown-unknown` | 성공 |
| `cargo clippy --all-targets` | 이번 변경에서 새로 생긴 경고는 `touch.rs` 테스트의 "항상 같은 값 assert" 2개뿐(아래 10번). `main.rs:207`의 `is_multiple_of` 경고는 옛 것 |
| 액션 버전 (`gh api`) | `actions/checkout@v7`(최신 v7.0.1), `actions/upload-pages-artifact@v5`(최신 v5.0.0), `actions/deploy-pages@v5`(최신 v5.0.1), `Swatinem/rust-cache@v2` 태그 모두 있음. `dtolnay/rust-toolchain@stable`은 태그가 아니라 **브랜치**로 있음(그 액션의 원래 쓰는 법이라 맞음) |
| 저장소 Pages 설정 (`gh api .../pages`) | `build_type: workflow`(GitHub Actions로 배포), 주소 `https://nirokimyijuni.github.io/Niro_First_Project/` |

## 꼭 고칠 것

### 1. 손가락 위치가 화면 배율(dpi)만큼 틀려서, 대부분의 폰에서 조이스틱·점프 버튼이 엉뚱한 곳에서 눌려요 (로건)

- **무엇이 문제인지**: 게임은 `high_dpi: true`예요. 이때 macroquad의 `touches()`는 손가락 위치를 **실제 화면 점(물리 픽셀) 단위**로 줘요(miniquad JS `mouse_relative_position`이 `* devicePixelRatio`를 하고, macroquad `touches()`는 그대로 돌려줌). 그런데 `screen_width()`와 `mouse_position()`은 **dpi로 나눈 값**을 줘요(`macroquad-0.4.16/src/input.rs:51`, `window.rs:58`). `view/input.rs`의 `fingers()`는 나누지 않은 손가락 위치를, 나눈 값으로 만든 `Letterbox`에 넣어요.
  ```rust
  .map(|t| Finger { id: t.id, phase: t.phase, pos: letterbox.to_virtual(t.position) })
  ```
  메뉴는 `mouse_position()`(나눈 값)을 써서 괜찮고, **플레이 화면 터치만** 틀려요.
- **어떤 경우에 틀리는지**: 아이폰 12 Pro 가로(창 844×390, 배율 3. 크롬 기기 툴바 기본값도 같음). 가상 화면 배율 0.722, 왼쪽 띠 75px.
  - 점프 버튼(가상 802, 450)을 누르면 → 손가락 (655, 325) × 3 = (1964, 975) → 가상 (2616, 1350). **점프가 안 돼요.** 대신 화면 왼쪽 위 (218, 108) 근처를 눌러야 점프가 돼요.
  - 왼쪽 아래 (100, 300)에 엄지를 대면 → 가상 (311, 1246). 조이스틱은 생기지만 화면 아래 밖이라 **안 보여요.**
  - "◀ 고르기" 버튼은 창 왼쪽 위 아주 작은 곳(x 28~65, y 19~33)을 눌러야 돼요.
  - 배율 1인 PC 화면(윈도 100%)에서만 맞게 돼서, PC에서 마우스로 흉내 낼 때는 안 보였을 수 있어요. 윈도 배율 125%·150%인 노트북에서도 틀려요.
- **가장 작은 고침**: `fingers()`에서 `t.position / screen_dpi_scale()`을 `to_virtual`에 넣어요. 테스트로 지키려면 이 계산을 작은 함수로 빼서(예: `fn touch_to_virtual(pos: Vec2, dpi: f32, lb: &Letterbox) -> Vec2`) "배율 3에서 점프 버튼 자리를 누르면 `button_at`이 Jump" 테스트 하나를 두면 돼요.
- **고칠 사람**: 로건

## 고치면 좋은 것

### 2. 터치 모드를 "첫 터치 뒤"에 켜는 것과 US-035·036 완료 조건이 어긋나요 (니로 스토리 고침이 가장 작음)

로건이 보고한 차이를 조건 하나씩 대 봤어요.

| 조건 | 지금 코드로는 | 어긋남? |
|---|---|---|
| US-035 "터치 기기에서 세로가 더 길면 안내", "타이틀·고르기·게임 **어느 화면에서** 세로로 돌려도 안내" | 링크를 세로로 열면 **아직 한 번도 안 만졌으니** 타이틀이 작은 글씨 그대로 보여요. 한 번 톡 하면 그때 안내가 나오고, 그 톡은 삼켜져서 화면이 넘어가지 않아요(좋아요) | **어긋남** (첫 화면만) |
| US-036 "기기 툴바를 켜고 새로고침하면 버튼이 보이고" | 버튼은 어차피 플레이 화면에만 있고, 폰에서 플레이까지 가려면 손가락으로 타이틀·고르기를 눌러야 하니 그때 이미 터치 모드예요 | 실제로는 맞음 |
| US-034 "터치 기기에서는 게임 중 고르기 버튼이 보인다" | 위와 같은 이유로 맞음 | 맞음 |
| US-036 참고 "터치 화면 노트북은 터치 기기로 보고 버튼을 보여 준다. 그래도 키보드는 그대로" | 노트북에서 화면을 한 번 만지기 전까지 버튼이 없고, **키를 하나 누르면 버튼이 사라져요** | **어긋남** |

- **어느 쪽이 작은지**: **스토리를 고치는 쪽이 훨씬 작아요** (니로, 문장 두 개). 아이가 실제로 노는 화면(플레이)에서는 차이가 없고, 차이는 "세로로 연 첫 화면"과 "터치 노트북"뿐이에요. 고칠 문장 예:
  - US-035: "터치 기기에서 화면을 **한 번 만진 뒤** 세로가 더 길면 안내가 나온다. 처음 열었을 때 세로이면 첫 톡에서 안내가 나오고, 그 톡으로 화면이 넘어가지 않는다."
  - US-036 참고: "터치와 키보드가 둘 다 있는 기기는 **마지막에 쓴 것**을 따른다: 화면을 만지면 버튼이 보이고, 키를 누르면 숨는다."
- **코드로 맞추고 싶다면** (가족이 "세로로 열자마자 안내가 나와야 해"라고 하면): `web/index.html`에 `loading_done`처럼 `importObject.env.is_touch_device = () => navigator.maxTouchPoints > 0 ? 1 : 0` 를 넣고, `view/web.rs`에 extern 하나, `main.rs`에서 `touch_mode` 처음 값을 그걸로 두면 돼요(10줄 안팎, 로건). 이렇게 하면 "키를 누르면 끔"은 터치 노트북에서만 의미가 있으니, US-036 참고대로 하려면 그 줄도 빼야 해요.
- **고칠 사람**: 니로 (스토리). 코드로 갈 경우 로건

### 3. 요즘 폰처럼 가로로 긴 화면에서 엄지를 왼쪽 끝에 대면 조이스틱이 반쯤 잘려 안 보여요 (로건)

- **무엇이 문제인지**: 조이스틱 받침은 손가락을 댄 자리(가상 좌표)에 그대로 생기는데, 그림은 가상 화면(960×540) 안에만 그려져요. 화면 양옆 띠(레터박스)나 아래쪽 끝에 손가락을 대면 받침 대부분이 그려지지 않아요. (1번을 고친 뒤 이야기)
- **어떤 경우에 틀리는지**: 아이폰 12 Pro 가로(844×390)는 양옆에 띠가 75px씩 있어요. 엄지를 편하게 (40, 370)에 대면 가상 (-49, 512) → 반지름 85인 받침이 왼쪽으로 36px만, 아래로는 28px만 보여요. 걷기는 되지만 US-032 "그 자리에 조이스틱이 생긴다", "안쪽 동그라미가 손가락을 따라간다"를 엘리가 볼 수 없어요.
- **가장 작은 고침**: 조이스틱을 만들 때 받침 가운데를 화면 안으로 당겨요. 예: `base = f.pos.clamp(vec2(R, R), vec2(VIRTUAL_WIDTH - R, VIRTUAL_HEIGHT - R))` (R = `JOYSTICK_RADIUS`). 손잡이는 지금처럼 손가락 쪽으로 반지름까지 밀려서 방향은 그대로 맞아요. 테스트: "(-50, 520)에 대면 받침이 화면 안".
- **고칠 사람**: 로건

### 4. 조이스틱 크기·데드존·버튼 크기가 `config.rs`가 아니라 `touch.rs`에 있어요 (로건 또는 니로)

- **무엇이 문제인지**: 스토리 머리말에 "수치(조이스틱 크기, 조금만 밀면 안 움직이는 거리, 버튼 크기·자리)는 `game/src/config.rs`에 둔다"고 적혀 있는데, `JOYSTICK_RADIUS`, `JOYSTICK_DEADZONE`, `BUTTON_RADIUS` 등이 `touch.rs` 위쪽에 있어요.
- **어떤 경우에 틀리는지**: 아이와 "조이스틱 더 크게 해 볼까?", "살짝 밀어도 걷게 해 줘"를 바꿀 때 `config.rs`를 열면 없어요. (프로젝트에서 그림 자리 상수는 view 파일에 두는 것이 지금까지의 방식이라, 어느 쪽이 틀렸다기보다 스토리와 코드가 다른 상태예요.)
- **가장 작은 고침**: 둘 중 하나. (가) 아이가 바꿀 만한 세 개(`JOYSTICK_RADIUS`, `JOYSTICK_DEADZONE`, `BUTTON_RADIUS`)만 `config.rs`로 옮기기(로건). (나) 니로가 스토리 문장을 "그림 자리 수치는 `view/touch.rs` 위쪽에 모은다"로 바꾸기.
- **고칠 사람**: 로건 또는 니로 (하나만)

### 5. US-031 "타이틀 구석에 버전 표시"가 아직 없어요 (로건) — 커밋을 막지는 않음

- **무엇이 문제인지**: `build.sh`가 `window.GAME_VERSION`과 콘솔에는 버전을 남기지만, 게임 화면에서 그 값을 읽어 그리는 코드가 아직 없어요(`grep GAME_VERSION game/src` → 없음).
- **어떤 경우에 틀리는지**: 엘리가 US-031 "새로고침만 해도 버전 표시가 바뀐다", "(실제 폰) 링크를 다시 누르면 버전 표시가 새 것"을 확인할 방법이 없어요(폰에는 콘솔이 없음).
- **가장 작은 고침**: `index.html`에서 `importObject.env`로 버전 문자열을 넘기거나, `web.rs`에 버전 길이·바이트를 받는 함수 하나. 또는 더 쉽게, 빌드 때 `build.sh`가 `GAME_VERSION`을 환경 변수로 넘기고 Rust에서 `option_env!("GAME_VERSION").unwrap_or("dev")`로 읽어 타이틀 구석에 그리기 (JS 연결이 필요 없음).
- **고칠 사람**: 로건 (빌드에 환경 변수 넘기는 줄은 케빈)

## 작은 것

### 6. 한 프레임 사이에 같은 손가락 번호가 떼졌다가 다시 쓰이면 새 손가락으로 못 알아봐요 (로건)

안드로이드 크롬은 뗀 손가락의 번호(id)를 바로 다음 손가락에 다시 줘요. 조이스틱 엄지(id 0)를 떼고 **같은 1/60초 안에** 오른손으로 점프 버튼을 누르면, macroquad의 손가락 목록에는 id 0이 `Started`로 한 번만 남아요(떼었다는 기록이 덮어써짐). 지금 코드는 "지난 프레임에 있던 id"면 새 손가락이 아니라고 봐서, 이 터치가 **점프가 아니라 조이스틱 이동**이 돼요(오른쪽으로 걸어감). 아주 드물어요. 고침: `update`에서 `f.phase == TouchPhase::Started`이면 그 id를 먼저 놓고(조이스틱·held에서 빼고) 새 손가락으로 처리. 테스트 하나: 조이스틱 id 1 → 다음 프레임 id 1 `Started`를 점프 자리에 → `jump_pressed`.

### 7. CI에서 `cargo test`가 두 번 돌아요 (케빈)

`deploy.yml`의 "테스트" 단계와 `build.sh` 첫 줄이 둘 다 `cargo test`를 해요. 두 번째는 캐시 덕에 몇 초라 해롭지는 않아요. 줄이고 싶으면 워크플로의 테스트 단계를 빼도 `build.sh`가 실패 시 멈춰 줘요(단, Actions 화면에서 "테스트 실패"로 따로 안 보이게 됨). 그대로 둬도 괜찮아요.

### 8. GitHub Pages는 `index.html`도 10분 동안 캐시해요 — 엘리 확인 방법에 적어 두기 (니로·엘리)

GitHub Pages는 모든 파일에 "10분 동안 다시 묻지 않아도 됨"(`Cache-Control: max-age=600`)을 붙여요. F5 새로고침은 서버에 다시 물어서 괜찮지만, US-031 "(실제 폰) 탭을 닫았다가 링크를 다시 누르면 새 버전"은 **배포 뒤 10분 안**이면 옛 `index.html`(옛 `?v=`)을 쓸 수 있어요. 코드로 막기 어려우니, 스토리에 "배포 뒤 10분 지나서 확인"을 적어 두면 엘리가 헷갈리지 않아요. 같은 이유로 `assets/`의 그림·글꼴은 `?v=`가 없어서, 그림을 바꾼 버전은 최대 10분 옛 그림이 보일 수 있어요(지금 스토리 범위 밖).

### 9. `build.sh`의 `sed`는 GNU sed 전용 문법(`\n`)을 써요 (케빈) — 지금은 문제없음

세 번째 `-e`의 바꿀 글자에 들어간 `\n`은 GNU sed(우분투 러너, Git Bash)에서는 줄바꿈이지만, 맥의 BSD sed에서는 글자 `n`이 돼요. 그래도 바로 아래 `grep` 검사가 `window.GAME_VERSION = "..."`만 찾으므로 맥에서는 줄이 붙은 채 통과해요(브라우저에서는 그래도 동작함). 지금 빌드하는 곳은 윈도 Git Bash와 우분투뿐이라 고칠 필요는 없고, 나중에 맥에서 빌드하게 되면 떠올리면 돼요. 버전 글자는 16진수·숫자·`-`뿐이라 `|`, `&` 같은 sed 특수 글자가 들어갈 일은 없어요.

### 10. `touch.rs` 테스트의 "항상 같은 값" assert 2개 (로건)

`buttons_are_big_and_on_screen`의 `assert!(JOYSTICK_RADIUS * 2.0 >= 110.0)`, `assert!(BACK_BUTTON.y >= 66.0)`는 clippy가 "컴파일할 때 이미 정해진 값"이라고 경고해요. REVIEW-002 6번과 같은 종류라, 할 때 `const { assert!(...) }`로 한꺼번에 바꾸면 돼요.

## 확인한 것 (문제 없음)

- **같은 프레임에 대고 떼기**: macroquad는 같은 id의 마지막 상태만 남겨서 `Ended`로만 보일 수 있어요. 점프 버튼은 `Ended`여도 새 손가락이면 점프하고(테스트 `quick_tap_within_one_frame_still_jumps`), 조이스틱은 뗀 손가락으로는 안 만들어요. 메뉴에서는 `is_mouse_button_pressed`가 같은 프레임에 켜지므로 톡이 놓치지 않아요.
- **손가락 3개**: 조이스틱은 하나만, 버튼 손가락은 여러 개 기억(`held`). 왼쪽에 둘째 손가락을 대도 조이스틱을 뺏지 않아요(테스트 있음). 조이스틱 엄지를 떼면, 왼쪽에 대고 있던 다른 손가락이 이어받지는 않아요(다시 대야 함) — 스토리에 그런 조건은 없어서 괜찮아요.
- **창 크기 바뀜·돌리기 중**: 세로가 되는 순간 `idle()`로 조이스틱을 놓고, 가로로 돌아오면 그때 닿아 있던 손가락은 무시해요. 그래서 돌리는 동안 좌표 기준이 바뀌어도 이상하게 걷지 않아요. 앱 전환 때는 macroquad가 모든 손가락을 `Ended`로 바꿔 줘요(`window_minimized_event`).
- **흉내 낸 마우스(`simulate_mouse_with_touch`)가 플레이로 새는지**: 플레이에서는 `menu.tap`을 안 써요. 마우스 "눌림"은 그 프레임에만 켜지고 다음 프레임에 지워져서, 타이틀 톡이 고르기 화면 카드를 고르지 않고, 카드 톡이 플레이 점프가 되지 않고(`idle` + 테스트 `finger_held_from_menu_is_ignored_in_play`), "◀ 고르기" 톡이 고르기 화면 카드를 고르지 않아요. 또 `mq_js_bundle.js`의 터치 이벤트가 모두 `preventDefault()`를 해서, 브라우저가 톡 뒤에 가짜 `mousedown`을 한 번 더 보내는 일도 없어요.
- **세로 안내 중 시간 멈춤이 logic 규칙을 우회하는지**: 안 해요. logic(`Game::update`, `update_with_time_scale`)을 **부르지 않을 뿐**이라, dt=0으로 멈춘 것과 같아요. 시계·코인 다시 나오기·캐릭터 모두 dt로만 움직이고, 안내 중에도 프레임은 계속 돌아서 돌아온 첫 프레임 dt도 보통 크기(그리고 `cap_frame_seconds`로 상한 있음)예요. 코인 반짝 효과(view)도 같이 멈춰요.
- **키보드 동작**: `merge_touch`는 키보드 OR 터치라서 ←/→/Space/↑는 그대로예요. Esc(`menu.back`)·F 빨리감기도 그대로. 키보드만 쓰는 PC는 손가락 목록이 늘 비어서 터치 모드가 꺼진 채이고, 세로 안내도 안 나와요(테스트 `rotate_hint_only_for_touch_portrait`). 마우스로 왼쪽을 끌어도 `touches()`에 안 들어와서 조이스틱이 안 생겨요.
- **`draw_play`·`draw_pill` 바뀐 곳**: `draw_play`를 부르는 곳은 `main.rs:177` 하나, `draw_pill`은 `pub(super)`로만 넓혔어요.
- **logic/view 경계**: `touch.rs`에 `use crate::logic` 없음. 세로 안내 판정·터치 모드 판정은 화면 쪽 일이라 view에 있는 게 맞아요. Wasm에서 안 되는 것(스레드, 파일, `Instant`) 없음. `#[allow(dead_code)]` 없음.
- **아이 눈높이**: 새 문구 "점프", "고르기", "가로로 돌려 주세요" 모두 쉬운 말. 안내 글자 크기는 창 너비의 10%(28~72px)라 세로 폰(390px)에서 39px — US-035의 24px 이상 조건을 넘어요.
- **워크플로 YAML**: 권한 `contents: read`, `pages: write`, `id-token: write`는 Pages 배포에 필요한 최소한이에요. `concurrency: pages` + `cancel-in-progress: false`는 GitHub 공식 예시와 같아요. `deploy`는 `environment: github-pages`라서 main에서만 배포돼요. 비밀 값(토큰 등)은 하나도 없어요(`id-token`은 GitHub가 그때그때 만들어 줌).
- **Pages 하위 경로**: 주소가 `.../Niro_First_Project/`인데, `index.html`의 `mq_js_bundle.js`, `miss-game.wasm`과 게임의 `assets/...`가 모두 **상대 주소**(앞에 `/` 없음)라 하위 경로에서도 맞아요. `dist/`에는 점으로 시작하는 파일이 없어서(`.gitkeep` 지움) `upload-pages-artifact@v5`가 빼먹는 파일도 없어요. `dist/`는 `.gitignore`에 있어요.
- **`?v=` 꼬리**: `fetch("miss-game.wasm?v=...")`는 GitHub Pages가 `.wasm`에 `application/wasm`을 붙여 줘서 `compileStreaming`이 그대로 돼요. CI의 `checkout`은 얕은 받기여도 `git rev-parse --short HEAD`가 돼요.

## 부모님께 질문 (코드 문제 아님)

이번 배포로 게임 주소 `https://nirokimyijuni.github.io/Niro_First_Project/`가 누구나 열 수 있게 돼요(저장소도 공개). 주소에 들어간 GitHub 아이디가 아이 이름과 비슷해 보이는데, 아이 이름이 주소에 드러나도 괜찮은지 한 번만 확인해 주세요. 괜찮으면 그대로 두면 돼요.

---

2번(스토리 문장)은 같은 커밋에 넣으면 좋고, 3~5번은 다음 작업으로 해도 돼요.

판정: 1번 고친 뒤 커밋

---

## 재확인 (2026-11-02)

고친 것을 `git diff`로 다시 봤어요. `build.sh`는 실행하지 않았어요.

| 번호 | 고친 것 | 결과 |
|---|---|---|
| 1 | `input.rs` `touch_to_virtual(physical, dpi, lb)` = `lb.to_virtual(physical / dpi)`. `fingers()`가 이걸 써요 | **맞아요.** 조이스틱·점프 버튼·"◀ 고르기"는 모두 `fingers()` 한 곳에서 나온 `Finger.pos`만 써서 한꺼번에 고쳐졌어요. 세로 안내는 손가락 위치가 아니라 `screen_width/height`(이미 나눈 값)만 보고, 메뉴는 `mouse_position()`(이미 나눈 값)이라 원래 맞아요. 단위가 섞인 곳이 더 없어요. 테스트 `jump_button_works_at_dpi_3`, `same_spot_gives_same_result_at_dpi_1_and_3` 있음(`Letterbox::for_window`로 GPU 없이 확인) |
| 2 | US-035 "한 번 만진 뒤", US-036 "마지막에 쓴 것을 따른다" | **맞아요.** 코드 동작(`next_touch_mode`)과 같아요 |
| 3 | `joystick_base`로 받침을 화면 안으로 당김 | **맞아요.** 손잡이는 받침 기준으로 계산해서 방향이 그대로예요. 테스트 `joystick_base_stays_on_screen_near_edges` 있음 |
| 4 | 스토리 문장을 "`view/touch.rs` 맨 위 상수"로 바꿈 (나 방법) | **맞아요.** 코드와 스토리가 같아졌어요 |
| 5 | `index.html` `game_version_byte` → `web.rs` `game_version()` → 타이틀 오른쪽 아래 `v ...` | **맞아요.** `window.GAME_VERSION`이 없으면(로컬) -1 → `None` → 안 그려요. Wasm 빌드 성공 |
| 6 | 같은 id가 `Started`로 다시 오면 예전 손가락을 놓고 새 손가락으로 처리 | **맞아요.** macroquad는 `Started`를 한 프레임 뒤 `Stationary`로 바꾸므로(`lib.rs:451`), 누르고 있는 손가락이 매 프레임 "새 손가락"이 되는 일은 없어요. 테스트 `reused_finger_id_started_again_is_a_new_finger` 있음 |
| 8 | US-031 "배포 끝나고 10분 지난 뒤" + 확인 메모 | **맞아요** |
| 10 | `const { assert!(...) }`로 바꿈 | **맞아요** |

**`web/index.html` ↔ `build.sh` sed**: `load("miss-game.wasm")`은 60번 줄에 한 번, `<script src="mq_js_bundle.js"></script>`는 45번 줄에 한 번만 그대로 있어요. 새로 넣은 `game_version_byte` 함수에는 두 글자가 들어 있지 않아서 sed 세 줄과 부딪히지 않고, `build.sh` 끝의 `grep` 검사도 그대로 통과할 모양이에요. 또 `window.GAME_VERSION`은 sed가 `mq_js_bundle.js` **앞**에 넣으므로 게임이 읽을 때는 이미 있어요.

**돌려 본 것** (별도 target 폴더)
- `cargo test`: 통과 (86 + 59 + 51 + 58개, 3개는 꺼 둔 것)
- `cargo build --target wasm32-unknown-unknown`: 성공

새로 생긴 문제는 없어요. 7·9번은 "그대로 둬도 됨"이라 그대로예요.

판정: 커밋해도 좋아요
