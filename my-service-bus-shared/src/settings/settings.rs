#[cfg(unix)]
fn get_home_dir() -> String {
    std::env::var("HOME").expect("HOME environment variable is not set")
}

#[cfg(windows)]
fn get_home_dir() -> String {
    std::env::var("USERPROFILE").expect("USERPROFILE environment variable is not set")
}

pub fn get_settings_filename_path(file_name: &str) -> String {
    let mut path = std::path::PathBuf::from(get_home_dir());
    path.push(file_name);
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let file_name = "file";
        let settings_name = get_settings_filename_path(file_name);

        let expected = {
            let mut p = std::path::PathBuf::from(get_home_dir());
            p.push(file_name);
            p.to_string_lossy().into_owned()
        };

        assert_eq!(settings_name, expected);
    }
}
