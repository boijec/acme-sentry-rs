use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

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
    pub fn write_to_file(&self, sub_dir: &str, filename: &str, data: &[u8]) -> Result<PathBuf, Box<dyn Error>> {
        let dir = self.sub_dir(sub_dir);
        let file_path = dir.join(filename);
        println!("Writing {}", file_path.display());
        fs::write(&file_path, data)?;
        Ok(file_path)
    }
}