fn main() {
    embuild::espidf::sysenv::output();

    dotenv::dotenv().ok();

    if let Ok(ssid) = std::env::var("SSID") {
        println!("cargo:rustc-env=SSID={ssid}");
    }

    if let Ok(password) = std::env::var("PASSWORD") {
        println!("cargo:rustc-env=PASSWORD={password}");
    }
}
