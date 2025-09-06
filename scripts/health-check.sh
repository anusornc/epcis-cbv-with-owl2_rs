#!/bin/bash

# Health Check Script for EPCIS Knowledge Graph

set -e

# Configuration
SERVICE_NAME="epcis-kg"
HEALTH_URL="http://localhost:8080/health"
METRICS_URL="http://localhost:8080/api/v1/monitoring/health"
MAX_RETRIES=3
RETRY_DELAY=5

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}EPCIS Knowledge Graph Health Check${NC}"
echo "=================================="

# Check if service is running
if systemctl is-active --quiet $SERVICE_NAME; then
    echo -e "${GREEN}✓ Service is running${NC}"
else
    echo -e "${RED}✗ Service is not running${NC}"
    exit 1
fi

# Check basic health endpoint
echo -e "${YELLOW}Checking basic health endpoint...${NC}"
for i in $(seq 1 $MAX_RETRIES); do
    if curl -f -s $HEALTH_URL > /dev/null; then
        echo -e "${GREEN}✓ Basic health check passed${NC}"
        break
    else
        if [[ $i -eq $MAX_RETRIES ]]; then
            echo -e "${RED}✗ Basic health check failed after $MAX_RETRIES attempts${NC}"
            exit 1
        else
            echo -e "${YELLOW}Attempt $i failed, retrying in $RETRY_DELAY seconds...${NC}"
            sleep $RETRY_DELAY
        fi
    fi
done

# Check detailed health metrics
echo -e "${YELLOW}Checking detailed health metrics...${NC}"
METRICS_RESPONSE=$(curl -s $METRICS_URL 2>/dev/null || echo "")
if [[ -n "$METRICS_RESPONSE" ]]; then
    echo -e "${GREEN}✓ Detailed health metrics available${NC}"
    
    # Extract key metrics
    STATUS=$(echo "$METRICS_RESPONSE" | jq -r '.status' 2>/dev/null || echo "unknown")
    UPTIME=$(echo "$METRICS_RESPONSE" | jq -r '.uptime_seconds' 2>/dev/null || echo "unknown")
    MEMORY=$(echo "$METRICS_RESPONSE" | jq -r '.memory_usage_mb' 2>/dev/null || echo "unknown")
    CPU=$(echo "$METRICS_RESPONSE" | jq -r '.cpu_usage_percent' 2>/dev/null || echo "unknown")
    
    echo "Status: $STATUS"
    echo "Uptime: $UPTIME seconds"
    echo "Memory Usage: $MEMORY MB"
    echo "CPU Usage: $CPU%"
    
    # Check for alerts
    ALERTS_COUNT=$(echo "$METRICS_RESPONSE" | jq -r '.active_alerts_count' 2>/dev/null || echo "unknown")
    if [[ "$ALERTS_COUNT" -gt 0 ]]; then
        echo -e "${YELLOW}⚠️  $ALERTS_COUNT active alerts detected${NC}"
    else
        echo -e "${GREEN}✓ No active alerts${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  Could not fetch detailed health metrics${NC}"
fi

# Check resource usage
echo -e "${YELLOW}Checking resource usage...${NC}"
CPU_USAGE=$(top -bn1 | grep "Cpu(s)" | sed "s/.*, *\([0-9.]*\)%* id.*/\1/" | awk '{print 100 - $1}')
MEMORY_USAGE=$(free -m | awk '/Mem:/ {print ($3/$2) * 100.0}')
DISK_USAGE=$(df -h / | awk 'NR==2 {print $5}' | sed 's/%//')

echo "CPU Usage: ${CPU_USAGE}%"
echo "Memory Usage: ${MEMORY_USAGE}%"
echo "Disk Usage: ${DISK_USAGE}%"

# Check thresholds
if (( $(echo "$CPU_USAGE > 80" | bc -l) )); then
    echo -e "${YELLOW}⚠️  High CPU usage: ${CPU_USAGE}%${NC}"
fi

if (( $(echo "$MEMORY_USAGE > 80" | bc -l) )); then
    echo -e "${YELLOW}⚠️  High memory usage: ${MEMORY_USAGE}%${NC}"
fi

if [[ $DISK_USAGE -gt 80 ]]; then
    echo -e "${YELLOW}⚠️  High disk usage: ${DISK_USAGE}%${NC}"
fi

# Check logs for recent errors
echo -e "${YELLOW}Checking for recent errors...${NC}"
RECENT_ERRORS=$(journalctl -u $SERVICE_NAME --since "1 hour ago" | grep -i "error\|warn\|failed" | wc -l)
if [[ $RECENT_ERRORS -gt 0 ]]; then
    echo -e "${YELLOW}⚠️  $RECENT_ERRORS errors/warnings in the last hour${NC}"
else
    echo -e "${GREEN}✓ No recent errors found${NC}"
fi

echo ""
echo -e "${GREEN}✓ Health check completed successfully${NC}"