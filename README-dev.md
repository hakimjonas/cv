# Development Guidelines for CV Project

This document outlines the development guidelines and best practices for the CV project.

## Build/Configuration Instructions

This project uses standard Rust tooling for building and configuration:

1. **Building the Project**:
   ```bash
   cargo build            # Development build
   cargo build --release  # Production build
   ```

2. **Running the Project**:
   ```bash
   cargo run              # Run in development mode
   cargo run --release    # Run in production mode
   ```

3. **Managing Dependencies**:
   - Add new dependencies to `Cargo.toml`
   - Run `cargo update` to update dependencies

## Testing Information

Testing is done using standard Rust testing tools:

1. **Running Tests**:
   ```bash
   cargo test             # Run all tests
   cargo test <test_name> # Run specific test
   cargo test -- --nocapture # Run tests with stdout output
   ```

2. **Test Coverage**:
   ```bash
   cargo install cargo-tarpaulin  # Install tarpaulin for coverage
   cargo tarpaulin                # Generate coverage report
   ```

3. **Writing Tests**:
   - Unit tests should be in the same file as the code they test
   - Integration tests should be placed in the `tests/` directory
   - Use descriptive test names following the pattern `test_<function_name>_<scenario>`

## Additional Development Information

### Code Style and Formatting

1. **Code Formatting**:
   ```bash
   cargo fmt              # Format code according to Rust style guidelines
   ```

2. **Linting**:
   ```bash
   cargo clippy           # Run the Rust linter
   ```

### Functional Programming Guidelines

This project follows functional programming principles:

1. **Immutability**:
   - ALWAYS use the `im` crate instead of standard data structures
   - Example:
     ```
     use im::{Vector, HashMap};

     // Instead of Vec<T>
     let immutable_vector = Vector::new();

     // Instead of HashMap<K, V>
     let immutable_map = HashMap::new();
     ```

2. **Pure Functions**:
   - Functions should not have side effects
   - Functions should return new data structures rather than modifying existing ones
   - Avoid mutable state when possible

3. **Function Composition**:
   - Use combinators like `map`, `filter`, `fold` instead of imperative loops
   - Chain operations using the method call syntax

4. **Error Handling**:
   - Use `Result` and `Option` types for error handling
   - Leverage the `?` operator for concise error propagation

### Dependency Management

- Add the `im` crate to your `Cargo.toml`:
  ```toml
  [dependencies]
  im = "15.1.0"  # Use the latest version
  ```

Remember to always prioritize immutability and functional programming patterns throughout the codebase.