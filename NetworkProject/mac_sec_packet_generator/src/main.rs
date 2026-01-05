use byteorder::{LittleEndian, WriteBytesExt};
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::HashSet;

/// MACsec frame format (SecTag + ICV appended to Ethernet frame)
///
/// Ethernet frame with MACsec:
/// Destination MAC (6) | Source MAC (6) | EtherType (2) | MACsec SecTag (16) | Payload | ICV (16)
///
/// SecTag structure (16 bytes):
/// - EtherType: 0x88E5 (2 bytes)
/// - TCI/AN: 1 byte
///   - Version (2 bits): Always 0
///   - End Station (1 bit): 0 for End Station, 1 for switch
///   - Control Port (1 bit): Whether SecTag is on control port
///   - Untagged (1 bit): 0 for tagged
///   - SC Present (1 bit): Whether SCI is present
///   - E bit (1 bit): Encryption enabled
///   - C bit (1 bit): Changed text
/// - Packet Number (4 bytes): Sequence number (increments per packet)
/// - SCI (Secure Channel Identifier) (8 bytes):
///   - System Identifier (MAC address) (6 bytes)
///   - Port Identifier (2 bytes)

const MACSEC_ETHERTYPE: u16 = 0x88E5;
const ICV_LENGTH: usize = 16; // MACsec Integrity Check Value length
const PAYLOAD_LENGTH: usize = 46; // Payload size (arbitrary for testing)

#[derive(Clone, Copy)]
struct SecTag {
    packet_number: u32,
    system_id: [u8; 6], // Source MAC that originated the packet
    port_id: u16,
    tci_an: u8, // TCI/AN byte with flags
}

impl SecTag {
    fn new(packet_number: u32, system_id: [u8; 6], port_id: u16) -> Self {
        // TCI/AN byte: V(2bits) ES(1) SC(1) SCB(1) E(1) C(1) AN(2)
        // Bit positions (MSB to LSB): 7 6 5 4 3 2 1 0
        // V = Version (bits 7-6): 00 = Version 0
        // ES = End Station (bit 5): 0 = End Station (not a switch)
        // SC = SCI Present (bit 4): 1 = SCI is present
        // SCB = Secure Channel Bound (bit 3): 0 = Not bound
        // E = Encryption (bit 2): 1 = Encrypted
        // C = Changed (bit 1): 1 = Changed
        // AN = Association Number (bits 0): 0 = AN 0
        // Binary: 00 0 1 0 1 1 00 = 0x14
        let tci_an = 0x2f; // V=0, ES=0, SC=1, SCB=0, E=1, C=1, AN=3
        SecTag {
            packet_number,
            system_id,
            port_id,
            tci_an,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(17);
        // EtherType (2 bytes)
        bytes.extend_from_slice(&MACSEC_ETHERTYPE.to_be_bytes());
        // TCI/AN byte (1 byte) - combined, not separate
        bytes.push(self.tci_an);
        // Short Length field (1 byte)
        let short_length = (PAYLOAD_LENGTH as u8) & 0x7F;
        bytes.push(short_length);
        // Packet Number (4 bytes)
        bytes.extend_from_slice(&self.packet_number.to_be_bytes());
        // System Identifier - SCI first 6 bytes (6 bytes)
        bytes.extend_from_slice(&self.system_id);
        // Port Identifier - SCI last 2 bytes (2 bytes)
        bytes.extend_from_slice(&self.port_id.to_be_bytes());
        bytes
    }
}

struct MACsecPacket {
    dest_mac: [u8; 6],
    src_mac: [u8; 6],
    sec_tag: SecTag,
    payload: Vec<u8>,
    icv: [u8; ICV_LENGTH],
}

impl MACsecPacket {
    fn new(
        dest_mac: [u8; 6],
        src_mac: [u8; 6],
        packet_number: u32,
        system_id: [u8; 6],
        port_id: u16,
    ) -> Self {
        let sec_tag = SecTag::new(packet_number, system_id, port_id);

        // Create a simple payload (in real scenario, this would be actual data)
        let mut payload = vec![0u8; PAYLOAD_LENGTH];
        // Fill with pattern for readability
        for (i, byte) in payload.iter_mut().enumerate() {
            *byte = (packet_number as u8).wrapping_add(i as u8);
        }

        // ICV is normally computed from the entire frame
        // For testing purposes, we'll use a simple pattern
        let mut icv = [0u8; ICV_LENGTH];
        for (i, byte) in icv.iter_mut().enumerate() {
            *byte = (packet_number as u8).wrapping_add(i as u8);
        }

        MACsecPacket {
            dest_mac,
            src_mac,
            sec_tag,
            payload,
            icv,
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.dest_mac);
        bytes.extend_from_slice(&self.src_mac);
        bytes.extend_from_slice(&self.sec_tag.to_bytes());
        bytes.extend_from_slice(&self.payload);
        bytes.extend_from_slice(&self.icv);
        bytes
    }
}

/// PCAP file format structures
struct PcapGlobalHeader {
    magic_number: u32,
    version_major: u16,
    version_minor: u16,
    thiszone: i32,
    sigfigs: u32,
    snaplen: u32,
    network: u32,
}

impl PcapGlobalHeader {
    fn new() -> Self {
        PcapGlobalHeader {
            magic_number: 0xa1b2c3d4, // PCAP magic number
            version_major: 2,
            version_minor: 4,
            thiszone: 0,
            sigfigs: 0,
            snaplen: 65535,
            network: 1, // Ethernet
        }
    }

    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_u32::<LittleEndian>(self.magic_number)?;
        writer.write_u16::<LittleEndian>(self.version_major)?;
        writer.write_u16::<LittleEndian>(self.version_minor)?;
        writer.write_i32::<LittleEndian>(self.thiszone)?;
        writer.write_u32::<LittleEndian>(self.sigfigs)?;
        writer.write_u32::<LittleEndian>(self.snaplen)?;
        writer.write_u32::<LittleEndian>(self.network)?;
        Ok(())
    }
}

struct PcapPacketHeader {
    ts_sec: u32,
    ts_usec: u32,
    incl_len: u32,
    orig_len: u32,
}

impl PcapPacketHeader {
    fn new(ts_sec: u32, ts_usec: u32, packet_len: u32) -> Self {
        PcapPacketHeader {
            ts_sec,
            ts_usec,
            incl_len: packet_len,
            orig_len: packet_len,
        }
    }

    fn write<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        writer.write_u32::<LittleEndian>(self.ts_sec)?;
        writer.write_u32::<LittleEndian>(self.ts_usec)?;
        writer.write_u32::<LittleEndian>(self.incl_len)?;
        writer.write_u32::<LittleEndian>(self.orig_len)?;
        Ok(())
    }
}

fn get_timestamp() -> (u32, u32) {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    (now.as_secs() as u32, now.subsec_micros())
}

fn generate_dropped_packets(total_packets: u32, drop_percentage: f32) -> HashSet<u32> {
    use std::collections::hash_map::RandomState;
    use std::hash::{BuildHasher, Hasher};

    let mut dropped = HashSet::new();
    let drop_count = ((total_packets as f32 * drop_percentage) / 100.0).ceil() as u32;

    // Use current time as seed for pseudo-random selection
    let mut hasher = RandomState::new().build_hasher();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    hasher.write_u128(timestamp);
    let mut seed = hasher.finish();

    // Simple LCG (Linear Congruential Generator) for reproducible randomness
    let mut dropped_so_far = 0;
    while dropped_so_far < drop_count {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let packet_num = (seed % total_packets as u64) as u32 + 1;

        if !dropped.contains(&packet_num) {
            dropped.insert(packet_num);
            dropped_so_far += 1;
        }
    }

    dropped
}

fn print_packet_hex(packet: &[u8]) {
    println!("    Hex: ", );
    // Print header fields
    println!("      Dest MAC: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
             packet[0], packet[1], packet[2], packet[3], packet[4], packet[5]);
    println!("      Src MAC:  {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
             packet[6], packet[7], packet[8], packet[9], packet[10], packet[11]);
    println!("      EtherType: 0x{:02x}{:02x}", packet[12], packet[13]);
    println!("      TCI/AN: 0x{:02x}", packet[14]);
    println!("      SL:  0x{:02x}", packet[15]);
    println!("      PN:  0x{:02x}{:02x}{:02x}{:02x}", packet[16], packet[17], packet[18], packet[19]);
    println!("      SysID: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
             packet[20], packet[21], packet[22], packet[23], packet[24], packet[25]);
    println!("      PortID: 0x{:02x}{:02x}", packet[26], packet[27]);
}

fn main() -> std::io::Result<()> {
    // Configuration
    let output_file = "macsec_traffic.pcap";
    let src_mac = [0x00, 0x11, 0x22, 0x33, 0x44, 0x55];
    let dst_mac = [0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff];

    // Create two flows with different system identifiers but same port identifier
    let flow1_system_id = src_mac; // Sender 1
    let flow2_system_id = dst_mac; // Sender 2
    let port_id = 1u16;

    // Create pcap file
    let mut file = File::create(output_file)?;

    // Write PCAP global header
    let global_header = PcapGlobalHeader::new();
    global_header.write(&mut file)?;

    // Generate packets for each flow
    let packets_per_direction = 100;
    let drop_percentage = 5.0; // Drop 5% of packets

    println!("Generating MACsec traffic pcap file...");
    println!("Packets per direction: {}", packets_per_direction);
    println!("Drop percentage: {}%", drop_percentage);
    println!();

    // Generate the set of dropped packet numbers for Flow 2
    let dropped_packets = generate_dropped_packets(packets_per_direction, drop_percentage);
    println!("Dropped packet numbers in Flow 2: {:?}", {
        let mut v: Vec<_> = dropped_packets.iter().collect();
        v.sort();
        v
    });
    println!();

    // Flow 1: src -> dst (no drops)
    println!("Flow 1 (src -> dst): {} packets (no drops)", packets_per_direction);
    for pkt_num in 1..=packets_per_direction {
        let macsec_pkt = MACsecPacket::new(
            dst_mac,
            src_mac,
            pkt_num as u32,
            flow1_system_id,
            port_id,
        );

        let packet_data = macsec_pkt.to_bytes();
        let (ts_sec, ts_usec) = get_timestamp();
        let packet_header = PcapPacketHeader::new(ts_sec, ts_usec, packet_data.len() as u32);

        packet_header.write(&mut file)?;
        file.write_all(&packet_data)?;

        if pkt_num <= 5 || pkt_num > packets_per_direction - 5 {
            println!(
                "  Packet {}: PN={}, SysID={:02x?}, PortID={}",
                pkt_num, pkt_num, flow1_system_id, port_id
            );
        } else if pkt_num == 6 {
            println!("  ... (packets 6-{} omitted from output) ...", packets_per_direction - 5);
        }
    }

    println!();

    // Flow 2: dst -> src (bidirectional with drops)
    println!("Flow 2 (dst -> src): {} packets (with {} drops)", packets_per_direction, dropped_packets.len());
    for pkt_num in 1..=packets_per_direction {
        // Skip dropped packets
        if dropped_packets.contains(&pkt_num) {
            println!("  Packet {}: DROPPED", pkt_num);
            continue;
        }

        let macsec_pkt = MACsecPacket::new(
            src_mac,
            dst_mac,
            pkt_num as u32,
            flow2_system_id,
            port_id,
        );

        let packet_data = macsec_pkt.to_bytes();
        let (ts_sec, ts_usec) = get_timestamp();
        let packet_header = PcapPacketHeader::new(ts_sec, ts_usec, packet_data.len() as u32);

        packet_header.write(&mut file)?;
        file.write_all(&packet_data)?;

        if pkt_num <= 5 || pkt_num > packets_per_direction - 5 {
            println!(
                "  Packet {}: PN={}, SysID={:02x?}, PortID={}",
                pkt_num, pkt_num, flow2_system_id, port_id
            );
        } else if pkt_num == 6 {
            println!("  ... (packets 6-{} omitted from output) ...", packets_per_direction - 5);
        }
    }

    println!();
    println!("Pcap file created: {}", output_file);
    println!(
        "Total packets: {}",
        packets_per_direction * 2
    );

    Ok(())
}
