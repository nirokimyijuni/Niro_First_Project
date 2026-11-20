# AWS에 게임 올리기 — 부모님 따라 하기 안내서

> 관리: 케빈(에이전트). 부모 결정 2026-11-20: AWS 계정 있음, 도메인은 나중(`cloudfront.net` 주소), GitHub Pages는 그대로 두고 **두 곳 다** 배포.
> 이 안내서는 **AWS 콘솔(웹 화면)만** 씁니다. 명령어(CLI)는 필요 없어요.
> 케빈은 AWS에 아무것도 만들지 않습니다. 만들기는 모두 부모님이 직접 합니다.

## 무엇이 만들어지나요?

템플릿 파일 하나([game-hosting.yaml](game-hosting.yaml))를 올리면 **스택(stack) 하나**로 아래가 한꺼번에 만들어집니다. 지울 때도 스택 하나만 지우면 됩니다.

| 만들어지는 것 | 하는 일 |
|---|---|
| S3 버킷 (공개 접근 차단) | 게임 파일을 넣어 두는 창고. 인터넷에서 직접 못 엶 |
| CloudFront 배포 + OAC | `https://xxxx.cloudfront.net` 주소로 게임을 보여 줌. 창고는 CloudFront만 열 수 있음 |
| GitHub OIDC 공급자 + 배포 역할 | GitHub Actions가 **비밀 키 없이** 들어와서 이 버킷에 파일 올리기·캐시 비우기**만** 할 수 있음. 이 저장소의 `main` 브랜치만 |
| 월 예산 알림 (AWS Budgets) | 이번 달 비용이 $5를 넘을 것 같으면 메일. **알림만** 하고 자동으로 멈추지는 않음 |

## 예상 비용 (월 1,000번 열기 기준)

지금 게임(`dist/`)은 약 4MB, 파일 66개입니다. 한 번 열 때 다 받는다고 넉넉히 잡으면:

| 항목 | 한 달 사용량 | 무료 한도 (항상 무료) | 예상 요금 |
|---|---|---|---|
| CloudFront 데이터 전송 | 약 4GB (4MB × 1,000) | 1TB | $0 |
| CloudFront 요청 | 약 66,000번 | 1,000만 번 | $0 |
| CloudFront 캐시 비우기 | 배포 1번에 경로 1개(`/*`) | 1,000개 | $0 |
| S3 → CloudFront 전송 | 약 4GB | S3 요금 없음 | $0 |
| S3 저장 | 약 4MB | — | 1센트 미만 |
| S3 올리기(PUT) | 배포 20번 × 약 66개 ≈ 1,300번 | — | 1센트 미만 |
| AWS Budgets 알림 | 예산 1개 | 알림만 쓰면 무료 | $0 |

**예상 합계: 한 달 1센트 안팎 (사실상 $0).** 1,000번이 아니라 10만 번 열어도 약 400GB라 CloudFront 무료 한도 안입니다.

가격 출처 (2026-11-20 확인):
- CloudFront 종량제: <https://aws.amazon.com/cloudfront/pricing/pay-as-you-go/> — 항상 무료 1TB 전송·1,000만 요청, 무효화 1,000경로 무료(그 뒤 경로당 $0.005). 한국은 아시아 태평양 묶음, 1TB 넘은 뒤 GB당 $0.120.
- S3: <https://aws.amazon.com/s3/pricing/> — S3에서 CloudFront로 가는 전송은 요금 없음. 서울 리전 저장·요청 단가는 페이지 표가 열리지 않아 확인 못 함(미국 동부 기준 GET 1,000번당 $0.0004). 이 게임 크기에서는 몇 센트도 안 됩니다.
- AWS Budgets: <https://aws.amazon.com/aws-cost-management/aws-budgets/pricing/> — 알림만 하는 예산은 무료.

---

## 시작 전에 (5분)

1. **루트 계정 말고 관리자 사용자로 로그인하세요.** 루트 계정(처음 가입한 메일)은 열쇠 꾸러미 전체라서 평소에는 쓰지 않는 게 안전합니다. 관리자 사용자가 없다면 IAM Identity Center 또는 IAM에서 하나 만들어 두세요.
2. **MFA(2단계 인증)를 켜 두세요.** 루트 계정과 관리자 사용자 둘 다. 콘솔 오른쪽 위 계정 이름 → **Security credentials**(보안 자격 증명) → **Multi-factor authentication (MFA)** → **Assign MFA device**.
3. 오른쪽 위 리전을 **Asia Pacific (Seoul) ap-northeast-2**(아시아 태평양 서울)로 바꿉니다.
4. 이 저장소의 [deploy/aws/game-hosting.yaml](game-hosting.yaml) 파일을 컴퓨터에 받아 둡니다. (GitHub에서 파일 열기 → 오른쪽 위 **Download raw file** 아이콘. 아직 커밋 전이면 이 PC의 `D:\...\미쓰 찾기 게임\deploy\aws\game-hosting.yaml`을 씁니다.)

## 1단계. 이미 GitHub OIDC 공급자가 있는지 보기 (1분)

계정에 하나만 만들 수 있어서, 있으면 템플릿에 "이미 있음"이라고 알려 줘야 합니다.

1. 위쪽 검색창에 **IAM** 입력 → IAM 열기.
2. 왼쪽 메뉴 **Identity providers**(자격 증명 공급자).
3. 목록에 `token.actions.githubusercontent.com`이 있나요?
   - **없다** → 2단계에서 `CreateOidcProvider`를 `true`로 둡니다 (기본값).
   - **있다** → 그 줄을 눌러 **ARN**(`arn:aws:iam::...:oidc-provider/token.actions.githubusercontent.com`)을 복사해 두고, 2단계에서 `CreateOidcProvider`를 `false`, `ExistingOidcProviderArn`에 그 ARN을 넣습니다.

## 2단계. CloudFormation으로 스택 만들기 (10~15분, 대부분 기다리기)

1. 검색창에 **CloudFormation** 입력 → 열기. 리전이 **Seoul**인지 다시 확인.
2. **Create stack**(스택 생성) → **With new resources (standard)**(새 리소스 사용(표준)).
3. **Prepare template**: **Choose an existing template**(기존 템플릿 선택).
   **Specify template**: **Upload a template file**(템플릿 파일 업로드) → **Choose file** → `game-hosting.yaml` 선택 → **Next**(다음).
4. **Stack name**(스택 이름): `miss-game-hosting`
5. **Parameters**(파라미터) — 아래처럼 넣습니다.

   | 이름 | 넣을 값 |
   |---|---|
   | `GitHubOwner` | `nirokimyijuni` (기본값 그대로) |
   | `GitHubRepo` | `Niro_First_Project` (기본값 그대로) |
   | `CreateOidcProvider` | 1단계 결과대로 `true` 또는 `false` |
   | `ExistingOidcProviderArn` | `false`일 때만 1단계에서 복사한 ARN. 아니면 비워 둠 |
   | `CreateBudget` | `true` |
   | `MonthlyBudgetUsd` | `5` |
   | `BudgetAlertEmail` | **부모님이 알림 받을 메일을 직접 입력** (비워 두면 예산이 안 만들어짐) |

   → **Next**.
6. **Configure stack options**(스택 옵션 구성): 아무것도 바꾸지 않고 맨 아래로.
   **Capabilities**(기능)에 나오는 체크 상자 **"I acknowledge that AWS CloudFormation might create IAM resources."**(IAM 리소스를 만들 수 있음을 인정) 에 체크 → **Next**.
   (배포 역할을 만들기 때문에 꼭 필요합니다. 이 역할이 할 수 있는 일은 이 버킷 쓰기와 이 CloudFront 캐시 비우기뿐입니다.)
7. **Review**(검토) 화면에서 값 확인 → **Submit**(제출).
8. **Events**(이벤트) 탭에서 진행을 봅니다. CloudFront 때문에 5~15분 걸립니다. 위쪽 상태가 **CREATE_COMPLETE**(생성 완료, 초록색)가 되면 성공.
   - **ROLLBACK_COMPLETE**(빨간색)이 되면 실패입니다. **Events** 탭에서 빨간 줄의 **Status reason**(상태 이유)을 복사해서 케빈에게 알려 주세요. 실패한 스택은 **Delete**로 지우고 다시 하면 됩니다(돈 안 듭니다).
   - 예산(`MonthlyBudget`) 때문에 실패하면: `CreateBudget`을 `false`로 바꿔 다시 만들고, 예산은 4단계의 "수동으로 만들기"를 따라 하세요.

## 3단계. 결과 값 복사해서 GitHub에 넣기 (5분)

1. 만든 스택 → **Outputs**(출력) 탭. 아래 4개가 보입니다.

   | Outputs의 키 | GitHub 변수 이름 |
   |---|---|
   | `DeployRoleArn` | `AWS_DEPLOY_ROLE_ARN` |
   | `BucketName` | `AWS_S3_BUCKET` |
   | `DistributionId` | `AWS_CLOUDFRONT_ID` |
   | `DistributionDomain` | `AWS_CLOUDFRONT_DOMAIN` |

   이 값들은 **비밀번호가 아닙니다**(이름표 같은 것). 비밀 키는 하나도 없어요. 케빈에게 알려 주셔도 괜찮습니다.
2. GitHub 저장소 `nirokimyijuni/Niro_First_Project` → **Settings**(설정) → 왼쪽 **Secrets and variables** → **Actions** → 위쪽 **Variables**(변수) 탭 → **New repository variable**(새 저장소 변수).
3. 위 표대로 4개를 하나씩 만듭니다 (**Name**에 변수 이름, **Value**에 Outputs 값 → **Add variable**).
   - 넣는 곳은 **Secrets가 아니라 Variables** 탭입니다.
   - `AWS_DEPLOY_ROLE_ARN`은 **맨 마지막에** 넣으세요. 이 변수가 생기는 순간 AWS 배포가 켜집니다.
4. (선택) 서울이 아닌 리전을 썼다면 `AWS_REGION` 변수도 만듭니다. 서울이면 안 만들어도 됩니다.

## 4단계. 예산 알림 확인 (2분)

1. 검색창 **Budgets** → **AWS Budgets** → `miss-game-hosting-monthly`가 있으면 끝.
2. **수동으로 만들기**(2단계에서 `CreateBudget=false`였을 때만): **Create budget** → **Use a template (simplified)**(템플릿 사용) → **Monthly cost budget**(월 비용 예산) → 금액 `5` → 받을 메일 입력 → **Create budget**.

## 5단계. 처음 파일 올리기 — 부모님은 안 해도 됩니다

파일 올리기는 GitHub Actions가 합니다. 케빈이 `deploy.yml`의 AWS 단계를 main에 올리면(출시 기준을 지킨 다음 커밋에서), 그 뒤로 main에 push할 때마다:

1. 시험(`cargo test`)·버그 확인·빌드 → GitHub Pages 배포 (지금과 같음)
2. Pages가 성공하면 같은 파일을 S3에 올리고 CloudFront 캐시를 비움
3. `https://<DistributionDomain>/`에서 버전이 맞는지, `.wasm` 종류가 `application/wasm`인지 자동으로 확인

당장 해 보고 싶으면: 저장소 **Actions** → **Deploy to GitHub Pages** → **Run workflow**(main). 단, 이것도 Pages 재배포를 포함합니다.

## 6단계. 잘 됐는지 확인 (1분)

1. 브라우저에서 `https://<DistributionDomain>/` 를 엽니다 (예: `https://d1234abcd.cloudfront.net/`).
2. 게임이 뜨면 성공. 버전까지 보려면 F12 → **Console**(콘솔)에서 `게임 버전: ...` 줄이 GitHub Pages 주소의 것과 같은지 봅니다.
3. GitHub **Actions** → 그 실행 → 맨 아래 **Summary**(요약)에 `AWS 플레이 주소: ...`와 `deploy-aws=success`가 보입니다.
4. **이 주소를 다른 사람에게 보내는 건 부모님이 정할 일입니다.** 케빈은 묻지 않고 공유하지 않아요.

---

## 되돌리기 (AWS를 그만 쓰고 싶을 때)

순서가 중요합니다.

1. **GitHub 먼저 끄기**: 저장소 Settings → Variables에서 `AWS_DEPLOY_ROLE_ARN` 삭제. 그러면 다음 배포부터 AWS 단계는 건너뜁니다(GitHub Pages는 계속).
2. **버킷 비우기**: 검색창 **S3** → 버킷(`miss-game-hosting-sitebucket-...`) 선택 → **Empty**(비우기) → 안내 글자 `permanently delete` 입력 → **Empty**. (버킷이 비어 있지 않으면 스택이 지워지지 않습니다.)
3. **스택 지우기**: **CloudFormation** → `miss-game-hosting` → **Delete**(삭제) → 확인. 버킷·CloudFront·역할·OIDC 공급자·예산이 함께 지워집니다. CloudFront는 끄는 데 15분쯤 걸려요.
   - 1단계에서 OIDC 공급자가 원래 있었다면(`false`로 했다면) 그건 이 스택 것이 아니라서 남습니다. 정상입니다.
4. 남은 GitHub 변수 3개(`AWS_S3_BUCKET`, `AWS_CLOUDFRONT_ID`, `AWS_CLOUDFRONT_DOMAIN`)도 지웁니다.

게임 내용을 예전 버전으로 되돌리는 건 지금처럼 케빈이 `git revert`로 다시 배포합니다. 그러면 Pages와 AWS가 같이 바뀝니다.

## 케빈이 할 일

- [ ] 부모님이 3단계 값을 넣었다고 알려 주시면, `deploy.yml`의 AWS 단계를 다음 릴리스 커밋에 함께 올린다 (출시 기준 확인 뒤).
- [ ] 첫 AWS 배포 뒤 공개 주소에서 `index.html` 버전, `.wasm` Content-Type, 그림 파일을 직접 확인한다.
- [ ] `deploy/RELEASES.md` "배포 시도 기록"에 AWS 결과(`deploy-aws=...`)도 적는다.
- [ ] 한 달 뒤 Billing(청구) 금액을 부모님께 여쭤 이 문서의 예상 비용과 비교한다.

## 알아 둘 위험

- **예산은 알림만 합니다.** 누가 주소를 아주 많이 열면 요금이 생길 수 있어요(1TB 넘으면 GB당 약 $0.12). 지금 크기로는 25만 번 넘게 열려야 1TB입니다.
- **역할 범위**: 배포 역할은 `nirokimyijuni/Niro_First_Project`의 `main` 브랜치에서만, 이 버킷과 이 CloudFront에만 쓸 수 있습니다. 다른 브랜치·다른 저장소·PR에서는 거부됩니다.
- `deploy-aws` 작업에 `environment:`를 붙이면 역할이 거부합니다 (GitHub가 보내는 이름표 모양이 바뀜). 바꾸려면 템플릿의 조건도 같이 바꿔야 해요.
- 버킷 이름이 들어간 주소(`...s3.amazonaws.com`)로는 게임이 안 열립니다. 정상입니다(공개 차단). 주소는 늘 `cloudfront.net` 쪽을 씁니다.
- 없는 주소(예: `https://<주소>/abc`)를 열면 영어로 된 **403 AccessDenied** 화면이 나와요. 정상이에요(창고 목록을 숨겨 둬서 404 대신 이렇게 보여요).
- 배포 직후 1분쯤은 새 버전과 옛 버전이 섞여 보일 수 있어요(예: 버전 글자는 새것인데 게임은 옛 것). 잠시 뒤 새로고침하면 맞아져요.
- CloudFront는 파일을 잠시 저장해 둡니다. 배포 때마다 자동으로 비우지만, 실패하면 옛 버전이 보일 수 있어요. 그때는 Actions 실행 결과를 케빈에게 보여 주세요.
