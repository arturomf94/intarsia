extern crate image;
extern crate imageproc;

use image::DynamicImage;
use imageproc::drawing::draw_line_segment_mut;

/// Function to draw a grid over the pixels of an image.
/// The grid size is determined by the width and height inputs,
/// and it does not necessarily match the dimensions of the
/// image (i.e. the number of pixels it has).
pub fn add_grid_to_image(image: &mut DynamicImage, grid_width: u32, grid_height: u32) {
    let width = image.width();
    let height = image.height();
    let pixel_width_size = width as f32 / grid_width as f32;
    let pixel_height_size = height as f32 / grid_height as f32;
    let black = image::Rgba([0u8, 0u8, 0u8, 255u8]);
    // Draw horizontal lines
    for i in 0..(grid_height as usize) {
        draw_line_segment_mut(
            image,
            (0 as f32, (i as f32 * pixel_height_size)),
            (width as f32, (i as f32 * pixel_height_size)),
            black,
        );
    }
    // Draw vertical lines
    for i in 0..(grid_width as usize) {
        draw_line_segment_mut(
            image,
            ((i as f32 * pixel_width_size), 0 as f32),
            ((i as f32 * pixel_width_size), height as f32),
            black,
        );
    }
}
