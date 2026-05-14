use super::super::*;

#[test]
fn read_file_returns_file_bytes() {
    let path = std::env::temp_dir().join("steg-read-file-test.bin");
    let bytes = b"file helper bytes";

    std::fs::write(&path, bytes).unwrap();

    assert_eq!(read_file(&path).unwrap(), bytes);

    std::fs::remove_file(path).unwrap();
}

#[test]
fn read_file_reports_missing_file() {
    let path = std::env::temp_dir().join("steg-missing-file-test.bin");

    assert!(read_file(path).is_err());
}
