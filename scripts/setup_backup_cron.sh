#!/bin/bash
#
# Setup Backup Cron Jobs
#
# This script sets up cron jobs for automated database backups.
# It creates daily, weekly, and monthly backup jobs with different retention periods.
# 
# Usage: ./setup_backup_cron.sh [options]
#
# Options:
#   -d, --database     Path to the SQLite database file (required)
#   -b, --backup-dir   Directory to store backups (default: /var/backups/blog-db)
#   -p, --prefix       Prefix for backup files (default: blog)
#   -u, --user         User to run the cron jobs as (default: current user)
#   -m, --monitor      Enable monitoring notifications (default: false)
#   -w, --webhook      Monitoring webhook URL (optional)
#   -h, --help         Show this help message
#
# Examples:
#   ./setup_backup_cron.sh --database /app/data/blog.db
#   ./setup_backup_cron.sh --database /app/data/blog.db --backup-dir /mnt/backups --user www-data
#   ./setup_backup_cron.sh --database /app/data/blog.db --monitor true --webhook https://hooks.slack.com/services/XXX/YYY/ZZZ

set -e

# Default values
DATABASE=""
BACKUP_DIR="/var/backups/blog-db"
PREFIX="blog"
USER=$(whoami)
MONITOR="false"
WEBHOOK=""

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
    -u|--user)
      USER="$2"
      shift 2
      ;;
    -m|--monitor)
      MONITOR="$2"
      shift 2
      ;;
    -w|--webhook)
      WEBHOOK="$2"
      shift 2
      ;;
    -h|--help)
      echo "Usage: $0 [options]"
      echo ""
      echo "Options:"
      echo "  -d, --database     Path to the SQLite database file (required)"
      echo "  -b, --backup-dir   Directory to store backups (default: /var/backups/blog-db)"
      echo "  -p, --prefix       Prefix for backup files (default: blog)"
      echo "  -u, --user         User to run the cron jobs as (default: current user)"
      echo "  -m, --monitor      Enable monitoring notifications (default: false)"
      echo "  -w, --webhook      Monitoring webhook URL (optional)"
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

# Get the absolute path to the backup script
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKUP_SCRIPT="${SCRIPT_DIR}/backup_database.sh"

if [ ! -f "$BACKUP_SCRIPT" ]; then
  echo "Error: Backup script not found: $BACKUP_SCRIPT"
  exit 1
fi

# Make sure the backup script is executable
chmod +x "$BACKUP_SCRIPT"

# Create the backup directory if it doesn't exist
if [ ! -d "$BACKUP_DIR" ]; then
  echo "Creating backup directory: $BACKUP_DIR"
  mkdir -p "$BACKUP_DIR"
  
  # Set secure permissions on backup directory
  chmod 700 "$BACKUP_DIR"
  
  # Change ownership if running as root and user is different
  if [ "$(id -u)" -eq 0 ] && [ "$USER" != "root" ]; then
    chown "$USER" "$BACKUP_DIR"
  fi
fi

# Create the cron job file
CRON_FILE="/tmp/blog_backup_cron"

# Add environment variable for webhook if provided
if [ -n "$WEBHOOK" ]; then
  echo "MONITOR_WEBHOOK_URL=$WEBHOOK" > "$CRON_FILE"
  echo "" >> "$CRON_FILE"
fi

# Add the cron jobs
cat << EOF >> "$CRON_FILE"
# Blog database backup cron jobs
# Daily backup at 2:00 AM
0 2 * * * $BACKUP_SCRIPT --database $DATABASE --backup-dir $BACKUP_DIR --prefix $PREFIX --type daily --retention 30 --monitor $MONITOR

# Weekly backup on Sundays at 3:00 AM
0 3 * * 0 $BACKUP_SCRIPT --database $DATABASE --backup-dir $BACKUP_DIR --prefix $PREFIX --type weekly --retention 90 --monitor $MONITOR

# Monthly backup on the 1st of each month at 4:00 AM
0 4 1 * * $BACKUP_SCRIPT --database $DATABASE --backup-dir $BACKUP_DIR --prefix $PREFIX --type monthly --retention 365 --monitor $MONITOR
EOF

# Install the cron jobs for the specified user
echo "Installing cron jobs for user: $USER"
if [ "$(id -u)" -eq 0 ] && [ "$USER" != "root" ]; then
  # If running as root and user is different, use crontab as that user
  crontab -u "$USER" "$CRON_FILE"
else
  # Otherwise, use crontab for the current user
  crontab "$CRON_FILE"
fi

# Clean up the temporary file
rm "$CRON_FILE"

echo "Cron jobs installed successfully"
echo "Daily backups will run at 2:00 AM (retention: 30 days)"
echo "Weekly backups will run on Sundays at 3:00 AM (retention: 90 days)"
echo "Monthly backups will run on the 1st of each month at 4:00 AM (retention: 365 days)"
echo "Backups will be stored in: $BACKUP_DIR"

# Create a Docker-compatible backup script for containerized environments
DOCKER_BACKUP_SCRIPT="${SCRIPT_DIR}/docker_backup_database.sh"

cat << 'EOF' > "$DOCKER_BACKUP_SCRIPT"
#!/bin/bash
#
# Docker-compatible Database Backup Script
#
# This script is designed to be run from a Docker container or as a Docker service.
# It creates backups of the SQLite database with configurable retention policies.

# Default values from environment variables with fallbacks
DATABASE=${DATABASE_PATH:-/app/data/blog.db}
BACKUP_DIR=${BACKUP_DIR:-/backups}
PREFIX=${BACKUP_PREFIX:-blog}
TYPE=${BACKUP_TYPE:-daily}
RETENTION=${BACKUP_RETENTION:-30}
MONITOR=${ENABLE_MONITORING:-false}
WEBHOOK_URL=${MONITOR_WEBHOOK_URL:-""}

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
  if [ "$MONITOR" = "true" ] && [ -n "$WEBHOOK_URL" ]; then
    # Send monitoring notification
    if command -v curl &> /dev/null; then
      curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"status\":\"error\",\"message\":\"Database backup failed: Database is locked or inaccessible\",\"database\":\"$DATABASE\"}" \
        "$WEBHOOK_URL" || true
    fi
  fi
  exit 1
fi

# Create backup
if sqlite3 "$DATABASE" ".backup '$BACKUP_FILE'"; then
  echo "Backup created successfully: $BACKUP_FILE"
  
  # Compress backup
  gzip -f "$BACKUP_FILE"
  BACKUP_FILE="${BACKUP_FILE}.gz"
  echo "Backup compressed: $BACKUP_FILE"
  
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
  if [ "$MONITOR" = "true" ] && [ -n "$WEBHOOK_URL" ]; then
    if command -v curl &> /dev/null; then
      curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"status\":\"success\",\"message\":\"Database backup completed successfully\",\"database\":\"$DATABASE\",\"backup_file\":\"$BACKUP_FILE\",\"backup_size\":\"$BACKUP_SIZE\",\"backup_type\":\"$TYPE\",\"retention_days\":$RETENTION,\"backup_count\":$BACKUP_COUNT}" \
        "$WEBHOOK_URL" || true
    fi
  fi
else
  echo "Error: Backup failed"
  if [ "$MONITOR" = "true" ] && [ -n "$WEBHOOK_URL" ]; then
    # Send monitoring notification
    if command -v curl &> /dev/null; then
      curl -s -X POST -H "Content-Type: application/json" \
        -d "{\"status\":\"error\",\"message\":\"Database backup failed\",\"database\":\"$DATABASE\"}" \
        "$WEBHOOK_URL" || true
    fi
  fi
  exit 1
fi
EOF

# Make the Docker backup script executable
chmod +x "$DOCKER_BACKUP_SCRIPT"

echo ""
echo "Docker-compatible backup script created: $DOCKER_BACKUP_SCRIPT"
echo "You can use this script in a Docker container with environment variables:"
echo "  DATABASE_PATH: Path to the SQLite database file"
echo "  BACKUP_DIR: Directory to store backups"
echo "  BACKUP_PREFIX: Prefix for backup files"
echo "  BACKUP_TYPE: Backup type (daily, weekly, monthly)"
echo "  BACKUP_RETENTION: Retention period in days"
echo "  ENABLE_MONITORING: Enable monitoring notifications (true/false)"
echo "  MONITOR_WEBHOOK_URL: Monitoring webhook URL"
echo ""
echo "Example Docker Compose service:"
echo "  backup:"
echo "    image: your-image"
echo "    volumes:"
echo "      - ./data:/app/data"
echo "      - ./backups:/backups"
echo "    environment:"
echo "      - DATABASE_PATH=/app/data/blog.db"
echo "      - BACKUP_DIR=/backups"
echo "      - BACKUP_TYPE=daily"
echo "      - BACKUP_RETENTION=30"
echo "    command: /app/scripts/docker_backup_database.sh"