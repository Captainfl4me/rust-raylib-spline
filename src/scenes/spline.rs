use crate::bezier::*;
use crate::colors::*;
use crate::scenes::Scene;
use raylib::prelude::*;
use std::cell::RefCell;
use std::ffi::CStr;
use std::rc::Rc;

const T_ANIMATION_SPEED: f32 = 0.005;

pub struct BezierSplineScene {
    points: Vec<Rc<RefCell<Box<dyn MovableGuiPoint>>>>,
    animated: bool,
    animation_bounce: bool,
    has_point_selected: bool,
    debug_draw: bool,
    t: f32,
    draw_bounding_box: bool,
    lock_move: bool,
    is_closed_loop: bool,
}
impl Scene for BezierSplineScene {
    fn get_title(&self) -> &str {
        "Bezier Spline Scene"
    }

    fn has_background(&self) -> bool {
        false
    }

    fn help_text(&self) -> Vec<&str> {
        [
            "ESC - Go back to main menu",
            "SPACE - Add new cubic Bezier to the spline with the last join at mouse position",
            "BACKSPACE - Remove last cubic Bezier set",
            "MOUSE CLICK - Move point",
            "ENTER - Close path (while close SPACE can no longer be use)",
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
        // Update self.points
        for point in self.points.iter_mut() {
            point.borrow_mut().udpate_gui(mouse_position);
            if point.borrow().is_selected() {
                point
                    .borrow_mut()
                    .set_position(mouse_position, self.lock_move);
            }
        }
        if rl_handle.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            if !self.has_point_selected {
                for point in self.points.iter_mut() {
                    if point.borrow().is_hovered() {
                        point.borrow_mut().set_selected(true);
                        self.has_point_selected = true;
                        break;
                    }
                }
            }
        } else if self.has_point_selected {
            for point in self.points.iter_mut() {
                point.borrow_mut().set_selected(false);
            }
            self.has_point_selected = false;
        }

        if let Some(key) = rl_handle.get_key_pressed() {
            if !self.has_point_selected {
                match key {
                    KeyboardKey::KEY_SPACE => {
                        if !self.is_closed_loop {
                            let pm1_pos =
                                self.points[self.points.len() - 1].borrow().get_position();
                            let pm2_pos =
                                self.points[self.points.len() - 2].borrow().get_position();
                            // First control point is mirrored with n-1 control point
                            let new_point = RefCell::new(Box::new(ControlPoint::new(
                                pm1_pos * 2.0 - pm2_pos,
                                Some(&self.points[self.points.len() - 2]),
                                Some(&self.points[self.points.len() - 1]),
                            )));
                            self.points.push(Rc::new(new_point));
                            // Update next control point ref on previous join
                            self.points[self.points.len() - 2]
                                .borrow_mut()
                                .set_constraint(
                                    JoinPointConstraintID::NextControlPoint as usize,
                                    &self.points[self.points.len() - 1],
                                );
                            self.points[self.points.len() - 3]
                                .borrow_mut()
                                .set_constraint(
                                    ControlPointConstraintID::LinkedControlPoint as usize,
                                    &self.points[self.points.len() - 1],
                                );

                            // Second control point between new point and last (n) control
                            self.points
                                .push(Rc::new(RefCell::new(Box::new(ControlPoint::new(
                                    (mouse_position + pm1_pos) * 0.5,
                                    None,
                                    None,
                                )))));
                            self.points
                                .push(Rc::new(RefCell::new(Box::new(JoinPoint::new(
                                    mouse_position,
                                    Some(&self.points[self.points.len() - 1]),
                                    None,
                                )))));
                            let linked_join_point_ref = &self.points[self.points.len() - 1];
                            self.points[self.points.len() - 2]
                                .borrow_mut()
                                .set_constraint(
                                    ControlPointConstraintID::MirrorJoinPoint as usize,
                                    linked_join_point_ref,
                                );
                        }
                    }
                    KeyboardKey::KEY_BACKSPACE => {
                        if self.points.len() > 4 {
                            for _ in 0..if self.is_closed_loop { 2 } else { 3 } {
                                self.points.pop();
                            }
                            self.is_closed_loop = false;
                        }
                    }
                    KeyboardKey::KEY_ENTER => {
                        if !self.is_closed_loop {
                            let pm1_pos =
                                self.points[self.points.len() - 1].borrow().get_position();
                            let pm2_pos =
                                self.points[self.points.len() - 2].borrow().get_position();
                            // First control point is mirrored with n-1 control point
                            let new_point = RefCell::new(Box::new(ControlPoint::new(
                                pm1_pos * 2.0 - pm2_pos,
                                Some(&self.points[self.points.len() - 2]),
                                Some(&self.points[self.points.len() - 1]),
                            )));
                            self.points.push(Rc::new(new_point));
                            // Update next control point ref on previous join
                            self.points[self.points.len() - 2]
                                .borrow_mut()
                                .set_constraint(
                                    JoinPointConstraintID::NextControlPoint as usize,
                                    &self.points[self.points.len() - 1],
                                );
                            self.points[self.points.len() - 3]
                                .borrow_mut()
                                .set_constraint(
                                    ControlPointConstraintID::LinkedControlPoint as usize,
                                    &self.points[self.points.len() - 1],
                                );

                            let pm1_pos = self.points[0].borrow().get_position();
                            let pm2_pos = self.points[1].borrow().get_position();
                            // Second control point is mirrored with first control point (arr[1])
                            self.points
                                .push(Rc::new(RefCell::new(Box::new(ControlPoint::new(
                                    pm1_pos * 2.0 - pm2_pos,
                                    Some(&self.points[1]),
                                    Some(&self.points[0]),
                                )))));
                            self.points[0].borrow_mut().set_constraint(
                                JoinPointConstraintID::PreviousControlPoint as usize,
                                &self.points[self.points.len() - 1],
                            );
                            self.points[1].borrow_mut().set_constraint(
                                ControlPointConstraintID::LinkedControlPoint as usize,
                                &self.points[self.points.len() - 1],
                            );
                            self.is_closed_loop = true;
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
        let bounding_box_toggle_text = CStr::from_bytes_with_nul(b"Draw Bouding box\0").unwrap();
        let debug_text = CStr::from_bytes_with_nul(b"Activate debug draw\0").unwrap();
        let lock_move_text = CStr::from_bytes_with_nul(b"Lock points\0").unwrap();

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
            rl_draw_handle.gui_toggle(
                Rectangle::new(40.0, 110.0, 300.0, 25.0),
                Some(bounding_box_toggle_text),
                &mut self.draw_bounding_box,
            );
        }
        rl_draw_handle.gui_toggle(
            Rectangle::new(
                40.0,
                if self.debug_draw { 140.0 } else { 50.0 },
                300.0,
                25.0,
            ),
            Some(lock_move_text),
            &mut self.lock_move,
        );

        for cubic_bezier_points in self.points.windows(4).step_by(3) {
            let cubic_bezier_points = cubic_bezier_points
                .iter()
                .map(|b| b.borrow().downcast_basic_point())
                .collect::<Vec<_>>();
            draw_bezier(
                &cubic_bezier_points,
                rl_draw_handle,
                if self.debug_draw { Some(self.t) } else { None },
            );
            if self.draw_bounding_box {
                if let Ok(bb) = cubic_bezier_bounding_box(&cubic_bezier_points) {
                    rl_draw_handle.draw_line_v(
                        Vector2::new(bb.x, bb.y),
                        Vector2::new(bb.x + bb.width, bb.y),
                        COLOR_RED,
                    );
                    rl_draw_handle.draw_line_v(
                        Vector2::new(bb.x + bb.width, bb.y),
                        Vector2::new(bb.x + bb.width, bb.y + bb.height),
                        COLOR_RED,
                    );
                    rl_draw_handle.draw_line_v(
                        Vector2::new(bb.x + bb.width, bb.y + bb.height),
                        Vector2::new(bb.x, bb.y + bb.height),
                        COLOR_RED,
                    );
                    rl_draw_handle.draw_line_v(
                        Vector2::new(bb.x, bb.y + bb.height),
                        Vector2::new(bb.x, bb.y),
                        COLOR_RED,
                    );
                }
            }
        }
        if self.is_closed_loop {
            let cubic_bezier_points = [
                &self.points[self.points.len() - 3],
                &self.points[self.points.len() - 2],
                &self.points[self.points.len() - 1],
                &self.points[0],
            ]
            .iter()
            .map(|b| b.borrow().downcast_basic_point())
            .collect::<Vec<_>>();
            draw_bezier(
                &cubic_bezier_points,
                rl_draw_handle,
                if self.debug_draw { Some(self.t) } else { None },
            );

            if self.draw_bounding_box {
                if let Ok(bb) = cubic_bezier_bounding_box(&cubic_bezier_points) {
                    rl_draw_handle.draw_line_v(
                        Vector2::new(bb.x, bb.y),
                        Vector2::new(bb.x + bb.width, bb.y),
                        COLOR_RED,
                    );
                    rl_draw_handle.draw_line_v(
                        Vector2::new(bb.x + bb.width, bb.y),
                        Vector2::new(bb.x + bb.width, bb.y + bb.height),
                        COLOR_RED,
                    );
                    rl_draw_handle.draw_line_v(
                        Vector2::new(bb.x + bb.width, bb.y + bb.height),
                        Vector2::new(bb.x, bb.y + bb.height),
                        COLOR_RED,
                    );
                    rl_draw_handle.draw_line_v(
                        Vector2::new(bb.x, bb.y + bb.height),
                        Vector2::new(bb.x, bb.y),
                        COLOR_RED,
                    );
                }
            }
        }
    }
}
impl Default for BezierSplineScene {
    fn default() -> Self {
        let mut points: Vec<Rc<RefCell<Box<dyn MovableGuiPoint>>>> = vec![Rc::new(RefCell::new(
            Box::new(JoinPoint::new(Vector2::new(300.0, 600.0), None, None)),
        ))];

        points.push(Rc::new(RefCell::new(Box::new(ControlPoint::new(
            Vector2::new(600.0, 300.0),
            None,
            Some(&points[0]),
        )))));

        points.push(Rc::new(RefCell::new(Box::new(ControlPoint::new(
            Vector2::new(900.0, 300.0),
            None,
            None,
        )))));

        points.push(Rc::new(RefCell::new(Box::new(JoinPoint::new(
            Vector2::new(1200.0, 600.0),
            Some(&points[2]),
            None,
        )))));

        points[0]
            .borrow_mut()
            .set_constraint(JoinPointConstraintID::NextControlPoint as usize, &points[1]);
        points[2].borrow_mut().set_constraint(
            ControlPointConstraintID::MirrorJoinPoint as usize,
            &points[3],
        );

        BezierSplineScene {
            points,
            animated: true,
            animation_bounce: false,
            has_point_selected: false,
            debug_draw: true,
            t: 0.5,
            draw_bounding_box: false,
            lock_move: true,
            is_closed_loop: false,
        }
    }
}
