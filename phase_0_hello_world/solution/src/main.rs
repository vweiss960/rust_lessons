// ============================================================================
// Phase 0 Solution: Hello World with Arguments
// ============================================================================
// This is the complete, working solution for the Phase 0 project.
// Compare your solution to this to understand the concepts.
// ============================================================================

fn main() {
    // Step 1: Print a welcome message
    // println! is a macro (note the !) that prints a line of text
    // The ! indicates it's a macro, not a regular function
    println!("Welcome to Rust!");

    // Step 2: Get command-line arguments
    // std::env::args() returns an iterator of command-line arguments
    // .collect() gathers them into a Vec<String> (a vector/list of strings)
    // Note: args[0] is always the program name
    let args: Vec<String> = std::env::args().collect();

    // Step 3: Check if any arguments were provided (besides the program name)
    // args.len() returns the number of arguments
    // If we have only 1 argument, that's just the program name, so print a message
    if args.len() > 1 {
        // We have arguments! Tell the user how many
        let argument_count = args.len() - 1;  // Subtract 1 for program name
        println!("You provided {} argument(s):", argument_count);

        // Step 4: Loop through arguments and print each one
        // .iter() creates an iterator over the vector
        // .skip(1) skips the first element (program name)
        // 'for arg in ...' loops through each argument
        for arg in args.iter().skip(1) {
            // Print each argument with a bullet point
            println!("  - {}", arg);
        }
    } else {
        // No arguments provided (only the program name)
        println!("No arguments provided. Try running:");
        println!("  cargo run -- Alice Bob");
    }
}

// ============================================================================
// EXPLANATION OF KEY CONCEPTS
// ============================================================================
//
// 1. fn main() - The entry point of every Rust program
//    Execution always starts here
//
// 2. println!() - A macro that prints text with a newline
//    The ! indicates it's a macro (special syntax-generating feature)
//    Compare to print!() which doesn't add a newline
//
// 3. let - Creates a variable (binding a name to a value)
//    Rust infers the type automatically
//    let name = "Alice";    // Type inferred: &str
//    let age = 30;          // Type inferred: i32
//
// 4. Vec<String> - A vector (resizable array) of Strings
//    Similar to Python's list of strings
//    Accessed with: vec[0], vec[1], etc.
//
// 5. std::env::args() - Gets command-line arguments from the OS
//    Returns an iterator (something you can loop over)
//    .collect() gathers it into a Vec<String>
//
// 6. .iter() and .skip(1) - Methods that transform collections
//    .iter() creates an iterator (safe borrowing)
//    .skip(1) skips the first element
//    They chain together (functional style)
//
// 7. for x in collection - Loop through items
//    'x' is a new variable each iteration
//    In Rust, this is very efficient and safe
//
// 8. {} in println! - Placeholder for values
//    println!("{}", value)      // Prints the value
//    println!("{} and {}", a, b) // Prints both values
//    println!("{name}")          // Prints variable directly (Rust 1.58+)
//
// ============================================================================
// HOW TO RUN THIS
// ============================================================================
//
// No arguments:
//   cargo run
//
// With arguments:
//   cargo run -- Alice
//   cargo run -- hello world
//   cargo run -- foo bar baz
//
// Note: The -- tells cargo to pass the following args to your program,
// not to cargo itself.
//
// ============================================================================
