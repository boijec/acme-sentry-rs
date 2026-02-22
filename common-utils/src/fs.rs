use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::debug;

pub struct FileSystem {
    base_dir: PathBuf,
}

impl FileSystem {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Result<Self, Box<dyn Error>> {
        let base_dir = base_dir.as_ref();
        if !base_dir.exists() {
            fs::create_dir_all(base_dir)?;
        }
        Ok(FileSystem {
            base_dir: base_dir.to_path_buf(),
        })
    }
    pub fn sub_dir(&self, name: &str) -> PathBuf {
        self.base_dir.join(name)
    }
    pub fn ensure_sub_dir(&self, name: &str) -> Result<PathBuf, Box<dyn Error>> {
        let target = self.sub_dir(name);
        if !target.exists() { 
            fs::create_dir_all(&target)?;
        }
        Ok(target)
    }
    pub fn file_exists(&self, dir: PathBuf, file_name: &str) -> bool {
        dir.join(file_name).exists()
    }
    pub fn write_to_file(&self, sub_dir: &str, filename: &str, data: &[u8]) -> Result<PathBuf, Box<dyn Error>> {
        let dir = self.sub_dir(sub_dir);
        let file_path = dir.join(filename);
        debug!("Writing file: {}", file_path.display());
        fs::write(&file_path, data)?;
        Ok(file_path)
    }
    pub fn read_from_file(&self, sub_dir: &str, filename: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let dir = self.sub_dir(sub_dir);
        let file_path = dir.join(filename);
        debug!("Reading file: {}", file_path.display());
        let vec = fs::read(&file_path)?;
        Ok(vec)
    }
}