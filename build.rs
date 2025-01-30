#[cfg(all(not(feature = "_tungstenite"), not(feature = "_websockets")))]
compile_error!("Please specify a websocket feature, see README.md for a list.");

#[cfg(all(feature = "_rustls-tls", feature = "_native-tls"))]
compile_error!("Please specify only one of the `rustls` or `native` features, not both.");

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
