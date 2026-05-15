use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader};
mod tree;
mod gl_related;

pub const MAX_RECTS: usize = 1024;
pub const BUF_LENGTH: usize = MAX_RECTS * 4;
pub const NUM_UNIFORM_ARRAYS: usize = 7;
pub const CANVAS_REF_WIDTH: f32 = 600.0;
pub const UBOS_NAMES: [&'static str; 7] = ["Position_size",
                                       "Center_x_y_bound_min_max_mat",
                                       "Straightleaf_left_radius_convastart_x_y_to_plus",
                                       "Dist_from_root_beta_prevleft_prevright",
                                       "Smoothstepcenter_xy_12",
                                       "Angle1_dif1_angle2_dif2",
                                       "Radius1_dif1_radius2_dif2"];

#[wasm_bindgen]
pub struct TreeUnit {
    left: u32,
    right: u32,
    repeats: u32,
}

#[wasm_bindgen]
struct Monkey {
    x_pos: f64,
    is_running: bool,
}

#[wasm_bindgen]
pub struct GameState {
    tree: Vec<TreeUnit>,
    monkey: Monkey,
    drawing_params: tree::DrawingParams,
    context: WebGl2RenderingContext,
    ubos_arr: [[f32; BUF_LENGTH]; NUM_UNIFORM_ARRAYS],
}

#[wasm_bindgen]
impl GameState {
    pub fn new() -> Result<GameState, JsValue> {
        let tree: Vec<TreeUnit> = vec![]; 
        let monkey = Monkey{x_pos: 300.0, is_running: false};
        let drawing_params = tree::DrawingParams{scaling: 0.0,
                                                                trunk_w: 0.0,
                                                                trunk_h: 0.0,
                                                                trunk_x0: 0.0,
                                                                trunk_y0: 0.0,
                                                                leaf_w: 0.0,
                                                                leaf_h: 0.0,
                                                                leaf_x0: 0.0,
                                                                leaf_y0: 0.0,
                                                                w_common: 0.0,
                                                                h_bottom: 0.0,
                                                                h_top: 0.0,
                                                                width_ratio: 0.0,
                                                                height_ratio: 0.0,
                                                                source_ref_ratio: 0.0,
                                                                x_circle: 0.0,
                                                                y_circle: 0.0,
                                                                radius: 0.0,
                                                                theta_max: 0.0,
                                                                theta_min: 0.0,};
        let(context, canvas_width) = gl_related::get_context_and_canvas_width()?;
        let ubos_arr = [[0.0; BUF_LENGTH]; NUM_UNIFORM_ARRAYS];  
        Ok(GameState {
            tree,
            monkey,
            drawing_params,
            context,
            ubos_arr,
        })
    }

    pub fn do_shader_stuff_and_constants(&mut self,
                                         img: web_sys::HtmlImageElement,
                                         width_ratio: f64,
                                         height_ratio: f64,
                                         trunk_ratio: f64,
                                         x_circle: f64,
                                         y_circle: f64,
                                         radius: f64,
                                         theta_max: f64,
                                         theta_min: f64) { //-> Result<(), JsValue> {
        
        let shader_str = self.generate_shader_and_populate_values(width_ratio,
                                                                          height_ratio,
                                                                          trunk_ratio,
                                                                          x_circle,
                                                                          y_circle,
                                                                          radius,
                                                                          theta_max,
                                                                          theta_min);
    }

    fn create_new_tree_unit(&mut self, left:u32, right:u32) {
        self.tree.push(TreeUnit{left, right, repeats: 1})
    }

    pub fn grow_tree(&mut self, left: u32, right: u32) {
        let last = self.tree.last_mut();
        match last {
            Some(last) => {
                if last.left == left && last.right == right {
                    last.repeats += 1;
                } else {
                    self.create_new_tree_unit(left, right);
                }
            }
            None => { self.create_new_tree_unit(left, right); }
        }
    }

    pub fn undo_grow_tree(&mut self) {
        let last = self.tree.last_mut();
        match last {
            Some(last) => {
                if last.repeats > 1 {
                    last.repeats -= 1;
                } else {
                    self.tree.pop();
                }
            }
            None => {}
        }
    }
    pub fn draw_tree(&mut self) {
        self.populate_arr();
        gl_related::bind_ubos_for_tree(&self.ubos_arr, &self.context, None, true)?;
        /*let index2 = context.get_uniform_location(&program, "rectCount");
        context.uniform1ui(index2.as_ref(), rects_count as u32);
        let index3 = context.get_uniform_location(&program, "canvas_w");
        context.uniform1f(index3.as_ref(), canvas_width);

        draw(&context, vert_count); */
    }
}