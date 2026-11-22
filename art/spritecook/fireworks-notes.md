# 폭죽 그림 (새 기록 화면용)

**SpriteCook이 아니라 코드(Pillow)로 그린 그림이에요. 크레딧 0.** 그래서 `spritecook-assets.json`에는 넣지 않았어요. (2026-11-22, 부모 결정: 크레딧이 0이라 코드로 그림)

## 어떻게 만들었나
- 스크립트: `art/spritecook/fireworks/draw_fireworks.py`. 다시 만들려면 `python.exe draw_fireworks.py` (확대 미리보기까지 보려면 `--preview`). 이 PC에서는 `/c/Users/SDS/AppData/Local/Programs/Python/Python313/python.exe`를 쓴다.
- `english/draw_english.py`와 같은 방법: 8배 크게 그린 뒤 LANCZOS로 줄이고, 어두운 테두리(34,36,48)를 둘렀다. 그래서 다른 그림들과 한 세트로 보인다. 테두리는 1.6픽셀이다.
- 한 개 크기 64×64, 배경 투명, 글자 없음. 그림은 가장자리에서 2픽셀 이상 떨어져 있어서 칸끼리 겹치지 않는다.
- 색은 gameover.rs의 CONFETTI_COLORS와 같다: 노랑 (255,219,56) / 분홍 (255,140,179) / 하늘색 (115,217,255). 가운데 반짝 점은 같은 색을 밝게 한 것이다.

## 파일 (줄 = 모양, 칸 = 색: 노랑, 분홍, 하늘색 순서)
- 시트 `art/spritecook/fireworks-sheet.png` (192×192, 3×3)
- 개별 `art/spritecook/fireworks/`
  - 윗줄 작은 별 모양 터짐: `burst_star_yellow.png`, `burst_star_pink.png`, `burst_star_blue.png`
  - 가운뎃줄 동그란 꽃 모양 터짐: `burst_flower_yellow.png`, `burst_flower_pink.png`, `burst_flower_blue.png`
  - 아랫줄 길게 떨어지는 반짝 줄기: `sparkle_fall_yellow.png`, `sparkle_fall_pink.png`, `sparkle_fall_blue.png`

## 쓰는 법 (로건에게)
- 정지 그림 그대로 코드가 커졌다 작아지게 쓴다. 터짐 두 가지는 가운데가 그림 한가운데(32,32)이고, 반짝 줄기는 위쪽 별이 (32,13)이다. 줄기는 위에서 아래로 떨어지는 모양이라 화면 위쪽에 놓으면 좋다.
- 확대할 때 흐려지지 않게 하려면 NEAREST보다 부드러운 확대(LINEAR)가 어울린다 (이 그림은 부드럽게 줄인 그림이라 픽셀 계단이 없다).
- 게임에서 쓸 최종 파일은 아직 `game/assets/`에 넣지 않았다.

## 확인한 것
- 9개 모두 64×64 RGBA, 네 모서리 알파 0(투명). 그림이 닿는 범위는 2~62(줄기는 위 2, 아래 57)라서 칸 밖으로 안 나간다.
- 시트를 어두운 남색 배경에 4배 확대해서 눈으로 봤다.
