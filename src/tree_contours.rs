use core::f64;
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
        
        let mut sp_obj = points.swap_remove(second_point);
        let prev_ind = points.len();
        if prev_ind != second_point {
            let point_swapped = &points[second_point];
            for l_n in &point_swapped.lines_starting {
                let line_starting = &mut self.lines[*l_n];
                line_starting.start_point_index = second_point;
            }
            for l_n in &point_swapped.lines_stopping {
                let line_stopping = &mut self.lines[*l_n];
                line_stopping.stop_point_index = second_point;
            }
        }
        for l_n in &sp_obj.lines_starting {
            let line_starting = &mut self.lines[*l_n];
            line_starting.start_point_index = first_point;
        }
        for l_n in &sp_obj.lines_stopping {
            let line_stopping = &mut self.lines[*l_n];
            line_stopping.stop_point_index = first_point;
        }
        
        let fp_obj = &mut points[first_point];
        // Adjusting line "pointers" in points
        fp_obj.lines_starting.append(&mut sp_obj.lines_starting);
        fp_obj.lines_stopping.append(&mut sp_obj.lines_stopping);
    }

    fn clean_points_after_merge_point_line(&mut self,
                                           pn_start: usize,
                                           pn_stop: usize,
                                           pn_center: usize,
                                           n_line: usize) {
            let n_line1 = self.lines.len() - 2;
            let n_line2 = self.lines.len() - 1;
            let p_start = &mut self.points[pn_start];
            let pos_to_swap = p_start.lines_starting.iter().position(|x| *x == n_line).unwrap();
            p_start.lines_starting[pos_to_swap] = n_line1;
            let p_stop = &mut self.points[pn_stop];
            let pos_to_swap = p_stop.lines_stopping.iter().position(|x| *x == n_line).unwrap();
            p_stop.lines_stopping[pos_to_swap] = n_line2;
            let p_center = &mut self.points[pn_center];
            p_center.lines_starting.push(n_line2);
            p_center.lines_stopping.push(n_line1);
        }

    fn arrange_two_lines_from_one(&mut self,
                                  old_line: LineForIntermediate,
                                  line_n:usize,
                                  line1: Line,
                                  line2: Line,
                                  p_n: usize) {
        let start_point_index = old_line.start_point_index;
        let stop_point_index = old_line.stop_point_index;
        let for_interm1 = LineForIntermediate{
            start_point_index,
            stop_point_index: p_n,
            line: line1
        };
        let for_interm2 = LineForIntermediate{
            start_point_index: p_n,
            stop_point_index,
            line: line2
        };
        self.lines.push(for_interm1);
        self.lines.push(for_interm2);
        self.clean_points_after_merge_point_line(start_point_index, stop_point_index, p_n, line_n);
    }

    fn remove_line_num_that_was_swapped(&mut self, line_n: usize) {
        let prev_position = self.lines.len();
        if line_n != prev_position {
            let line_now = &self.lines[line_n];
            let p1 = &mut self.points[line_now.start_point_index];
            let line_pos = p1.lines_starting.iter().position(|x| *x == prev_position).unwrap();
            p1.lines_starting.swap_remove(line_pos);
            let p2 = &mut self.points[line_now.stop_point_index];
            let line_pos = p2.lines_stopping.iter().position(|x| *x == prev_position).unwrap();
            p2.lines_stopping.swap_remove(line_pos);
        }
    }

    fn add_line_num_that_was_swapped(&mut self, line_n: usize) {
        if line_n != self.lines.len() - 2 {
            let line_now = &self.lines[line_n];
            let p1 = &mut self.points[line_now.start_point_index];
            p1.lines_starting.push(line_n);
            let p2 = &mut self.points[line_now.stop_point_index];
            p2.lines_stopping.push(line_n);
        }
    }

    fn merge_point_line(&mut self, res: ComparePointsLinesResult) {
        match res {
            ComparePointsLinesResult::PointArc((p_n, arc_n, angle)) => {
                let arc = self.lines.swap_remove(arc_n);
                self.remove_line_num_that_was_swapped(arc_n);
                match &arc.line {
                    Line::Segment(_seg) => {} // error if we are here
                    Line::Arc(inner_arc) => {
                        let new_arc_1 = Arc::new(inner_arc.center.clone(), inner_arc.radius, inner_arc.angle_start, angle);
                        let new_arc_2 = Arc::new(inner_arc.center.clone(), inner_arc.radius, angle, inner_arc.angle_stop);
                        let line1 = Line::Arc(new_arc_1);
                        let line2 = Line::Arc(new_arc_2);
                        self.arrange_two_lines_from_one(arc, arc_n, line1, line2, p_n);
                        self.add_line_num_that_was_swapped(arc_n);
                    }
                }            
            }
            ComparePointsLinesResult::PointSegment((p_n, seg_n)) => {
                let seg = self.lines.swap_remove(seg_n);
                self.remove_line_num_that_was_swapped(seg_n);
                match &seg.line {
                    Line::Arc(_arc) => {} // error if we are here
                    Line::Segment(inner_seg) => {
                        let new_pos = self.points[p_n].pos.clone();
                        let new_line1 = LineSegment::new(inner_seg.point1.clone(), new_pos.clone());
                        let new_line2 = LineSegment::new(new_pos, inner_seg.point2.clone());
                        let line1 = Line::Segment(new_line1);
                        let line2 = Line::Segment(new_line2);
                        self.arrange_two_lines_from_one(seg, seg_n, line1, line2, p_n);
                        self.add_line_num_that_was_swapped(seg_n);
                    }
                }
            }
        }
    }

    fn remove_line(&mut self, l_num: usize) {
        let l = self.lines.swap_remove(l_num);
        let pn1 = l.start_point_index;
        let pn2 = l.stop_point_index;
        let p1l = &mut self.points[pn1].lines_starting;
        let p1_ind = p1l.iter().position(|x| *x == l_num).unwrap();
        p1l.swap_remove(p1_ind);
        let p2l = &mut self.points[pn2].lines_stopping;
        let p2_ind = p2l.iter().position(|x| *x == l_num).unwrap();
        p2l.swap_remove(p2_ind);
        let now_len = self.lines.len();
        if l_num != now_len {
            let swapped_l = &self.lines[l_num];
            let pn_start = swapped_l.start_point_index;
            let pn_stop = swapped_l.stop_point_index;
            let p_start = &mut self.points[pn_start];
            let l_pos = p_start.lines_starting.iter().position(|x| *x == now_len).unwrap();
            p_start.lines_starting[l_pos] = l_num;
            let p_stop = &mut self.points[pn_stop];
            let l_pos = p_stop.lines_stopping.iter().position(|x| *x == now_len).unwrap();
            p_stop.lines_stopping[l_pos] = l_num;
        }
    }

    fn line_line(&mut self,
                 pos: &crate::Pos,
                 recipe1: ComparePointsLinesResult,
                 recipe2: ComparePointsLinesResult,
                 point_num: usize) {
        let point = PointForIntermediate {
            pos: pos.clone(),
            lines_starting: vec![],
            lines_stopping: vec![]
        };
        self.points.push(point);
        self.merge_point_line(recipe1);
        self.merge_point_line(recipe2);
        self.adjust_new_point(point_num);
    }

    fn merge_line_line(&mut self, l1_num: usize, l2_num: usize, result: IntersectLineLineResult) {
        match result {
            IntersectLineLineResult::SameAntiParallel => {
                self.remove_line(l1_num);
                self.remove_line(l2_num);
            }
            IntersectLineLineResult::SameParallel => {
                self.remove_line(l2_num);
            }
            IntersectLineLineResult::ArcSegmentOnePoint((pos, angle)) => {
                let point_num = self.points.len();
                let recipe1;
                let recipe2;
                match &self.lines[l1_num].line {
                    &Line::Arc(_) => {
                        recipe1 = ComparePointsLinesResult::PointArc((point_num, l1_num, angle));
                        recipe2 = ComparePointsLinesResult::PointSegment((point_num, l2_num));
                    }
                    &Line::Segment(_) => {
                        recipe1 = ComparePointsLinesResult::PointSegment((point_num, l1_num));
                        recipe2 = ComparePointsLinesResult::PointArc((point_num, l2_num, angle));
                    }
                }
                self.line_line(&pos, recipe1, recipe2, point_num);
            }
            IntersectLineLineResult::ArcSegmentTwoPoints(((pos1, angle1), (pos2, angle2))) => {
                let point_num = self.points.len();
                let recipe1;
                let recipe2;
                match &self.lines[l1_num].line {
                    &Line::Arc(_) => {
                        recipe1 = ComparePointsLinesResult::PointArc((point_num, l1_num, angle1));
                        recipe2 = ComparePointsLinesResult::PointSegment((point_num, l2_num));
                    }
                    &Line::Segment(_) => {
                        recipe1 = ComparePointsLinesResult::PointSegment((point_num, l1_num));
                        recipe2 = ComparePointsLinesResult::PointArc((point_num, l2_num, angle1));
                    }
                }
                self.line_line(&pos1, recipe1, recipe2, point_num);
                // TODO
            }
            IntersectLineLineResult::TwoArcsOnePoint((pos, angle1, angle2)) => {
                let point_num = self.points.len();
                let recipe1 = ComparePointsLinesResult::PointArc((point_num, l1_num, angle1));
                let recipe2 = ComparePointsLinesResult::PointArc((point_num, l2_num, angle2));
                self.line_line(&pos, recipe1, recipe2, point_num);
            }
            IntersectLineLineResult::TwoArcsTwoPoints(((pos1, angle1_pos1, angle2_pos1),
                                                       (pos2, angle1_pos2, angle2_pos2))) => {
                let point_num = self.points.len();
                let recipe1 = ComparePointsLinesResult::PointArc((point_num, l1_num, angle1_pos1));
                let recipe2 = ComparePointsLinesResult::PointArc((point_num, l2_num, angle2_pos1));
                self.line_line(&pos1, recipe1, recipe2, point_num);
                //TODO

            }
            IntersectLineLineResult::TwoSegmentsPoint(pos) => {
                let point_num = self.points.len();
                let recipe1 = ComparePointsLinesResult::PointSegment((point_num, l1_num));
                let recipe2 = ComparePointsLinesResult::PointSegment((point_num, l2_num));
                self.line_line(&pos, recipe1, recipe2, point_num);
            }
        }
    }

    fn merge_and_clean(&mut self) {
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
        let mut finished_points_lines = false;
        let mut start_i_point = 0;
        while !finished_points_lines {
            let res = self.compare_points_lines(start_i_point, 0);
            match res {
                None => {
                    finished_points_lines = true;
                }
                Some(result) => {
                    match result {
                        ComparePointsLinesResult::PointArc((p_n, _, __)) => {
                            self.merge_point_line(result);
                            start_i_point = p_n;
                        }
                        ComparePointsLinesResult::PointSegment((p_n, _)) => {
                            self.merge_point_line(result);
                            start_i_point = p_n;
                        }

                    }
                }
            }
        }
        let mut finished_points_circles = false;
        start_i_point = 0;
        while !finished_points_circles {
            let res = self.compare_points_circles(start_i_point);
            match res {
                None => {
                    finished_points_circles = true;
                }
                Some((p_n, c_n)) => {
                    self.merge_points_circle(p_n, c_n);
                    start_i_point = p_n;
                }
            }
        }
        let mut finished_lines_lines = false;
        let mut start_i_line = 0;
        while !finished_lines_lines {
            let res = self.compare_lines_lines(start_i_line);
            match res {
                None => {
                    finished_lines_lines = true;
                }
                Some((i_l1, i_l2, result)) => {
                    self.merge_line_line(i_l1, i_l2, result);
                    start_i_line = i_l1;
                }
            }
        }
    }

    fn merge_points_circle(&mut self, p_n: usize, c_n: usize) {
        let len_lines_before = self.lines.len();
        let pos = &self.points[p_n].pos;
        let circle = self.circles.swap_remove(c_n);
        let dif = pos.add(&circle.center.multiply(-1.));
        let angle = dif.y.atan2(dif.x);
        let new_arc = Arc::new(circle.center, circle.radius, angle - 2. * f64::consts::PI, angle);
        let new_line = Line::Arc(new_arc);
        let new_interm = LineForIntermediate {
            start_point_index: p_n,
            stop_point_index: p_n,
            line: new_line
        };
        self.lines.push(new_interm);
        let mut finished_points_lines = false;
        let mut start_i_point = p_n + 1;
        while !finished_points_lines {
            let res = self.compare_points_lines(start_i_point,
                                                                    len_lines_before);
            match res {
                None => {
                    finished_points_lines = true;
                }
                Some(result) => {
                    match result {
                        ComparePointsLinesResult::PointArc((p_num, _, _)) => {
                            self.merge_point_line(result);
                            start_i_point = p_num;
                        }
                        ComparePointsLinesResult::PointSegment(_) => {}// If we are here it's an error!
                    }
                }
            }
        }
    }

    fn compare_points_circles(&self, start_i_point: usize) -> Option<(usize, usize)> {
        for (counter, p) in self.points[start_i_point..].iter().enumerate() {
            let i = counter + start_i_point;
            for (j, circle) in self.circles.iter().enumerate() {
                if point_on_circle(&p.pos, &circle.center, circle.radius) {
                    return Some((i, j));
                }    
            }
        }
        None
    }

    fn compare_points_lines(&self, start_i_point: usize, start_i_line: usize) -> Option<ComparePointsLinesResult> {
        for (counter, p) in self.points[start_i_point..].iter().enumerate() {
            let i = counter + start_i_point;
            for (counter2, line) in self.lines[start_i_line..].iter().enumerate() {
                let j = counter2 + start_i_line;
                let res = compare_point_line(i, j, p, line);
                match res {
                    Some(_) => {
                        return res;
                    }
                    None => {}
                }
            }
        }
        None
    }

    fn adjust_new_point(&mut self, point_num: usize) {
        let mut done = false;
        let mut line_num_start = 0;
        while !done {
            let res = self.compare_new_point_lines(point_num, line_num_start);
            match res {
                Some(result) => {
                    match result {
                        ComparePointsLinesResult::PointArc((_, ln, _)) => {
                            line_num_start = ln;
                        }
                        ComparePointsLinesResult::PointSegment((_, ln)) => {
                            line_num_start = ln;
                        }
                    }
                    self.merge_point_line(result);
                }
                None => {
                    done = true;
                }
            }
        }
        done = false;
        let mut circles_num_start = 0;
        while !done {
            let res = self.compare_new_point_circles(point_num, circles_num_start);
            match res {
                None => {
                    done = true;
                }
                Some(c_n) => {
                    self.merge_points_circle(point_num, c_n);
                    circles_num_start = c_n;
                }
            }
        }
    }

    fn compare_new_point_lines(&self, point_num: usize, line_num_start: usize) -> Option<ComparePointsLinesResult> {
        let p = &self.points[point_num];
        for (counter, line) in self.lines[line_num_start..].iter().enumerate() {
            let j_line = counter + line_num_start;
            let res = compare_point_line(point_num, j_line, p, line);
            match res {
                Some(_) => {
                    return res;
                }
                None => {}
            }
        }
        None
    }

    fn compare_new_point_circles(&self, point_num: usize, circles_num_start: usize) -> Option<usize> {
        let p = &self.points[point_num];
        for (circle_counter, circle) in self.circles[circles_num_start..].iter().enumerate() {
            let circle_num = circle_counter + circles_num_start;
            if point_on_circle(&p.pos, &circle.center, circle.radius) {
                return Some(circle_num);
            }    
        }
        None
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

    fn compare_lines_lines(&self, start_i: usize) -> Option<(usize, usize, IntersectLineLineResult)>{
        for (counter, l1) in self.lines[start_i..self.lines.len() - 1].iter().enumerate() {
            let i = counter + start_i;
            for (counter2, l2) in self.lines[i + 1..].iter().enumerate() {
                let j = i + 1 + counter2;
                let res = self.check_intersect_line_line(l1, l2);
                match res {
                    None => {}
                    Some(result) => {
                        return Some((i, j, result));
                    }
                }
            }
        }
        None
    }

    fn check_intersect_line_line(&self, line1: &LineForIntermediate, line2: &LineForIntermediate) -> Option<IntersectLineLineResult> {
        match &line1.line {
            Line::Segment(seg1) => {
                match &line2.line {
                    Line::Segment(seg2) => {
                        let res = check_intersect_seg_seg(seg1,
                                                                                 line1.start_point_index,
                                                                                 line1.stop_point_index,
                                                                                 seg2,
                                                                                 line2.start_point_index,
                                                                                 line2.stop_point_index);
                        match res {
                            IntersectSegmentSegmentResult::None => {
                                return None;
                            }
                            IntersectSegmentSegmentResult::Point(pos) => {
                                return Some(IntersectLineLineResult::TwoSegmentsPoint(pos));
                            }
                            IntersectSegmentSegmentResult::SameParallel => {
                                return Some(IntersectLineLineResult::SameParallel);
                            }
                            IntersectSegmentSegmentResult::SameAntiParallel => {
                                return Some(IntersectLineLineResult::SameAntiParallel);
                            }
                        }
                    }
                    Line::Arc(arc2) => {
                        let res = check_intersect_seg_arc(seg1,
                                                                                     line1.start_point_index,
                                                                                     line1.stop_point_index,
                                                                                     arc2,
                                                                                     line2.start_point_index,
                                                                                     line2.stop_point_index);
                        match res {
                            IntersectSegmentArcResult::None => {
                                return None;
                            }
                            IntersectSegmentArcResult::OnePoint((pos, angle)) => {
                                return Some(IntersectLineLineResult::ArcSegmentOnePoint((pos, angle)));
                            }
                            IntersectSegmentArcResult::TwoPoints(((pos1, angle1), (pos2, angle2))) => {
                                return Some(IntersectLineLineResult::ArcSegmentTwoPoints(((pos1, angle1), (pos2, angle2))));
                            }
                        }
                    }
                }
            }
            Line::Arc(arc1) => {            
                match &line2.line {
                    Line::Segment(seg2) => {
                        let res = check_intersect_seg_arc(seg2,
                                                                                     line2.start_point_index,
                                                                                     line2.stop_point_index,
                                                                                     arc1,
                                                                                     line1.start_point_index,
                                                                                     line1.stop_point_index);
                        match res {
                            IntersectSegmentArcResult::None => {
                                return None;
                            }
                            IntersectSegmentArcResult::OnePoint((pos, angle)) => {
                                return Some(IntersectLineLineResult::ArcSegmentOnePoint((pos, angle)));
                            }
                            IntersectSegmentArcResult::TwoPoints(((pos1, angle1), (pos2, angle2))) => {
                                return Some(IntersectLineLineResult::ArcSegmentTwoPoints(((pos1, angle1), (pos2, angle2))));
                            }
                        }
                    }
                    Line::Arc(arc2) => {
                        let res = self.check_intersect_arc_arc(arc1,
                                                                                      line1.start_point_index,
                                                                                      line1.stop_point_index,
                                                                                      arc2,
                                                                                      line2.start_point_index,
                                                                                      line2.stop_point_index);
                        match res {
                            IntersectArcArcResult::None => {
                                return None;
                            }
                            IntersectArcArcResult::SameCircleParallel => {
                                return Some(IntersectLineLineResult::SameParallel);
                            }
                            IntersectArcArcResult::SameCircleAntiparallel => {
                                return Some(IntersectLineLineResult::SameAntiParallel);
                            }
                            IntersectArcArcResult::TwoCirclesOnePoint((pos, angle1, angle2)) => {
                                return Some(IntersectLineLineResult::TwoArcsOnePoint((pos, angle1, angle2)));
                            }
                            IntersectArcArcResult::TwoCirclesTwoPoints(((pos1, angle1_pos1, angle2_pos1),
                                                                        (pos2, angle1_pos2, angle2_pos2))) => {
                                return Some(IntersectLineLineResult::TwoArcsTwoPoints(((pos1, angle1_pos1, angle2_pos1),
                                                                                       (pos2, angle1_pos2, angle2_pos2))));
                            }
                        }
                    }
                }
            }
        }
    }

    fn check_intersect_arc_arc(&self,
                               arc1: &Arc,
                               arc1p1: usize,
                               arc1p2: usize,
                               arc2: &Arc,
                               arc2p1: usize,
                               arc2p2: usize) -> IntersectArcArcResult {
        if check_same_point(&arc1.center, &arc2.center) {
            if arc1.radius - arc2.radius < MIN_DELTA {
                if arc1p1 == arc2p1 && arc1p2 == arc2p2 {
                    if ((arc1.angle_stop - arc1.angle_start) > 0.) == ((arc2.angle_stop - arc2.angle_start) > 0.) {
                        return IntersectArcArcResult::SameCircleParallel;
                    } else {
                        return IntersectArcArcResult::None;
                    }
                } else {
                    if arc1p1 == arc2p2 && arc2p1 == arc1p2 {
                        if ((arc1.angle_stop - arc1.angle_start) > 0.) != ((arc2.angle_stop - arc2.angle_start) > 0.) {
                            return IntersectArcArcResult::SameCircleAntiparallel;
                        } else {
                            return IntersectArcArcResult::None;
                        }
                    } else {
                        return IntersectArcArcResult::None;
                    }
                }
            } else {
                return IntersectArcArcResult::None;
            }
        } else {
            if (arc1p1 == arc2p1 && arc1p2 == arc2p2) || (arc1p1 == arc2p2 && arc2p1 == arc1p2) {
                return IntersectArcArcResult::None;
            }
            let mut one_point_common = None;
            if arc1p1 == arc2p1 || arc1p1 == arc2p2 {one_point_common = Some(&self.points[arc1p1].pos)};
            if arc1p2 == arc2p1 || arc1p2 == arc2p2 {one_point_common = Some(&self.points[arc1p2].pos)};
            let res = intersect_circle_circle(&arc1.center, &arc2.center, arc1.radius, arc2.radius);
            match res {
                CircleIntersectionResult::None => {
                    return IntersectArcArcResult::None;
                }
                CircleIntersectionResult::OnePoint(pos) => {
                    match one_point_common {
                        Some(_) => {return IntersectArcArcResult::None;}
                        None => {
                            let at_arc1 = check_point_in_arc(arc1, &pos);
                            match at_arc1 {
                                None => {return IntersectArcArcResult::None;}
                                Some(angle1) => {
                                    let at_arc2 = check_point_in_arc(arc2, &pos);
                                    match at_arc2 {
                                        None => {return IntersectArcArcResult::None;}
                                        Some(angle2) => {
                                            return IntersectArcArcResult::TwoCirclesOnePoint((pos, angle1, angle2));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                CircleIntersectionResult::TwoPoints((pos1, pos2)) => {
                    match one_point_common {
                        None => { // arcs do not have common endpoint
                            let at_arc1_p1 = check_point_in_arc(arc1, &pos1);
                            match at_arc1_p1 {
                                None => {
                                    let at_arc1_p2 = check_point_in_arc(arc1, &pos2);
                                    match at_arc1_p2 {
                                        None => {return IntersectArcArcResult::None;}
                                        Some(angle_arc1_p2) => {
                                            let at_arc2_p2 = check_point_in_arc(arc2, &pos2);
                                            match at_arc2_p2 {
                                                None => {return IntersectArcArcResult::None;}
                                                Some(angle_arc2_p2) => {
                                                    return IntersectArcArcResult::TwoCirclesOnePoint((pos2, angle_arc1_p2, angle_arc2_p2));
                                                }
                                            }
                                        }
                                    } 
                                }
                                Some(angle_arc1_p1) => {
                                    let at_arc2_p1 = check_point_in_arc(arc2, &pos1);
                                    match at_arc2_p1 {
                                        None => {
                                            // one point still possible (p2)
                                            let at_arc1_p2 = check_point_in_arc(arc1, &pos2);
                                            match at_arc1_p2 {
                                                None => {return IntersectArcArcResult::None;}
                                                Some(angle_arc1_p2) => {
                                                    let at_arc2_p2 = check_point_in_arc(arc2, &pos2);
                                                    match at_arc2_p2 {
                                                        None => {return IntersectArcArcResult::None;}
                                                        Some(angle_arc2_p2) => {
                                                            return IntersectArcArcResult::TwoCirclesOnePoint((pos2, angle_arc1_p2, angle_arc2_p2));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Some(angle_arc2_p1) => {
                                            // first point guaranteed
                                            let at_arc1_p2 = check_point_in_arc(arc1, &pos2);
                                            match at_arc1_p2 {
                                                None => {
                                                    return IntersectArcArcResult::TwoCirclesOnePoint((pos1, angle_arc1_p1, angle_arc2_p1));
                                                }
                                                Some(angle_arc1_p2) => {
                                                    let at_arc2_p2 = check_point_in_arc(arc2, &pos2);
                                                    match at_arc2_p2 {
                                                        None => {
                                                            return IntersectArcArcResult::TwoCirclesOnePoint((pos1, angle_arc1_p1, angle_arc2_p1));
                                                        }
                                                        Some(angle_arc2_p2) => {
                                                            return IntersectArcArcResult::TwoCirclesTwoPoints(((pos1, angle_arc1_p1, angle_arc2_p1),
                                                                                                               (pos2, angle_arc1_p2, angle_arc2_p2)));
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    } 
                                }
                            }
                        }
                        Some(corner_pos) => {
                            let leftover_pos;
                            if check_same_point(corner_pos, &pos1) {
                                leftover_pos = pos2;
                            } else {
                                leftover_pos = pos1;
                            }
                            let at_arc1_p = check_point_in_arc(arc1, &leftover_pos);
                            match at_arc1_p {
                                None => {return IntersectArcArcResult::None;}
                                Some(angle_arc1_p) => {
                                    let at_arc2_p = check_point_in_arc(arc2, &leftover_pos);
                                    match at_arc2_p {
                                        None => {return IntersectArcArcResult::None;}
                                        Some(angle_arc2_p) => {
                                            return IntersectArcArcResult::TwoCirclesOnePoint((leftover_pos, angle_arc1_p, angle_arc2_p));
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

}

fn compare_point_line(i_point: usize,
                      j_line: usize,
                      p: &PointForIntermediate,
                      line: &LineForIntermediate) -> Option<ComparePointsLinesResult> {
        if i_point != line.start_point_index && i_point != line.stop_point_index {
            match &line.line {
                Line::Segment(seg) => {
                    if point_on_line(&seg.point1, &seg.point2, &p.pos) {
                        if check_point_on_segment_line(&p.pos, &seg.point1, &seg.point2) {
                            return Some(ComparePointsLinesResult::PointSegment((i_point, j_line)));
                        }
                    }
                }
                Line::Arc(arc) => {
                    if point_on_circle(&p.pos, &arc.center, arc.radius) {
                        let res = check_point_in_arc(&arc, &p.pos);
                        match res {
                            Some(angle) => {
                                return Some(ComparePointsLinesResult::PointArc((i_point, j_line, angle)));
                            }
                            None => {}
                        }
                    } 
                }
            }
        }
        None
    }

enum ComparePointsLinesResult {
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
        let d_root = d.sqrt();
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
        let d_sqrt = d.sqrt();
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

enum IntersectArcArcResult {
    None,
    SameCircleParallel,
    SameCircleAntiparallel,
    TwoCirclesOnePoint((crate::Pos, f64, f64)),
    TwoCirclesTwoPoints(((crate::Pos, f64, f64), (crate::Pos, f64, f64)))
}

enum IntersectSegmentArcResult {
    None,
    OnePoint((crate::Pos, f64)),
    TwoPoints(((crate::Pos, f64), (crate::Pos, f64)))
}

enum IntersectSegmentSegmentResult {
    None,
    SameParallel,
    SameAntiParallel,
    Point(crate::Pos)
}

enum IntersectLineLineResult {
    SameParallel,
    SameAntiParallel,
    TwoArcsOnePoint((crate::Pos, f64, f64)),
    TwoArcsTwoPoints(((crate::Pos, f64, f64), (crate::Pos, f64, f64))),
    ArcSegmentOnePoint((crate::Pos, f64)),
    ArcSegmentTwoPoints(((crate::Pos, f64), (crate::Pos, f64))),
    TwoSegmentsPoint(crate::Pos)
}

fn check_intersect_seg_seg(seg1: &LineSegment,
                           seg1_p1: usize,
                           seg1_p2: usize,
                           seg2: &LineSegment,
                           seg2_p1: usize,
                           seg2_p2: usize) -> IntersectSegmentSegmentResult {
    if (seg1_p1 == seg2_p1) && (seg1_p2 == seg2_p2) {
        return IntersectSegmentSegmentResult::SameParallel;
    }
    if (seg1_p1 == seg2_p2) && (seg2_p1 == seg1_p2) {
        return IntersectSegmentSegmentResult::SameAntiParallel;
    }
    if (seg1_p1 == seg2_p1) || (seg1_p1 == seg2_p2) || (seg1_p2 == seg2_p1) || (seg1_p2 == seg2_p2) {
        return IntersectSegmentSegmentResult::None;
    }
    let res = intersect_straight(&seg1.point1, &seg1.point2, &seg2.point1, &seg2.point2);
    match res {
        IntersectStraightResult::Parallel | IntersectStraightResult::SameLine => {
            return IntersectSegmentSegmentResult::None;
        }
        IntersectStraightResult::Point(pos) => {
            if check_point_on_segment_line(&pos, &seg1.point1, &seg1.point2) {
                if check_point_on_segment_line(&pos, &seg2.point1, &seg2.point2) {
                    return IntersectSegmentSegmentResult::Point(pos)
                }
            }
            return IntersectSegmentSegmentResult::None;
        }
    }
}

fn check_intersect_seg_arc(seg: &LineSegment,
                           seg_p1: usize,
                           seg_p2: usize,
                           arc: &Arc,
                           arc_p1: usize,
                           arc_p2: usize) -> IntersectSegmentArcResult {
    if ((seg_p1 == arc_p1) && (seg_p2 == arc_p2)) || ((seg_p1 == arc_p2) && (seg_p2 == arc_p1)) {
        return IntersectSegmentArcResult::None;
    }
    let mut one_point_common = None;
    if seg_p1 == arc_p1 || seg_p1 == arc_p2 {one_point_common = Some(&seg.point1)};
    if seg_p2 == arc_p1 || seg_p2 == arc_p2 {one_point_common = Some(&seg.point2)};
    let res = intersect_circle_line(&arc.center, arc.radius, &seg.point1, &seg.point2);
    match res {
        CircleIntersectionResult::None => {return IntersectSegmentArcResult::None;}
        CircleIntersectionResult::OnePoint(pos) => {
            match one_point_common {
                Some(_) => {return IntersectSegmentArcResult::None;}
                None => {
                    if check_point_on_segment_line(&pos, &seg.point1, &seg.point2) {
                        let point_arc = check_point_in_arc(arc, &pos);
                        match point_arc {
                            None => {return IntersectSegmentArcResult::None;}
                            Some(angle) => {
                                return IntersectSegmentArcResult::OnePoint((pos, angle));
                            }
                        }
                    }
                    return IntersectSegmentArcResult::None;
                }
            }
        }
        CircleIntersectionResult::TwoPoints((pos1, pos2)) => {
            match one_point_common {
                None => {
                    if check_point_on_segment_line(&pos1, &seg.point1, &seg.point2) {
                        let point1_arc = check_point_in_arc(arc, &pos1);
                        match point1_arc {
                            None => {
                                if check_point_on_segment_line(&pos2, &seg.point1, &seg.point2) {
                                    let point2_arc = check_point_in_arc(arc, &pos2);
                                    match point2_arc {
                                        None => {return IntersectSegmentArcResult::None;}
                                        Some(angle2) => {
                                            return IntersectSegmentArcResult::OnePoint((pos2, angle2));
                                        }
                                    }
                                } else {
                                    return IntersectSegmentArcResult::None;
                                }
                            }
                            Some(angle1) => {
                                if check_point_on_segment_line(&pos2, &seg.point1, &seg.point2) {
                                    let point2_arc = check_point_in_arc(arc, &pos2);
                                    match point2_arc {
                                        None => {
                                            return IntersectSegmentArcResult::OnePoint((pos1, angle1));
                                        }
                                        Some(angle2) => {
                                            return IntersectSegmentArcResult::TwoPoints(((pos1, angle1), (pos2, angle2)));
                                        }
                                    }
                                } else {
                                    return IntersectSegmentArcResult::OnePoint((pos1, angle1));
                                }
                            }
                        }
                    } else {
                        if check_point_on_segment_line(&pos2, &seg.point1, &seg.point2) {
                            let point2_arc = check_point_in_arc(arc, &pos2);
                            match point2_arc {
                                None => {return IntersectSegmentArcResult::None;}
                                Some(angle2) => {
                                    return IntersectSegmentArcResult::OnePoint((pos2, angle2));
                                }
                            }
                        } else {
                            return IntersectSegmentArcResult::None;
                        }
                    }
                }
                Some(corner_pos) => {
                    let leftover_pos;
                    if check_same_point(corner_pos, &pos1) {
                        leftover_pos = pos2;
                    } else {
                        leftover_pos = pos1;
                    }
                    if check_point_on_segment_line(&leftover_pos, &seg.point1, &seg.point2) {
                        let point_arc = check_point_in_arc(arc, &leftover_pos);
                        match point_arc {
                            None => {return IntersectSegmentArcResult::None;}
                            Some(angle) => {
                                return IntersectSegmentArcResult::OnePoint((leftover_pos, angle));
                            }
                        }
                    }
                    return IntersectSegmentArcResult::None;
                }
            }
        }
    }
}