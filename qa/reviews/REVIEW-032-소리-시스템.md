# REVIEW-032 소리 시스템 (US-112~117)

리뷰: 밥 / 2026-11-21 / 아직 커밋 전 변경(git diff + 새 파일 sound.rs, audio.rs, build.rs)

## 이 변경이 하는 일
브라우저에서 효과음 39개와 음악 4곡을 내는 소리 시스템이에요. 수산나가 "어떤 소리를 언제 얼마나"(logic/sound.rs)를, 로건이 "받아서 틀기"(view/audio.rs, main.rs, build.rs, index.html)를 맡았어요. 소리 부품(quad-snd)은 브라우저(wasm) 빌드에만 들어가요.

## 잘한 점
- logic 쪽 변경은 `mod.rs` 한 줄, `config.rs` 덧붙임(55줄 추가, 지운 줄 0), 새 파일 `sound.rs`뿐이에요. 게임 규칙·밤 몬스터 계산은 안 건드렸어요 (git diff로 확인).
- 소리 부품을 `target.'cfg(target_arch = "wasm32")'`에만 넣어서 PC 시험과 리눅스 CI가 알사(libasound)를 안 받아도 돼요.
- 못 받은 소리는 조용히 건너뛰고, 로그 한 줄만 남겨요. 소리가 없어도 게임은 그대로 돌아가요.
- 설정 저장은 값이 바뀔 때만 (`with_sound != settings`), 겹침 막기(같은 소리 0.05초, 0.25초에 4개)와 한 프레임 16개 한도가 있어요.

## 돌려 본 결과 (실제 출력)
- `cargo test`: 198 passed, 0 failed
- `cargo build --target wasm32-unknown-unknown` (dev, release 둘 다): 성공
- `cargo clippy --all-targets`: 경고 41개가 나오지만 sound.rs, audio.rs, pause.rs, main.rs의 새 줄에서 나온 것은 없어요 (눈에 띈 것은 옛 시험의 `is_multiple_of` 제안)
- `cargo audit`: 이 PC에 없어서 **못 했어요.** 알려진 문제 검사는 못 봤어요.

## 1. 공급망·빌드
- 새로 들어온 크레이트 11개 (Cargo.lock에서 새로 생긴 [[package]] 목록): audir-sles, audrey, dasp_frame, dasp_sample, hound, lewton, maybe-uninit, ogg, quad-alsa-sys, quad-snd 0.2.8, smallvec. 전부 `registry+crates.io` 출처이고 git 의존성은 없어요.
- 빌드 스크립트(build.rs)가 있는 것은 quad-snd(맥/iOS에서만 CoreAudio 링크, 우리 wasm에는 아무 일 없음)와 maybe-uninit(옛 크레이트, 버전 확인용). proc-macro는 새로 없어요 (macroquad_macro는 이미 있던 것).
- `--allow-undefined` (game/build.rs): `CARGO_CFG_TARGET_ARCH == "wasm32"`일 때만, `rustc-link-arg-bins`라서 우리 실행 파일에만 붙어요. PC·시험 빌드에는 영향 없어요. 위험은 "다른 함수가 진짜 빠져도 링크가 조용히 넘어간다"인데, `web/mq_js_bundle.js`가 이미 "빠진 함수는 경고 찍고 빈 함수로 채우기"(`add_missing_functions_stabs`)를 해서 게임이 안 켜지는 일은 없어요. 대신 빠진 소리 함수는 무음이 돼요. 번들에는 소리 함수 9개(audio_init 등)가 모두 있는 것을 확인했어요.
- CI(deploy.yml, ubuntu, `cargo test` → deploy/build.sh): 소리 부품은 wasm에서만 받으니 코드 읽기로는 문제 없어 보여요. 다만 실제 CI에서 돌려 보지는 못했어요(로컬 윈도우 빌드만).

## 2. 웹 (index.html)
- 핸들러 모두 `addEventListener(..., true)` 캡처 단계에서 `resume()`만 부르고, `preventDefault`·`stopPropagation`을 안 써서 게임 입력을 막지도 새게 하지도 않아요. 게임 번들의 touchend `preventDefault`가 가짜 click을 막으므로 touchend를 듣는 것은 옳아요.
- 모든 `resume/suspend`에 try/catch와 `.catch`가 있고, 저장·개인정보는 없어요. 등록은 한 번(즉시실행 함수)이고 contexts 배열은 많아야 2개(번들이 하나 더 만듦)라 새지 않아요.
- 작은 것: `window.__soundContexts`를 "확인용"으로 열어 둔 것 (해는 없어요).

## 3. 로직·화면
- 이벤트 → 소리 연결은 빠진 곳을 못 찾았어요. 멈춤 창을 열 때/누를 때는 막힌 효과음 때문에 `audio.play_sfx`로 바로 내고, 닫을 때는 mixer로 내서 중복이 없어요. 스피커 탭은 `menu.tap = None`으로 지워서 장면이 안 넘어가요.
- 멈춤 창 버튼 110 → 86: 폰 화면 값은 86×(360/540)=57로, 문턱을 70→55로 낮췄지만 손가락 크기 44는 넘어요. 새 시험 `sound_buttons_fit...`가 모든 버튼에 44 이상(높이 기준, 너비도 740/960 기준)·안 겹침·창 안쪽을 따로 확인해서 시험이 느슨해졌다고 보지는 않아요.

## 꼭 고칠 것
없음.

## 고치면 좋은 것
1. **15초 시간 초과가 "내려받기"에는 안 걸려요** (view/audio.rs `load_all`). `load_file(...).await`는 시간 제한이 없고, 15초는 그 뒤 "풀기(디코딩)"에만 걸려요. 네트워크가 멈추면(응답이 영영 안 옴) 그 파일에서 받기가 멈추고 뒤 소리들이 모두 안 들려요. 게임은 계속 돌아요. 작은 고침: 문서·주석을 "풀기 15초"로 정확히 쓰거나, 폰 약한 망을 고려한다면 파일마다 시작 시각을 재서 넘기기(단, `load_file`은 취소가 안 되어 어려움). 고칠 사람: 로건.
2. **`#![allow(dead_code)]`가 sound.rs에 남아 있어요** (12번째 줄). 주석이 "연결하면 지워도 된다"고 하는데 이미 연결됐어요. 지우고 안 쓰는 함수가 있으면 같이 정리하거나 `config`·시험용으로 표시. 고칠 사람: 수산나.
3. **소리가 꺼진 채 크기 칸을 누르면 소리는 안 켜지고 칸 색도 안 바뀌어요** (main.rs 멈춤 창, pause.rs `i == chosen && on`). 아이가 "작게"를 눌렀는데 아무 일도 없는 것처럼 보일 수 있어요. 작은 고침: 크기 칸을 누르면 `sound.set_on(true)`도 함께 (또는 꺼져 있어도 고른 칸은 옅게 표시). 고칠 사람: 로건.

## 작은 것
4. 저장된 기본 음악 70%는 4칸(0/25/50/80)에 없어서 "크게"가 골라진 것처럼 보여요 (`volume_cell_for`가 가장 가까운 칸). 기본값을 80이나 50으로 바꾸면 맞아요 (아이·부모에게 물어볼 거리).
5. 캐는 소리의 재료 구분이 "동굴이면 철, 아니면 나무"예요 (main.rs). 동굴 밖 돌을 캘 수 있다면 나무 소리가 날 수 있어요. 돌을 어디서 캐는지는 확인하지 못했어요 (질문).
6. 멈춤 창 위쪽이 화면에서 4px밖에 안 떨어져 빡빡해 보일 수 있어요 (PANEL y=4). 눈으로 확인은 엘리가.
7. `Waker::noop()`은 러스트 1.85 이상이 필요해요. CI는 `stable`이라 괜찮고, 로컬이 옛 버전이면 오류가 나요.

## 4. 저장
소리 값은 바뀔 때만 저장해요. `?qa`에서는 `save_settings`/`load_settings`가 `qa_mode()`를 보고 읽지도 쓰지도 않는 것을 web.rs에서 확인했어요.

## 5. 군더더기·규칙
- 규칙 숫자는 대부분 config.rs에 있어요. 화면 쪽 숫자(스피커 크기, 알림 글 크기, 볼륨 변화 0.002)는 모양값이라 괜찮다고 봐요.
- 문구는 쉬운 한글이고 무섭지 않아요 ("소리를 켰어요/껐어요", "소리 켜짐/꺼짐").

## 6. 라이선스
- `game/assets/sounds/CREDITS.md`: 표 줄이 43개 (음악 4 + 효과음 39). 실제 파일 4 + 39와 일대일로 맞고 (빠진 파일, 출처 없는 파일 모두 0), 43줄 모두 CC0예요. (파일 전체는 설명 줄 포함 52줄이라 "43줄"은 표 줄 기준이에요.)
- 출처 사이트(OpenGameArt, Kenney)가 정말 CC0인지는 **코드로는 확인 못 해요.** 문서의 "부모님이 License.txt 확인" 기록을 믿는 수밖에 없어요. 파일 총 크기는 3.8MB로 5MB 한도 아래예요.

## 못 본 것
- `cargo audit` (설치 안 됨), 실제 CI 실행, 브라우저·폰에서 소리 나는지(엘리 몫), 소리 크기·겹침을 귀로 듣는 것.

판정: 커밋해도 좋아요 (고치면 좋은 것 1~3번은 다음에 같이 해도 돼요)
