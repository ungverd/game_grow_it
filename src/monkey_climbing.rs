const DELTAX_FRONT: f32 = 2.0; //px
const DELTAX_BACK: f32 = 2.0; //px
const DELTAY_FRONT: f32 = 5.0; //px
const DELTAY_BACK: f32 = 5.0; //px
const PI: f32 = std::f32::consts::PI;

pub struct MonkeyClimbing {
    total_height: f32,
}

struct TreeElForClimbingNotLinear {
    center_x: f32,
    center_y: f32,
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
                               start_angle: f32,
                               dist_from_root: f32,) {
    let delta_angle = (reduced_unit.right - reduced_unit.left) as f32 * PI / 6.0; 
}

fn make_tree_for_climbing(tree: &Vec<crate::TreeUnit>,
                       start_x: f32,
                       start_y: f32) {
    let joined_tree = make_joined_tree(tree);
}