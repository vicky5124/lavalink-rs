#[cfg(all(not(feature = "rustls-marker"), not(feature = "native-marker")))]
compile_error!("Please specify a feature, either `rustls` or `native`");

fn main() {}
