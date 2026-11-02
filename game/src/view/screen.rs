//! 화면 맞춤: 게임은 항상 960×540 "가상 화면"에 그린다.
//! 창 크기가 달라도 비율을 지켜 가운데에 꽉 차게 키우고, 남는 곳은 띠(레터박스)로 둔다.

use macroquad::prelude::*;

pub const VIRTUAL_WIDTH: f32 = 960.0;
pub const VIRTUAL_HEIGHT: f32 = 540.0;
/// 화면 바깥 띠 색
const LETTERBOX_COLOR: Color = Color::new(0.16, 0.20, 0.28, 1.0);

/// 이번 프레임의 화면 맞춤 정보 (창 좌표 = 논리 픽셀, `screen_width()`와 같은 단위).
#[derive(Clone, Copy)]
pub struct Letterbox {
    /// 가상 1픽셀이 창에서 몇 픽셀인지
    pub scale: f32,
    /// 가상 화면 왼쪽 위가 창의 어디에 오는지
    pub offset: Vec2,
}

impl Letterbox {
    pub fn current() -> Letterbox {
        Letterbox::for_window(screen_width(), screen_height())
    }

    /// 창 크기(논리 픽셀)에 맞춘 화면 맞춤 정보.
    pub fn for_window(sw: f32, sh: f32) -> Letterbox {
        let scale = (sw / VIRTUAL_WIDTH).min(sh / VIRTUAL_HEIGHT).max(0.01);
        let offset = vec2(
            (sw - VIRTUAL_WIDTH * scale) / 2.0,
            (sh - VIRTUAL_HEIGHT * scale) / 2.0,
        );
        Letterbox { scale, offset }
    }

    /// 창 좌표(마우스·터치) → 가상 화면 좌표.
    pub fn to_virtual(&self, window_point: Vec2) -> Vec2 {
        (window_point - self.offset) / self.scale
    }

    /// 띠를 칠하고, 이후 그리기를 가상 화면 좌표로 하게 카메라를 건다.
    pub fn begin(&self) {
        clear_background(LETTERBOX_COLOR);
        let dpi = screen_dpi_scale();
        // viewport는 실제(물리) 픽셀, 아래쪽 기준. 위아래 띠가 같으므로 offset.y를 그대로 쓴다.
        let viewport = (
            (self.offset.x * dpi).round() as i32,
            (self.offset.y * dpi).round() as i32,
            (VIRTUAL_WIDTH * self.scale * dpi).round() as i32,
            (VIRTUAL_HEIGHT * self.scale * dpi).round() as i32,
        );
        set_camera(&Camera2D {
            target: vec2(VIRTUAL_WIDTH / 2.0, VIRTUAL_HEIGHT / 2.0),
            // y가 아래로 커지게 (화면 좌표와 같은 방향)
            zoom: vec2(2.0 / VIRTUAL_WIDTH, 2.0 / VIRTUAL_HEIGHT),
            viewport: Some(viewport),
            ..Default::default()
        });
    }

    pub fn end(&self) {
        set_default_camera();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_point_to_virtual() {
        // 창 1920×1200 → scale 2, 위아래 띠 60픽셀
        let lb = Letterbox { scale: 2.0, offset: vec2(0.0, 60.0) };
        assert_eq!(lb.to_virtual(vec2(0.0, 60.0)), vec2(0.0, 0.0));
        assert_eq!(lb.to_virtual(vec2(1920.0, 1140.0)), vec2(960.0, 540.0));
    }
}
