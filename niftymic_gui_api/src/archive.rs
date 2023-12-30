use crate::niftymic::*;
use log::debug;
use std::{
    fs::File,
    io::{BufReader, Read, Write},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;
use zip::{write::FileOptions, ZipWriter};

fn is_zip_archive(path: &Path) -> bool {
    if path.exists() && !path.is_dir() {
        if let Some(extension) = path.extension() {
            if extension == "zip" {
                return true;
            }
        }
    }
    false
}

fn zip_archive_is_accessible(path: &str) -> Result<File> {
    let path_obj = Path::new(path);

    debug!("Test if {} is accessible", path);

    if !is_zip_archive(path_obj) {
        return Err(Error::NotValidZipArchive(path.to_string()));
    }
    if !path_obj.exists() {
        return Err(Error::NotValidZipArchive(format!(
            "{} does not exists",
            path.to_string()
        )));
    }
    let file = File::open(path)?;

    Ok(file)
}

fn list_files(path: &PathBuf) -> Result<Vec<String>> {
    let mut files = Vec::new();
    for entry in WalkDir::new(path) {
        let entry = entry?;
        if entry.file_type().is_file() {
            files.push(entry.path().display().to_string());
        }
    }
    Ok(files)
}

pub fn extract_zip(input_file: &str, output_directory: &PathBuf) -> Result<Vec<String>> {
    let reader = zip_archive_is_accessible(input_file)?;
    let mut archive = zip::ZipArchive::new(reader)?;

    if archive.is_empty() {
        return Err(Error::NotValidZipArchive(format!(
            "{} is empty",
            input_file.to_string()
        )));
    }
    archive.extract(output_directory).or_else(|e| {
        Err(Error::NotValidZipArchive(format!(
            "{} {}",
            input_file.to_string(),
            e.to_string(),
        )))
    })?;
    Ok(list_files(&output_directory)?)
}

pub fn create_output_archive(input_directory: &str, filename: &str) -> Result<()> {
    let file = File::create(filename)?;
    let mut zip = ZipWriter::new(file);
    for entry in WalkDir::new(input_directory) {
        let entry = entry.unwrap();
        if let Some(extension) = entry.path().extension() {
            if extension == "dcm" {
                let input_file = File::open(entry.path())?;
                let mut reader = BufReader::new(input_file);
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer)?;
                zip.start_file(
                    entry
                        .path()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                    FileOptions::default(),
                )?;
                zip.write_all(&buffer)?;
            }
        }
    }
    Ok(())
}
