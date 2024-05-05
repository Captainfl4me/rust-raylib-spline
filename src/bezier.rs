use raylib::prelude::*;

pub fn binomial(n: u64, k: u64) -> u64 {
    if n >= 63 {
        panic!("N is too great, will overflow!");
    }
    if k > n {
        0
    } else if k == 0 {
        1
    } else {
        n * binomial(n - 1, n - k) / k
    }
}

const POINTS_RADIUS: f32 = 10.0;
const POINTS_RADIUS_HOVER: f32 = 15.0;
const ANIMATION_SPEED: f32 = 1.0;
#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub position: Vector2,
    pub radius: f32,
    pub color: Color,
    pub is_selected: bool,
    pub is_hovered: bool,
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
pub fn evalute_bezier_curve(points: &[Point], t: f32) -> Vector2 {
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
pub fn draw_bezier(points: &[Point], d: &mut RaylibDrawHandle, t: Option<f32>) {
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
        let rec_size = Vector2::new(POINTS_RADIUS, POINTS_RADIUS);
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
                d.draw_rectangle_v(*p - rec_size * 0.5, rec_size, Color::GREEN);
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
        d.draw_line_ex(line_points[0], line_points[1], 3.0, Color::GREEN);
    }

    if let Some(final_point) = final_point {
        d.draw_circle_v(final_point, POINTS_RADIUS / 2.0, Color::YELLOW);
    }

    for point in points.iter() {
        point.draw(d);
    }
}
