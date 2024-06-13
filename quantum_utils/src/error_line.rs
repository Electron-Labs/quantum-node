// quantum_type uses quantum utils hence error_line! cann't be imported here due to cyclical dependency
#[macro_export]
macro_rules! error_line {
    ($err:expr) => {
        format!("{} in file {} on {}", $err.to_string(), file!(), line!()).to_string()
    };
}