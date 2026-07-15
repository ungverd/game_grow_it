use std::f32::MIN;

use crate::START_X;

pub const DELTAX_FRONT: f32 = 3.0; //px
pub const DELTAX_BACK: f32 = 3.0; //px
pub const DELTAY_FRONT: f32 = 11.0; //px
pub const DELTAY_BACK: f32 = 11.0; //px
pub const LEG_SEGMENT_LENGTH: f32 = 10.0; //px
pub const ARM_SEGMENT_LENGTH: f32 = 9.0; //px
pub const MIN_DIST_FROM_ROOT: f32 = 5.0; //px, must be less then W * PI / 6 / 2
const PI: f32 = std::f32::consts::PI;
pub const W: f32 = crate::DEST_REF as f32; // three width in px
const CLIMBING_SPEED: f32 = 50.0; // px/s
pub const LEG_EXTENSION_COEFFICIENT: f32 = 1.9; // must be lightly under 2, bigger -> leg more straight
const STEP_RATIO: f32 = 2.0; // max distance between arm and leg is 2 times larger than min distance

impl crate::Pos {
    fn new() -> crate::Pos {
        crate::Pos{x: 0.0, y: 0.0}
    }
}

pub struct MonkeyClimbing {
    pub total_height: f32,
    now_segment: usize,
    body_pos: crate::Pos,
    left_arm_right_arm_left_leg_right_leg: [crate::Pos; 4],
    goal_height: f32,
    pub on_goal: bool,
    pub vertex_arr: [f32; 60],
    pub texture_arr: [f32; 60]
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

pub struct TreeUnitReduced {
    right: u32,
    left: u32,
}


impl crate::TreeStruct {
    fn make_joined_tree(&mut self) {
        let tree = &self.tree;
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
        self.joined_tree = joined_tree;
    }
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
        let right = reduced_unit.right as f32;
        let left = reduced_unit.left as f32;
        let dif_right_left = right - left;
        let sum_right_left = right + left;
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

impl crate::TreeStruct {
    pub fn make_tree_for_climbing(&mut self,
                            straight_up_dist: f32,
                            straight_down_dist: f32) {
        self.make_joined_tree();
        let mut tree_for_climbing: Vec<TreeElForClimbing> = vec![];
        let mut start_x = self.x_start;
        let mut start_y = self.y_start;
        let mut dist_from_root_start = 0.0;
        let mut angle_start = 0.0;
        for reduced_unit in &self.joined_tree {
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
        self.tree_for_climbing = tree_for_climbing;
    }

    pub fn monkey_on_animation_frame(&mut self, deltat: f32, v: f32) {
        let mut new_height;
        if !self.monkey_climbing.on_goal {
            if self.monkey_climbing.goal_height > self.monkey_climbing.total_height {
                new_height = self.monkey_climbing.total_height + deltat * v;
                if new_height >= self.monkey_climbing.goal_height {
                    new_height = self.monkey_climbing.goal_height;
                    self.monkey_climbing.on_goal = true;
                }
            }
            else {
                new_height = self.monkey_climbing.total_height - deltat * v;
                if new_height <= self.monkey_climbing.goal_height {
                    new_height = self.monkey_climbing.goal_height;
                    self.monkey_climbing.on_goal = true;
                }
            }
            self.monkey_climbing.total_height = new_height;
            self.update_monkey_based_on_height()
        }
    }

    pub fn set_monkey_height(&mut self, height: f32) {
        self.monkey_climbing.total_height = height;
        self.update_monkey_based_on_height();
    }

    pub fn get_monkey_height(&self) -> f32 {
        let h0 = self.monkey_climbing.total_height;
        let h_head = h0 + DELTAY_FRONT + 5.0;
        let monkey_segment = self.monkey_climbing.now_segment;
        let h_arm = h0 + self.tree_for_climbing[monkey_segment].arm_dist;
        if h_head > h_arm {h_head} else {h_arm}
    }
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

impl crate::TreeStruct {
    fn get_segment(&self,
                   dist_from_root: f32,
                   segment_num: usize) -> Option<usize> {
        let mut segment_num = segment_num;
        let tree = &self.tree_for_climbing;
        let last = tree.last().unwrap();
        let max_dist_from_root = last.dist_from_root_start + last.length;
        if dist_from_root > max_dist_from_root {
            return None;
        }
        let tree_len = tree.len();
        let last  = tree.last().unwrap();
        loop {
            let now_segment = tree.get(segment_num);
            let segment_unwrapped;
            match now_segment {
                None => {
                    segment_unwrapped = last;
                    segment_num = tree_len - 1;
                }
                Some(tree_el) => {
                    segment_unwrapped = tree_el;
                }
            }
            let start_dist = segment_unwrapped.dist_from_root_start;
            let max_dist = start_dist + segment_unwrapped.length;
            if dist_from_root >= start_dist &&
            dist_from_root <= max_dist {
                return Some(segment_num);
            } else {
                if dist_from_root < start_dist {
                    segment_num -= 1;
                } else {
                
                    segment_num += 1;
                }
            }
        }
    }


    fn get_segment_x_y_dist_from_root_body(&self,
                    dist_from_root: f32,
                    segment_num: usize) -> (usize, f32, f32, f32) {
        let mut segment_num = segment_num;
        let tree = &self.tree_for_climbing;
        let tree_len = tree.len();
        if segment_num >= tree_len {
            segment_num = tree_len - 1;
        }
        let res_segment;
        let mut actual_dist_from_root = dist_from_root;
        if dist_from_root < MIN_DIST_FROM_ROOT {
            res_segment = tree.first().unwrap();
            actual_dist_from_root = MIN_DIST_FROM_ROOT;
            segment_num = 0;
        } else {
            let segment_num_or_none = self.get_segment(actual_dist_from_root, segment_num);
            match segment_num_or_none {
                None => { // current pos is higher than total tree length
                    res_segment = tree.last().unwrap();
                    actual_dist_from_root = res_segment.dist_from_root_start + res_segment.length;
                    segment_num = tree_len - 1;
                }
                Some(number) => {
                    segment_num = number;
                    res_segment = tree.get(number).unwrap();
                }
            }
        }
        let (x, y) = get_segment_x_y_after_verif(actual_dist_from_root, res_segment);
        (segment_num, x, y, actual_dist_from_root)
    }
}

impl MonkeyClimbing {
    pub fn new() -> MonkeyClimbing {
        MonkeyClimbing{
            total_height: 0.0,
            now_segment: 0,
            body_pos: crate::Pos::new(),
            left_arm_right_arm_left_leg_right_leg: [crate::Pos::new(), crate::Pos::new(), crate::Pos::new(), crate::Pos::new()],
            goal_height: 0.0,
            on_goal: false,
            vertex_arr: [0.0; 60],
            texture_arr: [261.0 / 1024.0; 60],
        }
    }
    pub fn set_goal(&mut self, goal: f32) {
        self.goal_height = goal;
        self.on_goal = false;
    }

    pub fn refresh_arrays_climbing(&mut self) {

        let mut vertex_vec: Vec<f32> = vec![];
        for limb in &self.left_arm_right_arm_left_leg_right_leg {
            let x_left = (limb.x - 2.0) * 2.0 / 600.0 - 1.0;
            let x_right = (limb.x + 2.0) * 2.0 / 600.0 - 1.0;
            let y_top = (limb.y + 2.0) * 2.0 / 600.0 - 1.0;
            let y_bottom = (limb.y - 2.0) * 2.0 / 600.0 - 1.0; 
            let arr = [x_left,  y_bottom,
                           x_right, y_bottom,
                           x_left,  y_top,
                           x_right, y_bottom,
                           x_right, y_top,
                           x_left,  y_top,];
            vertex_vec.extend(arr.iter());
        }
        let x_left = (self.body_pos.x - 2.0) * 2.0 / 600.0 - 1.0; 
        let x_right = (self.body_pos.x + 2.0) * 2.0 / 600.0 - 1.0; 
        let y_top = (self.body_pos.y + 2.0) * 2.0 / 600.0 - 1.0; 
        let y_bottom = (self.body_pos.y - 2.0) * 2.0 / 600.0 - 1.0; 
        let arr = [x_left,  y_bottom,
                        x_right, y_bottom,
                        x_left,  y_top,
                        x_right, y_bottom,
                        x_right, y_top,
                        x_left,  y_top,];
        vertex_vec.extend(arr.iter());
        self.vertex_arr = vertex_vec.try_into().unwrap();
    }
}


impl crate::TreeStruct {
    fn go_to_top(&mut self) {
        let tree_for_climbing = &self.tree_for_climbing;
        let monkey_climbing = &mut self.monkey_climbing; 
        let last = tree_for_climbing.last().unwrap();
        monkey_climbing.total_height = last.dist_from_root_start + last.length;
        self.update_monkey_based_on_height();
    }

    fn get_cand_pre_pos(&self, ind: usize) -> Option<(&LimbsPos, &LimbsPos)> {
        let pos1 = self.limbs_vec.get(ind);
        let pos2 = self.limbs_vec.get(ind + 1);
        match pos1 {
            Some(po1) => {
                match pos2 {
                    Some(po2) => {return Some((po1, po2))}
                    None => {return Some((po1, po1))}
                }
            }
            None => {
                match pos2 {
                    Some(po2) => {return Some((po2, po2))}
                    None => {return None}
                }
            }
        }
    }
    
    fn update_monkey_based_on_height(&mut self) {
        let dist_from_root = self.monkey_climbing.total_height;
        let segment_num = self.monkey_climbing.now_segment;
        let (actual_segment_num,
             x,
             y,
             actual_dist_from_root) = self.get_segment_x_y_dist_from_root_body(dist_from_root, segment_num);
        self.monkey_climbing.total_height = actual_dist_from_root;
        let body_pos = crate::Pos{x, y};
        self.monkey_climbing.body_pos = body_pos;
        self.monkey_climbing.now_segment = actual_segment_num;
        let mut found = false;
        let mut ind = 0;
        while !found {
            let res = self.get_cand_pre_pos(ind);
            match res {
                None => {
                    found = true;
                    ind = if ind == 0 {0} else {ind - 1};
                }
                Some(re) => {
                    let (pos1, pos2) = re;
                    if self.monkey_climbing.total_height > pos2.center_pos {
                        ind += 1;
                    } else {
                        found = true;
                    }
                }
            }
        }
        let (pos1, pos2) = self.get_cand_pre_pos(ind).unwrap();
        let body_height = self.monkey_climbing.total_height;
        let center1 = pos1.center_pos;
        let center2 = pos2.center_pos;
        let coef;
        let dif = center2 - center1;
        if dif.abs() < 0.000000001 {
            coef = 0.5
        } else {
            coef = (body_height - center1) / dif
        }
        let arr1 = &pos1.left_arm_right_arm_left_leg_right_leg;
        let arr2 = &pos2.left_arm_right_arm_left_leg_right_leg;
        let mut left_arm_right_arm_left_leg_right_leg: Vec<crate::Pos> = vec![];
        for it in arr1.iter().zip(arr2.iter()) {
            let (p1, p2) = it;
            let x = (p2.x - p1.x) * coef + p1.x;
            let y = (p2.y - p1.y) * coef + p1.y;
            left_arm_right_arm_left_leg_right_leg.push(crate::Pos{x, y});
        }
        self.monkey_climbing.left_arm_right_arm_left_leg_right_leg = left_arm_right_arm_left_leg_right_leg.try_into().unwrap();
        self.monkey_climbing.refresh_arrays_climbing()
    }

}

impl crate::Monkey {
    pub fn update_if_segment_disappears(&mut self,
                                       tree_structs: &mut Vec<crate::TreeStruct>) {
        // Do this after tree undo
        match &self.monkey_state {
            crate::MonkeyState::Running => {}
            crate::MonkeyState::Climbing => {
                let tree_struct = tree_structs.get_mut(self.climbing_num).unwrap();
                let tree_for_climbing = &tree_struct.tree_for_climbing;
                let climbing = &tree_struct.monkey_climbing;
                let current_segment = tree_for_climbing.get(climbing.now_segment);
                match &current_segment {
                    None => {
                        if tree_for_climbing.is_empty() {
                            self.monkey_state = crate::MonkeyState::Running;
                            self.running.set_on_pos(tree_struct.x_start, tree_struct.y_start as f32);
                        } else {
                            tree_struct.go_to_top();
                        }
                    }
                    Some(_) => {}
                }
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

pub struct LimbsPos {
    left_arm_right_arm_left_leg_right_leg: [crate::Pos; 4],
    center_pos:f32, // if center is between center_pos1 and center_pos2,
                    // leg_end will be interpolated between leg_pos1 and leg_pos2.
                    // center_pos is distance from root
}

fn get_limb_pos_x_y_segment_known(dist_from_root: f32,
                             segment: &TreeElForClimbing,
                             is_left: bool) -> crate::Pos {
    let dist_from_start_segment = dist_from_root - segment.dist_from_root_start;
    let multiplier = if is_left {-1.0} else {1.0};
    match &segment.for_not_linear {
        None => {
            let center_x = segment.start_x - dist_from_start_segment * segment.angle_start.sin();
            let center_y = segment.start_y + dist_from_start_segment * segment.angle_start.cos();
            let pos_x = center_x + W / 2.0 * multiplier * segment.angle_start.cos();
            let pos_y = center_y + W / 2.0 * multiplier * segment.angle_start.sin();
            crate::Pos{x: pos_x, y: pos_y}
        }
        Some(el) => {
            let now_radius = el.radius + W / 2.0 * multiplier;
            let now_angle = segment.angle_start + dist_from_start_segment / segment.length * (el.angle_stop - segment.angle_start);
            let pos_x = el.center_x + now_radius * now_angle.cos();
            let pos_y = el.center_y + now_radius * now_angle.sin();
            crate::Pos{x: pos_x, y: pos_y}
        }
    }
}

impl crate::TreeStruct {
    fn get_limb_pos_x_y(&self,
                dist_from_root: f32,
                current_segment: usize,
                is_left: bool) -> crate::Pos {
        let tree = &self.tree_for_climbing;
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
                        let dist_end = current_segment_obj.dist_from_root_start + current_segment_obj.length;
                        if dist_from_root <= dist_end {
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
                    if dist_from_root >= current_segment_obj.dist_from_root_start {
                        return get_limb_pos_x_y_segment_known(dist_from_root,
                                    current_segment_obj,
                                    is_left);
                    }
                }
            }
        }
        return crate::Pos{x: -1.0, y: -1.0}; // must not get here
    }
}    

#[derive(Clone)]
struct ArmLegCharact {
    a: f32, // arm distance
    l: f32, // leg distance
    step: f32,
}

struct MonkeyClimbingNodePoint {
    is_right_up: bool,
    down_arm_leg: ArmLegCharact,
    up_arm_leg: ArmLegCharact,
    center_pos: f32,
    down_leg_pos: f32,
    up_leg_pos: f32,
    down_arm_pos: f32,
    up_arm_pos: f32,
    is_final: bool,
    center_segment_num: usize,
}

fn make_in_bounds(input: f32, lower: f32, upper: f32) -> f32 {
    if input < lower {lower} else {if input > upper {upper} else {input}}
}

impl MonkeyClimbingNodePoint {
    fn new(prev: &MonkeyClimbingNodePoint,
           new_arm_leg_charact: ArmLegCharact,
           tree_struct: &crate::TreeStruct) -> MonkeyClimbingNodePoint {
        let tree = &tree_struct.tree_for_climbing;
        let is_right_up = !prev.is_right_up;
        let known_arm_pos = prev.up_arm_pos;
        let known_leg_pos = prev.up_leg_pos;
        let step1 = prev.up_arm_leg.step;
        let l1 = prev.up_arm_leg.l;
        let a1 = prev.up_arm_leg.a;
        let step2 = new_arm_leg_charact.step;
        let a2 = new_arm_leg_charact.a;
        let l2 = new_arm_leg_charact.l;
        let new_arm_pos;
        let new_leg_pos;
        let new_center_pos;
        if step2 <= step1 {
            new_arm_pos = known_arm_pos + step2;
            new_leg_pos = known_arm_pos - (l2 + a2 - step2 * 2.0);
            new_center_pos = known_leg_pos + l1;
        } else {
            new_leg_pos = known_arm_pos - (l1 + a1 - step1 * 2.0);
            new_arm_pos = new_leg_pos + (l2 + a2 - step2);
            new_center_pos = known_leg_pos + step1 + l2 - step2;
        }
        let last = tree.last().unwrap();
        let max_pos = last.dist_from_root_start + last.length;
        let is_final = new_center_pos >= max_pos; 
        let center_pos = make_in_bounds(new_center_pos, 0.0, max_pos);
        MonkeyClimbingNodePoint {
            is_right_up,
            down_arm_leg: prev.up_arm_leg.clone(),
            up_arm_leg: new_arm_leg_charact,
            center_pos,
            down_leg_pos: make_in_bounds(known_leg_pos, 0.0, max_pos),
            down_arm_pos: make_in_bounds(known_arm_pos, 0.0, max_pos),
            up_leg_pos: make_in_bounds(new_leg_pos, 0.0, max_pos),
            up_arm_pos: make_in_bounds(new_arm_pos, 0.0, max_pos),
            is_final,
            center_segment_num: tree_struct.get_segment(center_pos, prev.center_segment_num).unwrap(),
        }
    }

    fn get_underlying_steps(&self, tree_struct: &crate::TreeStruct) -> Vec<ArmLegCharact> {
        let tree = &tree_struct.tree_for_climbing;
        let pos1 = self.down_leg_pos;
        let segment1_num = tree_struct.get_segment(pos1, self.center_segment_num).unwrap();
        let pos2 = self.up_arm_pos;
        let segment2_num = tree_struct.get_segment(pos2, self.center_segment_num).unwrap();
        let mut steps: Vec<ArmLegCharact> = vec![];
        for el_step in  tree[segment1_num..=segment2_num].iter().map(
            |el| ArmLegCharact{a: el.arm_dist,
                                                   l: el.leg_dist,
                                                   step: el.step}) {
            steps.push(el_step);
        }
        steps.sort_by(|a, b| b.step.total_cmp(&a.step)); //descending order
        steps
    }

    fn generate_first_el(tree_struct: &crate::TreeStruct) -> MonkeyClimbingNodePoint {
        let tree = &tree_struct.tree_for_climbing;
        let is_right_up = true; // arbitrary choice
        let center_pos = MIN_DIST_FROM_ROOT;
        let up_arm_pos = tree_struct.get_first_right_arm_dist(center_pos);
        let a = tree[0].arm_dist;
        let l = tree[0].leg_dist;
        let step = tree[0].step;
        let down_arm_leg = ArmLegCharact {a, l, step};
        let up_arm_leg = ArmLegCharact {a, l, step};
        let tree_last = tree.last().unwrap();
        let tree_length = tree_last.dist_from_root_start + tree_last.length;
        let down_arm_pos = make_in_bounds(up_arm_pos - step, 0.0, tree_length);
        let down_leg_pos = make_in_bounds(center_pos - l, 0.0, tree_length);
        let up_leg_pos = make_in_bounds(center_pos - l + step, 0.0, tree_length);
        let center_segment_num = tree_struct.get_segment(center_pos, 0).unwrap();
        let is_final = center_pos >= tree_length;
        MonkeyClimbingNodePoint {
            is_right_up,
            down_arm_leg,
            up_arm_leg,
            center_pos,
            down_leg_pos,
            up_leg_pos,
            down_arm_pos,
            up_arm_pos,
            is_final,
            center_segment_num,
         }
    }
}

impl TreeElForClimbing {
    fn is_onclick(&self, x: f32, y: f32) -> Option<f32> {
        match &self.for_not_linear {
            Some(not_linear) => {
                let dif_x = x - not_linear.center_x;
                let dif_y = y - not_linear.center_y;
                let r = (dif_x * dif_x + dif_y * dif_y).sqrt();
                if (not_linear.radius.abs() - r).abs() > W / 2.0 {
                    return None;
                }
                let mut angle = dif_y.atan2(dif_x);
                if not_linear.radius < 0.0 {
                    angle = angle - PI;
                }
                let converted_alpha1 = (self.angle_start - angle) / (2.0 * PI);
                let converted_alpha2 = (not_linear.angle_stop - angle) / (2.0 * PI);
                let candidate;
                if not_linear.angle_stop > self.angle_start {
                    candidate = converted_alpha2.floor();
                    if candidate < converted_alpha1 {
                        return None
                    }
                } else {
                    candidate = converted_alpha2.ceil();
                    if candidate > converted_alpha1 {
                        return None
                    }
                }
                return Some(self.dist_from_root_start +
                            self.length * (candidate - converted_alpha1) / (converted_alpha2 - converted_alpha1))
            },
            None => {
                let x_conv = x - self.start_x;
                let y_conv = y - self.start_y;
                // 2D rotation matrix
                let alp = self.angle_start;
                let a = alp.cos();
                let b = alp.sin();
                let c = -b;
                let d = a;
                let x_rotated = a * x_conv + b * y_conv;
                let y_rotated = c * x_conv + d * y_conv;

                if x_rotated.abs() <= W / 2.0 &&
                    y_rotated >= 0.0 &&
                    y_rotated <= self.length {
                        return Some(self.dist_from_root_start + y_rotated); // pos, dist from root
                    }
                return None;
            }
        }
    }
}

impl crate::TreeStruct {
    fn try_a(&self, a: f32, dist: f32) -> bool {
        let tree = &self.tree_for_climbing;
        for el in tree {
            if el.dist_from_root_start >= dist {
                return true;
            }
            if el.arm_dist < a {
                return false;
            }
        }
        return true;
    }

    fn get_first_right_arm_dist(&self, center_pos: f32) -> f32 {
        let tree = &self.tree_for_climbing;
        let first_segment = tree.first().unwrap();
        let a = first_segment.arm_dist;
        let try_right_arm = center_pos + a;
        let right_arm_segment = self.get_segment(try_right_arm, 0);
        let max_dist;
        let last_el_num;
        match right_arm_segment {
            None => {
                let last = tree.last().unwrap();
                max_dist = last.dist_from_root_start + last.length;
                last_el_num = tree.len() - 1;
            }
            Some(right_arm_segment_num) => {
                max_dist = try_right_arm;
                last_el_num = right_arm_segment_num;
            }
        }
        let mut a_s: Vec<f32> = vec![max_dist - center_pos];
        for el_a in  tree[..=last_el_num].iter().map(|el| el.arm_dist) {
            if center_pos + el_a < max_dist {
                a_s.push(el_a);
            }
        }
        a_s.sort_by(|a, b| b.total_cmp(a)); //descending order
        for el_a in a_s {
            let dist = center_pos + el_a;
            if self.try_a(el_a, dist) {
                return dist;
            }
        }
        tree.first().unwrap().length
    }

    fn generate_poses_vector(&self) -> Vec<MonkeyClimbingNodePoint> {
        let tree = &self.tree_for_climbing;
        let first_pos = MonkeyClimbingNodePoint::generate_first_el(&self);
        let mut is_final = first_pos.is_final;
        let mut poses: Vec<MonkeyClimbingNodePoint> = vec![first_pos];
        while !is_final {
            let last = poses.last().unwrap();
            let guess_segment = self.get_segment(last.up_arm_pos, last.center_segment_num);
            let segment_num = match guess_segment {None => {tree.len() - 1}, Some(num) => {num}};
            let segment = tree.get(segment_num).unwrap();
            let new_arm_leg_charact = ArmLegCharact{a: segment.arm_dist,
                                                                l: segment.leg_dist,
                                                                step: segment.step};
            let next_segment = MonkeyClimbingNodePoint::new(&last,
                                                                                    new_arm_leg_charact,
                                                                                    &self);
            let underlying_steps = next_segment.get_underlying_steps(&self); // descending order
            let underlying_min_el = underlying_steps.last().unwrap();
            let min_a = underlying_min_el.a / 2.0;
            let min_l = underlying_min_el.l / 2.0;
            let min_step = (min_a + min_l + (min_a + min_l) / STEP_RATIO) / 2.0;
            let min_charact = ArmLegCharact {
                a: min_a,
                l: min_l,
                step: min_step,
            };
            let mut segment_to_add = MonkeyClimbingNodePoint::new(&last,
                                                        min_charact,
                                                        &self); // fallback values, should not be used
            for charact in underlying_steps {
                let step = charact.step;
                let new_next_segment = MonkeyClimbingNodePoint::new(&last,
                                                            charact,
                                                            &self);
                let new_underlying_steps = new_next_segment.get_underlying_steps(&self);
                let min_values = new_underlying_steps.last().unwrap();
                if min_values.step >= step {
                    segment_to_add = new_next_segment;
                    break;
                } 
            }
            is_final = segment_to_add.is_final;
            poses.push(segment_to_add);
        }
        poses
    }

    pub fn generate_arms_legs_vectors(&mut self) {
        let poses = self.generate_poses_vector();
        let mut limbs_vec: Vec<LimbsPos> = vec![];
        for pos in poses {
            let center_pos = pos.center_pos;
            let left_arm_pos;
            let right_arm_pos;
            let left_leg_pos;
            let right_leg_pos;
            if pos.is_right_up {
                right_arm_pos = pos.up_arm_pos;
                left_arm_pos = pos.down_arm_pos;
                right_leg_pos = pos.down_leg_pos;
                left_leg_pos = pos.up_leg_pos;
            } else {
                left_arm_pos = pos.up_arm_pos;
                right_arm_pos = pos.down_arm_pos;
                left_leg_pos = pos.down_leg_pos;
                right_leg_pos = pos.up_leg_pos;
            }
            let left_arm_coords = self.get_limb_pos_x_y(left_arm_pos,
                                                        pos.center_segment_num,
                                                        true);
            let right_arm_coords = self.get_limb_pos_x_y(right_arm_pos,
                                                        pos.center_segment_num,
                                                        false);
            let left_leg_coords = self.get_limb_pos_x_y(left_leg_pos,
                                                        pos.center_segment_num,
                                                        true);
            let right_leg_coords = self.get_limb_pos_x_y(right_leg_pos,
                                                        pos.center_segment_num,
                                                        false);
            let left_arm_right_arm_left_leg_right_leg = [left_arm_coords,
                                                                   right_arm_coords,
                                                                   left_leg_coords,
                                                                   right_leg_coords];
            limbs_vec.push(LimbsPos { left_arm_right_arm_left_leg_right_leg, center_pos });
        }
        self.limbs_vec = limbs_vec;
    }

    pub fn get_dest_on_click(&self, x:f32, y: f32) -> Option<f32> {
        for el in self.tree_for_climbing.iter().rev() {
            match el.is_onclick(x, y) {
                Some(val) => {
                    let res = if val > MIN_DIST_FROM_ROOT {val} else {MIN_DIST_FROM_ROOT};
                    return Some(res)
                }
                None => {}
            }
        }
        return None;
    }
}