# Justfile for Rust Microservices Project
# Run `just --list` to see all available commands

# Default recipe
default:
    @just --list

# Build all services
build:
    cargo build

# Build all services in release mode
build-release:
    cargo build --release

# Run tests for all services
test:
    cargo test

# Run tests with coverage
test-coverage:
    cargo tarpaulin --out Html --output-dir coverage

# Run clippy linting
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Run security audit
audit:
    cargo audit

# Clean build artifacts
clean:
    cargo clean

# Start all services with docker-compose
up:
    docker-compose up -d

# Stop all services
down:
    docker-compose down

# View logs
logs service="":
    @if [ "{{service}}" = "" ]; then \
        docker-compose logs -f; \
    else \
        docker-compose logs -f {{service}}; \
    fi

# Run database migrations
migrate:
    cd crates/database && sqlx migrate run

# Create new migration
migrate-create name:
    cd crates/database && sqlx migrate add {{name}}

# Reset database
migrate-reset:
    cd crates/database && sqlx database reset -y

# Start API service
run-api:
    cargo run --bin api-service

# Start worker service
run-worker:
    cargo run --bin worker-service

# Start event service
run-event:
    cargo run --bin event-service

# Start API service in development mode
dev-api:
    cargo watch -x "run --bin api-service"

# Start worker service in development mode
dev-worker:
    cargo watch -x "run --bin worker-service"

# Start event service in development mode
dev-event:
    cargo watch -x "run --bin event-service"

# Setup development environment
setup:
    @echo "Setting up development environment..."
    @echo "Installing required tools..."
    cargo install cargo-watch cargo-tarpaulin cargo-audit sqlx-cli
    @echo "Creating .env file from example..."
    @if [ ! -f .env ]; then cp .env.example .env; fi
    @echo "Starting dependencies with docker-compose..."
    docker-compose up -d postgres redis kafka
    @echo "Waiting for services to be ready..."
    sleep 10
    @echo "Running database migrations..."
    just migrate
    @echo "Setup complete!"

# Check if all services are healthy
health:
    @echo "Checking service health..."
    @curl -s http://localhost:8080/health || echo "API service not responding"
    @echo ""

# Generate API documentation
docs:
    cargo doc --no-deps --open

# Run benchmarks
bench:
    cargo bench

# Install development dependencies
install-deps:
    cargo install cargo-watch cargo-tarpaulin cargo-audit sqlx-cli

# Update dependencies
update:
    cargo update

# Check for outdated dependencies
outdated:
    cargo outdated

# Run full CI pipeline locally
ci: fmt-check lint test audit
    @echo "All CI checks passed!"

# Deploy to staging
deploy-staging:
    @echo "Deploying to staging..."
    # Add deployment commands here

# Deploy to production
deploy-prod:
    @echo "Deploying to production..."
    # Add deployment commands here

# Generate Kubernetes manifests
k8s-generate:
    @echo "Generating Kubernetes manifests..."
    # Add k8s generation commands here

# Apply Kubernetes manifests
k8s-apply:
    kubectl apply -f k8s/

# Remove Kubernetes resources
k8s-delete:
    kubectl delete -f k8s/

# Monitor services
monitor:
    @echo "Opening monitoring dashboard..."
    @echo "Prometheus: http://localhost:9090"
    @echo "Grafana: http://localhost:3000"

# Backup database
backup:
    @echo "Creating database backup..."
    docker-compose exec postgres pg_dump -U postgres app > backup_$(date +%Y%m%d_%H%M%S).sql

# Restore database from backup
restore file:
    @echo "Restoring database from {{file}}..."
    docker-compose exec -T postgres psql -U postgres app < {{file}}

# Load test data
seed:
    @echo "Loading test data..."
    # Add data seeding commands here

# Run performance tests
perf:
    @echo "Running performance tests..."
    # Add performance testing commands here

# Generate load
load-test:
    @echo "Running load tests..."
    # Add load testing commands here

# Check dependencies for vulnerabilities
security:
    cargo audit
    # Add additional security checks here

# Generate project report
report:
    @echo "Generating project report..."
    @echo "Lines of code:"
    @find . -name "*.rs" -not -path "./target/*" | xargs wc -l | tail -1
    @echo ""
    @echo "Dependencies:"
    @cargo tree --depth 1
    @echo ""
    @echo "Test coverage:"
    @cargo tarpaulin --print-summary

# Help command
help:
    @echo "Rust Microservices Project"
    @echo "========================="
    @echo ""
    @echo "Common commands:"
    @echo "  just setup     - Setup development environment"
    @echo "  just build     - Build all services"
    @echo "  just test      - Run all tests"
    @echo "  just up        - Start all services with Docker"
    @echo "  just dev-api   - Start API service in dev mode"
    @echo "  just ci        - Run full CI pipeline"
    @echo ""
    @echo "Run 'just --list' for all available commands"