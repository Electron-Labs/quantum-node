#[macro_export]
macro_rules! error_line {
    ($err:expr) => {
        format!("{} in file {} on {}", $err.to_string(), file!(), line!()).to_string()
    };
}