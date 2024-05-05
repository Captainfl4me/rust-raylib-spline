use ::core::cell::RefCell;
use raylib::prelude::*;
use std::rc::Rc;

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
pub struct BasicPoint {
    pub position: Vector2,
    pub radius: f32,
    pub color: Color,
    pub is_selected: bool,
    pub is_hovered: bool,
}
impl BasicPoint {
    pub fn new(position: Vector2, color: Color) -> Self {
        Self {
            position,
            radius: POINTS_RADIUS,
            color,
            is_selected: false,
            is_hovered: false,
        }
    }
}
impl Point for BasicPoint {
    fn get_position(&self) -> Vector2 {
        self.position
    }
}
impl MovablePoint for BasicPoint {
    fn set_position(&mut self, position: Vector2, _with_constraint: bool) {
        self.position = position;
    }
}
impl PointGui for BasicPoint {
    fn get_radius(&self) -> f32 {
        self.radius
    }
    fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }
    fn get_color(&self) -> Color {
        self.color
    }
    fn is_hovered(&self) -> bool {
        self.is_hovered
    }
    fn set_hover_state(&mut self, state: bool) {
        self.is_hovered = state;
    }
    fn is_selected(&self) -> bool {
        self.is_selected
    }
    fn set_selected(&mut self, state: bool) {
        self.is_selected = state;
    }
}

#[derive(Clone)]
pub struct JoinPoint {
    position: Vector2,
    radius: f32,
    is_selected: bool,
    is_hovered: bool,

    previous_control_point: Option<Rc<RefCell<Box<dyn MovableGuiPoint>>>>,
    next_control_point: Option<Rc<RefCell<Box<dyn MovableGuiPoint>>>>,
}
impl JoinPoint {
    pub fn new(
        position: Vector2,
        previous_control_point: Option<&Rc<RefCell<Box<dyn MovableGuiPoint>>>>,
        next_control_point: Option<&Rc<RefCell<Box<dyn MovableGuiPoint>>>>,
    ) -> Self {
        Self {
            position,
            radius: POINTS_RADIUS,
            is_selected: false,
            is_hovered: false,
            previous_control_point: previous_control_point.cloned(),
            next_control_point: next_control_point.cloned(),
        }
    }
}
impl Point for JoinPoint {
    fn get_position(&self) -> Vector2 {
        self.position
    }
}
impl MovablePoint for JoinPoint {
    fn set_position(&mut self, position: Vector2, with_constraint: bool) {
        let movement_diff = self.position - position;
        self.position = position;

        if with_constraint {
            if let Some(previous_control_point) = &self.previous_control_point {
                let previous_position = previous_control_point.borrow().get_position();
                previous_control_point
                    .borrow_mut()
                    .set_position(previous_position - movement_diff, false);
            }
            if let Some(next_control_point) = &self.next_control_point {
                let next_position = next_control_point.borrow().get_position();
                next_control_point
                    .borrow_mut()
                    .set_position(next_position - movement_diff, false);
            }
        }
    }
}
impl PointGui for JoinPoint {
    fn get_radius(&self) -> f32 {
        self.radius
    }
    fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }
    fn get_color(&self) -> Color {
        Color::BLUE
    }
    fn is_hovered(&self) -> bool {
        self.is_hovered
    }
    fn set_hover_state(&mut self, state: bool) {
        self.is_hovered = state;
    }
    fn is_selected(&self) -> bool {
        self.is_selected
    }
    fn set_selected(&mut self, state: bool) {
        self.is_selected = state;
    }
}
pub enum JoinPointConstraintID {
    PreviousControlPoint = 0,
    NextControlPoint = 1,
}
impl TryFrom<usize> for JoinPointConstraintID {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::PreviousControlPoint),
            1 => Ok(Self::NextControlPoint),
            _ => Err(()),
        }
    }
}
impl MovableGuiPoint for JoinPoint {
    fn downcast_basic_point(&self) -> BasicPoint {
        let mut basic_point = BasicPoint::new(self.position, self.get_color());
        basic_point.radius = self.radius;
        basic_point.is_selected = self.is_selected;
        basic_point.is_hovered = self.is_hovered;
        basic_point
    }

    fn set_constraint(
        &mut self,
        constraint_id: usize,
        constraint: &Rc<RefCell<Box<dyn MovableGuiPoint>>>,
    ) {
        match constraint_id.try_into() {
            Ok(JoinPointConstraintID::PreviousControlPoint) => {
                self.previous_control_point = Some(constraint.clone())
            }
            Ok(JoinPointConstraintID::NextControlPoint) => {
                self.next_control_point = Some(constraint.clone())
            }
            Err(_) => {}
        };
    }
}

#[derive(Clone)]
pub struct ControlPoint {
    position: Vector2,
    radius: f32,
    is_selected: bool,
    is_hovered: bool,

    linked_control_point: Option<Rc<RefCell<Box<dyn MovableGuiPoint>>>>,
    mirror_join_point: Option<Rc<RefCell<Box<dyn MovableGuiPoint>>>>,
}
impl ControlPoint {
    pub fn new(
        position: Vector2,
        linked_point: Option<&Rc<RefCell<Box<dyn MovableGuiPoint>>>>,
        mirror_join_point: Option<&Rc<RefCell<Box<dyn MovableGuiPoint>>>>,
    ) -> Self {
        Self {
            position,
            radius: POINTS_RADIUS,
            is_selected: false,
            is_hovered: false,
            linked_control_point: linked_point.cloned(),
            mirror_join_point: mirror_join_point.cloned(),
        }
    }
}
impl Point for ControlPoint {
    fn get_position(&self) -> Vector2 {
        self.position
    }
}
impl MovablePoint for ControlPoint {
    fn set_position(&mut self, position: Vector2, with_constraint: bool) {
        self.position = position;

        if with_constraint {
            if let Some(linked_point) = &self.linked_control_point {
                let join_position = self
                    .mirror_join_point
                    .as_ref()
                    .unwrap()
                    .borrow()
                    .get_position();
                linked_point
                    .borrow_mut()
                    .set_position(join_position * 2.0 - position, false);
            }
        }
    }
}
impl PointGui for ControlPoint {
    fn get_radius(&self) -> f32 {
        self.radius
    }
    fn set_radius(&mut self, radius: f32) {
        self.radius = radius;
    }
    fn get_color(&self) -> Color {
        Color::WHITE
    }
    fn is_hovered(&self) -> bool {
        self.is_hovered
    }
    fn set_hover_state(&mut self, state: bool) {
        self.is_hovered = state;
    }
    fn is_selected(&self) -> bool {
        self.is_selected
    }
    fn set_selected(&mut self, state: bool) {
        self.is_selected = state;
    }
}
pub enum ControlPointConstraintID {
    LinkedControlPoint = 0,
    MirrorJoinPoint = 1,
}
impl TryFrom<usize> for ControlPointConstraintID {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::LinkedControlPoint),
            1 => Ok(Self::MirrorJoinPoint),
            _ => Err(()),
        }
    }
}
impl MovableGuiPoint for ControlPoint {
    fn downcast_basic_point(&self) -> BasicPoint {
        let mut basic_point = BasicPoint::new(self.position, self.get_color());
        basic_point.radius = self.radius;
        basic_point.is_selected = self.is_selected;
        basic_point.is_hovered = self.is_hovered;
        basic_point
    }
    fn set_constraint(
        &mut self,
        constraint_id: usize,
        constraint: &Rc<RefCell<Box<dyn MovableGuiPoint>>>,
    ) {
        match constraint_id.try_into() {
            Ok(ControlPointConstraintID::LinkedControlPoint) => {
                self.linked_control_point = Some(constraint.clone())
            }
            Ok(ControlPointConstraintID::MirrorJoinPoint) => {
                self.mirror_join_point = Some(constraint.clone())
            }
            Err(_) => {}
        };
    }
}

pub trait Point {
    fn get_position(&self) -> Vector2;
}

pub trait PointGui: Point {
    fn get_radius(&self) -> f32;
    fn set_radius(&mut self, radius: f32);
    fn get_color(&self) -> Color;

    fn is_hovered(&self) -> bool;
    fn set_hover_state(&mut self, state: bool);
    fn is_selected(&self) -> bool;
    fn set_selected(&mut self, state: bool);

    /// Update the point (default implementation)
    fn udpate_gui(&mut self, mouse_position: Vector2) {
        self.set_hover_state(
            mouse_position.distance_to(self.get_position()) < self.get_radius()
                || self.is_selected(),
        );
        if self.is_hovered() {
            if self.get_radius() < POINTS_RADIUS_HOVER {
                self.set_radius(self.get_radius() + ANIMATION_SPEED);
            } else if self.get_radius() > POINTS_RADIUS_HOVER {
                self.set_radius(POINTS_RADIUS_HOVER);
            }
        } else if self.get_radius() > POINTS_RADIUS {
            self.set_radius(self.get_radius() - ANIMATION_SPEED);
        } else if self.get_radius() < POINTS_RADIUS {
            self.set_radius(POINTS_RADIUS);
        }
    }

    /// Draw the point (default implementation)
    fn draw(&self, d: &mut RaylibDrawHandle) {
        d.draw_circle_v(self.get_position(), self.get_radius(), self.get_color());
    }
}

pub trait MovablePoint {
    fn set_position(&mut self, position: Vector2, with_constraint: bool);
}

pub trait MovableGuiPoint: MovablePoint + PointGui {
    fn downcast_basic_point(&self) -> BasicPoint;
    fn set_constraint(
        &mut self,
        constraint_id: usize,
        constraint: &Rc<RefCell<Box<dyn MovableGuiPoint>>>,
    );
}

const SAMPLES: usize = 50;

/// Evaluate a point on the curve
pub fn evalute_bezier_curve(points: &[impl Point], t: f32) -> Vector2 {
    let n = points.len() - 1;
    let tuple_point = points.iter().enumerate().fold((0.0, 0.0), |acc, (i, e)| {
        let a = (binomial(n as u64, i as u64) as f32)
            * (1.0 - t).powi((n - i) as i32)
            * t.powi(i as i32);
        (
            acc.0 + e.get_position().x * a,
            acc.1 + e.get_position().y * a,
        )
    });
    Vector2::new(tuple_point.0, tuple_point.1)
}

/// Draw the curve
pub fn draw_bezier(points: &[impl PointGui], d: &mut RaylibDrawHandle, t: Option<f32>) {
    for line_points in points.windows(2) {
        d.draw_line_ex(
            line_points[0].get_position(),
            line_points[1].get_position(),
            3.0,
            Color::RED,
        );
    }

    let mut final_point = None;
    if let Some(t) = t {
        let rec_size = Vector2::new(POINTS_RADIUS, POINTS_RADIUS);
        let mut debug_points: Vec<Vector2> =
            points.iter().map(|p| p.get_position()).collect::<Vec<_>>();
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
