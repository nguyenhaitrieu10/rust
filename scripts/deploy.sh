#!/bin/bash

# Deployment script for Rust Microservices

set -e

# Configuration
ENVIRONMENT=${1:-staging}
REGISTRY=${DOCKER_REGISTRY:-"localhost:5000"}
VERSION=${VERSION:-$(git rev-parse --short HEAD)}

echo "🚀 Deploying Rust Microservices to $ENVIRONMENT..."
echo "📦 Registry: $REGISTRY"
echo "🏷️  Version: $VERSION"

# Validate environment
if [[ ! "$ENVIRONMENT" =~ ^(staging|production)$ ]]; then
    echo "❌ Invalid environment. Use 'staging' or 'production'"
    exit 1
fi

# Check if we're on a clean git state for production
if [ "$ENVIRONMENT" = "production" ]; then
    if [ -n "$(git status --porcelain)" ]; then
        echo "❌ Working directory is not clean. Commit your changes before production deployment."
        exit 1
    fi
fi

# Build and tag Docker images
echo "🔨 Building Docker images..."

services=("api-service" "worker-service" "event-service")

for service in "${services[@]}"; do
    echo "📦 Building $service..."
    docker build -f crates/$service/Dockerfile -t $REGISTRY/rust-microservices-$service:$VERSION .
    docker tag $REGISTRY/rust-microservices-$service:$VERSION $REGISTRY/rust-microservices-$service:latest
    
    echo "📤 Pushing $service to registry..."
    docker push $REGISTRY/rust-microservices-$service:$VERSION
    docker push $REGISTRY/rust-microservices-$service:latest
done

# Deploy to Kubernetes
if command -v kubectl &> /dev/null; then
    echo "☸️  Deploying to Kubernetes..."
    
    # Apply namespace
    kubectl apply -f k8s/namespace.yml
    
    # Apply ConfigMaps and Secrets
    kubectl apply -f k8s/configmap-$ENVIRONMENT.yml
    kubectl apply -f k8s/secrets-$ENVIRONMENT.yml
    
    # Apply services
    for service in "${services[@]}"; do
        echo "🚀 Deploying $service to Kubernetes..."
        
        # Update image tag in deployment
        sed "s/{{VERSION}}/$VERSION/g" k8s/$service/deployment.yml | kubectl apply -f -
        kubectl apply -f k8s/$service/service.yml
        
        # Wait for rollout
        kubectl rollout status deployment/rust-microservices-$service -n rust-microservices --timeout=300s
    done
    
    # Apply ingress
    kubectl apply -f k8s/ingress-$ENVIRONMENT.yml
    
    echo "✅ Kubernetes deployment completed"
else
    echo "⚠️  kubectl not found, skipping Kubernetes deployment"
fi

# Deploy with Docker Compose (alternative)
if [ -f "docker-compose.$ENVIRONMENT.yml" ]; then
    echo "🐳 Deploying with Docker Compose..."
    
    # Update image tags in compose file
    export IMAGE_TAG=$VERSION
    docker-compose -f docker-compose.$ENVIRONMENT.yml up -d
    
    echo "✅ Docker Compose deployment completed"
fi

# Run health checks
echo "🏥 Running health checks..."
sleep 30

# Check API health
if curl -f http://localhost:8080/health > /dev/null 2>&1; then
    echo "✅ API service is healthy"
else
    echo "❌ API service health check failed"
    exit 1
fi

# Check metrics endpoint
if curl -f http://localhost:9090/metrics > /dev/null 2>&1; then
    echo "✅ Metrics endpoint is accessible"
else
    echo "⚠️  Metrics endpoint not accessible"
fi

echo "🎉 Deployment to $ENVIRONMENT completed successfully!"
echo ""
echo "📊 Monitoring URLs:"
echo "  API Health: http://localhost:8080/health"
echo "  Metrics: http://localhost:9090/metrics"
echo "  Grafana: http://localhost:3000"
echo "  Jaeger: http://localhost:16686"
echo ""
echo "🔧 Management commands:"
echo "  View logs: docker-compose logs -f"
echo "  Scale services: docker-compose up -d --scale api-service=3"
echo "  Rollback: git checkout <previous-commit> && ./scripts/deploy.sh $ENVIRONMENT"