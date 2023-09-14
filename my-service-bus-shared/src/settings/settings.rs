#[cfg(target_os = "windows")] 
pub fn get_settings_filename_path(file_name: &str) -> String{
    let home_path = env!("HOME");
    let filename = format!("{}\\{}", home_path, file_name);
    filename
}

#[cfg(not(target_os = "windows"))] 
pub fn get_settings_filename_path(file_name: &str) -> String{
    let home_path = env!("HOME");
    let filename = format!("{}/{}", home_path, file_name);
    filename
}

#[cfg(test)]
mod tests {

    fn get_settings_directory() -> String{
        let home_path = env!("HOME");
        String::from(home_path)
    }

    #[test]
    #[cfg(target_os = "windows")] 
    fn test_format() {
        let file_name = "file";
        let settings_name = super::get_settings_filename_path(file_name);

        let path_to_filename = format!("{}\\{}", get_settings_directory(), file_name);

        assert_eq!(settings_name, String::from(path_to_filename));
    }

    #[test]
    #[cfg(not(target_os = "windows"))] 
    fn test_format() {
        let file_name = "file";
        let settings_name = super::get_settings_filename_path(file_name);

        let path_to_filename = format!("{}/{}", get_settings_directory(), file_name);

        assert_eq!(settings_name, String::from(path_to_filename));
    }
}