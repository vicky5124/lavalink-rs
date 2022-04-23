#[cfg(all(not(feature = "rustls"), not(feature = "native")))]
compile_error!("Please specify a feature, either `rustls` or `native`.");

#[cfg(all(feature = "rustls", feature = "native"))]
compile_error!("Please specify a single TLS features, either `rustls` or `native`, but not both (hint: `default-features = false`).");

#[cfg(all(not(feature = "songbird"), not(feature = "discord-gateway")))]
compile_error!("Set either `songbird` or `discord-gateway` as a feature to be able to connect to voicce channels.");

#[cfg(all(not(feature = "tracing-log"), not(feature = "normal-log")))]
compile_error!("Set either `tracing-log` or `normal-log` in your features.");

#[cfg(all(feature = "tracing-log", feature = "normal-log"))]
compile_error!("You can only choose one of `tracing-log` and `normal-log` as your logging crate. (hint: `default-features = false`)");

use version_check::Version;

fn main() {
    let version = match version_check::triple() {
        Some((ver, ..)) => ver,
        None => Version::parse("1.0.0").unwrap(),
    };

    if version.to_mmp().1 < 51 {
        panic!("Minimum rust version required is 1.51, please update your rust version via `rustup update`");
    }
}
