use std::path::PathBuf;
use std::time::Instant;
use macroquad::miniquad::window::set_window_size;
use macroquad::prelude::*;
use native_dialog::FileDialog;
use crate::space::{io, Space};

mod space;
mod test;

const CELL_SIZE : f32 = 28. ; // 20
const START_GRID_X_DIM: u16 = 25;
const START_GRID_Y_DIM: u16 = 25;

const ASCII_ART: &str = "
                                                ██████╗ ██╗   ██╗███████╗████████╗██╗   ██╗
                                                ██╔══██╗██║   ██║██╔════╝╚══██╔══╝╚██╗ ██╔╝
                                                ██████╔╝██║   ██║███████╗   ██║    ╚████╔╝
                                                ██╔══██╗██║   ██║╚════██║   ██║     ╚██╔╝
                                                ██║  ██║╚██████╔╝███████║   ██║      ██║
                                                ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝      ╚═╝

 ██████╗███████╗██╗     ██╗     ██╗   ██╗██╗      █████╗ ██████╗      █████╗ ██╗   ██╗████████╗ ██████╗ ███╗   ███╗ █████╗ ████████╗ █████╗
██╔════╝██╔════╝██║     ██║     ██║   ██║██║     ██╔══██╗██╔══██╗    ██╔══██╗██║   ██║╚══██╔══╝██╔═══██╗████╗ ████║██╔══██╗╚══██╔══╝██╔══██╗
██║     █████╗  ██║     ██║     ██║   ██║██║     ███████║██████╔╝    ███████║██║   ██║   ██║   ██║   ██║██╔████╔██║███████║   ██║   ███████║
██║     ██╔══╝  ██║     ██║     ██║   ██║██║     ██╔══██║██╔══██╗    ██╔══██║██║   ██║   ██║   ██║   ██║██║╚██╔╝██║██╔══██║   ██║   ██╔══██║
╚██████╗███████╗███████╗███████╗╚██████╔╝███████╗██║  ██║██║  ██║    ██║  ██║╚██████╔╝   ██║   ╚██████╔╝██║ ╚═╝ ██║██║  ██║   ██║   ██║  ██║
 ╚═════╝╚══════╝╚══════╝╚══════╝ ╚═════╝ ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝    ╚═╝  ╚═╝ ╚═════╝    ╚═╝    ╚═════╝ ╚═╝     ╚═╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝
                                                                                                                                            ";

fn window_conf() -> Conf {
    Conf {
        window_title: "Cellular Automata".to_string(),
        window_width: START_GRID_X_DIM as i32 * CELL_SIZE as i32,
        window_height: START_GRID_Y_DIM as i32 * CELL_SIZE as i32,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {

    println!("{}", ASCII_ART);
    let mut run: bool = false;
    let mut space = Space::new(START_GRID_X_DIM, START_GRID_Y_DIM);
    let mut time_step_start: usize = 0;
    let mut settings = Settings::new(screen_width(), screen_height());
    let time = Instant::now();
    loop {
        let time_step_current = space.displayed_time;
        clear_background(BLACK);
        if run {
            space.compute_conways_game_of_life_multithreaded();
            space.save_state(time_step_current + 1);
        }
        let current_screen_width = screen_width();
        let current_screen_height = screen_height();
        // Resizing Window
        {
            if current_screen_width != settings.screen_width || current_screen_height != settings.screen_height {
                let mut resized_space = Space::new((current_screen_width / CELL_SIZE) as u16, (current_screen_height / CELL_SIZE) as u16);
                for alive_cell in space.get_alive_cells() {
                    let res_cell = resized_space.get_cell_mut(alive_cell.x, alive_cell.y);
                    if res_cell.is_ok() {
                        res_cell.unwrap().revive();
                    }
                }
                space = resized_space;
                settings.screen_width = current_screen_width;
                settings.screen_height = current_screen_height;
            }
        }
        draw(&mut space, settings.tracing, &settings.color, &settings.fps, settings.fps_is_on);
        if settings.fps_is_on {
            settings.compute_fps(time);
        }
        let mouse_position: (f32, f32) = mouse_position();
        if settings.is_active {
            settings.draw(current_screen_width, current_screen_height);
            process_red_slider(&mut settings, mouse_position);
            process_green_slider(&mut settings, mouse_position);
            process_blue_slider(&mut settings, mouse_position);
        } else {
            // Mouse Control
            {
                if is_mouse_button_pressed(MouseButton::Left) {
                    let cell_x = (mouse_position.0 / CELL_SIZE).floor() as u16;
                    let cell_y = (mouse_position.1 / CELL_SIZE).floor() as u16;
                    if cell_x < space.x_dim() && cell_y < space.y_dim() {
                        space.get_cell_mut(cell_x, cell_y).unwrap().revive();
                        settings.dragging = true;
                    }
                }
                if is_mouse_button_pressed(MouseButton::Right) {
                    let cell_x = (mouse_position.0 / CELL_SIZE).floor() as u16;
                    let cell_y = (mouse_position.1 / CELL_SIZE).floor() as u16;
                    if cell_x < space.x_dim() && cell_y < space.y_dim() {
                        space.get_cell_mut(cell_x, cell_y).unwrap().kill();
                        settings.dragging = true;
                    }
                }
                if settings.dragging {
                    let cell_x = (mouse_position.0 / CELL_SIZE).floor() as usize;
                    let cell_y = (mouse_position.1 / CELL_SIZE).floor() as usize;
                    if cell_x < space.x_dim() as usize && cell_y < space.y_dim() as usize {
                        if is_mouse_button_down(MouseButton::Left) {
                            space.get_cell_mut(cell_x as u16, cell_y as u16).unwrap().revive();
                        }
                        if is_mouse_button_down(MouseButton::Right) {
                            space.get_cell_mut(cell_x as u16, cell_y as u16).unwrap().kill();
                        }
                    }
                }
                if is_mouse_button_released(MouseButton::Left) || is_mouse_button_released(MouseButton::Right) {
                    settings.dragging = false;
                    if space.states_hash_map.contains_key(&(time_step_current + 1)) {
                        let mut time_key_to_remove = time_step_current + 1;
                        while space.states_hash_map.contains_key(&time_key_to_remove) {
                            space.states_hash_map.remove(&time_key_to_remove);
                            time_key_to_remove += 1;
                        }
                    }
                    space.save_state(time_step_current +1);
                }
            }
        }
        // Key Control
        {
            if is_key_pressed(KeyCode::Space) {
                if !run {
                    time_step_start = time_step_current;
                }
                run = !run;
            }
            if is_key_pressed(KeyCode::Left) {
                run = false;
                if time_step_current > 0 {
                    space.load_state(time_step_current - 1);
                }
            }
            if is_key_pressed(KeyCode::Right) {
                if space.states_hash_map.contains_key(&(time_step_current + 1)) {
                    space.load_state(time_step_current +1);
                } else {
                    space.compute_conways_game_of_life_multithreaded();
                    space.save_state(time_step_current +1);
                }
                run = false;
            }
            if is_key_pressed(KeyCode::Up) {
                if time_step_current > time_step_start {
                    space.load_state(time_step_start);
                }
                run = false;
            }
            if is_key_pressed(KeyCode::Down) {
                if time_step_current < space.states_hash_map.len() - 1 {
                    space.load_state(space.states_hash_map.len() - 1);
                }
                run = false;
            }
            if is_key_pressed(KeyCode::E) && is_key_down(KeyCode::LeftControl) {
                if let Some(path) = show_export_dialog().await {
                    io::write_to_file(path.to_str().unwrap(), &space).unwrap();
                }
            }
            if is_key_pressed(KeyCode::I) && is_key_down(KeyCode::LeftControl) {
                if let Some(path) = show_import_dialog().await {
                    space = io::read_from_file(path.to_str().unwrap()).unwrap();
                    set_window_size((space.x_dim() as f32 * CELL_SIZE) as u32, (space.y_dim() as f32 * CELL_SIZE) as u32);
                }
            }
            if is_key_pressed(KeyCode::KpAdd) && is_key_down(KeyCode::X) {
                let mut increment: u16 = 1;
                if is_key_down(KeyCode::LeftControl) {
                    increment = 10;
                }
                let mut new_space = Space::new(space.x_dim()+increment, space.y_dim());
                for alive_cell in space.get_alive_cells() {
                    new_space.get_cell_mut(alive_cell.x, alive_cell.y).unwrap().revive();
                }
                space = new_space;
                set_window_size((space.x_dim() as f32 * CELL_SIZE) as u32, (space.y_dim() as f32 * CELL_SIZE) as u32);
            }
            if is_key_pressed(KeyCode::KpSubtract) && is_key_down(KeyCode::X) {
                let mut decrement: u16 = 1;
                if is_key_down(KeyCode::LeftControl) {
                    if space.x_dim() >= 27 {
                        decrement = 10;
                    }
                }
                if space.x_dim() <= 17 {
                    decrement = 0;
                }
                let mut new_space = Space::new(space.x_dim()-decrement, space.y_dim());
                for alive_cell in space.get_alive_cells() {
                    let res = new_space.get_cell_mut(alive_cell.x, alive_cell.y);
                    if res.is_ok() {
                        res.unwrap().revive();
                    }
                }
                space = new_space;
                set_window_size((space.x_dim() as f32 * CELL_SIZE) as u32, (space.y_dim() as f32 * CELL_SIZE) as u32);
            }
            if is_key_pressed(KeyCode::KpAdd) && is_key_down(KeyCode::Z) { // English Layout
                let mut increment: u16 = 1;
                if is_key_down(KeyCode::LeftControl) {
                    increment = 10;
                }
                let mut new_space = Space::new(space.x_dim(), space.y_dim()+increment);
                for alive_cell in space.get_alive_cells() {
                    new_space.get_cell_mut(alive_cell.x, alive_cell.y).unwrap().revive();
                }
                space = new_space;
                set_window_size((space.x_dim() as f32 * CELL_SIZE) as u32, (space.y_dim() as f32 * CELL_SIZE) as u32);
            }
            if is_key_pressed(KeyCode::KpSubtract) && is_key_down(KeyCode::Z) {
                let mut decrement: u16 = 1;
                if is_key_down(KeyCode::LeftControl) {
                    if space.y_dim() > 10 {
                        decrement = 10;
                    }
                }
                if space.y_dim() == 1 {
                    decrement = 0;
                }
                let mut new_space = Space::new(space.x_dim(), space.y_dim()-decrement);
                for alive_cell in space.get_alive_cells() {
                    let res = new_space.get_cell_mut(alive_cell.x, alive_cell.y);
                    if res.is_ok() {
                        res.unwrap().revive();
                    }
                }
                space = new_space;
                set_window_size((space.x_dim() as f32 * CELL_SIZE) as u32, (space.y_dim() as f32 * CELL_SIZE) as u32);
            }
            if is_key_down(KeyCode::R) {
                if is_key_down(KeyCode::LeftControl) {
                    space.revive_random_cells(10);
                } else {
                    space.revive_random_cells(1);
                }
            }
            if is_key_released(KeyCode::R) {
                space.save_state(time_step_current +1);
            }
            if is_key_pressed(KeyCode::T) {
                settings.tracing = !settings.tracing;
            }
            if is_key_pressed(KeyCode::K) {
                space.kill_all_cells();
            }
            if is_key_pressed(KeyCode::Escape) {
                settings.is_active = !settings.is_active;
            }
            if is_key_pressed(KeyCode::F) {
                settings.fps_is_on = !settings.fps_is_on;
                if settings.fps_is_on {
                    settings.fps_time_start = time.elapsed().as_secs();
                }
            }
        }
        next_frame().await
    }
}

fn process_red_slider(settings: &mut Settings, mouse_position: (f32, f32)) {
    if settings.is_in_red_slider(mouse_position) {
        if is_mouse_button_pressed(MouseButton::Left) {
            settings.slider_red_dragging = true;
        }
    }
    if is_mouse_button_released(MouseButton::Left) {
        if settings.slider_red_dragging {
            settings.slider_red_dragging = false;
        }
    }
    if settings.slider_red_dragging && is_mouse_button_down(MouseButton::Left) {
        if mouse_position.0 >= settings.bar_x_position && mouse_position.0 <= settings.bar_x_position + settings.color_bar_width {
            let relative_position: f32 = mouse_position.0 - settings.bar_x_position;
            let float_value: f32 = relative_position / settings.color_bar_width;
            settings.color.0 = float_value;
        }
    }
}

fn process_green_slider(settings: &mut Settings, mouse_position: (f32, f32)) {
    if settings.is_in_green_slider(mouse_position) {
        if is_mouse_button_pressed(MouseButton::Left) {
            settings.slider_green_dragging = true;
        }
    }
    if is_mouse_button_released(MouseButton::Left) {
        if settings.slider_green_dragging {
            settings.slider_green_dragging = false;
        }
    }
    if settings.slider_green_dragging && is_mouse_button_down(MouseButton::Left) {
        if mouse_position.0 >= settings.bar_x_position && mouse_position.0 <= settings.bar_x_position + settings.color_bar_width {
            let relative_position: f32 = mouse_position.0 - settings.bar_x_position;
            let float_value: f32 = relative_position / settings.color_bar_width;
            settings.color.1 = float_value;
        }
    }
}

fn process_blue_slider(settings: &mut Settings, mouse_position: (f32, f32)) {
    if settings.is_in_blue_slider(mouse_position) {
        if is_mouse_button_pressed(MouseButton::Left) {
            settings.slider_blue_dragging = true;
        }
    }
    if is_mouse_button_released(MouseButton::Left) {
        if settings.slider_blue_dragging {
            settings.slider_blue_dragging = false;
        }
    }
    if settings.slider_blue_dragging && is_mouse_button_down(MouseButton::Left) {
        if mouse_position.0 >= settings.bar_x_position && mouse_position.0 <= settings.bar_x_position + settings.color_bar_width {
            let relative_position: f32 = mouse_position.0 - settings.bar_x_position;
            let float_value: f32 = relative_position / settings.color_bar_width;
            settings.color.2 = float_value;
        }
    }
}

fn draw(space: &mut Space, tracing: bool, color: &(f32, f32, f32), fps: &u64, fps_is_on: bool) {
    if tracing {
        for cell in space.get_cells_with_energy() {
            let color = Color::new(color.0, color.1, color.2, cell.get_state() as f32 / 255.);
            draw_rectangle(cell.x as f32 * CELL_SIZE, cell.y as f32 * CELL_SIZE, CELL_SIZE, CELL_SIZE, color);
        }
    } else {
        for cell in space.get_alive_cells() {
            let color = Color::new(color.0, color.1, color.2, 1.);
            draw_rectangle(cell.x as f32 * CELL_SIZE, cell.y as f32 * CELL_SIZE, CELL_SIZE, CELL_SIZE, color);
        }
    }
    if fps_is_on {
        draw_text(&*fps.to_string(), 20., 20., 20., WHITE);
    }
}

async fn show_export_dialog() -> Option<PathBuf>{
    let dialog_window = FileDialog::new()
        .set_title("Export Grid")
        .set_location(std::env::current_dir().unwrap().as_path())
        .set_filename("export.space")
        .show_save_single_file();
    match dialog_window {
        Ok(Some(path)) => Some(path),
        _ => None,
    }
}

async fn show_import_dialog() -> Option<PathBuf>{
    let dialog_window = FileDialog::new()
        .set_title("Import Grid")
        .set_location(std::env::current_dir().unwrap().as_path())
        .set_filename("export.space")
        .show_open_single_file();
    match dialog_window {
        Ok(Some(path)) => Some(path),
        _ => None,
    }
}

struct Settings {
    screen_width: f32,
    screen_height: f32,
    dragging: bool,
    tracing: bool,
    is_active: bool,
    settings_width: f32,
    settings_height: f32,
    color: (f32, f32, f32),
    color_bar_width: f32,
    color_bar_height: f32,
    slider_width: f32,
    slider_height: f32,
    slider_red_position: (f32, f32),
    slider_red_dragging: bool,
    slider_green_position: (f32, f32),
    slider_green_dragging: bool,
    slider_blue_position: (f32, f32),
    slider_blue_dragging: bool,
    bar_x_position: f32,
    fps: u64,
    fps_is_on: bool,
    fps_counter: u64,
    fps_time_start: u64,
}

impl Settings {
    fn new(screen_width: f32, screen_height: f32) -> Settings {
        Settings {
            screen_width,
            screen_height,
            dragging: false,
            tracing: false,
            is_active: false,
            settings_width: 300.,
            settings_height: 100.,
            color: (0.05, 0.15, 1.), // (R, G, B)
            color_bar_width: 200.,
            color_bar_height: 20.,
            slider_width: 10.,
            slider_height: 24.,
            slider_red_position: (0.0, 0.0),
            slider_red_dragging: false,
            slider_green_position: (0.0, 0.0),
            slider_green_dragging: false,
            slider_blue_position: (0.0, 0.0),
            slider_blue_dragging: false,
            bar_x_position: 0.0,
            fps: 0,
            fps_is_on: false,
            fps_counter: 0,
            fps_time_start: 0,
        }
    }
    fn get_position(&self, current_width: f32, current_height: f32) -> (f32, f32) {
        (0.5 * current_width - 0.5 * self.settings_width, 0.5 * current_height - 0.5 * self.settings_height)
    }

    fn draw(&mut self, current_width: f32, current_height: f32) {
        let position: (f32, f32) = self.get_position(current_width, current_height);
        draw_rectangle(position.0, position.1, self.settings_width, self.settings_height, WHITE);
        self.bar_x_position = position.0 + 20.;
        let red_bar_position = (self.bar_x_position, position.1 + 10.);
        draw_rectangle(red_bar_position.0, red_bar_position.1, self.color_bar_width, self.color_bar_height, RED);
        self.slider_red_position = (
            red_bar_position.0 + self.color.0 * self.color_bar_width - self.slider_width/2.,
            red_bar_position.1 + self.color_bar_height/2. - self.slider_height/2.,
        );
        draw_rectangle(
            self.slider_red_position.0, self.slider_red_position.1,
            self.slider_width, self.slider_height, BLACK
        );
        let green_bar_position: (f32, f32) = (self.bar_x_position, position.1 + 20. + self.color_bar_height);
        draw_rectangle(green_bar_position.0, green_bar_position.1, self.color_bar_width, self.color_bar_height, GREEN);
        self.slider_green_position = (
            green_bar_position.0 + self.color.1 * self.color_bar_width - self.slider_width/2.,
            green_bar_position.1 + self.color_bar_height/2. - self.slider_height/2.,
        );
        draw_rectangle(
            self.slider_green_position.0, self.slider_green_position.1, self.slider_width, self.slider_height, BLACK
        );
        let blue_bar_position: (f32, f32) = (self.bar_x_position, position.1 + 30. + 2. * self.color_bar_height);
        draw_rectangle(blue_bar_position.0, blue_bar_position.1, self.color_bar_width, self.color_bar_height, BLUE);
        self.slider_blue_position = (
            blue_bar_position.0 + self.color.2 * self.color_bar_width - self.slider_width/2.,
            blue_bar_position.1 + self.color_bar_height/2. - self.slider_height/2.,
        );
        draw_rectangle(
            self.slider_blue_position.0, self.slider_blue_position.1,
            self.slider_width, self.slider_height, BLACK
        );
        let color: Color = Color::new(self.color.0, self.color.1, self.color.2, 1.);
        draw_rectangle(
            position.0 + 30. + self.color_bar_width,
            position.1 + 15., self.settings_width - self.color_bar_width - 40.,
            self.settings_height - 40.,
            color
        );
    }

    fn is_in_red_slider(&self, mouse_position: (f32, f32)) -> bool {
        mouse_position.0 >= self.slider_red_position.0 && mouse_position.0 <= self.slider_red_position.0 + self.slider_width &&
            mouse_position.1 >= self.slider_red_position.1 && mouse_position.1 <= self.slider_red_position.1 + self.slider_height
    }

    fn is_in_green_slider(&self, mouse_position: (f32, f32)) -> bool {
        mouse_position.0 >= self.slider_green_position.0 && mouse_position.0 <= self.slider_green_position.0 + self.slider_width &&
            mouse_position.1 >= self.slider_green_position.1 && mouse_position.1 <= self.slider_green_position.1 + self.slider_height
    }

    fn is_in_blue_slider(&self, mouse_position: (f32, f32)) -> bool {
        mouse_position.0 >= self.slider_blue_position.0 && mouse_position.0 <= self.slider_blue_position.0 + self.slider_width &&
            mouse_position.1 >= self.slider_blue_position.1 && mouse_position.1 <= self.slider_blue_position.1 + self.slider_height
    }

    fn compute_fps(&mut self, time: Instant) {
        self.fps_counter += 1;
        let time_current = time.elapsed().as_secs();
        let time_difference = time_current - self.fps_time_start;
        if time_difference >= 1 {
            self.fps = self.fps_counter / time_difference;
            self.fps_counter = 0;
            self.fps_time_start = time.elapsed().as_secs();
        }
    }
}