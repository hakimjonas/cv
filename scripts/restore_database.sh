#!/bin/bash
#
# SQLite Database Restoration Script
#
# This script restores a SQLite database from a backup file.
# It supports both compressed and uncompressed backups.
# 
# Usage: ./restore_database.sh [options]
#
# Options:
#   -b, --backup       Path to the backup file (required)
#   -d, --database     Path to the target SQLite database file (required)
#   -t, --test         Test restoration to a temporary database first (default: true)
#   -f, --force        Force restoration without confirmation (default: false)
#   -v, --verify       Verify database integrity after restoration (default: true)
#   -m, --monitor      Send monitoring notifications (default: false)
#   -h, --help         Show this help message
#
# Examples:
#   ./restore_database.sh --backup /backups/backup-daily-20250723-120000.sql.gz --database /app/data/blog.db
#   ./restore_database.sh --backup /backups/backup-daily-20250723-120000.sql --database /app/data/blog.db --test false
#   ./restore_database.sh --backup /backups/backup-daily-20250723-120000.sql.gz --database /app/data/blog.db --force true

set -e

# Default values
BACKUP=""
DATABASE=""
TEST=true
FORCE=false
VERIFY=true
MONITOR=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    -b|--backup)
      BACKUP="$2"
      shift 2
      ;;
    -d|--database)
      DATABASE="$2"
      shift 2
      ;;
    -t|--test)
      TEST="$2"
      shift 2
      ;;
    -f|--force)
      FORCE="$2"
      shift 2
      ;;
    -v|--verify)
      VERIFY="$2"
      shift 2
      ;;
    -m|--monitor)
      MONITOR="$2"
      shift 2
      ;;
    -h|--help)
      echo "Usage: $0 [options]"
      echo ""
      echo "Options:"
      echo "  -b, --backup       Path to the backup file (required)"
      echo "  -d, --database     Path to the target SQLite database file (required)"
      echo "  -t, --test         Test restoration to a temporary database first (default: true)"
      echo "  -f, --force        Force restoration without confirmation (default: false)"
      echo "  -v, --verify       Verify database integrity after restoration (default: true)"
      echo "  -m, --monitor      Send monitoring notifications (default: false)"
      echo "  -h, --help         Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      exit 1
      ;;
  esac
done

# Validate required parameters
if [ -z "$BACKUP" ]; then
  echo "Error: Backup file path is required"
  echo "Use --help for usage information"
  exit 1
fi

if [ -z "$DATABASE" ]; then
  echo "Error: Target database path is required"
  echo "Use --help for usage information"
  exit 1
fi

if [ ! -f "$BACKUP" ]; then
  echo "Error: Backup file does not exist: $BACKUP"
  exit 1
fi

# Check if the backup is compressed
IS_COMPRESSED=false
if [[ "$BACKUP" == *.gz ]]; then
  IS_COMPRESSED=true
  echo "Backup file is compressed"
else
  echo "Backup file is not compressed"
fi

# Log start of restoration
echo "Starting restoration from $BACKUP to $DATABASE"

# Create a temporary directory for restoration
TEMP_DIR=$(mktemp -d)
chmod 700 "$TEMP_DIR"
echo "Created temporary directory: $TEMP_DIR"

# Function to clean up temporary files
cleanup() {
  echo "Cleaning up temporary files"
  rm -rf "$TEMP_DIR"
}

# Set up trap to clean up on exit
trap cleanup EXIT

# Extract backup if compressed
if [ "$IS_COMPRESSED" = true ]; then
  EXTRACTED_BACKUP="$TEMP_DIR/backup.sql"
  echo "Extracting compressed backup to $EXTRACTED_BACKUP"
  gunzip -c "$BACKUP" > "$EXTRACTED_BACKUP"
  BACKUP_TO_RESTORE="$EXTRACTED_BACKUP"
else
  BACKUP_TO_RESTORE="$BACKUP"
fi

# Test restoration if requested
if [ "$TEST" = true ]; then
  echo "Testing restoration to a temporary database"
  TEST_DB="$TEMP_DIR/test.db"
  
  # Create empty test database
  touch "$TEST_DB"
  
  # Restore to test database
  if sqlite3 "$TEST_DB" < "$BACKUP_TO_RESTORE"; then
    echo "Test restoration successful"
    
    # Verify test database integrity
    if [ "$VERIFY" = true ]; then
      echo "Verifying test database integrity"
      if sqlite3 "$TEST_DB" "PRAGMA integrity_check;" | grep -q "ok"; then
        echo "Test database integrity check passed"
      else
        echo "Error: Test database integrity check failed"
        if [ "$MONITOR" = true ]; then
          if command -v curl &> /dev/null; then
            curl -s -X POST -H "Content-Type: application/json" \
              -d "{\"status\":\"error\",\"message\":\"Database restoration failed: Test database integrity check failed\",\"backup\":\"$BACKUP\",\"database\":\"$DATABASE\"}" \
              "${MONITOR_WEBHOOK_URL:-http://localhost:8080/api/restore-status}" || true
          fi
        fi
        exit 1
      fi
    fi
  else
    echo "Error: Test restoration failed"
    if [ "$MONITOR" = true ]; then
      if command -v curl &> /dev/null; then
        curl -s -X POST -H "Content-Type: application/json" \
          -d "{\"status\":\"error\",\"message\":\"Database restoration failed: Test restoration failed\",\"backup\":\"$BACKUP\",\"database\":\"$DATABASE\"}" \
          "${MONITOR_WEBHOOK_URL:-http://localhost:8080/api/restore-status}" || true
      fi
    fi
    exit 1
  fi
fi

# Check if target database exists and ask for confirmation
if [ -f "$DATABASE" ] && [ "$FORCE" != true ]; then
  echo "Warning: Target database already exists: $DATABASE"
  read -p "Do you want to overwrite it? (y/N) " -n 1 -r
  echo
  if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Restoration cancelled"
    exit 0
  fi
fi

# Create a backup of the existing database if it exists
if [ -f "$DATABASE" ]; then
  TIMESTAMP=$(date +"%Y%m%d-%H%M%S")
  DB_BACKUP="${DATABASE}.${TIMESTAMP}.bak"
  echo "Creating backup of existing database: $DB_BACKUP"
  cp "$DATABASE" "$DB_BACKUP"
  chmod 600 "$DB_BACKUP"
fi

# Restore the database
echo "Restoring database from $BACKUP_TO_RESTORE to $DATABASE"

# Ensure the target directory exists
mkdir -p "$(dirname "$DATABASE")"

# Create empty database file with secure permissions
touch "$DATABASE"
chmod 600 "$DATABASE"

# Perform the restoration
if sqlite3 "$DATABASE" < "$BACKUP_TO_RESTORE"; then
  echo "Database restoration successful"
  
  # Verify database integrity
  if [ "$VERIFY" = true ]; then
    echo "Verifying database integrity"
    if sqlite3 "$DATABASE" "PRAGMA integrity_check;" | grep -q "ok"; then
      echo "Database integrity check passed"
    else
      echo "Error: Database integrity check failed"
      if [ -f "$DB_BACKUP" ]; then
        echo "Restoring from backup: $DB_BACKUP"
        cp "$DB_BACKUP" "$DATABASE"
        echo "Original database restored from backup"
      fi
      if [ "$MONITOR" = true ]; then
        if command -v curl &> /dev/null; then
          curl -s -X POST -H "Content-Type: application/json" \
            -d "{\"status\":\"error\",\"message\":\"Database restoration failed: Integrity check failed\",\"backup\":\"$BACKUP\",\"database\":\"$DATABASE\"}" \
            "${MONITOR_WEBHOOK_URL:-http://localhost:8080/api/restore-status}" || true
        fi
      fi
      exit 1
    fi
  fi
  
  # Send monitoring notification if enabled
  if [ "$MONITOR" = true ]; then
    if command -v curl &> /dev/null; then
      curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"status\":\"success\",\"message\":\"Database restoration completed successfully\",\"backup\":\"$BACKUP\",\"database\":\"$DATABASE\"}" \
        "${MONITOR_WEBHOOK_URL:-http://localhost:8080/api/restore-status}" || true
    fi
  fi
else
  echo "Error: Database restoration failed"
  if [ -f "$DB_BACKUP" ]; then
    echo "Restoring from backup: $DB_BACKUP"
    cp "$DB_BACKUP" "$DATABASE"
    echo "Original database restored from backup"
  fi
  if [ "$MONITOR" = true ]; then
    if command -v curl &> /dev/null; then
      curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"status\":\"error\",\"message\":\"Database restoration failed\",\"backup\":\"$BACKUP\",\"database\":\"$DATABASE\"}" \
        "${MONITOR_WEBHOOK_URL:-http://localhost:8080/api/restore-status}" || true
    fi
  fi
  exit 1
fi

# Log completion
echo "Restoration process completed successfully"
echo "Restored database: $DATABASE"