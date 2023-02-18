use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::peripherals::Peripherals;

use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition, wifi::EspWifi};

use embedded_svc::wifi::{ClientConfiguration, Configuration, Wifi};

use embedded_svc::{
    http::{client::Client as HttpClient,Status},
    utils::io,
};

use esp_idf_svc::http::client::{Configuration as HttpConfiguration, EspHttpConnection};

use std::{thread, time};

#[toml_cfg::toml_config]
struct Credentials {
    #[default("")]
    ssid: &'static str,
    #[default("")]
    passphrase: &'static str,
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    println!(
        "Starting to connect to wifi: {:?} with password: {:?}",
        CREDENTIALS.ssid, CREDENTIALS.passphrase
    );

    let peripherals = Peripherals::take().unwrap();
    let sys_loop = EspSystemEventLoop::take().unwrap();
    let nvs = EspDefaultNvsPartition::take().unwrap();

    let mut wifi_driver = EspWifi::new(peripherals.modem, sys_loop, Some(nvs)).unwrap();

    wifi_driver
        .set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: CREDENTIALS.ssid.into(),
            password: CREDENTIALS.passphrase.into(),
            ..Default::default()
        }))
        .unwrap();

    wifi_driver.start().unwrap();
    wifi_driver.connect().unwrap();
    while !wifi_driver.is_connected().unwrap() {
        let config = wifi_driver.get_configuration().unwrap();
        println!("Waiting for station {:?}", config)
    }
    println!("Connection established");

    // create http client
    let mut client = HttpClient::wrap(
        EspHttpConnection::new(&HttpConfiguration {
            crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),
            ..Default::default()
        })
        .unwrap(),
    );

    thread::sleep(time::Duration::from_secs(5));

    get(&mut client, "http://ifconfig.net");
}

fn get(client: &mut HttpClient<EspHttpConnection>, url: &str) {
    println!("Making GET HTTP Request to: {}", url);
    let request = client.get(url).unwrap();

    let mut response = request.submit().unwrap();

    let status = response.status();
    println!("<- Status: {}", status);
    let (_headers, mut body) = response.split();
    let mut buf = [0u8; 1024];
    let bytes_read = io::try_read_full(&mut body, &mut buf)
        .map_err(|e| e.0)
        .unwrap();
    println!("Read {} bytes", bytes_read);
    match std::str::from_utf8(&buf[0..bytes_read]) {
        Ok(body_string) => println!("Response body: {:?}", body_string),
        Err(e) => eprintln!("Error decoding response body: {}", e),
    };

    while body.read(&mut buf).unwrap() > 0 {}
}
