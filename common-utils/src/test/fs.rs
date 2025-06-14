use tempfile::tempdir;
use std::fs::read_to_string;
use crate::fs::FileSystem;

#[test]
fn test_create_sub_dir() {
    let tmp_dir = tempdir().unwrap();
    let fs = FileSystem::new(tmp_dir.path()).unwrap();

    let sub = fs.ensure_sub_dir("test_sub_dir").unwrap();
    assert!(sub.exists());
    assert!(sub.is_dir());
}

#[test]
fn test_write_to_file() {
    let tmp_dir = tempdir().unwrap();
    let fs = FileSystem::new(tmp_dir.path()).unwrap();
    let sub = fs.ensure_sub_dir("data").unwrap();
    assert!(sub.exists());

    let file_path = fs.write_to_file("data", "example.txt", b"Hello, world!").unwrap();
    assert!(file_path.exists());
    let content = read_to_string(file_path).unwrap();
    assert_eq!(content, "Hello, world!");
}