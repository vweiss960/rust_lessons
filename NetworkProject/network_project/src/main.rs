// use pnet::datalink;

// fn main() {
//     // Get all network interfaces
//     let interfaces = datalink::interfaces();
    
//     println!("Available network interfaces:");
//     println!("{:-<60}", "");
    

//     for interface in interfaces {
//         let mut status = "".to_string();
//         if interface.is_up() {
//             status = "UP".to_string();
//         } else {
//             status = "DOWN".to_string();
//         }
//         println!("Name: {}", interface.name);
//         println!("  Status {status}");
//         println!("  Index: {}", interface.index);
//         println!("  MAC: {:?}", interface.mac);
//         println!("  IPs: {}", interface.ips[0]);
//         println!("  Flags: {:?}", interface.flags);
//         println!();
//     }
// }



// use pnet::datalink::{self, NetworkInterface};

// fn main() {
//     let interfaces = datalink::interfaces();
    
//     println!("Available interfaces:");
//     for iface in &interfaces {
//         println!("  {} - {}", iface.name, 
//                  if iface.is_up() { "UP" } else { "DOWN" });
//     }
    
//     // Select interface (modify as needed)
//     let interface = interfaces
//         .into_iter()
//         .find(|iface| iface.index==12 && !iface.is_loopback())
//         .expect("No suitable interface");
    
//     println!("\nCapturing on: {}", interface.name);
    
//     let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
//         Ok(datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
//         Ok(_) => panic!("Unsupported channel type"),
//         Err(e) => panic!("Error: {}", e),
//     };
    
//     let mut count = 0;
//     loop {
//         match rx.next() {
//             Ok(packet) => {
//                 count += 1;
//                 println!("Packet #{}: {} bytes", count, packet.len());
                
//                 // Print first 64 bytes in hex
//                 let preview_len = packet.len().min(64);
//                 print!("  ");
//                 for (i, byte) in packet[..preview_len].iter().enumerate() {
//                     print!("{:02x} ", byte);
//                     if (i + 1) % 16 == 0 {
//                         print!("\n  ");
//                     }
//                 }
//                 println!();
                
//                 if count >= 10 {
//                     break; // Stop after 10 packets for demo
//                 }
//             }
//             Err(e) => eprintln!("Error: {}", e),
//         }
//     }
// }




use pnet::datalink::{self, NetworkInterface, Channel::Ethernet};
use pnet::packet::ethernet::{EthernetPacket, EtherTypes};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::tcp::TcpPacket;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;

fn main() {

    let interface_number = 12;
    // Get list of network interfaces
    let interfaces = datalink::interfaces();
    
    // Find a suitable interface (you might want to filter by name or other criteria)
    let interface = interfaces
        .into_iter()
        .find(|iface| iface.index==interface_number && !iface.is_loopback())
        .expect("No suitable network interface found");
    
    println!("Using interface: {}", interface.name);
    
    // Create a channel to receive packets
    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("Failed to create datalink channel: {}", e),
    };
    
    println!("Starting packet capture...");
    
    // Process packets
    loop {
        match rx.next() {
            Ok(packet) => {
                process_packet(packet);
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
            }
        }
    }
}

fn process_packet(packet: &[u8]) {
    if let Some(ethernet) = EthernetPacket::new(packet) {
        println!("\n--- New Packet ---");
        println!("Source MAC: {}", ethernet.get_source());
        println!("Dest MAC: {}", ethernet.get_destination());
        
        match ethernet.get_ethertype() {
            EtherTypes::Ipv4 => {
                if let Some(ipv4) = Ipv4Packet::new(ethernet.payload()) {
                    process_ipv4_packet(&ipv4);
                }
            }
            EtherTypes::Ipv6 => {
                if let Some(ipv6) = Ipv6Packet::new(ethernet.payload()) {
                    process_ipv6_packet(&ipv6);
                }
            }
            _ => {
                println!("Unknown ethernet type: {:?}", ethernet.get_ethertype());
            }
        }
    }
}

fn process_ipv4_packet(ipv4: &Ipv4Packet) {
    println!("IPv4 Packet:");
    println!("  Source IP: {}", ipv4.get_source());
    println!("  Dest IP: {}", ipv4.get_destination());
    println!("  Protocol: {:?}", ipv4.get_next_level_protocol());
    
    match ipv4.get_next_level_protocol() {
        IpNextHeaderProtocols::Tcp => {
            if let Some(tcp) = TcpPacket::new(ipv4.payload()) {
                process_tcp_packet(&tcp);
            }
        }
        IpNextHeaderProtocols::Udp => {
            if let Some(udp) = UdpPacket::new(ipv4.payload()) {
                process_udp_packet(&udp);
            }
        }
        _ => {}
    }
}

fn process_ipv6_packet(ipv6: &Ipv6Packet) {
    println!("IPv6 Packet:");
    println!("  Source IP: {}", ipv6.get_source());
    println!("  Dest IP: {}", ipv6.get_destination());
    println!("  Next Header: {:?}", ipv6.get_next_header());
}

fn process_tcp_packet(tcp: &TcpPacket) {
    println!("  TCP Segment:");
    println!("    Source Port: {}", tcp.get_source());
    println!("    Dest Port: {}", tcp.get_destination());
    println!("    Sequence: {}", tcp.get_sequence());
    println!("    Flags: SYN={} ACK={} FIN={} RST={}", 
             tcp.get_flags() & 0x02 != 0,
             tcp.get_flags() & 0x10 != 0,
             tcp.get_flags() & 0x01 != 0,
             tcp.get_flags() & 0x04 != 0);
}

fn process_udp_packet(udp: &UdpPacket) {
    println!("  UDP Datagram:");
    println!("    Source Port: {}", udp.get_source());
    println!("    Dest Port: {}", udp.get_destination());
    println!("    Length: {}", udp.get_length());
}