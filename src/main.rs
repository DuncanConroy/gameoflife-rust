use std::process::exit;
use std::thread;
use std::time::Duration;

use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;
const NEIGHBOR_LIMIT: u8 = 3;

type CellArray = [[(u8, u8); WIDTH]; HEIGHT];

fn main() {
    let mut cells = [[(0, 0); WIDTH]; HEIGHT];
    let mut generation = 0;

    cells = seed(cells, WIDTH, HEIGHT);

    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new((WIDTH * 2) as f64, (HEIGHT * 2) as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };
    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture).unwrap()
    };

    event_loop.run(move |event, _, control_flow| {
        // if generation % 100 == 0 {
        render(&cells, WIDTH, HEIGHT, generation, pixels.get_frame());
        if pixels
            .render()
            .is_err()
        {
            *control_flow = ControlFlow::Exit;
            return;
        }
        // }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }
        }

        // Update internal state and request a redraw
        cells = tick(cells, WIDTH as usize, HEIGHT as usize, generation);
        generation = generation + 1;
        // thread::sleep(Duration::from_millis(10));
    })
}

fn seed(mut cells: CellArray, width:usize, height:usize) -> CellArray {
    // cells[2][4] = (1, 0xff);
    // cells[3][4] = (1, 0xff);
    // cells[5][5] = (1, 0xff);
    // cells[5][6] = (1, 0xff);
    // cells[5][4] = (1, 0xff);
    // cells[2][3] = (1, 0xff);
    // cells[3][8] = (1, 0xff);
    // cells[4][8] = (1, 0xff);
    // cells[4][9] = (1, 0xff);

    for y in 0..height {
        for x in 0..width {
            cells[y][x] = if rand::random::<f32>() > 0.01 { (1, 0xff) } else { (0, 0) };
        }
    }

    cells
}

fn tick(mut cells: CellArray, width: usize, height: usize, generation: usize) -> CellArray {
    calculate_state(cells, width, height, generation)
}

fn calculate_state(mut cells: CellArray, width: usize, height: usize, generation: usize) -> CellArray {
    let mut has_changes = false;
    let wrap = |index: usize, amount: i16, limit: usize| if (index as i16 + amount) < 0 { limit as i16 + amount } else if index as i16 + amount >= limit as i16 { 0i16 + amount } else { index as i16 + amount } as usize;
    for y in 0..height {
        for x in 0..width {
            let x_wrapped_left = wrap(x, -1, width);
            let x_wrapped_right = wrap(x, 1, width);
            let y_wrapped_top = wrap(y, -1, height);
            let y_wrapped_bottom = wrap(y, 1, height);
            let top_left = cells[y_wrapped_top][x_wrapped_left].0;
            let top_mid = cells[y_wrapped_top][x].0;
            let top_right = cells[y_wrapped_top][x_wrapped_right].0;
            let mid_left = cells[y][x_wrapped_left].0;
            let mid_right = cells[y][x_wrapped_right].0;
            let bottom_left = cells[y_wrapped_bottom][x_wrapped_left].0;
            let bottom_mid = cells[y_wrapped_bottom][x].0;
            let bottom_right = cells[y_wrapped_bottom][x_wrapped_right].0;
            let alive_neighbors = top_left + top_mid + top_right + mid_left + mid_right + bottom_left + bottom_mid + bottom_right;
            if cells[y][x].0 == 1 {
                if alive_neighbors < NEIGHBOR_LIMIT - 1 {
                    // underpopulation
                    cells[y][x].0 = 0;
                    cells[y][x].1 = (cells[y][x].1 as f32 * 0.5) as u8;
                    has_changes = true;
                } else if alive_neighbors < NEIGHBOR_LIMIT + 1 {
                    // balanced/living
                    // no change
                } else if alive_neighbors > NEIGHBOR_LIMIT {
                    // overpopulation
                    cells[y][x].0 = 0;
                    cells[y][x].1 = (cells[y][x].1 as f32 * 0.5) as u8;
                    has_changes = true;
                }
            } else if alive_neighbors == NEIGHBOR_LIMIT {
                // reproduction
                cells[y][x].0 = 1;
                cells[y][x].1 = 0xff;
                has_changes = true;
            } else {
                cells[y][x].1 = (cells[y][x].1 as f32 * 0.5) as u8;
            }
        }
    }

    if !has_changes {
        println!("\nStable at current generation {}", generation);
        // exit(0);
    }
    cells
}

fn render(cells: &CellArray, width: usize, height: usize, generation: usize, frame_buffer: &mut [u8]) {
    for (i, pixel) in frame_buffer.chunks_exact_mut(4).enumerate() {
        let x = i % width;
        let y = i / width;

        let rgba = [cells[y][x].1; 4];

        pixel.copy_from_slice(&rgba);
    }
}
