# REVIEW-028 내 도메인 붙이기 (findmis.nirostudio.com)

- 리뷰: 밥(에이전트), 2026-11-21
- 본 파일: `deploy/aws/game-hosting.yaml`, `.github/workflows/deploy.yml`(deploy-aws 공개 주소 확인), `deploy/aws/도메인.md`(새 안내서), `deploy/aws/README.md`·`deploy/seo/README.md` 한 줄씩
- 아직 커밋 전(작업 폴더 변경). 게임 코드(`game/`)는 바뀌지 않아 `cargo test`·wasm 빌드는 돌리지 않았다.
- cfn-lint, actionlint는 이 PC에 없다(`which`로 확인). 템플릿은 **눈으로** 봤다. 대신 아래 두 가지는 직접 돌려 봤다.
  - 워크플로의 `for host in "$CF_DOMAIN" ${PUBLIC_DOMAIN:+"$PUBLIC_DOMAIN"}`: 변수가 비면 `[a.cf]` 하나, 있으면 `[a.cf] [b.com]` 두 개로 도는 것을 bash로 확인.
  - 파라미터 패턴: `findmis.nirostudio.com` 통과, `Findmis.nirostudio.com`(대문자)·`https://findmis.nirostudio.com` 거부, `arn:aws:acm:us-east-1:<12자리>:certificate/<uuid>` 통과를 grep -E로 확인.
- 같은 diff에 섞인 `.webmanifest` 올리기(SEO 쪽 변경)도 훑었다: `build.sh`가 `manifest.webmanifest`를 늘 복사하므로 `for f in dist/*.webmanifest`가 빈 글롭(파일이 없어 `*`가 그대로 남는 것)이 되지 않는다. 문제없음.

## 이 변경이 하는 일
CloudFormation 템플릿에 칸 3개(`DomainName`, `AcmCertificateArn`, `HostedZoneId`)를 더해, 넣으면 CloudFront에 `findmis.nirostudio.com` 이름과 ACM 인증서를 붙이고 Route 53에 A·AAAA 별칭(주소를 CloudFront로 이어 주는 표지판)을 만든다. 셋 다 비우면 지금 스택과 똑같다. 배포 작업은 GitHub 변수 `AWS_PUBLIC_DOMAIN`이 있으면 새 주소에서도 버전과 `.wasm` 종류를 확인한다.

## 잘한 점
- **비웠을 때 지금과 똑같다.** `Aliases`는 `AWS::NoValue`(속성을 아예 빼는 것)로, `ViewerCertificate`는 예전과 같은 `CloudFrontDefaultCertificate: true`로 돌아간다. 새 리소스(레코드 2개)와 출력 `PublicDomain`은 조건이 거짓이면 안 생긴다. 그래서 빈 값으로 업데이트하면 바뀌는 리소스가 없다.
- **교체(Replacement, 지우고 새로 만들기)가 없다.** 바뀌는 리소스는 `SiteDistribution` 하나이고, 바뀌는 속성은 `DistributionConfig` 안의 `Aliases`·`ViewerCertificate`라 "중단 없는 수정"이다. 배포 ID·`cloudfront.net` 주소가 그대로라 `AWS_CLOUDFRONT_ID`·`AWS_CLOUDFRONT_DOMAIN` 변수가 안 깨진다. `SiteBucket`·`SiteOac`·`DeployRole`·OIDC·버킷 정책은 한 글자도 안 바뀌었다. 역할 권한도 배포 ARN 기준이라 그대로 맞다.
- **문법이 맞다.** `!If [조건, 값, !Ref AWS::NoValue]`, `!If`로 `ViewerCertificate` 통째로 고르기(ACM 쪽은 `AcmCertificateArn`+`SslSupportMethod: sni-only`+`MinimumProtocolVersion: TLSv1.2_2021`, 기본 쪽은 `CloudFrontDefaultCertificate: true` 하나 — 두 모양 모두 CloudFront가 요구하는 그대로), `!And` 안 `!Condition UseCustomDomain`, 출력의 `Condition`. 별칭 레코드는 `AliasTarget.HostedZoneId: Z2FDTNDATAQYW2`(CloudFront 고정값), `DNSName: !GetAtt SiteDistribution.DomainName`, `EvaluateTargetHealth: false`로 맞다. 배포에 `IPV6Enabled: true`가 이미 있어 AAAA 레코드도 실제로 쓰인다.
- **실수를 미리 막는다.** `Rules`에서 Fn::Not/Fn::Equals만 써서 규칙 문법 안에 있고, "도메인만", "인증서만", "호스팅 존만" 넣은 경우를 모두 만들기 전에 막는다(호스팅 존 → 도메인 필요 → 인증서 필요로 이어짐). ARN 패턴이 `us-east-1`만 받아 서울 인증서를 넣는 흔한 실수도 막힌다. 각 패턴에 `^$|`가 있어 기본값 `""`도 통과한다.
- 워크플로는 주소마다 10번 제한이 그대로이고, 변수가 없으면 예전과 같은 한 번만 돈다.
- 비밀 정보: 12자리 계정 번호·메일 주소가 다섯 파일 어디에도 없다(grep 확인). 연락처는 콘솔에서만 넣으라고 적혀 있다.
- 안내서 5단계 표의 칸 이름·`PublicDomain` 출력·6단계 변수 이름이 템플릿·워크플로와 정확히 맞다. "Replacement: True가 보이면 멈추기"를 넣은 것도 좋다.

## 꼭 고칠 것
1. **안내서 4단계의 와일드카드 검증 레코드 설명이 틀렸다.** (`deploy/aws/도메인.md` 4단계 8번 아래 줄, 11번)
   - 무엇이 문제: "와일드카드를 같이 넣었으면 ... 검증 레코드는 같은 것 하나라서 하나만 만들어져도 정상"이라고 적혀 있다. ACM에서 `*.nirostudio.com`은 `nirostudio.com`과 검증 레코드를 나눠 쓰지, `findmis.nirostudio.com`과는 다르다. 그래서 `_aaaa.findmis.nirostudio.com`과 `_bbbb.nirostudio.com` **두 줄**이 생긴다. 11번도 `_xxxx.findmis...` 한 줄만 지우지 말라고 한다.
   - 어떤 경우에 틀리나: 부모님이 케빈 추천대로 와일드카드를 넣으면 Route 53에 처음 보는 `_bbbb.nirostudio.com` CNAME이 생긴다. 안내서에 없는 줄이라 정리하다 지우면 지금은 아무 일 없다가, 13개월 뒤 ACM 자동 갱신이 실패하고 인증서가 끝나면 `findmis.nirostudio.com`이 "안전하지 않음"으로 안 열린다. 반대로 한 줄만 생겼다면 "정상"이라고 믿고 넘어가 `Pending validation`에서 멈춘다.
   - 가장 작은 고침: 8번 아래 줄을 "와일드카드를 같이 넣었으면 검증 레코드가 **두 줄**(`_….findmis.nirostudio.com`, `_….nirostudio.com`) 생깁니다. 둘 다 있어야 Issued가 됩니다."로, 11번을 "`_`로 시작하는 검증용 CNAME 줄(한 줄 또는 두 줄)은 모두 지우지 마세요."로 바꾼다.
   - 고칠 사람: 케빈.

## 고치면 좋은 것
2. **"다른 길"로 레코드를 손으로 만든 뒤 나중에 `HostedZoneId`를 넣으면 업데이트가 실패한다.** (`도메인.md` 5단계 끝 "다른 길")
   - 어떤 경우: 처음엔 `HostedZoneId`를 비우고 콘솔에서 `findmis` A·AAAA를 만들었다가, 나중에 스택이 관리하게 하려고 `HostedZoneId`를 넣으면 `DomainRecordA`가 "이미 있음"으로 실패하고 `UPDATE_ROLLBACK_COMPLETE`가 된다(게임은 계속 열리지만 원인을 알기 어렵다).
   - 가장 작은 고침: "다른 길" 끝에 "나중에 템플릿 길로 바꾸려면 손으로 만든 `findmis` A·AAAA 두 줄을 먼저 지운 뒤 `HostedZoneId`를 넣으세요." 한 줄.
   - 고칠 사람: 케빈.

## 작은 것
3. `HostedZoneNeedsDomain`의 문구는 "DomainName과 AcmCertificateArn도"라고 하지만 실제로는 `DomainName`만 확인한다. 도메인을 넣으면 `DomainNeedsCertificate`가 이어서 인증서를 요구하므로 결과는 맞다. 문구를 고칠 필요는 없고, 이렇게 두 규칙이 이어진다는 것만 기록해 둔다.
4. `AWS_PUBLIC_DOMAIN`에 실수로 `https://`나 끝의 `/`를 넣으면 `https://https://...`로 확인하다 실패한다. 실패 문구에 주소가 그대로 찍혀 원인은 보이므로 그대로 둬도 된다(6단계 2번에 이미 "없이"라고 적혀 있음).

## 못 본 것
- 실제 CloudFormation 변경 세트(미리보기)는 AWS에 들어가지 않아 못 봤다. 부모님이 5단계 7번에서 `SiteDistribution` = Modify, 레코드 2개 = Add, **Replacement 없음**을 꼭 확인해 주세요. 빈 값으로는 이 업데이트를 미리 해 볼 필요가 없다(바뀌는 것이 없다).
- 도메인 가격은 확인하지 않았다(안내서도 "확인 못 함"으로 솔직히 적어 둠).

판정: 1번 고친 뒤 커밋 (부모님은 1번이 고쳐진 안내서로 4단계를 진행)
