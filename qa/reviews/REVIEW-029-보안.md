# REVIEW-029 보안 검토

- 리뷰어: 밥(에이전트) / 날짜: 2026-11-21
- 부모님 요청: "보안에 문제 없을지 한번 검토해보자"
- 본 것: 작업 폴더 그대로 (커밋 안 된 변경 포함). `deploy/aws/game-hosting.yaml`, `.github/workflows/deploy.yml`, `deploy/build.sh`, `web/index.html`, `web/about.html`, `game/src/main.rs`·`game/src/view/web.rs`·`game/src/view/qa.rs`(검색), `.gitignore`, git에 이미 커밋된 기록(`git grep`, `git log --all -p`), 커밋 안 된 새 파일.

## 이 검토가 하는 일

게임은 그림·글자·wasm 파일만 있는 "정적 사이트"(서버 프로그램이 없는 사이트)라서 해커가 노릴 곳이 적어요. 그래서 이번 검토는 세 가지를 봤어요: (1) AWS에 올리는 열쇠(배포 역할)가 이 저장소 main 브랜치에만 열리는지, (2) GitHub 자동 배포(Actions)가 남의 코드에 열쇠를 넘기지 않는지, (3) 공개 저장소에 비밀 값이나 아이 개인정보가 올라가지 않았는지.

## 잘한 점

- **AWS 열쇠가 키 없이(OIDC) 들어온다.** 저장소에 AWS 비밀 키가 하나도 없어요. 신뢰 정책은 `aud = sts.amazonaws.com`과 `sub`를 와일드카드(`*`) 없이 **main 브랜치 두 모양만** 정확히 비교해요. 다른 브랜치·포크·PR에서는 역할을 못 써요.
- **배포 역할 권한이 아주 좁다.** 이 버킷 하나에 목록 보기·올리기·지우기, 이 CloudFront 배포 하나에 캐시 비우기만 돼요. `*` 자원이 없어요. 버킷 읽기·삭제 권한이나 IAM 권한은 없어요.
- **S3 버킷이 꽁꽁 닫혀 있다.** 공개 차단 4개 모두 켜짐, ACL 끔(BucketOwnerEnforced), 암호화(AES256), 버킷 정책은 "이 CloudFront 배포(OAC)만 읽기" + "HTTPS 아닌 접근 거부". 기록상 S3 직접 주소는 403으로 막힌 것도 확인됐어요.
- **CloudFront가 HTTPS로 돌린다** (`redirect-to-https`), 내 도메인일 때 TLS 1.2 이상(`TLSv1.2_2021`), GET·HEAD만 받아요.
- **예산 알림 메일 주소가 파일에 없다** (콘솔에서 파라미터로만 넣음).
- **워크플로에 스크립트 주입이 없다.** `run:` 안에 `${{ }}`로 들어가는 값은 `needs.*.result`, `github.sha`처럼 바깥 사람이 못 바꾸는 값뿐이고, 저장소 변수(vars)는 `env:`로 넘겨요. `pull_request` 트리거가 없어서 포크 PR이 배포 작업을 돌릴 수 없어요.
- **게임은 네트워크 요청을 하지 않는다.** 게임 코드에 `fetch`·웹소켓·외부 주소가 없고, 외부 스크립트(광고·통계·CDN)도 없어요. 저장하는 것은 브라우저 `localStorage`의 숫자 두 개(최고 날 수, 최고 점수)뿐이고 개인정보가 아니에요. 값이 이상하면 0으로 시작하게 막혀 있어요.
- **비밀 값 없음.** 커밋된 파일·기록 전체에서 AWS 키(`AKIA…`), GitHub 토큰(`ghp_`, `github_pat_`), SpriteCook 키(`sc_live_`), 개인 키, 12자리 AWS 계정 번호를 찾았는데 하나도 없었어요. 커밋 작성자 메일도 GitHub noreply 주소예요. `.claude/settings.local.json`은 `.gitignore`에 있어요.

---

## 꼭 고칠 것

### 1. 아이 실명이 들어간 폴더 경로가 공개 저장소에 있다 (일부는 이미 커밋됨) — 케빈(파일), 부모님(기록 지우기 결정)
- **무엇이 문제**: 작업 폴더 경로(`D:\<실명>\...`)에 아이 실명이 있어요. 이 경로가 파일 안에 그대로 적힌 곳이 있어요.
  - 이미 커밋되어 공개됨: `.claude/launch.json` 7·13번째 줄 (커밋 92e27e0, db3b986부터), `qa/reviews/REVIEW-002-그래픽-업그레이드.md` 52번째 줄 (커밋 71c7184).
  - 아직 커밋 안 됨: `art/icons/make_icons.py` 5번째 줄 `ROOT = r'D:\<실명>\...'`.
- **이렇게 틀려요**: 누구나 GitHub에서 `.claude/launch.json`을 열면 실명을 볼 수 있어요. `make_icons.py`를 지금 모양 그대로 커밋하면 한 곳이 더 늘어요.
- **가장 작은 고침**:
  - `make_icons.py`: 절대 경로 대신 스크립트 위치에서 찾기 (`ROOT = Path(__file__).resolve().parents[2]`) 로 바꾼 뒤 커밋.
  - `.claude/launch.json`: 절대 경로를 상대 경로(`dist`, `tools/dashboard/server.py`)로 바꾸기. 
  - `REVIEW-002` 52번째 줄: 경로를 `D:\<실명>`처럼 가리기.
  - 이미 기록(history)에 남은 것은 파일을 고쳐도 옛 커밋에서 보여요. 완전히 지우려면 기록 다시 쓰기(git filter-repo + 강제 push)가 필요한데, 배포·링크에 영향이 있으니 **부모님이 정할 일**이에요. 지우지 않기로 해도 위 세 파일은 고치는 게 좋아요.

### 2. `book/` 폴더가 `.gitignore`에 없다 — 케빈
- **무엇이 문제**: `book/`은 커밋하지 않는 폴더인데 `.gitignore`에 없어요 (`git check-ignore book/` 결과 없음). 지금은 커밋 안 된 상태(`??`)예요.
- **이렇게 틀려요**: 누군가 `git add -A`나 `git add .`로 커밋하면 `book/decisions.md`, `book/story.md`가 공개 저장소에 올라가요.
- **가장 작은 고침**: `.gitignore`에 `book/` 한 줄 추가.

---

## 고치면 좋은 것

### 3. 빌드 작업(build)도 AWS·Pages 열쇠를 받을 수 있다 — 케빈
- **무엇이 문제**: 워크플로 맨 위 `permissions:`에 `id-token: write`와 `pages: write`가 있어서 **build 작업에도** 적용돼요. build 작업은 남이 만든 코드(`dtolnay/rust-toolchain`, `Swatinem/rust-cache`, `pip install fonttools`, cargo 의존성 빌드)를 가장 많이 돌리는 곳이에요.
- **이렇게 틀려요**: 그중 하나가 해킹되면 build 작업 안에서 OIDC 토큰을 받을 수 있어요. main에서 돈 토큰이라 AWS 배포 역할의 조건(main 브랜치)을 통과해서, 버킷의 게임 파일을 다른 것으로 바꿔 올릴 수 있어요.
- **가장 작은 고침**: 맨 위는 `permissions: contents: read`만 두고, `deploy` 작업에 `pages: write`, `id-token: write`를 따로 적기 (`deploy-aws`는 이미 따로 적혀 있어요). `upload-pages-artifact`는 `pages: write`가 필요 없어요.

### 4. 외부 액션이 SHA가 아니라 태그·브랜치로 고정돼 있다 — 케빈
- **무엇이 문제**: `actions/checkout@v7`, `Swatinem/rust-cache@v2`, `aws-actions/configure-aws-credentials@v4` 등은 태그라서 주인이 태그를 옮기면 다른 코드가 돌아요. 특히 `dtolnay/rust-toolchain@stable`은 **브랜치**라 매번 최신 코드가 돌아요.
- **이렇게 틀려요**: 2025년 `tj-actions/changed-files` 사건처럼 태그가 나쁜 코드로 옮겨지면 다음 main push 때 그 코드가 우리 작업에서 돌아요 (3번과 겹치면 AWS 역할까지).
- **가장 작은 고침**: 적어도 GitHub 공식(`actions/*`)이 아닌 것(`dtolnay`, `Swatinem`, `aws-actions`)은 커밋 SHA로 고정하고 옆에 `# v2.x` 주석. 업데이트는 Dependabot(`.github/dependabot.yml`, `package-ecosystem: github-actions`)으로 받기.

### 5. CloudFront에 보안 응답 헤더 정책이 없다 — 케빈
- **무엇이 문제**: `ResponseHeadersPolicyId`가 없어서 HSTS(늘 HTTPS로 오라는 표시), `X-Content-Type-Options: nosniff`, `X-Frame-Options`(다른 사이트가 게임을 틀(iframe)에 넣지 못하게) 같은 헤더가 안 붙어요.
- **이렇게 틀려요**: 다른 사이트가 우리 게임을 iframe 안에 넣어 자기 광고와 함께 보여 줄 수 있어요. 처음 `http://`로 들어올 때 한 번은 암호 없이 가요(바로 https로 돌려보내지만).
- **가장 작은 고침**: `DefaultCacheBehavior`에 AWS 관리형 정책 "SecurityHeadersPolicy" `ResponseHeadersPolicyId: 67f7725c-6f97-4210-82d7-5512b31e9d03` 한 줄. (GitHub Pages 쪽은 헤더를 못 바꿔요. 그쪽은 그대로 둬도 돼요.) 부모님이 스택 업데이트를 해야 적용돼요.

### 6. 페이지에 CSP(불러와도 되는 곳 목록)가 없다 — 로건
- **무엇이 문제**: `web/index.html`, `web/about.html`에 Content-Security-Policy가 없어요.
- **이렇게 틀려요**: 지금은 외부 스크립트도 사용자 입력도 없어서 실제 위험은 낮아요. 나중에 누가 실수로 외부 스크립트(통계 등)를 넣어도 막아 주는 장치가 없어요.
- **가장 작은 고침**: `<meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:">` 정도. wasm·인라인 스크립트가 있어서 **엘리가 브라우저에서 게임이 열리는지 꼭 확인**해야 해요. 급하지 않아요.

---

## 괜찮음 (확인만)

7. **`?qa` 시험 단축**: 실제 배포에서도 주소 끝에 `?qa`를 붙이면 열려요. 하는 일은 **그 사람 브라우저 안의 게임**에서 코인·재료 채우기, 날 넘기기, 로고 건너뛰기뿐이에요. 서버가 없어서 다른 사람 게임이나 저장소에는 영향이 없어요. 이 모드로 놀면 그 브라우저의 최고 기록이 커질 수 있지만 자기 기기 안의 숫자일 뿐이에요. `?debug`는 화면 크기·터치 위치만 보여 줘요. 보안 문제 아님.
8. **CloudFront 기본 주소(`*.cloudfront.net`)의 TLS 최소 버전**: 기본 인증서는 AWS가 정해서 바꿀 수 없어요. 내 도메인 주소는 TLS 1.2 이상이라 괜찮아요.
9. **GitHub 숫자 ID, Route 53 별칭 존 ID(`Z2FDTNDATAQYW2`), cloudfront.net 주소**: 모두 공개 정보예요. AWS 계정 번호가 아니에요. 템플릿의 ARN 패턴은 `[0-9]{12}` 모양만 있고 진짜 번호는 없어요.
10. **S3 버전 관리 꺼짐**: 혹시 잘못 올라가도 git에서 다시 빌드해 올리면 돼요. 켜면 옛 파일 요금이 쌓여서 지금처럼 꺼 두는 게 맞아요.
11. **`workflow_dispatch`**: 쓰기 권한 있는 사람만 누를 수 있고, main이 아닌 브랜치에서 누르면 AWS 역할이 거절해요.
12. **about.html 내용**: 실명·학교·사는 곳 없음. "아이", "가족", 캐릭터 이름(Niro, Ellie, Logan)만 나와요.
13. **예산 알림**: 50%·100%(실제), 100%(예상) 세 번 메일. 단, 메일 칸이 비면 예산이 **조용히 안 만들어져요** (조건 `MakeBudget`). 아래 콘솔 확인 목록에 넣었어요.

## 이번 검토와 따로 (참고)

- **GitHub 사용자 이름**: 저장소 주인 이름 자체에 아이 이름을 로마자로 쓴 것으로 보이는 부분이 있어요. 저장소 주소·Pages 주소·템플릿 기본값에 모두 들어가요. 바꾸려면 GitHub 계정 이름 변경(주소가 바뀌고 OIDC 조건도 바뀜)이라 큰 일이에요. 괜찮은지 **부모님이 정할 일**로만 적어 둬요.

## 못 본 것 (15분 안에)

- `art/concept/`, `art/icons/`, `web/og-image.png` 등 **그림 안의 내용**(아이 손글씨 이름, 사진 속 정보)은 열어 보지 않았어요. 아이가 그린 그림에 이름을 써 넣었는지 부모님이 한 번 봐 주세요.
- `book/` 안의 글 내용, `tools/dashboard/`(배포 안 되는 로컬 도구)는 자세히 보지 않았어요.
- 실제 AWS 콘솔·GitHub 설정 상태는 코드로 확인할 수 없어요 (아래 목록).
- 이번은 보안 검토라 `cargo test`·wasm 빌드는 돌리지 않았어요.

---

## 부모님이 콘솔에서 직접 확인할 것

**AWS**
- [ ] 루트 계정(가입한 메일로 들어가는 계정)에 **MFA**(휴대폰 인증 앱 등 2단계 인증)가 켜져 있나요? (IAM → 보안 권장 사항)
- [ ] 루트 계정에 **액세스 키가 없나요**? 있으면 지워요.
- [ ] 평소에는 루트 대신 **관리자 사용자(IAM Identity Center 또는 IAM 사용자) + MFA**로 들어가나요?
- [ ] **Budgets**에 `miss-game-hosting-monthly`가 있고, 알림 메일 구독 확인 메일을 눌렀나요? (메일 칸이 비었으면 예산이 안 만들어졌어요)
- [ ] Billing → **결제 알림/Free Tier 알림** 메일 받기가 켜져 있나요?
- [ ] **Route 53 → 등록된 도메인**: 연락처 **개인정보 보호(Privacy protection)** 켜짐, **이전 잠금(Transfer lock)** 켜짐, 자동 갱신 켜짐.
- [ ] IAM → 역할에서 이번 스택이 만든 배포 역할 말고 쓰지 않는 역할·사용자·액세스 키가 남아 있지 않나요?
- [ ] (선택) CloudTrail 기본 이벤트 기록이 켜져 있나요? (무료, 무슨 일이 있었는지 나중에 볼 수 있어요)

**GitHub**
- [ ] 계정에 **2단계 인증(2FA)** 이 켜져 있고, 복구 코드를 안전한 곳에 적어 두었나요?
- [ ] 계정 설정 → Emails → **"Keep my email addresses private"** 켜짐 (지금 커밋 메일은 noreply라 잘 되어 있어요).
- [ ] 저장소 Settings → Branches(또는 Rulesets): **main 강제 push·삭제 막기**.
- [ ] Settings → Actions → General: **포크 PR 워크플로는 승인 후 실행**, 기본 `GITHUB_TOKEN` 권한은 **읽기 전용**.
- [ ] Settings → Environments → `github-pages`: 배포 브랜치가 **main만**인지.
- [ ] Settings → Code security: **Secret scanning·Push protection** 켜기 (공개 저장소는 무료).
- [ ] 계정 → Developer settings → Personal access tokens: 안 쓰는 토큰 지우기. 여러 계정을 바꿔 쓰는 PC라 `gh auth status`로 어떤 계정이 로그인돼 있는지도 가끔 확인.

**기타**
- [ ] SpriteCook API 키는 저장소에 없어요(확인함). 키가 들어 있는 PC 설정 파일을 남에게 보내지 않기.

---

## 묶음 정리

- 꼭 고칠 것: **1**(실명 경로), **2**(`book/` 무시 목록)
- 고치면 좋은 것: 3(build 작업 권한), 4(액션 SHA 고정), 5(CloudFront 보안 헤더), 6(CSP)
- 괜찮음: 7~13

판정: 1, 2번 고친 뒤 커밋
