#!/bin/bash

# Setup script for Rust Microservices development environment

set -e

echo "🚀 Setting up Rust Microservices development environment..."

# Check if required tools are installed
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ $1 is not installed. Please install it first."
        exit 1
    else
        echo "✅ $1 is installed"
    fi
}

echo "📋 Checking required tools..."
check_tool "cargo"
check_tool "docker"
check_tool "docker-compose"

# Install Rust tools
echo "🔧 Installing Rust development tools..."
cargo install cargo-watch cargo-tarpaulin cargo-audit sqlx-cli --locked || echo "⚠️  Some tools may already be installed"

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file from example..."
    cp .env.example .env
    echo "✅ .env file created. Please review and update the configuration."
else
    echo "✅ .env file already exists"
fi

# Start infrastructure services
echo "🐳 Starting infrastructure services with Docker Compose..."
docker-compose up -d postgres redis kafka

# Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
sleep 30

# Check if services are healthy
echo "🔍 Checking service health..."
docker-compose ps

# Run database migrations
echo "🗄️  Running database migrations..."
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/app"
sqlx database create || echo "Database may already exist"
cargo run --bin sqlx migrate run || echo "⚠️  Migrations may need to be run manually"

# Build all services
echo "🔨 Building all services..."
cargo build

echo "✅ Setup complete!"
echo ""
echo "🎯 Next steps:"
echo "1. Review and update .env file with your configuration"
echo "2. Run 'just dev-api' to start the API service in development mode"
echo "3. Run 'just dev-worker' to start the worker service in development mode"
echo "4. Run 'just dev-event' to start the event service in development mode"
echo "5. Visit http://localhost:8080/health to check API health"
echo "6. Visit http://localhost:3000 for Grafana dashboard (admin/admin)"
echo "7. Visit http://localhost:9091 for Prometheus metrics"
echo "8. Visit http://localhost:16686 for Jaeger tracing"
echo ""
echo "📚 Run 'just help' for more available commands"