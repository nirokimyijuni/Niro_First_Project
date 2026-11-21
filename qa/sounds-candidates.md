# 소리 후보 목록 (받기 전, 부모님 승인용)

## 타이틀 곡 더 밝은 후보 (2026-11-21)

부모님 요청: "타이틀 음악이 조금 더 밝았으면". 지금 곡은 Classical Pop (Chiptune). **아래는 모두 내장 브라우저로 페이지를 열어 읽기만 했다 (파일은 안 받음, 로그인 없음). 곡을 직접 듣지는 못했다.** 분위기는 페이지의 태그·설명·댓글 근거다. "CC0 하나뿐"은 페이지의 License(s) 칸을 눈으로 본 결과다.

**부모님이 직접 고른 곡: 1번 Children's March Theme ("이거 좋은데").** 페이지 확인 결과 쓸 수 있다.

**교체 결과 (2026-11-21)**: 부모님이 zip을 `art/sounds-download/`에 직접 넣어 줬다. zip 안은 mp3·ogg·wav·readme.txt 4개뿐, 경로 이상 없음, readme.txt에도 "Creative Commons Zero (CC0)"라고 적혀 페이지와 같다. 원본 ogg(320kbps, 64.0초)를 libvorbis 96kbps로 다시 만들어 **704,434바이트 (약 0.70MB), 63.99초**. `music/title_children_march.ogg`로 넣고 `title_classical_pop.ogg`는 지웠다 (옛 파일은 git f880c4d에 있어 되돌릴 수 있다). 소리 합계 4,045,912바이트 (약 3.86MB, 5MB 이내). **듣기 확인은 부모님 몫.**

**코인 소리 교체 (2026-11-21)**: 슈퍼마리오 느낌의 "띠링"을 닌텐도 파일 없이 **직접 합성**했다 (ffmpeg aevalsrc 사각파, 988Hz 0.08초 다음 1319Hz 약 0.32초, 끝으로 줄어듦, 모노 22.05kHz). `sfx/coin_pickup.wav`(17,718바이트, 0.40초, 최대 -7.9dB, 평균 -12.3dB)로 덮어씀, 이름이 같아 코드는 안 고침. 옛 소리는 Kenney RPG Audio handleCoins2 (0.34초, 15KB)이고 파일은 지움. 대안으로 Kenney Digital Audio twoTone1, RPG Audio handleCoins가 있다 (zip에 있음, 이번엔 비교 안 함). 사인파를 섞은 더 부드러운 버전(평균 -7.3dB)도 만들어 봤으나 쓰지 않음. **직접 듣지 못했다. 듣기 확인은 부모님 몫.**

| 번호 | 곡 이름 | 만든 사람 | 소리 페이지 (여기서 미리 듣기 가능) | 라이선스 | 길이 | 형식·크기 | 반복 이음새 | 분위기 한 줄 | 지금 곡과 다른 점 |
|---|---|---|---|---|---|---|---|---|---|
| **1 (부모님 선택)** | Children's March Theme | CleytonKauffman | https://opengameart.org/content/childrens-march-theme | **CC0 하나뿐 확인함** | 1분 4초 | 압축 파일 하나 15.4MB 안에 WAV·MP3·OGG 세 가지 (곡 하나씩 따로 받는 길은 없음). 게임용 ogg로 다시 만들면 64kbps 약 0.5MB, 96kbps 약 0.77MB로 예상 (예상일 뿐) | **"이음새 없이 반복: 예"** 작가가 적음. mp3는 틈이 생기니 ogg나 wav를 쓰라고 작가가 적음 | 태그: 신나는(upbeat), 귀여운, 아이 같은(children), 즐거운, 장난스러운, 행진, 트롬본·작은북·플루트. 작가 설명: 아이 같은 가벼운 행진곡. 무섭지 않아 보임 | 지금 곡(칩튠 팝, 84초)보다 길이는 짧고, 칩튠 대신 **진짜 악기 소리 행진곡** 느낌. 더 밝고 장난스러운지는 들어 봐야 앎 |
| 2 | Flowerbed Fields [Loop] | Zane Little Music | https://opengameart.org/content/flowerbed-fields-loop | CC0 하나뿐 확인함 | 페이지에 없음 | ogg 1.7MB (그대로 쓰면 1MB 넘음, 줄이면 약 0.8MB 예상), wav 37.4MB | 제목에 [Loop], 태그 loop | "모험을 시작하는 귀여운 칩튠". 태그: 밝은, 귀여운, NES, 게임보이. 댓글: "통통 튀는 곡" | 지금 곡과 같은 칩튠 계열이지만 더 귀엽고 밝다고 적혀 있음. 바꿈 폭은 작음 |
| 3 | Catsong | Dan Knoflicek (올린 사람은 josepharaoh99, 페이지에 그렇게 적음) | https://opengameart.org/content/catsong | CC0 하나뿐 확인함 | 짧은 곡(길이 안 적힘) | ogg 1MB, mp3 1.1MB | 반복 표시 없음 | "행복하고 웃긴 분위기, 오르락내리락, 웃음 나는 곡". 짧음 | 아주 장난스러움. 짧아서 질릴 수 있음 |
| 4 | Toy Soldiers | Zane Little Music | https://opengameart.org/content/toy-soldiers | CC0 하나뿐 확인함 | 페이지에서 확인 못 함 | 페이지 앞부분만 읽어 파일 크기 못 봄 | 못 봄 | 태그: 장난감, 귀여운, 행복, 재미, 칩튠, 행진. 장난감 병정이 행진하는 곡 | 1번과 비슷한 행진 느낌인데 칩튠 |
| 5 | Beach Sports Theme (Loop) | Trex0n (Cal McEachern) | https://opengameart.org/content/beach-sports-theme-loop | CC0 하나뿐 확인함 | 페이지에 없음 | ogg 611.6KB (작아서 그대로 가능) | 제목에 (Loop) | 해변 + 스포츠 태그뿐, 설명은 "친구 게임용 짧은 반복 곡, 피크민에서 영감". 해변 화면에 어울릴 수 있으나 근거 약함 | 해변 타이틀 화면 느낌. 밝은지는 근거 부족 |

**뺀 곡**: The Rainbow Train(라이선스 칸에 CC-BY 등이 여럿 같이 적혀 있어 뺌, 크기 8.5MB), Happy Clappy Loop(wav 6.2MB뿐), Loop Town(ogg 2.4MB, 빠른 곡), Summers(mp3 3.6MB, 작가가 "크레딧 부탁"), Montage·Happy Loop(mp3뿐, 이음새 틈 걱정), Sunny Day Day 6(wav 24.6MB), Summer Sunday(wav 5.7MB), vintage hawaii(우쿨렐레 느낌이지만 mp3 2.7MB, 로파이 느린 곡), Ukulele Forest(숲 느낌의 짧은 곡 묶음이라 타이틀로는 약함).

**예전 표에서 아직 못 열어 봤던 곡을 이번에 열어 봄 (모두 CC0 하나뿐 확인함)**
- M2 Adventure Theme (CleytonKauffman): 압축 11.3MB, ogg, 반복은 "no intro" 파일을 쓰라고 함. 태그가 록·빠른·레이싱·슈퍼히어로라 **타이틀로는 너무 힘차서 뺌**.
- M3 Cool Adventure (MintoDog): mp3 4.3MB, ogg 3.8MB, 반복 가능. 태그 신스·댄스 음악(DM)이라 **크고 전자음악 느낌이라 뺌**.
- M8 Quaint Town (neonarkade): 반복용 mp3 1.6MB / wav 18.1MB (일반 mp3 2.1MB). 태그: 마을, 평화로운, 행복한, 반복. 밝기보다 **평화로운 쪽**이라 "더 밝게"에는 약함. 쓰려면 mp3를 ogg로 바꿔야 함.

**선택지**
- A. **1번 Children's March Theme으로 바꾸기 (부모님 선택, 추천)**: 받는 것은 압축 파일 15.4MB 하나이고 그중 ogg 하나만 쓴다. 쓰면 지금 타이틀 곡 0.5MB가 빠진다 (소리 합계 약 3.67MB에서 약 0.2~0.3MB 늘어남 예상, 5MB 안쪽).
- B. 지금 곡 (Classical Pop Chiptune) 계속 쓰기.
- C. 2번 Flowerbed Fields (칩튠 그대로 더 귀엽게).

**못 확인한 것**
- 곡을 **직접 듣지 못했다.** 밝은지·무섭지 않은지는 부모님이 위 페이지 주소의 미리 듣기로 들어 보고 정한다.
- 1번의 ogg 실제 크기와 앞뒤 무음은 압축 파일을 안 받아서 모른다.
- 4번은 파일 크기를 못 봤다.


작성: 로건 (2026-11-21) / 대상: US-112 ~ US-117 / 규칙: D113 (무료 CC0만)

**지금은 아무 파일도 받지 않았다.** 이 표를 보고 부모님이 "이거 받아"라고 번호로 말해 주면 그때 받는다.

## 먼저 알아 둘 것 (솔직하게)

1. **Kenney 소리 묶음은 소리 묶음 페이지에서 CC0라고 직접 확인했다.** (확인한 날 2026-11-21)
2. **OpenGameArt(OGA) 음악**: 처음에는 자동 읽기가 막혀(403) 못 봤다. **2026-11-21 두 번째 조사에서 내장 브라우저로 페이지를 직접 열어 읽었다** (읽기만, 로그인·다운로드 없음). 직접 연 곡은 표에 **"페이지 직접 확인함"**이라고 적었다: M1, M4, M5, M6, M7, N0~N5(N6은 M4). **직접 못 연 곡**(M2 Adventure Theme, M3 Cool Adventure, M8 Quaint Town)은 아직 검색 설명으로만 적은 것이라, 받기 전에 사람이 열어서 확인해야 하고 확인 못 하면 받지 않는다.
3. Kenney 묶음은 **소리 하나씩이 아니라 압축 파일(zip) 통째**로 받는 곳이다. 그중에 고른 파일만 게임에 넣고 나머지는 버린다. 그래서 아래 효과음 "파일 이름"은 묶음 안에서 **열어 보고 고를 후보 이름**이다 (페이지에는 이름이 안 보여서 이름은 아직 확정 아님). 소리를 직접 들어 보는 것도 받은 뒤에 한다.
4. 크기는 모두 **예상**이다 (페이지에 크기가 안 나온다). 받은 뒤 실제 크기로 고친다.
5. 게임에서 쓰는 소리 방식: **macroquad 0.4.16** `audio`는 **wav와 ogg**를 읽는다 (네이티브는 wav·ogg만, 웹은 브라우저가 풀어 준다). mp3는 쓰지 않는다. **아이폰 사파리가 ogg를 풀 수 있는지는 확인하지 못했다** (옛 아이폰은 ogg가 안 날 수 있다). 그래서 **짧은 효과음은 wav(작게 변환), 음악은 ogg로 하되 실제 아이폰에서 듣고 확인**하는 것을 제안한다. 안 나오면 음악도 wav 대신 다른 방법을 다시 정한다.
6. **지금 게임에는 소리 기능이 꺼져 있다.** `game/Cargo.toml`의 `macroquad = "0.4"`에 audio 기능이 안 켜져 있다 (Cargo.lock에 소리 부품 quad-snd가 없다). 구현 때 `features = ["audio"]`를 켜야 하고, wasm이 조금 커진다. 웹 쪽 `web/mq_js_bundle.js`에는 소리를 푸는 부분이 이미 들어 있다. (수산나가 game/ 고치는 중이라 지금은 건드리지 않았다.)

## 받은 결과 (2026-11-21, 부모님 승인 후)

| 소리 | 상태 | 실제 크기 | 비고 |
|---|---|---|---|
| M1 낮 Overworld Theme alt (Louswan) | **받음** | 837,583바이트 (약 0.84MB), 41.4초, ogg 44.1kHz 스테레오 | 페이지 라이선스 CC0 하나뿐 다시 확인. 그대로 `game/assets/sounds/music/day_overworld.ogg` |
| M7 상점 Urban Shop (MintoDog) | **받음** (다시 인코딩) | 원본 2,735,750바이트 -> **995,194바이트 (약 0.95MB)**, 83.5초 | 페이지 CC0 하나뿐 확인. libvorbis 96kbps로 다시 만듦. 앞뒤 무음 안 자름, 길이 그대로(83.478초). 80kbps는 827KB였지만 이음새 반복 음악이라 음질 여유를 위해 96kbps 선택 |
| 효과음 Kenney 5묶음 (S01~S36) | **받음** (부모님이 zip 5개를 직접 받음) | 39개 973,644바이트 (약 0.93MB) | 자세한 선택은 아래 "효과음 선택 결과". 5개 zip 모두 License.txt에서 CC0 다시 확인 |
| N1 밤+동굴 musicbox2_cute_tune (Aureolus_Omicron) | **받음** (다시 인코딩) | 원본 3,407,685바이트 -> **532,333바이트 (약 0.52MB)**, 137.1초 | 페이지 CC0 하나뿐 확인. zip(14.2MB)은 받지 않고 페이지에 따로 있는 곡 ogg 하나만 받음. libvorbis 48kbps, 앞뒤 무음 안 자름. 오르골은 소리가 단순해 낮은 비트레이트로 충분하다고 봄(직접 들어 보지는 못함) |
| 타이틀 Classical Pop (Chiptune) (Pro Sensory) | **받음** (mp3->ogg) | 원본 3,376,135바이트 -> **500,109바이트 (약 0.5MB)**, 84.4초 | 페이지 CC0 하나뿐 확인. libvorbis 64kbps, 앞뒤 무음 안 자름. 원본에 "반복 가능" 표시가 없어 이음새 틈은 들어 보고 확인 필요 |
| M6 Crystal Cave | **안 받음** | - | 승인대로 (동굴은 밤 음악 사용) |

현재 소리 합계: 음악 4곡 2,865,219바이트 (약 2.73MB) + 효과음 39개 973,644바이트 (약 0.93MB) = 약 3.66MB, 5MB 이내.

## 1. 배경음악 후보 (8곡)

모두 OGA. 검색 결과에서 CC0로 나온 곡만. **라이선스 확인 방법**: 아래 주소 페이지 아래쪽 "License(s)"에 **CC0 하나만** 있는지 본다 (CC-BY, BY-SA가 같이 있으면 빼기).

| 번호 | 어디에 | 곡 이름 | 만든 사람(표시 이름) | 길이 | 형식 | 크기(예상) | 소리 페이지 | 확인 상태 |
|---|---|---|---|---|---|---|---|---|
| **M1 확정(아이 선택 2026-11-21)** | 낮 | Overworld Theme | Louswan | 33초 / 다른 버전(alt) 41초 | ogg (wav도 있음) | **ogg 670.6KB (33초), alt ogg 837.6KB (41초)** | https://opengameart.org/content/overworld-theme-0 | **페이지 직접 확인함 (2026-11-21): 라이선스 칸 CC0 하나뿐.** 태그: 신나는(cheerful), 반복 가능(loopable), 8비트, 칩튠. 작가가 "alt 버전을 더 좋아한다"고 적음. US-113 "30초 이상"을 채우려면 **alt(41초)를 추천**, 고르는 것은 부모님·아이 |
| M2 | 낮 | Adventure Theme | CleytonKauffman | 페이지에서 확인 | ogg | 약 1MB | https://opengameart.org/content/adventure-theme | 검색 설명 CC0. "intro 없는 파일"이 이음새 없이 이어진다고 작가가 적음. 조금 힘차다 |
| M3 | 낮 (대안) | Cool Adventure | MintoDog | 페이지에서 확인 | mp3, ogg | 약 1~2MB | https://opengameart.org/content/cool-adventure | 검색 설명 CC0. 전자음악 느낌, 아이에게 시끄러울 수 있음 |
| M4 | 타이틀 후보 (미정) | Happy Lullaby (song17) | cynicmusic (The Cynic Project / pixelsphere.org) | 페이지에 길이 없음 | **mp3만 있음** (ogg 없음) | mp3 442.2KB | https://opengameart.org/content/happy-lullaby-song17 | **페이지 직접 확인함: CC0 하나뿐.** 태그: 자장가, 즐거운, 차분한, 종소리. "이음새 쉽다"고 작가가 적음. 작가가 "쓰면 알려 달라"고 부탁함(CC0라 의무는 아님) |
| M5 | 밤 (대안, 밤 후보 표 참고) | Calm Loop | wipics | 페이지에 길이 없음 | mp3만 | mp3 314.2KB | https://opengameart.org/content/calm-loop | **페이지 직접 확인함: CC0 하나뿐** ("Public Domain"이라고도 적힘). 태그: 편안한, 앰비언트, 신스, 타악기. 밤의 "오징어게임 느낌"과는 거리가 있음 |
| **M6 확정(아이 선택 2026-11-21)** | 동굴 | Crystal Cave (song18) | cynicmusic (The Cynic Project / pixelsphere.org) | 페이지에 길이 없음 (댓글에 "약 50~60초"라는 말이 있음, 확인은 아님) | **mp3만 있음** (ogg 없음) | mp3 534.9KB | https://opengameart.org/content/crystal-cave-song18 | **페이지 직접 확인함: 라이선스 칸 CC0 하나뿐.** 태그: 앰비언트, 종소리, 동굴, 수정, 편안한. "이음새 쉽다"고 작가가 적음. 작가가 "쓰면 알려 달라, 링크해 달라"고 부탁함(CC0라 의무는 아님, 부모님이 연락할지 정함). **mp3만 있어서 게임에 넣으려면 ogg로 바꿔야 하고, mp3를 바꾸면 이음새에 짧은 틈이 생길 수 있어 들어 보고 확인** |
| **M7 확정(아이 선택 2026-11-21)** | 상점 | Urban Shop | MintoDog | 페이지에 길이 없음 (92 BPM) | ogg, mp3 | **ogg 2.7MB**, mp3 3.3MB | https://opengameart.org/content/urban-shop | **페이지 직접 확인함: 라이선스 칸 CC0 하나뿐.** 태그: RPG, 트럼펫, 색소폰, 도시, 상점, 밝은, 반복 가능. 작가가 "반복 가능"이라고 적음. **크기가 커서(2.7MB) 받은 뒤 작은 ogg로 다시 만들어야 5곡 3MB 안에 든다** |
| M8 | 상점·타이틀 (대안) | Quaint Town | neonarkade | 페이지에서 확인 | mp3, wav | wav면 커서 변환 필요 | https://opengameart.org/content/quaint-town | 검색 설명 CC0. 마을 평화로운 느낌, 이음새 버전 있음 |

**지금 정해진 곡 (아이 선택 2026-11-21)**: 낮 = **M1**, 동굴 = **M6**, 상점 = **M7**. 아직 미정: **타이틀**(M4 Happy Lullaby 또는 다른 것)과 **밤**(바로 아래 표).

### 밤 후보 (오징어게임 느낌)

아이 말: 밤 음악은 "오징어 게임 테마 같은 느낌". 그 곡 자체는 저작권이 있어 쓸 수 없고 (D113 CC0만), **느낌만 비슷한** CC0 곡을 찾았다. 느낌: 장난감 같은 오르골·동요풍, 약간 긴장, 조용히 두근두근, 짧은 멜로디 반복. 단 **8살용이라 무섭지 않게**.
아래는 **모두 페이지를 직접 열어 확인함 (2026-11-21): 라이선스 칸 CC0 하나뿐.** 곡을 **들어 보지는 못했고**, 느낌은 페이지의 태그·설명·곡 이름을 근거로 한 것이다. 길이는 페이지에 안 나와서 받은 뒤 잰다.

| 번호 | 곡 이름 | 만든 사람 | 형식 / 크기 | 소리 페이지 | 쉬운 한글 한 줄 설명 (근거) |
|---|---|---|---|---|---|
| **N0 부모 선택 후보 (2026-11-21)** | Classical Pop (Chiptune) | Pro Sensory (Alex McCulloch) | **mp3만 있음** (ogg·wav 없음), **3.4MB** | https://opengameart.org/content/classical-pop-chiptune | **페이지 직접 확인함: 라이선스 칸 CC0 하나뿐.** 작가: "내 음악은 모두 퍼블릭 도메인, 이름을 적어 주면 고맙지만 꼭은 아님". 태그: 재즈, 클래식, 칩튠, 팝, 록. 길이는 페이지에 없음. **"반복 가능"이라는 말이 페이지에 없다** (이음새 틈은 들어 보고 확인). 댓글 2개는 "타이틀 화면에 딱 맞았다"는 이야기. 분위기는 설명에 없고 태그로는 **밝은 칩튠 팝**이라 무섭지는 않아 보이지만 "밤·긴장" 느낌은 태그로 알 수 없다. **mp3뿐이라 ogg로 바꿔야 하고 3.4MB라 줄여야 한다.** 밤보다 **타이틀 후보**에 더 어울릴 수 있다 |
| N1 | 4 Music Box Tracks 중 **musicbox2_cute_tune** | Aureolus_Omicron | ogg (4곡 묶음 zip **14.2MB**, 곡당 약 3.5MB로 추정, 곡 하나씩 따로 받는 길은 없음) | https://opengameart.org/content/4-music-box-tracks | 오르골 소리의 **귀여운 곡**. 태그: 귀여운, 오르골, 첼레스타, 반복(looping ogg). 장난감 오르골 느낌이 제일 가깝다. 이 곡이 첫째 추천. 긴장감은 약할 수 있음 |
| N2 | 같은 묶음의 **musicbox1_spooky_waltz** | Aureolus_Omicron | 위와 같음 | 위와 같음 | 오르골 **왈츠**. 곡 이름이 "으스스한(spooky)"이라서 오징어게임 느낌(긴장)에는 가깝지만 **무서울 수 있다**. 부모님이 들어 보고 아이에게 들려줄지 정한다. 아이가 무섭다고 하면 뺀다 |
| N3 | Intro Music (game.mp3) | RonyDkid | mp3만 **14.5MB** (줄여서 다시 만들어야 함) | https://opengameart.org/content/intro-music-0 | 현악기 **튕기는 소리(피치카토)** 로 "신비한 듯한(mysterious-ish)" 멜로디. 살금살금 두근두근 느낌, 귀엽게 긴장. 만든 사람 설명: 메인 화면용. 길고 커서 부담 |
| N4 | Stealth Music ("Stealth in the Woods") | Lisboa | mp3만 **6.1MB** (줄여야 함) | https://opengameart.org/content/stealth-music | "숲속에서 살금살금(stealth)" 반복곡. 태그가 stealth 하나뿐이라 느낌은 이름 근거뿐이다. 밤에 살금살금 느낌 후보 |
| N5 | Sneaky Music Pack 중 **dodging_lights** | Umplix | wav **6.2MB** (줄여야 함). 같은 묶음 다른 곡은 10.8MB, 15.4MB | https://opengameart.org/content/sneaky-music-pack | 스파이·살금살금 계열 곡 묶음 (태그: sneaky, spy, agent). 곡 이름만 보고 "불빛 피하기"로 골랐다. 이 묶음이 반복(loop)된다는 말은 페이지에 **없다**, 들어 보고 확인 |
| N6 | Happy Lullaby (M4 다시) | cynicmusic | mp3 442.2KB | 위 M4 주소 | 종소리 **자장가**. 두근두근보다는 포근함. 오징어게임 느낌은 약하지만 가장 작고 가장 안전한 대안 |

**N0(부모 선택)에 대해**: 라이선스는 통과(CC0 하나뿐)다. 막히는 것은 형식과 느낌이다. mp3뿐이라 변환이 필요하고, 페이지에 오징어게임 같은 긴장·오르골 근거가 없다. 그래서 **밤 곡으로 쓰려면 부모님이 먼저 들어 보고** 정하고, 아니면 가장 가까운 대안 **N1(오르골 cute_tune)** 을 쓴다. N0은 타이틀로 돌려도 좋다.

**내가 보기엔 (들어 보기 전 기준)**: 1순위 **N1**(오르골, 귀엽다), 2순위 **N3**(살금살금), 안전한 대안 **N6**. N2는 부모님이 먼저 들어 보고 결정. 이 중 **들어 보고 아이에게 들려줄 두세 곡**을 부모님이 고르면 된다 (US-113 "아이가 들어 보고 무서워하면 바꾼다").

**뺀 것**: Creepy(TokyoGeisha), Horror Loop(TinyWorlds)은 CC0지만 공포 태그라 뺐다. Spy Loop은 댓글에 원곡이 CC-BY라는 말이 있어 뺐다. Night of the Streets [Horror/Suspense], Emotional piano loop(슬픔·무서움 태그)도 뺐다. 3xBlast 15곡 묶음(CC0 확인, 155MB)과 Music loop strong downtempo seamless(Nostromo, CC0 확인, wav 12.7MB)는 느낌과 크기가 맞지 않아 뺐다.

**쓸 때 알아 둘 것**
- 낮·밤 음악이 18:00·06:00에 바뀔 때 앞 곡이 작아지고 새 곡이 커진다(US-113). 밤 곡이 **mp3뿐이면** ogg로 바꾸는 작업이 필요하다. 바꾸는 것은 CC0라서 허락 없이 해도 된다. 변환 도구가 이 PC에 있는지는 아직 확인하지 않았다.

**뺀 것 (라이선스 못 믿어서)**: Forgotten Lullaby (CC-BY·BY-SA), Ice Shine Bells (CC-BY와 CC0 같이 적힘), Music Box Game Over (CC-BY). Happy Adventure (Loop)는 CC0지만 mp3뿐이고 이음새 틈이 있다는 댓글이 있어 뺐다. Moonlit Lullaby for the Lost, Loopable Dungeon Ambience, Dark Cavern Ambient는 CC0라도 **어둡고 으스스해서** 8살용이 아니라 뺐다.

Kenney **Music Jingles**(85개, CC0 확인)는 긴 배경음악이 아니라 **2~5초짜리 짧은 가락**이라 음악 후보가 아니고, 아래 효과음(레벨업·퀴즈 등)에 쓴다.

## 2. 효과음 후보

출처는 모두 **Kenney** (www.kenney.nl, 만든 사람 표시 이름 "Kenney", 라이선스 **Creative Commons CC0**, 각 묶음 페이지에서 확인함). 묶음 5개:

| 묶음 | 파일 수 | 주소 | 어울리는 곳 |
|---|---|---|---|
| Interface Sounds | 100 | https://kenney.nl/assets/interface-sounds | 눌러요, 열고 닫기, 정답, 아쉬움 |
| Impact Sounds | 130 | https://kenney.nl/assets/impact-sounds | 맞힘, 쿵, 톡, 상자 |
| RPG Audio | 50 | https://kenney.nl/assets/rpg-audio | 칼 휘두르기, 코인 짤랑, 포션, 문 |
| Digital Audio | 60 | https://kenney.nl/assets/digital-audio | 뿅, 퓽, 레벨업, 반짝, 총 |
| Music Jingles | 85 | https://kenney.nl/assets/music-jingles | 축하, 레벨업, 클리어, 쓰러짐 |

**확인 방법**: 각 주소 페이지에 "License: Creative Commons CC0"라고 적혀 있다. 받은 zip 안에도 License.txt가 있다 (받은 뒤 CREDITS.md에 옮긴다).

각 줄의 "후보 a/b/c"는 한 순간에 2~3개씩이다. 아이(또는 부모님)가 들어 보고 하나 고르면 나머지는 버린다. 한 파일은 **약 10~40KB**(짧은 wav 변환 기준)로 본다.

### US-115 싸울 때

| 번호 | 순간 | 후보 a | 후보 b | 후보 c | 한 줄 설명 |
|---|---|---|---|---|---|
| S01 | 휘두르기 (맨손) | Impact Sounds의 부드러운 "휙/탁" | Digital Audio의 짧은 "뿅" | Interface Sounds의 "툭" | 짧고 작게 |
| S02 | 칼·도끼 휘두르기 | RPG Audio의 칼 뽑기 소리 | Impact Sounds의 "휙" | Digital Audio의 낮은 "슝" | 칼과 도끼는 높낮이로 다르게 |
| S03 | 총(주사기총) | Digital Audio의 "퓽" (laser 계열) | Digital Audio의 phaser 계열 | Interface Sounds의 "톡" | 장난감 소리, 진짜 총소리 아님 |
| S04 | 맞힘 | Impact Sounds의 soft "통" | Impact Sounds의 나무 "톡" | RPG Audio의 "툭" | 아파하는 소리 없음 |
| S05 | 몬스터 쓰러짐 (질문 3번: 뿅/퐁/띠용) | Digital Audio의 "뿅"(pep 계열) | Interface Sounds의 "퐁"(drop 계열) | Digital Audio의 "띠용"(위로 올라가는 소리) | 이 셋이 아이 질문 카드와 같다 |
| S06 | 내가 맞음 | Impact Sounds의 아주 부드러운 "통" | Interface Sounds의 낮은 "똑" | (없음도 가능, 부모 결정) | 놀라지 않게 작게 |
| S07 | 스켈레톤 화살 "휙" | Impact Sounds의 가벼운 "휙" | Digital Audio의 짧은 "슈" | - | |
| S08 | 폭발 시민 "뽕" | Digital Audio의 "뽕"(낮은 pep) | Impact Sounds의 풍선 같은 "퐁" | - | 큰 폭발음 아님 |
| S09 | 쿵(곡괭이 센 공격) | Impact Sounds의 낮은 "쿵" | Impact Sounds의 나무 큰 "쿵" | - | |
| S10 | 독약 포션 "칙" | RPG Audio의 포션/물 소리 | Digital Audio의 짧은 "치익" | - | |
| S11 | 상자 치기 "톡" | Impact Sounds의 나무 "톡" | RPG Audio의 "톡" | - | 부서질 땐 소리 없음 (D87) |

### US-116 줍고 캐고 만들 때

| 번호 | 순간 | 후보 a | 후보 b | 후보 c | 한 줄 설명 |
|---|---|---|---|---|---|
| S12 | 코인 줍기 "띵" (질문 4번) | RPG Audio의 handleCoins 계열 | Interface Sounds의 맑은 "띵"(bong/pluck) | Digital Audio의 "띠링"(twoTone) | 맑고 짧게 |
| S13 | 아이템 줍기 "뾰롱" | Digital Audio의 powerUp 계열 | Interface Sounds의 confirmation | - | |
| S14 | 캐는 중 "톡톡"/"캉캉" | Impact Sounds의 나무 "톡" | Impact Sounds의 쇠·돌 "캉" | - | 나무와 광석 따로 |
| S15 | 다 캠 "퐁" | Interface Sounds의 drop 계열 | Digital Audio의 높은 "퐁" | - | |
| S16 | 보석 나옴 반짝 | Music Jingles의 짧은 반짝 가락 | Digital Audio의 powerUp 계열 | Interface Sounds의 맑은 "딩" | 금·은·다이아·에메랄드·루비 |
| S17 | 제작대 "짠!" | Music Jingles의 짧은 가락 | Digital Audio의 threeTone 계열 | - | |
| S18 | 상점 사고팔기 "짤랑" | RPG Audio의 handleCoins2 계열 | RPG Audio의 handleCoins 계열 | - | |
| S19 | 코인 모자람 "뿌우" | Interface Sounds의 부드러운 낮은 소리 | Digital Audio의 lowDown 계열 | - | 놀리는 소리 아님. 날카로운 "삑" 금지 |
| S20 | 체력 포션 "꿀꺽" | RPG Audio의 포션 병 소리 | Interface Sounds의 "퐁" | - | |
| S21 | 자동차 "부릉" (아이가 정함 D84) | **Kenney에는 차 엔진 소리가 마땅치 않다** (못 찾음) | OGA·Freesound에서 CC0 엔진 소리를 따로 찾아야 함 | 대신 Digital Audio의 낮은 "부웅"(lowDown) 계열로 임시 | 이 한 개는 **후보를 못 정했다** (아래 "못 한 것") |
| S22 | 동굴 들어가고 나오기 "쏙" | Interface Sounds의 open/close 계열 | Digital Audio의 phaserUp/Down 계열 | - | |

가방 열고 닫기 소리는 **없다** (D105, 아이 결정). 상자 부서질 때도 없다 (D87).

### US-117 기쁜 순간과 하루가 바뀔 때

| 번호 | 순간 | 후보 a | 후보 b | 후보 c | 한 줄 설명 |
|---|---|---|---|---|---|
| S23 | 레벨업 (질문 5번: 빰빠밤/별/뿅뿅) | Music Jingles의 나팔 느낌(SAX·STEEL 계열) | Digital Audio의 반짝 올라가는 소리 | Music Jingles의 게임기 느낌(NES 계열) | 이 셋이 아이 질문 카드와 같다 |
| S24 | 미스 찾음 "띠링" | Interface Sounds의 confirmation 계열 | Digital Audio의 twoTone | - | |
| S25 | 퀴즈·미션 시작 "띵동" | Interface Sounds의 bong 계열 | Interface Sounds의 pluck 계열 | - | |
| S26 | 정답 "딩동댕" / 미션 성공 | Music Jingles의 밝은 짧은 가락 | Interface Sounds의 confirmation | Digital Audio의 threeTone | |
| S27 | 오답 / 미션 실패 (부드럽게) | Interface Sounds의 부드러운 낮은 소리 | Music Jingles의 느린 아쉬운 가락 | - | 놀리거나 무섭지 않게, "삑" 금지 |
| S28 | 밤 시작 종소리 | Interface Sounds의 bong 낮은 것 | Music Jingles의 종 느낌(STEEL 계열) | - | |
| S29 | 아침 (질문: 꼬끼오/새소리/딩동) | Interface Sounds의 밝은 "딩동" | Music Jingles의 밝은 가락 | **꼬끼오·새소리는 Kenney에 없다 (못 찾음)** | 새소리·닭소리는 다른 곳에서 CC0로 찾아야 함 |
| S30 | 센 아이템 "와우!" 반짝 | Digital Audio의 powerUp 계열 | Music Jingles의 짧은 반짝 | - | |
| S31 | 7일째 클리어 춤 음악 | Music Jingles의 긴 가락 (HIT·PIZZI·SAX 계열) | 위 음악 M1을 다시 틀기 | - | 5곡 밖이면 새로 안 받고 있는 소리 재사용도 가능 |
| S32 | 쓰러짐 "띠로리" (질문: 웃긴/조용) | Music Jingles의 짧은 내려가는 가락 | Digital Audio의 부드러운 lowDown | - | 슬프거나 무섭지 않게 |

### 그 밖의 순간

| 번호 | 순간 | 후보 a | 후보 b | 한 줄 설명 |
|---|---|---|---|---|
| S33 | 버튼 누르기 | Interface Sounds의 click 계열 | Interface Sounds의 tick 계열 | 모든 버튼 하나로 |
| S34 | 창 열기 / 닫기 (퀴즈, 제작대, 상점) | Interface Sounds의 open / close | Interface Sounds의 maximize / minimize | 가방은 소리 없음 |
| S35 | 연습용 몬스터 맞힘 | S04 재사용 | - | 튜토리얼에서 새로 받지 않는다 |
| S36 | 튜토리얼 칭찬 "잘했어요!" | S24 재사용 | S26 재사용 | 새로 받지 않는다 |

효과음 하나 하나는 Kenney 위 5묶음 안에서 고르는 것이라 **받는 zip은 5개**다. 그중 쓰는 파일은 약 **30~35개**로 줄인다.

## 3. 크기 합계와 로딩

| 항목 | 개수 | 예상 크기 |
|---|---|---|
| 배경음악 (낮 M1 alt 0.84MB + 동굴 M6 0.53MB(mp3, ogg로 바꾸면 비슷) + 상점 M7 **2.7MB** + 타이틀 M4 0.44MB + 밤 미정) | 5곡 | 받은 그대로면 **상점 때문에 약 4.5MB + 밤**. M7을 작은 ogg(약 1MB)로 다시 만들면 약 2.8MB + 밤 (밤 곡도 1MB 안으로 줄이면 약 3.8MB 안팎) |
| 효과음 (S01~S36 중 고른 것, wav 짧게·모노 22kHz 변환) | 약 30~35개 | 약 0.6~1MB |
| **합계** | | **약 3~3.5MB** |

- 지금 `임시-118`은 **총 3MB 이하**다. 위 예상은 3MB를 살짝 넘을 수 있다.
- **제안**: 시작 한도를 **총 5MB 이내 (음악 5곡 3MB 이내, 효과음 1MB 이내)**로 올리고, 부모님이 임시-118을 고치는 것에 동의하면 한도를 정한다. 곡이 5MB를 넘으면 짧은 곡으로 바꾼다. (한도 바꾸는 것은 니로 스토리 고침이라 나는 안 건드렸다.)
- ogg와 wav는 이미 눌린 파일이라 **gzip이 거의 안 줄여 준다.** 그래서 wasm gzip 크기 검사에는 소리 파일을 **포함하지 않는 것**이 맞다 (검사 기준을 정할 때 수산나·케빈과 확인). 지금 `game/target`의 wasm 크기는 이번에 재지 않았다.
- **"Now Loading"을 다시 길게 하지 않는 방법 (제안)**
  1. 처음 화면 로딩에는 소리를 **하나도 넣지 않는다.** (`눌러서 시작` 전에는 소리를 쓸 수 없으니 받아 둘 이유도 없다.)
  2. **효과음(약 1MB)** 은 "눌러서 시작"을 누른 **직후** 바로 받기 시작한다 (macroquad `coroutines`로 프레임마다 조금씩 풀어서 게임이 안 멈추게). 아직 못 받은 소리는 그냥 안 난다 (US-112 "소리 없어도 게임은 된다").
  3. **타이틀 음악 1곡**을 제일 먼저 받고, **낮 → 밤 → 동굴 → 상점** 순으로 뒤에서 받는다. 낮이 시작될 때쯤 낮 음악이 와 있게 한다.
  4. 받는 중에 실패해도 에러를 내지 않고 소리 없이 계속한다.

## 4. 못 한 것, 못 본 것

- **OGA M2, M3, M8 페이지는 아직 직접 못 열었다** (검색 설명으로만 적음). 나머지(M1, M4~M7, 밤 N1~N5)는 2026-11-21에 내장 브라우저로 직접 열어 CC0 하나뿐인 것을 확인했다.
- **곡 길이**: 페이지에 길이가 적힌 곡은 M1(33초/41초)뿐이다. 나머지는 받은 뒤 잰다.
- **오징어게임 느낌**: 곡을 들어 보지 못했고 태그·설명·곡 이름으로만 골랐다. 실제로 비슷한지는 부모님이 들어 보고 정한다.
- **mp3만 있는 곡**(M4, M5, M6, 밤 N3·N4)은 게임용 ogg로 바꿔야 한다. 변환 도구가 이 PC에 있는지, 바꾼 뒤 이음새 틈이 없는지는 확인하지 못했다.
- **"쓰면 알려 달라"는 부탁**: cynicmusic(M4, M6)이 페이지에 적었다. CC0라 의무는 아니지만 부모님이 연락할지 정한다. 어느 쪽이든 CREDITS.md에는 이름을 적는다.
- Kenney 효과음의 **정확한 파일 이름, 길이, 크기, 실제 소리**를 못 봤다 (zip을 받지 않아서). 후보 이름은 묶음 이름의 종류(click, bong, powerUp 같은 이름 모음)를 기준으로 쓴 것이라 **받은 뒤 들어 보고 확정**한다.
- **자동차 "부릉"(S21), 닭·새소리(S29)** 는 후보를 못 정했다. Kenney에 없어서 OGA나 Freesound에서 CC0를 따로 찾아야 한다 (Freesound는 받으려면 계정이 필요해서 부모님 확인이 필요하다. 이번엔 안 찾아봤다).
- **아이폰 사파리에서 ogg가 나오는지** 못 봤다.
- 소리를 **직접 들어 보지는** 못했다 (이 일은 받은 뒤).
- 아이 답이 없는 소리 질문(낮·밤 분위기, 몬스터 쓰러짐, 캐는 소리, 레벨업, 아침)은 위 표에서 후보를 여럿 넣어 둔 것으로 대신했다. 고르는 것은 아이.

## 5. 소리 시스템 설계 (한 장)

**목적**: 소리를 한 곳(`game/src/view/audio.rs`)에서 낸다. 규칙(logic)은 소리를 모른다.

**부품**
- `Cargo.toml`: `macroquad = { version = "0.4", features = ["audio"] }` (지금은 꺼져 있음). 수산나 작업이 끝난 뒤에 건드린다.
- `view/audio.rs`: `AudioBank { sounds: HashMap<SoundId, Option<Sound>>, music: ... }`. 못 받은 소리는 `None`이라 조용히 건너뛴다.
- 소리 하나 = `SoundId`(S05 같은 이름). 이벤트 → 소리 짝은 `match` 한 곳에 모은다 (`GameEvent`, `CombatEvent`, `GatherEvent` 등을 읽기만 한다). 짝 짓는 함수는 순수 함수로 만들어 단위 시험을 쓴다.
- 수치는 `config.rs`: 같은 소리 다시 나지 않는 시간 0.05초 (임시-117), 한꺼번에 나는 효과음 최대 개수(제안 6), 음악 바꿈 2초 (임시-116), 크기 4칸(끔 0 / 작게 0.25 / 보통 0.5 / 크게 0.8, 제안 값).

**설정 저장**
- 이미 있는 `Settings.sound_on`, `music_percent`, `effects_percent`를 읽고 쓴다. 바뀔 때마다 저장(US-125 저장 길).
- 최종 크기 = 칸 값 × 켬/끔. 끄면 음악도 `stop`이 아니라 크기 0으로 해서, 다시 켤 때 음악이 바로 이어지게 한다.

**M 키**
- `is_key_pressed(KeyCode::M)`로 `sound_on`을 뒤집고 화면에 "소리를 껐어요 / 켰어요"를 1초 보인다 (글은 `config.rs`). 입력은 `view/input.rs`에서 읽어 `main.rs`가 audio에 넘긴다.

**브라우저 정책 (첫 누름 뒤에 소리)**
- 브라우저는 처음 누르기(클릭·터치·키) 전에는 소리를 막는다. 이 게임은 **"눌러서 시작" 화면이 그 역할**을 한다.
- 순서: 로고 (소리 없음) → "눌러서 시작" 누름 (`audio.unlock()` 호출, 소리 받기 시작) → 타이틀 음악.
- 로고 때 소리(US-118, 아이 목소리)를 쓰려면 **로고가 눌러서 시작 뒤에 나와야 한다.** 지금 순서가 "로고 → 눌러서 시작"이면 로고 소리는 첫 누름 전이라 안 난다. 순서를 바꾸거나, 로고 소리를 "눌러서 시작 뒤"로 미뤄야 한다 (**부모님·니로가 정할 것**).
- 설정(`Settings`)은 **눌러서 시작 뒤에** 읽어서 크기를 정한다 (그 전에는 읽어도 쓸 데가 없다). 저장이 늦게 읽혀도 첫 소리는 저장된 크기로 나게 하려면 눌러서 시작 순간에 한 번 읽는다.

**Safari·iPhone**
- 소리 풀기: 첫 `touchend`(손가락을 뗀 순간)에서 `AudioContext.resume()`이 불려야 한다. macroquad 번들(`mq_js_bundle.js`)에 `resume()`이 있어 보이지만, **언제 불리는지** 아직 확인 못 했다. 실제 아이폰에서 첫 터치 뒤 소리가 나는지 로건이 시험 코드(`?qa=sound`처럼 짧은 소리 하나)로 확인한다.
- 무음 스위치가 켜져 있으면 웹 소리가 안 날 수 있다. 부모님께 알려 드린다 (US-112 메모).
- ogg가 안 나오면 효과음은 wav, 음악은 짧은 wav나 다른 방법으로 바꾼다 (위 5번).
- 탭을 바꾸거나 앱을 나가면 소리를 멈추고 돌아오면 다시 낸다 (visibilitychange 또는 macroquad가 알려 주는 정지·재개). 이건 US-112 완료 조건.

**단위 시험으로 쓸 것 (구현 때)**
- 이벤트 → 소리 이름 짝이 맞는가, 같은 소리가 0.05초 안에 또 나지 않는가, 한꺼번에 나는 개수 한도, 끔·크기 계산, 음악 바꿈 2초 동안 앞 곡 크기가 줄고 새 곡이 느는가, 못 받은 소리(`None`)는 조용히 건너뛰는가.

**지키는 것**: 소리 파일은 `game/assets/sounds/`, 출처 기록은 같은 폴더 `CREDITS.md`에 파일마다 한 줄 (파일 이름, 출처 사이트, 소리 페이지 주소, 라이선스 CC0, 만든 사람, 받은 날).

## 효과음 선택 결과 (2026-11-21, 부모님이 받은 Kenney zip 5개에서 로건이 고름)

**들어 보지 못하고 파일 이름과 길이로 골랐다.** 그래서 아이가 직접 들어 보고 바꿀 수 있게 대안 이름을 적었다. 대안은 압축 안에 있던 원래 이름이고, 게임 폴더에는 들어 있지 않다 (바꾸고 싶으면 부모님이 `art/sounds-download/`의 zip에서 꺼내 달라고 하면 된다). 5개 zip 모두 License.txt에 CC0라고 적혀 있다.

- 게임 폴더: `game/assets/sounds/sfx/` — **39개**, 합계 **973,644바이트 (약 0.93MB)**. 전부 wav, 모노, 22.05kHz.
- 음악 4곡 2,865,219바이트(2.73MB)와 합치면 **3,838,863바이트 (약 3.66MB)**, 5MB 안쪽.
- 길이를 자른 것 3개: toy_gun(앞 0.5초), clear_dance(앞 1.4초), fall_down_funny(앞 1초). 끝은 살짝 줄어들게 했다. 나머지는 원래 길이 그대로.
- 30~35개 안팎보다 조금 많은 이유: 아이 질문 카드에 "셋 중 고르기"가 있는 소리(쓰러짐 뿅/퐁/띠용, 레벨업 빰빠밤/별/뿅뿅)는 셋 다 넣었다. 크기는 넉넉하다.

| 소리 | 고른 파일 (게임 안 이름) | 원래 이름 | 대안 후보 (원래 이름) |
|---|---|---|---|
| S01 맨손 휘두르기 | swing_fist | RPG cloth2 | cloth3, Impact impactSoft_medium_000 |
| S02 칼·도끼 | swing_blade | RPG knifeSlice | knifeSlice2, drawKnife1 |
| S03 주사기총 | toy_gun | Digital laser3 | laser2, zap1 |
| S04 맞힘 / S35 연습용 몬스터 | hit | Impact impactSoft_medium_001 | impactWood_light_000, impactGeneric_light_000 |
| S05 쓰러짐 "뿅" | monster_down_pop | Digital pepSound3 | pepSound1, pepSound4 |
| S05 쓰러짐 "퐁" | monster_down_bloop | Interface drop_003 | drop_001 |
| S05 쓰러짐 "띠용" | monster_down_boing | Digital phaserUp3 | phaseJump2, phaserUp1 |
| S06 내가 맞음 | player_hit | Impact impactSoft_heavy_003 | Interface toggle_004 (또는 소리 없음) |
| S07 화살 "휙" | arrow_whoosh | RPG cloth4 | Digital highDown, Interface scroll_001 |
| S08 폭발 시민 "뽕" | bomb_pop | Digital pepSound5 | pepSound2, Impact impactSoft_heavy_001 |
| S09 센 공격 "쿵" | heavy_thud | Impact impactWood_heavy_000 | impactPlank_medium_000, impactPunch_heavy_002 |
| S10 독약 "칙" | poison_fizz | Interface scratch_004 | Digital phaserDown2 |
| S11 상자 치기 | box_tap | Impact impactWood_light_000 | impactWood_light_002, RPG bookPlace1 |
| S12 코인 줍기 | coin_pickup | RPG handleCoins2 | Interface pluck_001, Digital twoTone1 |
| S13 아이템 줍기 | item_pickup | Digital powerUp5 | Interface confirmation_001, powerUp4 |
| S14 캐는 중(나무) | dig_wood | Impact impactWood_medium_000 | impactWood_light_001 |
| S14 캐는 중(광석) | dig_stone | Impact impactMetal_light_001 | impactMining_000, impactMetal_medium_001 |
| S15 다 캠 "퐁" | dig_done | Interface drop_004 | drop_002, Digital highUp |
| S16 보석 반짝 | gem_sparkle | Jingles STEEL09 | Digital powerUp10, Interface glass_004 |
| S17 제작대 "짠!" | craft_done | Digital threeTone2 | Jingles HIT03, PIZZI04 |
| S18 상점 "짤랑" | shop_trade | RPG handleCoins | handleCoins2 |
| S19 코인 모자람 | not_enough_coins | Digital lowDown | Interface error_005 (날카로우면 안 씀), drop_001 |
| S20 체력 포션 "꿀꺽" | potion_gulp | Interface drop_002 | RPG handleSmallLeather |
| S21 자동차 "부릉" | **없음** | Kenney에 엔진 소리가 없다 | OGA·Freesound에서 따로 찾기 (부모님 확인 필요). 임시로 Digital lowRandom 가능 |
| S22 동굴 쏙 | cave_door | Digital phaserDown2 | phaserUp5, Interface open_003 |
| S23 레벨업 "빰빠밤" | level_up_fanfare | Jingles SAX03 | SAX01, STEEL01 |
| S23 레벨업 "별" | level_up_star | Digital powerUp3 | powerUp1, powerUp12 |
| S23 레벨업 "뿅뿅" | level_up_chip | Jingles NES13 | NES05, NES12 |
| S24 미스 찾음 / S36 튜토리얼 칭찬 | miss_found | Digital twoTone1 | Interface confirmation_002 |
| S25 퀴즈·미션 시작 "띵동" | quiz_start | Interface question_001 | bong_001, pluck_002 |
| S26 정답 / 미션 성공 | correct | Jingles PIZZI12 | Interface confirmation_004, Digital threeTone1 |
| S27 오답 / 미션 실패 | wrong | Digital highDown | Jingles PIZZI00, Interface error_005 |
| S28 밤 종소리 | night_bell | Impact impactBell_heavy_002 | Interface bong_001, Jingles STEEL00 |
| S29 아침 "딩동" | morning_ding | Digital twoTone2 | Interface confirmation_002, Jingles PIZZI14 |
| S29 닭 "꼬끼오"·새소리 | **없음** | Kenney에 없다 | 다른 곳에서 CC0로 찾기 |
| S30 센 아이템 "와우" | rare_item_wow | Digital powerUp12 | powerUp1, powerUp11 |
| S31 7일째 클리어 | clear_dance | Jingles SAX07 (앞 1.4초) | STEEL07, NES00, 또는 음악 M1 다시 틀기 |
| S32 쓰러짐 "띠로리" (웃긴) | fall_down_funny | Digital zapThreeToneDown (앞 1초) | Jingles SAX10 |
| S32 쓰러짐 (조용) | **따로 안 넣음** | - | Digital phaserDown1 또는 lowDown (wrong과 같은 소리로도 가능) |
| S33 버튼 누르기 | button_click | Interface click_001 | tick_004, pluck_001, select_003 |
| S34 창 열기 | window_open | Interface open_002 | maximize_001, open_003 |
| S34 창 닫기 | window_close | Interface close_002 | minimize_001, close_003 |

**알아 둘 것**
- 코드에서 쓰는 이름은 위 "고른 파일" 이름이다. 소리를 바꾸고 싶으면 같은 이름으로 덮어쓰면 코드를 안 고쳐도 된다.
- 무서운 소리(큰 폭발, 비명)는 하나도 안 골랐다. 날카로운 "삑"으로 들릴 수 있는 Interface error 계열은 오답에 쓰지 않았다.
- 압축 안의 의심스러운 것: 디지털 오디오 zip 안에 `Audio/desktop.ini`(윈도우 폴더 설정 파일) 하나만 있었다. 열지도 풀지도 않았고 쓰지 않았다. 그 밖에 스크립트·실행 파일은 없었고, 경로가 폴더 밖으로 나가는 항목도 없었다.
