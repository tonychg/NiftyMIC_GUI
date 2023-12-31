use log::{debug, info};
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use thiserror::Error;
use ulid::Ulid;
use walkdir::WalkDir;

use crate::{
    archive::{create_output_archive, extract_zip},
    config::Config,
    spawn::{spawn_command, DockerWrapper},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to convert: {0}")]
    FailedToConvertDicom(String),
    #[error("IOError: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Failed to access working directory: {0}")]
    FailedToAccess(#[from] walkdir::Error),
    #[error("Failed to convert file to archive")]
    FailedToConvertFileToArchive(#[from] zip::result::ZipError),
    #[error("Not a valid zip archive: {0}")]
    NotValidZipArchive(String),
    #[error("Working directory already exists.")]
    WorkingDirectoryAlreadyExists,
    #[error("Command failed: {0}")]
    CommandFailed(String),
    #[error("ConfigError: {0}")]
    ConfigError(#[from] config::ConfigError),
    #[error("Failed to open {0}")]
    FailedToOpen(String),
    #[error("Failed to create working directory {0}")]
    FailedToCreateWorkingDirectory(String),
    #[error("Failed to start bot")]
    FailedToStartBot,
}

pub type Result<T> = std::result::Result<T, self::Error>;

pub struct Options {
    alpha: f32,
    outlier_rejection: u64,
    threshold_first: f32,
    threshold: f32,
    intensity_correction: u64,
    isotropic_resolution: f32,
    two_step_cycles: u64,
}

impl Options {
    pub fn default() -> Options {
        Options {
            alpha: 0.01,
            outlier_rejection: 1,
            threshold_first: 0.5,
            threshold: 0.85,
            intensity_correction: 1,
            isotropic_resolution: 0.8,
            two_step_cycles: 3,
        }
    }

    pub fn to_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        args.push("--alpha".to_string());
        args.push(self.alpha.to_string());
        args.push("--outlier-rejection".to_string());
        args.push(self.outlier_rejection.to_string());
        args.push("--threshold-first".to_string());
        args.push(self.threshold_first.to_string());
        args.push("--threshold".to_string());
        args.push(self.threshold.to_string());
        args.push("--intensity-correction".to_string());
        args.push(self.intensity_correction.to_string());
        args.push("--isotropic-resolution".to_string());
        args.push(self.isotropic_resolution.to_string());
        args.push("--two-step-cycles".to_string());
        args.push(self.two_step_cycles.to_string());
        args.push("--verbose".to_string());
        args.push("1".to_string());
        args
    }
}

pub struct WorkingDirectory {
    pub directory: String,
    pub path: PathBuf,
    pub archive: PathBuf,
    pub nii: PathBuf,
    pub masks: PathBuf,
    pub output_nii: PathBuf,
    pub output_dicom: PathBuf,
}

impl WorkingDirectory {
    pub fn new(path: &str) -> WorkingDirectory {
        let path = Path::new(path);
        let directory_name = path.file_name().unwrap().to_str().unwrap().to_string();

        WorkingDirectory {
            path: path.to_path_buf(),
            directory: directory_name,
            archive: path.join("archive"),
            nii: path.join("nii"),
            masks: path.join("masks"),
            output_nii: path.join("output_nii"),
            output_dicom: path.join("output_dicom"),
        }
    }

    pub fn from_archive(archive_path: &str, base_directory: &str) -> Result<WorkingDirectory> {
        debug!(
            "Creating working directory from {} {}",
            archive_path, base_directory
        );
        let base_directory = Path::new(base_directory);
        let directory_name = Self::generate_working_directory_name(archive_path);
        let path = base_directory.join(directory_name);
        let working_directory = WorkingDirectory::new(&path.to_str().unwrap());

        fs::create_dir(&working_directory.path).or_else(|error| {
            Err(Error::FailedToCreateWorkingDirectory(format!(
                "{} {}",
                path.display().to_string(),
                error.to_string()
            )))
        })?;
        fs::create_dir(&working_directory.archive)?;
        fs::create_dir(&working_directory.nii)?;
        fs::create_dir(&working_directory.masks)?;
        fs::create_dir(&working_directory.output_nii)?;
        fs::create_dir(&working_directory.output_dicom)?;
        let files = extract_zip(&archive_path, &working_directory.archive)?;
        for file in files {
            debug!("{}", file.to_string());
        }

        Ok(working_directory)
    }

    pub fn generate_working_directory_name(input_file_path: &str) -> String {
        let input_file_stem = Path::new(input_file_path).file_stem().unwrap();
        format!(
            "{}-{}",
            input_file_stem.to_str().unwrap(),
            Ulid::new().to_string()
        )
    }

    pub fn get_nifti_filename(&self) -> String {
        format!("{}.nii.gz", self.directory)
    }

    pub fn get_dicom_filename(&self) -> String {
        format!("{}.zip", self.directory)
    }

    pub fn get_relative_mask_directory(&self, relative_to: &str) -> String {
        self.replace_base_directory(&self.masks.display().to_string(), relative_to)
    }

    pub fn get_relative_nifti_output(&self, relative_to: &str) -> String {
        self.replace_base_directory(
            &self
                .output_nii
                .join(self.get_nifti_filename())
                .display()
                .to_string(),
            relative_to,
        )
    }

    pub fn get_absolute_nifti_output(&self) -> String {
        self.absolute(&self.output_nii)
            .join(self.get_nifti_filename())
            .display()
            .to_string()
    }

    pub fn get_absolute_dicom_output(&self) -> String {
        self.absolute(&self.path)
            .join(self.get_dicom_filename())
            .display()
            .to_string()
    }

    pub fn get_absolute_dicom_output_directory(&self) -> String {
        self.absolute(&self.output_dicom).display().to_string()
    }

    pub fn get_relative_nifti_images(&self, relative_to: &str) -> Vec<String> {
        self.switch_working_directory(
            self.search_files_by_extension(&self.nii.display().to_string(), "nii"),
            relative_to,
        )
    }

    pub fn get_relative_mask_images(&self, relative_to: &str) -> Vec<String> {
        self.switch_working_directory(
            self.search_files_by_extension(&self.masks.display().to_string(), "gz"),
            relative_to,
        )
    }

    pub fn absolute_path(&self) -> String {
        self.absolute(&self.path).display().to_string()
    }

    pub fn clean_output_dicom(&self) -> Result<()> {
        self.clean(&self.output_dicom)
    }

    fn clean(&self, path: &PathBuf) -> Result<()> {
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
        {
            fs::remove_file(entry.path())?;
        }
        Ok(())
    }

    fn absolute(&self, path: &PathBuf) -> PathBuf {
        match std::fs::canonicalize(path) {
            Ok(path) => path.clone(),
            Err(error) => panic!(
                "Failed to canonicalize {}: {}",
                path.display().to_string(),
                error.to_string()
            ),
        }
    }

    fn replace_base_directory(&self, path: &str, replacement: &str) -> String {
        let remove_prefix = Path::new(path)
            .strip_prefix(&self.path.display().to_string())
            .unwrap();
        let replace_prefix = Path::new(replacement).join(remove_prefix);
        replace_prefix.display().to_string()
    }

    fn search_files_by_extension(&self, target_directory: &str, extension: &str) -> Vec<String> {
        let mut files = Vec::new();

        for entry in WalkDir::new(target_directory)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| Some(OsStr::new(extension)) == e.path().extension())
        {
            files.push(entry.path().display().to_string());
        }
        files.sort();
        for file in &files {
            debug!("Found {} in {}", file, target_directory);
        }
        files
    }

    fn switch_working_directory(&self, files: Vec<String>, working_directory: &str) -> Vec<String> {
        files
            .into_iter()
            .map(|e| self.replace_base_directory(&e, working_directory))
            .collect()
    }
}

pub struct NiftyMic {
    working_directory: WorkingDirectory,
    docker_wrapper: DockerWrapper,
    config: Config,
}

impl NiftyMic {
    pub fn new(archive_path: &str, config: &Config) -> Result<NiftyMic> {
        Ok(NiftyMic {
            working_directory: WorkingDirectory::from_archive(
                archive_path,
                &config.output.base_directory,
            )?,
            docker_wrapper: DockerWrapper::from_config(config),
            config: config.clone(),
        })
    }

    pub fn from_working_directory(working_directory: &str, config: &Config) -> Result<NiftyMic> {
        Ok(NiftyMic {
            working_directory: WorkingDirectory::new(working_directory),
            docker_wrapper: DockerWrapper::from_config(config),
            config: config.clone(),
        })
    }

    pub fn generate_masks_from_nifti(&self) -> Result<()> {
        let mut args = Vec::new();
        args.push("--filenames".to_string());
        args.append(
            &mut self
                .working_directory
                .get_relative_nifti_images(&self.docker_wrapper.working_directory),
        );
        args.push("--dir-output".to_string());
        args.push(
            self.working_directory
                .get_relative_mask_directory(&self.docker_wrapper.working_directory),
        );
        info!("Generating masks from NIfTI images");
        self.docker_wrapper.run(
            "niftymic_segment_fetal_brains",
            &args,
            &self.working_directory.absolute_path(),
        )?;
        info!("Successfully generated masks from NifTI images");
        Ok(())
    }

    pub fn reconstruct(&self, options: Options) -> Result<()> {
        let mut args = Vec::new();
        args.push("--filenames".to_string());
        args.append(
            &mut self
                .working_directory
                .get_relative_nifti_images(&self.docker_wrapper.working_directory),
        );
        args.push("--filenames-masks".to_string());
        args.append(
            &mut self
                .working_directory
                .get_relative_mask_images(&self.docker_wrapper.working_directory),
        );
        args.append(&mut options.to_args());
        args.push("--output".to_string());
        args.push(
            self.working_directory
                .get_relative_nifti_output(&self.docker_wrapper.working_directory),
        );
        info!("Starting reconstruction of the volume");
        self.docker_wrapper.run(
            "niftymic_reconstruct_volume",
            &args,
            &self.working_directory.absolute_path(),
        )?;
        info!("Successfully reconstruct volume");
        Ok(())
    }

    pub fn convert_dicom_to_nifti(&self) -> Result<()> {
        info!("Start converting DICOM to NIfTI");
        spawn_command(
            &self.config.executables.dcm2niix,
            &vec![
                "-o".to_string(),
                self.working_directory.nii.display().to_string(),
                self.working_directory.archive.display().to_string(),
            ],
            None,
        )?;
        info!("Successfully convert input archive to nifti files");
        Ok(())
    }

    pub fn convert_nifti_to_dicom(&self) -> Result<String> {
        info!("Start converting NIfTI to DICOM");
        debug!(
            "Set working_directory to: {}",
            self.working_directory.get_absolute_dicom_output_directory()
        );
        self.working_directory.clean_output_dicom()?;
        spawn_command(
            &self.config.executables.medcon,
            &vec![
                "-f".to_string(),
                self.working_directory.get_absolute_nifti_output(),
                "-split3d".to_string(),
                "-c".to_string(),
                "dicom".to_string(),
            ],
            Some(&self.working_directory.get_absolute_dicom_output_directory()),
        )?;
        info!("Successfully convert NIfTI to DICOM");
        info!(
            "Creating output archive {}",
            self.working_directory.get_dicom_filename()
        );
        create_output_archive(
            &self.working_directory.get_absolute_dicom_output_directory(),
            &self.working_directory.get_absolute_dicom_output(),
        )?;
        Ok(self.working_directory.get_absolute_dicom_output())
    }
}
