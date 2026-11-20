# REVIEW-027 AWS 호스팅 (CloudFormation 템플릿 + deploy-aws 작업 + 부모님 안내서)

- 리뷰: 밥(에이전트), 2026-11-20
- 본 파일: `deploy/aws/game-hosting.yaml`, `.github/workflows/deploy.yml`(deploy-aws, record), `deploy/aws/README.md`
- 아직 커밋 전(작업 폴더 변경). 게임 코드(`game/`)는 바뀌지 않아 `cargo test`·wasm 빌드는 돌리지 않았다.
- 문법 검사 도구(cfn-lint, actionlint)는 설치하지 않았다. 이 PC 파이썬에 PyYAML도 없어서 **모두 눈으로** 봤다.

## 이 변경이 하는 일
GitHub Pages는 그대로 두고, 같은 `dist/`를 AWS(S3 창고 + CloudFront 주소)에도 올린다. AWS 쪽은 부모님이 콘솔에서 템플릿 하나로 만들고, GitHub Actions는 비밀 키 없이(OIDC = GitHub가 "나는 이 저장소 main이에요"라는 이름표를 AWS에 보여 주는 방식) 들어와 파일 올리기와 캐시 비우기만 한다. 저장소 변수 `AWS_DEPLOY_ROLE_ARN`이 없으면 이 작업은 건너뛴다.

## 잘한 점
- **권한이 아주 좁다.** 신뢰 정책이 `aud = sts.amazonaws.com`, `sub = repo:nirokimyijuni/Niro_First_Project:ref:refs/heads/main`을 `StringEquals`(와일드카드 없음)로 묶어서 다른 브랜치·PR·다른 저장소는 거부된다. 역할 권한도 `ListBucket`은 버킷 ARN만, `PutObject/DeleteObject`는 `버킷/*`만, `CreateInvalidation/GetInvalidation`은 이 배포 ARN만이다. `s3 sync`와 `wait invalidation-completed`에 필요한 것이 딱 들어 있고 남는 것이 없다.
- **두 길(CreateOidcProvider true/false)이 모두 맞다.** `!If [MakeOidcProvider, !Ref GitHubOidcProvider, !Ref ExistingOidcProviderArn]`에서 조건부 리소스를 `!If` 안에서만 참조하므로 false일 때도 템플릿 오류가 없다. `!Ref` OIDCProvider는 ARN을 돌려준다.
- 버킷 정책의 CloudFront 허용은 `AWS:SourceArn`으로 **이 배포 하나**에만 열려 있고, 공개 차단 4개를 모두 켠 상태에서도 통과하는 모양(서비스 주체 + SourceArn)이다. `DenyNonHttps`는 뷰어가 `redirect-to-https`라 CloudFront가 S3에 HTTPS로 가므로 CloudFront 읽기를 막지 않는다.
- OAC 설정(`S3OriginConfig.OriginAccessIdentity: ""` + `OriginAccessControlId`, `RegionalDomainName`), `DefaultRootObject: index.html`, 관리형 CachingOptimized ID(`658327ea-...`)가 맞다. 리소스 사이 순환 참조도 없다(버킷 → OAC → 배포 → 버킷 정책·역할).
- 워크플로: 작업 단위 `permissions`(contents: read, id-token: write), `environment:`를 붙이지 않은 이유를 주석으로 남김, 변수 없으면 `if`로 건너뛰고 `record`는 `always()`라 그대로 기록, 공개 주소 확인 반복 10번 제한 + 작업 `timeout-minutes: 20`, 무효화 대기도 AWS 쪽 횟수 제한(20초×30번)이 있다. `.wasm`은 `application/wasm`으로 못 박고 공개 주소에서 실제 헤더까지 확인한다. 올리는 순서도 "나머지 → wasm → index.html 마지막"이라 반쯤 올라간 상태에서 새 index가 먼저 보이지 않는다.
- 비밀 정보: 12자리 계정 번호·메일 주소가 세 파일 어디에도 없다(grep 확인). 메일은 콘솔 파라미터로만 받는다.
- 안내서 단계가 템플릿과 맞다: 파라미터 7개 이름·기본값, Outputs 4개 ↔ 변수 4개, 버킷 이름 모양(`miss-game-hosting-sitebucket-...`), 예산 이름(`miss-game-hosting-monthly`), IAM 확인 체크 상자(역할 이름을 정하지 않아 `CAPABILITY_IAM`이면 충분), 지울 때 버킷 먼저 비우기(DeletionPolicy: Delete와 일치).

## 꼭 고칠 것
없음.

## 고치면 좋은 것
1. **`CreateOidcProvider=false`인데 ARN을 비워 두면 알아보기 힘든 실패가 난다.**
   - 어떤 경우: 1단계에서 공급자가 "있다"를 보고 `false`로 바꿨지만 `ExistingOidcProviderArn` 칸을 깜빡함 → `Federated: ""`로 역할 만들기가 실패하고, CloudFront를 5~15분 기다린 뒤 `ROLLBACK_COMPLETE`가 된다. 오류 문구도 IAM 정책 문법 오류처럼 보여 부모님이 원인을 찾기 어렵다.
   - 가장 작은 고침: 템플릿에 `Rules`를 넣어 스택 만들기 **전에** 막는다.
     ```yaml
     Rules:
       ExistingArnNeededWhenNotCreating:
         RuleCondition: !Equals [!Ref CreateOidcProvider, "false"]
         Assertions:
           - Assert: !Not [!Equals [!Ref ExistingOidcProviderArn, ""]]
             AssertDescription: CreateOidcProvider가 false이면 ExistingOidcProviderArn을 넣어 주세요
     ```
   - 고칠 사람: 케빈.

## 작은 것
2. **찾는 파일이 없을 때 404가 아니라 403(AccessDenied XML)이 나온다.** CloudFront(OAC)에 `s3:ListBucket`을 주지 않았기 때문이다(보안상 맞는 선택). 예: `https://<주소>/abc` → 영어 XML 화면. 게임은 한 페이지라 실제로 볼 일은 거의 없다. 바꾸지 않아도 되고, 안내서 "알아 둘 위험"에 "없는 주소는 AccessDenied 글자가 나와요, 정상"이라고 한 줄 넣으면 부모님이 놀라지 않는다. 고칠 사람: 케빈.
3. **CachingOptimized는 `?v=버전` 꼬리를 캐시 열쇠에 넣지 않는다.** `build.sh`가 `miss-game.wasm?v=...`로 캐시를 깨려 하지만 CloudFront 가장자리에서는 꼬리가 무시된다. 배포 직후 무효화가 끝나기 전 1분 남짓 동안 새 index.html(no-cache라 금방 새것) + 옛 wasm이 섞여 보일 수 있다(예: 버전 표시는 새것인데 게임은 옛 것). 지금은 무효화를 기다리고 확인까지 하므로 짧은 순간뿐이다. 바꾸려면 쿼리 문자열을 캐시 열쇠에 넣는 관리형 정책(`UseOriginCacheControlHeaders-QueryStrings`)으로 바꾸는 방법이 있다. 급하지 않음. 고칠 사람: 케빈.
4. 질문: `actions/download-artifact@v4`가 `upload-pages-artifact@v5`가 올린 `github-pages`(안의 `artifact.tar`)를 그대로 받는 것은 첫 실행에서 확인해 주세요. 둘 다 같은 아티팩트 저장소(v4 이후)를 써서 될 것으로 보지만, 이 PC에서 돌려 볼 수 없어 확인하지 못했다.

## 못 본 것
- 실제 AWS 콘솔에 올려 보기, cfn-lint·actionlint 같은 도구 검사(설치하지 않기로 함). 첫 스택 만들기와 첫 deploy-aws 실행이 사실상의 시험이다.
- `AWS::Budgets::Budget`이 서울 리전 스택에서 만들어지는지는 문서로 확인하지 못했다. 안내서에 실패 시 `CreateBudget=false` + 수동 만들기 길이 있어 막히지는 않는다.

판정: 커밋해도 좋아요 (1번은 부모님이 콘솔에 올리기 전에 넣으면 더 좋아요)
