use crate::bezier::*;
use crate::colors::*;
use crate::scenes::Scene;
use raylib::prelude::*;
use std::ffi::CStr;

const T_ANIMATION_SPEED: f32 = 0.005;

pub struct BezierCurveScene {
    points: Vec<BasicPoint>,
    animated: bool,
    animation_bounce: bool,
    has_point_selected: bool,
    debug_draw: bool,
    t: f32,
}
impl Scene for BezierCurveScene {
    fn get_title(&self) -> &str {
        "Bezier Curve Scene"
    }

    fn has_background(&self) -> bool {
        false
    }

    fn help_text(&self) -> Vec<&str> {
        [
            "ESC - Go back to main menu",
            "SPACE - Add new control point on mouse position",
            "BACKSPACE - Remove last point",
            "MOUSE CLICK - Move point",
        ]
        .to_vec()
    }

    fn update(&mut self, rl_handle: &mut RaylibHandle) {
        // Update inputs
        let mouse_position = Vector2::new(
            rl_handle.get_mouse_x() as f32,
            rl_handle.get_mouse_y() as f32,
        );

        // Scene computation
        // Update points
        for point in self.points.iter_mut() {
            point.udpate_gui(mouse_position);
            if point.is_selected {
                point.set_position(mouse_position, false);
            }
        }
        if rl_handle.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            if !self.has_point_selected {
                for point in self.points.iter_mut() {
                    if point.is_hovered {
                        point.is_selected = true;
                        self.has_point_selected = true;
                        break;
                    }
                }
            }
        } else if self.has_point_selected {
            for point in self.points.iter_mut() {
                point.is_selected = false;
            }
            self.has_point_selected = false;
        }

        if let Some(key) = rl_handle.get_key_pressed() {
            if !self.has_point_selected {
                match key {
                    KeyboardKey::KEY_SPACE => {
                        if self.points.len() < 62 {
                            // Prevent binomial overflow
                            self.points.last_mut().unwrap().color = COLOR_LIGHT;
                            let new_point = BasicPoint::new(mouse_position, COLOR_BLUE);
                            self.points.push(new_point);
                        }
                    }
                    KeyboardKey::KEY_BACKSPACE => {
                        if self.points.len() > 2 {
                            self.points.pop();
                            self.points.last_mut().unwrap().color = COLOR_BLUE;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Update Animation
        if self.animated {
            self.t += if self.animation_bounce {
                -T_ANIMATION_SPEED
            } else {
                T_ANIMATION_SPEED
            };
            if self.t >= 1.0 {
                self.t = 1.0;
                self.animation_bounce = true;
            } else if self.t <= 0.0 {
                self.t = 0.0;
                self.animation_bounce = false;
            }
        }
    }

    fn draw(&mut self, rl_draw_handle: &mut RaylibDrawHandle) {
        let left_slider_text = CStr::from_bytes_with_nul(b"0.0\0").unwrap();
        let right_slider_text = CStr::from_bytes_with_nul(b"1.0\0").unwrap();
        let animation_toggle_text = CStr::from_bytes_with_nul(b"Animate T value\0").unwrap();
        let debug_text = CStr::from_bytes_with_nul(b"Activate debug draw\0").unwrap();

        // Draw GUI Controls
        rl_draw_handle.gui_toggle(
            Rectangle::new(40.0, 20.0, 300.0, 25.0),
            Some(debug_text),
            &mut self.debug_draw,
        );
        if self.debug_draw {
            rl_draw_handle.gui_slider_bar(
                Rectangle::new(40.0, 50.0, 300.0, 25.0),
                Some(left_slider_text),
                Some(right_slider_text),
                &mut self.t,
                0.0,
                1.0,
            );
            rl_draw_handle.gui_toggle(
                Rectangle::new(40.0, 80.0, 300.0, 25.0),
                Some(animation_toggle_text),
                &mut self.animated,
            );
        }

        draw_bezier(
            &self.points,
            rl_draw_handle,
            if self.debug_draw { Some(self.t) } else { None },
        );
    }
}
impl Default for BezierCurveScene {
    fn default() -> Self {
        // Initialize
        let points = [
            Vector2::new(300.0, 600.0),
            Vector2::new(600.0, 300.0),
            Vector2::new(900.0, 300.0),
            Vector2::new(1200.0, 600.0),
        ]
        .iter()
        .enumerate()
        .map(|(i, pos)| {
            let color = if i == 0 || i == 4 - 1 {
                COLOR_BLUE
            } else {
                COLOR_LIGHT
            };
            BasicPoint::new(*pos, color)
        })
        .collect::<Vec<_>>();

        BezierCurveScene {
            points,
            animated: true,
            animation_bounce: false,
            has_point_selected: false,
            debug_draw: true,
            t: 0.5,
        }
    }
}
