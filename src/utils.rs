extern crate color_reduction;
extern crate image;
extern crate imageproc;
extern crate palette_extract;

use color_reduction::image::{Pixel, Rgb};
use image::DynamicImage;
use imageproc::drawing::draw_line_segment_mut;
use palette_extract::Color;
use std::collections::HashMap;

/// Function to draw a grid over the pixels of an image.
/// The grid size is determined by the width and height inputs,
/// and it does not necessarily match the dimensions of the
/// image (i.e. the number of pixels it has).
pub fn add_grid_to_image(image: &mut DynamicImage, grid_width: u32, grid_height: u32) {
    let width = image.width();
    let height = image.height();
    // let pixel_width_size = width as f32 / grid_width as f32;
    // let pixel_height_size = height as f32 / grid_height as f32;
    let pixel_width_size = (width / grid_width) as f32;
    let pixel_height_size = (height / grid_height) as f32;
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

/// Convert a `Color` instance from `palette_extract` crate into
/// an `Rgb` instance from the `image` crate.
pub fn colour2rgb(colour: Color) -> Rgb<u8> {
    Rgb::from([colour.r, colour.g, colour.b])
}

pub fn colour_distance(c1: &Rgb<u8>, c2: &Rgb<u8>) -> f32 {
    let ch1 = c1.channels();
    let ch2 = c2.channels();
    let r1 = ch1[0] as f32;
    let r2 = ch2[0] as f32;
    let g1 = ch1[1] as f32;
    let g2 = ch2[1] as f32;
    let b1 = ch1[2] as f32;
    let b2 = ch2[2] as f32;
    f32::sqrt((r2 - r1).powf(2.0) + (g2 - g1).powf(2.0) + (b2 - b1).powf(2.0))
}

pub fn mode(numbers: &[usize]) -> Option<usize> {
    let mut counts = HashMap::new();
    numbers.iter().copied().max_by_key(|&n| {
        let count = counts.entry(n).or_insert(0);
        *count += 1;
        *count
    })
}

pub fn min_index(array: &[f32]) -> usize {
    let mut i = 0;

    for (j, &value) in array.iter().enumerate() {
        if value < array[i] {
            i = j;
        }
    }

    i
}

/// Sets a mutable reference of a pixel in an image to its
/// closest colour in a given palette reference, which is a
/// vector of candidate colours.
pub fn set_closest_colour(pixel: (u32, u32, &mut Rgb<u8>), palette: &[Rgb<u8>]) {
    let distances: Vec<f32> = palette
        .iter()
        .map(|x| colour_distance(x, pixel.2))
        .collect();
    let min_index = min_index(&distances[..]);
    // let min_index;
    *pixel.2 = palette[min_index];
}
