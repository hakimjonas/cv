# Database Functionality and Data Privacy Features

## Changes Made

Added `#[allow(dead_code)]` attributes to the following unused constants and methods in `src/config.rs`:

1. `DEFAULT_DB_STORAGE` constant
2. `is_public` method
3. `db_storage` method
4. `store_in_db` method

## Explanation

These constants and methods are part of the database functionality and data privacy features of the application. They are currently not being used in the codebase, which is why `cargo clippy` was generating warnings about them.

The database functionality appears to be partially implemented but not fully integrated yet. The application has command-line arguments for:

- `--migrate-to-db`: Migrates CV data from JSON to SQLite database
- `--use-db`: Determines whether to load CV data from the database or from JSON
- `--db-path`: Allows specifying a custom database path
- `--public-data`: Allows specifying custom public data settings
- `--db-storage`: Allows specifying custom database storage settings

However, the actual filtering of CV data based on these settings is not implemented yet. There's a comment in `main.rs` indicating that "in a real implementation, we would create a filtered copy of the CV data based on the public_data configuration."

Rather than removing these constants and methods, I've added `#[allow(dead_code)]` attributes to suppress the warnings. This keeps the code in place for future use as the database functionality is more fully implemented.

## Future Work

To fully implement the database functionality and data privacy features, the following steps could be taken:

1. Implement the actual filtering of CV data based on the `public_data` configuration
2. Use the `store_in_db` method to determine what data should be stored in the database
3. Add tests for the database functionality and data privacy features
4. Update the documentation to explain how to use these features