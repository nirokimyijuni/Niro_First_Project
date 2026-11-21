// 소리 부품(quad-snd)은 브라우저(index.html/mq_js_bundle.js)가 넣어 주는 함수(audio_init 등)를 부른다.
// 요즘 러스트는 wasm에서 "아직 없는 함수"를 링크 오류로 막으므로, 브라우저에서 채워 줄 것을 링크가 허락하게 한다.
// (macroquad/miniquad 본체는 이미 같은 방식으로 브라우저 함수를 쓴다.) PC 빌드에는 아무 영향 없다.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let target = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target == "wasm32" {
        println!("cargo:rustc-link-arg-bins=--allow-undefined");
    }
}
