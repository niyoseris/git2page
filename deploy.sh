#!/bin/bash

# Git2Page Deployment Script
set -e

echo "🚀 Git2Page Deployment Script"

# Check if .env file exists
if [ ! -f .env ]; then
    echo "❌ .env file not found. Copy .env.example to .env and configure it."
    exit 1
fi

# Build and deploy with Docker Compose
echo "📦 Building Docker image..."
docker-compose build

echo "🚀 Starting services..."
docker-compose up -d

echo "⏳ Waiting for service to be healthy..."
sleep 10

# Check if service is running
if curl -f http://localhost:5001/config > /dev/null 2>&1; then
    echo "✅ Git2Page is running at http://localhost:5001"
    echo "📊 Check logs with: docker-compose logs -f git2page"
    echo "🛑 Stop with: docker-compose down"
else
    echo "❌ Service failed to start. Check logs:"
    docker-compose logs git2page
    exit 1
fi
