use clap::{Parser, Subcommand};
use log::error;

use niftymic_gui::config::Config;
use niftymic_gui::niftymic::*;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<String>,

    #[arg(short, long, value_name = "DIRECTORY")]
    working_directory: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    ConvertDicom { archive_path: String },
    Pipeline { archive_path: String },
    GenerateMasks { working_directory: String },
    Reconstruct { working_directory: String },
    ConvertNifti { working_directory: String },
}

fn execute_cmdline() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::new(cli.config)?;

    match &cli.command {
        Commands::ConvertDicom { archive_path } => {
            let niftymic = NiftyMic::new(archive_path, &config)?;
            niftymic.convert_dicom_to_nifti()
        }
        Commands::Pipeline { archive_path } => {
            let niftymic = NiftyMic::new(archive_path, &config)?;
            niftymic.convert_dicom_to_nifti()?;
            niftymic.generate_masks_from_nifti()?;
            niftymic.reconstruct(Options::default())?;
            let result = niftymic.convert_nifti_to_dicom()?;
            log::info!("Result: {}", result);
            Ok(())
        }
        Commands::GenerateMasks { working_directory } => {
            let niftymic = NiftyMic::from_working_directory(working_directory, &config)?;
            niftymic.generate_masks_from_nifti()
        }
        Commands::Reconstruct { working_directory } => {
            let niftymic = NiftyMic::from_working_directory(working_directory, &config)?;
            let options = Options::default();
            niftymic.reconstruct(options)
        }
        Commands::ConvertNifti { working_directory } => {
            let niftymic = NiftyMic::from_working_directory(working_directory, &config)?;
            let result = niftymic.convert_nifti_to_dicom()?;
            log::info!("Result: {}", result);
            Ok(())
        }
    }
}

fn main() {
    pretty_env_logger::init();
    match execute_cmdline() {
        Ok(_) => println!("Terminated with no errors"),
        Err(err) => error!("{}", err.to_string()),
    }
}
