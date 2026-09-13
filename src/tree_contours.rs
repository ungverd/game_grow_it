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
    is_host_first: bool,
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
                
            }
        }
        FigureIntermediate { lines, points, circles }
    }
}

fn check_intersect_arc_arc(arc1: &Arc, arc2: &Arc) {
    if ((arc1.center.x - arc2.center.x).abs() > MIN_DELTA || (arc1.center.y - arc2.center.y).abs() > MIN_DELTA) {
        // We don't do anything here if centers are the same point: that will be circle-circle or lines

    }
}
