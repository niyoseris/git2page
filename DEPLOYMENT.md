# Git2Page Deployment Guide

## Overview
Git2Page is a Rust web application that generates beautiful landing pages from GitHub profiles. This guide covers multiple deployment options.

## Prerequisites
- Docker and Docker Compose (for containerized deployment)
- Git
- GitHub Personal Access Token (recommended)
- LLM API access (Ollama Cloud, OpenAI, or compatible)

## Environment Variables

Create a `.env` file based on `.env.example`:

```bash
# LLM Configuration
LLM_API_URL=https://ollama.com
LLM_API_KEY=your_api_key_here
LLM_MODEL=llama3

# GitHub Configuration
GITHUB_TOKEN=ghp_your_github_token_here

# Server Configuration
RUST_LOG=info
```

### Environment Variables Explained

| Variable | Required | Description |
|----------|----------|-------------|
| `LLM_API_URL` | Yes | LLM API endpoint (Ollama, OpenAI, etc.) |
| `LLM_API_KEY` | No | API key if required by LLM service |
| `LLM_MODEL` | Yes | Model name (llama3, glm-5:cloud, gpt-4, etc.) |
| `GITHUB_TOKEN` | No | GitHub token for higher rate limits |
| `RUST_LOG` | No | Log level (debug, info, warn, error) |

## Deployment Options

### 1. Docker Compose (Recommended)

**Quick Start:**
```bash
# Clone and navigate
git clone <your-repo-url>
cd gitpage

# Copy environment file
cp .env.example .env
# Edit .env with your actual values

# Build and run
docker-compose up -d
```

**Production Deployment:**
```bash
# Build with production optimizations
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

### 2. Docker Standalone

```bash
# Build the image
docker build -t git2page:latest .

# Run with environment variables
docker run -d \
  --name git2page \
  -p 5001:5001 \
  -e LLM_API_URL=https://ollama.com \
  -e LLM_API_KEY=your_api_key \
  -e LLM_MODEL=llama3 \
  -e GITHUB_TOKEN=your_github_token \
  git2page:latest
```

### 3. Native Installation

**Prerequisites:**
- Rust 1.75+
- OpenSSL development libraries

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone <your-repo-url>
cd gitpage
cp .env.example .env
# Edit .env file

# Build and run
cargo build --release
./target/release/git2page
```

## Cloud Deployment

### Railway

1. Connect your GitHub repository to Railway
2. Set environment variables in Railway dashboard
3. Railway will automatically detect and deploy the Rust application

### Render

1. Create a new Web Service on Render
2. Connect your GitHub repository
3. Set environment variables
4. Use build command: `cargo build --release`
5. Use start command: `./target/release/git2page`

### DigitalOcean App Platform

1. Create a new app
2. Connect your GitHub repository
3. Configure as a Docker service
4. Set environment variables
5. Deploy

### AWS ECS

```bash
# Build and push to ECR
aws ecr create-repository --repository-name git2page
docker build -t git2page .
docker tag git2page:latest <account-id>.dkr.ecr.<region>.amazonaws.com/git2page:latest
docker push <account-id>.dkr.ecr.<region>.amazonaws.com/git2page:latest

# Deploy using ECS task definition
```

## Production Considerations

### Security
- Use HTTPS in production
- Set strong API keys
- Use environment variables, not hardcoded secrets
- Consider adding rate limiting

### Performance
- The app processes repositories in batches of 8 to avoid timeouts
- Adjust batch size in `src/main.rs` if needed
- Use a reverse proxy (nginx, traefik) for SSL termination

### Monitoring
- Health check endpoint: `GET /config`
- Logs are output to stdout (captured by Docker)
- Consider adding metrics collection

### Scaling
- The app is stateless and can be horizontally scaled
- Consider using a load balancer for multiple instances
- LLM API calls are the main bottleneck

## Troubleshooting

### Common Issues

**Timeout Errors:**
- Check LLM API connectivity
- Reduce batch size in `src/main.rs`
- Verify API key and model name

**GitHub Rate Limits:**
- Add a GitHub Personal Access Token
- Check token permissions

**Build Failures:**
- Ensure Rust 1.75+ is installed
- Install OpenSSL development libraries
- Clear cargo cache: `cargo clean`

### Health Checks

```bash
# Check if service is running
curl http://localhost:5001/config

# Check Docker logs
docker-compose logs git2page

# Check service status
docker-compose ps
```

## Support

For deployment issues:
1. Check the logs: `docker-compose logs git2page`
2. Verify environment variables
3. Test LLM API connectivity separately
4. Check GitHub token permissions

## License

Add your license information here.
