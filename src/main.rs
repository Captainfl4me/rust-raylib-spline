use raylib::prelude::*;
use std::ffi::CStr;
use std::time::SystemTime;

mod bezier;
use bezier::*;

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

    const TITLE_FONT_SIZE: i32 = 60;
    let mut scene_to_load = None;
    while !rl_handle.window_should_close() {
        {
            let mut rl_draw_handle = rl_handle.begin_drawing(&rl_thread);
            let screen_width = rl_draw_handle.get_screen_width();
            let screen_height = rl_draw_handle.get_screen_height();
            draw_background(&mut rl_draw_handle);

            rl_draw_handle.draw_text(
                "Spline Drawer",
                (screen_width - rl_draw_handle.measure_text("Spline Drawer", TITLE_FONT_SIZE)) / 2,
                (screen_height - TITLE_FONT_SIZE) / 2,
                TITLE_FONT_SIZE,
                Color::WHITE,
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
                    (screen_height + TITLE_FONT_SIZE*2 + 10) as f32 / 2.0,
                    200.0,
                    25.0,
                ),
                Some(CStr::from_bytes_with_nul(b"Load Bezier spline\0").unwrap()),
            ) {
                scene_to_load = Some(Scenes::BezierSpline);
            }
        }

        match scene_to_load {
            Some(Scenes::BezierCurve) => bezier_curve_scene(&mut rl_handle, &rl_thread),
            Some(Scenes::BezierSpline) => bezier_curve_scene(&mut rl_handle, &rl_thread),
            None => {}
        }
        scene_to_load = None;
    }
}

fn draw_background(rl_draw_handle: &mut RaylibDrawHandle) {
    rl_draw_handle.clear_background(Color::BLACK);
}

const T_ANIMATION_SPEED: f32 = 0.005;
fn bezier_curve_scene(rl_handle: &mut RaylibHandle, rl_thread: &RaylibThread) {
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
            Color::BLUE
        } else {
            Color::WHITE
        };
        Point::new(*pos, color)
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
            point.udpate(mouse_position);
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
                            points.last_mut().unwrap().color = Color::WHITE;
                            let new_point = Point::new(mouse_position, Color::BLUE);
                            points.push(new_point);
                        }
                    }
                    KeyboardKey::KEY_BACKSPACE => {
                        if points.len() > 2 {
                            points.pop();
                            points.last_mut().unwrap().color = Color::BLUE;
                        }
                    }
                    KeyboardKey::KEY_ESCAPE => { break; }
                    _ => {}
                }
            }
        }

        // Update Frame
        let mut rl_draw_handle = rl_handle.begin_drawing(rl_thread);
        let screen_width = rl_draw_handle.get_screen_width();
        draw_background(&mut rl_draw_handle);

        let current_fps_text = format!("{} FPS", rl_draw_handle.get_fps());
        rl_draw_handle.draw_text(
            current_fps_text.as_str(),
            screen_width - rl_draw_handle.measure_text(current_fps_text.as_str(), 14) - 30,
            20,
            14,
            Color::GREEN,
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
            Color::WHITE,
        );

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
