use mdns_sd::{ServiceDaemon, ServiceInfo};

pub fn start_advertiser(
    device_name: String,
    ip: String,
    port: u16,
) {
    let mdns = ServiceDaemon::new().unwrap();
    let service_type = "_snaplan._tcp.local.";
    let instance = device_name.clone();
    let host = format!("{}.local.", device_name.replace(" ", "-")); // Remove spaces for host
    let properties = [("version", "1.0")];
    let service = ServiceInfo::new(
        service_type,
        &instance,
        &host,
        ip,
        port,
        &properties[..]
    ).unwrap();
    mdns.register(service).unwrap();
    println!("Advertising SnapLAN service...");
}