# Database Backup and Restore Guide

This document provides comprehensive documentation for backing up and restoring the SQLite database used in the CV Blog application.

## Table of Contents

1. [Overview](#overview)
2. [Manual Backup](#manual-backup)
3. [Automated Backups](#automated-backups)
4. [Restoring from Backup](#restoring-from-backup)
5. [Docker Integration](#docker-integration)
6. [Best Practices](#best-practices)
7. [Troubleshooting](#troubleshooting)

## Overview

The CV Blog application uses SQLite for data storage. Regular backups are essential to prevent data loss. This guide explains how to use the provided scripts for database backup and restoration.

## Manual Backup

The application includes a comprehensive backup script that can create backups with configurable retention policies.

### Usage

```bash
./scripts/backup_database.sh [options]
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `-d, --database` | Path to the SQLite database file (required) | |
| `-b, --backup-dir` | Directory to store backups | ./backups |
| `-p, --prefix` | Prefix for backup files | backup |
| `-r, --retention` | Retention period in days | 30 |
| `-c, --compress` | Compress backups with gzip | true |
| `-t, --type` | Backup type: daily, weekly, monthly | daily |
| `-m, --monitor` | Send monitoring notifications | false |
| `-h, --help` | Show help message | |

### Examples

```bash
# Basic backup
./scripts/backup_database.sh --database /app/data/blog.db

# Specify backup directory and retention
./scripts/backup_database.sh --database /app/data/blog.db --backup-dir /mnt/backups --retention 60

# Create a weekly backup
./scripts/backup_database.sh --database /app/data/blog.db --type weekly --prefix blog
```

## Automated Backups

For production environments, it's recommended to set up automated backups using the provided script.

### Setting Up Cron Jobs

The `setup_backup_cron.sh` script configures cron jobs for daily, weekly, and monthly backups with different retention periods.

```bash
./scripts/setup_backup_cron.sh [options]
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `-d, --database` | Path to the SQLite database file (required) | |
| `-b, --backup-dir` | Directory to store backups | /var/backups/blog-db |
| `-p, --prefix` | Prefix for backup files | blog |
| `-u, --user` | User to run the cron jobs as | current user |
| `-m, --monitor` | Enable monitoring notifications | false |
| `-w, --webhook` | Monitoring webhook URL | |
| `-h, --help` | Show help message | |

### Example

```bash
./scripts/setup_backup_cron.sh --database /app/data/blog.db --backup-dir /mnt/backups --user www-data
```

This will set up:
- Daily backups at 2:00 AM (retention: 30 days)
- Weekly backups on Sundays at 3:00 AM (retention: 90 days)
- Monthly backups on the 1st of each month at 4:00 AM (retention: 365 days)

## Restoring from Backup

If you need to restore the database from a backup, use the `restore_database.sh` script.

### Usage

```bash
./scripts/restore_database.sh [options]
```

### Options

| Option | Description | Default |
|--------|-------------|---------|
| `-b, --backup` | Path to the backup file (required) | |
| `-d, --database` | Path to the target SQLite database file (required) | |
| `-t, --test` | Test restoration to a temporary database first | true |
| `-f, --force` | Force restoration without confirmation | false |
| `-v, --verify` | Verify database integrity after restoration | true |
| `-m, --monitor` | Send monitoring notifications | false |
| `-h, --help` | Show help message | |

### Examples

```bash
# Restore from a backup
./scripts/restore_database.sh --backup /backups/backup-daily-20250723-120000.sql.gz --database /app/data/blog.db

# Skip test restoration
./scripts/restore_database.sh --backup /backups/backup-daily-20250723-120000.sql --database /app/data/blog.db --test false

# Force restoration without confirmation
./scripts/restore_database.sh --backup /backups/backup-daily-20250723-120000.sql.gz --database /app/data/blog.db --force true
```

## Docker Integration

For Docker environments, a Docker-compatible backup script is automatically created when you run `setup_backup_cron.sh`.

### Using the Docker Backup Script

```bash
docker run --rm \
  -v ./data:/app/data \
  -v ./backups:/backups \
  -e DATABASE_PATH=/app/data/blog.db \
  -e BACKUP_DIR=/backups \
  -e BACKUP_TYPE=daily \
  -e BACKUP_RETENTION=30 \
  your-image /app/scripts/docker_backup_database.sh
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_PATH` | Path to the SQLite database file | /app/data/blog.db |
| `BACKUP_DIR` | Directory to store backups | /backups |
| `BACKUP_PREFIX` | Prefix for backup files | blog |
| `BACKUP_TYPE` | Backup type (daily, weekly, monthly) | daily |
| `BACKUP_RETENTION` | Retention period in days | 30 |
| `ENABLE_MONITORING` | Enable monitoring notifications | false |
| `MONITOR_WEBHOOK_URL` | Monitoring webhook URL | |

### Docker Compose Example

```yaml
backup:
  image: your-image
  volumes:
    - ./data:/app/data
    - ./backups:/backups
  environment:
    - DATABASE_PATH=/app/data/blog.db
    - BACKUP_DIR=/backups
    - BACKUP_TYPE=daily
    - BACKUP_RETENTION=30
  command: /app/scripts/docker_backup_database.sh
```

## Best Practices

1. **Regular Backups**: Set up automated backups using the `setup_backup_cron.sh` script
2. **Multiple Backup Types**: Use daily, weekly, and monthly backups with different retention periods
3. **Secure Storage**: Store backups in a secure location with appropriate permissions
4. **Off-site Backups**: Copy backups to an off-site location for disaster recovery
5. **Test Restoration**: Regularly test the restoration process to ensure backups are valid
6. **Monitoring**: Enable monitoring notifications to be alerted of backup failures

## Troubleshooting

### Backup Fails with "Database is locked"

**Problem**: The backup script fails with a "Database is locked" error.

**Solution**: Ensure no other processes are accessing the database during backup. You may need to stop the application temporarily or use a read-only copy of the database.

### Restoration Fails with "Integrity check failed"

**Problem**: The restoration process fails with an "Integrity check failed" error.

**Solution**: The backup file may be corrupted. Try using an earlier backup or run the restoration with the `--verify false` option (not recommended for production).

### Cron Jobs Not Running

**Problem**: The automated backups are not running as scheduled.

**Solution**: 
1. Check if the cron service is running: `systemctl status cron`
2. Verify the cron jobs are installed: `crontab -l`
3. Check the system logs for cron-related errors: `grep CRON /var/log/syslog`