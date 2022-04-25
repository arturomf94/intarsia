extern crate clap_verbosity_flag;
use structopt::StructOpt;
#[macro_use]
extern crate anyhow;
use anyhow::Context;
use anyhow::Result;
extern crate dirs;
extern crate image;
use image::io::Reader as ImageReader;
use std::fs;

const WELCOME_MSG: &str = "
'||' '|.   '|' |''||''|     |     '||''|.    .|'''.|  '||'     |     
 ||   |'|   |     ||       |||     ||   ||   ||..  '   ||     |||    
 ||   | '|. |     ||      |  ||    ||''|'     ''|||.   ||    |  ||   
 ||   |   |||     ||     .''''|.   ||   |.  .     '||  ||   .''''|.  
.||. .|.   '|    .||.   .|.  .||. .||.  '|' |'....|'  .||. .|.  .||. 
";

#[derive(Debug, StructOpt)]
enum SubCommand {
    /// Create a new project.
    New {
        /// The name of this new project.
        name: String,
        /// The (absolute) path to the image that will serve as
        /// the basis for this new projct.
        #[structopt(short, long)]
        image: String,
    },
}

#[derive(StructOpt, Debug)]
#[structopt(about = WELCOME_MSG)]
struct Ds {
    #[structopt(subcommand)]
    cmd: SubCommand,
}

/// Creates a new intarsia project, with a given name.
fn create_new_project(name: &String, image: String) -> Result<()> {
    // let home_dir_opt = dirs::home_dir();
    let mut project_path_buf =
        dirs::home_dir().with_context(|| "Could not determine home directory!")?;
    project_path_buf.push(".intarsia/");
    if !project_path_buf.as_path().exists() {
        fs::create_dir(project_path_buf.as_path())
            .with_context(|| "Could not create new project.")?;
    }
    project_path_buf.push(format!("{}/", name));
    // Throw error if path already exists
    if project_path_buf.as_path().exists() {
        return Err(anyhow!("This project already exists!"));
    }
    fs::create_dir(project_path_buf.as_path()).with_context(|| "Could not create new project.")?;
    let _img = ImageReader::open(&image)
        .with_context(|| "Could not open image!")?
        .decode()
        .with_context(|| "Could not decode image :(")?;
    println!(
        "Successfully created new project {}, with the image file `{}`",
        name, image
    );
    Ok(())
}

fn clean_up_project(name: String) -> Result<()> {
    println!("Remove {}", name);
    Ok(())
}

fn main() -> Result<()> {
    // Run subcommand
    match Ds::from_args().cmd {
        SubCommand::New { name, image } => match create_new_project(&name, image) {
            Err(err) => {
                clean_up_project(name)?;
                return Err(err);
            }
            Ok(_) => (),
        },
    }
    Ok(())
}
