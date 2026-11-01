# Rust 설치 가이드 (Windows)

> 관리: 케빈(에이전트). M0 준비 단계. 이 PC에 이미 있는 것: winget, git

모든 명령은 **PowerShell**에서 실행한다.

## 1. C++ 빌드 도구 설치 (약 2~3GB, 10~20분)
Rust가 Windows에서 프로그램을 만들려면 Microsoft C++ 빌드 도구가 필요하다. 설치 중에 관리자 권한 창이 뜨면 "예"를 누른다.
```powershell
winget install --id Microsoft.VisualStudio.2022.BuildTools -e --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

## 2. Rust 설치
```powershell
winget install --id Rustlang.Rustup -e
```
설치가 끝나면 **PowerShell 창을 닫고 새로 연다**. 그래야 `cargo` 명령을 찾을 수 있다.

## 3. 잘 깔렸는지 확인
```powershell
rustc --version
cargo --version
```
둘 다 버전 번호가 나오면 성공이다.

## 4. 웹(Wasm)용 빌드 대상 추가
```powershell
rustup target add wasm32-unknown-unknown
```

## 5. 내 컴퓨터에서 게임을 열어 볼 작은 웹 서버
```powershell
cargo install basic-http-server
```

## 문제가 생기면
| 증상 | 해결 |
|---|---|
| `cargo`를 찾을 수 없음 | PowerShell 창을 새로 열기. 그래도 안 되면 PC 재시작 |
| `link.exe not found` | 1단계(C++ 빌드 도구)가 빠졌거나 설치가 덜 됨 → 1단계 다시 실행 |
| 회사 PC라 설치가 막힘 | IT 정책 문제일 수 있음. 집 PC에서 진행 |
