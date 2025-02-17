use ifcfg::{AddressFamily, IfCfg};
use std::net::{IpAddr, UdpSocket};

/// get_local_address - get the local ip address, return an `Option<String>`. when it fails, return `None`.
pub fn get_local_addr() -> Option<IpAddr> {
    // bind to IN_ADDR_ANY, can be multiple interfaces/addresses
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(_) => return None,
    };
    // try to connect to Google DNS so that we bind to an interface connected to the internet
    match socket.connect("8.8.8.8:80") {
        Ok(()) => (),
        Err(_) => return None,
    };
    // now we can return the IP address of this interface
    match socket.local_addr() {
        Ok(addr) => Some(addr.ip()),
        Err(_) => None,
    }
}

pub fn get_interfaces() -> Vec<String> {
    let mut interfaces: Vec<String> = Vec::new();
    let ifaces = IfCfg::get().expect("could not get interfaces");
    for iface in ifaces {
        for addr in iface.addresses {
            if let AddressFamily::IPv4 = addr.address_family {
                interfaces.push(addr.address.unwrap().ip().to_string());
            }
        }
    }
    interfaces
}
