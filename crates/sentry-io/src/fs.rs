use std::{env, fs};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tracing::{debug, instrument};

pub struct BasicUnixFileSystem {
    base_dir: PathBuf,
}
pub type FileResult<T> = Result<T, FileSystemError>;

#[derive(Error, Debug)]
pub enum FileSystemError {
    #[error("Unable to initialize file system operations at :: {1}")]
    UnableToInitialize(io::Error, String),
    #[error("Unable to initialize file system operations because of missing resources")]
    UnableToEstablishHomeFolder,
    #[error("Unable to create subdirectory :: {1}")]
    UnableToEnsureSubDir(io::Error, String),
    #[error("Unable to read from file :: {1}")]
    UnableToReadFile(io::Error, String),
    #[error("Unable to write to file :: {1}")]
    UnableToWriteFile(io::Error, String),
    #[error("Unable to find directory :: {1}")]
    UnableToFindDirectory(io::Error, String),
    #[error("Unable to find directory declaration in env path :: {0}")]
    UnableToLocateDirectoryEnvironment(String),
    #[error("Unknown filesystem error :: {0}")]
    UnknownFileSystemError(io::Error),
}

pub trait FileSystemOperations {
    fn sub_dir(&self, name: &str) -> PathBuf;
    fn ensure_sub_dir(&self, name: &str) -> FileResult<PathBuf>;
    fn file_exists_adaptive(&self, dir: PathBuf, file_name: &str) -> bool;
    fn file_exists(&self, dir: PathBuf, file_name: &str) -> bool;
    fn touch_file(
        &self,
        sub_dir: &str,
        filename: &str,
    ) -> FileResult<()>;
    fn write_to_file(
        &self,
        sub_dir: &str,
        filename: &str,
        data: &[u8],
    ) -> FileResult<PathBuf>;
    fn read_from_file(&self, sub_dir: &str, filename: &str) -> FileResult<Vec<u8>>;
}

pub enum FileSystemOrigin {
    CurrentDirectory,
    UserHomeDirectory,
    OptionsDirectory
}

impl BasicUnixFileSystem {
    const OPT_BASE_DIR: &str = "/opt";

    pub fn new<P: AsRef<Path>>(base_dir: P) -> FileResult<Self> {
        let base_dir = base_dir.as_ref();
        if !base_dir.exists() {
            return Err(FileSystemError::UnableToEstablishHomeFolder);
        }
        Ok(BasicUnixFileSystem {
            base_dir: base_dir.to_path_buf(),
        })
    }
    pub fn initialize<P: AsRef<Path>>(base_dir: P, origin_type: FileSystemOrigin) -> FileResult<Self> {
        let base = match origin_type {
            FileSystemOrigin::CurrentDirectory => env::current_dir().map_err(|err| FileSystemError::UnknownFileSystemError(err))?,
            FileSystemOrigin::UserHomeDirectory => env::home_dir().ok_or_else(|| FileSystemError::UnableToLocateDirectoryEnvironment("user home directory not found".into()))?,
            FileSystemOrigin::OptionsDirectory => Self::OPT_BASE_DIR.into(),
        };
        let origin = base.join(base_dir.as_ref());
        if !origin.exists() {
            fs::create_dir_all(&origin).map_err(|e| {
                FileSystemError::UnableToInitialize(
                    e,
                    origin.as_os_str().to_string_lossy().to_string(),
                )
            })?;
        }
        Ok(BasicUnixFileSystem {
            base_dir: origin.to_path_buf(),
        })
    }
    pub fn scan_for_home_folder(base_dir_name: &str) -> FileResult<String> {
        let user_home = env::home_dir()
            .ok_or_else(|| FileSystemError::UnableToLocateDirectoryEnvironment("Home dir".into()))?;
        let path_in_user_home = user_home.join(PathBuf::from(base_dir_name));
        let exists_in_user = fs::exists(&path_in_user_home)
            .map_err(|e| FileSystemError::UnknownFileSystemError(e))?;
        if exists_in_user {
            return Ok(path_in_user_home.into_string().unwrap());
        }
        let curr_dir = env::current_dir()
            .map_err(|e| FileSystemError::UnableToFindDirectory(e, "current directory".into()))?;
        let path_in_current_dir = curr_dir.join(PathBuf::from(base_dir_name));
        let exists_in_curr_dir = fs::exists(&path_in_current_dir)
            .map_err(|e| FileSystemError::UnknownFileSystemError(e))?;
        if exists_in_curr_dir {
            return Ok(path_in_current_dir.into_string().unwrap());
        }
        let path_in_opt = PathBuf::from(Self::OPT_BASE_DIR).join(PathBuf::from(base_dir_name));
        let exists_in_opt = fs::exists(&path_in_opt)
            .map_err(|e| FileSystemError::UnknownFileSystemError(e))?;
        if exists_in_opt {
            return Ok(path_in_opt.into_os_string().into_string().unwrap());
        }

        Err(FileSystemError::UnableToEstablishHomeFolder)
    }
}

impl FileSystemOperations for BasicUnixFileSystem {
    fn sub_dir(&self, name: &str) -> PathBuf {
        self.base_dir.join(name)
    }
    fn ensure_sub_dir(&self, name: &str) -> FileResult<PathBuf> {
        let target = self.sub_dir(name);
        if !target.exists() {
            fs::create_dir_all(&target)
                .map_err(|e| FileSystemError::UnableToEnsureSubDir(e, name.to_string()))?;
        }
        Ok(target)
    }
    fn file_exists_adaptive(&self, dir: PathBuf, file_name: &str) -> bool {
        dir.join(file_name).exists()
    }
    fn file_exists(&self, dir: PathBuf, file_name: &str) -> bool {
        self.base_dir.join(dir).join(file_name).exists()
    }
    #[instrument(skip(self, sub_dir, filename))]
    fn touch_file(
        &self,
        sub_dir: &str,
        filename: &str,
    ) -> FileResult<()> {
        let dir = self.sub_dir(sub_dir);
        let file_path = dir.join(filename);
        debug!("Touching file: {}", file_path.display());
        let _ = File::create_new(file_path)
            .map_err(|e| FileSystemError::UnknownFileSystemError(e))?;
        Ok(())
    }
    #[instrument(skip(self, sub_dir, filename, data))]
    fn write_to_file(
        &self,
        sub_dir: &str,
        filename: &str,
        data: &[u8],
    ) -> FileResult<PathBuf> {
        let dir = self.sub_dir(sub_dir);
        let file_path = dir.join(filename);
        debug!("Writing file: {}", file_path.display());
        fs::write(&file_path, data)
            .map_err(|e| FileSystemError::UnableToWriteFile(e, filename.to_string()))?;
        Ok(file_path)
    }
    #[instrument(skip(self, sub_dir, filename))]
    fn read_from_file(&self, sub_dir: &str, filename: &str) -> FileResult<Vec<u8>> {
        let dir = self.base_dir.join(self.sub_dir(sub_dir));
        let file_path = dir.join(filename);
        debug!("Reading file: {}", file_path.display());
        let vec = fs::read(&file_path)
            .map_err(|e| FileSystemError::UnableToReadFile(e, filename.to_string()))?;
        Ok(vec)
    }
}
