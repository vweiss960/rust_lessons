/// Integration tests for the binary protocol parser
///
/// These tests verify that the parser works correctly with complete protocol messages

use binary_protocol_parser::{parse, Message};

// TODO: Add at least 2 integration tests

// Test 1: Parse a valid "Hello World" message
#[test]
fn test_parse_hello_world() {
    // The protocol format:
    // [0x01] Version 1
    // [0x05] Message Type 5
    // [0x00, 0x0A] Length: 10 bytes (big-endian)
    // [0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x57, 0x6F, 0x72, 0x6C, 0x64] "Hello World" (11 bytes)
    //        H    e    l    l    o    space W    o    r    l    d
    // [0xXX] Checksum: XOR of all payload bytes

    // TODO: Calculate the checksum
    // XOR: 0x48 ^ 0x65 ^ 0x6C ^ 0x6C ^ 0x6F ^ 0x20 ^ 0x57 ^ 0x6F ^ 0x72 ^ 0x6C ^ 0x64
    // Hint: This should be calculated, not hardcoded

    let packet = vec![
        0x01,                                          // Version
        0x05,                                          // Type
        0x00, 0x0B,                                    // Length: 11 (0x0B = 11)
        0x48, 0x65, 0x6C, 0x6C, 0x6F,                 // "Hello"
        0x20,                                          // space
        0x57, 0x6F, 0x72, 0x6C, 0x64,                 // "World"
        0x00,  // TODO: Replace with correct checksum
    ];

    // TODO: Parse and verify
    // match parse(&packet) {
    //     Ok(msg) => {
    //         assert_eq!(msg.version, 1);
    //         assert_eq!(msg.message_type, 5);
    //         assert_eq!(msg.payload.len(), 11);
    //     }
    //     Err(e) => panic!("Parse failed: {}", e),
    // }
}

// Test 2: Round-trip test (create message -> serialize -> parse -> verify)
#[test]
fn test_round_trip() {
    // TODO: Create a message
    let original = Message::new(1, 10, vec![1, 2, 3, 4, 5]);

    // TODO: Serialize to bytes
    let bytes = original.to_bytes();

    // TODO: Parse back
    // match parse(&bytes) {
    //     Ok(parsed) => {
    //         assert_eq!(parsed.version, original.version);
    //         assert_eq!(parsed.message_type, original.message_type);
    //         assert_eq!(parsed.payload, original.payload);
    //     }
    //     Err(e) => panic!("Round-trip failed: {}", e),
    // }
}
