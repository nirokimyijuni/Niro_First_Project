# 구글 검색 준비 (Search Console 등록 안내)

관리: 케빈(에이전트). 부모 결정 2026-11-21.
대표 주소(확정, 2026-11-21): `https://findmis.niro-studio.com/` (`site-url.txt` 한 줄)
→ 부모 결정 2026-11-21: 도메인이 열려서 `https://findmis.niro-studio.com/`로 바꿨다. 처음 값은 `https://nirokimyijuni.github.io/Niro_First_Project/`였다. 순서: [../aws/도메인.md](../aws/도메인.md) 8단계. **바꿈 완료.**

## 이 폴더에 있는 것

| 파일 | 하는 일 |
|---|---|
| `site-url.txt` | 대표 주소 한 줄. **주소를 바꿀 때는 이 파일만 고친다** (https://로 시작, /로 끝). |
| `robots.txt` | 모두 허용 + `Sitemap: <대표 주소>sitemap.xml`. 빌드 때 주소가 채워진다. |
| `sitemap.xml` | 대표 주소와 소개 페이지(about.html) 두 개. `lastmod`는 빌드 날짜(UTC)로 자동. |
| `google….html` | (아직 없음) Search Console 소유 확인 파일. 받으면 여기 넣는다. |

`deploy/build.sh`가 위 파일을 `dist/` 맨 위로 복사한다. AWS에도 같은 파일이 올라가지만 sitemap과 robots의 주소는 대표 주소만 가리킨다.

## 알아둘 한계

- GitHub Pages 프로젝트 사이트라서 우리 `robots.txt`는 `…github.io/Niro_First_Project/robots.txt`에 놓인다. 검색 로봇은 **도메인 맨 위**(`nirokimyijuni.github.io/robots.txt`)만 읽으므로 이 파일은 로봇에게 읽히지 않는다. 맨 위 파일은 `nirokimyijuni.github.io` 저장소(사용자 사이트)가 있어야 만들 수 있다. 없으면 GitHub 기본 동작(모두 허용)이라 막히지는 않는다.
- 그래서 **sitemap은 Search Console에 직접 제출한다.** 그러면 robots.txt 없이도 구글이 sitemap을 읽는다. 하위 경로의 sitemap도 그 경로 안의 주소만 담고 있으면 문제없다.
- AWS 주소에서는 robots.txt가 도메인 맨 위에 놓여 실제로 읽힌다. 거기의 Sitemap 줄은 GitHub 주소를 가리키는데, 다른 도메인 sitemap을 가리키는 것은 규칙상 허용된다.
- 같은 게임이 두 주소에 있어도 `index.html`·`about.html`의 canonical이 대표 주소를 가리켜서 구글은 대표 주소를 고른다. web/ 파일에는 `__SITE_URL__` 자리표시로 들어 있고 빌드가 `site-url.txt` 값으로 채운다.

## 내일 할 일 나누기

### 부모님이 할 것
1. https://search.google.com/search-console 에 구글 계정으로 들어간다.
2. **속성 추가 → "URL 접두어"** 를 고르고 대표 주소를 그대로 넣는다 (끝의 `/`까지). "도메인" 방식은 DNS를 바꿔야 해서 github.io에서는 못 쓴다.
3. 소유 확인 방법을 하나 고른다.
   - **HTML 파일 (추천)**: `google…….html` 파일을 내려받아 케빈에게 넘긴다. `index.html`을 건드리지 않아서 로건 작업과 부딪히지 않는다.
   - **HTML 태그(메타 태그)**: `<meta name="google-site-verification" content="…">` 한 줄을 복사해 넘긴다. 이 경우 `web/index.html`의 `<head>`에 넣어야 해서 로건과 순서를 맞춘다.
   - 이 값들은 비밀번호가 아니다. 공개 페이지에 그대로 올라간다.
4. 케빈이 배포를 끝냈다고 하면 Search Console에서 **"확인"** 을 누른다.
5. 왼쪽 **Sitemaps** 메뉴에 `sitemap.xml`을 입력하고 제출한다 (대표 주소 뒤에 자동으로 붙는다).
6. (선택) **URL 검사**에 대표 주소를 넣고 **"색인 생성 요청"** 을 누른다.

### 케빈이 할 것
1. 받은 확인 파일을 `deploy/seo/`에 넣는다 (빌드가 dist 맨 위로 복사한다). 메타 태그 방식이면 로건에게 넘긴다.
2. 출시 기준을 확인하고 커밋·push해 배포한다.
3. 공개 주소에서 `robots.txt`, `sitemap.xml`, 확인 파일이 열리는지 확인하고 부모님께 "확인 눌러도 돼요"라고 알린다.
4. 배포 시도를 `deploy/RELEASES.md`에 적는다.

## 얼마나 걸리나
- 소유 확인은 바로 된다.
- sitemap 상태가 "성공"이 되기까지 몇 시간~며칠.
- 실제로 구글 검색에 나오기까지 **며칠~몇 주**. 새 사이트는 늦을 수 있고, 꼭 나온다는 보장은 없다.
- 소유 확인 파일이나 메타 태그는 **지우지 않는다.** 지우면 나중에 소유 확인이 풀린다.
