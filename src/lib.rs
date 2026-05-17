use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlUniformLocation, WebGlVertexArrayObject};
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

#[wasm_bindgen]
struct Monkey {
    x_pos: f64,
    is_running: bool,
    is_left: bool,
}

#[wasm_bindgen]
struct GlParameters {
    rect_count_index: Option<WebGlUniformLocation>,
    canvas_w_index: Option<WebGlUniformLocation>,
    vert_count: i32,
}

struct TreeState {
    right: u32,
    left: u32,
}

#[wasm_bindgen]
pub struct GameState {
    tree: Vec<TreeUnit>,
    monkey: Monkey,
    drawing_params: tree::DrawingParams,
    context: WebGl2RenderingContext,
    ubos_arr: [[f32; BUF_LENGTH]; NUM_UNIFORM_ARRAYS],
    canvas_width: f32,
    gl_params: GlParameters,
    tree_state: TreeState,
    monkey_running_program: Option<WebGlProgram>,
    leaf_program: Option<WebGlProgram>,
    monkey_vao: Option<WebGlVertexArrayObject>,
    leaf_vao: Option<WebGlVertexArrayObject>
}

#[wasm_bindgen]
impl GameState {
    pub fn new() -> Result<GameState, JsValue> {
        let tree: Vec<TreeUnit> = vec![]; 
        let monkey = Monkey{x_pos: 200.0, is_running: false, is_left: false};
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
        let tree_state = TreeState{
            right: 0,
            left: 0,
        };
        Ok(GameState {
            tree,
            monkey,
            drawing_params,
            context,
            ubos_arr,
            canvas_width,
            gl_params,
            tree_state,
            monkey_running_program: None,
            leaf_program: None,
            monkey_vao: None,
            leaf_vao: None,
        })
    }

    #[wasm_bindgen]
    pub fn left_plus(&mut self) -> u32 {
        self.tree_state.left += 1;
        self.tree_state.left
    }

    #[wasm_bindgen]
    pub fn left_minus(&mut self) -> u32 {
        if self.tree_state.left > 0 {
            self.tree_state.left -= 1;
        }
        self.tree_state.left
    }

    #[wasm_bindgen]
    pub fn right_plus(&mut self) -> u32 {
        self.tree_state.right += 1;
        self.tree_state.right
    }

    #[wasm_bindgen]
    pub fn right_minus(&mut self) -> u32 {
        if self.tree_state.right > 0 {
            self.tree_state.right -= 1;
        }
        self.tree_state.right
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
                                         theta_min: f64) -> Result<(), JsValue> {
        
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
             leaf_vao) = gl_related::prepare_gl(img_leaf,
                                                      &shader_str,
                                                      &self.context)?;
        self.leaf_vao = Some(leaf_vao);
        self.leaf_program = Some(program);
        gl_related::bind_ubos_for_tree(&self.ubos_arr, &self.context, self.leaf_program.as_ref(), true)?;
        self.gl_params.rect_count_index = rect_count_index;
        self.gl_params.canvas_w_index = canvas_w_index;
        self.gl_params.vert_count = 6; // TODO change to meaningful!
        let (monkey_running_program,
            monkey_vao) = gl_related::prepare_monkey(img_monkey,
                                                                             &self.context,
                                                                             self.monkey.x_pos as f32,
                                                                             1.0)?;
        self.monkey_running_program = Some(monkey_running_program);
        self.monkey_vao = Some(monkey_vao);
        Ok(())
    }

    fn create_new_tree_unit(&mut self, left:u32, right:u32) {
        self.tree.push(TreeUnit{left, right, repeats: 1})
    }

    #[wasm_bindgen]
    pub fn grow_tree(&mut self) -> Result<(), JsValue> {
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
        self.draw_tree()?;
        Ok(())
    }

    #[wasm_bindgen]
    pub fn undo_grow_tree(&mut self) -> Result<(), JsValue> {
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
        self.draw_tree()?;
        Ok(())
    }

    fn draw_tree(&mut self) -> Result<(), JsValue> {
        self.context.use_program(self.leaf_program.as_ref());
        self.context.bind_vertex_array(self.leaf_vao.as_ref());
        let rects_count = self.populate_arr();
        gl_related::bind_ubos_for_tree(&self.ubos_arr, &self.context, None, false)?;
        self.context.uniform1ui(self.gl_params.rect_count_index.as_ref(), rects_count as u32);
        self.context.uniform1f(self.gl_params.canvas_w_index.as_ref(), self.canvas_width);

        gl_related::draw(&self.context, 6, true, 0);
        self.draw_monkey()?;
        Ok(())
    }

    pub fn draw_monkey(&self) -> Result<(), JsValue> {
        self.context.use_program(self.monkey_running_program.as_ref());
        self.context.bind_vertex_array(self.monkey_vao.as_ref());
        gl_related::draw(&self.context, 6, false, 0);
        Ok(())
    }
}