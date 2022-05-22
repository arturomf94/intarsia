extern crate image;
extern crate imageproc;

use image::DynamicImage;
use imageproc::drawing::draw_line_segment_mut;

/// Function to draw a grid over the pixels of an image
pub fn add_grid_to_image(image: &mut DynamicImage) {
    let width = image.width();
    let height = image.height();
    let pixel_width_size = width as f32 / 50f32;
    let pixel_height_size = height as f32 / 50f32;
    println!("{} {}", pixel_width_size, pixel_height_size);
    let black = image::Rgba([0u8, 0u8, 0u8, 255u8]);
    // Draw horizontal lines
    for i in 0..50 {
        draw_line_segment_mut(
            image,
            (0 as f32, (i as f32 * pixel_height_size)),
            (width as f32, (i as f32 * pixel_height_size)),
            black,
        );
    }
    // Draw vertical lines
    for i in 0..50 {
        draw_line_segment_mut(
            image,
            ((i as f32 * pixel_width_size), 0 as f32),
            ((i as f32 * pixel_width_size), height as f32),
            black,
        );
    }
}
