#!/bin/bash

# Production Deployment Script for EPCIS Knowledge Graph

set -e

# Configuration
APP_NAME="epcis-knowledge-graph"
APP_USER="epcis"
APP_DIR="/opt/epcis-kg"
SERVICE_FILE="/etc/systemd/system/epcis-kg.service"
CONFIG_FILE="/etc/epcis-kg/production.toml"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}Starting EPCIS Knowledge Graph deployment...${NC}"

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo -e "${RED}This script must be run as root${NC}"
   exit 1
fi

# Create user and directories
echo -e "${YELLOW}Creating user and directories...${NC}"
useradd -r -s /bin/false $APP_USER || true
mkdir -p $APP_DIR /var/lib/epcis-kg/data /var/log/epcis-kg /var/backups/epcis-kg /etc/epcis-kg
chown -R $APP_USER:$APP_USER $APP_DIR /var/lib/epcis-kg /var/log/epcis-kg /var/backups/epcis-kg /etc/epcis-kg

# Copy application files
echo -e "${YELLOW}Copying application files...${NC}"
cp target/release/$APP_NAME $APP_DIR/
chmod +x $APP_DIR/$APP_NAME
chown $APP_USER:$APP_USER $APP_DIR/$APP_NAME

# Copy configuration
echo -e "${YELLOW}Installing configuration...${NC}"
cp config/production.toml $CONFIG_FILE
chown $APP_USER:$APP_USER $CONFIG_FILE

# Copy ontologies
echo -e "${YELLOW}Installing ontologies...${NC}"
cp -r ontologies $APP_DIR/
chown -R $APP_USER:$APP_USER $APP_DIR/ontologies

# Create systemd service
echo -e "${YELLOW}Creating systemd service...${NC}"
cat > $SERVICE_FILE << EOF
[Unit]
Description=EPCIS Knowledge Graph Service
After=network.target

[Service]
Type=simple
User=$APP_USER
WorkingDirectory=$APP_DIR
ExecStart=$APP_DIR/$APP_NAME serve --config $CONFIG_FILE
Restart=always
RestartSec=10
Environment=RUST_LOG=info
Environment=RUST_BACKTRACE=1

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/epcis-kg/data
ReadWritePaths=/var/log/epcis-kg
ReadWritePaths=/var/backups/epcis-kg

# Resource limits
LimitNOFILE=65536
MemoryMax=8G

[Install]
WantedBy=multi-user.target
EOF

# Reload systemd
systemctl daemon-reload

# Enable and start service
echo -e "${YELLOW}Enabling and starting service...${NC}"
systemctl enable epcis-kg
systemctl start epcis-kg

# Wait for service to start
echo -e "${YELLOW}Waiting for service to start...${NC}"
sleep 10

# Check service status
if systemctl is-active --quiet epcis-kg; then
    echo -e "${GREEN}✓ Service started successfully${NC}"
    echo -e "${GREEN}✓ Deployment completed successfully${NC}"
    echo ""
    echo "Service status:"
    systemctl status epcis-kg --no-pager -l
    echo ""
    echo "To view logs: journalctl -u epcis-kg -f"
    echo "To stop service: systemctl stop epcis-kg"
    echo "To restart service: systemctl restart epcis-kg"
else
    echo -e "${RED}✗ Service failed to start${NC}"
    echo "Check logs: journalctl -u epcis-kg"
    exit 1
fi