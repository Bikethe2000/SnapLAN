use mdns_sd::{ServiceDaemon, ServiceEvent};
use crate::state::{AppState, DiscoveryDevice};
use std::sync::{Arc, Mutex};

pub fn start_browser(state: Arc<Mutex<AppState>>) {
    let mdns = ServiceDaemon::new().unwrap();
    let receiver = mdns.browse("_snaplan._tcp.local.").unwrap();
    std::thread::spawn(move || {
        while let Ok(event) = receiver.recv() {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    let ip = info.get_addresses().iter().next()
                        .map(|a| a.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    
                    let mut state = state.lock().unwrap();
                    let name = info.get_fullname();
                    
                    println!("mDNS: Resolved discovery device: {}", name);
                    
                    state.add_discovery_device(DiscoveryDevice {
                        name: name.to_string(),
                        ip: ip,
                        port: info.get_port(),
                    });

                    // Broadcast discovery list immediately
                    let devices: Vec<_> = state.discovery_devices.values().cloned().collect();
                    let event = serde_json::json!({
                        "type": "discovery_list",
                        "data": devices,
                    });
                    state.emit(event);
                }
                ServiceEvent::ServiceRemoved(_type, name) => {
                    let mut state = state.lock().unwrap();
                    println!("mDNS: Removed discovery device: {}", name);
                    state.remove_discovery_device(&name);

                    // Broadcast updated list
                    let devices: Vec<_> = state.discovery_devices.values().cloned().collect();
                    let event = serde_json::json!({
                        "type": "discovery_list",
                        "data": devices,
                    });
                    state.emit(event);
                }
                _ => {}
            }
        }
    });
}