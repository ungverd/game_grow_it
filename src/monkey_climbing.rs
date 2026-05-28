use crate::START_X;

pub const DELTAX_FRONT: f32 = 3.0; //px
pub const DELTAX_BACK: f32 = 3.0; //px
pub const DELTAY_FRONT: f32 = 11.0; //px
pub const DELTAY_BACK: f32 = 11.0; //px
pub const LEG_SEGMENT_LENGTH: f32 = 10.0; //px
pub const ARM_SEGMENT_LENGTH: f32 = 9.0; //px
const MIN_DIST_FROM_ROOT: f32 = 5.0; //px, must be less then W * PI / 6 / 2
const PI: f32 = std::f32::consts::PI;
pub const W: f32 = crate::DEST_REF as f32; // three width in px
const CLIMBING_SPEED: f32 = 50.0; // px/s
pub const LEG_EXTENSION_COEFFICIENT: f32 = 1.9; // must be lightly under 2, bigger -> leg more straight
const STEP_RATIO: f32 = 2.0; // max distance between arm and leg is 2 times larger than min distance

pub struct MonkeyClimbing {
    total_height: f32,
    now_segment: usize,
    body_x: f32,
    body_y: f32,
}

struct TreeElForClimbingNotLinear {
    center_x: f32,
    center_y: f32,
    radius: f32,
    angle_stop: f32,
}

pub struct TreeElForClimbing {
    angle_start: f32,
    dist_from_root_start: f32,
    length: f32,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    right: u32,
    left: u32,
    for_not_linear: Option<TreeElForClimbingNotLinear>,
    arm_dist: f32,
    leg_dist: f32,
    step: f32, // each step, monkey's center does this dist, and moving limb does 2x this
               // and it's also a difference between bended and extended arms (or legs)
}

struct TreeUnitReduced {
    right: u32,
    left: u32,
}



fn make_joined_tree(tree: &Vec<crate::TreeUnit>) -> Vec<TreeUnitReduced>{
    let mut joined_tree: Vec<TreeUnitReduced> = vec![];
    if !tree.is_empty() {
        for unit in tree {
            let last = joined_tree.last_mut();
            match last {
                Some(last) => {
                    if last.left * unit.right == last.right * unit.left {
                        last.left += unit.left * unit.repeats;
                        last.right += unit.right * unit.repeats
                    } else {
                        joined_tree.push(TreeUnitReduced{left: unit.left * unit.repeats,
                                                         right: unit.right * unit.repeats,});
                    }
                }
                None => { joined_tree.push(TreeUnitReduced{left: unit.left * unit.repeats,
                                                           right: unit.right * unit.repeats}); }
            }
        }
    }
    joined_tree
}

fn make_tree_for_climbing_unit(reduced_unit: &TreeUnitReduced,
                               start_x: f32,
                               start_y: f32,
                               angle_start: f32,
                               dist_from_root_start: f32,
                               straight_up_dist: f32,
                               straight_down_dist: f32) -> (TreeElForClimbing, f32, f32, f32, f32) {
    let for_not_linear;
    let length;
    let end_x;
    let end_y;
    let angle_stop;
    let (arm_dist, leg_dist);
    if reduced_unit.right != reduced_unit.left {
        let dif_right_left = (reduced_unit.right - reduced_unit.left) as f32;
        let sum_right_left = (reduced_unit.right - reduced_unit.left) as f32;
        let delta_angle = dif_right_left * PI / 6.0;
        let radius = sum_right_left / dif_right_left * W / 2.0;
        length = delta_angle * radius;
        let center_x = start_x - radius * angle_start.cos();
        let center_y = start_y - radius * angle_start.sin();
        angle_stop = angle_start + delta_angle;
        end_x = center_x + radius * angle_stop.cos();
        end_y = center_y + radius * angle_stop.sin();
        (arm_dist, leg_dist) = get_circ_extended_length(radius);
        for_not_linear = Some(TreeElForClimbingNotLinear {
                                                    center_x,
                                                    center_y,
                                                    radius,
                                                    angle_stop,
        });
    } else {
        for_not_linear = None;
        length = W * PI / 6.0 * (reduced_unit.right as f32);
        end_x = start_x - length * angle_start.sin();
        end_y = start_y + length * angle_start.cos();
        angle_stop = angle_start;
        arm_dist = straight_up_dist;
        leg_dist = straight_down_dist;
    }
    let step = (arm_dist + leg_dist) * (STEP_RATIO - 1.0) / 2.0 / STEP_RATIO;
    (TreeElForClimbing {
        angle_start,
        dist_from_root_start,
        length,
        start_x,
        start_y,
        end_x,
        end_y,
        right: reduced_unit.right,
        left: reduced_unit.left,
        for_not_linear,
        arm_dist,
        leg_dist,
        step,
    }, end_x, end_y, dist_from_root_start + length, angle_stop)
}

pub fn make_tree_for_climbing(tree: &Vec<crate::TreeUnit>,
                          start_x: f32,
                          start_y: f32,
                          straight_up_dist: f32,
                          straight_down_dist: f32) -> Vec<TreeElForClimbing>{
    let joined_tree = make_joined_tree(tree);
    let mut tree_for_climbing: Vec<TreeElForClimbing> = vec![];
    let mut start_x = start_x;
    let mut start_y = start_y;
    let mut dist_from_root_start = 0.0;
    let mut angle_start = 0.0;
    for reduced_unit in &joined_tree {
        let (for_climbing_unit,
             start_x_new,
             start_y_new,
             dist_from_root_new,
             angle_start_new) = make_tree_for_climbing_unit(reduced_unit,
                start_x,
                start_y,
                angle_start,
                dist_from_root_start,
                straight_up_dist,
                straight_down_dist);
        start_x = start_x_new;
        start_y = start_y_new;
        angle_start = angle_start_new;
        dist_from_root_start = dist_from_root_new;
        tree_for_climbing.push(for_climbing_unit);
    }
    tree_for_climbing
}

fn get_segment_x_y_after_verif(dist_from_root: f32,
                               segment: &TreeElForClimbing) -> (f32, f32) {
    let dist_from_start = dist_from_root - segment.dist_from_root_start;
    match &segment.for_not_linear {
        None => {
            let delta_x = -dist_from_start * segment.angle_start.sin();
            let delta_y = dist_from_start * segment.angle_start.cos();
            return (segment.start_x + delta_x, segment.start_y + delta_y)
        }
        Some(not_linear) => {
            let segment_angle = not_linear.angle_stop - segment.angle_start;
            let angle_now = segment.angle_start + segment_angle * dist_from_start / segment.length;
            let center_x = not_linear.center_x;
            let center_y = not_linear.center_y;
            let radius = not_linear.radius;
            return (center_x + radius * angle_now.cos(),
                    center_y + radius * angle_now.sin())
        }
    }
}

fn get_segment(dist_from_root: f32,
                   tree: &Vec<TreeElForClimbing>,
                   segment_num: usize) -> Option<usize> {
    let mut segment_num = segment_num;
    let last = tree.last().unwrap();
    let max_dist_from_root = last.dist_from_root_start + last.length;
    if dist_from_root > max_dist_from_root {
        return None;
    }
    let tree_len = tree.len();
    let mut res_segment = tree.first().unwrap();
    let last  = tree.last().unwrap();
    let mut found = false;
    let mut now_segment;
    while !found {
        now_segment = tree.get(segment_num);
        match(now_segment) {
            None => {
                found = true;
                res_segment = last;
                actual_dist_from_root = last.dist_from_root_start + last.length
            }
            Some(tree_el) => {
                if dist_from_root >= tree_el.dist_from_root_start &&
                dist_from_root <= tree_el.dist_from_root_start + tree_el.length {
                    found = true;
                    res_segment = tree_el;
                    actual_dist_from_root = dist_from_root;
                } else {
                    if dist_from_root < tree_el.dist_from_root_start {
                        if dist_from_root < MIN_DIST_FROM_ROOT {
                            found = true;
                            res_segment = tree.first().unwrap();
                            actual_dist_from_root = MIN_DIST_FROM_ROOT;
                        } else {
                            segment_num -= 1;
                        }
                    } else {
                        if segment_num >= tree.len() - 1 {
                            found = true;
                            segment_num = tree.len() - 1;
                            res_segment = last;
                            actual_dist_from_root = last.dist_from_root_start + last.length;
                        } else {
                        segment_num += 1;
                        }
                    }
                }
            }
        }
    }
    Some(segment_num)
}


fn get_segment_x_y_dist_from_root(dist_from_root: f32,
                   tree: &Vec<TreeElForClimbing>,
                   segment_num: usize) -> (usize, f32, f32, f32) {
    let mut segment_num = segment_num;
    let tree_len = tree.len();
    if segment_num >= tree_len {
        segment_num = tree_len - 1;
    }
    let mut res_segment = tree.first().unwrap();
    let mut actual_dist_from_root = dist_from_root;
    if dist_from_root < MIN_DIST_FROM_ROOT {
        res_segment = tree.first().unwrap();
        actual_dist_from_root = MIN_DIST_FROM_ROOT;
    } else {
        let last  = tree.last().unwrap();
        let mut found = false;
        let mut now_segment;
        while !found {
            now_segment = tree.get(segment_num);
            match(now_segment) {
                None => {
                    found = true;
                    res_segment = last;
                    actual_dist_from_root = last.dist_from_root_start + last.length
                }
                Some(tree_el) => {
                    if dist_from_root >= tree_el.dist_from_root_start &&
                    dist_from_root <= tree_el.dist_from_root_start + tree_el.length {
                        found = true;
                        res_segment = tree_el;
                        actual_dist_from_root = dist_from_root;
                    } else {
                        if dist_from_root < tree_el.dist_from_root_start {
                            if dist_from_root < MIN_DIST_FROM_ROOT {
                                found = true;
                                res_segment = tree.first().unwrap();
                                actual_dist_from_root = MIN_DIST_FROM_ROOT;
                            } else {
                                segment_num -= 1;
                            }
                        } else {
                            if segment_num >= tree.len() - 1 {
                                found = true;
                                segment_num = tree.len() - 1;
                                res_segment = last;
                                actual_dist_from_root = last.dist_from_root_start + last.length;
                            } else {
                            segment_num += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    let (x, y) = get_segment_x_y_after_verif(actual_dist_from_root, res_segment);
    (segment_num, x, y, actual_dist_from_root)
}

impl MonkeyClimbing {
    pub fn new() -> MonkeyClimbing {
        MonkeyClimbing{
            total_height: 0.0,
            now_segment: 0,
            body_x: 0.0,
            body_y: 0.0,
        }
    }

    fn go_to_top(&mut self, tree_for_climbing: &Vec<TreeElForClimbing>) {
        let last = tree_for_climbing.last().unwrap();
        let num = tree_for_climbing.len() - 1;
        self.now_segment = num;
        self.total_height = last.dist_from_root_start + last.length;
        (self.body_x, self.body_y) = get_segment_x_y_after_verif(self.total_height,
                                                                        last);
    }
}

impl crate::Monkey {
    fn get_body_x_y_and_update_segment(&mut self,
                                       tree_for_climbing: &Vec<TreeElForClimbing>) {
        // Do this after updating total_height or tree undo 
        let current_segment = tree_for_climbing.get(self.climbing.now_segment);
        match &current_segment {
            None => {
                if tree_for_climbing.is_empty() {
                    self.monkey_state = crate::MonkeyState::running;
                    self.running.set_on_pos(crate::START_X as f32, crate::START_Y as f32);
                } else {
                    self.climbing.go_to_top(tree_for_climbing);
                }
            }
            Some(_) => {
                let (new_seg_num,
                     new_x,
                     new_y,
                     new_total_height) = get_segment_x_y_dist_from_root(self.climbing.total_height,
                                                                             tree_for_climbing,
                                                                 self.climbing.now_segment);
                self.climbing.body_x = new_x;
                self.climbing.body_y = new_y;
                self.climbing.total_height = new_total_height;
                self.climbing.now_segment = new_seg_num;
            }
        }
    } 
}

fn get_circle_circle_intersection(r1: f32, r2: f32, deltax: f32) -> (f32, f32) {
    // center of coordinates in center of big circle (r1),
    // center of r2 is deplaced at left on deltax
    let x_intersect = (r1*r1 - r2*r2 + deltax*deltax) / 2.0 / deltax;
    let y = (r1*r1 - x_intersect*x_intersect).sqrt();
    (x_intersect, y)
}

fn get_circ_extended_length(r: f32) -> (f32, f32) {
    let r_arm_extended = ARM_SEGMENT_LENGTH * LEG_EXTENSION_COEFFICIENT;
    let r_leg_extended = LEG_SEGMENT_LENGTH * LEG_EXTENSION_COEFFICIENT;
    let r_center = r.abs();
    let r_big = r_center + W / 2.0;
    let deltax_arm = r_center + DELTAX_FRONT;
    let deltax_leg = r_center + DELTAX_BACK;
    let (x_intersection_arm, y_intersection_arm) = get_circle_circle_intersection(
        r_big,
        r_arm_extended,
        deltax_arm);
    let angle_intersection_arm = y_intersection_arm.atan2(x_intersection_arm);
    let (x_intersection_leg, y_intersection_leg) = get_circle_circle_intersection(
        r_big,
        r_leg_extended,
        deltax_leg);
    let angle_intersection_leg = y_intersection_leg.atan2(x_intersection_leg);
    let angle_body_up = DELTAY_FRONT / r_center;
    let angle_body_down = DELTAY_BACK / r_center;
    let angle_up = angle_intersection_arm + angle_body_up;
    let angle_down = angle_body_down + angle_intersection_leg;
    let total_angle = angle_up + angle_down;
    let max_angle = PI * 2.0 / 3.0;
    let (converted_angle_up, converted_angle_down) = if total_angle < max_angle {
        (angle_up, angle_down)
    } else {
        (max_angle * angle_up / total_angle, max_angle * angle_down / total_angle)
    };
    let converted_length_up = converted_angle_up * r_center;
    let converted_length_down = converted_angle_down * r_center;
    (converted_length_up, converted_length_down)
}

struct LegEndPos {
    leg_pos_x: f32,
    leg_pos_y: f32,
    center_pos:f32, // if center is between center_pos1 and center_pos2,
                    // leg_end will be interpolated between leg_pos1 and leg_pos2.
                    // center_pos is distance from root
}

fn get_limb_pos_x_y_segment_known(dist_from_root: f32,
                             segment: &TreeElForClimbing,
                             is_left: bool) -> (f32, f32) {
    let dist_from_start_segment = dist_from_root - segment.dist_from_root_start;
    let multiplier = if is_left {-1.0} else {1.0};
    match &segment.for_not_linear {
        None => {
            let center_x = segment.start_x - dist_from_start_segment * segment.angle_start.sin();
            let center_y = segment.start_y + dist_from_start_segment * segment.angle_start.cos();
            let pos_x = center_x + W / 2.0 * multiplier * segment.angle_start.cos();
            let pos_y = center_y + W / 2.0 * multiplier * segment.angle_start.sin();
            (pos_x, pos_y)
        }
        Some(el) => {
            let now_radius = el.radius + W / 2.0 * multiplier;
            let now_angle = dist_from_start_segment / segment.length * (el.angle_stop - segment.angle_start);
            let pos_x = el.center_x + now_radius * now_angle.cos();
            let pos_y = el.center_y + now_radius * now_angle.sin();
            (pos_x, pos_y)
        }
    }
}

fn get_limb_pos_x_y(dist_from_root: f32,
               tree: &Vec<TreeElForClimbing>,
               current_segment: usize,
               is_left: bool) -> (f32, f32) {
    //let mut dist_from_root = dist_from_root;
    let current_segment_obj = tree.get(current_segment).unwrap();
    if dist_from_root >= current_segment_obj.dist_from_root_start {
        let dist_end = current_segment_obj.dist_from_root_start + current_segment_obj.length;
        if dist_from_root <= dist_end {
            return get_limb_pos_x_y_segment_known(dist_from_root,
                             current_segment_obj,
                                      is_left);
        } else {
            let last_segment = tree.last().unwrap();
            let max_dist_from_root = last_segment.dist_from_root_start + last_segment.length;
            if dist_from_root > max_dist_from_root {
                return get_limb_pos_x_y_segment_known(max_dist_from_root,
                                    last_segment,
                                    is_left);
            } else {
                for current_segment_i in current_segment+1..tree.len() {
                    let current_segment_obj = tree.get(current_segment_i).unwrap();
                    if dist_from_root >= current_segment_obj.dist_from_root_start {
                        return get_limb_pos_x_y_segment_known(dist_from_root,
                                    current_segment_obj,
                                    is_left);
                    }
                }
            }
        }
    } else {
        if dist_from_root <= 0.0 {
            return get_limb_pos_x_y_segment_known(0.0,
                                    tree.first().unwrap(),
                                    is_left)
        } else {
            for current_segment_i in (0..current_segment).rev() {
                let current_segment_obj = tree.get(current_segment_i).unwrap();
                let dist_end = current_segment_obj.dist_from_root_start + current_segment_obj.length;
                if dist_from_root <= dist_end {
                    return get_limb_pos_x_y_segment_known(dist_from_root,
                                current_segment_obj,
                                is_left);
                }
            }
        }
    }
    return (-1.0, -1.0); // must not get here
}

fn generate_leg_arrays(tree: &Vec<TreeElForClimbing>) {
    let mut left_leg_vec: Vec<LegEndPos> = vec![]; 
    let mut right_leg_vec: Vec<LegEndPos> = vec![]; 
    let mut left_arm_vec: Vec<LegEndPos> = vec![]; 
    let mut right_arm_vec: Vec<LegEndPos> = vec![];
    let mut finished = false;
    let mut i = 0;
    let mut first_pose = true;
    while !finished {
        if first_pose {
            let (left_leg_pos_x, left_leg_pos_y) = get_limb_pos_x_y(0.0,
                                                                        tree,
                                                                        0,
                                                                        true);
            let center_pos = MIN_DIST_FROM_ROOT;
            left_leg_vec.push(LegEndPos{leg_pos_x: left_leg_pos_x,
                                        leg_pos_y: left_leg_pos_y,
                                        center_pos});
            let (right_leg_pos_x, right_leg_pos_y) = get_limb_pos_x_y(0.0,
                                                                        tree,
                                                                        0,
                                                                        false);
            right_leg_vec.push(LegEndPos{leg_pos_x: right_leg_pos_x,
                                        leg_pos_y: right_leg_pos_y,
                                        center_pos});
            let now_segment = tree.first().unwrap();
            let l = now_segment.leg_dist;
            let a = now_segment.arm_dist;
            let step = now_segment.step;
            let try_right_arm = center_pos + a;
            let right_arm_segment = 
            first_pose = false;
        }
        let now_segment = tree.get(i).unwrap();
    }
}