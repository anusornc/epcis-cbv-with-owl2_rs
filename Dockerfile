# Dockerfile for EPCIS Knowledge Graph
FROM rust:1.75-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Runtime image
FROM debian:12-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 epcis

# Create directories
RUN mkdir -p /app /var/lib/epcis-kg/data /var/log/epcis-kg /var/backups/epcis-kg && \
    chown -R epcis:epcis /app /var/lib/epcis-kg /var/log/epcis-kg /var/backups/epcis-kg

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/epcis-knowledge-graph /app/

# Copy configuration files
COPY --from=builder /app/config/ /app/config/

# Copy ontologies
COPY --from=builder /app/ontologies/ /app/ontologies/

# Set permissions
RUN chmod +x /app/epcis-knowledge-graph && \
    chown -R epcis:epcis /app

# Switch to non-root user
USER epcis

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Run the application
CMD ["./epcis-knowledge-graph", "serve", "--config", "/app/config/production.toml"]