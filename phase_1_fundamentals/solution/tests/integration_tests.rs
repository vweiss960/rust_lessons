/// Integration tests for the binary protocol parser
///
/// These tests verify end-to-end functionality with complete protocol messages

use binary_protocol_parser::{parse, parse_multiple, Message};

/// Test parsing a "Hello World" message as specified in the protocol spec
///
/// This demonstrates parsing a real message that follows the protocol format exactly.
#[test]
fn test_parse_hello_world() {
    // Construct "Hello World" message as specified
    // Version: 1
    // Type: 5
    // Payload: "Hello World" (11 bytes)
    // Checksum: XOR of all payload bytes

    let payload = b"Hello World";
    let expected_checksum = payload.iter().fold(0u8, |acc, &b| acc ^ b);

    let packet = vec![
        0x01,                                    // Version 1
        0x05,                                    // Message Type 5
        0x00, 0x0B,                              // Length: 11 (0x0B in big-endian)
        0x48, 0x65, 0x6C, 0x6C, 0x6F,            // "Hello"
        0x20,                                    // space
        0x57, 0x6F, 0x72, 0x6C, 0x64,            // "World"
        expected_checksum,                       // Calculated checksum
    ];

    // Parse the packet
    let msg = parse(&packet).expect("Failed to parse Hello World message");

    // Verify all fields
    assert_eq!(msg.version, 1);
    assert_eq!(msg.message_type, 5);
    assert_eq!(msg.payload.len(), 11);
    assert_eq!(msg.payload, payload);
    assert_eq!(msg.checksum, expected_checksum);

    // Verify display format includes version, type, and length
    let display_str = msg.to_string();
    assert!(display_str.contains("v1"));
    assert!(display_str.contains("type=5"));
    assert!(display_str.contains("payload_len=11"));
}

/// Test round-trip: create message -> serialize -> parse -> verify
///
/// This ensures that serialization and parsing are inverses of each other.
#[test]
fn test_round_trip() {
    // Create a message with various types of data
    let original_payload = vec![1, 2, 3, 4, 5, 0xFF, 0xAA, 0x55, 0x00];
    let original = Message::new(1, 10, original_payload.clone());

    // Serialize to bytes
    let bytes = original.to_bytes();

    // Parse back
    let parsed = parse(&bytes).expect("Failed to parse serialized message");

    // Verify everything matches
    assert_eq!(parsed.version, original.version);
    assert_eq!(parsed.message_type, original.message_type);
    assert_eq!(parsed.payload, original.payload);
    assert_eq!(parsed.checksum, original.checksum);

    // Double check by serializing again - should be identical
    assert_eq!(parsed.to_bytes(), bytes);
}

/// Test parsing multiple consecutive messages
///
/// Verifies that the parser can handle multiple messages in a single byte stream.
#[test]
fn test_parse_multiple_messages() {
    // Create several different messages
    let msg1 = Message::new(1, 1, vec![0x11, 0x22, 0x33]);
    let msg2 = Message::new(1, 2, vec![0xAA, 0xBB, 0xCC, 0xDD]);
    let msg3 = Message::new(1, 3, vec![]);  // Empty payload

    // Combine them into one byte stream
    let mut combined = msg1.to_bytes();
    combined.extend_from_slice(&msg2.to_bytes());
    combined.extend_from_slice(&msg3.to_bytes());

    // Parse all at once
    let messages = parse_multiple(&combined).expect("Failed to parse multiple messages");

    // Verify we got all messages
    assert_eq!(messages.len(), 3);

    // Verify each message
    assert_eq!(messages[0].message_type, 1);
    assert_eq!(messages[0].payload, vec![0x11, 0x22, 0x33]);

    assert_eq!(messages[1].message_type, 2);
    assert_eq!(messages[1].payload, vec![0xAA, 0xBB, 0xCC, 0xDD]);

    assert_eq!(messages[2].message_type, 3);
    assert_eq!(messages[2].payload.is_empty(), true);
}

/// Test parsing empty payload message
///
/// Verifies that messages with no payload are handled correctly.
#[test]
fn test_parse_empty_payload() {
    let packet = vec![
        0x01,           // Version
        0x05,           // Type
        0x00, 0x00,     // Length = 0 (no payload)
        0x00,           // Checksum of empty data
    ];

    let msg = parse(&packet).expect("Failed to parse empty payload message");
    assert_eq!(msg.version, 1);
    assert_eq!(msg.message_type, 5);
    assert_eq!(msg.payload.len(), 0);
    assert_eq!(msg.checksum, 0);  // XOR of nothing is 0
}

/// Test parsing message with maximum payload size
///
/// Verifies that large payloads are handled correctly.
#[test]
fn test_parse_large_payload() {
    // Create a message with 1000 bytes of payload
    let large_payload = vec![0x42; 1000];
    let msg = Message::new(1, 99, large_payload.clone());

    let bytes = msg.to_bytes();
    let parsed = parse(&bytes).expect("Failed to parse large payload");

    assert_eq!(parsed.payload.len(), 1000);
    assert_eq!(parsed.payload, large_payload);
}

/// Test that invalid versions are rejected
#[test]
fn test_invalid_version() {
    for invalid_version in &[0, 2, 3, 255] {
        let packet = vec![
            *invalid_version,  // Invalid version
            0x05,
            0x00, 0x00,
            0x00,
        ];

        let result = parse(&packet);
        assert!(
            result.is_err(),
            "Should reject version {}",
            invalid_version
        );
    }
}

/// Test that checksum mismatches are detected
#[test]
fn test_checksum_validation() {
    let packet = vec![
        0x01,           // Version
        0x05,           // Type
        0x00, 0x03,     // Length = 3
        0x11, 0x22, 0x33,  // Payload
        0xFF,           // Wrong checksum (0x11 ^ 0x22 ^ 0x33 != 0xFF)
    ];

    let result = parse(&packet);
    assert!(result.is_err(), "Should detect checksum mismatch");
}

/// Test binary data with null bytes
///
/// Ensures the parser handles binary data containing zeros correctly.
#[test]
fn test_binary_payload_with_nulls() {
    let payload = vec![0x00, 0x01, 0x00, 0x02, 0x00, 0xFF];
    let msg = Message::new(1, 7, payload.clone());

    let bytes = msg.to_bytes();
    let parsed = parse(&bytes).expect("Failed to parse binary with nulls");

    assert_eq!(parsed.payload, payload);
}

/// Test XOR checksum calculation
#[test]
fn test_checksum_correctness() {
    // Test specific known checksums
    let tests = vec![
        (vec![], 0u8),                           // Empty: 0
        (vec![5], 5u8),                          // Single byte: 5
        (vec![5, 5], 0u8),                       // 5 ^ 5 = 0
        (vec![1, 2, 3], 0u8),                    // 1 ^ 2 ^ 3 = 0
        (vec![0xFF, 0xFF], 0u8),                 // All ones: 0
        (vec![0xAA, 0x55], 0xFFu8),              // Complementary: 0xFF
    ];

    for (payload, expected_checksum) in tests {
        let msg = Message::new(1, 1, payload.clone());
        assert_eq!(
            msg.checksum, expected_checksum,
            "Checksum mismatch for payload {:?}",
            payload
        );

        // Verify by round-trip
        let bytes = msg.to_bytes();
        let parsed = parse(&bytes).expect("Failed round-trip");
        assert_eq!(parsed.checksum, expected_checksum);
    }
}

/// Test that message displays correctly
#[test]
fn test_message_display() {
    let msg = Message::new(1, 42, vec![1, 2, 3, 4, 5]);
    let display = format!("{}", msg);

    assert!(display.contains("v1"));
    assert!(display.contains("type=42"));
    assert!(display.contains("payload_len=5"));
}

/// Stress test with many messages
#[test]
fn test_stress_many_messages() {
    let mut combined = Vec::new();

    // Create and combine 100 messages
    for i in 0..100 {
        let payload = vec![(i % 256) as u8; (i + 1) % 10];
        let msg = Message::new(1, (i % 256) as u8, payload);
        combined.extend_from_slice(&msg.to_bytes());
    }

    // Parse all of them
    let messages = parse_multiple(&combined).expect("Stress test failed");
    assert_eq!(messages.len(), 100);

    // Spot check some messages
    assert_eq!(messages[0].message_type, 0);
    assert_eq!(messages[50].message_type, 50);
    assert_eq!(messages[99].message_type, 99);
}
