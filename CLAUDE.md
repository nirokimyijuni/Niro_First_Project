# 미쓰 찾기 게임 — 프로젝트 안내

8살 아이와 함께 만드는 **Rust + WebAssembly 2D 생존 게임**. 브라우저에서 링크로 바로 플레이한다.

## 꼭 지킬 것
- 아이가 **함께 만드는 사람**이다. 결정이 필요한 재미 요소(몬스터 이름, 색깔, 아이템)는 아이에게 물어볼 거리로 남긴다.
- 아이는 Claude, GitHub, SpriteCook을 직접 쓰지 않는다. 아이가 말하면 부모가 입력한다. 게임과 문서에는 별명만 쓴다. 전체 원칙: [docs/05-agent-rnr.md](docs/05-agent-rnr.md)
- 게임 안 문구는 **쉬운 한글**로, 무섭거나 잔인한 표현은 쓰지 않는다 (피 표현 X, "쓰러졌어요" 정도).
- 원본 지침: [docs/01-game-guide.md](docs/01-game-guide.md) / 개발 계획: [docs/02-plan.md](docs/02-plan.md)
- **`docs/02-plan.md`의 "결정 사항"이 원본 지침보다 우선한다.** (예: 낮/밤 시간, 체력, 몬스터 공격력)
- 캐릭터 원화는 [art/concept/](art/concept/)에 둔다. 아이가 새로 그린 그림도 여기에 넣는다.
- 게임 수치(낮/밤 시간, 공격 횟수, 코인 상한 등)는 코드에 박지 않고 `game/src/config.rs` 한 곳에 모은다.

## 팀 (서브에이전트, `.claude/agents/`)
| 에이전트 | 역할 | 주로 일하는 곳 |
|---|---|---|
| 감독 (메인 Claude) | 일 나누기와 보고. 코드 수정, 버그 종결, 리뷰 판정, 스토리 상태 변경은 하지 않는다 | `qa/gates.md` |
| `niro` 니로 | 스토리텔러 · 유저스토리 관리 | `stories/` |
| `susanna` 수산나 | 백엔드 (게임 규칙·로직) | `game/src/logic/` |
| `logan` 로건 | 프론트 (그림·입력·UI·사운드) | `game/src/view/`, `game/src/main.rs`, `game/assets/`, `web/` |
| `hailey` 헤일리 | 캐릭터 디자이너 (SpriteCook으로 그림 만들기) | `art/spritecook/` |
| `bob` 밥 | 코드 리뷰어 (고치지 않고 읽고 지적) | `qa/reviews/` |
| `ellie` 엘리 | 테스터 | `qa/`, `game/tests/` |
| `kevin` 케빈 | 배포 관리 | `deploy/`, `.github/workflows/`, `tools/dashboard/` |

> 에이전트 이름(니로·엘리·로건)과 게임 캐릭터 이름이 같다. 문서에서 캐릭터는 **캐릭터 Niro**, 에이전트는 **니로(에이전트)** 처럼 구분해서 쓴다.

## 작업 흐름
1. 니로가 유저스토리를 `stories/backlog.md`에 쓴다 (완료 조건 포함).
2. 수산나(로직)와 로건(화면)이 구현한다. 둘의 경계는 `game/src/logic` ↔ `game/src/view`.
3. 밥이 코드를 리뷰하고 `qa/reviews/`에 남긴다. 꼭 고칠 것은 수산나·로건이 고친다.
4. 엘리가 완료 조건대로 확인하고 버그는 `qa/bugs/`에 남긴다.
5. 케빈이 웹에 올리고 플레이 링크를 공유한다. main에 push하면 바로 배포된다.

출시 기준: 이번 릴리스의 스토리가 모두 엘리 통과, 열린 S1·S2 버그 0개, `cargo test`와 wasm 빌드 통과. 그 밖의 예외는 부모 승인을 `deploy/RELEASES.md`에 남긴다. 자세한 역할과 심각도: [docs/05-agent-rnr.md](docs/05-agent-rnr.md)

## 시험이 멈추지 않게
- 시험 코드에 "될 때까지 반복"을 쓰지 않는다. `while`·`loop`에는 횟수 제한을 넣어, 넘으면 실패로 끝나게 한다.
- `cargo test`는 `timeout 300 cargo test`처럼 시간 제한을 걸고 돌린다. 시간을 넘기면 기다리지 말고 그 시험을 끄고, 멈췄다고 바로 보고한다.

## 리뷰와 시험을 가볍게 (부모 승인 2026-11-08)
- 몇 줄짜리 작은 고침은 밥의 재리뷰 없이 엘리가 시험으로만 확인한다. 큰 변경은 지금처럼 밥과 엘리가 둘 다 본다.
- 일부러 코드를 망가뜨려 보는 시험은 규칙(logic)이 크게 바뀔 때만 한다.
- 엘리의 브라우저 확인은 화면(view, main.rs, web)이 바뀔 때만 한다. 규칙만 바뀌면 `cargo test`로 확인한다.
- 리뷰와 시험은 각각 15분 안에 끝낸다. 못 본 것은 못 봤다고 적는다.
- 감독은 기능마다 에이전트를 새로 시작한다. 긴 대화를 이어 쓰지 않는다.
- 출시 기준은 바뀌지 않는다.

## 파일 고치기
- 파일은 Edit·Write 도구로 고친다. 이 PC의 `python`은 아무것도 하지 않는 윈도우 가짜 명령이라, 셸에서 `python`으로 쓴 수정은 조용히 사라진다. 꼭 필요하면 `/c/Users/SDS/AppData/Local/Programs/Python/Python313/python.exe`를 쓴다.
- 고쳤다고 보고하기 전에 grep으로 파일에 실제로 들어갔는지 확인한다.

## 빌드 폴더
- 빌드 결과물은 기본 위치인 `game/target`에 둔다. `CARGO_TARGET_DIR`을 임시 폴더(스크래치)나 C 드라이브로 지정하지 않는다. C 드라이브가 꽉 찬다.
- 동시에 빌드해야 하면 `CARGO_TARGET_DIR=target-<이름>`으로 `game/target-<이름>`을 쓴다 (`.gitignore`에 이미 있다).
- 다 쓴 `game/target-<이름>`은 작업이 끝나면 지운다.

## 기술 기본값
- 엔진: `macroquad` (Wasm 빌드가 가장 단순함). 빌드 타깃 `wasm32-unknown-unknown`.
- 배포: GitHub Pages (정적 파일 `index.html` + `.wasm`).
