use std::f32::MIN;

use crate::START_X;

pub const DELTAX_FRONT: f32 = 3.0; //px
pub const DELTAX_BACK: f32 = 3.0; //px
pub const DELTAY_FRONT: f32 = 9.0; //px
pub const DELTAY_BACK: f32 = 9.0; //px
pub const LEG_SEGMENT_LENGTH: f32 = 10.0; //px
pub const ARM_SEGMENT_LENGTH: f32 = 9.0; //px
pub const ARM_WIDTH_START: f32 = 4.0; //px
pub const ARM_WIDTH_MID: f32 = 3.0; //px
pub const ARM_WIDTH_END: f32 = 2.0; //px
pub const LEG_WIDTH_START: f32 = 5.0; //px
pub const LEG_WIDTH_MID: f32 = 3.25; //px
pub const LEG_WIDTH_END: f32 = 2.5; //px
pub const MIN_DIST_FROM_ROOT: f32 = 5.0; //px, must be less then W * PI / 6 / 2
const TAIL_PERIOD: f32 = 2.0; //seconds
const TAIL_ROWS: usize = 10;
const TAIL_COLUMNS: usize = 3;
const TAIL_FRAMES: usize = (TAIL_ROWS * TAIL_COLUMNS) * 4 - 4;
const TAIL_FULLEN: f32 = 40.0; //px
const TAIL_DELTAY: f32 = 5.0; //px
const TAIL_FRAMEWIDTH: f32 = 20.0; //px
const TAIL_FRAMEHEIGHT: f32 = 48.0; //px
const TAIL_DELTAX_START: f32 = 6.0; //px
const TAIL_X_CENTER: f32 = TAIL_FRAMEWIDTH / 2.0; //px
const TAIL_DELTAY_BOTTOM: f32 = TAIL_FRAMEHEIGHT - TAIL_FULLEN - TAIL_DELTAY;
const IMAGE_SIDE: f32 = 1024.0; //px

const PI: f32 = std::f32::consts::PI;
pub const W: f32 = crate::DEST_REF as f32; // three width in px
const CLIMBING_SPEED: f32 = 50.0; // px/s
pub const LEG_EXTENSION_COEFFICIENT: f32 = 1.95; // must be lightly under 2, bigger -> leg more straight
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
    pub vertex_arr: [f32; 168],
    pub texture_arr: [f32; 168],
    pub tail_time: f32,
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

    pub fn get_monkey_max_height(&self) -> f32 {
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
        if dist_from_root < 0.0 {
            res_segment = tree.first().unwrap();
            actual_dist_from_root = 0.0;
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
            vertex_arr: [0.0; 168],
            texture_arr: [261.0 / 1024.0; 168],
            tail_time: 0.0,
        }
    }
    pub fn set_goal(&mut self, goal: f32) {
        self.goal_height = goal;
        self.on_goal = false;
    }

    pub fn convert_vert_arr_to_screen_coords(&mut self, start: usize, stop: usize) {
        for i in start / 2 .. stop / 2 {
            self.vertex_arr[2 * i] = self.vertex_arr[2 * i] * 2.0 / crate::CANVAS_REF_WIDTH - 1.0;
            self.vertex_arr[2 * i + 1] = self.vertex_arr[2 * i + 1] * 2.0 / crate::CANVAS_REF_HEIGHT - 1.0;
        }
    }

    pub fn refresh_arrays_climbing(&mut self) {
        let x_left = self.body_pos.x - 2.0; 
        let x_right = self.body_pos.x + 2.0; 
        let y_top = self.body_pos.y + 2.0; 
        let y_bottom = self.body_pos.y - 2.0; 
        let arr = [x_left,  y_bottom,
                        x_right, y_bottom,
                        x_left,  y_top,
                        x_right, y_bottom,
                        x_right, y_top,
                        x_left,  y_top,];
        self.vertex_arr[144..156].copy_from_slice(&arr);
        self.convert_vert_arr_to_screen_coords(0, 156);
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
             _x,
             _y,
             actual_dist_from_root) = self.get_segment_x_y_dist_from_root_body(dist_from_root, segment_num);
        let adjusted_dist_from_root = if actual_dist_from_root < MIN_DIST_FROM_ROOT {MIN_DIST_FROM_ROOT} else {actual_dist_from_root};
        // I moved this adjustement that distance must be greater then MIN_DIST_FROM_ROOT
        // because this get_segment_x_y function has other uses  
        let (final_actual_segment_num,
             x,
             y,
             _actual_dist_from_root) = self.get_segment_x_y_dist_from_root_body(adjusted_dist_from_root, actual_segment_num);
        self.monkey_climbing.total_height = adjusted_dist_from_root;
        
        let body_pos = crate::Pos{x, y};
        self.monkey_climbing.body_pos = body_pos;
        self.monkey_climbing.now_segment = final_actual_segment_num;
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
                    let (_pos1, pos2) = re;
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
        let are_arm = [true, true, false, false];
        let are_left = [true, false, true, false];
        for i in 0..4 {
            let is_arm = are_arm[i];
            let is_left = are_left[i];
            let start = i * 36;
            self.populate_limb_triangles(i, is_arm, is_left, start);
        }
        self.monkey_climbing.refresh_arrays_climbing()
    }

    fn get_pos_angle_extrapolate(&self, height: f32) -> (crate::Pos, f32) {
        let x;
        let y;
        let angle;
        if height <= 0.0 {
            x = self.x_start;
            y = self.y_start + height;
            angle = 0.0;
        } else {
            let last_segment = self.tree_for_climbing.last().unwrap();
            let tree_total_length = last_segment.dist_from_root_start + last_segment.length;
            if height >= tree_total_length {
                let dif_height = height - tree_total_length;
                match &last_segment.for_not_linear {
                    None => {
                        angle = last_segment.angle_start;
                    }
                    Some(not_linear) => {
                        angle = not_linear.angle_stop;
                    }
                }
                x = last_segment.end_x - dif_height * angle.sin();
                y = last_segment.end_y + dif_height * angle.cos();
            } else {
                let segment_num = self.monkey_climbing.now_segment;
                let actual_segment_num;
                let _actual_dist_from_root;
                (actual_segment_num,
                x,
                y,
                _actual_dist_from_root) = self.get_segment_x_y_dist_from_root_body(height, segment_num);
                let now_segment = &self.tree_for_climbing[actual_segment_num];
                match &now_segment.for_not_linear {
                    None => {
                        angle = now_segment.angle_start;
                    }
                    Some(not_linear) => {
                        let height_dif = height - now_segment.dist_from_root_start;
                        let length = now_segment.length;
                        let angle_start = now_segment.angle_start;
                        let angle_stop = not_linear.angle_stop;
                        angle = angle_start + (angle_stop - angle_start) * height_dif / length;
                    }
                }
            }
        }
        (crate::Pos{x, y}, angle)
    }
    
    fn get_x_y_start_limb(&self, deltay: f32, deltax: f32) -> crate::Pos {
        let height_start = self.monkey_climbing.total_height + deltay;
        let (pos_center, angle) = self.get_pos_angle_extrapolate(height_start);
        let x = pos_center.x + deltax * angle.cos();
        let y = pos_center.y + deltax * angle.sin();
        return crate::Pos{x, y};
    }

    fn populate_limb_triangles(&mut self, limb_num: usize, is_arm: bool, is_left: bool, start: usize) {
        let deltay;
        let deltax;
        let is_clockwise;
        let l;
        let w1;
        let w2;
        let w3;
        if is_arm {
            l = ARM_SEGMENT_LENGTH;
            w1 = ARM_WIDTH_START;
            w2 = ARM_WIDTH_MID;
            w3 = ARM_WIDTH_END;
            deltay = DELTAY_FRONT;
            if is_left {
                deltax = -DELTAX_FRONT;
                is_clockwise = false;
            } else {
                deltax = DELTAX_FRONT;
                is_clockwise = true;
            }
        } else {
            l = LEG_SEGMENT_LENGTH;
            w1 = LEG_WIDTH_START;
            w2 = LEG_WIDTH_MID;
            w3 = LEG_WIDTH_END;
            deltay = -DELTAY_BACK;
            if is_left {
                deltax = -DELTAX_BACK;
                is_clockwise = true;
            } else {
                deltax = DELTAX_BACK;
                is_clockwise = false;
            }
        }
        let limb = &self.monkey_climbing.left_arm_right_arm_left_leg_right_leg[limb_num];
        let pos_start = self.get_x_y_start_limb(deltay, deltax);
        let pos_ancle = get_ancle(&pos_start,
                                       limb,
                                       l,
                                       is_clockwise);
        //    5____________6 
        //  4 _\   _____---| end
        // 3____8--________7
        // |   /|    /|\
        // |  / |     / //is_clockwise here is false
        // | /  |____/
        // |/   |
        // 2____1
        // start

        let m = if is_clockwise {1.0} else {-1.0};
        let deltax1 = pos_ancle.x - pos_start.x;
        let deltay1 = pos_ancle.y - pos_start.y;
        let ratio1 = w1 * 0.5 / l;
        let x1 = pos_start.x - deltay1 * ratio1 * m;
        let y1 = pos_start.y + deltax1 * ratio1 * m;
        let x2 = pos_start.x + deltay1 * ratio1 * m;
        let y2 = pos_start.y - deltax1 * ratio1 * m;
        let ratio2 = w2 * 0.5 / l;
        let x3 = pos_ancle.x + deltay1 * ratio2 * m;
        let y3 = pos_ancle.y - deltax1 * ratio2 * m;
        let deltax2 = limb.x - pos_ancle.x;
        let deltay2 = limb.y - pos_ancle.y;
        let x5 = pos_ancle.x + deltay2 * ratio2 * m;
        let y5 = pos_ancle.y - deltax2 * ratio2 * m;
        let ratio3 = w3 * 0.5 / l;
        let x6 = limb.x + deltay2 * ratio3 * m;
        let y6 = limb.y - deltax2 * ratio3 * m;
        let x7 = limb.x - deltay2 * ratio3 * m;
        let y7 = limb.y + deltax2 * ratio3 * m;
        // we need ghost points to calculate intersection
        let g3x = pos_ancle.x - deltay1 * ratio2 * m;
        let g3y = pos_ancle.y + deltax1 * ratio2 * m;
        let g5x = pos_ancle.x - deltay2 * ratio2 * m;
        let g5y = pos_ancle.y + deltax2 * ratio2 * m;
        let pos1 = crate::Pos{x:x1, y:y1};
        let pos2 = crate::Pos{x:g3x, y:g3y};
        let pos3 = crate::Pos{x:g5x, y:g5y};
        let pos4 = crate::Pos{x:x7, y:y7};
        let pos8 = lines_intersection(pos1, pos2, pos3, pos4);
        let x8 = pos8.x;
        let y8 = pos8.y;
        
        let x4_center = (x3 + x5) * 0.5;
        let y4_center = (y3 + y5) * 0.5;
        let deltax_4 = x4_center - pos_ancle.x;
        let deltay_4 = y4_center - pos_ancle.y;
        let delta_4_len = (deltax_4 * deltax_4 + deltay_4 * deltay_4).sqrt() + 0.000000001;
        let ratio = w2 * 0.5 / delta_4_len;
        let deltax_4_actual = deltax_4 * ratio;
        let deltay_4_actual = deltay_4 * ratio;
        let x4 = pos_ancle.x + deltax_4_actual;
        let y4 = pos_ancle.y + deltay_4_actual;
        let triangles = [
            x8, y8, x2, y2, x1, y1,
            x8, y8, x3, y3, x2, y2,
            x8, y8, x4, y4, x3, y3,
            x8, y8, x5, y5, x4, y4,
            x8, y8, x6, y6, x5, y5,
            x8, y8, x7, y7, x6, y6
        ];
        self.monkey_climbing.vertex_arr[start..start + 36].copy_from_slice(&triangles);
    }

    pub fn update_tail(&mut self, deltat: f32) { // deltat in seconds
        self.monkey_climbing.tail_time = (self.monkey_climbing.tail_time + deltat) % TAIL_PERIOD;
        let tail_i = (self.monkey_climbing.tail_time * (TAIL_FRAMES as f32) / TAIL_PERIOD).floor() as usize;
        let (row,
             col,
             flip_horizontal,
             flip_vertical) = get_ninframes_row_col_fliph_flipv(tail_i);
        let angle_start = (tail_i as f32) * PI * 2.0 / (TAIL_FRAMES as f32);
        let deltax_from_center = angle_start.cos() * TAIL_DELTAX_START;
        let (o1x, o2x, o1y, o2y) = get_points_original(row, col);
        let x1;
        let x2;
        let y1;
        let y2;
        let deltax_from_start_to_p1 = TAIL_X_CENTER + deltax_from_center;
        let deltay_from_start_to_p1;
        if flip_horizontal {
            x1 = o2x / IMAGE_SIDE;
            x2 = o1x / IMAGE_SIDE;
        } else {
            x1 = o1x / IMAGE_SIDE;
            x2 = o2x / IMAGE_SIDE;
        }
        if flip_vertical {
            y1 = o2y / IMAGE_SIDE;
            y2 = o1y / IMAGE_SIDE;
            deltay_from_start_to_p1 = TAIL_DELTAY_BOTTOM;
        } else {
            y1 = o1y / IMAGE_SIDE;
            y2 = o2y / IMAGE_SIDE;
            deltay_from_start_to_p1 = TAIL_DELTAY;
        }
        let uv_triangles = [
            x1, y1,
            x2, y1,
            x2, y2,
            x1, y1,
            x1, y2,
            x2, y2];
        self.monkey_climbing.texture_arr[156..168].copy_from_slice(&uv_triangles);
        let tail_start_h = self.monkey_climbing.total_height - DELTAY_BACK - LEG_WIDTH_START * 0.5;
        let (pos_tail_start, angle) = self.get_pos_angle_extrapolate(tail_start_h);
        let deltax_render = deltax_from_start_to_p1 * crate::MONKEY_SCALING;
        let deltay_render = deltay_from_start_to_p1 * crate::MONKEY_SCALING;
        // both these deltax and deltay are positive
        // P1-----P2
        // |start |
        // |      |
        // |      |
        // |      |
        // P3-----P4
        let p1_x = pos_tail_start.x - deltax_render * angle.cos() - deltay_render * angle.sin();
        let p1_y = pos_tail_start.y - deltax_render * angle.sin() + deltay_render * angle.cos();
        let width_render = TAIL_FRAMEWIDTH * crate::MONKEY_SCALING;
        let height_render = TAIL_FRAMEHEIGHT * crate::MONKEY_SCALING;
        let p2_x = p1_x + width_render * angle.cos();
        let p2_y = p1_y + width_render * angle.sin();
        let p3_x = p1_x + height_render * angle.sin();
        let p3_y = p1_y - height_render * angle.cos();
        let p4_x = p3_x + width_render * angle.cos();
        let p4_y = p3_y + width_render * angle.sin();
        let pos_triangles = [
            p1_x, p1_y,
            p2_x, p2_y,
            p4_x, p4_y,
            p1_x, p1_y,
            p3_x, p3_y,
            p4_x, p4_y
        ];
        self.monkey_climbing.vertex_arr[156..168].copy_from_slice(&pos_triangles);
        self.monkey_climbing.convert_vert_arr_to_screen_coords(156, 168);
    }
}

fn get_points_original(row: usize, col: usize) -> (f32, f32, f32, f32) {
    let p0x = IMAGE_SIDE - (TAIL_COLUMNS as f32) * TAIL_FRAMEWIDTH;
    let p0y = 0.0;
    let o1x = p0x + TAIL_FRAMEWIDTH * (col as f32);
    let o2x = p0x + TAIL_FRAMEWIDTH * ((col + 1) as f32);
    let o1y = p0y + TAIL_FRAMEHEIGHT * (row as f32);
    let o2y = p0y + TAIL_FRAMEHEIGHT * ((row + 1) as f32);
    (o1x, o2x, o1y, o2y)
}

fn get_ninframes_row_col_fliph_flipv(tail_i: usize) -> (usize, usize, bool, bool) {
    let n_frames_in_serie = TAIL_COLUMNS * TAIL_ROWS - 1;
    let quart = tail_i / n_frames_in_serie;
    let flip_horizontal;
    let flip_vertical;
    let n_in_serie = tail_i % n_frames_in_serie;
    let n_in_frames;
    if quart == 0 {
        flip_horizontal = false;
        flip_vertical = false;
        n_in_frames = n_frames_in_serie - n_in_serie;
    } else {
        if quart == 1 {
            flip_horizontal = false;
            flip_vertical = true;
            n_in_frames = n_in_serie;
        } else {
            if quart == 2 {
                flip_horizontal = true;
                flip_vertical = false;
                n_in_frames = n_frames_in_serie - n_in_serie;
            } else { // quart == 3
                flip_horizontal = true;
                flip_vertical = true;
                n_in_frames = n_in_serie;
            }
        }
    }
    let col = n_in_frames / TAIL_ROWS;
    let row = n_in_frames % TAIL_ROWS;
    (row, col, flip_horizontal, flip_vertical)
}

fn lines_intersection(pos1: crate::Pos, pos2: crate::Pos, pos3: crate::Pos, pos4: crate::Pos) -> crate::Pos{
    let deltax1 = pos2.x - pos1.x;
    let deltay1 = pos2.y - pos1.y;
    let deltax2 = pos4.x - pos3.x;
    let deltay2 = pos4.y - pos3.y;
    let mult1 = deltax2 * deltay1;
    let mult2 = deltay2 * deltax1;
    let x;
    let y;
    if mult2 != mult1 {
        y = ((pos3.x - pos1.x) * deltay2 * deltay1 - pos3.y * mult1 + pos1.y * mult2) / (mult2 - mult1);
        x = ((pos3.y - pos1.y) * deltax2 * deltax1 - pos3.x * mult2 + pos1.x * mult1) / (mult1 - mult2);
    } else { // fallback if they are parallel
        x = (pos1.x + pos4.x) * 0.5;
        y = (pos1.y + pos4.y) * 0.5;
    }
    crate::Pos{x, y}
}

fn get_ancle(pos1: &crate::Pos, pos2: &crate::Pos, l: f32, is_clockwise: bool) -> crate::Pos {
    let deltax = pos2.x - pos1.x;
    let deltay = pos2.y - pos1.y;
    let center_x = (pos1.x + pos2.x) / 2.0;
    let center_y = (pos1.y + pos2.y) / 2.0;
    let dist_squared = deltax * deltax + deltay * deltay;
    let h_dist_squared = l * l - dist_squared / 4.0;
    let h_dist = if h_dist_squared > 0.0 {h_dist_squared.sqrt()} else {0.0};
    let dist = dist_squared.sqrt();
    let m = if is_clockwise {1.0} else {-1.0};
    let ratio = h_dist / dist;
    let x = center_x + m * deltay * ratio;
    let y = center_y - m * deltax * ratio;
    crate::Pos{x, y}
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