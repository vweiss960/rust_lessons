#!/usr/bin/env python3
"""
Synthetic PCAP Generator for Stress Testing

Generates deterministic PCAP files with:
- Configurable number of packets
- Realistic packet diversity
- Controllable packet loss (gaps)
- Multiple protocol types
- Reproducible results (seedable)

Usage:
    python3 pcap_generator.py --packets 10000 --flows 1000 --output test.pcap
    python3 pcap_generator.py --packets 100000 --gap-rate 0.05 --seed 42 --output large_test.pcap
"""

import struct
import random
import time
import argparse
import sys
import json
import os
from dataclasses import dataclass
from typing import List, Tuple
from datetime import datetime, timezone


@dataclass
class PcapGlobalHeader:
    """PCAP global header structure"""
    magic_number: int = 0xa1b2c3d4  # Little-endian PCAP magic
    version_major: int = 2
    version_minor: int = 4
    thiszone: int = 0
    sigfigs: int = 0
    snaplen: int = 65535
    network: int = 1  # Ethernet

    def pack(self) -> bytes:
        """Pack to binary format (little-endian)"""
        return struct.pack(
            '<IHHIIII',
            self.magic_number,
            self.version_major,
            self.version_minor,
            self.thiszone,
            self.sigfigs,
            self.snaplen,
            self.network
        )


@dataclass
class PcapPacketHeader:
    """PCAP packet header structure"""
    ts_sec: int
    ts_usec: int
    incl_len: int  # Included length (what we captured)
    orig_len: int  # Original length (actual packet size)

    def pack(self) -> bytes:
        """Pack to binary format (little-endian)"""
        return struct.pack(
            '<IIII',
            self.ts_sec,
            self.ts_usec,
            self.incl_len,
            self.orig_len
        )


class EthernetFrame:
    """Minimal Ethernet frame builder"""

    def __init__(self, dst_mac: str, src_mac: str, ethertype: int, payload: bytes):
        self.dst_mac = self._parse_mac(dst_mac)
        self.src_mac = self._parse_mac(src_mac)
        self.ethertype = ethertype
        self.payload = payload

    @staticmethod
    def _parse_mac(mac_str: str) -> bytes:
        """Parse MAC address string to bytes"""
        if isinstance(mac_str, bytes):
            return mac_str
        return bytes.fromhex(mac_str.replace(':', ''))

    def pack(self) -> bytes:
        """Pack Ethernet frame"""
        return self.dst_mac + self.src_mac + struct.pack('>H', self.ethertype) + self.payload


class IPv4Packet:
    """Minimal IPv4 packet builder"""

    def __init__(self, src_ip: str, dst_ip: str, protocol: int, ttl: int = 64, payload: bytes = b''):
        self.src_ip = self._parse_ip(src_ip)
        self.dst_ip = self._parse_ip(dst_ip)
        self.protocol = protocol
        self.ttl = ttl
        self.payload = payload
        self.identification = random.randint(0, 65535)

    @staticmethod
    def _parse_ip(ip_str: str) -> bytes:
        """Parse IP address string to bytes"""
        parts = ip_str.split('.')
        return bytes(int(p) for p in parts)

    def pack(self) -> bytes:
        """Pack IPv4 packet (without checksum calculation for simplicity)"""
        ihl = 5  # Header length in 32-bit words
        version_ihl = (4 << 4) | ihl  # Version 4, IHL 5
        dscp_ecn = 0
        total_length = 20 + len(self.payload)
        flags_fragment = 0  # DF=0, MF=0, offset=0 (16-bit combined)
        checksum = 0  # Skip checksum for synthetic packets

        # IPv4 header format: Version+IHL, DSCP+ECN, Total Length, Identification, Flags+FragmentOffset, TTL, Protocol, Checksum, SrcIP, DstIP
        header = struct.pack(
            '>BHHHHBBH4s4s',
            version_ihl,        # 1 byte: version(4 bits) + IHL(4 bits)
            dscp_ecn,           # 1 byte: DSCP(6 bits) + ECN(2 bits)
            total_length,       # 2 bytes (H)
            self.identification,# 2 bytes (H)
            flags_fragment,     # 2 bytes (H): Flags(3 bits) + Fragment Offset(13 bits)
            self.ttl,           # 1 byte (B)
            self.protocol,      # 1 byte (B)
            checksum,           # 2 bytes (H)
            self.src_ip,        # 4 bytes (4s)
            self.dst_ip         # 4 bytes (4s)
        )

        return header + self.payload


class MACsecPacket:
    """MACsec packet builder (simplified)"""

    def __init__(self, packet_number: int, sci: int = 0, payload: bytes = b''):
        self.packet_number = packet_number
        self.sci = sci  # Secure Channel Identifier (8 bytes)
        self.payload = payload

    def pack(self) -> bytes:
        """Pack MACsec packet (TCI/AN + packet number + SCI + payload)"""
        # TCI (1 byte) bit layout (from MSB to LSB):
        # Bits 7-6: Version (00)
        # Bit 5: SC (Security Channel present) = 1 (SCI is included)
        # Bit 4: E (Encryption) = 0
        # Bits 3-2: SCI/Short SCI length = 00 (full 8-byte SCI)
        # Bits 1-0: AN (Association Number) = 01
        tci = 0x21  # Binary: 0010 0001 (Version 0, SC=1, SCI length 00, AN=01)
        # AN (1 byte): association number
        an = 0x00
        # Packet number (4 bytes, big-endian)
        pkt_num_bytes = struct.pack('>I', self.packet_number)
        # SCI (8 bytes, big-endian) - Secure Channel Identifier for flow identification
        sci_bytes = struct.pack('>Q', self.sci)

        return bytes([tci, an]) + pkt_num_bytes + sci_bytes + self.payload


class IPsecPacket:
    """IPsec ESP packet builder (simplified)"""

    def __init__(self, spi: int, seq_num: int, payload: bytes = b''):
        self.spi = spi
        self.seq_num = seq_num
        self.payload = payload

    def pack(self) -> bytes:
        """Pack IPsec ESP packet (SPI + sequence number + payload)"""
        return struct.pack('>II', self.spi, self.seq_num) + self.payload


class PcapWriter:
    """Write PCAP files in binary format"""

    def __init__(self, filename: str, verbose: bool = False):
        self.filename = filename
        self.file = open(filename, 'wb')
        self.verbose = verbose
        self.packet_count = 0
        self.start_time = None

        # Write global header
        global_header = PcapGlobalHeader()
        self.file.write(global_header.pack())

        if self.verbose:
            print(f"[PcapWriter] Created PCAP file: {filename}")

    def write_packet(self, packet_data: bytes, timestamp: float = None):
        """Write a single packet to PCAP file"""
        if timestamp is None:
            if self.start_time is None:
                self.start_time = time.time()
            timestamp = time.time()

        # Convert timestamp to seconds and microseconds
        ts_sec = int(timestamp)
        ts_usec = int((timestamp - ts_sec) * 1_000_000)

        # Create and write packet header
        pkt_header = PcapPacketHeader(
            ts_sec=ts_sec,
            ts_usec=ts_usec,
            incl_len=len(packet_data),
            orig_len=len(packet_data)
        )
        self.file.write(pkt_header.pack())
        self.file.write(packet_data)

        self.packet_count += 1

        if self.verbose and self.packet_count % 1000 == 0:
            print(f"[PcapWriter] Written {self.packet_count} packets...")

    def close(self):
        """Close PCAP file"""
        self.file.close()
        if self.verbose:
            print(f"[PcapWriter] Closed PCAP file: {self.packet_count} packets written")


def generate_synthetic_packets(
    num_packets: int,
    num_flows: int,
    gap_rate: float = 0.05,
    seed: int = None,
    verbose: bool = False,
    protocol: str = 'macsec',
    macsec_ratio: float = 0.5,
    ipsec_ratio: float = 0.3,
    generic_ratio: float = 0.2
) -> Tuple[List[bytes], dict]:
    """
    Generate synthetic packets with:
    - Multiple flows (different source MACs for MACsec, IPs for IPsec)
    - Mixed protocol support (MACsec, IPsec, generic L3)
    - Realistic packet sizes (64-1500 bytes)
    - Injected gaps (packet loss simulation)

    Args:
        num_packets: Total packets to generate
        num_flows: Number of unique flows
        gap_rate: Proportion of packets to skip (0.05 = 5% gaps)
        seed: Random seed for reproducibility
        verbose: Print progress
        protocol: 'macsec', 'ipsec', 'generic', or 'mixed' (default: 'macsec')
        macsec_ratio: Proportion of MACsec packets (0.0-1.0, used with 'mixed')
        ipsec_ratio: Proportion of IPsec packets (0.0-1.0, used with 'mixed')
        generic_ratio: Proportion of generic IPv4 packets (0.0-1.0, used with 'mixed')

    Returns:
        Tuple of (packet list, gap statistics dict)
    """

    if seed is not None:
        random.seed(seed)

    packets = []
    flow_states = {}

    # Determine protocol distribution
    if protocol == 'mixed':
        # Normalize ratios to sum to 1.0
        total = macsec_ratio + ipsec_ratio + generic_ratio
        if total == 0:
            macsec_ratio = 0.5
            ipsec_ratio = 0.3
            generic_ratio = 0.2
            total = 1.0
        macsec_ratio /= total
        ipsec_ratio /= total
        generic_ratio /= total
        protocols = ['macsec', 'ipsec', 'generic']
        protocols_weights = [macsec_ratio, ipsec_ratio, generic_ratio]
    else:
        protocols = [protocol]
        protocols_weights = [1.0]

    # Initialize flows with gap tracking
    for flow_id in range(num_flows):
        flow_states[flow_id] = {
            'seq_num': random.randint(1, 1000000),
            'spi': random.randint(256, 65536),
            'sci': flow_id,  # Use flow_id as SCI for unique flow identification in MACsec
            'protocol': random.choices(protocols, weights=protocols_weights)[0],
            'src_ip': f'192.168.{(flow_id >> 8) & 0xFF}.{flow_id & 0xFF}',
            'dst_ip': f'10.0.{(flow_id >> 8) & 0xFF}.{flow_id & 0xFF}',
            'lost_packets': 0,  # Track lost packets per flow
            'gaps': 0,  # Track gap count per flow
        }

    packet_sizes = [64, 128, 256, 512, 1024, 1500]

    if verbose:
        print(f"[Generator] Generating {num_packets} packets across {num_flows} flows...")
        print(f"[Generator] Gap rate: {gap_rate * 100:.1f}%")

    for pkt_idx in range(num_packets):
        flow_id = pkt_idx % num_flows
        flow = flow_states[flow_id]

        # Inject gaps (skip sequence numbers to simulate packet loss)
        if random.random() < gap_rate:
            gap_size = random.randint(2, 20)  # Skip 2-20 packets
            flow['seq_num'] += gap_size
            flow['lost_packets'] += gap_size
            flow['gaps'] += 1

        # Randomize packet size
        payload_size = random.choice(packet_sizes)

        # Choose protocol
        proto_type = flow['protocol']

        if proto_type == 'macsec':
            # Build MACsec packet
            # Note: MACsec payload now includes: TCI(1) + AN(1) + PacketNum(4) + SCI(8) + data
            macsec = MACsecPacket(
                packet_number=flow['seq_num'],
                sci=flow['sci'],  # Unique flow identifier
                payload=b'\x00' * (payload_size - 14)  # 14 bytes for TCI+AN+pkt_num+SCI header
            )
            payload = macsec.pack()
            ethertype = 0x88E5  # MACsec

            # Ethernet + MACsec
            # Use flow-specific source MAC for additional flow differentiation
            src_mac_bytes = (flow['sci']).to_bytes(6, 'big')
            src_mac = ':'.join(f'{b:02x}' for b in src_mac_bytes)
            eth = EthernetFrame(
                dst_mac='00:11:22:33:44:55',
                src_mac=src_mac,
                ethertype=ethertype,
                payload=payload
            )

        elif proto_type == 'ipsec':
            # Build IPsec ESP packet
            esp = IPsecPacket(
                spi=flow['spi'],
                seq_num=flow['seq_num'],
                payload=b'\x00' * (payload_size - 8)  # 8 bytes for SPI+seq header
            )
            payload = esp.pack()

            # Ethernet + IPv4 + IPsec (ESP protocol 50)
            ipv4 = IPv4Packet(
                src_ip=flow['src_ip'],
                dst_ip=flow['dst_ip'],
                protocol=50,  # ESP
                payload=payload
            )
            eth = EthernetFrame(
                dst_mac='00:11:22:33:44:55',
                src_mac='66:77:88:99:aa:bb',
                ethertype=0x0800,  # IPv4
                payload=ipv4.pack()
            )

        else:  # generic
            # Build generic IPv4 packet (UDP-like payload, not IPsec)
            payload = b'\x00' * (payload_size - 8)  # Generic payload

            # Ethernet + IPv4 (protocol 17 = UDP)
            ipv4 = IPv4Packet(
                src_ip=flow['src_ip'],
                dst_ip=flow['dst_ip'],
                protocol=17,  # UDP
                payload=payload
            )
            eth = EthernetFrame(
                dst_mac='00:11:22:33:44:55',
                src_mac='66:77:88:99:aa:bb',
                ethertype=0x0800,  # IPv4
                payload=ipv4.pack()
            )

        packets.append(eth.pack())

        # Increment sequence number
        flow['seq_num'] += 1

        if verbose and (pkt_idx + 1) % 1000 == 0:
            print(f"[Generator] Generated {pkt_idx + 1} packets...")

    if verbose:
        print(f"[Generator] Complete: {len(packets)} packets generated")

    # Build gap statistics
    gap_stats = {
        'total_packets_generated': len(packets),
        'total_flows': num_flows,
        'gap_rate': gap_rate,
        'flows': {}
    }

    for flow_id, state in flow_states.items():
        gap_stats['flows'][str(flow_id)] = {
            'protocol': state['protocol'],
            'lost_packets': state['lost_packets'],
            'gaps_count': state['gaps'],
        }

    return packets, gap_stats


def main():
    parser = argparse.ArgumentParser(
        description='Generate synthetic PCAP files for stress testing'
    )
    parser.add_argument(
        '--packets',
        type=int,
        default=1000,
        help='Number of packets to generate (default: 1000)'
    )
    parser.add_argument(
        '--flows',
        type=int,
        default=None,
        help='Number of unique flows (default: packets / 10)'
    )
    parser.add_argument(
        '--gap-rate',
        type=float,
        default=0.05,
        help='Packet loss rate (0.05 = 5%, default: 0.05)'
    )
    parser.add_argument(
        '--seed',
        type=int,
        default=None,
        help='Random seed for reproducibility'
    )
    parser.add_argument(
        '--output',
        required=True,
        help='Output PCAP filename'
    )
    parser.add_argument(
        '--verbose',
        action='store_true',
        help='Enable verbose output'
    )
    parser.add_argument(
        '--protocol',
        choices=['macsec', 'ipsec', 'generic', 'mixed'],
        default='macsec',
        help='Protocol to use: macsec, ipsec, generic (IPv4/UDP), or mixed (default: macsec)'
    )
    parser.add_argument(
        '--macsec-ratio',
        type=float,
        default=0.5,
        help='Proportion of MACsec packets in mixed mode (0.0-1.0, default: 0.5)'
    )
    parser.add_argument(
        '--ipsec-ratio',
        type=float,
        default=0.3,
        help='Proportion of IPsec packets in mixed mode (0.0-1.0, default: 0.3)'
    )
    parser.add_argument(
        '--generic-ratio',
        type=float,
        default=0.2,
        help='Proportion of generic IPv4 packets in mixed mode (0.0-1.0, default: 0.2)'
    )

    args = parser.parse_args()

    num_flows = args.flows if args.flows else args.packets // 10

    if args.verbose:
        print(f"Configuration:")
        print(f"  Packets: {args.packets}")
        print(f"  Flows: {num_flows}")
        print(f"  Protocol: {args.protocol}")
        if args.protocol == 'mixed':
            print(f"  Protocol mix: MACsec {args.macsec_ratio*100:.0f}%, IPsec {args.ipsec_ratio*100:.0f}%, Generic {args.generic_ratio*100:.0f}%")
        print(f"  Gap rate: {args.gap_rate * 100:.1f}%")
        print(f"  Seed: {args.seed}")
        print(f"  Output: {args.output}")
        print()

    # Generate packets
    packets, gap_stats = generate_synthetic_packets(
        num_packets=args.packets,
        num_flows=num_flows,
        gap_rate=args.gap_rate,
        seed=args.seed,
        verbose=args.verbose,
        protocol=args.protocol,
        macsec_ratio=args.macsec_ratio,
        ipsec_ratio=args.ipsec_ratio,
        generic_ratio=args.generic_ratio
    )

    # Write to PCAP file
    writer = PcapWriter(args.output, verbose=args.verbose)
    for packet in packets:
        writer.write_packet(packet)
    writer.close()

    # Write gap statistics to JSON file
    json_output = os.path.splitext(args.output)[0] + '.json'
    with open(json_output, 'w') as f:
        json.dump(gap_stats, f, indent=2)

    if args.verbose:
        print(f"[Generator] Gap statistics written to: {json_output}")

    if args.verbose:
        file_size = os.path.getsize(args.output)
        print(f"\nOutput file: {args.output}")
        print(f"File size: {file_size / 1024:.1f} KB")
        print(f"Packets: {len(packets)}")
        print(f"Avg packet size: {file_size / len(packets):.0f} bytes")


if __name__ == '__main__':
    main()
