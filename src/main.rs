use raylib::prelude::*;
use std::ffi::CStr;

const T_ANIMATION_SPEED: f32 = 0.005;
fn main() {
    let (mut rl_handle, rl_thread) = raylib::init()
        .size(640, 480)
        .resizable()
        .title("Hello, World")
        .build();
    rl_handle.set_target_fps(60);
    rl_handle.set_window_state(rl_handle.get_window_state().set_window_maximized(true));

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

    let mut t = 0.5;
    let left_slider_text = CStr::from_bytes_with_nul(b"0.0\0").unwrap();
    let right_slider_text = CStr::from_bytes_with_nul(b"1.0\0").unwrap();
    let animation_toggle_text = CStr::from_bytes_with_nul(b"Animate T value\0").unwrap();

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
        if rl_handle.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
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
                        points.last_mut().unwrap().color = Color::WHITE;
                        let new_point = Point::new(mouse_position, Color::BLUE);
                        points.push(new_point);
                    }
                    KeyboardKey::KEY_BACKSPACE => {
                        if points.len() > 2 {
                            points.pop();
                            points.last_mut().unwrap().color = Color::BLUE;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Update Frame
        let mut rl_draw_handle = rl_handle.begin_drawing(&rl_thread);
        let screen_width = rl_draw_handle.get_screen_width();

        #[cfg(debug_assertions)]
        rl_draw_handle.draw_fps(screen_width - 100, 10);

        // Draw GUI Controls
        t = rl_draw_handle.gui_slider_bar(
            Rectangle::new(30.0, 20.0, 200.0, 25.0),
            Some(left_slider_text),
            Some(right_slider_text),
            t,
            0.0,
            1.0,
        );
        animated = rl_draw_handle.gui_toggle(
            Rectangle::new(30.0, 50.0, 200.0, 25.0),
            Some(animation_toggle_text),
            animated,
        );

        draw_bezier(&points, &mut rl_draw_handle, Some(t));

        rl_draw_handle.clear_background(Color::BLACK);

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
    }
}

fn binomial(n: u64, k: u64) -> u64 {
    if n > 67 {
        panic!("n is too large");
    }
    let mut result = 1;
    for i in 0..k {
        result *= n - i;
        result /= i + 1;
    }
    result
}

const POINTS_RADIUS: f32 = 10.0;
const POINTS_RADIUS_HOVER: f32 = 15.0;
const ANIMATION_SPEED: f32 = 1.0;
#[derive(Debug, Clone, Copy)]
struct Point {
    position: Vector2,
    radius: f32,
    color: Color,
    is_selected: bool,
    is_hovered: bool,
}
impl Point {
    pub fn new(position: Vector2, color: Color) -> Self {
        Self {
            position,
            radius: POINTS_RADIUS,
            color,
            is_selected: false,
            is_hovered: false,
        }
    }

    pub fn udpate(&mut self, mouse_position: Vector2) {
        self.is_hovered =
            mouse_position.distance_to(self.position) < self.radius || self.is_selected;
        if self.is_hovered {
            if self.radius < POINTS_RADIUS_HOVER {
                self.radius += ANIMATION_SPEED;
            } else if self.radius > POINTS_RADIUS_HOVER {
                self.radius = POINTS_RADIUS_HOVER;
            }
        } else if self.radius > POINTS_RADIUS {
            self.radius -= ANIMATION_SPEED;
        } else if self.radius < POINTS_RADIUS {
            self.radius = POINTS_RADIUS;
        }

        if self.is_selected {
            self.position = mouse_position;
        }
    }

    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        d.draw_circle_v(self.position, self.radius, self.color);
    }
}

const SAMPLES: usize = 50;

/// Evaluate a point on the curve
fn evalute_bezier_curve(points: &[Point], t: f32) -> Vector2 {
    let n = points.len() - 1;
    let tuple_point = points.iter().enumerate().fold((0.0, 0.0), |acc, (i, e)| {
        let a = (binomial(n as u64, i as u64) as f32)
            * (1.0 - t).powi((n - i) as i32)
            * t.powi(i as i32);
        (acc.0 + e.position.x * a, acc.1 + e.position.y * a)
    });
    Vector2::new(tuple_point.0, tuple_point.1)
}

/// Draw the curve
fn draw_bezier(points: &[Point], d: &mut RaylibDrawHandle, t: Option<f32>) {
    for line_points in points.windows(2) {
        d.draw_line_ex(
            line_points[0].position,
            line_points[1].position,
            3.0,
            Color::RED,
        );
    }

    let mut final_point = None;
    if let Some(t) = t {
        let mut debug_points: Vec<Vector2> = points.iter().map(|p| p.position).collect::<Vec<_>>();
        while debug_points.len() > 2 {
            let next_points = debug_points
                .windows(2)
                .map(|w| w[0].lerp(w[1], t))
                .collect::<Vec<_>>();
            // Drawing lines before points so that points will override them
            for p in next_points.windows(2) {
                d.draw_line_ex(p[0], p[1], 2.0, Color::RED);
            }
            // Draw lerp points for this run
            for p in next_points.iter() {
                d.draw_circle_v(p, POINTS_RADIUS / 2.0, Color::GREEN);
            }
            debug_points = next_points;
        }
        final_point = Some(debug_points[0].lerp(debug_points[1], t));
    }

    let step = 1.0 / SAMPLES as f32;
    let step_points = (0..=SAMPLES)
        .map(|i| evalute_bezier_curve(points, i as f32 * step))
        .collect::<Vec<_>>();

    for line_points in step_points.windows(2) {
        d.draw_line_ex(line_points[0], line_points[1], 4.0, Color::GREEN);
    }

    if let Some(final_point) = final_point {
        d.draw_circle_v(final_point, POINTS_RADIUS / 2.0, Color::YELLOW);
    }

    for point in points.iter() {
        point.draw(d);
    }
}
