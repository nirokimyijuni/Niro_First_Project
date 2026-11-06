# BUG-015 CARGO_TARGET_DIR를 쓰면 build.sh가 옛 wasm을 dist에 넣음
- 관련 스토리: 배포 (deploy/build.sh)
- 심각도: 보통 (게임 버그는 아니다. 옛 빌드를 새 빌드로 착각하게 된다)
- 재현: 1) `CARGO_TARGET_DIR`를 프로젝트 밖으로 둔다 2) `bash deploy/build.sh` 3) `dist/miss-game.wasm`과 새로 빌드된 wasm의 sha256을 비교한다
- 기대: 방금 빌드한 wasm이 dist에 들어간다. 없으면 멈춘다
- 실제: 스크립트의 `WASM` 경로가 `game/target/...`로 고정이라, 어제 22:13의 옛 wasm(706,589바이트, `ca31dc14`)이 조용히 복사됐다. 새 wasm(708,296바이트, `dc2b299e`)은 CARGO_TARGET_DIR 쪽에 있었다. 빌드는 성공으로 끝나고 버전 꼬리는 새 시각이라 화면으로는 구별이 안 된다
- GitHub Actions는 CARGO_TARGET_DIR를 안 쓰고, 플레이 링크 배포에는 영향이 없어 보인다 (deploy.yml을 읽은 결과, 돌려 보지는 않음)
- 고칠 사람: 케빈
- 환경: Windows Git Bash, 2026-11-07
- 상태: 닫힘 (2026-11-07 케빈: build.sh가 `${CARGO_TARGET_DIR:-$ROOT/game/target}`에서 wasm을 찾는다. 파일이 없으면 cp가 실패해 멈춘다)
