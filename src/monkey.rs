const MONKEY_START_X: f32 = 200.0;
const MONKEY_START_Y: f32 = 1.0;
const MONKEY_SCALING: f32 = 0.7; // 0.5 - monkey will be drawn smaller

pub struct Monkey {
    x_pos: f32,
    y_pos: f32,
    is_running: bool,
    is_left: bool,
    parameters: MonkeyRunningDrawParameters,
    pub vertex_arr: [f32; 12],
    pub texture_arr: [f32; 12],
    start_x: f32,
    desired_x: f32,
}

struct MonkeyRunningDrawParameters {
    monkey_width: f32,
    monkey_height: f32,
    width_screen: f32,
    height_screen: f32,
    width_to_draw: f32,
    height_to_draw: f32,
    texture_width: f32,
    texture_height: f32,
    n_frames_running: i32,
    loop_duration: f32,
    advancement_px: f32, // number of pixels monkey will advance during one animation loop
    number_horizontal: i32,
    number_vertical: i32,
    v: f32,
}

impl Monkey {
    pub fn new() -> Monkey {
        let parameters = MonkeyRunningDrawParameters {
            monkey_width: 0.0,
            monkey_height: 0.0,
            width_to_draw: 0.0,
            height_to_draw: 0.0,
            width_screen: 0.0,
            height_screen: 0.0,
            texture_width: 0.0,
            texture_height: 0.0,
            n_frames_running: 0,
            loop_duration: 0.0,
            advancement_px: 0.0,
            number_horizontal: 0,
            number_vertical: 0,
            v: 0.0,
        };
        Monkey {
            x_pos: MONKEY_START_X,
            y_pos: MONKEY_START_Y,
            is_running: false,
            is_left: false,
            parameters,
            vertex_arr: [0.0; 12],
            texture_arr: [0.0; 12],
            start_x: MONKEY_START_X,
            desired_x: MONKEY_START_X,
        }
    } 
    pub fn populate_parameters(&mut self,
                               width: f32,
                               height: f32,
                               frame_width: f32,
                               frame_height: f32,
                               n_frames: i32,
                               time_loop: f32,
                               advance_loop: f32,) {
        self.parameters.texture_width = width;
        self.parameters.texture_height = height;
        self.parameters.monkey_width = frame_width;
        self.parameters.monkey_height = frame_height;
        self.parameters.width_to_draw = frame_width * MONKEY_SCALING;
        self.parameters.height_to_draw = frame_height * MONKEY_SCALING;
        self.parameters.n_frames_running = n_frames;
        self.parameters.loop_duration = time_loop;
        self.parameters.advancement_px = advance_loop * self.parameters.width_to_draw;
        self.parameters.width_screen = crate::CANVAS_REF_WIDTH;
        self.parameters.height_screen = crate::CANVAS_REF_WIDTH;
        self.parameters.number_vertical = (height / frame_height).floor() as i32;
        self.parameters.number_horizontal = (width / frame_width).floor() as i32;
        self.parameters.v = self.parameters.advancement_px / time_loop;
    }

    fn calculate_hor_vert_frame(&self, frame_number: i32) -> (f32, f32) {
        let vertical = frame_number / self.parameters.number_horizontal;
        let horizontal = frame_number % self.parameters.number_horizontal;
        (horizontal as f32, vertical as f32)
    }

    pub fn onclick(&mut self, new_x: f32) {
        if new_x != self.x_pos {
            self.desired_x = new_x;
            let new_is_left = new_x < self.x_pos;
            if !self.is_running || new_is_left != self.is_left {
                self.start_x = self.x_pos;
            }
            self.is_running = true;
            self.is_left = new_is_left;
        }
    }

    pub fn on_animation_frame(&mut self, deltat: f32) {
        //crate::log(&format!("deltat {:?}", deltat));
        if self.is_running {
            self.calculate_new_x(deltat);
        }
        let frame_number;
        if self.is_running {
            frame_number = self.calculate_frame_number();
        } else {
            frame_number = self.parameters.n_frames_running;
        }
        let (monkey_frame_hor, monkey_frame_vert) = self.calculate_hor_vert_frame(frame_number);
        self.refresh_arrays(monkey_frame_hor, monkey_frame_vert);
    }

    fn stop_monkey(&mut self) {
        self.x_pos = self.desired_x;
        self.start_x = self.x_pos;
        self.is_running = false;
    }

    fn calculate_new_x(&mut self, deltat: f32) {
        if self.is_left {
            let new_x = self.x_pos - deltat * self.parameters.v;
            if new_x <= self.desired_x {
                self.stop_monkey();
            } else {
                self.x_pos = new_x;
            }
        } else {
            let new_x = self.x_pos + deltat * self.parameters.v;
            if new_x >= self.desired_x {
                self.stop_monkey();
            } else {
                self.x_pos = new_x;
            }
        }
    }

    fn calculate_frame_number(&self) -> i32 {
        let total_deltax = (self.start_x - self.x_pos).abs();
        let one_cycle_delta = (total_deltax / self.parameters.advancement_px).fract();
        let frame_f = (one_cycle_delta * (self.parameters.n_frames_running as f32)).floor();
        frame_f as i32
    }

    fn refresh_arrays(&mut self, monkey_frame_hor: f32, monkey_frame_vert: f32) {
        let x_left = (self.x_pos - self.parameters.width_to_draw / 2.0) * 2.0 / self.parameters.width_screen - 1.0;
        let x_right = (self.x_pos + self.parameters.width_to_draw / 2.0) * 2.0 / self.parameters.width_screen - 1.0;
        let y_bottom = self.y_pos * 2.0 / self.parameters.height_screen - 1.0;
        let y_top = (self.y_pos + self.parameters.height_to_draw) * 2.0 / self.parameters.height_screen - 1.0;
        self.vertex_arr = [x_left,  y_bottom,
                           x_right, y_bottom,
                           x_left,  y_top,
                           x_right, y_bottom,
                           x_right, y_top,
                           x_left,  y_top,];
        let mut x_coord_left = monkey_frame_hor * self.parameters.monkey_width / self.parameters.texture_width; 
        let mut x_coord_right = ((monkey_frame_hor + 1.0) * self.parameters.monkey_width - 1.0) / self.parameters.texture_width; 
        let y_coord_bottom = monkey_frame_vert * self.parameters.monkey_height / self.parameters.texture_height; 
        let y_coord_top = ((monkey_frame_vert + 1.0) * self.parameters.monkey_height - 1.0) / self.parameters.texture_height;
        if self.is_left {
            (x_coord_left, x_coord_right) = (x_coord_right, x_coord_left);
        }
        self.texture_arr = [x_coord_left,  y_coord_bottom,
                           x_coord_right, y_coord_bottom,
                           x_coord_left,  y_coord_top,
                           x_coord_right, y_coord_bottom,
                           x_coord_right, y_coord_top,
                           x_coord_left,  y_coord_top,];
    }
}