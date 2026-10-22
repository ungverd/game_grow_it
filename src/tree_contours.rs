use std::f64::MIN;

const MIN_DELTA: f64 = 0.000001;

pub struct LineSegment {
    point1: crate::Pos,
    point2: crate::Pos,
}

pub struct Arc {
    center: crate::Pos,
    radius: f64,
    angle_start: f64,
    angle_stop: f64,
}

pub struct Circle {
    center: crate::Pos,
    radius: f64,
    is_outer: bool,
}

pub enum Line {
    Segment(LineSegment),
    Arc(Arc),
}

pub struct ContourComplex {
    lines: Vec<Line>,
    points: Vec<crate::Pos>,
}

pub enum Contour {
    Circle(Circle),
    ContourComplex(ContourComplex),
}

pub struct Figure {
    contours: Vec<Contour>
}

impl LineSegment {
    pub fn new(point1: crate::Pos, point2: crate::Pos) -> LineSegment {
        LineSegment {
            point1,
            point2
        }
    }
}

impl Arc {
    pub fn new(center: crate::Pos, radius: f64, angle_start:f64, angle_stop:f64) -> Arc {
        Arc {
            center,
            radius,
            angle_start,
            angle_stop,
        }
    }
}

impl Circle {
    pub fn new(center: crate::Pos, radius: f64, is_outer: bool) -> Circle {
        Circle {
            center,
            radius,
            is_outer,
        }
    }
}

impl ContourComplex {
    pub fn new(lines: Vec<Line>, points: Vec<crate::Pos>) -> ContourComplex{
        ContourComplex {
            lines,
            points,
        }
    }
}

impl Contour {
    pub fn new_from_complex(contour: ContourComplex) -> Contour {
        Contour::ContourComplex(contour)
    }
    pub fn new_from_circle(circle: Circle) -> Contour {
        Contour::Circle(circle)
    }
}

impl Figure {
    pub fn new_from_contours(contours: Vec<Contour>) -> Figure {
        Figure {
            contours,
        }
    }
}

struct LineForIntermediate {
    start_point_index: usize,
    stop_point_index: usize,
    line: Line,
}

struct PointForIntermediate {
    pos: crate::Pos,
    lines_starting: Vec<usize>,
    lines_stopping: Vec<usize>
}

struct FigureIntermediate {
    lines: Vec<LineForIntermediate>,
    points: Vec<PointForIntermediate>,
    circles: Vec<Circle>,
}

impl FigureIntermediate {
    fn new(fig_1: Figure, fig_2: Figure) -> FigureIntermediate {
        let mut lines = vec![];
        let mut points = vec![];
        let mut circles = vec![];
        // TODO: if inner cercle and outer circle are of the same center and radius, they collapse
        for fig in [fig_1, fig_2] {
            for contour in fig.contours {
                match contour {
                    Contour::Circle(circle) => {
                        circles.push(circle);
                    }
                    Contour::ContourComplex(cont) => {
                        let first_line_num = lines.len();
                        let first_point_num = points.len();
                        let cont_len = cont.lines.len();
                        for (i, line) in cont.lines.into_iter().enumerate() {
                            let l = LineForIntermediate{
                                start_point_index: first_point_num + i,
                                stop_point_index: first_point_num + (i + 1) % cont_len,
                                line
                            };
                            lines.push(l);
                        }
                        for (i, point) in cont.points.into_iter().enumerate() {
                            let p = PointForIntermediate{
                                pos: point.clone(),
                                lines_starting: vec![first_line_num + i,],
                                lines_stopping: vec![first_line_num + (i + cont_len - 1) % cont_len,],
                            };
                            points.push(p);
                        }
                    }
                }
            }
        }
        FigureIntermediate { lines, points, circles }
    }

    fn merge_points(&mut self, p1: usize, p2: usize) {
        let (first_point, second_point) = if p1 < p2 {(p1, p2)} else {(p2, p1)};
        // second point will disappear
        let points = &mut self.points;
        let last_ind = points.len() - 1;
        let lines = &mut self.lines;
        
        let mut sp_obj = points.swap_remove(second_point);
        // Adjusting point "pointers" in lines
        for line in lines {
            if line.start_point_index == second_point {
                line.start_point_index = first_point;
            } else {
                if line.start_point_index == last_ind {
                    line.start_point_index = second_point;
                }
            }
            if line.stop_point_index == second_point {
                line.stop_point_index = first_point;
            } else {
                if line.stop_point_index == last_ind {
                    line.stop_point_index = second_point;
                }
            }
        }
        let fp_obj = &mut points[first_point];
        // Adjusting line "pointers" in points
        fp_obj.lines_starting.append(&mut sp_obj.lines_starting);
        fp_obj.lines_stopping.append(&mut sp_obj.lines_stopping);
    }

    fn merge_point_line(&mut self, res: ComparePointsLinesResult) {

    }

    fn merge_and_clean(&mut self) {
        let mut finished = false;
        while !finished {
            let mut finished_points = false;
            let mut start_i = 0;
            while !finished_points {
                let res = self.compare_points(start_i);
                match res {
                    None => {
                        finished_points = true;
                    }
                    Some((p1, p2)) => {
                        self.merge_points(p1, p2);
                        start_i = p1;
                    }
                }
            }
        }

    }

    fn compare_points_lines(&self, start_i_point: usize) -> ComparePointsLinesResult {
        for (counter, p) in self.points[start_i_point..].iter().enumerate() {
            let i = counter + start_i_point;
            for (j, line) in self.lines.iter().enumerate() {
                match &line.line {
                    Line::Segment(seg) => {
                        if point_on_line(&seg.point1, &seg.point2, &p.pos) {
                            if check_point_on_segment_line(&p.pos, &seg.point1, &seg.point2) {
                                return ComparePointsLinesResult::PointSegment((i, j));
                            }
                        }
                    }
                    Line::Arc(arc) => {
                        if point_on_circle(&p.pos, &arc.center, arc.radius) {
                            let res = check_point_in_arc(&arc, &p.pos);
                            match res {
                                Some(angle) => {
                                    return ComparePointsLinesResult::PointArc((i, j, angle));
                                }
                                None => {}
                            }
                        } 
                    }
                }
            }
        }
        ComparePointsLinesResult::None
    }
    
    fn compare_points(&self, start_i: usize) -> Option<(usize, usize)> {
        for (counter, p1) in self.points[start_i..self.points.len() - 1].iter().enumerate() {
            let i = counter + start_i;
            for (counter2, p2) in self.points[i + 1..].iter().enumerate() {
                let j = i + 1 + counter2;
                if check_same_point(&p1.pos, &p2.pos) {
                    return Some((i, j));
                }
            }
        }
        None
    }
}

enum ComparePointsLinesResult {
    None,
    PointSegment((usize, usize)),
    PointArc((usize, usize, f64)) // point, arc, angle
}

fn check_same_point(p1: &crate::Pos, p2: &crate::Pos) -> bool {
    let dif = p1.add(&p2.multiply(-1.0));
    dif.x.abs() < MIN_DELTA && dif.y.abs() < MIN_DELTA
}

fn get_y_circles(x1: f64,
                 x2: f64,
                 y1: f64,
                 y2: f64,
                 r1: f64,
                 r2: f64) -> Option<(f64, f64)> {
    let a = 4. * ((y1 - y2).powi(2) + (x1 - x2).powi(2));
    let b = (-4.) * ((y1 - y2) * (y1.powi(2) - y2.powi(2) - r1.powi(2) + r2.powi(2)) + (y1 + y2) * (x1 + x2).powi(2));
    let c = ((y1.powi(2) - r1.powi(2)) - (x1 - x2).powi(2) - (y2.powi(2) - r2.powi(2))).powi(2) + 4. * (y1.powi(2) - r1.powi(2)) * (x1 - x2).powi(2);
    let d = b.powi(2) - 4.*a*c;
    if d > 0.0 {
        let d_root = d.powf(0.5);
        let y_1 = (-b + d_root) * 0.5 / a;
        let y_2 = (-b - d_root) * 0.5 / a;
        Some((y_1, y_2))
    } else {
        if d.abs() < MIN_DELTA {
            let y_1 = -b * 0.5 / a;
            Some((y_1, y_1))
        } else {
            None
        }
    }
}

fn get_x_circles(x1: f64, x2: f64, y1: f64, y2: f64, r1: f64, r2: f64) -> Option<(f64, f64)> {
    get_y_circles(y1, y2, x1, x2, r1, r2)
}

fn get_y_circle_line(x1: f64,
                     x2: f64,
                     xc: f64,
                     y1: f64,
                     y2: f64,
                     yc: f64,
                     r: f64) -> Option<(f64, f64)> {
    let dify = y2 - y1;
    let difx = x2 - x1;
    let dify2 = dify.powi(2);
    let difx2 = difx.powi(2);
    let xy_expr = y1 * x2 - y2 * x1 + xc * dify;
    let a = dify2 + difx2;
    let b = -2. * (yc * dify2 + difx * xy_expr);
    let c = yc.powi(2) * dify2 + xy_expr.powi(2) - r.powi(2) * dify2;
    let d = b.powi(2) - 4. * a * c;

    if d > 0. {
        let d_sqrt = d.powf(0.5);
        let y_1 = (-b + d_sqrt) * 0.5 / a;
        let y_2 = (-b - d_sqrt) * 0.5 / a;
        Some((y_1, y_2))
    } else {
        if d.abs() < MIN_DELTA {
            let y_1 = -b * 0.5 / a;
            Some((y_1, y_1))
        } else {
            None
        }
    }
}

fn get_x_circle_line(x1: f64, x2: f64, xc: f64, y1: f64, y2: f64, yc: f64, r: f64) -> Option<(f64, f64)> {
    get_y_circle_line(y1, y2, yc, x1, x2, xc, r)
}

enum CircleIntersectionResult {
    None,
    OnePoint(crate::Pos),
    TwoPoints((crate::Pos, crate::Pos))
}

fn intersect_circle_circle(c1: &crate::Pos, c2: &crate::Pos, r1: f64, r2: f64) -> CircleIntersectionResult {
    // we assume here that centers are not same point
    let res = get_y_circles(c1.x, c2.x, c1.y, c2.y, r1, r2);
    match res {
        None => {CircleIntersectionResult::None}
        Some((y_big, y_small)) => {
            let res2 = get_x_circles(c1.x, c2.x, c1.y, c2.y, r1, r2);
            match res2 {
                None => {CircleIntersectionResult::None}
                Some((x_big, x_small)) => {
                    let res_p1;
                    let res_p2;
                    if (c1.x > c2.x) == (c1.y > c2.y) {
                        res_p1 = crate::Pos{x: x_big, y: y_small};
                        res_p2 = crate::Pos{x: x_small, y: y_big};
                    } else {
                        res_p1 = crate::Pos{x: x_big, y: y_big};
                        res_p2 = crate::Pos{x: x_small, y: y_small};
                    }
                    if check_same_point(&res_p1, &res_p2) {
                        CircleIntersectionResult::OnePoint(res_p1)
                    } else {
                        CircleIntersectionResult::TwoPoints((res_p1, res_p2))
                    }
                }
            }
        }
    }
}

fn intersect_circle_line(c: &crate::Pos, r: f64, p1: &crate::Pos, p2: &crate::Pos) -> CircleIntersectionResult {
    let res = get_y_circle_line(p1.x, p2.x, c.x, p1.y, p2.y, c.y, r);
    match res {
        None => {CircleIntersectionResult::None}
        Some((y_big, y_small)) => {
            let res2 = get_x_circle_line(p1.x, p2.x, c.x, p1.y, p2.y, c.y, r);
            match res2 {
                None => {CircleIntersectionResult::None}
                Some((x_big, x_small)) => {
                    let res_p1;
                    let res_p2;
                    if (p1.x > p2.x) != (p1.y > p2.y) {
                        res_p1 = crate::Pos{x: x_big, y: y_small};
                        res_p2 = crate::Pos{x: x_small, y: y_big};
                    } else {
                        res_p1 = crate::Pos{x: x_big, y: y_big};
                        res_p2 = crate::Pos{x: x_small, y: y_small};
                    }
                    if check_same_point(&res_p1, &res_p2) {
                        CircleIntersectionResult::OnePoint(res_p1)
                    } else {
                        CircleIntersectionResult::TwoPoints((res_p1, res_p2))
                    }
                }
            }
        }
    }
}

fn check_point_in_arc(arc: &Arc, p: &crate::Pos) -> Option<f64> {
    // we assume here that point is at circle
    let dif = p.add(&arc.center.multiply(-1.));
    let angle_p = dif.y.atan2(dif.x);
    let angle1 = (arc.angle_start - angle_p) * 0.5 / std::f64::consts::PI;
    let angle2 = (arc.angle_stop - angle_p) * 0.5 / std::f64::consts::PI;
    // if they ceil to the same number, there is no integer between them -> angle_p is not between our angles, 2pi repetitions adjusted
    let angle1ceil = angle1.ceil();
    let angle2ceil = angle2.ceil();
    if angle1ceil == angle2ceil {
        None
    } else {
        let asmall = if angle1ceil < angle2ceil {angle1ceil} else {angle2ceil};
        let adjusted_p = angle_p + asmall * 2. * std::f64::consts::PI;
        Some(adjusted_p)
    }
}

enum IntersectStraightResult {
    Parallel,
    Point(crate::Pos),
    SameLine
}

fn get_y_line_line(x1: f64,
                   x2: f64,
                   x3: f64,
                   x4: f64,
                   y1: f64,
                   y2: f64,
                   y3: f64,
                   y4: f64,
                   divider: f64) -> f64 {
    ((x3 * y4 - y3 * x4) * (y2 - y1) - (x1 * y2 - y1 * x2) * (y4 - y3)) / divider    
}

fn intersect_straight(p1: &crate::Pos,
                      p2: &crate::Pos,
                      p3: &crate::Pos,
                      p4: &crate::Pos) -> IntersectStraightResult {
    let divider = (p2.x - p1.x) * (p4.y - p3.y) * (p4.x - p3.x) * (p2.y - p1.y);
    if divider.abs() < MIN_DELTA {
        // Parallel or same line
        let div_2 = (p2.x - p1.x) * (p3.y - p1.y) - (p2.y - p1.y) * (p3.x - p1.x);
        if div_2.abs() < MIN_DELTA {
            IntersectStraightResult::SameLine
        } else {
            IntersectStraightResult::Parallel
        }
    } else {
        let y_intersect = get_y_line_line(p1.x,
                                               p2.x,
                                               p3.x,
                                               p4.x,
                                               p1.y,
                                               p2.y,
                                               p3.y,
                                               p4.y,
                                               divider);
        let x_intersect = get_y_line_line(p1.y,
                                               p2.y,
                                               p3.y,
                                               p4.y,
                                               p1.x,
                                               p2.x,
                                               p3.x,
                                               p4.x,
                                               -divider);
        let res = crate::Pos{
            x: x_intersect,
            y: y_intersect
        };
        IntersectStraightResult::Point(res)
    }
}

fn point_on_line(p1: &crate::Pos, p2: &crate::Pos, p: &crate::Pos) -> bool {
    ((p.x - p1.x) * (p2.y - p1.y) - (p.y - p1.y) * (p2.x - p1.x)).abs() < MIN_DELTA
}

fn point_on_circle(p: &crate::Pos, c: &crate::Pos, r: f64) -> bool {
    p.distance_squared(&c) - r * r < MIN_DELTA
}

fn check_point_on_segment_line(p: &crate::Pos, p1: &crate::Pos, p2: &crate::Pos) -> bool {
    // we assume point is on line
    let minus_p = p.multiply(-1.);
    let v1 = p1.add(&minus_p);
    let v2 = p2.add(&minus_p);
    let su = v1.add(&v2);
    let dist_between_points_squared = p1.distance_squared(&p2);
    let sum_squared = su.length_squared();
    sum_squared < dist_between_points_squared
}

fn check_intersect_arc_arc(arc1: &Arc, arc2: &Arc) {
    if check_same_point(&arc1.center, &arc2.center) {
        if arc1.radius - arc2.radius < MIN_DELTA {

        } else {
            // return None
        }

    } else {

    }
}
