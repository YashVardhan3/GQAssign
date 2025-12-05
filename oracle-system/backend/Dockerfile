# Use Rust official image
FROM rust:latest as builder

WORKDIR /usr/src/app
COPY . .

# Update dependencies to resolve conflicts
RUN cargo update

# Build the application
# We use --release for optimized binary
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install OpenSSL and ca-certificates (needed for HTTPS and Postgres)
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

# Copy binary from builder
COPY --from=builder /usr/src/app/target/release/oracle-backend .
# Copy migrations folder so sqlx::migrate! can find it at runtime if using file path
# Note: sqlx::migrate! macro embeds migrations if using specific options, but default might look for files.
# Actually sqlx::migrate! embeds them by default in the binary if pointing to a directory? 
# Wait, sqlx::migrate! macro embeds them. So we don't strictly need the folder at runtime if using the macro.
# But let's verify. The macro `migrate!` embeds them. 
# "The macro will embed the migrations into the binary." -> Yes.
# So we don't need to copy migrations folder.

CMD ["./oracle-backend"]
