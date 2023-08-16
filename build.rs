#[cfg(all(not(feature = "rustls"), not(feature = "native-tls")))]
compile_error!("Please specify a feature, either `rustls` or `native`.");

#[cfg(all(feature = "rustls", feature = "native-tls"))]
compile_error!("Please specify one of `rustls` or `native` as a feature, not both.");

use version_check::Version;

fn main() {
    let version = match version_check::triple() {
        Some((ver, ..)) => ver,
        None => Version::parse("1.0.0").unwrap(),
    };

    if version.to_mmp().1 < 70 {
        panic!("Minimum rust version required is 1.70, please update your rust version via `rustup update`");
    }
}
