const MIN_DELTA: f32 = 0.00001;

pub struct LineSegment {
    point1: crate::Pos,
    point2: crate::Pos,
}

pub struct Arc {
    center: crate::Pos,
    radius: f32,
    angle_start: f32,
    angle_stop: f32,
}

pub struct Circle {
    center: crate::Pos,
    radius: f32,
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
    pub fn new(center: crate::Pos, radius: f32, angle_start:f32, angle_stop:f32) -> Arc {
        Arc {
            center,
            radius,
            angle_start,
            angle_stop,
        }
    }
}

impl Circle {
    pub fn new(center: crate::Pos, radius: f32, is_outer: bool) -> Circle {
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

    fn make_intersections(&mut self) {
        let was_there_difference = false;

    } 
}

fn check_same_point(p1: &crate::Pos, p2: &crate::Pos) {
    let dif = p1.add(&p2.multiply(-1.0));
}

fn check_intersect_circle_circle(c1: &crate::Pos, c2: &crate::Pos, r1: f32, r2: f32) {

}

fn check_intersect_arc_arc(arc1: &Arc, arc2: &Arc) {
    if ((arc1.center.x - arc2.center.x).abs() > MIN_DELTA || (arc1.center.y - arc2.center.y).abs() > MIN_DELTA) {
        // We don't do anything here if centers are the same point: that will be circle-circle or lines

    }
}
