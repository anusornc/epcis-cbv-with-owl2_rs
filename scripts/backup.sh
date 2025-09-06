#!/bin/bash

# Backup Script for EPCIS Knowledge Graph

set -e

# Configuration
APP_NAME="epcis-knowledge-graph"
BACKUP_DIR="/var/backups/epcis-kg"
DATA_DIR="/var/lib/epcis-kg/data"
CONFIG_DIR="/etc/epcis-kg"
RETENTION_DAYS=7
S3_BUCKET=""
ENCRYPT=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --s3-bucket)
            S3_BUCKET="$2"
            shift 2
            ;;
        --encrypt)
            ENCRYPT=true
            shift
            ;;
        --retention-days)
            RETENTION_DAYS="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --s3-bucket BUCKET   Upload backup to S3 bucket"
            echo "  --encrypt            Encrypt backup files"
            echo "  --retention-days N   Keep backups for N days (default: 7)"
            echo "  --help               Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting EPCIS Knowledge Graph backup...${NC}"

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This script must be run as root${NC}"
   exit 1
fi

# Create backup directory
BACKUP_DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_PATH="$BACKUP_DIR/backup_$BACKUP_DATE"
mkdir -p "$BACKUP_PATH"

# Stop service for consistent backup
echo -e "${YELLOW}Stopping service...${NC}"
systemctl stop $APP_NAME

# Create backup
echo -e "${YELLOW}Creating backup...${NC}"

# Backup data directory
if [[ -d "$DATA_DIR" ]]; then
    echo "Backing up data directory..."
    cp -r "$DATA_DIR" "$BACKUP_PATH/data"
fi

# Backup configuration
if [[ -d "$CONFIG_DIR" ]]; then
    echo "Backing up configuration..."
    cp -r "$CONFIG_DIR" "$BACKUP_PATH/config"
fi

# Backup service logs
echo "Backing up service logs..."
journalctl -u $APP_NAME > "$BACKUP_PATH/service.log"

# Create backup manifest
cat > "$BACKUP_PATH/manifest.json" << EOF
{
    "backup_date": "$BACKUP_DATE",
    "app_name": "$APP_NAME",
    "backup_version": "1.0",
    "includes": ["data", "config", "logs"],
    "encrypted": $ENCRYPT
}
EOF

# Compress backup
echo -e "${YELLOW}Compressing backup...${NC}"
tar -czf "$BACKUP_PATH.tar.gz" -C "$BACKUP_DIR" "backup_$BACKUP_DATE"
rm -rf "$BACKUP_PATH"

# Encrypt backup if requested
if [[ "$ENCRYPT" == "true" ]]; then
    echo -e "${YELLOW}Encrypting backup...${NC}"
    if command -v gpg &> /dev/null; then
        gpg --symmetric --cipher-algo AES256 --output "$BACKUP_PATH.tar.gz.gpg" "$BACKUP_PATH.tar.gz"
        rm "$BACKUP_PATH.tar.gz"
        BACKUP_FILE="$BACKUP_PATH.tar.gz.gpg"
    else
        echo -e "${RED}GPG not found, skipping encryption${NC}"
        BACKUP_FILE="$BACKUP_PATH.tar.gz"
    fi
else
    BACKUP_FILE="$BACKUP_PATH.tar.gz"
fi

# Upload to S3 if requested
if [[ -n "$S3_BUCKET" ]]; then
    echo -e "${YELLOW}Uploading to S3...${NC}"
    if command -v aws &> /dev/null; then
        aws s3 cp "$BACKUP_FILE" "s3://$S3_BUCKET/backup_$BACKUP_DATE.tar.gz${BACKUP_FILE##*.tar.gz}"
        echo -e "${GREEN}✓ Backup uploaded to S3${NC}"
    else
        echo -e "${RED}AWS CLI not found, skipping S3 upload${NC}"
    fi
fi

# Start service
echo -e "${YELLOW}Starting service...${NC}"
systemctl start $APP_NAME

# Verify service started
sleep 5
if systemctl is-active --quiet $APP_NAME; then
    echo -e "${GREEN}✓ Service started successfully${NC}"
else
    echo -e "${RED}✗ Service failed to start${NC}"
    exit 1
fi

# Clean old backups
echo -e "${YELLOW}Cleaning old backups...${NC}"
find "$BACKUP_DIR" -name "backup_*.tar.gz*" -mtime +$RETENTION_DAYS -delete

# Calculate backup size
BACKUP_SIZE=$(du -h "$BACKUP_FILE" | cut -f1)

echo ""
echo -e "${GREEN}✓ Backup completed successfully${NC}"
echo "Backup file: $BACKUP_FILE"
echo "Backup size: $BACKUP_SIZE"
echo "Retention period: $RETENTION_DAYS days"

if [[ -n "$S3_BUCKET" ]]; then
    echo "S3 bucket: $S3_BUCKET"
fi

if [[ "$ENCRYPT" == "true" ]]; then
    echo "Encryption: Enabled"
fi