const DELTAX_FRONT: f32 = 3.0; //px
const DELTAX_BACK: f32 = 3.0; //px
const DELTAY_FRONT: f32 = 11.0; //px
const DELTAY_BACK: f32 = 11.0; //px
const LEG_SEGMENT_LENGTH: f32 = 10.0; //px
const ARM_SEGMENT_LENGTH: f32 = 9.0; //px
const PI: f32 = std::f32::consts::PI;
const W: f32 = crate::DEST_REF as f32; // three width in px

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

struct TreeElForClimbing {
    angle_start: f32,
    dist_from_root_start: f32,
    length: f32,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    is_linear: bool,
    right: u32,
    left: u32,
    for_not_linear: Option<TreeElForClimbingNotLinear>,
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
                               dist_from_root_start: f32,) -> (TreeElForClimbing, f32, f32, f32, f32) {
    let for_not_linear;
    let is_linear;
    let length;
    let end_x;
    let end_y;
    let angle_stop;
    if reduced_unit.right != reduced_unit.left {
        is_linear = false;
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
        for_not_linear = Some(TreeElForClimbingNotLinear {
                                                    center_x,
                                                    center_y,
                                                    radius,
                                                    angle_stop,
        });
    } else {
        is_linear = true;
        for_not_linear = None;
        length = W * PI / 6.0 * (reduced_unit.right as f32);
        end_x = start_x - length * angle_start.sin();
        end_y = start_y + length * angle_start.cos();
        angle_stop = angle_start;
    }
    (TreeElForClimbing {
        angle_start,
        dist_from_root_start,
        length,
        start_x,
        start_y,
        end_x,
        end_y,
        is_linear,
        right: reduced_unit.right,
        left: reduced_unit.left,
        for_not_linear,
    }, end_x, end_y, dist_from_root_start + length, angle_stop)
}

fn make_tree_for_climbing(tree: &Vec<crate::TreeUnit>,
                          start_x: f32,
                          start_y: f32) -> Vec<TreeElForClimbing>{
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
             angle_start_new) = make_tree_for_climbing_unit(reduced_unit, start_x, start_y, angle_start, dist_from_root_start);
        start_x = start_x_new;
        start_y = start_y_new;
        angle_start = angle_start_new;
        dist_from_root_start = dist_from_root_new;
        tree_for_climbing.push(for_climbing_unit);
    }
    tree_for_climbing
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
    fn get_body_x_y_and_update_segment(&mut self, tree_for_climbing: &Vec<TreeElForClimbing>) {
        let current_segment = tree_for_climbing.get(self.now_segment);
        match current_segment {
            None => {}
            Some(&TreeElForClimbing) => {}
        }
    } 
}