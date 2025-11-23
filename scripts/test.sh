#!/bin/bash

# Test script for Rust Microservices

set -e

echo "🧪 Running Rust Microservices test suite..."

# Set test environment
export ENVIRONMENT=test
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Start test infrastructure if not running
echo "🐳 Starting test infrastructure..."
docker-compose -f docker-compose.test.yml up -d postgres-test redis-test kafka-test || {
    echo "⚠️  Test infrastructure not available, using development services"
    docker-compose up -d postgres redis kafka
}

# Wait for services
echo "⏳ Waiting for test services..."
sleep 10

# Set test database URL
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/app_test"
export REDIS_URL="redis://localhost:6379/1"

# Create test database
echo "🗄️  Setting up test database..."
sqlx database create || echo "Test database may already exist"

# Run migrations on test database
echo "📊 Running test migrations..."
sqlx migrate run || echo "⚠️  Test migrations may need manual setup"

# Run unit tests
echo "🔬 Running unit tests..."
cargo test --lib

# Run integration tests
echo "🔗 Running integration tests..."
cargo test --test '*'

# Run doc tests
echo "📚 Running documentation tests..."
cargo test --doc

# Run tests with coverage
if command -v cargo-tarpaulin &> /dev/null; then
    echo "📈 Running tests with coverage..."
    cargo tarpaulin --out Html --output-dir coverage --skip-clean
    echo "📊 Coverage report generated in coverage/tarpaulin-report.html"
else
    echo "⚠️  cargo-tarpaulin not installed, skipping coverage"
fi

# Run clippy linting
echo "🔍 Running clippy linting..."
cargo clippy --all-targets --all-features -- -D warnings

# Check formatting
echo "🎨 Checking code formatting..."
cargo fmt --all -- --check

# Security audit
if command -v cargo-audit &> /dev/null; then
    echo "🔒 Running security audit..."
    cargo audit
else
    echo "⚠️  cargo-audit not installed, skipping security audit"
fi

# Cleanup test infrastructure
echo "🧹 Cleaning up test infrastructure..."
docker-compose -f docker-compose.test.yml down || echo "Using development services"

echo "✅ All tests completed successfully!"