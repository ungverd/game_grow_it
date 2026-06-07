use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject};

mod tree;
mod gl_related;
mod monkey_running;
mod monkey_climbing;

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
pub const DEST_REF: i32 = 26; // width of tree in pixels

pub const START_X: f32 = 300.0;
pub const START_Y: f32 = 1.0; // coordinates where three starts

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);

    // The `console.log` is quite polymorphic, so we can bind it with multiple
    // signatures. Note that we need to use `js_name` to ensure we always call
    // `log` in JS.
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log_u32(a: u32);

    // Multiple arguments too!
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    pub fn log_many(a: &str, b: &str);
}

#[wasm_bindgen]
pub struct TreeUnit {
    left: u32,
    right: u32,
    repeats: u32,
}

struct GlParameters {
    rect_count_index: Option<WebGlUniformLocation>,
    canvas_w_index: Option<WebGlUniformLocation>,
    vert_count: i32,
}

struct TreeState {
    right: u32,
    left: u32,
}

enum MonkeyState {
    running,
    climbing,
}

struct Monkey {
    monkey_state: MonkeyState,
    running: monkey_running::MonkeyRunning,
    climbing_num: usize,
}

struct TreeStruct {
    tree: Vec<TreeUnit>,
    joined_tree: Vec<monkey_climbing::TreeUnitReduced>,
    tree_state: TreeState,
    tree_for_climbing: Vec<monkey_climbing::TreeElForClimbing>,
    left_arm_vec: Vec<monkey_climbing::LimbEndPos>,
    right_arm_vec: Vec<monkey_climbing::LimbEndPos>,
    left_leg_vec: Vec<monkey_climbing::LimbEndPos>,
    right_leg_vec: Vec<monkey_climbing::LimbEndPos>,
    x_start: f32,
    y_start: f32,
    monkey_climbing: monkey_climbing::MonkeyClimbing
}

impl TreeStruct {
    fn new(x_start: f32, y_start: f32) -> TreeStruct {
        TreeStruct {
            tree: vec![],
            joined_tree: vec![],
            tree_state: TreeState { right: 0, left: 0 },
            tree_for_climbing: vec![],
            left_arm_vec: vec![],
            right_arm_vec: vec![],
            left_leg_vec: vec![],
            right_leg_vec: vec![],
            x_start,
            y_start,
            monkey_climbing: monkey_climbing::MonkeyClimbing::new(),
        }
    }

    fn create_new_tree_unit(&mut self, left:u32, right:u32) {
        self.tree.push(TreeUnit{left, right, repeats: 1})
    }

    fn update_tree_for_climbing(&mut self, straight_up_dist: f32, straight_down_dist: f32) {
        self.make_tree_for_climbing(straight_up_dist,
                                    straight_down_dist);
        self.generate_arms_legs_vectors();
    }

    pub fn grow_tree(&mut self, straight_up_dist: f32, straight_down_dist: f32) {
        let left = self.tree_state.left;
        let right = self.tree_state.right;
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
        self.update_tree_for_climbing(straight_up_dist, straight_down_dist);
    }

    pub fn undo_grow_tree(&mut self, straight_up_dist: f32, straight_down_dist: f32) {
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
        self.update_tree_for_climbing(straight_up_dist, straight_down_dist);
    }
}

#[wasm_bindgen]
pub struct GameState {
    tree_structs: Vec<TreeStruct>,
    monkey: Monkey,
    drawing_params: tree::DrawingParams,
    context: WebGl2RenderingContext,
    ubos_arr: [[f32; BUF_LENGTH]; NUM_UNIFORM_ARRAYS],
    canvas_width: f32,
    gl_params: GlParameters,
    monkey_running_program: Option<WebGlProgram>,
    leaf_program: Option<WebGlProgram>,
    monkey_vao: Option<WebGlVertexArrayObject>,
    leaf_vao: Option<WebGlVertexArrayObject>,
    monkey_position_buffer: Option<WebGlBuffer>,
    monkey_tex_coord_buffer: Option<WebGlBuffer>,
    background_framebuffer: Option<WebGlFramebuffer>,
    background_program: Option<WebGlProgram>,
    background_vao: Option<WebGlVertexArrayObject>,
    straight_up_dist: f32,
    straight_down_dist: f32,
}

#[wasm_bindgen]
impl GameState {
    pub fn new() -> Result<GameState, JsValue> {
        let monkey_now = monkey_running::MonkeyRunning::new();
        let monkey = Monkey{
            monkey_state: MonkeyState::running,
            running: monkey_now,
            climbing_num: 0,
        };
        let (straight_up_dist, straight_down_dist) = get_straight_extended_length();
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
        let gl_params = GlParameters {
            rect_count_index: None,
            canvas_w_index: None,
            vert_count: 0,
        };
        let tree_struct = TreeStruct::new(START_X, START_Y);
        let tree_structs: Vec<TreeStruct> = vec![tree_struct];
        Ok(GameState {
            tree_structs,
            monkey,
            drawing_params,
            context,
            ubos_arr,
            canvas_width,
            gl_params,
            monkey_running_program: None,
            leaf_program: None,
            monkey_vao: None,
            leaf_vao: None,
            monkey_position_buffer: None,
            monkey_tex_coord_buffer: None,
            background_framebuffer: None,
            background_program: None,
            background_vao: None,
            straight_up_dist,
            straight_down_dist,
        })
    }

    #[wasm_bindgen]
    pub fn left_plus(&mut self) -> u32 { // TODO rewrite for multiple trees
        self.tree_structs.first_mut().unwrap().tree_state.left += 1;
        self.tree_structs.first().unwrap().tree_state.left
    }

    #[wasm_bindgen]
    pub fn left_minus(&mut self) -> u32 { // TODO rewrite for multiple trees
        if self.tree_structs.first().unwrap().tree_state.left > 0 {
            self.tree_structs.first_mut().unwrap().tree_state.left -= 1;
        }
        self.tree_structs.first().unwrap().tree_state.left
    }

    #[wasm_bindgen]
    pub fn right_plus(&mut self) -> u32 { // TODO rewrite for multiple trees
        self.tree_structs.first_mut().unwrap().tree_state.right += 1;
        self.tree_structs.first().unwrap().tree_state.right
    }

    #[wasm_bindgen]
    pub fn right_minus(&mut self) -> u32 { // TODO rewrite for multiple trees
        if self.tree_structs.first().unwrap().tree_state.right > 0 {
            self.tree_structs.first_mut().unwrap().tree_state.right -= 1;
        }
        self.tree_structs.first().unwrap().tree_state.right
    }

    pub fn do_shader_stuff_and_constants(&mut self,
                                         img_leaf: web_sys::HtmlImageElement,
                                         img_monkey: web_sys::HtmlImageElement,
                                         width_ratio: f64,
                                         height_ratio: f64,
                                         trunk_ratio: f64,
                                         x_circle: f64,
                                         y_circle: f64,
                                         radius: f64,
                                         theta_max: f64,
                                         theta_min: f64,
                                         width: f32,
                                         height: f32,
                                         frame_width: f32,
                                         frame_height: f32,
                                         n_frames: i32,
                                         time_loop: f32,
                                         advance_loop: f32,) -> Result<(), JsValue> {
        self.monkey.running.populate_parameters(width,
                                                         height,
                                                         frame_width,
                                                         frame_height,
                                                         n_frames,
                                                         time_loop,
                                                         advance_loop);
        let shader_str = self.generate_shader_and_populate_values(width_ratio,
                                                                          height_ratio,
                                                                          trunk_ratio,
                                                                          x_circle,
                                                                          y_circle,
                                                                          radius,
                                                                          theta_max,
                                                                          theta_min);
        let (rect_count_index,
             canvas_w_index,
             program,
             leaf_vao,
             background_framebuffer) = gl_related::prepare_gl(img_leaf,
                                                      &shader_str,
                                                      &self.context)?;
        self.leaf_vao = Some(leaf_vao);
        self.leaf_program = Some(program);
        self.background_framebuffer = Some(background_framebuffer);
        gl_related::bind_ubos_for_tree(&self.ubos_arr, &self.context, self.leaf_program.as_ref(), true)?;
        self.gl_params.rect_count_index = rect_count_index;
        self.gl_params.canvas_w_index = canvas_w_index;
        self.gl_params.vert_count = 6; // TODO change to meaningful!
        let (monkey_running_program,
            monkey_vao,
            monkey_position_buffer,
            monkey_tex_coords_buffer) = gl_related::prepare_monkey(img_monkey,
                                                                             &self.context,
                                                                             &self.monkey.running.vertex_arr,
                                                                             &self.monkey.running.texture_arr)?;
        self.monkey_running_program = Some(monkey_running_program);
        self.monkey_vao = Some(monkey_vao);
        self.monkey_position_buffer = Some(monkey_position_buffer);
        self.monkey_tex_coord_buffer = Some(monkey_tex_coords_buffer);
        let (background_program,
             background_vao) = gl_related::prepare_to_draw_background(&self.context)?;
        self.background_program = Some(background_program);
        self.background_vao = Some(background_vao);
        self.draw_tree()?;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn grow_tree(&mut self) -> Result<(), JsValue> {
        for tree_struct in &mut self.tree_structs {
            tree_struct.grow_tree(self.straight_up_dist, self.straight_down_dist);
        }
        self.draw_tree()?;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn undo_grow_tree(&mut self) -> Result<(), JsValue> {
        for tree_struct in &mut self.tree_structs {
            tree_struct.undo_grow_tree(self.straight_up_dist, self.straight_down_dist);
        }
        self.draw_tree()?;
        Ok(())
    }

    fn draw_tree(&mut self) -> Result<(), JsValue> { // TODO: rewrite for multiple trees
        self.context.use_program(self.leaf_program.as_ref());
        self.context.bind_vertex_array(self.leaf_vao.as_ref());
        let drawing_params = &self.drawing_params;
        let ubos_arr = &mut self.ubos_arr;
        let tree_struct = self.tree_structs.first().unwrap(); // TODO: rewrite!
        let rects_count = tree_struct.populate_arr(drawing_params, ubos_arr);
        gl_related::bind_ubos_for_tree(&self.ubos_arr, &self.context, None, false)?;
        self.context.uniform1ui(self.gl_params.rect_count_index.as_ref(), rects_count as u32);
        self.context.uniform1f(self.gl_params.canvas_w_index.as_ref(), self.canvas_width);
        self.context.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, self.background_framebuffer.as_ref());
        self.context.viewport(0, 0, gl_related::TARGET_TEXTURE_WIDTH, gl_related::TARGET_TEXTURE_WIDTH);
        gl_related::draw(&self.context, 6, true, 0);
        self.context.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
        Ok(())
    }

    fn draw_monkey(&self) -> Result<(), JsValue> {
        self.context.use_program(self.background_program.as_ref());
        self.context.bind_vertex_array(self.background_vao.as_ref());
        gl_related::draw(&self.context, 6, true, 0);
        self.context.use_program(self.monkey_running_program.as_ref());
        self.context.bind_vertex_array(self.monkey_vao.as_ref());
        gl_related::draw(&self.context, 6, false, 0);
        Ok(())
    }

    pub fn onclick(&mut self, x_click: f32, y_click: f32) {
        for tree_struct in self.tree_structs.iter().rev() {
            if tree_struct.
        }
        self.monkey.running.onclick(x_click);
    }

    pub fn on_animation_frame(&mut self, deltat: f32) -> Result<(), JsValue> {
        self.monkey.running.on_animation_frame(deltat);
        gl_related::apply_arrays_monkey(&self.context,
            &self.monkey.running.vertex_arr,
            &self.monkey.running.texture_arr,
            self.monkey_position_buffer.as_ref(),
            self.monkey_tex_coord_buffer.as_ref());
        self.draw_monkey()?;
        Ok(())
    }
}

fn get_circle_vertical_intersection(r: f32, x: f32) -> f32 {
    // center of coordinates at circle's center, vertical straight line 
    (r*r - x*x).sqrt()
}

fn get_straight_extended_length() -> (f32, f32) {
    let r_arm_extended = monkey_climbing::ARM_SEGMENT_LENGTH * monkey_climbing::LEG_EXTENSION_COEFFICIENT;
    let r_leg_extended = monkey_climbing::LEG_SEGMENT_LENGTH * monkey_climbing::LEG_EXTENSION_COEFFICIENT;
    let deltax_arm = monkey_climbing::W / 2.0 - monkey_climbing::DELTAX_FRONT;
    let deltax_leg = monkey_climbing::W / 2.0 - monkey_climbing::DELTAX_BACK;
    let deltay_arm = get_circle_vertical_intersection(r_arm_extended, deltax_arm); 
    let deltay_leg = get_circle_vertical_intersection(r_leg_extended, deltax_leg);
    (deltay_arm + monkey_climbing::DELTAY_FRONT, monkey_climbing::DELTAY_BACK + deltay_leg)
}