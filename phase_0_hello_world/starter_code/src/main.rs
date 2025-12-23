// ============================================================================
// Phase 0 Project: Hello World with Arguments
// ============================================================================
//
// YOUR TASK:
// 1. Print a greeting message when the program starts
// 2. Read command-line arguments
// 3. Echo back the arguments to the user
// 4. Handle the case where no arguments are provided
//
// HOW TO RUN:
// cargo run                              # No arguments
// cargo run -- Alice                     # With one argument
// cargo run -- hello world rust          # With multiple arguments
//
// EXPECTED OUTPUT:
// When run with: cargo run -- Alice Bob
// Should print something like:
//   Welcome to Rust!
//   You provided 2 arguments:
//   - Alice
//   - Bob
//
// ============================================================================

fn main() {
    // TODO: Print a welcome message here
    // Hint: Use println!("...") to print text


    // TODO: Get command-line arguments
    // Hint: Use std::env::args().collect() and store in a variable
    // Example: let args: Vec<String> = std::env::args().collect();


    // TODO: Check if any arguments were provided
    // Hint: Use an if statement to check args.len()


    // TODO: Loop through the arguments (skip the first one—it's the program name)
    // Hint: Use a for loop with args.iter().skip(1)
    // Example: for arg in args.iter().skip(1) { ... }


    // TODO: Print each argument
    // Hint: Use println!("- {}", arg) inside the loop


    // BONUS: Can you print a message when no arguments are provided?
    // Hint: Use an else clause in your if statement
}
