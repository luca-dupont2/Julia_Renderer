use macroquad::{prelude::*, miniquad::window::screen_size};
use num_complex::Complex;
mod cache;
use cache::{Cache, View};
use scarlet::colormap::{ListedColorMap};

// Define constants
const MAX_ITERS : usize = 250;
const START_BOUNDARY : f64 = 2.;
const ESCAPE_RANGE : f64 = START_BOUNDARY;
const ZOOM_FACTOR: f64 = 1.1;
const SCROLL_FACTOR: f64 = 50.;
const TRANSPARENT_GREY: Color = Color{r : 220., g : 220., b : 220., a : 0.2};
const POST_ESCAPE_ITERATIONS : usize = 2;
const FONT_SIZE : f32 = 30.0;
const TEXT_X : f32 = 10.0;
const C_TEXT_Y : f32 = 30.0;
const FPS_TEXT_Y : f32 = 60.;
const WHITE_GRADIENT_RANGE : usize = 5; // Number of white-like colors to be appended to the color map
const GRADIENT_START_VAL : f64 = 0.75;
const INCREMENT : f64 = (1.0-GRADIENT_START_VAL) / WHITE_GRADIENT_RANGE as f64;

// Map value from one range to another
fn map_value(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    // Normalize value to the range [0, 1]
    let normalized_value = (value - from_min) / (from_max - from_min);

    // Scale normalized value to the new range
    normalized_value * (to_max - to_min) + to_min
}

// Function recursively applied to z
fn f(mut z : Complex<f64>, c : Complex<f64>) -> f64 {
    let mut post_iters = 0;
    for i in 0..MAX_ITERS {
        // f_(n+1)(z) = f_(n)(z)^2 + c
        z = z*z + c;
        let norm = z.norm();

        if norm > ESCAPE_RANGE && post_iters <= POST_ESCAPE_ITERATIONS {
            // Let it run a couple more iterations after it has escaped
            post_iters += 1;
        } else if norm > ESCAPE_RANGE {
            // Subtract log2(log2(|z|)) from the iterations for smoothness
            return i as f64 - norm.log2().log2()
        }
    }
    // Return the max iterations if it hasn't escaped
    MAX_ITERS as f64
}

// Custom color mapping from color vector
fn transform(color_map : &Vec<[f64; 3]>, val : f64) -> Color  {
    let map_max_index = color_map.len() as f64 - 1.;

    let index = (map_max_index*val).floor() as usize;

    let color_vals = color_map[index];

    Color {r : color_vals[0] as f32, g : color_vals[1] as f32, b : color_vals[2] as f32, a : 1.}
}

#[macroquad::main("Julia Set Simulation")]
async fn main() {
    // Initialize window and image
    let (width, height) = screen_size();

    let mut image = Image::gen_image_color(width as u16, height as u16, BLACK);
    let mut texture = Texture2D::from_image(&image);
    let mut cache = Cache::new();

    let (mut w, mut h) = (image.width() as f64, image.height() as f64);

    // Keep screen coordinates separate from the mapped Julia parameter.
    let mut last_mouse = (-1., -1.);
    let (mut mx, mut my) = (0.,0.);
    let mut last_c = None;

    // If the c value changes or not
    let mut freeze = false;

    // Initial graph boundaries
    let mut boundary = START_BOUNDARY;

    // Graph translation offsets
    let mut x_offset = 0.;
    let mut y_offset = 0.;

    // Initialize color map
    let mut magma_color_map : Vec<[f64; 3]> = ListedColorMap::magma().vals;
    // Add a gradient at the end of the color map, going from yellow to white
    for i in 0..WHITE_GRADIENT_RANGE {
        let b = GRADIENT_START_VAL + (i as f64) * INCREMENT; // Generate values between 0.75 and 1.0
        let color = [1.0, 1.0, b]; // Create an array of the rgb values
        magma_color_map.push(color);
    }

    loop {
        // Check for window resizing
        let (new_width, new_height) = screen_size();
        if new_width as usize != image.width() || new_height as usize != image.height() {
            image = Image::gen_image_color(new_width as u16, new_height as u16, WHITE);
            texture = Texture2D::from_image(&image);

            (w, h) = (image.width() as f64, image.height() as f64);
        }

        // Check for mouse movement if the screen is not frozen
        let (new_mx, new_my) = mouse_position();
        if (new_mx, new_my) != last_mouse && !freeze {
            last_mouse = (new_mx, new_my);
            (mx, my) = (new_mx, new_my);
            // Map the mouse position to the range in which c lies
            (mx, my) = (map_value(mx as f64, 0., w, -START_BOUNDARY, START_BOUNDARY,) as f32, map_value(my as f64, 0., h, START_BOUNDARY, -START_BOUNDARY,) as f32);
        }

        // Handle key presses
        if is_key_down(KeyCode::Equal) {
            // Zoom in
            boundary /= ZOOM_FACTOR;
        }
        if is_key_down(KeyCode::Minus) {
            // Zoom out
            boundary *= ZOOM_FACTOR;
        }
        if is_key_down(KeyCode::Right) {
            // Scroll right
            x_offset += boundary / SCROLL_FACTOR;
        }
        if is_key_down(KeyCode::Left) {
            // Scroll left
            x_offset -= boundary / SCROLL_FACTOR;
        }
        if is_key_down(KeyCode::Down) {
            // Scroll down
            y_offset += boundary / SCROLL_FACTOR;
        }
        if is_key_down(KeyCode::Up) {
            // Scroll up
            y_offset -= boundary / SCROLL_FACTOR;
        }
        if is_key_pressed(KeyCode::Space) {
            // Toggle screen freeze
            freeze = !freeze
        }

        // Initialize c based on the mouse position
        let c = Complex::new(mx as f64, my as f64);

        // Reset background
        clear_background(BLACK);

        if last_c != Some(c) {
            cache.clear();
            last_c = Some(c);
        }
        let view = View { width: w as usize, height: h as usize, boundary, x: x_offset, y: y_offset };
        if cache.update(view, |x, y| f(Complex::new(x, y), c)) {
            for (index, value) in cache.values().enumerate() {
                let value = map_value(value, 0., MAX_ITERS as f64, 1.0, 0.0).clamp(0.0, 1.0);
                image.set_pixel((index % view.width) as u32, (index / view.width) as u32, transform(&magma_color_map, value));
            }
            texture.update(&image);
        }
        draw_texture(&texture, 0., 0., WHITE);

        // Write the current applied c value
        let c_text = &format!("c = {mx:.3} + {my:.3}i");

        // Write the current fps
        let fps = get_fps();
        let fps_text = &format!("Current fps : {fps}");

        let c_size = measure_text(c_text, None, FONT_SIZE as u16, 1.0);
        let fps_text_height = measure_text(fps_text, None, FONT_SIZE as u16, 1.0).height;

        // Draw a semi-transparent rectangle in case the text is over a black section
        draw_rectangle(0.,0.,c_size.width+TEXT_X, c_size.height+fps_text_height+C_TEXT_Y, TRANSPARENT_GREY);

        // Draw the text onto the screen
        draw_text(c_text, TEXT_X, C_TEXT_Y, FONT_SIZE, BLACK);
        draw_text(fps_text, TEXT_X, FPS_TEXT_Y, FONT_SIZE, BLACK);

        next_frame().await;
    }
}
