use crate::err::Error;
use crate::utils::{add_grid_to_image, colour2rgb, plot_image_with_axes, set_closest_colour};

use image::imageops::blur;
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::DynamicImage;
use image::Rgb;
use palette_extract::{get_palette_with_options, MaxColors, PixelEncoding, PixelFilter, Quality};
use std::process::Command;
use std::str::FromStr;
use std::{fs, path::PathBuf};

/// Represents the type of a given image. It could either be the
/// original image or the processed image.
#[derive(Debug)]
pub enum ImageType {
    Original,
    Processed,
}

/// Represents an image. This struct holds information about the
/// type, the path where it's stored and the actual data.
///
/// *Note that the image type is currently not being used.*
#[derive(Debug)]
pub struct Image {
    /// The image type of this image.
    pub _image_type: ImageType,
    /// The path where this image is located.
    pub path: PathBuf,
    /// The actual data of this image.
    pub data: DynamicImage,
}

/// Represents a project instance. This holds information about
/// its name, where its data is stored, and both the original
/// and the processed image.
#[derive(Debug)]
pub struct Intarsia {
    /// The name of this project.
    pub name: String,
    /// The path to the project files.
    pub path: PathBuf,
    /// Original image.
    pub original_image: Option<Image>,
    /// Processed image.
    pub processed_image: Option<Image>,
}

impl Intarsia {
    /// Creates a new instance of a project, given a specific
    /// file-path where the image is located. This function
    /// receives a number of parameters that determines the
    /// processed image. Namely, `output_width`: the number of
    /// squares in the x-axis; `output_height`: the number of
    /// suquares in the y-axis; `colours`: the number of colours
    /// in the output image; `add_axes`: a boolean, indicating
    /// whether to add axes to the output image or not; finally,
    /// `projects_path`: an optional variable to indicate
    /// where the project should be stored.
    pub fn new(
        name: &String,
        image_path: &str,
        output_width: u32,
        output_height: u32,
        colours: u8,
        add_axes: bool,
        projects_path: Option<&str>,
    ) -> Result<Intarsia, Error> {
        let mut path: PathBuf;
        if let Some(projects_path) = projects_path {
            path = PathBuf::from_str(projects_path)
                .map_err(|e| Error::InvalidProjectPath(e.to_string()))?;
        } else {
            path = dirs::home_dir().expect("Could not determine HOME directory!");
            path.push(".intarsia/");
        }
        if !path.as_path().exists() {
            fs::create_dir(path.as_path()).map_err(|e| Error::Generic(e.to_string()))?;
        }
        path.push(format!("{}/", name));
        // Throw error if path already exists
        if path.as_path().exists() {
            return Err(Error::ExistsAlready.into());
        }
        fs::create_dir(path.as_path()).map_err(|e| Error::Generic(e.to_string()))?;
        let mut new_obj = Intarsia {
            name: name.to_string(),
            path,
            original_image: None,
            processed_image: None,
        };
        new_obj.read_image(image_path)?;
        new_obj.transform_image(output_width, output_height, colours, add_axes)?;
        Ok(new_obj)
    }

    /// Loads an existing project, given a name. If the project
    /// does not exist yet it throws an error. The input
    /// `projects_path` is optional, and it indicates where the
    /// project is stored. By default, this is set to
    /// `.intarsia/`.
    pub fn load(name: &str, projects_path: Option<&str>) -> Result<Intarsia, Error> {
        let mut path: PathBuf;
        if let Some(projects_path) = projects_path {
            path = PathBuf::from_str(projects_path)
                .map_err(|e| Error::InvalidProjectPath(e.to_string()))?;
            path.push(format!("{}/", name));
        } else {
            path = dirs::home_dir().expect("Could not determine HOME directory!");
            path.push(".intarsia/");
            path.push(format!("{}/", name));
        }
        // If the project does not exist, throw an error.
        if !path.as_path().exists() {
            return Err(Error::DoesNotExist.into());
        }
        // Load images
        let mut original_image_path: PathBuf = path.clone();
        original_image_path.push("original.png");
        let original_image = ImageReader::open(&original_image_path)
            .map_err(|e| Error::Generic(e.to_string()))?
            .decode()
            .map_err(|e| Error::DecodingError(e.to_string()))?;
        let original_image = Some(Image {
            _image_type: ImageType::Original,
            path: original_image_path,
            data: original_image,
        });
        let mut processed_image_path: PathBuf = path.clone();
        processed_image_path.push("processed.png");
        let processed_image = ImageReader::open(&processed_image_path)
            .map_err(|e| Error::Generic(e.to_string()))?
            .decode()
            .map_err(|e| Error::DecodingError(e.to_string()))?;
        let processed_image = Some(Image {
            _image_type: ImageType::Original,
            path: processed_image_path,
            data: processed_image,
        });
        Ok(Intarsia {
            name: name.to_string(),
            path,
            original_image,
            processed_image,
        })
    }

    /// This function displays a given image (either the
    /// original or the processed) by running the `open` command
    /// in the execution environment.
    ///
    /// **Note that this function will not work if the**
    /// **execution environment does not support the `open`**
    /// **command.**
    pub fn show(self, image_type: ImageType) -> Result<(), Error> {
        let image_file: PathBuf;
        match image_type {
            ImageType::Original => {
                if let Some(image) = self.original_image {
                    image_file = image.path
                } else {
                    return Err(Error::EmptyOriginal);
                }
            }
            ImageType::Processed => {
                if let Some(image) = self.processed_image {
                    image_file = image.path
                } else {
                    return Err(Error::EmptyProcessed);
                }
            }
        }
        Command::new("open")
            .arg(image_file)
            .output()
            .map_err(|e| Error::OpenFailure(e.to_string()))?;
        Ok(())
    }

    /// This function removes the current project, if it indeed
    /// exists already.
    pub fn remove_project(&self) {
        if self.path.exists() {
            fs::remove_dir_all(self.path.as_path()).unwrap();
        }
    }

    /// Reads a new image, given a file-path string, and saves
    /// it in the project folder under the name `original.png`.
    fn read_image(&mut self, image: &str) -> Result<(), Error> {
        let image = ImageReader::open(&image)
            .map_err(|e| Error::Generic(e.to_string()))?
            .decode()
            .map_err(|e| Error::DecodingError(e.to_string()))?;
        let mut path: PathBuf = self.path.clone();
        path.push("original.png");
        image
            .save(&path)
            .map_err(|e| Error::Generic(e.to_string()))?;
        self.original_image = Some(Image {
            _image_type: ImageType::Original,
            path,
            data: image,
        });
        Ok(())
    }

    /// Reduces the number of colours (i.e. "quantizes") an image
    /// with the number of desired colours and image dimensions
    /// as parameters.
    fn reduce_colours(&self, image: DynamicImage, colours: u8) -> Result<DynamicImage, Error> {
        let mut input_path: PathBuf = self.path.clone();
        input_path.push("quantization_input.png");
        image
            .save(&input_path)
            .map_err(|e| Error::Generic(e.to_string()))?;
        let image = ImageReader::open(&input_path)
            .map_err(|e| Error::Generic(e.to_string()))?
            .decode()
            .map_err(|e| Error::DecodingError(e.to_string()))?;
        let image_bytes = image.as_bytes();
        let colour_palette = get_palette_with_options(
            &image_bytes,
            PixelEncoding::Rgba,
            Quality::default(),
            MaxColors::default(),
            PixelFilter::None,
        );
        let palette: Vec<Rgb<u8>> = colour_palette.iter().map(|x| colour2rgb(*x)).collect();
        let mut quantized_image = image.to_rgb8();
        for pixel in quantized_image.enumerate_pixels_mut() {
            set_closest_colour(pixel, &palette[0..(colours as usize)]);
        }
        let mut quantized_path: PathBuf = self.path.clone();
        quantized_path.push("quantized.png");
        quantized_image
            .save(&quantized_path)
            .map_err(|e| Error::Generic(e.to_string()))?;
        let mut output_path: PathBuf = self.path.clone();
        output_path.push("quantized.png");
        let output_image = ImageReader::open(&output_path)
            .map_err(|e| Error::Generic(e.to_string()))?
            .decode()
            .map_err(|e| Error::DecodingError(e.to_string()))?;
        Ok(output_image)
    }

    /// This function has most of the functionality. It
    /// transforms the input image by doing the following:
    /// 1. Resizing down the original image
    /// 2. Resizing the image back up to its original dimensions
    /// 3. Reducing the number of colours by calling the
    ///     `reduce_colours` function.
    /// 4. Adding a grid to the image by calling the
    ///     `add_grid_to_image` function.
    /// 5. (Optionally) Adding a Cartesian map to the processed
    ///     image.
    /// Finally, it stores the output in the project path.
    fn transform_image(
        &mut self,
        output_width: u32,
        output_height: u32,
        colours: u8,
        add_axes: bool,
    ) -> Result<(), Error> {
        let mut image = self.original_image.as_ref().unwrap().data.clone();
        let width = image.width();
        let height = image.height();
        image = DynamicImage::ImageRgba8(blur(&image, 3.0));
        image = image.resize_exact(output_width, output_height, FilterType::Nearest);
        let mut path: PathBuf = self.path.clone();
        path.push("resized_down.png");
        image
            .save(&path)
            .map_err(|e| Error::Generic(e.to_string()))?;
        image = image.resize_exact(width, height, FilterType::Nearest);
        let mut path: PathBuf = self.path.clone();
        path.push("resized_up.png");
        image
            .save(&path)
            .map_err(|e| Error::Generic(e.to_string()))?;
        image = self
            .reduce_colours(image, colours)
            .map_err(|e| Error::ColourReductionErr(e.to_string()))?;
        let mut path: PathBuf = self.path.clone();
        path.push("processed.png");
        add_grid_to_image(&mut image, output_width, output_height);
        image
            .save(&path)
            .map_err(|e| Error::Generic(e.to_string()))?;
        if add_axes {
            plot_image_with_axes(
                path.to_str().unwrap(),
                path.to_str().unwrap(),
                output_width,
                output_height,
            )
            .unwrap();
        }
        self.processed_image = Some(Image {
            _image_type: ImageType::Processed,
            path,
            data: image,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::err::Error;
    #[test]
    fn image_does_not_exist() {
        // remove project path if it already exists
        let test_project_path = PathBuf::from_str("test_data/test_file_does_not_exist/").unwrap();
        if test_project_path.exists() {
            fs::remove_dir_all(&test_project_path).unwrap();
        }
        // fail if the image does not exist
        let test_new = Intarsia::new(
            &"test_file_does_not_exist".to_owned(),
            "test_data/fake_image.png",
            5,
            5,
            2,
            true,
            Some("test_data/"),
        );
        assert!(test_new.is_err());
        assert_eq!(
            test_new.unwrap_err(),
            Error::Generic("No such file or directory (os error 2)".to_owned())
        );
    }
    #[test]
    fn create_and_remove() {
        // remove project path if it already exists
        let test_project_path = PathBuf::from_str("test_data/test_new/").unwrap();
        if test_project_path.exists() {
            fs::remove_dir_all(&test_project_path).unwrap();
        }
        // create new project
        let test_new = Intarsia::new(
            &"test_new".to_owned(),
            "test_data/test_image.png",
            5,
            5,
            2,
            true,
            Some("test_data/"),
        );
        assert!(test_new.is_ok());
        test_new.unwrap().remove_project();
        assert!(!test_project_path.exists());
    }
    #[test]
    fn load() {
        let test_load = Intarsia::load("test_load", Some("test_data/"));
        assert!(test_load.is_ok());
    }
}
