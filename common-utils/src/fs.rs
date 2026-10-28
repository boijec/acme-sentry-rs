use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::debug;

pub struct BasicUnixFileSystem {
    base_dir: PathBuf,
}

#[derive(Error, Debug)]
pub enum FileSystemError {
    #[error("Unable to initialize file system operations at: {1}")]
    UnableToInitialize(io::Error, String),
    #[error("Unable to create subdirectory: {1}")]
    UnableToEnsureSubDir(io::Error, String),
    #[error("Unable to read from file: {1}")]
    UnableToReadFile(io::Error, String),
    #[error("Unable to write to file: {1}")]
    UnableToWriteFile(io::Error, String),
}

pub trait FileSystemOperations {
    fn sub_dir(&self, name: &str) -> PathBuf;
    fn ensure_sub_dir(&self, name: &str) -> Result<PathBuf, FileSystemError>;
    fn file_exists(&self, dir: PathBuf, file_name: &str) -> bool;
    fn write_to_file(&self, sub_dir: &str, filename: &str, data: &[u8]) -> Result<PathBuf, FileSystemError>;
    fn read_from_file(&self, sub_dir: &str, filename: &str) -> Result<Vec<u8>, FileSystemError>;
}

impl BasicUnixFileSystem {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Result<Self, FileSystemError> {
        let base_dir = base_dir.as_ref();
        if !base_dir.exists() {
            fs::create_dir_all(base_dir)
                .map_err(|e| FileSystemError::UnableToInitialize(e, base_dir.as_os_str().to_string_lossy().to_string()))?;
        }
        Ok(BasicUnixFileSystem {
            base_dir: base_dir.to_path_buf(),
        })
    }
}

impl FileSystemOperations for BasicUnixFileSystem {

    fn sub_dir(&self, name: &str) -> PathBuf {
        self.base_dir.join(name)
    }
    fn ensure_sub_dir(&self, name: &str) -> Result<PathBuf, FileSystemError> {
        let target = self.sub_dir(name);
        if !target.exists() { 
            fs::create_dir_all(&target)
                .map_err(|e| FileSystemError::UnableToEnsureSubDir(e, name.to_string()))?;
        }
        Ok(target)
    }
    fn file_exists(&self, dir: PathBuf, file_name: &str) -> bool {
        dir.join(file_name).exists()
    }
    fn write_to_file(&self, sub_dir: &str, filename: &str, data: &[u8]) -> Result<PathBuf, FileSystemError> {
        let dir = self.sub_dir(sub_dir);
        let file_path = dir.join(filename);
        debug!("Writing file: {}", file_path.display());
        fs::write(&file_path, data)
            .map_err(|e| FileSystemError::UnableToWriteFile(e, filename.to_string()))?;
        Ok(file_path)
    }
    fn read_from_file(&self, sub_dir: &str, filename: &str) -> Result<Vec<u8>, FileSystemError> {
        let dir = self.sub_dir(sub_dir);
        let file_path = dir.join(filename);
        debug!("Reading file: {}", file_path.display());
        let vec = fs::read(&file_path)
            .map_err(|e| FileSystemError::UnableToReadFile(e, filename.to_string()))?;
        Ok(vec)
    }
}