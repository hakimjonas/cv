# Security Considerations for CV Generator

This document outlines security considerations and configuration options for the CV Generator application.

## Data Storage and Privacy

The CV Generator application stores data in two main locations:

1. **JSON Files**: By default, CV data is stored in JSON files in the `data` directory.
2. **SQLite Database**: Optionally, CV data can be stored in a SQLite database.

### Configuration Options

The application provides several configuration options to control what data is stored where and what data is publicly visible:

#### Public Data

You can control which fields are publicly visible by setting the `public_data` configuration option. This is a comma-separated list of field names.

Default public data fields:
```
name,title,summary,experiences,education,skill_categories,projects,languages,certifications
```

To change this, use the `--public-data` command-line option:
```bash
cv --public-data "name,title,summary,projects"
```

#### Database Storage

You can control which fields are stored in the database by setting the `db_storage` configuration option. This is a comma-separated list of field names.

Default database storage fields:
```
personal_info,experiences,education,skill_categories,projects,languages,certifications,github_sources
```

To change this, use the `--db-storage` command-line option:
```bash
cv --db-storage "experiences,education,projects"
```

## Sensitive Information

The following fields may contain sensitive information and should be handled with care:

- **Email**: Personal email address
- **Phone**: Personal phone number
- **Location**: Physical address or location
- **GitHub Token**: API token for GitHub

### Secure Storage of Tokens

GitHub API tokens are stored securely using the system's keyring when available, or in an encrypted file as a fallback.

To set a GitHub token:
```bash
cv --set-token <your-token>
```

To remove a GitHub token:
```bash
cv --remove-token
```

## Best Practices

1. **Limit Public Data**: Only include fields that you want to be publicly visible in the `public_data` configuration.
2. **Use Database for Sensitive Data**: Store sensitive information in the database rather than JSON files when possible.
3. **Secure Your Database**: Ensure that your SQLite database file has appropriate file permissions.
4. **Use Environment Variables**: For CI/CD environments, use environment variables rather than storing tokens in files.
5. **Regular Audits**: Regularly review what data is being stored and where.

## Implementation Details

The application uses the following security measures:

1. **Keyring Integration**: Uses the system's keyring (when available) to securely store tokens.
2. **Fallback Encryption**: When the keyring is not available, tokens are stored in an encrypted file.
3. **Configuration Controls**: Provides fine-grained control over what data is stored where.
4. **Immutable Data Structures**: Uses immutable data structures to prevent accidental data modification.