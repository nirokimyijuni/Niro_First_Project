//! 웹페이지(web/index.html)와 주고받기.
//! 브라우저에서는 index.html이 `loading_done` 함수를 넣어 준다. PC(네이티브)에서는 할 일이 없다.

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    fn loading_done();
}

/// "불러오는 중..." 글자를 치운다. 게임이 첫 화면을 그린 뒤에 한 번 부른다.
pub fn hide_loading_screen() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        loading_done();
    }
}
