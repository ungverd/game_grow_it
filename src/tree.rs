use wasm_bindgen::prelude::*;
const DEST_REF: i32 = 20;
const LINE_WIDTH: f64 = 2.0;
const SHADOW_WIDTH: f64 = 2.5;
const SHADOW_STRENGTH: f64 = 0.2;
const F_DEST_REF: f64 = DEST_REF as f64;
const PI: f64 = std::f64::consts::PI;
//const STEM: [[u32; 3]; 6] = [[5,5,1], [2,1,12], [1,11,1], [5,5,5], [11,12,10], [5, 3, 2]]; // [right, left, repeats]
//const STEM: [[u32; 3]; 8] = [[5,5,1], [2,1,12], [1,11,1], [3,3,2], [5,6,3], [5, 3, 2], [1, 0, 5], [0, 1, 4]];
//const STEM: [[u32; 3]; 3] = [[5,5,1], [1, 0, 4], [0, 1, 4]];
//const STEM: [[u32; 3]; 3] = [[5,5,1], [1,2,11], [5,5,1]]; // [right, left, repeats]
//const STEM: [[u32; 3]; 2] = [[5,5,1], [1,2,11]]; // [right, left, repeats]
//const STEM: [[u32; 3]; 1] = [[12,11,4]]; // [right, left, repeats]
//const STEM: [[u32; 3]; 1] = [[1,1,1]]; // [right, left, repeats]
//const STEM: [[u32; 3]; 1] = [[1,0,1]]; // [right, left, repeats]
const SIZE_X: usize = 600; // Needs to be changed according to canvas proportions
const SIZE_Y: usize = 600; // Needs to be changed according to canvas proportions
const START_X: i32 = 256;
const START_Y: i32 = 1;
const LEFT_BOOL: bool = true;
const RIGHT_BOOL: bool = false;
const SEMI_W: f64 = F_DEST_REF / 2f64;

const ABS_BETA_LIMIT: f64 = 1.8; // a bit arbitrary value where fancy algorithm doesn't work well


fn get_size(width: f64, height: f64, f_source_ref: f64) -> (f64, f64, f64) {
    let w_common = width * F_DEST_REF / f_source_ref;
    let h_bottom = PI * F_DEST_REF / 6f64;
    let h_top = height * F_DEST_REF / f_source_ref - h_bottom;
    (w_common, h_bottom, h_top)
}

struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

struct Segment {
    center_bottom_x: f64,
    center_bottom_y: f64,
    distance_from_root: f64,
    radius: f64,
    angle_start: f64,
    left: bool,
    straight: bool,
    is_with_zero_opposite: bool
}

/* brezenham-like selection of which leaf is in front of which in arc */

fn bresenham(left: u32, right: u32, results: &mut Vec<bool>) {
    let (x, y) = if left > right { (left, right) } else { (right, left) };
    let y = y as f64;
    let x = x as f64;
    let slope = y / x;
    let minor;
    let major;
    if left > right {
        minor = RIGHT_BOOL;
        major = LEFT_BOOL;
    } else {
        minor = LEFT_BOOL;
        major = RIGHT_BOOL;
    }
    let mut delta = 0f64;
    let total_leafs = left + right; 
    let threshold = (1.0 - slope) / 2.0;
    for _i in 0..total_leafs {
        if delta >= threshold {
            results.push(minor);
            delta -= 1f64;
        } else {
            results.push(major);
            delta += slope;
        }
    }
}

fn add_arc_leaf_segment(center_bottom_x: f64,
                        center_bottom_y: f64,
                        angle: f64,
                        counter: u32,
                        radius: f64,
                        total_distance_from_root: f64,
                        segment_angle: f64,
                        segment_length: f64) -> (f64, f64, u32, f64, f64) {
    let angle_start = angle + (counter as f64) * segment_angle;
    let distance_from_root = total_distance_from_root + (counter as f64) * segment_length;
    let counter = counter + 1;
    let angle_stop = angle + (counter as f64) * segment_angle;
    let center_bottom_x = center_bottom_x + radius * (angle_stop.cos() - angle_start.cos());
    let center_bottom_y = center_bottom_y + radius * (angle_stop.sin() - angle_start.sin());
    (center_bottom_x, center_bottom_y, counter, distance_from_root, angle_start)
}
fn add_straight_leaf_segment(center_bottom_x: f64,
                             center_bottom_y: f64,
                             angle: f64,
                             counter: u32,
                             total_distance_from_root: f64,
                             segment_length: f64) -> (f64, f64, u32, f64, f64) {
    let angle_start = angle;
    let distance_from_root = total_distance_from_root + (counter as f64) * segment_length;
    let counter = counter + 1;
    let center_bottom_x = center_bottom_x - segment_length * angle.sin();
    let center_bottom_y = center_bottom_y + segment_length * angle.cos();
    (center_bottom_x, center_bottom_y, counter, distance_from_root, angle_start)
}
fn add_leaf_segment(straight: bool,
                    center_bottom_x: f64,
                    center_bottom_y: f64,
                    angle: f64,
                    counter: u32,
                    radius: f64,
                    total_distance_from_root: f64,
                    segment_angle: f64,
                    segment_length: f64) -> (f64, f64, u32, f64, f64) {
    if straight {
        add_straight_leaf_segment(center_bottom_x, center_bottom_y, angle, counter, total_distance_from_root, segment_length)
    } else {
        add_arc_leaf_segment(center_bottom_x, center_bottom_y, angle, counter, radius, total_distance_from_root, segment_angle, segment_length)
    }
}

fn get_segments(w: f64, stem: &Vec<crate::TreeUnit>) -> Vec<Segment> {
    let mut segments_capacity = 0;
    for item in stem {
        segments_capacity += item.right * item.repeats;
        segments_capacity += item.left * item.repeats;
    }
    let mut segments = Vec::with_capacity(segments_capacity as usize);
    let mut center_bottom_x_global = START_X as f64;
    let mut center_bottom_y_global = START_Y as f64;
    let mut angle = 0f64;
    let mut total_distance_from_root = 0f64;
    for item in stem {
        let right = item.right;
        let left = item.left;
        let f_right = right as f64;
        let f_left = left as f64;
        let total_leafs = (left + right) as usize; 
        let mut bres_seq = Vec::with_capacity(total_leafs);
        bresenham(left, right, &mut bres_seq);
        let straight;
        let radius;
        let arc_angle;
        let total_item_length;
        if right != left {
            radius = (f_right + f_left) * w / (f_right - f_left) / 2f64; /* positive if inclined left, negative if inclined right */
            arc_angle = (f_right - f_left) * PI / 6f64; /* positive if inclined left, negative if inclined right */
            straight = false;
            total_item_length = radius * arc_angle;
        } else {
            straight = true;
            radius = 0f64;
            arc_angle = 0f64;
            total_item_length = f_left * w * PI / 6f64;
        }
        let left_segment_angle = arc_angle / f_left;
        let right_segment_angle = arc_angle / f_right;
        let left_segment_length = total_item_length / f_left;
        let right_segment_length = total_item_length / f_right;
        let mut center_bottom_x_left = center_bottom_x_global;
        let mut center_bottom_y_left = center_bottom_y_global;
        let mut center_bottom_x_right = center_bottom_x_global;
        let mut center_bottom_y_right = center_bottom_y_global;
        let is_with_zero_opposite = right == 0 || left == 0; 
        for _i in 0..item.repeats {
            let mut counter_left = 0;
            let mut counter_right = 0;
            for el in &bres_seq {

                let center_bottom_x_left_next;
                let center_bottom_y_left_next;
                let center_bottom_x_right_next;
                let center_bottom_y_right_next;
                let center_bottom_x;
                let center_bottom_y;
                let left;
                let angle_start;
                let distance_from_root;
                if *el == LEFT_BOOL {
                    left = true;
                    (center_bottom_x_left_next,
                     center_bottom_y_left_next,
                     counter_left,
                     distance_from_root,
                     angle_start) = add_leaf_segment(straight,
                                                    center_bottom_x_left,
                                                    center_bottom_y_left,
                                                    angle,
                                                    counter_left,
                                                    radius,
                                                    total_distance_from_root,
                                                    left_segment_angle,
                                                    left_segment_length);
                    center_bottom_x = center_bottom_x_left;
                    center_bottom_y = center_bottom_y_left;
                    center_bottom_x_left = center_bottom_x_left_next;
                    center_bottom_y_left = center_bottom_y_left_next;
                    
                } else { // *el == RIGHT_BOOL
                    left = false;
                    (center_bottom_x_right_next,
                     center_bottom_y_right_next,
                     counter_right,
                     distance_from_root,
                     angle_start) = add_leaf_segment(straight,
                                                    center_bottom_x_right,
                                                    center_bottom_y_right,
                                                    angle,
                                                    counter_right,
                                                    radius,
                                                    total_distance_from_root,
                                                    right_segment_angle,
                                                    right_segment_length);
                    center_bottom_x = center_bottom_x_right;
                    center_bottom_y = center_bottom_y_right;
                    center_bottom_x_right = center_bottom_x_right_next;
                    center_bottom_y_right = center_bottom_y_right_next;
                }
                let leaf_segment = Segment {center_bottom_x,
                                                     center_bottom_y,
                                                     distance_from_root,
                                                     radius,
                                                     angle_start,
                                                     left,
                                                     straight,
                                                     is_with_zero_opposite};
                segments.push(leaf_segment);
            }
            if right == 0 {
                center_bottom_x_global = center_bottom_x_left;
                center_bottom_y_global = center_bottom_y_left + center_bottom_y_right;
            } else if left == 0 {
                center_bottom_x_global = center_bottom_x_right;
                center_bottom_y_global = center_bottom_y_right;
            } else {
                center_bottom_x_global = (center_bottom_x_left + center_bottom_x_right) / 2f64;
                center_bottom_y_global = (center_bottom_y_left + center_bottom_y_right) / 2f64;
            }
            angle += arc_angle;
            total_distance_from_root += total_item_length;
        }
    }
    segments
}

fn get_rect(w: f64, h: f64, segment: &Segment) -> Rect {
    let mut points = Vec::with_capacity(8);
    let delta_w2 = w - SEMI_W;
    let de_w1;
    let de_w2;
    let real_rad;
    if segment.left {
        de_w1 = SEMI_W;
        de_w2 = -delta_w2;
        real_rad = segment.radius - SEMI_W; // has meaning only if segment is arc
    } else {
        de_w1 = delta_w2;
        de_w2 = -SEMI_W;
        real_rad = segment.radius + SEMI_W; // has meaning only if segment is arc
    }
    let angle_end;
    let center_top_x;
    let center_top_y;
    if segment.straight {
        angle_end = segment.angle_start;
        center_top_x = segment.center_bottom_x - h * angle_end.sin();
        center_top_y = segment.center_bottom_y + h * angle_end.cos();
    } else {
        let delta_angle = h / real_rad;
        angle_end = segment.angle_start + delta_angle;
        center_top_x = segment.center_bottom_x + segment.radius * (angle_end.cos() - segment.angle_start.cos());
        center_top_y = segment.center_bottom_y + segment.radius * (angle_end.sin() - segment.angle_start.sin());
    }
    for de_w in [de_w1, de_w2] {
        let pointx = segment.center_bottom_x + de_w * segment.angle_start.cos();
        let pointy = segment.center_bottom_y + de_w * segment.angle_start.sin();
        points.push([pointx, pointy]);
        let pointx = center_top_x + de_w * angle_end.cos();
        let pointy = center_top_y + de_w * angle_end.sin();
        points.push([pointx, pointy]);
    }
    if segment.straight == false {
        let start_ceil = (segment.angle_start / (PI / 2f64)).ceil() as i32;
        let end_ceil = (angle_end / (PI / 2f64)).ceil() as i32;
        let (min_ceil, max_ceil) = if start_ceil > end_ceil {(end_ceil, start_ceil)} else {(start_ceil, end_ceil)};
        if max_ceil - min_ceil > 0 {
            let mut counter = 0;
            let mut pos = min_ceil;
            let center_x = segment.center_bottom_x - segment.angle_start.cos() * segment.radius;
            let center_y = segment.center_bottom_y - segment.angle_start.sin() * segment.radius;
            let this_radius;
            if (segment.radius > 0f64 && (!segment.left)) || (segment.radius < 0f64 && segment.left) {
                this_radius = segment.radius.abs() + delta_w2;
            } else {
                this_radius = segment.radius.abs() + SEMI_W;
            }
            while counter < 4 && pos < max_ceil {
                let val = ((pos % 4) + 4) % 4;
                match val {
                    0 => points.push([center_x + this_radius, center_y]),
                    1 => points.push([center_x, center_y - this_radius]),
                    2 => points.push([center_x - this_radius, center_y]),
                    3 => points.push([center_x, center_y + this_radius]),
                    _ => panic!("val must be 0, 1, 2, or 3"),
                }
                counter += 1;
                pos += 1;
            }
        }
    }
    let mut x_max = points[0][0];
    let mut x_min = points[0][0];
    let mut y_max = points[0][1];
    let mut y_min = points[0][1];
    for point in points {
        if point[0] > x_max {
            x_max = point[0]
        }
        if point[0] < x_min {
            x_min = point[0]
        }
        if point[1] > y_max {
            y_max = point[1]
        }
        if point[1] < y_min {
            y_min = point[1]
        }
    }
    if x_max > SIZE_X as f64 - 1.0 {
        x_max = SIZE_X as f64 - 1.0;
    }
    if y_max > SIZE_Y as f64 - 1.0 {
        y_max = SIZE_Y as f64 - 1.0;
    }
    let x = if x_min.floor() as i32 > 0 {x_min.floor() as i32} else {0};
    let y = if y_min.floor() as i32 > 0 {y_min.floor() as i32} else {0};
    let width = x_max.ceil() as i32 - x;
    let height = y_max.ceil() as i32 - y;
    Rect{x, y, width, height} 
}

fn mat_vec_mul(mat: [[f64; 2]; 2], x: f64, y: f64) -> (f64, f64) {
    let new_x = mat[0][0] * x + mat[0][1] * y;
    let new_y = mat[1][0] * x + mat[1][1] * y;
    (new_x, new_y)
}

fn xi_yi(theta: f64, r: f64) -> (f64, f64) {
    let xi = r * theta.cos();
    let yi = r * theta.sin();
    (xi, yi)
}

fn x1_y1_scaling2(segment: &Segment,
                  x_circle: f64,
                  y_circle: f64,
                  scaling: f64,
                  source_y0: f64,
                  source_h: f64,
                  beta: f64) -> (f64, f64, f64) {
    // scaling is source/dest, source is original texture, dest is final pixels
    let x_circle_scaled = x_circle / scaling;
    let y_circle_scaled = y_circle / scaling; 
    let x1;
    if segment.left {
        x1 = -segment.radius - SEMI_W + x_circle_scaled;
    } else {
        x1 = -segment.radius + SEMI_W - x_circle_scaled;
    }
    let scaling2 = beta / (source_h / scaling);
    let y1 = -segment.angle_start / scaling2 - y_circle_scaled + source_y0 / scaling;
    (x1, y1, scaling2)
}

fn xfyf(xi: f64, yi: f64, x1: f64, y1: f64, scaling2: f64, center_x:f64, center_y: f64) -> (f64, f64) {
    let r2 = xi - x1;
    let alpha = (yi - y1) * scaling2;
    (center_x + r2 * alpha.cos(), center_y + r2 * alpha.sin())
}

fn get_dydx(theta: f64, scaling2: f64, x1: f64, y1: f64, radius_scaled: f64) -> (f64, f64) {
    // generated with sympy, see circle1.py
    let sith = theta.sin();
    let coth = theta.cos();
    let cos2 = (scaling2*(radius_scaled*sith - y1)).cos();
    let sin2 = (scaling2*(radius_scaled*sith - y1)).sin();
    let dy = scaling2*(radius_scaled*coth - x1)*coth*cos2 - sith*sin2;
    let dx = -scaling2*(radius_scaled*coth - x1)*sin2*coth - sith*cos2;
    (dy, dx)
}

struct PointForSmoothstep {
    xf: f64,
    yf: f64,
    dfx: f64,
    dfy: f64,
}

fn get_k1_k2_b_perpendicular(p: &PointForSmoothstep) -> (f64, f64, f64) { // coefficients of straight line perpendicular to tangent
    // k1*y - k2*x = b
    let k1 = p.dfy;
    let k2 = -p.dfx;
    let b = p.yf*p.dfy + p.xf*p.dfx;
    (k1, k2, b)
}

fn get_intersection_or_parallel(k1_f: f64, k2_f: f64, b_f: f64, k1_s: f64, k2_s: f64, b_s: f64) -> (bool, f64, f64) {
    // intersection of two straight lines
    // with coefs f (first) and s (second)
    let coef1 = k2_s * k1_f - k1_s * k2_f;
    if coef1 == 0.0 {
        (true, 0.0, 0.0)
    } else {
        let coef2 = b_f*k1_s - b_s*k1_f;
        let coef3 = b_f*k2_s - b_s*k2_f;
        let x = coef2 / coef1;
        let y = coef3 / coef1;
        (false, x, y)
    }
}

fn get_angle_start_angle_diff_r_start_r_diff(p1: &PointForSmoothstep,
                                             p2: &PointForSmoothstep,
                                             x_intersect: f64,
                                             y_intersect: f64,
                                             segment: &Segment) -> (f64, f64, f64, f64) {
    let x1 = p1.xf - x_intersect;
    let y1 = p1.yf - y_intersect;
    let x2 = p2.xf - x_intersect;
    let y2 = p2.yf - y_intersect;

    let angle_start = (y1).atan2(x1);
    let angle_stop = (y2).atan2(x2);
    let mut angle_diff = angle_stop - angle_start;

    let xd = p1.dfx;
    let yd = p1.dfy;
    let det;
    if segment.left {
        det = -x1*yd + y1*xd;
    } else {
        det = x1*yd - y1*xd;
    }
    if det > 0.0 && angle_diff > 0.0 {
        angle_diff -= 2.0 * PI;
    } else if det < 0.0 && angle_diff < 0.0 {
        angle_diff += 2.0 * PI;
    }
    let r_start = (x1 * x1 + y1 * y1).sqrt();
    let r_stop = (x2 * x2 + y2 * y2).sqrt();
    let r_diff = r_stop - r_start;
    (angle_start, angle_diff, r_start, r_diff)
}

fn get_values_segment(p1: &PointForSmoothstep,
                      p2: &PointForSmoothstep, 
                      segment: &Segment) -> (f64, f64, f64, f64, f64, f64) {
    let (k1_f, k2_f, b_f) = get_k1_k2_b_perpendicular(&p1);
    let (k1_s, k2_s, b_s) = get_k1_k2_b_perpendicular(&p2);
    let (are_parallel, x_intersect, y_intersect) = get_intersection_or_parallel(k1_f, k2_f, b_f, k1_s, k2_s, b_s);
    if are_parallel {
        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
    } else {
        let (angle_start, angle_diff, r_start, r_diff) = get_angle_start_angle_diff_r_start_r_diff(&p1,
                                                                                             &p2,
                                                                                             x_intersect,
                                                                                             y_intersect,
                                                                                             segment);
        (x_intersect, y_intersect, angle_start, angle_diff, r_start, r_diff)
    }
}

fn smoothstep_border_algorithm(segment: &Segment,
                          theta_min: f64,
                          theta_max: f64,
                          circle_radius: f64,
                          x_circle: f64,
                          y_circle: f64,
                          scaling: f64,
                          source_y0: f64,
                          source_h: f64,
                          beta: f64,
                          center_x: f64,
                          center_y: f64,
                          arr: &mut [[f32; crate::BUF_LENGTH]],
                          entity_num: usize) {
    let (x1, y1, scaling2) = x1_y1_scaling2(segment,
                                                           x_circle,
                                                           y_circle,
                                                           scaling,
                                                           source_y0,
                                                           source_h,
                                                           beta);
    let theta_interm = (theta_min + theta_max) / 2.0;
    let radius_scaled = circle_radius / scaling;
    let mut points: Vec<PointForSmoothstep> = vec![];
    let thetas = [theta_max, theta_interm, theta_min];
    for theta in thetas {
        let theta_to_use;
        if segment.left {
            theta_to_use = PI - theta;
            //theta_to_use = theta;
        } else {
            theta_to_use = theta;
        }
        let (xi, yi) = xi_yi(theta_to_use, radius_scaled);
        let (xf, yf) = xfyf(xi, yi, x1, y1, scaling2, center_x, center_y);
        let (dfy, dfx) = get_dydx(theta_to_use, scaling2, x1, y1, radius_scaled);
        let point = PointForSmoothstep{xf, yf, dfx, dfy};
        points.push(point);
    }
    for i in 0..2 {
        let point1 = &points[i];
        let point2 = &points[i + 1];
        let (x_intersect,
             y_intersect,
             angle_start,
             angle_diff,
             r_start,
             r_diff) = get_values_segment(point1, point2, segment);
        arr[4][entity_num*4 + i*2] = x_intersect as f32;
        arr[4][entity_num*4 + i*2 + 1] = y_intersect as f32;
        arr[5][entity_num*4 + i*2] = angle_start as f32;
        arr[5][entity_num*4 + i*2 + 1] = angle_diff as f32;
        arr[6][entity_num*4 + i*2] = r_start as f32;
        arr[6][entity_num*4 + i*2 + 1] = r_diff as f32;
    }
}

fn populate_leaf(rect: &Rect,
                 segment: &Segment,
                 source_w: f64,
                 source_h: f64,
                 source_x0: f64,
                 source_y0: f64,
                 is_leaf: bool,
                 f_source_ref: f64,
                 scaling: f64,
                 mut arr: &mut [[f32; crate::BUF_LENGTH]],
                 entity_num: usize,
                 theta_min: f64,
                 theta_max: f64,
                 circle_radius: f64,
                 x_circle: f64,
                 y_circle: f64,
                 prev_left: i32,
                 prev_right: i32) {
    let straightleaf = ((segment.straight as i32) * 2 + (is_leaf as i32)) as f32;
    arr[0][entity_num * 4] = rect.x as f32;
    arr[0][entity_num * 4 + 1] = rect.y as f32;
    arr[0][entity_num * 4 + 2] = rect.width as f32;
    arr[0][entity_num * 4 + 3] = rect.height as f32;
    arr[2][entity_num * 4] = straightleaf;
    arr[3][entity_num * 4] = segment.distance_from_root as f32;
    arr[3][entity_num * 4 + 2] = prev_left as f32;
    arr[3][entity_num * 4 + 3] = prev_right as f32;
    let semi_w_big_side = (source_w - f_source_ref / 2f64) / scaling;
    if segment.straight {
        let si = segment.angle_start.sin();
        let co = segment.angle_start.cos();
        let mat;
        let corner_dest_x;
        let corner_dest_y;
        if segment.left {
            mat = [[-scaling * co, -scaling * si],
                   [-scaling * si, scaling * co]];
            corner_dest_x = segment.center_bottom_x + SEMI_W * co;
            corner_dest_y = segment.center_bottom_y + SEMI_W * si;
        } else {
            mat = [[scaling * co, scaling * si],
                   [-scaling * si, scaling * co]];
            corner_dest_x = segment.center_bottom_x - SEMI_W * co;
            corner_dest_y = segment.center_bottom_y - SEMI_W * si;
        }
        let (conv_cdx, conv_cdy) = mat_vec_mul(mat, corner_dest_x, corner_dest_y);
        let x_to_plus = source_x0 - conv_cdx;
        let y_to_plus = source_y0 - conv_cdy;

        arr[1][entity_num * 4] = mat[0][0] as f32;
        arr[1][entity_num * 4 + 1] = mat[0][1] as f32;
        arr[1][entity_num * 4 + 2] = mat[1][0] as f32;
        arr[1][entity_num * 4 + 3] = mat[1][1] as f32;

        arr[2][entity_num * 4 + 2] = x_to_plus as f32;
        arr[2][entity_num * 4 + 3] = y_to_plus as f32;
    } else {
        let beta;
        if segment.left {
            beta = (source_h / scaling) / (segment.radius - SEMI_W);
        } else {
            beta = (source_h / scaling) / (segment.radius + SEMI_W);
        }
        let center_x = segment.center_bottom_x - segment.radius * segment.angle_start.cos();
        let center_y = segment.center_bottom_y - segment.radius * segment.angle_start.sin();
        let bound_min;
        let bound_max;
        let abs_radius = segment.radius.abs();
        if segment.left {
            if segment.radius > 0.0 {
                bound_min = abs_radius - semi_w_big_side;
                bound_max = abs_radius + SEMI_W;
            } else {
                bound_min = abs_radius - SEMI_W;
                bound_max = abs_radius + semi_w_big_side;
            }
        } else {
            if segment.radius > 0.0 {
                bound_min = abs_radius - SEMI_W;
                bound_max = abs_radius + semi_w_big_side;
            } else {
                bound_min = abs_radius - semi_w_big_side;
                bound_max = abs_radius + SEMI_W;
            }
        }
        let conv_angle_start;
        if segment.radius > 0.0 {
            conv_angle_start = segment.angle_start;
        } else {
            conv_angle_start = segment.angle_start + PI;
        }
        arr[1][entity_num * 4] = center_x as f32;
        arr[1][entity_num * 4 + 1] = center_y as f32;
        arr[1][entity_num * 4 + 2] = bound_min as f32;
        arr[1][entity_num * 4 + 3] = bound_max as f32;
        arr[2][entity_num * 4 + 1] = if segment.left { 1.0 } else { 0.0 };
        arr[2][entity_num * 4 + 2] = segment.radius as f32;
        arr[2][entity_num * 4 + 3] = conv_angle_start as f32;
        arr[3][entity_num * 4 + 1] = beta as f32;
        if beta.abs() <= ABS_BETA_LIMIT {
            smoothstep_border_algorithm(segment,
                          theta_min,
                          theta_max,
                          circle_radius,
                          x_circle,
                          y_circle,
                          scaling,
                          source_y0,
                          source_h,
                          beta,
                          center_x,
                          center_y,
                          &mut arr,
                          entity_num);
        }
    }
}

fn get_trunk_segment(segments: &Vec<Segment>, segment_num: usize) -> i32 {
    let source_segment = &segments[segment_num];
    let mut i: i32 = (segment_num as i32) - 1;
    while i >= 0 {
        if *(&segments[i as usize].left) == source_segment.left || *(&segments[i as usize].is_with_zero_opposite) {
            break;
        }
        i -= 1;
    }
    i
}

#[wasm_bindgen]
pub struct DrawingParams {
    pub scaling: f64,
    pub trunk_w: f64,
    pub trunk_h: f64,
    pub trunk_x0: f64,
    pub trunk_y0: f64,
    pub leaf_w: f64,
    pub leaf_h: f64,
    pub leaf_x0: f64,
    pub leaf_y0: f64,
    pub w_common: f64,
    pub h_bottom: f64,
    pub h_top: f64,
    pub width_ratio: f64,
    pub height_ratio: f64,
    pub source_ref_ratio: f64,
    pub x_circle: f64,
    pub y_circle: f64,
    pub radius: f64,
    pub theta_max: f64,
    pub theta_min: f64,
    
}

impl crate::GameState {
    pub fn generate_shader_and_populate_values(&mut self,
                                               width_ratio: f64,
                                               height_ratio: f64,
                                               source_ref_ratio: f64,
                                               x_circle: f64,
                                               y_circle: f64,
                                               radius: f64,
                                               theta_max: f64,
                                               theta_min: f64,) -> String {

        let scaling: f64 = source_ref_ratio / (F_DEST_REF);
        let trunk_w = source_ref_ratio;
        let trunk_h = source_ref_ratio * PI / 6f64;
        let trunk_x0 = 0f64;
        let trunk_y0 = 0f64;
        let leaf_w = width_ratio;
        let leaf_h = height_ratio - trunk_h;
        let leaf_x0 = 0f64;
        let leaf_y0 = trunk_h;
        let (w_common, h_bottom, h_top) = get_size(width_ratio, height_ratio, source_ref_ratio);
        self.drawing_params = DrawingParams {
            scaling,
            trunk_w,
            trunk_h,
            trunk_x0,
            trunk_y0,
            leaf_w,
            leaf_h,
            leaf_x0,
            leaf_y0,
            w_common,
            h_bottom,
            h_top,
            width_ratio,
            height_ratio,
            source_ref_ratio,
            x_circle,
            y_circle,
            radius,
            theta_max,
            theta_min,
        };

        let shader_str_with_value = format!(r##"#version 300 es
        
            precision mediump float;
            const uint MAX_RECTS = {:?}u;
            const float SCALING = {:?};
            const float PI = 3.1415926538;
            const float leaf_x0 = {:?};
            const float leaf_y0 = {:?};
            const float leaf_w = {:?};
            const float leaf_h = {:?};
            const float trunk_x0 = {:?};
            const float trunk_y0 = {:?};
            const float trunk_w = {:?};
            const float trunk_h = {:?};
            const float SEMI_W = {:?};
            const float MIN_A_DIF = SEMI_W * PI;
            const float CANVAS_REF_WIDTH = {:?};
            const float x_circle = {:?};
            const float y_circle = {:?};
            const float radius_circle = {:?};
            const float line_width = {:?};
            const float ABS_BETA_LIMIT = {:?};
            const float shadow_width = {:?};
            const float SHADOW_STRENGTH = {:?};
            const vec4 border_color = vec4(0.7, 0.8, 0.5, 1.0);
            const vec4 shadow_color = vec4(0.0, 0.0, 0.1, 1.0);

            layout(std140) uniform Position_size {{
                vec4 position_size[MAX_RECTS];
            }};
            layout(std140) uniform Center_x_y_bound_min_max_mat {{
                vec4 center_x_y_bound_min_max_mat[MAX_RECTS];
            }};
            layout(std140) uniform Straightleaf_left_radius_convastart_x_y_to_plus {{
                vec4 straightleaf_left_radius_convastart_x_y_to_plus[MAX_RECTS]; // if straight, left (y) is not used
            }};
            layout(std140) uniform Dist_from_root_beta_prevleft_prevright {{
                vec4 dist_from_root_beta_prevleft_prevright[MAX_RECTS];
            }};
            layout(std140) uniform Smoothstepcenter_xy_12 {{
                vec4 smoothstepcenter_xy_12[MAX_RECTS];
            }};
            layout(std140) uniform Angle1_dif1_angle2_dif2 {{ // for radial smoothstep interpolation
                vec4 angle1_dif1_angle2_dif2[MAX_RECTS];
            }};
            layout(std140) uniform Radius1_dif1_radius2_dif2 {{ // for radial smoothstep interpolation
                vec4 radius1_dif1_radius2_dif2[MAX_RECTS];
            }};

            uniform uint rectCount;
            uniform float canvas_w;
            uniform sampler2D u_image;
            
            out vec4 outColor;

            float get_dist_from_root(float x_source,
                                    float y_source,
                                    float source_h,
                                    float source_w,
                                    float source_x0,
                                    float source_y0,
                                    uint i,
                                    bool is_straight) {{
                if (x_source < source_x0 ||
                    x_source > source_x0 + source_w ||
                    y_source < source_y0 ||
                    y_source > source_y0 + source_h ||
                    texture(u_image, vec2(x_source, y_source)).w < 0.5) return -MIN_A_DIF * 4.0;
                float dist_from_base = (y_source - source_y0) / SCALING;
                if (!is_straight) {{
                    float radius = straightleaf_left_radius_convastart_x_y_to_plus[i].z;
                    float left = straightleaf_left_radius_convastart_x_y_to_plus[i].y;
                    if (left > 0.5)
                        dist_from_base = dist_from_base * radius / (radius - SEMI_W);
                    else
                        dist_from_base = dist_from_base * radius / (radius + SEMI_W);
                }}
                float base_dist_from_root = dist_from_root_beta_prevleft_prevright[i].x;
                return base_dist_from_root + dist_from_base;
            }}

            vec4 get_out_color(float radius_index, float x_source, float y_source) {{
                if (radius_index < 1.0)
                    return border_color * (1.0 - radius_index) +
                                texture(u_image, vec2(x_source, y_source)) * radius_index;
                    
                return texture(u_image, vec2(x_source, y_source));
            }}

            vec4 get_shadow_color(float radius_index) {{
                radius_index = radius_index * SHADOW_STRENGTH + 1.0 - SHADOW_STRENGTH;
                return shadow_color * (1.0 - radius_index) +
                    outColor * radius_index;
            }}

            float my_smoothstep(float x, float width, float height) {{ // x between 0 and width
                // original smoothstep: 3 * x**2 - 2 * x**3; 0->0, 1->1
                float x_scaled = x / width; // from 0 to 1
                return height * (3.0 * x_scaled*x_scaled - 2.0 * x_scaled*x_scaled*x_scaled); // from 0 to height
            }}

            vec2 get_x_y_straight(uint i, vec2 pos) {{
                // matrix multiplication
                // (x y) (x)
                //  z w   y
                float x_to_plus = straightleaf_left_radius_convastart_x_y_to_plus[i].z;
                float y_to_plus = straightleaf_left_radius_convastart_x_y_to_plus[i].w;
                float x_source = center_x_y_bound_min_max_mat[i].x * pos.x +
                                center_x_y_bound_min_max_mat[i].y * pos.y +
                                x_to_plus;
                float y_source = center_x_y_bound_min_max_mat[i].z * pos.x +
                                center_x_y_bound_min_max_mat[i].w * pos.y +
                                y_to_plus;
                return vec2(x_source, y_source);
            }}

            float get_radius_index_smoothstep(uint i, vec2 pos, uint segment_number, float lineshadow_width) {{
                float x_intersect;
                float y_intersect;
                float angle_start;
                float angle_diff;
                float r_start;
                float r_diff;
                if (segment_number == 0u) {{
                    x_intersect = smoothstepcenter_xy_12[i].x;
                    y_intersect = smoothstepcenter_xy_12[i].y;
                    angle_start = angle1_dif1_angle2_dif2[i].x;
                    angle_diff = angle1_dif1_angle2_dif2[i].y;
                    r_start = radius1_dif1_radius2_dif2[i].x;
                    r_diff = radius1_dif1_radius2_dif2[i].y;
                }} else {{
                    x_intersect = smoothstepcenter_xy_12[i].z;
                    y_intersect = smoothstepcenter_xy_12[i].w;
                    angle_start = angle1_dif1_angle2_dif2[i].z;
                    angle_diff = angle1_dif1_angle2_dif2[i].w;
                    r_start = radius1_dif1_radius2_dif2[i].z;
                    r_diff = radius1_dif1_radius2_dif2[i].w;
                }}
                float deltax = pos.x - x_intersect;
                float deltay = pos.y - y_intersect;
                float angle = atan(deltay, deltax);
                float radius = sqrt(deltay*deltay + deltax*deltax);
                float w = angle_diff;
                float h = r_diff;
                float y2 = radius - r_start;
                if (angle_diff > 0.0 && angle < angle_start)
                    angle += PI * 2.0;
                else {{
                    if (angle_diff < 0.0 && angle > angle_start)
                        angle -= PI * 2.0;
                }}
                float x2 = angle - angle_start;
                if (abs(x2  - w/2.0) > abs(w/2.0)) return 10.0; // just arbitrary number > 1
                float y_ref = my_smoothstep(x2, w, h);
                return abs(y_ref - y2) / lineshadow_width;
            }}

            float get_shadow_ratio_curled(uint i, vec2 pos, float source_h, float source_y0) {{
                if (abs(beta) > ABS_BETA_LIMIT - 0.0001) {{ // a bit arbitrary value where fancy algorithm doesn't work well
                    float center_x = center_x_y_bound_min_max_mat[i].x;
                    float center_y = center_x_y_bound_min_max_mat[i].y;
                    float bound_min = center_x_y_bound_min_max_mat[i].z;
                    float bound_max = center_x_y_bound_min_max_mat[i].w;
                    float left = straightleaf_left_radius_convastart_x_y_to_plus[i].y;
                    float radius = straightleaf_left_radius_convastart_x_y_to_plus[i].z;
                    float converted_angle_start = straightleaf_left_radius_convastart_x_y_to_plus[i].w;
                    float beta = dist_from_root_beta_prevleft_prevright[i].y;

                    float deltax = pos.x - center_x;
                    float deltay = pos.y - center_y;
                    float dist = sqrt(deltax * deltax +
                                        deltay * deltay);
                    if (bound_min > dist ||
                        dist > bound_max) return 10.0; // arbitrary big number
                    float x_dest;
                    if (left > 0.5) {{
                        if (radius > 0.0)
                            x_dest = bound_max - dist;
                        else
                            x_dest = dist - bound_min;
                    }} else {{
                        if (radius > 0.0)
                            x_dest = dist - bound_min;
                        else
                            x_dest = bound_max - dist;
                    }}
                    float x_source = x_dest * SCALING;
                    float gamma = atan(deltay, deltax);
                    float converted_alpha1 = (converted_angle_start - gamma)
                                                / (2.0 * PI);
                    float converted_alpha2 = (converted_angle_start - gamma
                                                + beta)
                                                / (2.0 * PI);
                    int start;
                    int end;
                    if (beta > 0.0) {{
                        start = int(ceil(converted_alpha1));
                        end = int(floor(converted_alpha2));
                    }} else {{
                        start = int(ceil(converted_alpha2));
                        end = int(floor(converted_alpha1));
                    }}
                    if (end < start) return 10.0; // arbitrary big number
                    for (int y_before_conversion=start; y_before_conversion<=end; y_before_conversion++) {{
                        float y_angle = float(y_before_conversion) * (2.0 * PI) + gamma;
                        float beta_rate = (y_angle - converted_angle_start) / beta;
                        float height_this_point_source = source_h * beta_rate;
                        float y_source = source_y0 + height_this_point_source;
                        
                        float deltay_source = y_source - y_circle;
                        float x_intersect = x_circle - sqrt(radius_circle*radius_circle -
                                                            deltay_source*deltay_source);
                        float radius_index = abs(x_source - x_intersect) / (shadow_width * SCALING);
                        if (radius_index < 1.0) return radius_index;
                    }}
                }} else {{
                    float radius_index1 = get_radius_index_smoothstep(i, pos, 0u, shadow_width);
                    if (radius_index1 < 1.0) return radius_index1;
                    else {{
                        float radius_index2 = get_radius_index_smoothstep(i, pos, 1u, shadow_width);
                        return radius_index2;
                    }}
                }}
                return 10.0; // arbitrary number > 1
            }}

            float get_shadow_ratio(float prev_i, vec2 pos) {{
                if (prev_i < 0.0) return 10.0; // arbitrary big number
                uint i = uint(round(prev_i));
                bool is_straight;
                bool is_leaf;
                float straight_leaf = straightleaf_left_radius_convastart_x_y_to_plus[i].x;
                if (straight_leaf < 0.5) {{
                    is_straight = false;
                    is_leaf = false;
                }} else {{
                    if (straight_leaf < 1.5) {{
                        is_straight = false;
                        is_leaf = true;
                    }} else {{
                        if (straight_leaf < 2.5) {{
                            is_straight = true;
                            is_leaf = false;
                        }} else {{
                            is_straight = true;
                            is_leaf = true;
                        }}
                    }}
                }}
                float source_w;
                float source_h;
                float source_x0;
                float source_y0;
                if (is_leaf) {{
                    source_w = leaf_w;
                    source_h = leaf_h;
                    source_x0 = leaf_x0;
                    source_y0 = leaf_y0;
                }} else {{
                    source_w = trunk_w;
                    source_h = trunk_h;
                    source_x0 = trunk_x0;
                    source_y0 = trunk_y0;
                }}
                if (is_straight) {{
                    vec2 x_y_source = get_x_y_straight(i, pos);
                    float x_source = x_y_source.x;
                    float y_source = x_y_source.y;
                    float radius_x = x_source - x_circle;
                    float radius_y = y_source - y_circle;
                    float now_radius = sqrt(radius_x*radius_x + radius_y*radius_y);
                    float radius_index = abs(now_radius - radius_circle) / (shadow_width * SCALING);
                    return radius_index;
                }} else {{
                    float radius_index = get_shadow_ratio_curled(i, pos, source_h, source_y0);
                    return radius_index;
                }}
            }}

            void main() {{
                float a = - MIN_A_DIF * 2.0;
                vec2 pos = gl_FragCoord.xy / vec2(canvas_w / CANVAS_REF_WIDTH, canvas_w / CANVAS_REF_WIDTH);
                
                for (uint i=0u; i < rectCount; i++) {{
                    if (pos.x < position_size[i].x ||
                        pos.y < position_size[i].y ||
                        pos.x > position_size[i].x + position_size[i].z ||
                        pos.y > position_size[i].y + position_size[i].w) continue;
                    
                    bool is_straight;
                    bool is_leaf;
                    float straight_leaf = straightleaf_left_radius_convastart_x_y_to_plus[i].x;
                    if (straight_leaf < 0.5) {{
                        is_straight = false;
                        is_leaf = false;
                    }} else {{
                        if (straight_leaf < 1.5) {{
                            is_straight = false;
                            is_leaf = true;
                        }} else {{
                            if (straight_leaf < 2.5) {{
                                is_straight = true;
                                is_leaf = false;
                            }} else {{
                                is_straight = true;
                                is_leaf = true;
                            }}
                        }}
                    }}

                    float source_w;
                    float source_h;
                    float source_x0;
                    float source_y0;
                    if (is_leaf) {{
                        source_w = leaf_w;
                        source_h = leaf_h;
                        source_x0 = leaf_x0;
                        source_y0 = leaf_y0;
                    }} else {{
                        source_w = trunk_w;
                        source_h = trunk_h;
                        source_x0 = trunk_x0;
                        source_y0 = trunk_y0;
                    }}
                    if (is_straight) {{
                        vec2 x_y_source = get_x_y_straight(i, pos);
                        float x_source = x_y_source.x;
                        float y_source = x_y_source.y;
                        float now_a = get_dist_from_root(x_source,
                                                        y_source,
                                                        source_h,
                                                        source_w,
                                                        source_x0,
                                                        source_y0,
                                                        i,
                                                        is_straight);
                        if (now_a > a + MIN_A_DIF) {{
                            a = now_a;
                            float radius_x = x_source - x_circle;
                            float radius_y = y_source - y_circle;
                            float now_radius = sqrt(radius_x*radius_x + radius_y*radius_y);
                            float radius_index = abs(now_radius - radius_circle) / (line_width * SCALING);
                            outColor = get_out_color(radius_index, x_source, y_source);
                            float prev_left = dist_from_root_beta_prevleft_prevright[i].z;
                            float prev_right = dist_from_root_beta_prevleft_prevright[i].w;
                            float shadow_index1 = get_shadow_ratio(prev_left, pos);
                            if (shadow_index1 < 1.0) {{
                                outColor = get_shadow_color(shadow_index1);
                            }}
                            float shadow_index2 = get_shadow_ratio(prev_right, pos);
                            if (shadow_index2 < 1.0) {{
                                outColor = get_shadow_color(shadow_index2);
                            }}
                        }}
                    }} else {{
                        float center_x = center_x_y_bound_min_max_mat[i].x;
                        float center_y = center_x_y_bound_min_max_mat[i].y;
                        float bound_min = center_x_y_bound_min_max_mat[i].z;
                        float bound_max = center_x_y_bound_min_max_mat[i].w;
                        float left = straightleaf_left_radius_convastart_x_y_to_plus[i].y;
                        float radius = straightleaf_left_radius_convastart_x_y_to_plus[i].z;
                        float converted_angle_start = straightleaf_left_radius_convastart_x_y_to_plus[i].w;
                        float beta = dist_from_root_beta_prevleft_prevright[i].y;

                        float deltax = pos.x - center_x;
                        float deltay = pos.y - center_y;
                        float dist = sqrt(deltax * deltax +
                                        deltay * deltay);
                        if (bound_min > dist ||
                            dist > bound_max) continue;
                        float x_dest;
                        if (left > 0.5) {{
                            if (radius > 0.0)
                                x_dest = bound_max - dist;
                            else
                                x_dest = dist - bound_min;
                        }} else {{
                            if (radius > 0.0)
                                x_dest = dist - bound_min;
                            else
                                x_dest = bound_max - dist;
                        }}
                        float x_source = x_dest * SCALING;
                        float gamma = atan(deltay, deltax);
                        float converted_alpha1 = (converted_angle_start - gamma)
                                                / (2.0 * PI);
                        float converted_alpha2 = (converted_angle_start - gamma
                                                + beta)
                                                / (2.0 * PI);
                        int start;
                        int end;
                        if (beta > 0.0) {{
                            start = int(ceil(converted_alpha1));
                            end = int(floor(converted_alpha2));
                        }} else {{
                            start = int(ceil(converted_alpha2));
                            end = int(floor(converted_alpha1));
                        }}
                        if (end < start) continue;
                        for (int y_before_conversion=start; y_before_conversion<=end; y_before_conversion++) {{
                            float y_angle = float(y_before_conversion) * (2.0 * PI) + gamma;
                            float beta_rate = (y_angle - converted_angle_start) / beta;
                            float height_this_point_source = source_h * beta_rate;
                            float y_source = source_y0 + height_this_point_source;
                            float now_a = get_dist_from_root(x_source,
                                                            y_source,
                                                            source_h,
                                                            source_w,
                                                            source_x0,
                                                            source_y0,
                                                            i,
                                                            is_straight);
                            if (now_a > a + MIN_A_DIF) {{
                                a = now_a;
                                if (abs(beta) > ABS_BETA_LIMIT - 0.0001) {{ // a bit arbitrary value where fancy algorithm doesn't work well
                                    float deltay_source = y_source - y_circle;
                                    float x_intersect = x_circle - sqrt(radius_circle*radius_circle -
                                                                        deltay_source*deltay_source);
                                    float radius_index = abs(x_source - x_intersect) / (line_width * SCALING);
                                    outColor = get_out_color(radius_index, x_source, y_source);
                                }} else {{
                                    float radius_index1 = get_radius_index_smoothstep(i, pos, 0u, line_width);
                                    if (radius_index1 < 1.0) outColor = get_out_color(radius_index1, x_source, y_source);
                                    else {{
                                        float radius_index2 = get_radius_index_smoothstep(i, pos, 1u, line_width);
                                        outColor = get_out_color(radius_index2, x_source, y_source);
                                    }}
                                }}
                                float prev_left = dist_from_root_beta_prevleft_prevright[i].z;
                                float prev_right = dist_from_root_beta_prevleft_prevright[i].w;
                                float shadow_index1 = get_shadow_ratio(prev_left, pos);
                                if (shadow_index1 < 1.0) {{
                                    outColor = get_shadow_color(shadow_index1);
                                }}
                                float shadow_index2 = get_shadow_ratio(prev_right, pos);
                                if (shadow_index2 < 1.0) {{
                                    outColor = get_shadow_color(shadow_index2);
                                }}
                            }}
                        }}
                    }}
                }}
            }}
            "##, crate::MAX_RECTS,
                scaling,
                leaf_x0,
                leaf_y0,
                leaf_w,
                leaf_h,
                trunk_x0,
                trunk_y0,
                trunk_w,
                trunk_h,
                SEMI_W,
                crate::CANVAS_REF_WIDTH,
                x_circle,
                y_circle,
                radius,
                LINE_WIDTH,
                ABS_BETA_LIMIT,
                SHADOW_WIDTH,
                SHADOW_STRENGTH);
        shader_str_with_value
    }

    pub fn populate_arr(&mut self) -> usize {
        let segments = get_segments(F_DEST_REF, &self.tree);

        let mut entities_num = 0;
        let mut prev_left = -1;
        let mut prev_right = -1;
        for segment_num in 0..segments.len() {
            let leaf_segment = &segments[segment_num];
            let rect = get_rect(self.drawing_params.w_common, self.drawing_params.h_top, leaf_segment);
            populate_leaf(&rect,
                    leaf_segment,
                    self.drawing_params.leaf_w,
                    self.drawing_params.leaf_h,
                    self.drawing_params.leaf_x0,
                    self.drawing_params.leaf_y0,
                    true,
                    self.drawing_params.source_ref_ratio,
                    self.drawing_params.scaling,
                    &mut self.ubos_arr,
                    entities_num,
                    self.drawing_params.theta_min,
                    self.drawing_params.theta_max,
                    self.drawing_params.radius,
                    self.drawing_params.x_circle,
                    self.drawing_params.y_circle,
                    prev_left,
                    prev_right);
            let this_num = entities_num as i32;
            entities_num += 1;
            let trunk_segment_num = get_trunk_segment(&segments, segment_num);
            if trunk_segment_num >= 0 {
                let trunk_segment = &segments[trunk_segment_num as usize];
                let rect = get_rect(self.drawing_params.w_common, self.drawing_params.h_bottom, trunk_segment);
                populate_leaf(&rect,
                    trunk_segment,
                    self.drawing_params.trunk_w,
                    self.drawing_params.trunk_h,
                    self.drawing_params.trunk_x0,
                    self.drawing_params.trunk_y0,
                    false,
                    self.drawing_params.source_ref_ratio,
                    self.drawing_params.scaling,
                    &mut self.ubos_arr,
                    entities_num,
                    self.drawing_params.theta_min,
                    self.drawing_params.theta_max,
                    self.drawing_params.radius,
                    self.drawing_params.x_circle,
                    self.drawing_params.y_circle,
                    prev_left,
                    prev_right);
                entities_num += 1;
            }
            if leaf_segment.left {
                prev_left = this_num;
            } else {
                prev_right = this_num;
            }
        }
        
        entities_num
    }
}
