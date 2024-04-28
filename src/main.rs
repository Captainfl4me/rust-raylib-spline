use raylib::prelude::*;

fn main() {
    let (mut rl_handle, rl_thread) = raylib::init()
        .size(640, 480)
        .resizable()
        .title("Hello, World")
        .build();
    rl_handle.set_target_fps(60);
    rl_handle.set_window_state(rl_handle.get_window_state().set_window_maximized(true));

    // Initialize
    let points = [
        Vector2::new(300.0, 600.0),
        Vector2::new(600.0, 300.0),
        Vector2::new(900.0, 600.0),
    ];
    let mut curve = BezierCurve::new(points);

    while !rl_handle.window_should_close() {
        let mouse_position = Vector2::new(
            rl_handle.get_mouse_x() as f32,
            rl_handle.get_mouse_y() as f32,
        );

        curve.update(mouse_position);
        if rl_handle.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
            curve.select_hovered_point();
        } else if curve.has_point_selected {
            curve.unselect_all_points();
        }

        let mut rl_draw_handle = rl_handle.begin_drawing(&rl_thread);

        #[cfg(debug_assertions)]
        rl_draw_handle.draw_fps(12, 12);

        curve.draw(&mut rl_draw_handle);

        rl_draw_handle.clear_background(Color::BLACK);
        // d.draw_text("Hello, world!", 12, 12, 20, Color::BLACK);
    }
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

struct BezierCurve<const N: usize> {
    points: [Point; N],
    has_point_selected: bool,
}
impl<const N: usize> BezierCurve<N> {
    pub fn new(points: [Vector2; N]) -> Self {
        let points = points
            .iter()
            .enumerate()
            .map(|(i, pos)| {
                let color = if i == 0 || i == N - 1 {
                    Color::BLUE
                } else {
                    Color::WHITE
                };
                Point::new(*pos, color)
            })
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Self {
            points,
            has_point_selected: false,
        }
    }

    /// Update the curve points for animation
    pub fn update(&mut self, mouse_position: Vector2) {
        for point in self.points.iter_mut() {
            point.udpate(mouse_position);
        }
    }

    /// Select the first hovered point if no point is already selected
    pub fn select_hovered_point(&mut self) {
        if self.has_point_selected {
            return;
        }

        for point in self.points.iter_mut() {
            if point.is_hovered {
                point.is_selected = true;
                self.has_point_selected = true;
                break;
            }
        }
    }

    /// Unselect all points
    pub fn unselect_all_points(&mut self) {
        for point in self.points.iter_mut() {
            point.is_selected = false;
        }
        self.has_point_selected = false;
    }

    /// Draw the curve
    pub fn draw(&self, d: &mut RaylibDrawHandle) {
        for point in self.points.iter() {
            point.draw(d);
        }
    }

    /* pub fn mouse_can_select_point(&self, mouse_position: Vector2) -> Option<usize> {
            let mut closest_point = 0;
            let mut closest_distance = mouse_position.distance_to(self.points[0]);
            for (i, point) in self.points.iter().skip(1).enumerate() {
                let distance = mouse_position.distance_to(*point);
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_point = i;
                }
            }
            if closest_distance < POINTS_RADIUS {
                return Some(closest_point);
            }
            None
        }
    */
    /* pub fn move_point(&mut self, point: usize, new_position: Vector2) {
        self.points[point] = new_position;
    } */
}
