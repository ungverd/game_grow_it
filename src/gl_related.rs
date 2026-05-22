use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlProgram, WebGlShader, WebGlUniformLocation, WebGlVertexArrayObject};

pub const TARGET_TEXTURE_WIDTH: i32 = 600;
pub const TARGET_TEXTURE_HEIGHT: i32 = 600;

pub fn get_context_and_canvas_width() -> Result<(WebGl2RenderingContext, f32), JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    let canvas_width = canvas.width() as f32;
    let context = canvas
        .get_context("webgl2")?
        .unwrap()
        .dyn_into::<WebGl2RenderingContext>()?;
    Ok((context, canvas_width))
}

pub fn prepare_gl(img: web_sys::HtmlImageElement,
                  tree_shader_str: &str,
                  context:  &WebGl2RenderingContext) -> Result<(Option<WebGlUniformLocation>,
                                                                Option<WebGlUniformLocation>,
                                                                WebGlProgram,
                                                                WebGlVertexArrayObject,
                                                                WebGlFramebuffer), JsValue> {

    let vert_shader = compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es
 
        in vec4 position;
        in vec2 a_texCoord;
        out vec2 screen_coord;

        void main() {
        
            gl_Position = position;
            screen_coord = a_texCoord;
        }
        "##,
    )?;
    
    let frag_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        tree_shader_str,
    )?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    let vertices: [f32; 12] = [-1.0, -1.0,
                               1.0, -1.0,
                               -1.0, 1.0,
                               1.0, -1.0,
                               1.0, 1.0,
                               -1.0, 1.0];

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
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    context.enable_vertex_attrib_array(position_attribute_location as u32);

    let texture_coords: [f32; 12] = [0.0, 0.0, 
                                     crate::CANVAS_REF_WIDTH, 0.0,
                                     0.0, crate::CANVAS_REF_WIDTH,
                                     crate::CANVAS_REF_WIDTH, 0.0,
                                     crate::CANVAS_REF_WIDTH, crate::CANVAS_REF_WIDTH,
                                     0.0, crate::CANVAS_REF_WIDTH];                     
    let tex_coord_attribute_location = context.get_attrib_location(&program, "a_texCoord");
    //crate::log(&format!("tex_coord {:?}", tex_coord_attribute_location));
    let tex_coord_buffer = context.create_buffer().ok_or("Failed to create buffer")?;
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&tex_coord_buffer));

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        let texture_coords_array_buf_view = js_sys::Float32Array::view(&texture_coords);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &texture_coords_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }
    context.enable_vertex_attrib_array(tex_coord_attribute_location as u32);
    context.vertex_attrib_pointer_with_i32(
        tex_coord_attribute_location as u32,
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );

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
        WebGl2RenderingContext::NEAREST as i32,
    );
    context.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MAG_FILTER,
        WebGl2RenderingContext::NEAREST as i32,
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
    let rect_count_index = context.get_uniform_location(&program, "rectCount");
    let canvas_w_index = context.get_uniform_location(&program, "canvas_w");
    let leaf_texture_index = context.get_uniform_location(&program, "u_image");
    context.uniform1i(leaf_texture_index.as_ref(), 0);
 
    // Creating texture to draw tree in!
    let texture_to_buffer = context.create_texture().expect("Cannot create gl texture");
    let level = 0;
    let internal_format = WebGl2RenderingContext::RGBA;
    let src_format = WebGl2RenderingContext::RGBA;
    let src_type = WebGl2RenderingContext::UNSIGNED_BYTE;

    context.active_texture(WebGl2RenderingContext::TEXTURE1);
    context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture_to_buffer));
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
        WebGl2RenderingContext::NEAREST as i32,
    );
    context.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MAG_FILTER,
        WebGl2RenderingContext::NEAREST as i32,
    );
    let err = context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
                WebGl2RenderingContext::TEXTURE_2D,
                level,
                internal_format as i32,
                TARGET_TEXTURE_WIDTH,
                TARGET_TEXTURE_HEIGHT,
                0,
                src_format,
                src_type,
                None
            );
    match err {
    Ok(()) => (),
    Err(val) => {return Err(val)}
    };
    // Create and bind the framebuffer
    let fb = context.create_framebuffer().expect("Cannot create gl texture");
    context.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&fb));
 
    // attach the texture as the first color attachment
    let attachment_point = WebGl2RenderingContext::COLOR_ATTACHMENT0;
    context.framebuffer_texture_2d(WebGl2RenderingContext::FRAMEBUFFER,
                                 attachment_point,
                                 WebGl2RenderingContext::TEXTURE_2D,
                                 Some(&texture_to_buffer),
                                 level);

    Ok((rect_count_index, canvas_w_index, program, vao, fb))

}


pub fn draw(context: &WebGl2RenderingContext, vert_count: i32, clear: bool, offset: i32) {
    if clear {
        context.clear_color(0.85, 1.0, 0.8, 1.0);
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);
    }
    context.draw_arrays(WebGl2RenderingContext::TRIANGLES, offset, vert_count);
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


pub fn bind_ubos_for_tree(arr: &[[f32; crate::BUF_LENGTH]],
                      context: &WebGl2RenderingContext,
                      program: Option<&WebGlProgram>,
                      is_first_time: bool) -> Result<(), JsValue> {
    for i in 0..crate::NUM_UNIFORM_ARRAYS {
        let ubo_name = crate::UBOS_NAMES[i];
        let rects_arr = &arr[i];
        // Get the index of the Uniform Block from any program
        //let block_index = context.get_uniform_block_index(&program, ubo_name);

        // Get the size of the Uniform Block in bytes
        //let block_size = context.get_active_uniform_block_parameter(
        //    &program,
        //    block_index,
        //    WebGl2RenderingContext::UNIFORM_BLOCK_DATA_SIZE
        //);
        //alert(&format!("block size in bytes {:?}!", block_size));
        // Create Uniform Buffer to store our data
        let ubo_buffer = context.create_buffer().ok_or("Failed to create buffer")?;

        // Bind it to tell WebGL we are working on this buffer
        context.bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, Some(&ubo_buffer));
        unsafe {
            let rects_array_buf_view = js_sys::Float32Array::view(rects_arr);

            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::UNIFORM_BUFFER,
                &rects_array_buf_view,
                WebGl2RenderingContext::DYNAMIC_DRAW,
            );
        }

        context.bind_buffer(WebGl2RenderingContext::UNIFORM_BUFFER, None);

        context.bind_buffer_base(WebGl2RenderingContext::UNIFORM_BUFFER, i as u32, Some(&ubo_buffer));

        if is_first_time {
            let index = context.get_uniform_block_index(&program.unwrap(), ubo_name);
            context.uniform_block_binding(&program.unwrap(), index, i as u32);
        }
    }
    Ok(())
}

pub fn apply_arrays_monkey(context:  &WebGl2RenderingContext,
                           monkey_vertex_array: &[f32],
                           monkey_texture_array: &[f32],
                           monkey_vertex_buffer: Option<&WebGlBuffer>,
                           monkey_tex_coord_buffer: Option<&WebGlBuffer>,) {
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, monkey_vertex_buffer);

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        let positions_array_buf_view = js_sys::Float32Array::view(monkey_vertex_array);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, monkey_tex_coord_buffer);

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        let tex_coords_array_buf_view = js_sys::Float32Array::view(monkey_texture_array);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &tex_coords_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
}

pub fn prepare_monkey(img: web_sys::HtmlImageElement,
                      context:  &WebGl2RenderingContext,
                      monkey_vertex_array: &[f32],
                      monkey_texture_array: &[f32],) -> Result<(WebGlProgram,
                                                                WebGlVertexArrayObject,
                                                                WebGlBuffer,
                                                                WebGlBuffer,), JsValue> {
    let vert_shader = compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es
 
        in vec4 position;
        in vec2 a_texCoord;
        out vec2 v_texCoord;

        void main() {
            gl_Position = position;
            v_texCoord = a_texCoord;
        }
        "##,
    )?;
    
    let frag_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r##"#version 300 es
    
        precision mediump float;
        uniform sampler2D monkey_image;
        
        in vec2 v_texCoord;
        out vec4 outColor;
        
        void main() {
            if (texture(monkey_image, v_texCoord).x > 0.5) {
                outColor = vec4(0.0, 0.0, 0.0, 1.0);
            } else {
                discard;
            }
        }"##,
    )?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));
    
    let position_attribute_location = context.get_attrib_location(&program, "position");
    //crate::log(&format!("position_attribute_location {:?}", position_attribute_location));
    let positions_buffer = context.create_buffer().ok_or("Failed to create buffer")?;
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&positions_buffer));

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        let positions_array_buf_view = js_sys::Float32Array::view(monkey_vertex_array);

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
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    context.enable_vertex_attrib_array(position_attribute_location as u32);

    /*let texture_coords: [f32; 12] = [0.0, 0.0, 
                                     1.0, 0.0,
                                     0.0, 1.0,
                                     1.0, 0.0,
                                     1.0, 1.0,
                                     0.0, 1.0];  */                          
    let tex_coord_attribute_location = context.get_attrib_location(&program, "a_texCoord");
    //crate::log(&format!("tex_coord {:?}", tex_coord_attribute_location));
    let tex_coord_buffer = context.create_buffer().ok_or("Failed to create buffer")?;
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&tex_coord_buffer));

    // Note that `Float32Array::view` is somewhat dangerous (hence the
    // `unsafe`!). This is creating a raw view into our module's
    // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // causing the `Float32Array` to be invalid.
    //
    // As a result, after `Float32Array::view` we have to be very careful not to
    // do any memory allocations before it's dropped.
    unsafe {
        let texture_coords_array_buf_view = js_sys::Float32Array::view(monkey_texture_array);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &texture_coords_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }
    context.enable_vertex_attrib_array(tex_coord_attribute_location as u32);
    context.vertex_attrib_pointer_with_i32(
        tex_coord_attribute_location as u32,
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );

    let texture = context.create_texture().expect("Cannot create gl texture");
    let level = 0;
    let internal_format = WebGl2RenderingContext::R8;
    let src_format = WebGl2RenderingContext::RED;
    let src_type = WebGl2RenderingContext::UNSIGNED_BYTE;

    context.active_texture(WebGl2RenderingContext::TEXTURE2);
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
        WebGl2RenderingContext::NEAREST as i32,
    );
    context.tex_parameteri(
        WebGl2RenderingContext::TEXTURE_2D,
        WebGl2RenderingContext::TEXTURE_MAG_FILTER,
        WebGl2RenderingContext::NEAREST as i32,
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
    let monkey_texture_index = context.get_uniform_location(&program, "monkey_image");
    context.uniform1i(monkey_texture_index.as_ref(), 2);
    Ok((program, vao, positions_buffer, tex_coord_buffer))
}

pub fn prepare_to_draw_background(context: &WebGl2RenderingContext) -> Result<(WebGlProgram,
                                                                               WebGlVertexArrayObject), JsValue> {
    let vert_shader = compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        r##"#version 300 es
 
        in vec4 position;
        in vec2 a_texCoord;
        out vec2 v_texCoord;

        void main() {
            gl_Position = position;
            v_texCoord = a_texCoord;
        }
        "##,
    )?;

    let frag_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r##"#version 300 es
    
        precision mediump float;
        uniform sampler2D background_image;
        
        in vec2 v_texCoord;
        out vec4 outColor;
        
        void main() {
            outColor = texture(background_image, v_texCoord);
        }"##,
    )?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));
    let vertices: [f32; 12] = [-1.0, -1.0,
                               1.0, -1.0,
                               -1.0, 1.0,
                               1.0, -1.0,
                               1.0, 1.0,
                               -1.0, 1.0];
    let texture_coords: [f32; 12] = [0.0, 0.0, 
                                     1.0, 0.0,
                                     0.0, 1.0,
                                     1.0, 0.0,
                                     1.0, 1.0,
                                     0.0, 1.0];
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
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    context.enable_vertex_attrib_array(position_attribute_location as u32);

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
        let tex_coord_array_buf_view = js_sys::Float32Array::view(&texture_coords);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &tex_coord_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    let tex_coord_attribute_location = context.get_attrib_location(&program, "a_texCoord");
    context.vertex_attrib_pointer_with_i32(
        tex_coord_attribute_location as u32,
        2,
        WebGl2RenderingContext::FLOAT,
        false,
        0,
        0,
    );
    context.enable_vertex_attrib_array(tex_coord_attribute_location as u32);
    let leaf_texture_index = context.get_uniform_location(&program, "background_image");
    context.uniform1i(leaf_texture_index.as_ref(), 1);
    Ok((program, vao))
}