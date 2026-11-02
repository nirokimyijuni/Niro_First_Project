# 미쓰 찾기 게임 — 프로젝트 안내

8살 아이와 함께 만드는 **Rust + WebAssembly 2D 생존 게임**. 브라우저에서 링크로 바로 플레이한다.

## 꼭 지킬 것
- 아이가 **함께 만드는 사람**이다. 결정이 필요한 재미 요소(몬스터 이름, 색깔, 아이템)는 아이에게 물어볼 거리로 남긴다.
- 게임 안 문구는 **쉬운 한글**로, 무섭거나 잔인한 표현은 쓰지 않는다 (피 표현 X, "쓰러졌어요" 정도).
- 원본 지침: [docs/01-game-guide.md](docs/01-game-guide.md) / 개발 계획: [docs/02-plan.md](docs/02-plan.md)
- **`docs/02-plan.md`의 "결정 사항"이 원본 지침보다 우선한다.** (예: 낮/밤 시간, 체력, 몬스터 공격력)
- 캐릭터 원화는 [art/concept/](art/concept/)에 둔다. 아이가 새로 그린 그림도 여기에 넣는다.
- 게임 수치(낮/밤 시간, 공격 횟수, 코인 상한 등)는 코드에 박지 않고 `game/src/config.rs` 한 곳에 모은다.

## 팀 (서브에이전트, `.claude/agents/`)
| 에이전트 | 역할 | 주로 일하는 곳 |
|---|---|---|
| `niro` 니로 | 스토리텔러 · 유저스토리 관리 | `stories/` |
| `susanna` 수산나 | 백엔드 (게임 규칙·로직) | `game/src/logic/` |
| `logan` 로건 | 프론트 (그림·입력·UI·사운드) | `game/src/view/`, `game/assets/`, `web/` |
| `bob` 밥 | 코드 리뷰어 (고치지 않고 읽고 지적) | `qa/reviews/` |
| `ellie` 엘리 | 테스터 | `qa/`, `game/tests/` |
| `kevin` 케빈 | 배포 관리 | `deploy/`, `.github/workflows/` |

> 에이전트 이름(니로·엘리·로건)과 게임 캐릭터 이름이 같다. 문서에서 캐릭터는 **캐릭터 Niro**, 에이전트는 **니로(에이전트)** 처럼 구분해서 쓴다.

## 작업 흐름
1. 니로가 유저스토리를 `stories/backlog.md`에 쓴다 (완료 조건 포함).
2. 수산나(로직)와 로건(화면)이 구현한다. 둘의 경계는 `game/src/logic` ↔ `game/src/view`.
3. 밥이 코드를 리뷰하고 `qa/reviews/`에 남긴다. 꼭 고칠 것은 수산나·로건이 고친다.
4. 엘리가 완료 조건대로 확인하고 버그는 `qa/bugs/`에 남긴다.
5. 케빈이 웹에 올리고 플레이 링크를 공유한다.

## 기술 기본값
- 엔진: `macroquad` (Wasm 빌드가 가장 단순함). 빌드 타깃 `wasm32-unknown-unknown`.
- 배포: GitHub Pages (정적 파일 `index.html` + `.wasm`).
