extern crate clap_verbosity_flag;
extern crate dirs;
extern crate image;
extern crate imageproc;
extern crate palette_extract;
extern crate strum_macros;
mod err;
mod utils;

use crate::err::Error;
use crate::utils::{add_grid_to_image, colour2rgb, set_closest_colour};
use image::imageops::blur;
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::DynamicImage;
use image::Rgb;
use palette_extract::{get_palette_with_options, MaxColors, PixelEncoding, PixelFilter, Quality};
use std::process;
use std::{fs, path::PathBuf};
use structopt::StructOpt;
use strum_macros::EnumString;

const WELCOME_MSG: &str = "
'||' '|.   '|' |''||''|     |     '||''|.    .|'''.|  '||'     |     
 ||   |'|   |     ||       |||     ||   ||   ||..  '   ||     |||    
 ||   | '|. |     ||      |  ||    ||''|'     ''|||.   ||    |  ||   
 ||   |   |||     ||     .''''|.   ||   |.  .     '||  ||   .''''|.  
.||. .|.   '|    .||.   .|.  .||. .||.  '|' |'....|'  .||. .|.  .||. 
";

#[derive(EnumString)]
enum ImageType {
    #[strum(serialize = "original")]
    Original,
    #[strum(serialize = "processed")]
    Processed,
}

struct Image {
    _image_type: ImageType,
    _path: PathBuf,
    image: DynamicImage,
}

struct Instructions {
    /// This is the text with the instructions.
    /// TODO: Add functionality so that these instructions can
    /// be read from a given point within the project.
    _text: String,
}

/// Represents a project instance. This holds information about
/// its name, where its data is stored, and the images
struct Project {
    /// The name of this project.
    name: String,
    /// The path to the project files.
    path: PathBuf,
    /// Original image.
    original_image: Option<Image>,
    /// Processed image.
    processed_image: Option<Image>,
    /// The instructions for this crochet project.
    _instructions: Option<Instructions>,
}

impl Project {
    /// Creates a new instance of a project, with a given name
    /// and path.
    fn new(name: &String) -> Result<Project, Error> {
        // Determine the home directory.
        let mut path: PathBuf;
        match dirs::home_dir() {
            Some(ppb) => {
                path = ppb;
            }
            None => {
                return Err(Error::External("Could not determine home dir".to_string()));
            }
        }
        path.push(".intarsia/");
        if !path.as_path().exists() {
            fs::create_dir(path.as_path()).map_err(|e| Error::External(e.to_string()))?;
        }
        path.push(format!("{}/", name));
        // Throw error if path already exists
        if path.as_path().exists() {
            return Err(Error::ExistsAlready.into());
        }
        fs::create_dir(path.as_path()).map_err(|e| Error::External(e.to_string()))?;
        Ok(Project {
            name: name.to_string(),
            path,
            original_image: None,
            processed_image: None,
            _instructions: None,
        })
    }

    /// This function removes the current project, if it indeed
    /// exists already.
    fn remove_project(&self) {
        if self.path.exists() {
            fs::remove_dir_all(self.path.as_path()).unwrap();
        }
    }

    /// Reads a new image, given a file-path string, and saves
    /// it in the project folder under the name `original.png`.
    fn read_image(&mut self, image: String) -> Result<(), Error> {
        let image = ImageReader::open(&image)
            .map_err(|e| Error::External(e.to_string()))?
            .decode()
            .map_err(|e| Error::External(e.to_string()))?;
        let mut path: PathBuf = self.path.clone();
        path.push("original.png");
        image
            .save(&path)
            .map_err(|e| Error::External(e.to_string()))?;
        self.original_image = Some(Image {
            _image_type: ImageType::Original,
            _path: path,
            image,
        });
        Ok(())
    }

    // Reduces the number of colours (i.e. "quantizes") an image
    // with the number of desired colours and image dimensions
    // as parameters.
    fn reduce_colours(&self, image: DynamicImage, colours: u8) -> Result<DynamicImage, Error> {
        let mut input_path: PathBuf = self.path.clone();
        input_path.push("quantization_input.png");
        image
            .save(&input_path)
            .map_err(|e| Error::External(e.to_string()))?;
        let image = ImageReader::open(&input_path)
            .map_err(|e| Error::External(e.to_string()))?
            .decode()
            .map_err(|e| Error::External(e.to_string()))?;
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
            .map_err(|e| Error::External(e.to_string()))?;
        let mut output_path: PathBuf = self.path.clone();
        output_path.push("quantized.png");
        let output_image = ImageReader::open(&output_path)
            .map_err(|e| Error::External(e.to_string()))?
            .decode()
            .map_err(|e| Error::External(e.to_string()))?;
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
    /// Finally, it stores the output in the project path.
    fn transform_image(
        &mut self,
        output_width: u32,
        output_height: u32,
        colours: u8,
    ) -> Result<(), Error> {
        let mut image = self.original_image.as_ref().unwrap().image.clone();
        let width = image.width();
        let height = image.height();
        image = DynamicImage::ImageRgba8(blur(&image, 3.0));
        image = image.resize_exact(output_width, output_height, FilterType::Nearest);
        let mut path: PathBuf = self.path.clone();
        path.push("resized_down.png");
        image
            .save(&path)
            .map_err(|e| Error::External(e.to_string()))?;
        image = image.resize_exact(width, height, FilterType::Nearest);
        let mut path: PathBuf = self.path.clone();
        path.push("resized_up.png");
        image
            .save(&path)
            .map_err(|e| Error::External(e.to_string()))?;
        image = self
            .reduce_colours(image, colours)
            .map_err(|e| Error::External(e.to_string()))?;
        add_grid_to_image(&mut image, output_width, output_height);
        let mut path: PathBuf = self.path.clone();
        path.push("processed.png");
        image
            .save(&path)
            .map_err(|e| Error::External(e.to_string()))?;
        self.processed_image = Some(Image {
            _image_type: ImageType::Processed,
            _path: path,
            image,
        });
        Ok(())
    }
}

#[derive(StructOpt)]
enum SubCommand {
    /// Create a new project.
    New {
        /// The name of this new project.
        name: String,
        /// The (absolute) path to the image that will serve as
        /// the basis for this new projct.
        #[structopt(short, long)]
        image: String,
        /// The width of the output image.
        #[structopt(short, long)]
        width: u32,
        /// The height of the output image.
        #[structopt(short, long)]
        height: u32,
        /// The number of colours in the output image.
        #[structopt(short, long)]
        colours: u8,
    },
    Remove {
        _name: String,
    },
    Show {
        _name: String,
        #[structopt(short, long)]
        _type: String,
    },
    Instructions {
        _name: String,
    },
}

#[derive(StructOpt)]
#[structopt(about = WELCOME_MSG)]
struct Intarsia {
    #[structopt(subcommand)]
    cmd: SubCommand,
}

fn main() {
    // Run subcommand
    match Intarsia::from_args().cmd {
        SubCommand::New {
            name,
            image,
            width,
            height,
            colours,
        } => {
            let project = Project::new(&name);
            match project {
                Err(err) => {
                    eprintln!("Could not create new project. Error: {}", err);
                    process::exit(1);
                }
                Ok(mut project) => {
                    match project.read_image(image) {
                        Err(err) => {
                            eprintln!("Could not read image. Error: {}", err);
                            project.remove_project();
                            process::exit(1);
                        }
                        _ => (),
                    }
                    match project.transform_image(width, height, colours) {
                        Err(err) => {
                            eprintln!("Could not transform image. Error: {}", err);
                            project.remove_project();
                            process::exit(1);
                        }
                        _ => (),
                    }
                    println!(
                        "Succsessfully created project {}, stored at {:?}",
                        project.name, project.path
                    );
                }
            }
        }
        _ => {
            println!("Not implemented.")
        }
    }
}
