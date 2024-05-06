use ::core::cell::RefCell;
use raylib::core::texture::Image;
use raylib::prelude::*;
use std::ffi::CStr;
use std::rc::Rc;
use std::time::SystemTime;

mod bezier;
use bezier::*;

mod colors;
use colors::*;

enum Scenes {
    BezierCurve,
    BezierSpline,
}

fn main() {
    let (mut rl_handle, rl_thread) = raylib::init()
        .size(640, 480)
        .resizable()
        .title("Spline drawer")
        .build();
    rl_handle.set_target_fps(60);
    rl_handle.set_exit_key(None);
    rl_handle.set_window_state(rl_handle.get_window_state().set_window_maximized(true));

    let image_bytes = include_bytes!("../assets/background_tile.png");
    let mut background_tile_image = Image::load_image_from_mem(".png", image_bytes).unwrap();
    background_tile_image.resize(256, 256);
    let background_tile_texture = rl_handle
        .load_texture_from_image(&rl_thread, &background_tile_image)
        .unwrap();

    const TITLE_FONT_SIZE: i32 = 60;
    let mut scene_to_load = None;
    while !rl_handle.window_should_close() {
        {
            let mut rl_draw_handle = rl_handle.begin_drawing(&rl_thread);
            let screen_width = rl_draw_handle.get_screen_width();
            let screen_height = rl_draw_handle.get_screen_height();
            draw_background(&mut rl_draw_handle, &background_tile_texture);

            rl_draw_handle.draw_text(
                "Spline Drawer",
                (screen_width - rl_draw_handle.measure_text("Spline Drawer", TITLE_FONT_SIZE)) / 2,
                (screen_height - TITLE_FONT_SIZE) / 2,
                TITLE_FONT_SIZE,
                COLOR_LIGHT,
            );

            if rl_draw_handle.gui_button(
                Rectangle::new(
                    (screen_width - 200) as f32 / 2.0,
                    (screen_height + TITLE_FONT_SIZE + 10) as f32 / 2.0,
                    200.0,
                    25.0,
                ),
                Some(CStr::from_bytes_with_nul(b"Load Bezier curve\0").unwrap()),
            ) {
                scene_to_load = Some(Scenes::BezierCurve);
            }

            if rl_draw_handle.gui_button(
                Rectangle::new(
                    (screen_width - 200) as f32 / 2.0,
                    (screen_height + TITLE_FONT_SIZE * 2 + 10) as f32 / 2.0,
                    200.0,
                    25.0,
                ),
                Some(CStr::from_bytes_with_nul(b"Load Bezier spline\0").unwrap()),
            ) {
                scene_to_load = Some(Scenes::BezierSpline);
            }
        }

        match scene_to_load {
            Some(Scenes::BezierCurve) => {
                bezier_curve_scene(&mut rl_handle, &rl_thread, &background_tile_texture)
            }
            Some(Scenes::BezierSpline) => {
                bezier_spline_scene(&mut rl_handle, &rl_thread, &background_tile_texture)
            }
            None => {}
        }
        scene_to_load = None;
    }
}

fn draw_background(rl_draw_handle: &mut RaylibDrawHandle, tile_texture: &Texture2D) {
    rl_draw_handle.clear_background(COLOR_DARK);
    let screen_width = rl_draw_handle.get_screen_width();
    let screen_height = rl_draw_handle.get_screen_height();
    for i in 0..=screen_width / tile_texture.width() {
        for j in 0..=screen_height / tile_texture.height() {
            rl_draw_handle.draw_texture(
                tile_texture,
                i * tile_texture.width(),
                j * tile_texture.height(),
                COLOR_LIGHT,
            );
        }
    }
    rl_draw_handle.draw_text(
        "CREDITS: Captainfl4me",
        screen_width - rl_draw_handle.measure_text("CREDITS: Captainfl4me", 32) - 20,
        screen_height - 40,
        32,
        COLOR_BLACK,
    );
}

const T_ANIMATION_SPEED: f32 = 0.005;
fn bezier_curve_scene(
    rl_handle: &mut RaylibHandle,
    rl_thread: &RaylibThread,
    tile_texture: &Texture2D,
) {
    // Initialize
    let mut points = [
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
    let mut animated = true;
    let mut animation_bounce = false;
    let mut has_point_selected = false;
    let mut debug_draw = true;
    let mut clock_divider = 0;

    let mut t = 0.5;
    let left_slider_text = CStr::from_bytes_with_nul(b"0.0\0").unwrap();
    let right_slider_text = CStr::from_bytes_with_nul(b"1.0\0").unwrap();
    let animation_toggle_text = CStr::from_bytes_with_nul(b"Animate T value\0").unwrap();
    let debug_text = CStr::from_bytes_with_nul(b"Activate debug draw\0").unwrap();
    let mut current_draw_time_text = String::new();

    while !rl_handle.window_should_close() {
        // Update inputs
        let mouse_position = Vector2::new(
            rl_handle.get_mouse_x() as f32,
            rl_handle.get_mouse_y() as f32,
        );

        // Scene computation
        // Update points
        for point in points.iter_mut() {
            point.udpate_gui(mouse_position);
            if point.is_selected {
                point.set_position(mouse_position, false);
            }
        }
        if rl_handle.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            if !has_point_selected {
                for point in points.iter_mut() {
                    if point.is_hovered {
                        point.is_selected = true;
                        has_point_selected = true;
                        break;
                    }
                }
            }
        } else if has_point_selected {
            for point in points.iter_mut() {
                point.is_selected = false;
            }
            has_point_selected = false;
        }

        if let Some(key) = rl_handle.get_key_pressed() {
            if !has_point_selected {
                match key {
                    KeyboardKey::KEY_SPACE => {
                        if points.len() < 62 {
                            // Prevent binomial overflow
                            points.last_mut().unwrap().color = COLOR_LIGHT;
                            let new_point = BasicPoint::new(mouse_position, COLOR_BLUE);
                            points.push(new_point);
                        }
                    }
                    KeyboardKey::KEY_BACKSPACE => {
                        if points.len() > 2 {
                            points.pop();
                            points.last_mut().unwrap().color = COLOR_BLUE;
                        }
                    }
                    KeyboardKey::KEY_ESCAPE => {
                        break;
                    }
                    _ => {}
                }
            }
        }
        let show_help_page = rl_handle.is_key_down(KeyboardKey::KEY_H);

        // Update Frame
        let mut rl_draw_handle = rl_handle.begin_drawing(rl_thread);
        let screen_width = rl_draw_handle.get_screen_width();
        draw_background(&mut rl_draw_handle, tile_texture);

        let current_fps_text = format!("{} FPS", rl_draw_handle.get_fps());
        rl_draw_handle.draw_text(
            current_fps_text.as_str(),
            screen_width - rl_draw_handle.measure_text(current_fps_text.as_str(), 14) - 30,
            20,
            14,
            COLOR_GREEN,
        );

        // Draw GUI Controls
        rl_draw_handle.gui_toggle(
            Rectangle::new(30.0, 20.0, 200.0, 25.0),
            Some(debug_text),
            &mut debug_draw,
        );
        if debug_draw {
            rl_draw_handle.gui_slider_bar(
                Rectangle::new(30.0, 50.0, 200.0, 25.0),
                Some(left_slider_text),
                Some(right_slider_text),
                &mut t,
                0.0,
                1.0,
            );
            rl_draw_handle.gui_toggle(
                Rectangle::new(30.0, 80.0, 200.0, 25.0),
                Some(animation_toggle_text),
                &mut animated,
            );
        }

        let draw_time_start = SystemTime::now();
        draw_bezier(
            &points,
            &mut rl_draw_handle,
            if debug_draw { Some(t) } else { None },
        );
        if clock_divider == 0 {
            current_draw_time_text = format!("{:?}", draw_time_start.elapsed().unwrap());
        }
        rl_draw_handle.draw_text(
            current_draw_time_text.as_str(),
            screen_width - rl_draw_handle.measure_text(current_draw_time_text.as_str(), 14) - 30,
            50,
            14,
            COLOR_LIGHT,
        );

        if show_help_page {
            draw_help_page(&mut rl_draw_handle);
        }

        // Update Animation
        if animated {
            t += if animation_bounce {
                -T_ANIMATION_SPEED
            } else {
                T_ANIMATION_SPEED
            };
            if t >= 1.0 {
                t = 1.0;
                animation_bounce = true;
            } else if t <= 0.0 {
                t = 0.0;
                animation_bounce = false;
            }
        }

        clock_divider += 1;
        if clock_divider >= 60 {
            clock_divider = 0;
        }
    }
}

fn bezier_spline_scene(
    rl_handle: &mut RaylibHandle,
    rl_thread: &RaylibThread,
    tile_texture: &Texture2D,
) {
    // Initialize
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
    let linked_control_point_ref = &points[1];
    points[0].borrow_mut().set_constraint(
        JoinPointConstraintID::NextControlPoint as usize,
        linked_control_point_ref,
    );
    let linked_join_point_ref = &points[3];
    points[2].borrow_mut().set_constraint(
        ControlPointConstraintID::MirrorJoinPoint as usize,
        linked_join_point_ref,
    );

    let mut animated = true;
    let mut animation_bounce = false;
    let mut has_point_selected = false;
    let mut debug_draw = true;
    let mut draw_bounding_box = false;
    let mut lock_move = true;
    let mut clock_divider = 0;
    let mut is_closed_loop = false;

    let mut t = 0.5;
    let left_slider_text = CStr::from_bytes_with_nul(b"0.0\0").unwrap();
    let right_slider_text = CStr::from_bytes_with_nul(b"1.0\0").unwrap();
    let animation_toggle_text = CStr::from_bytes_with_nul(b"Animate T value\0").unwrap();
    let bounding_box_toggle_text = CStr::from_bytes_with_nul(b"Draw Bouding box\0").unwrap();
    let debug_text = CStr::from_bytes_with_nul(b"Activate debug draw\0").unwrap();
    let lock_move_text = CStr::from_bytes_with_nul(b"Lock points movement\0").unwrap();
    let mut current_draw_time_text = String::new();

    while !rl_handle.window_should_close() {
        // Update inputs
        let mouse_position = Vector2::new(
            rl_handle.get_mouse_x() as f32,
            rl_handle.get_mouse_y() as f32,
        );

        // Scene computation
        // Update points
        for point in points.iter_mut() {
            point.borrow_mut().udpate_gui(mouse_position);
            if point.borrow().is_selected() {
                point.borrow_mut().set_position(mouse_position, lock_move);
            }
        }
        if rl_handle.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            if !has_point_selected {
                for point in points.iter_mut() {
                    if point.borrow().is_hovered() {
                        point.borrow_mut().set_selected(true);
                        has_point_selected = true;
                        break;
                    }
                }
            }
        } else if has_point_selected {
            for point in points.iter_mut() {
                point.borrow_mut().set_selected(false);
            }
            has_point_selected = false;
        }

        if let Some(key) = rl_handle.get_key_pressed() {
            if !has_point_selected {
                match key {
                    KeyboardKey::KEY_SPACE => {
                        if !is_closed_loop {
                            let pm1_pos = points[points.len() - 1].borrow().get_position();
                            let pm2_pos = points[points.len() - 2].borrow().get_position();
                            // First control point is mirrored with n-1 control point
                            let new_point = RefCell::new(Box::new(ControlPoint::new(
                                pm1_pos * 2.0 - pm2_pos,
                                Some(&points[points.len() - 2]),
                                Some(&points[points.len() - 1]),
                            )));
                            points.push(Rc::new(new_point));
                            // Update next control point ref on previous join
                            points[points.len() - 2].borrow_mut().set_constraint(
                                JoinPointConstraintID::NextControlPoint as usize,
                                &points[points.len() - 1],
                            );
                            points[points.len() - 3].borrow_mut().set_constraint(
                                ControlPointConstraintID::LinkedControlPoint as usize,
                                &points[points.len() - 1],
                            );

                            // Second control point between new point and last (n) control
                            points.push(Rc::new(RefCell::new(Box::new(ControlPoint::new(
                                (mouse_position + pm1_pos) * 0.5,
                                None,
                                None,
                            )))));
                            points.push(Rc::new(RefCell::new(Box::new(JoinPoint::new(
                                mouse_position,
                                Some(&points[points.len() - 1]),
                                None,
                            )))));
                            let linked_join_point_ref = &points[points.len() - 1];
                            points[points.len() - 2].borrow_mut().set_constraint(
                                ControlPointConstraintID::MirrorJoinPoint as usize,
                                linked_join_point_ref,
                            );
                        }
                    }
                    KeyboardKey::KEY_BACKSPACE => {
                        if points.len() > 4 {
                            for _ in 0..if is_closed_loop { 2 } else { 3 } {
                                points.pop();
                            }
                            is_closed_loop = false;
                        }
                    }
                    KeyboardKey::KEY_ENTER => {
                        if !is_closed_loop {
                            let pm1_pos = points[points.len() - 1].borrow().get_position();
                            let pm2_pos = points[points.len() - 2].borrow().get_position();
                            // First control point is mirrored with n-1 control point
                            let new_point = RefCell::new(Box::new(ControlPoint::new(
                                pm1_pos * 2.0 - pm2_pos,
                                Some(&points[points.len() - 2]),
                                Some(&points[points.len() - 1]),
                            )));
                            points.push(Rc::new(new_point));
                            // Update next control point ref on previous join
                            points[points.len() - 2].borrow_mut().set_constraint(
                                JoinPointConstraintID::NextControlPoint as usize,
                                &points[points.len() - 1],
                            );
                            points[points.len() - 3].borrow_mut().set_constraint(
                                ControlPointConstraintID::LinkedControlPoint as usize,
                                &points[points.len() - 1],
                            );

                            let pm1_pos = points[0].borrow().get_position();
                            let pm2_pos = points[1].borrow().get_position();
                            // Second control point is mirrored with first control point (arr[1])
                            points.push(Rc::new(RefCell::new(Box::new(ControlPoint::new(
                                pm1_pos * 2.0 - pm2_pos,
                                Some(&points[1]),
                                Some(&points[0]),
                            )))));
                            points[0].borrow_mut().set_constraint(
                                JoinPointConstraintID::PreviousControlPoint as usize,
                                &points[points.len() - 1],
                            );
                            points[1].borrow_mut().set_constraint(
                                ControlPointConstraintID::LinkedControlPoint as usize,
                                &points[points.len() - 1],
                            );
                            is_closed_loop = true;
                        }
                    }
                    KeyboardKey::KEY_ESCAPE => {
                        break;
                    }
                    _ => {}
                }
            }
        }
        let show_help_page = rl_handle.is_key_down(KeyboardKey::KEY_H);

        // Update Frame
        let mut rl_draw_handle = rl_handle.begin_drawing(rl_thread);
        let screen_width = rl_draw_handle.get_screen_width();
        draw_background(&mut rl_draw_handle, tile_texture);

        let current_fps_text = format!("{} FPS", rl_draw_handle.get_fps());
        rl_draw_handle.draw_text(
            current_fps_text.as_str(),
            screen_width - rl_draw_handle.measure_text(current_fps_text.as_str(), 14) - 30,
            20,
            14,
            COLOR_GREEN,
        );

        // Draw GUI Controls
        rl_draw_handle.gui_toggle(
            Rectangle::new(30.0, 20.0, 200.0, 25.0),
            Some(debug_text),
            &mut debug_draw,
        );
        if debug_draw {
            rl_draw_handle.gui_slider_bar(
                Rectangle::new(30.0, 50.0, 200.0, 25.0),
                Some(left_slider_text),
                Some(right_slider_text),
                &mut t,
                0.0,
                1.0,
            );
            rl_draw_handle.gui_toggle(
                Rectangle::new(30.0, 80.0, 200.0, 25.0),
                Some(animation_toggle_text),
                &mut animated,
            );
            rl_draw_handle.gui_toggle(
                Rectangle::new(30.0, 110.0, 200.0, 25.0),
                Some(bounding_box_toggle_text),
                &mut draw_bounding_box,
            );
        }
        rl_draw_handle.gui_toggle(
            Rectangle::new(30.0, if debug_draw { 140.0 } else { 50.0 }, 200.0, 25.0),
            Some(lock_move_text),
            &mut lock_move,
        );

        let draw_time_start = SystemTime::now();
        for cubic_bezier_points in points.windows(4).step_by(3) {
            let cubic_bezier_points = cubic_bezier_points
                .iter()
                .map(|b| b.borrow().downcast_basic_point())
                .collect::<Vec<_>>();
            draw_bezier(
                &cubic_bezier_points,
                &mut rl_draw_handle,
                if debug_draw { Some(t) } else { None },
            );
            if draw_bounding_box {
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
        if is_closed_loop {
            let cubic_bezier_points = [
                &points[points.len() - 3],
                &points[points.len() - 2],
                &points[points.len() - 1],
                &points[0],
            ]
            .iter()
            .map(|b| b.borrow().downcast_basic_point())
            .collect::<Vec<_>>();
            draw_bezier(
                &cubic_bezier_points,
                &mut rl_draw_handle,
                if debug_draw { Some(t) } else { None },
            );

            if draw_bounding_box {
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

        if clock_divider == 0 {
            current_draw_time_text = format!("{:?}", draw_time_start.elapsed().unwrap());
        }
        rl_draw_handle.draw_text(
            current_draw_time_text.as_str(),
            screen_width - rl_draw_handle.measure_text(current_draw_time_text.as_str(), 14) - 30,
            50,
            14,
            COLOR_LIGHT,
        );

        if show_help_page {
            draw_help_page(&mut rl_draw_handle);
        }

        // Update Animation
        if animated {
            t += if animation_bounce {
                -T_ANIMATION_SPEED
            } else {
                T_ANIMATION_SPEED
            };
            if t >= 1.0 {
                t = 1.0;
                animation_bounce = true;
            } else if t <= 0.0 {
                t = 0.0;
                animation_bounce = false;
            }
        }

        clock_divider += 1;
        if clock_divider >= 60 {
            clock_divider = 0;
        }
    }
}

pub fn draw_help_page(rl_draw_handle: &mut RaylibDrawHandle) {
    let screen_width = rl_draw_handle.get_screen_width();
    let screen_height = rl_draw_handle.get_screen_height();

    let text_to_display = ["ESC - Go back to main menu", "SPACE - Add new (point or spline part) on mouse position", "BACKSPACE - Remove last (point or spline part)", "ENTER (spline only) - Close spline", "MOUSE CLICK - move point"];

    let panel_width: i32 = 600;
    let panel_height: i32 = 10 + 32 + 10 + 20*(text_to_display.len() as i32) + 10;
    let panel_x = (screen_width - panel_width) / 2;
    let panel_y = (screen_height - panel_height) / 2;
    rl_draw_handle.draw_rectangle_rounded(
        Rectangle::new(
            panel_x as f32,
            panel_y as f32,
            panel_width as f32,
            panel_height as f32,
        ),
        0.1,
        20,
        COLOR_DARK,
    );
    
    rl_draw_handle.draw_text("HELP", panel_x + (panel_width - rl_draw_handle.measure_text("HELP", 32))/2, panel_y + 10, 32, COLOR_LIGHT);
    for (i, text) in text_to_display.iter().enumerate() {
        rl_draw_handle.draw_text(text, panel_x + 10, panel_y + 52 + (i as i32)*20, 18, COLOR_LIGHT);
    }
}
