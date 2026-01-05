/// Example application using the binary protocol parser
///
/// This demonstrates how to use the parser library in a practical application.

use binary_protocol_parser::{parse, Message};

fn main() {
    println!("Binary Protocol Parser - Starter Code");
    println!("=====================================\n");

    // TODO: Create a sample message
    // Example of how to construct a message manually:
    // let msg = Message::new(1, 5, vec![72, 101, 108, 108, 111]);  // "Hello"
    // println!("{}", msg);

    // TODO: Create a sample binary packet and parse it
    // Example of how to create protocol bytes:
    // let packet = vec![
    //     0x01,        // Version 1
    //     0x05,        // Message Type 5
    //     0x00, 0x05,  // Length: 5 bytes (big-endian)
    //     0x48, 0x65, 0x6C, 0x6C, 0x6F,  // "Hello"
    //     0xXX,        // Checksum (to be calculated)
    // ];
    // match parse(&packet) {
    //     Ok(msg) => println!("Parsed: {}", msg),
    //     Err(e) => println!("Parse error: {}", e),
    // }

    println!("Run `cargo test` to execute tests.");
    println!("See lib.rs for the parser implementation.");
}
