use rand::prelude::*;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlProgram, WebGlShader, WebGlUniformLocation};

const DEST_REF: i32 = 20;
const F_DEST_REF: f64 = DEST_REF as f64;
const PI: f64 = std::f64::consts::PI;
//const STEM: [[u32; 3]; 6] = [[5,5,1], [2,1,12], [1,11,1], [5,5,5], [11,12,10], [5, 3, 2]]; // [right, left, repeats]
const STEM: [[u32; 3]; 8] = [[5,5,1], [2,1,12], [1,11,1], [3,3,2], [5,6,3], [5, 3, 2], [1, 0, 5], [0, 1, 4]];
//const STEM: [[u32; 3]; 3] = [[5,5,1], [1, 0, 4], [0, 1, 4]];
//const STEM: [[u32; 3]; 3] = [[5,5,1], [1,2,11], [5,5,1]]; // [right, left, repeats]
//const STEM: [[u32; 3]; 2] = [[5,5,1], [1,2,11]]; // [right, left, repeats]
//const STEM: [[u32; 3]; 1] = [[5,5,1]]; // [right, left, repeats]
const SIZE_X: usize = 512;
const SIZE_Y: usize = 512;
const START_X: i32 = 256;
const START_Y: i32 = 511;
const LEFT_BOOL: bool = true;
const RIGHT_BOOL: bool = false;
const SEMI_W: f64 = F_DEST_REF / 2f64;
const MAX_RECTS: usize = 40;
const RECTS_ARR_LENGTH: usize = MAX_RECTS * 8;
const CANVAS_REF_WIDTH: f32 = 600.0;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn render(img: web_sys::HtmlImageElement,
              trunk_ratio: f32) -> Result<(), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;

    let vert_shader = compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es
 
        in vec4 position;

        void main() {
        
            gl_Position = position;
        }
        "##,
    )?;
    
    let shader_str_with_value = format!(r##"#version 300 es
    
        precision mediump float;
        const uint MAX_RECTS = {:?}u;
        const float SCALING = {:?};
        const float PI = 3.1415926538;
        const float leaf_x0 = {:?};
        const float leaf_y0 = {:?};
        const float leaf_w = {:?};
        xonst float leaf_h = {:?};
        const float trunk_x0 = {:?};
        const float trunk_y0 = {:?};
        const float trunk_w = {:?};
        const float trunk_h = {:?};
        const float SEMI_W = {:?};
        const float CANVAS_REF_WIDTH = {:?};
        layout(std140) uniform rectData {{
            vec4 position_size[MAX_RECTS];
            vec4 center_x_y_bound_min_max_mat[MAX_RECTS];
            vec4 straight_left_radius_convastart_x_y_to_plus[MAX_RECTS];
            vec4 is_leaf_dist_from_root_beta[MAX_RECTS];
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
                                 float source_y0) {{
            if (x_source < source_x0 ||
                x_source > source_x0 + source_w ||
                y_source < source_y0 ||
                y_source > source_y0 + source_h ||
                texture(u_image, vec2(x_source, y_source)).w < 0.5) return -1;
            float dist_from_base = y_source / scaling;
            if (is_leaf_dist_from_root_beta[i].x > 0.5) {{
                if (straight_left_radius_convastart_x_y_to_plus[i].y > 0.5)
                    dist_from_base = dist_from_base * straight_left_radius_convastart_x_y_to_plus[i].z
                                     / (straight_left_radius_convastart_x_y_to_plus[i].z - SEMI_W);
                else
                    dist_from_base = dist_from_base * straight_left_radius_convastart_x_y_to_plus[i].z
                                     / (straight_left_radius_convastart_x_y_to_plus[i].z + SEMI_W);
            }}
            return is_leaf_dist_from_root_beta[i].y + dist_from_base;
        }}

        void main() {{
            float a = 0f;
            vec2 pos = gl_FragCoord.xy / vec2(canvas_w / CANVAS_REF_WIDTH, canvas_w / CANVAS_REF_WIDTH);
            for (uint i=0u; i < rectCount; i++) {{
                if (pos.x < position_size[i].x ||
                    pos.y < position_size[i].y ||
                    pos.x > position_size[i].x + position_size[i].z ||
                    pos.y > position_size[i].y + position_size[i].w) continue;
                if (is_leaf_dist_from_root_beta[i].x > 0.5) {{
                    float source_w = leaf_w;
                    float source_h = leaf_h;
                    float source_x0 = leaf_x0;
                    float source_y0 = leaf_y0;
                }} else {{
                    float source_w = trunk_w;
                    float source_h = trunk_h;
                    float source_x0 = trunk_x0;
                    float source_y0 = trunk_y0;
                }}
                if (straight_radius_x_y_to_plus[i].x > 0.5) {{
                    // matrix multiplication
                    // (x y) (x)
                    //  z w   y
                    float x_source = center_x_y_bound_min_max_mat[i].x * pos.x +
                                     center_x_y_bound_min_max_mat[i].y * pos.y +
                                     straight_left_radius_convastart_x_y_to_plus[i].y;
                    float y_source = center_x_y_bound_min_max_mat[i].z * pos.x +
                                     center_x_y_bound_min_max_mat[i].w * pos.y +
                                     straight_left_radius_convastart_x_y_to_plus[i].z;
                    float now_a = get_dist_from_root(x_source,
                                                     y_source,
                                                     source_h,
                                                     source_w,
                                                     source_x0,
                                                     source_y0);
                    if (now_a > a) {{
                        a = now_a;
                        outColor = texture(u_image, vec2(source_x, source_y));
                    }}
                }} else {{
                    float deltax = pos.x - center_x_y_bound_min_max_mat[i].x;
                    float deltay = pos.y - center_x_y_bound_min_max_mat[i].y;
                    float dist = sqrt(deltax * deltax +
                                      deltay * deltay);
                    if (center_x_y_bound_min_max_mat[i].z > dist ||
                        dist > center_x_y_bound_min_max_mat[i].w) continue;
                    float x_dest;
                    if (straight_left_radius_convastart_x_y_to_plus[i].y > 0.5) {{
                        if (straight_left_radius_convastart_x_y_to_plus[i].z > 0.0)
                            x_dest = dist - center_x_y_bound_min_max_mat[i].z;
                        else
                            x_dest = center_x_y_bound_min_max_mat[i].w - dist;
                    }} else {{
                        if (straight_left_radius_convastart_x_y_to_plus[i].z > 0.0)
                            x_dest = center_x_y_bound_min_max_mat[i].w - dist;
                        else
                            x_dest = dist - center_x_y_bound_min_max_mat[i].z;
                    }}
                    float x_source = x_dest * SCALING;
                    float gamma = atan(deltay, deltax);
                    float converted_alpha1 = (straight_left_radius_convastart_x_y_to_plus[i].w - gamma)
                                             / (2.0 * PI);
                    float converted_alpha2 = (straight_left_radius_convastart_x_y_to_plus[i].w - gamma
                                              + is_leaf_dist_from_root_beta[i].z)
                                              / (2.0 * PI);
                    int start;
                    int end;
                    if (is_leaf_dist_from_root_beta[i].z > 0.0) {{
                        start = int(ceil(converted_alpha1));
                        end = int(floor(converted_alpha2));
                    }} else {{
                        start = int(ceil(converted_alpha2));
                        end = int(floor(converted_alpha1));
                    }}
                    if (end < start) continue;
                    for (int y_before_conversion=start; y_before_conversion<=end; y_before_conversion++) {{
                        float y_angle = (float(y_before_conversion) * (2.0 * PI) + gamma;
                        float beta_rate = (y_angle - conv_angle_start) / is_leaf_dist_from_root_beta[i].z;
                        float height_this_point_source = source_h * beta_rate;
                        float y_source = source_y0 + height_this_point_source;
                        float now_a = get_dist_from_root(x_source,
                                                         y_source,
                                                         source_h,
                                                         source_w,
                                                         source_x0,
                                                         source_y0);
                        if (now_a > a + ) {{
                            a = now_a;
                            outColor = texture(u_image, vec2(source_x, source_y));
                        }}
                    }}
                }}
            }}
        }}
        "##, MAX_RECTS, scaling, leaf_x0, leaf_y0, leaf_w, leaf_h, trunk_x0, trunk_y0, trunk_w, trunk_h, SEMI_W, CANVAS_REF_WIDTH);

    let frag_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        shader_str_with_value.as_str(),
    )?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    let vertices: [f32; 18] = [-1.0, -1.0, 0.0, 1.0, -1.0, 0.0, -1.0, 1.0, 0.0, 1.0, -1.0, 0.0, 1.0, 1.0, 0.0, -1.0, 1.0, 0.0];

    let position_attribute_location = context.get_attrib_location(&program, "position");
    let buffer = context.create_buffer().ok_or("Failed to create buffer")?;
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        let positions_array_buf_view = js_sys::Float32Array::view(&vertices);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    let vao = context
        .create_vertex_array()
        .ok_or("Could not create vertex array object")?;
    context.bind_vertex_array(Some(&vao));

    context.vertex_attrib_pointer_with_i32(
        position_attribute_location as u32,
        3,
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    context.enable_vertex_attrib_array(position_attribute_location as u32);

    context.bind_vertex_array(Some(&vao));

    let vert_count = (vertices.len() / 3) as i32;
    
    let texture = context.create_texture().expect("Cannot create gl texture");
    let level = 0;
    let internal_format = WebGl2RenderingContext::RGBA;
    let src_format = WebGl2RenderingContext::RGBA;
    let src_type = WebGl2RenderingContext::UNSIGNED_BYTE;

    context.active_texture(WebGl2RenderingContext::TEXTURE0);
    context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
    context.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_WRAP_S,
        WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );
    context.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_WRAP_T,
        WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
    );
    context.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MIN_FILTER,
        WebGl2RenderingContext::LINEAR as i32,
    );
    context.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MAG_FILTER,
        WebGl2RenderingContext::LINEAR as i32,
    );

    let err = context.tex_image_2d_with_u32_and_u32_and_html_image_element(
                WebGl2RenderingContext::TEXTURE_2D,
                level,
                internal_format as i32,
                src_format,
                src_type,
                &img,
            );
    match err {
    Ok(()) => (),
    Err(val) => {return Err(val)}
    };

    let mut rects_arr: [f32; RECTS_ARR_LENGTH] = [0.0; RECTS_ARR_LENGTH];
    let rects_count = 18;
    generate_rects(rects_count,
                   0.05,
                   0.8,
                   0.05,
                   0.07,
                   &mut rects_arr);
    // Get the index of the Uniform Block from any program
    let blockIndex = context.get_uniform_block_index(&program, "rectData");

    // Get the size of the Uniform Block in bytes
    let blockSize = context.get_active_uniform_block_parameter(
        &program,
        blockIndex,
        WebGl2RenderingContext::UNIFORM_BLOCK_DATA_SIZE
    );
    alert(&format!("block size in bytes {:?}!", blockSize));

    // Create Uniform Buffer to store our data
    let ubo_buffer = context.create_buffer().ok_or("Failed to create buffer")?;

    // Bind it to tell WebGL we are working on this buffer
    context.bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, Some(&ubo_buffer));

    unsafe {
        let rects_array_buf_view = js_sys::Float32Array::view(&rects_arr);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::UNIFORM_BUFFER,
            &rects_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }

    // Allocate memory for our buffer equal to the size of our Uniform Block
    // We use dynamic draw because we expect to respecify the contents of the buffer frequently
    //gl.bufferData(gl.UNIFORM_BUFFER, blockSize, gl.DYNAMIC_DRAW);

    // Unbind buffer when we're done using it for now
    // Good practice to avoid unintentionally working on it
    context.bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, None);

    // Bind the buffer to a binding point
    // Think of it as storing the buffer into a special UBO ArrayList
    // The second argument is the index you want to store your Uniform Buffer in
    // Let's say you have 2 unique UBO, you'll store the first one in index 0 and the second one in index 1
    context.bind_buffer_base(WebGl2RenderingContext::UNIFORM_BUFFER, 0, Some(&ubo_buffer));

    // Name of the member variables inside of our Uniform Block
    //const uboVariableNames = ["u_PointSize", "u_Color"];

    // Get the respective index of the member variables inside our Uniform Block
    //const uboVariableIndices = gl.getUniformIndices(
    //program_normal,
    //uboVariableNames
    //);

    // Get the offset of the member variables inside our Uniform Block in bytes
    //const uboVariableOffsets = gl.getActiveUniforms(
    //program_normal,
    //uboVariableIndices,
    //gl.UNIFORM_OFFSET
    //);

    // Create an object to map each variable name to its respective index and offset
    //const uboVariableInfo = {};

    //uboVariableNames.forEach((name, index) => {
    //uboVariableInfo[name] = {
    //    index: uboVariableIndices[index],
    //    offset: uboVariableOffsets[index],
    //};
    //});

    // The 3rd argument is the binding point of our Uniform Buffer
    // uniformBlockBinding tells WebGL to
    // link the Uniform Block inside of this program
    // to the Uniform Buffer at index X of our Special UBO ArrayList
    //
    // Remember that we placed our UBO at index 0 of our Special UBO ArrayList in line 213 in Part A

    let index = context.get_uniform_block_index(&program, "rectData");
    context.uniform_block_binding(&program, index, 0);
    let index2 = context.get_uniform_location(&program, "rectCount");
    context.uniform1ui(index2.as_ref(), rects_count as u32);

    draw(&context, vert_count);
    Ok(())

}


fn draw(context: &WebGl2RenderingContext, vert_count: i32) {
    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

    context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, vert_count);
}

pub fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

fn generate_rects(count: usize,
                  pos_min: f32,
                  pos_max: f32,
                  size_min:f32,
                  size_max: f32,
                  arr: &mut [f32]) {
    for i in 0..count {
        let pos_x = rand::random_range(pos_min..pos_max);
        let pos_y = rand::random_range(pos_min..pos_max);
        let size_x = rand::random_range(size_min..size_max);
        let size_y = rand::random_range(size_min..size_max);
        let r = rand::random_range(0f32..1f32);
        let g = rand::random_range(0f32..1f32);
        let b = rand::random_range(0f32..1f32);
        arr[i * 4] = pos_x;
        arr[i * 4 + 1] = pos_y;
        arr[i * 4 + 2] = size_x;
        arr[i * 4 + 3] = size_y;
        arr[MAX_RECTS * 4 + i * 4] = r;
        arr[MAX_RECTS * 4 + i * 4 + 1] = g;
        arr[MAX_RECTS * 4 + i * 4 + 2] = 0.0;
        arr[MAX_RECTS * 4 + i * 4 + 3] = 1.0;
    }
}

fn get_size(width: i32, height: i32, f_source_ref: f64) -> (f64, f64, f64) {
    let width = width as f64;
    let height = height as f64;
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
    let center_bottom_y = center_bottom_y - radius * (angle_stop.sin() - angle_start.sin());
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
    let center_bottom_y = center_bottom_y - segment_length * angle.cos();
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

fn get_segments(w: f64) -> Vec<Segment> {
    let mut segments_capacity = 0;
    for item in STEM {
        segments_capacity += item[0] * item[2];
        segments_capacity += item[1] * item[2];
    }
    let mut segments = Vec::with_capacity(segments_capacity as usize);
    let mut center_bottom_x_global = START_X as f64;
    let mut center_bottom_y_global = START_Y as f64;
    let mut angle = 0f64;
    let mut total_distance_from_root = 0f64;
    for item in STEM {
        let right = item[0];
        let left = item[1];
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
        for _i in 0..item[2] {
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
        center_top_y = segment.center_bottom_y - h * angle_end.cos();
    } else {
        let delta_angle = h / real_rad;
        angle_end = segment.angle_start + delta_angle;
        center_top_x = segment.center_bottom_x + segment.radius * (angle_end.cos() - segment.angle_start.cos());
        center_top_y = segment.center_bottom_y - segment.radius * (angle_end.sin() - segment.angle_start.sin());
    }
    for de_w in [de_w1, de_w2] {
        let pointx = segment.center_bottom_x + de_w * segment.angle_start.cos();
        let pointy = segment.center_bottom_y - de_w * segment.angle_start.sin();
        points.push([pointx, pointy]);
        let pointx = center_top_x + de_w * angle_end.cos();
        let pointy = center_top_y - de_w * angle_end.sin();
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
            let center_y = segment.center_bottom_y + segment.angle_start.sin() * segment.radius;
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

fn convert_coords_straight(x: i32,
                           y: i32,
                           mat: [[f64; 2]; 2],
                           x_to_plus: f64,
                           y_to_plus: f64) -> (i32, i32) {
    let x_d = x as f64 + 0.5;
    let y_d = y as f64 + 0.5;
    let (x_before_add, y_before_add) = mat_vec_mul(mat, x_d, y_d);
    ((x_before_add + x_to_plus).ceil() as i32, (y_before_add + y_to_plus).ceil() as i32)
}

fn mat_vec_mul(mat: [[f64; 2]; 2], x: f64, y: f64) -> (f64, f64) {
    let new_x = mat[0][0] * x + mat[0][1] * y;
    let new_y = mat[1][0] * x + mat[1][1] * y;
    (new_x, new_y)
}

fn populate_leaf(rect: &Rect,
                 segment: &Segment,
                 is_leaf: bool,
                 f_source_ref: f64,
                 scaling: f64,
                 width: i32,
                 height: i32,
                 arr: &mut [f32]) {
    let semi_w_big_side = (source_w - f_source_ref / 2f64) / scaling;
    if segment.straight {
        let si = segment.angle_start.sin();
        let co = segment.angle_start.cos();
        let mat;
        let corner_dest_x;
        let corner_dest_y;
        if segment.left {
            mat = [[scaling * co, -scaling * si],
                    [scaling * si, scaling * co]];
            corner_dest_x = segment.center_bottom_x + SEMI_W * co;
            corner_dest_y = segment.center_bottom_y - SEMI_W * si;
        } else {
            mat = [[-scaling * co, scaling * si],
                    [scaling * si, scaling * co]];
            corner_dest_x = segment.center_bottom_x - SEMI_W * co;
            corner_dest_y = segment.center_bottom_y + SEMI_W * si;
        }
        let (conv_cdx, conv_cdy) = mat_vec_mul(mat, corner_dest_x, corner_dest_y);
        let x_to_plus = source_x0 + source_w - conv_cdx;
        let y_to_plus = source_y0 + source_h - conv_cdy;

        /*for x in rect.x..=rect.x + rect.width {
            for y in rect.y..=rect.y + rect.height {
                let (x_source, y_source) =  convert_coords_straight(x, y, mat, x_to_plus, y_to_plus);
                place_pixel(x_source, y_source, x, y, dest_img, distances_arr, source_img, source_w, source_h, source_x0, source_y0, segment, scaling, width, height);
            }
        } */
    } else {
        let beta;
        if segment.left {
            beta = (source_h / scaling) / (segment.radius - SEMI_W);
        } else {
            beta = (source_h / scaling) / (segment.radius + SEMI_W);
        }
        let center_x = segment.center_bottom_x - segment.radius * segment.angle_start.cos();
        let center_y = segment.center_bottom_y + segment.radius * segment.angle_start.sin();
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
        /*for x in rect.x..=rect.x + rect.width {
            let xf = x as f64 + 0.5;
            for y in rect.y..=rect.y + rect.height {
                let yf = y as f64 + 0.5;
                let dist = ((xf - center_x).powf(2f64) + (yf - center_y).powf(2f64)).sqrt();
                if bound_min < dist && dist < bound_max {
                    let x_dest;
                    if segment.left {
                        if segment.radius > 0.0 {
                            x_dest = bound_max - dist;
                        } else {
                            x_dest = dist - bound_min;
                        }
                    } else {
                        if segment.radius > 0.0 {
                            x_dest = dist - bound_min;
                        } else {
                            x_dest = bound_max - dist;
                        }
                    }
                    let x_source = (source_x0 + source_w - x_dest * scaling).ceil() as i32;
                    let gamma =  (center_y - yf).atan2(xf - center_x);
                    let converted_alpha1 = (conv_angle_start - gamma) / (2.0 * PI);
                    let converted_alpha2 = (conv_angle_start + beta - gamma) / (2.0 * PI);
                    let start;
                    let end;
                    if beta > 0.0 {
                        start = converted_alpha1.ceil() as i32;
                        end = converted_alpha2.floor() as i32;
                    } else {
                        start = converted_alpha2.ceil() as i32;
                        end = converted_alpha1.floor() as i32;
                    }
                    if end >= start {
                        for y_before_conversion in start..=end {
                            let y_angle = (y_before_conversion as f64) * (2.0 * PI) + gamma;
                            let beta_rate = (y_angle - conv_angle_start) / beta;
                            let height_this_point_source = source_h * beta_rate;
                            let y_source = (source_y0 + source_h - height_this_point_source).ceil() as i32;
                            place_pixel(x_source, y_source, x, y, dest_img, distances_arr, source_img, source_w, source_h, source_x0, source_y0, segment, scaling, width, height);
                        }
                    }
                }
            }
        } */
    }
}*/

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

fn generate_shader_and_arr(width: i32,
                           height: i32,
                           source_ref: i32,
                           arr: &mut [f32]) {
    let f_source_ref: f64 = source_ref as f64;
    let scaling: f64 = f_source_ref / (F_DEST_REF);
    let trunk_w = width as f64;
    let trunk_h = f_source_ref * PI / 6f64;
    let trunk_x0 = 0f64;
    let trunk_y0 = height as f64 - trunk_h;
    let leaf_w = width as f64;
    let leaf_h = trunk_y0;
    let leaf_x0 = 0f64;
    let leaf_y0 = 0f64;
    let mut distances_arr = vec![vec![-1f64; SIZE_X]; SIZE_Y];
    let (w_common, h_bottom, h_top) = get_size(width, height, f_source_ref);
    let segments = get_segments(F_DEST_REF);

    for segment_num in 0..segments.len() {
        let leaf_segment = &segments[segment_num];
        let rect = get_rect(w_common, h_top, leaf_segment);
        //draw_leaf(&rect, leaf_segment, &mut dest_img, &mut distances_arr, &source_img, leaf_w, leaf_h, leaf_x0, leaf_y0, f_source_ref, scaling, width, height);
        let trunk_segment_num = get_trunk_segment(&segments, segment_num);
        if trunk_segment_num >= 0 {
            let trunk_segment = &segments[trunk_segment_num as usize];
            let rect = get_rect(w_common, h_bottom, trunk_segment);
            //draw_leaf(&rect, trunk_segment, &mut dest_img, &mut distances_arr, &source_img, trunk_w, trunk_h, trunk_x0, trunk_y0, f_source_ref, scaling, width, height);
        }
    }
    //dest_img.save("new2.png").unwrap();
}
