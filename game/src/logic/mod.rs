//! 게임 규칙 (수산나). 화면 그리기·입력·소리는 여기서 하지 않는다.
//! 시간은 `dt`(초)로 받고, 결과는 상태로 돌려준다 → `cargo test`로 검사할 수 있다.

pub mod character;
pub mod citizen;
pub mod clock;
pub mod coins;
pub mod combat;
pub mod game;
pub mod math;
pub mod miss;
pub mod monster;
pub mod player;
pub mod skeleton;
pub mod weapon;
pub mod world;
