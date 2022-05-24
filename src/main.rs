extern crate clap_verbosity_flag;
extern crate dirs;
extern crate image;
extern crate imageproc;
extern crate palette_extract;
extern crate strum_macros;
mod err;
mod utils;

use crate::err::Error;
use crate::utils::{add_grid_to_image, colour2rgb};
use color_reduction::image::open;
use color_reduction::image::Rgb;
use color_reduction::reduce_colors;
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use image::DynamicImage;
use palette_extract::get_palette_rgb;
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
    image_type: ImageType,
    path: PathBuf,
    image: DynamicImage,
}

struct Instructions {
    /// This is the text with the instructions.
    /// TODO: Add functionality so that these instructions can
    /// be read from a given point within the project.
    text: String,
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
    instructions: Option<Instructions>,
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
            instructions: None,
        })
    }

    /// This function removes the current project, if it indeed
    /// exists already.
    fn remove_project(&self) {
        if self.path.exists() {
            fs::remove_dir_all(self.path.as_path()).unwrap();
        }
    }

    fn read_image(&mut self, image: String) -> Result<(), Error> {
        let image = ImageReader::open(&image)
            .map_err(|e| Error::External(e.to_string()))?
            .decode()
            .map_err(|e| Error::External(e.to_string()))?;
        let mut path: PathBuf = self.path.clone();
        path.push("original.jpg");
        image
            .save(&path)
            .map_err(|e| Error::External(e.to_string()))?;
        self.original_image = Some(Image {
            image_type: ImageType::Original,
            path,
            image,
        });
        Ok(())
    }

    fn reduce_colours(&self, image: DynamicImage, colours: u8) -> Result<DynamicImage, Error> {
        let mut input_path: PathBuf = self.path.clone();
        input_path.push("resized.jpg");
        image
            .save(&input_path)
            .map_err(|e| Error::External(e.to_string()))?;
        let image = open(input_path).map_err(|e| Error::External(e.to_string()))?;
        let image_bytes = image.as_bytes();
        let colour_palette = get_palette_rgb(image_bytes);
        let palette: Vec<Rgb<u8>> = colour_palette.iter().map(|x| colour2rgb(*x)).collect();
        let quantized_image = reduce_colors(image, &palette[0..(colours as usize)]);
        let mut output_path: PathBuf = self.path.clone();
        output_path.push("quantized.jpg");
        quantized_image
            .save(&output_path)
            .map_err(|e| Error::External(e.to_string()))?;
        let output_image = ImageReader::open(&output_path)
            .map_err(|e| Error::External(e.to_string()))?
            .decode()
            .map_err(|e| Error::External(e.to_string()))?;
        Ok(output_image)
    }

    fn transform_image(
        &mut self,
        output_width: u32,
        output_height: u32,
        colours: u8,
    ) -> Result<(), Error> {
        let mut image = self.original_image.as_ref().unwrap().image.clone();
        let width = image.width();
        let height = image.height();
        image = self
            .reduce_colours(image, colours)
            .map_err(|e| Error::External(e.to_string()))?;
        image = image.resize_exact(output_width, output_height, FilterType::Nearest);
        image = image.resize_exact(width, height, FilterType::Nearest);
        add_grid_to_image(&mut image, output_width, output_height);
        let mut path: PathBuf = self.path.clone();
        path.push("processed.jpg");
        image
            .save(&path)
            .map_err(|e| Error::External(e.to_string()))?;
        self.processed_image = Some(Image {
            image_type: ImageType::Processed,
            path,
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
        name: String,
    },
    Show {
        name: String,
        #[structopt(short, long)]
        r#type: String,
    },
    Instructions {
        name: String,
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
