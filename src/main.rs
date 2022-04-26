extern crate clap_verbosity_flag;
use structopt::StructOpt;
// use anyhow::Result;
extern crate dirs;
extern crate image;
mod err;
use err::Error;
use image::io::Reader as ImageReader;
use std::process;
use std::{fs, path::PathBuf};

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
struct Intarsia {
    #[structopt(subcommand)]
    cmd: SubCommand,
}

/// Creates a new intarsia project, with a given name.
fn create_new_project(name: &String, image: String) -> Result<(), Error> {
    let mut project_path_buf: PathBuf;
    match dirs::home_dir() {
        Some(ppb) => {
            project_path_buf = ppb;
        }
        None => {
            return Err(Error::External("Could not determine home dir".to_string()));
        }
    }
    project_path_buf.push(".intarsia/");
    if !project_path_buf.as_path().exists() {
        fs::create_dir(project_path_buf.as_path()).map_err(|e| Error::External(e.to_string()))?;
    }
    project_path_buf.push(format!("{}/", name));
    // Throw error if path already exists
    if project_path_buf.as_path().exists() {
        return Err(Error::ExistsAlready.into());
    }
    fs::create_dir(project_path_buf.as_path()).map_err(|e| Error::External(e.to_string()))?;
    let _img = ImageReader::open(&image)
        .map_err(|e| Error::External(e.to_string()))?
        .decode()
        .map_err(|e| Error::External(e.to_string()))?;
    println!(
        "Successfully created new project {}, with the image file `{}`",
        name, image
    );
    Ok(())
}

fn clean_up_project(name: String) -> Result<(), Error> {
    println!("Remove {}", name);
    Ok(())
}

fn main() {
    // Run subcommand
    match Intarsia::from_args().cmd {
        SubCommand::New { name, image } => {
            match create_new_project(&name, image) {
                // If creation failed because the project exists
                // already, then return the error, but do not
                // clean-up the environment.
                Err(Error::ExistsAlready) => {
                    eprintln!("{}", Error::ExistsAlready);
                    process::exit(1);
                }
                Err(err) => {
                    if let Err(err) = clean_up_project(name) {
                        eprintln!("{}", err);
                        process::exit(1);
                    };
                    eprintln!("{}", err);
                    process::exit(1);
                }
                Ok(_) => (),
            }
        }
    }
}
