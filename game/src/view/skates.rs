//! 스케이트를 신으면 캐릭터 발에 스케이트 색 도형이 보인다 (US-134).
//! 새 그림은 없다: 발 아래 날 한 줄과 바퀴 둘을 도형으로 그린다. 신었는지는 logic(`shop.has_skates()`)이 알려 주고,
//! 여기서는 위치와 모양만 계산한다. 모든 숫자와 색은 `config.rs`.

#[cfg(test)]
use super::assets::character_cut;
use super::sprites::Pose;
use crate::config::*;
use crate::logic::character::CharacterKind;
use crate::logic::player::Facing;
use macroquad::prelude::*;

/// 그릴 모양 하나: 날(가운데, 크기), 바퀴 둘의 가운데, 바퀴 반지름, 기울기(라디안).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SkateShape {
    pub blade_center: Vec2,
    pub blade_size: Vec2,
    pub wheels: [Vec2; 2],
    pub wheel_radius: f32,
    pub rotation: f32,
}

/// 도형을 그릴지: 신었고 판이 끝나지 않았을 때만. 쓰러졌어요·클리어 화면의 캐릭터는 지금과 같게 둔다.
pub fn skates_shown(has_skates: bool, game_ended: bool) -> bool {
    has_skates && !game_ended
}

fn color_of(c: [f32; 4]) -> Color {
    Color::new(c[0], c[1], c[2], c[3])
}

pub fn skates_color() -> Color {
    color_of(SKATES_COLOR)
}

fn foot_config(kind: CharacterKind) -> (f32, f32, f32) {
    let index = CharacterKind::ALL.iter().position(|k| *k == kind).unwrap_or(0);
    SKATES_FEET[index]
}

/// 잘라 낸 캐릭터 그림의 가로/세로 (원화 비율). 시험에서 실제 그림 대신 쓴다.
#[cfg(test)]
pub fn character_aspect(kind: CharacterKind) -> f32 {
    let (_, _, w, h) = character_cut(kind);
    w as f32 / h as f32
}

/// `feet`: 평소 그림의 발 가운데 (`draw_character`에 넘긴 값), `height`: 그림 높이, `aspect`: 그림 가로/세로.
/// `draw_character`와 같은 기준(발에서 뜨기, 발을 중심으로 기울기, 좌우 뒤집기)을 따라서 발에 붙어 다닌다.
pub fn skate_shape(kind: CharacterKind, feet: Vec2, height: f32, aspect: f32, facing: Facing, pose: Pose) -> SkateShape {
    let (dx, length, up) = foot_config(kind);
    let width = height * aspect * pose.scale_x;
    let foot = vec2(feet.x, feet.y - pose.lift * height);
    let side = if facing == Facing::Right { 1.0 } else { -1.0 };
    let blade_length = (width * length).max(SKATES_MIN_LENGTH);
    let wheel_y = -up * height * pose.scale_y - SKATES_WHEEL_RADIUS;
    let center_x = side * dx * width;
    let (sin, cos) = pose.tilt.sin_cos();
    // 발을 중심으로 돌린다 (macroquad의 그림 기울기와 같은 방향)
    let place = |local: Vec2| foot + vec2(local.x * cos - local.y * sin, local.x * sin + local.y * cos);
    let wheel_dx = blade_length * 0.3;
    SkateShape {
        blade_center: place(vec2(center_x, wheel_y - SKATES_BLADE_THICKNESS / 2.0)),
        blade_size: vec2(blade_length, SKATES_BLADE_THICKNESS),
        wheels: [place(vec2(center_x - wheel_dx, wheel_y)), place(vec2(center_x + wheel_dx, wheel_y))],
        wheel_radius: SKATES_WHEEL_RADIUS,
        rotation: pose.tilt,
    }
}

/// 날과 바퀴를 그린다. `tint`는 캐릭터에 곱하는 색 (밤에 푸르스름해지는 것과 깜박임 투명도를 따른다).
pub fn draw_skate_shape(shape: &SkateShape, tint: Color) {
    let shade = |c: Color| Color::new(c.r * tint.r, c.g * tint.g, c.b * tint.b, c.a * tint.a);
    let (blade, wheel, line) = (shade(skates_color()), shade(color_of(SKATES_WHEEL_COLOR)), shade(color_of(SKATES_OUTLINE_COLOR)));
    let rect = |size: Vec2, color: Color| {
        let params = DrawRectangleParams { offset: vec2(0.5, 0.5), rotation: shape.rotation, color };
        draw_rectangle_ex(shape.blade_center.x, shape.blade_center.y, size.x, size.y, params);
    };
    rect(shape.blade_size + Vec2::splat(SKATES_OUTLINE * 2.0), line);
    rect(shape.blade_size, blade);
    for w in shape.wheels {
        draw_circle(w.x, w.y, shape.wheel_radius + SKATES_OUTLINE, line);
        draw_circle(w.x, w.y, shape.wheel_radius, wheel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::sprites::{pose, PLAY_SPRITE_HEIGHT};
    use crate::view::assets::height_with_antenna;
    use crate::logic::player::Motion;

    const FEET: Vec2 = Vec2::new(480.0, 440.0);

    fn shape(kind: CharacterKind, facing: Facing, p: Pose) -> SkateShape {
        skate_shape(kind, FEET, height_with_antenna(kind, PLAY_SPRITE_HEIGHT), character_aspect(kind), facing, p)
    }

    #[test]
    fn only_shown_when_worn_and_playing() {
        assert!(skates_shown(true, false));
        assert!(!skates_shown(false, false));
        assert!(!skates_shown(true, true));
        assert!(!skates_shown(false, true));
    }

    #[test]
    fn every_character_gets_a_shape_on_screen_near_the_feet() {
        for kind in CharacterKind::ALL {
            for facing in [Facing::Left, Facing::Right] {
                for p in [pose(Motion::Idle, 0.3, false), pose(Motion::Walk, 0.1, false), pose(Motion::Jump, 0.2, true)] {
                    let s = shape(kind, facing, p);
                    for at in [s.blade_center, s.wheels[0], s.wheels[1]] {
                        assert!(at.x > 0.0 && at.x < 960.0 && at.y > 0.0 && at.y < 540.0, "{kind:?} 화면 안");
                        assert!((at.x - FEET.x).abs() < 40.0, "{kind:?} 발 옆");
                        assert!((at.y - FEET.y).abs() < 60.0, "{kind:?} 발 근처");
                    }
                    assert!(s.blade_size.x >= SKATES_MIN_LENGTH && s.blade_size.y >= SKATES_BLADE_THICKNESS);
                    assert!(s.wheel_radius >= SKATES_WHEEL_RADIUS);
                }
            }
        }
    }

    #[test]
    fn left_and_right_are_mirrored() {
        let p = pose(Motion::Idle, 0.0, false);
        for kind in CharacterKind::ALL {
            let (r, l) = (shape(kind, Facing::Right, p), shape(kind, Facing::Left, p));
            assert!((r.blade_center.x - FEET.x + (l.blade_center.x - FEET.x)).abs() < 1e-3, "{kind:?} 가운데 좌우 대칭");
            assert!((r.blade_center.y - l.blade_center.y).abs() < 1e-3);
            assert!((r.wheels[0].x - FEET.x + (l.wheels[1].x - FEET.x)).abs() < 1e-3, "{kind:?} 바퀴 좌우 대칭");
        }
    }

    #[test]
    fn follows_the_feet_when_jumping_and_stays_below_the_picture_bottom() {
        let kind = CharacterKind::Niro;
        let ground = shape(kind, Facing::Right, pose(Motion::Idle, 0.0, false));
        let walk = pose(Motion::Walk, 0.25 / 4.4 * 2.0, false);
        assert!(walk.lift > 0.0);
        let hop = shape(kind, Facing::Right, walk);
        assert!(hop.wheels[0].y < ground.wheels[0].y, "뜨면 같이 뜬다");
        assert!(ground.wheels[0].y + ground.wheel_radius <= FEET.y + 0.01, "땅 아래로 안 내려간다");
    }

    #[test]
    fn values_come_from_config() {
        assert_eq!(skates_color(), Color::new(SKATES_COLOR[0], SKATES_COLOR[1], SKATES_COLOR[2], SKATES_COLOR[3]));
        assert_eq!(SKATES_FEET.len(), CharacterKind::ALL.len());
        let s = shape(CharacterKind::Ellie, Facing::Right, pose(Motion::Idle, 0.0, false));
        assert_eq!(s.wheel_radius, SKATES_WHEEL_RADIUS);
    }
}
