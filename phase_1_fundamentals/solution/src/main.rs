//! Example usage of the binary protocol parser
//!
//! Demonstrates creating, serializing, parsing, and validating messages

use binary_protocol_parser::{parse, parse_multiple, Message};

fn main() {
    println!("Binary Protocol Parser - Solution");
    println!("==================================\n");

    // Example 1: Create and serialize a message
    println!("Example 1: Create and Serialize");
    println!("---------------------------------");
    let msg = Message::new(1, 5, b"Hello".to_vec());
    println!("Created: {}", msg);

    let bytes = msg.to_bytes();
    println!("Serialized bytes: {:?}", bytes);
    println!("Serialized hex: {}", hex_encode(&bytes));
    println!();

    // Example 2: Parse a message
    println!("Example 2: Parse Message");
    println!("-----------------------");
    match parse(&bytes) {
        Ok(parsed) => {
            println!("Parsed successfully: {}", parsed);
            println!("  Version: {}", parsed.version);
            println!("  Type: {}", parsed.message_type);
            println!("  Payload: {:?}", String::from_utf8_lossy(&parsed.payload));
            println!("  Checksum: 0x{:02X}", parsed.checksum);
        }
        Err(e) => println!("Parse failed: {}", e),
    }
    println!();

    // Example 3: Validate message
    println!("Example 3: Message Validation");
    println!("-----------------------------");
    match msg.validate() {
        Ok(_) => println!("Message is valid ✓"),
        Err(e) => println!("Validation failed: {}", e),
    }
    println!();

    // Example 4: Parse multiple messages
    println!("Example 4: Parse Multiple Messages");
    println!("----------------------------------");
    let msg1 = Message::new(1, 1, b"First".to_vec());
    let msg2 = Message::new(1, 2, b"Second".to_vec());

    let mut combined = msg1.to_bytes();
    combined.extend_from_slice(&msg2.to_bytes());

    println!("Combined packet size: {} bytes", combined.len());
    match parse_multiple(&combined) {
        Ok(messages) => {
            println!("Parsed {} messages:", messages.len());
            for (i, msg) in messages.iter().enumerate() {
                println!(
                    "  [{}] Type {} - Payload: {:?}",
                    i + 1,
                    msg.message_type,
                    String::from_utf8_lossy(&msg.payload)
                );
            }
        }
        Err(e) => println!("Parse failed: {}", e),
    }
    println!();

    // Example 5: Error handling
    println!("Example 5: Error Handling");
    println!("------------------------");
    let invalid_packet = vec![0x02, 0x05, 0x00, 0x00, 0x00];  // Invalid version
    match parse(&invalid_packet) {
        Ok(_) => println!("Parse succeeded"),
        Err(e) => println!("Parse failed (expected): {}", e),
    }
    println!();

    // Example 6: "Hello World" example from spec
    println!("Example 6: \"Hello World\" from Protocol Spec");
    println!("-------------------------------------------");
    let hello_world = Message::new(1, 5, b"Hello World".to_vec());
    let hw_bytes = hello_world.to_bytes();

    println!("Message: {}", hello_world);
    println!("Hex bytes: {}", hex_encode(&hw_bytes));
    println!("Byte breakdown:");
    println!("  [0] Version: 0x{:02X} ({})", hw_bytes[0], hw_bytes[0]);
    println!("  [1] Type: 0x{:02X} ({})", hw_bytes[1], hw_bytes[1]);
    let len = u16::from_be_bytes([hw_bytes[2], hw_bytes[3]]);
    println!("  [2-3] Length: 0x{:04X} ({})", len, len);
    println!(
        "  [4..{}] Payload: {:?}",
        4 + len,
        String::from_utf8_lossy(&hw_bytes[4..4 + len as usize])
    );
    println!("  [{}] Checksum: 0x{:02X}", 4 + len, hw_bytes[4 + len as usize]);
    println!();

    println!("Run `cargo test` to execute the test suite.");
}

/// Helper function to encode bytes as hex string
fn hex_encode(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}
