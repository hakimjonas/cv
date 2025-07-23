#!/bin/bash
#
# Automated SQLite Database Backup Script
#
# This script creates backups of SQLite databases with configurable retention policies.
# It supports daily, weekly, and monthly backups with different retention periods.
# 
# Usage: ./backup_database.sh [options]
#
# Options:
#   -d, --database     Path to the SQLite database file (required)
#   -b, --backup-dir   Directory to store backups (default: ./backups)
#   -p, --prefix       Prefix for backup files (default: backup)
#   -r, --retention    Retention period in days (default: 30)
#   -c, --compress     Compress backups with gzip (default: true)
#   -t, --type         Backup type: daily, weekly, monthly (default: daily)
#   -m, --monitor      Send monitoring notifications (default: false)
#   -h, --help         Show this help message
#
# Examples:
#   ./backup_database.sh --database /app/data/blog.db
#   ./backup_database.sh --database /app/data/blog.db --backup-dir /mnt/backups --retention 60
#   ./backup_database.sh --database /app/data/blog.db --type weekly --prefix blog
#
# Cron setup examples:
#   # Daily backup at 2:00 AM
#   0 2 * * * /path/to/backup_database.sh --database /app/data/blog.db --type daily
#
#   # Weekly backup on Sundays at 3:00 AM
#   0 3 * * 0 /path/to/backup_database.sh --database /app/data/blog.db --type weekly
#
#   # Monthly backup on the 1st of each month at 4:00 AM
#   0 4 1 * * /path/to/backup_database.sh --database /app/data/blog.db --type monthly

set -e

# Default values
DATABASE=""
BACKUP_DIR="./backups"
PREFIX="backup"
RETENTION=30
COMPRESS=true
TYPE="daily"
MONITOR=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    -d|--database)
      DATABASE="$2"
      shift 2
      ;;
    -b|--backup-dir)
      BACKUP_DIR="$2"
      shift 2
      ;;
    -p|--prefix)
      PREFIX="$2"
      shift 2
      ;;
    -r|--retention)
      RETENTION="$2"
      shift 2
      ;;
    -c|--compress)
      COMPRESS="$2"
      shift 2
      ;;
    -t|--type)
      TYPE="$2"
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
      echo "  -d, --database     Path to the SQLite database file (required)"
      echo "  -b, --backup-dir   Directory to store backups (default: ./backups)"
      echo "  -p, --prefix       Prefix for backup files (default: backup)"
      echo "  -r, --retention    Retention period in days (default: 30)"
      echo "  -c, --compress     Compress backups with gzip (default: true)"
      echo "  -t, --type         Backup type: daily, weekly, monthly (default: daily)"
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
if [ -z "$DATABASE" ]; then
  echo "Error: Database path is required"
  echo "Use --help for usage information"
  exit 1
fi

if [ ! -f "$DATABASE" ]; then
  echo "Error: Database file does not exist: $DATABASE"
  exit 1
fi

# Set retention period based on backup type
case $TYPE in
  daily)
    RETENTION=${RETENTION:-30}  # Keep daily backups for 30 days by default
    ;;
  weekly)
    RETENTION=${RETENTION:-90}  # Keep weekly backups for 90 days by default
    ;;
  monthly)
    RETENTION=${RETENTION:-365}  # Keep monthly backups for 365 days by default
    ;;
  *)
    echo "Error: Invalid backup type: $TYPE"
    echo "Valid types are: daily, weekly, monthly"
    exit 1
    ;;
esac

# Create backup directory if it doesn't exist
mkdir -p "$BACKUP_DIR"

# Set secure permissions on backup directory
chmod 700 "$BACKUP_DIR"

# Generate backup filename
TIMESTAMP=$(date +"%Y%m%d-%H%M%S")
BACKUP_FILE="${BACKUP_DIR}/${PREFIX}-${TYPE}-${TIMESTAMP}.sql"

# Log start of backup
echo "Starting backup of $DATABASE to $BACKUP_FILE"
echo "Backup type: $TYPE, Retention: $RETENTION days"

# Check if database is locked
if sqlite3 "$DATABASE" "PRAGMA busy_timeout = 5000; PRAGMA query_only = ON; SELECT count(*) FROM sqlite_master;" > /dev/null 2>&1; then
  echo "Database is accessible, proceeding with backup"
else
  echo "Error: Database is locked or inaccessible"
  if [ "$MONITOR" = "true" ]; then
    # Send monitoring notification
    if command -v curl &> /dev/null; then
      curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"status\":\"error\",\"message\":\"Database backup failed: Database is locked or inaccessible\",\"database\":\"$DATABASE\"}" \
        "${MONITOR_WEBHOOK_URL:-http://localhost:8080/api/backup-status}" || true
    fi
  fi
  exit 1
fi

# Create backup
if sqlite3 "$DATABASE" ".backup '$BACKUP_FILE'"; then
  echo "Backup created successfully: $BACKUP_FILE"
  
  # Compress backup if requested
  if [ "$COMPRESS" = "true" ]; then
    gzip -f "$BACKUP_FILE"
    BACKUP_FILE="${BACKUP_FILE}.gz"
    echo "Backup compressed: $BACKUP_FILE"
  fi
  
  # Set secure permissions on backup file
  chmod 600 "$BACKUP_FILE"
  
  # Calculate backup size
  BACKUP_SIZE=$(du -h "$BACKUP_FILE" | cut -f1)
  
  # Log backup details
  echo "Backup completed successfully"
  echo "Backup file: $BACKUP_FILE"
  echo "Backup size: $BACKUP_SIZE"
  
  # Clean up old backups
  echo "Cleaning up backups older than $RETENTION days"
  find "$BACKUP_DIR" -name "${PREFIX}-${TYPE}-*.sql*" -type f -mtime +$RETENTION -delete
  
  # Count remaining backups
  BACKUP_COUNT=$(find "$BACKUP_DIR" -name "${PREFIX}-${TYPE}-*.sql*" -type f | wc -l)
  echo "Remaining $TYPE backups: $BACKUP_COUNT"
  
  # Send monitoring notification if enabled
  if [ "$MONITOR" = "true" ]; then
    if command -v curl &> /dev/null; then
      curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"status\":\"success\",\"message\":\"Database backup completed successfully\",\"database\":\"$DATABASE\",\"backup_file\":\"$BACKUP_FILE\",\"backup_size\":\"$BACKUP_SIZE\",\"backup_type\":\"$TYPE\",\"retention_days\":$RETENTION,\"backup_count\":$BACKUP_COUNT}" \
        "${MONITOR_WEBHOOK_URL:-http://localhost:8080/api/backup-status}" || true
    fi
  fi
else
  echo "Error: Backup failed"
  if [ "$MONITOR" = "true" ]; then
    # Send monitoring notification
    if command -v curl &> /dev/null; then
      curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"status\":\"error\",\"message\":\"Database backup failed\",\"database\":\"$DATABASE\"}" \
        "${MONITOR_WEBHOOK_URL:-http://localhost:8080/api/backup-status}" || true
    fi
  fi
  exit 1
fi