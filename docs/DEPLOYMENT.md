# EPCIS Knowledge Graph Deployment Guide

## Overview

This guide provides comprehensive instructions for deploying the EPCIS Knowledge Graph in production environments. It covers various deployment strategies, configuration options, and best practices for ensuring reliability and performance.

## Deployment Options

### 1. Docker Deployment (Recommended)

#### Quick Start
```bash
# Build the image
docker build -t epcis-knowledge-graph .

# Run the container
docker run -d \
  --name epcis-kg \
  -p 8080:8080 \
  -v epcis-data:/var/lib/epcis-kg/data \
  -v epcis-logs:/var/log/epcis-kg \
  -v epcis-backups:/var/backups/epcis-kg \
  epcis-knowledge-graph
```

#### Docker Compose Deployment
```bash
# Start all services
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f epcis-knowledge-graph

# Stop services
docker-compose down
```

### 2. Systemd Service Deployment

#### Using Deployment Script
```bash
# Run the deployment script (requires root)
sudo ./scripts/deploy.sh

# Check service status
systemctl status epcis-kg

# View logs
journalctl -u epcis-kg -f

# Restart service
systemctl restart epcis-kg
```

#### Manual Service Installation
```bash
# Create user and directories
sudo useradd -r -s /bin/false epcis
sudo mkdir -p /opt/epcis-kg /var/lib/epcis-kg/data /var/log/epcis-kg
sudo chown -R epcis:epcis /opt/epcis-kg /var/lib/epcis-kg /var/log/epcis-kg

# Copy binary and files
sudo cp target/release/epcis-knowledge-graph /opt/epcis-kg/
sudo cp -r ontologies /opt/epcis-kg/
sudo cp config/production.toml /etc/epcis-kg/config.toml

# Create systemd service
sudo systemctl daemon-reload
sudo systemctl enable epcis-kg
sudo systemctl start epcis-kg
```

### 3. Kubernetes Deployment

#### Create Namespace
```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: epcis-kg
```

#### Deployment Configuration
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: epcis-knowledge-graph
  namespace: epcis-kg
spec:
  replicas: 3
  selector:
    matchLabels:
      app: epcis-knowledge-graph
  template:
    metadata:
      labels:
        app: epcis-knowledge-graph
    spec:
      containers:
      - name: epcis-knowledge-graph
        image: epcis-knowledge-graph:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        - name: EPCIS_KG_CONFIG_PATH
          value: "/etc/epcis-kg/config.toml"
        volumeMounts:
        - name: config
          mountPath: /etc/epcis-kg
        - name: data
          mountPath: /var/lib/epcis-kg/data
        - name: logs
          mountPath: /var/log/epcis-kg
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
      volumes:
      - name: config
        configMap:
          name: epcis-kg-config
      - name: data
        persistentVolumeClaim:
          claimName: epcis-kg-data
      - name: logs
        persistentVolumeClaim:
          claimName: epcis-kg-logs
```

#### Service Configuration
```yaml
apiVersion: v1
kind: Service
metadata:
  name: epcis-knowledge-graph-service
  namespace: epcis-kg
spec:
  selector:
    app: epcis-knowledge-graph
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_LOG` | Logging level | `info` |
| `EPCIS_KG_CONFIG_PATH` | Configuration file path | `/etc/epcis-kg/config.toml` |
| `EPCIS_KG_DATABASE_PATH` | Database directory | `/var/lib/epcis-kg/data` |
| `EPCIS_KG_PORT` | Server port | `8080` |
| `EPCIS_KG_HOST` | Server host | `0.0.0.0` |

### Configuration Files

#### Production Configuration (`config/production.toml`)
```toml
[server]
host = "0.0.0.0"
port = 8080
workers = 4
max_connections = 1000
timeout_seconds = 30

[database]
path = "/var/lib/epcis-kg/data"
max_connections = 100
connection_pool_size = 10
backup_enabled = true
backup_interval_hours = 24
backup_retention_days = 30

[reasoning]
enabled = true
max_depth = 5
cache_size = 10000
parallel_processing = true
batch_size = 1000
timeout_seconds = 300

[monitoring]
enabled = true
metrics_interval_seconds = 30
health_check_interval_seconds = 10
alerting_enabled = true
alert_thresholds = { cpu_usage = 80.0, memory_usage = 85.0, disk_usage = 90.0 }

[logging]
level = "info"
console_output = false
file_output = true
log_directory = "/var/log/epcis-kg"
max_file_size_mb = 100
max_files = 10
include_timestamps = true
format = "json"

[security]
enable_cors = true
allowed_origins = ["http://localhost:3000"]
rate_limit_enabled = true
rate_limit_requests = 100
rate_limit_window_seconds = 60
```

#### Development Configuration (`config/development.toml`)
```toml
[server]
host = "127.0.0.1"
port = 8080
workers = 2
max_connections = 100
timeout_seconds = 60

[database]
path = "./data"
max_connections = 10
backup_enabled = false

[reasoning]
enabled = true
max_depth = 3
cache_size = 1000
parallel_processing = false

[monitoring]
enabled = true
metrics_interval_seconds = 10

[logging]
level = "debug"
console_output = true
file_output = false
format = "text"
```

## Infrastructure Requirements

### Minimum Requirements

- **CPU**: 2 cores
- **Memory**: 4 GB RAM
- **Storage**: 10 GB SSD
- **Network**: 1 Gbps
- **OS**: Linux (Ubuntu 20.04+, CentOS 8+)

### Recommended Requirements

- **CPU**: 4+ cores
- **Memory**: 8+ GB RAM
- **Storage**: 50+ GB SSD
- **Network**: 1+ Gbps
- **OS**: Linux (Ubuntu 22.04+, CentOS 9+)

### Large Scale Deployment

- **CPU**: 8+ cores
- **Memory**: 16+ GB RAM
- **Storage**: 100+ GB SSD
- **Network**: 10+ Gbps
- **Load Balancer**: Yes
- **Database**: External PostgreSQL
- **Cache**: Redis
- **Monitoring**: Prometheus + Grafana

## Security Considerations

### 1. Network Security

#### Firewall Configuration
```bash
# Allow HTTP/HTTPS traffic
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp

# Allow SSH (if needed)
sudo ufw allow 22/tcp

# Enable firewall
sudo ufw enable
```

#### SSL/TLS Configuration
```bash
# Install Nginx as reverse proxy
sudo apt install nginx

# Create SSL certificate (using Let's Encrypt)
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d your-domain.com

# Configure Nginx reverse proxy
sudo tee /etc/nginx/sites-available/epcis-kg << EOF
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://\$server_name\$request_uri;
}

server {
    listen 443 ssl;
    server_name your-domain.com;
    
    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;
    
    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
    }
}
EOF

# Enable site
sudo ln -s /etc/nginx/sites-available/epcis-kg /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

### 2. Application Security

#### User Permissions
```bash
# Create dedicated user
sudo useradd -r -s /bin/false epcis

# Set proper permissions
sudo chown -R epcis:epcis /opt/epcis-kg
sudo chown -R epcis:epcis /var/lib/epcis-kg
sudo chown -R epcis:epcis /var/log/epcis-kg
sudo chmod 750 /opt/epcis-kg
sudo chmod 750 /var/lib/epcis-kg
sudo chmod 750 /var/log/epcis-kg
```

#### Service Hardening
```bash
# Create systemd service with security settings
sudo tee /etc/systemd/system/epcis-kg.service << EOF
[Unit]
Description=EPCIS Knowledge Graph Service
After=network.target

[Service]
Type=simple
User=epcis
WorkingDirectory=/opt/epcis-kg
ExecStart=/opt/epcis-kg/epcis-knowledge-graph serve --config /etc/epcis-kg/config.toml
Restart=always
RestartSec=10
Environment=RUST_LOG=info

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/lib/epcis-kg/data
ReadWritePaths=/var/log/epcis-kg

# Resource limits
LimitNOFILE=65536
MemoryMax=8G

[Install]
WantedBy=multi-user.target
EOF
```

### 3. Data Security

#### Backup Encryption
```bash
# Enable GPG encryption for backups
sudo apt install gnupg

# Generate GPG key (if needed)
gpg --gen-key

# Configure backup with encryption
./scripts/backup.sh --encrypt --s3-bucket your-backup-bucket
```

#### Database Security
```bash
# Set proper file permissions
sudo chmod 600 /var/lib/epcis-kg/data/*
sudo chmod 700 /var/lib/epcis-kg/data

# Enable database encryption (if supported)
# Add to configuration:
# [database]
# encryption_enabled = true
# encryption_key_path = "/etc/epcis-kg/encryption.key"
```

## Monitoring and Alerting

### 1. Health Checks

#### Built-in Health Check
```bash
# Check service health
curl http://localhost:8080/health

# Detailed health check
curl http://localhost:8080/api/v1/monitoring/health

# Use health check script
./scripts/health-check.sh
```

#### Custom Health Checks
```bash
#!/bin/bash
# custom-health-check.sh

# Check if service is running
if ! systemctl is-active --quiet epcis-kg; then
    echo "CRITICAL: Service is not running"
    exit 2
fi

# Check if responding to requests
if ! curl -f -s http://localhost:8080/health > /dev/null; then
    echo "CRITICAL: Service not responding"
    exit 2
fi

# Check memory usage
memory_usage=$(free | awk '/Mem:/ {printf "%.0f", $3/$2 * 100}')
if [ "$memory_usage" -gt 90 ]; then
    echo "WARNING: High memory usage: ${memory_usage}%"
    exit 1
fi

# Check disk usage
disk_usage=$(df /var/lib/epcis-kg | awk 'NR==2 {print $5}' | sed 's/%//')
if [ "$disk_usage" -gt 90 ]; then
    echo "WARNING: High disk usage: ${disk_usage}%"
    exit 1
fi

echo "OK: Service is healthy"
exit 0
```

### 2. Metrics Collection

#### Prometheus Integration
```yaml
# prometheus.yml
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'epcis-kg'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/api/v1/monitoring/metrics'
    scrape_interval: 30s
```

#### Grafana Dashboard

Create a Grafana dashboard with these panels:
- System Metrics (CPU, Memory, Disk)
- Application Metrics (Requests, Response Times)
- Database Metrics (Triples, Query Performance)
- Reasoning Metrics (Inference Time, Cache Hits)

### 3. Alerting

#### Alertmanager Configuration
```yaml
# alertmanager.yml
global:
  smtp_smarthost: 'localhost:587'
  smtp_from: 'alertmanager@your-company.com'

route:
  group_by: ['alertname', 'severity']
  group_wait: 10s
  group_interval: 10s
  repeat_interval: 1h
  receiver: 'web.hook'

receivers:
- name: 'web.hook'
  email_configs:
  - to: 'team@your-company.com'
```

#### Alert Rules
```yaml
# alerts.yml
groups:
- name: epcis-kg
  rules:
  - alert: HighMemoryUsage
    expr: memory_usage_mb > 7000
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High memory usage detected"

  - alert: ServiceDown
    expr: up == 0
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "EPCIS Knowledge Graph service is down"
```

## Backup and Recovery

### 1. Backup Strategy

#### Automated Backups
```bash
# Set up cron job for automated backups
sudo crontab -e

# Add this line for daily backups at 2 AM
0 2 * * * /path/to/scripts/backup.sh --encrypt --s3-bucket your-backup-bucket

# Add this line for weekly full backups
0 3 * * 0 /path/to/scripts/backup.sh --encrypt --s3-bucket your-backup-bucket --retention-days 90
```

#### Backup Types
- **Full Backups**: Complete database and configuration backup
- **Incremental Backups**: Only changes since last backup
- **Configuration Backups**: Only configuration files
- **Ontology Backups**: Only ontology files

### 2. Recovery Procedures

#### Service Recovery
```bash
# Check service status
systemctl status epcis-kg

# Restart service
systemctl restart epcis-kg

# Check logs for errors
journalctl -u epcis-kg -n 100

# Reset service if needed
systemctl reset-failed epcis-kg
systemctl start epcis-kg
```

#### Data Recovery
```bash
# Stop service
systemctl stop epcis-kg

# Backup current data
cp -r /var/lib/epcis-kg/data /var/lib/epcis-kg/data.backup.$(date +%Y%m%d)

# Restore from backup
tar -xzf /path/to/backup.tar.gz -C /var/lib/epcis-kg/

# Set proper permissions
chown -R epcis:epcis /var/lib/epcis-kg/data

# Start service
systemctl start epcis-kg
```

#### Disaster Recovery
```bash
# On new server:
# 1. Install dependencies
sudo apt update
sudo apt install -y curl git build-essential

# 2. Create user and directories
sudo useradd -r -s /bin/false epcis
sudo mkdir -p /opt/epcis-kg /var/lib/epcis-kg/data /var/log/epcis-kg
sudo chown -R epcis:epcis /opt/epcis-kg /var/lib/epcis-kg/data /var/log/epcis-kg

# 3. Restore from backup
aws s3 cp s3://your-backup-bucket/latest-backup.tar.gz /tmp/
sudo -u epcis tar -xzf /tmp/latest-backup.tar.gz -C /var/lib/epcis-kg/

# 4. Restore configuration
sudo cp -r /var/lib/epcis-kg/config /etc/epcis-kg/

# 5. Start service
systemctl start epcis-kg
```

## Performance Tuning

### 1. System Optimization

#### Kernel Parameters
```bash
# Add to /etc/sysctl.conf
# Increase file descriptor limit
fs.file-max = 1000000

# Network optimization
net.core.rmem_max = 16777216
net.core.wmem_max = 16777216
net.ipv4.tcp_rmem = 4096 65536 16777216
net.ipv4.tcp_wmem = 4096 65536 16777216

# Apply changes
sudo sysctl -p
```

#### File System Optimization
```bash
# Use ext4 or xfs for better performance
# Mount with noatime option
# /etc/fstab
/dev/sdb1 /var/lib/epcis-kg ext4 defaults,noatime 0 2

# Create separate filesystem for data
sudo mkfs.ext4 /dev/sdb1
sudo mount /dev/sdb1 /var/lib/epcis-kg
```

### 2. Application Tuning

#### Memory Settings
```toml
# config/production.toml
[reasoning]
cache_size = 50000  # Increase for large datasets
parallel_processing = true
batch_size = 2000   # Optimize for your data size

[database]
max_connections = 200  # Increase for high concurrency
connection_pool_size = 20
```

#### Worker Configuration
```toml
[server]
workers = 8  # Set to number of CPU cores
max_connections = 2000
timeout_seconds = 30
```

### 3. Database Optimization

#### Query Optimization
- Use appropriate indexes
- Optimize SPARQL queries
- Monitor query performance
- Use query caching

#### Storage Optimization
- Use SSD storage
- Separate data and log files
- Monitor disk usage
- Implement data retention policies

## Scaling and High Availability

### 1. Horizontal Scaling

#### Load Balancer Setup
```bash
# Install HAProxy
sudo apt install haproxy

# Configure load balancer
sudo tee /etc/haproxy/haproxy.cfg << EOF
frontend epcis-kg
    bind *:80
    default_backend epcis-kg-backend

backend epcis-kg-backend
    balance roundrobin
    server epcis-kg-1 10.0.0.1:8080 check
    server epcis-kg-2 10.0.0.2:8080 check
    server epcis-kg-3 10.0.0.3:8080 check
EOF

# Restart HAProxy
sudo systemctl restart haproxy
```

#### Multi-Node Deployment
- Use Kubernetes for orchestration
- Implement shared storage (NFS, S3)
- Use external database cluster
- Implement service discovery

### 2. High Availability

#### Active-Passive Setup
```yaml
# Kubernetes example with PodDisruptionBudget
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: epcis-kg-pdb
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app: epcis-knowledge-graph
```

#### Database Replication
- Use PostgreSQL with streaming replication
- Implement read replicas
- Use connection pooling
- Monitor replication lag

## Troubleshooting

### 1. Common Issues

#### Service Won't Start
```bash
# Check service status
systemctl status epcis-kg

# Check logs
journalctl -u epcis-kg -n 50

# Check configuration
sudo -u epcis /opt/epcis-kg/epcis-knowledge-graph --config /etc/epcis-kg/config.toml validate

# Check permissions
ls -la /opt/epcis-kg/
ls -la /var/lib/epcis-kg/
```

#### High Memory Usage
```bash
# Check memory usage
free -h
ps aux | grep epcis-knowledge-graph

# Monitor memory over time
watch -n 1 'free -h'

# Check for memory leaks
valgrind --leak-check=full /opt/epcis-kg/epcis-knowledge-graph
```

#### Slow Performance
```bash
# Check system resources
top
htop
iotop

# Check database performance
sudo -u epcis /opt/epcis-kg/epcis-knowledge-graph stats

# Check network performance
netstat -tuln
ss -tuln
```

### 2. Debug Mode

#### Enable Debug Logging
```bash
# Set debug logging
export RUST_LOG=debug
systemctl edit epcis-kg

# Add this:
[Service]
Environment=RUST_LOG=debug
```

#### Performance Profiling
```bash
# Install profiling tools
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin epcis-knowledge-graph -- serve

# Check performance counters
perf stat /opt/epcis-kg/epcis-knowledge-graph serve
```

## Maintenance

### 1. Regular Maintenance Tasks

#### Daily Tasks
- Check service health
- Monitor resource usage
- Review logs for errors
- Check backup status

#### Weekly Tasks
- Update software packages
- Review performance metrics
- Clean up old logs
- Test backup restoration

#### Monthly Tasks
- Security updates
- Performance review
- Capacity planning
- Documentation updates

### 2. Updates and Upgrades

#### Application Updates
```bash
# Backup current installation
./scripts/backup.sh

# Download new version
wget https://github.com/your-repo/epcis-knowledge-graph/releases/latest/download/epcis-knowledge-graph

# Stop service
systemctl stop epcis-kg

# Replace binary
sudo cp epcis-knowledge-graph /opt/epcis-kg/
sudo chmod +x /opt/epcis-kg/epcis-knowledge-graph

# Start service
systemctl start epcis-kg

# Verify upgrade
curl http://localhost:8080/health
```

#### System Updates
```bash
# Update system packages
sudo apt update && sudo apt upgrade -y

# Update Rust toolchain
rustup update

# Update dependencies
cargo update
```

## Best Practices

### 1. Security Best Practices
- Use SSL/TLS for all communications
- Implement proper authentication and authorization
- Regular security updates
- Monitor for security vulnerabilities
- Use least privilege principle

### 2. Performance Best Practices
- Monitor resource usage continuously
- Implement proper caching strategies
- Use connection pooling
- Optimize database queries
- Regular performance testing

### 3. Reliability Best Practices
- Implement comprehensive monitoring
- Regular backup testing
- Disaster recovery planning
- High availability setup
- Load testing

### 4. Operational Best Practices
- Document all procedures
- Implement proper logging
- Use configuration management
- Regular maintenance
- Team training

## Support

### 1. Getting Help
- **Documentation**: Check the `/docs` directory
- **Logs**: Review application and system logs
- **Metrics**: Check monitoring dashboards
- **Community**: GitHub issues and discussions

### 2. Reporting Issues
- Include system information
- Provide error logs
- Describe reproduction steps
- Include configuration details
- Specify impact level

### 3. Emergency Procedures
- Service down: Check logs and restart service
- Data corruption: Restore from backup
- Security incident: Follow incident response plan
- Performance issues: Check monitoring and scale resources