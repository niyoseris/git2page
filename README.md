# Git2Page

Generate beautiful landing pages from GitHub profiles using AI analysis.

## ✨ Features

- **AI-Powered Analysis**: Uses LLM to analyze repositories and generate detailed descriptions
- **Multi-Language Support**: Supports 15+ languages for output
- **Smart Code Discovery**: Analyzes source code when README is missing
- **Export Options**: Export results as HTML, JSON, CSV, or Markdown
- **Batch Processing**: Handles large profiles with many repositories
- **Modern UI**: Beautiful, responsive design with Tailwind CSS

## 🚀 Quick Start

### Using Docker (Recommended)

1. **Clone the repository**
   ```bash
   git clone <your-repo-url>
   cd gitpage
   ```

2. **Configure environment**
   ```bash
   cp .env.example .env
   # Edit .env with your API keys
   ```

3. **Deploy**
   ```bash
   ./deploy.sh
   ```

4. **Open your browser**
   Navigate to `http://localhost:5001`

### Manual Installation

1. **Install Rust** (1.75+)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Build and run**
   ```bash
   cargo build --release
   ./target/release/git2page
   ```

## ⚙️ Configuration

Create a `.env` file:

```bash
# LLM Configuration
LLM_API_URL=https://ollama.com
LLM_API_KEY=your_api_key_here
LLM_MODEL=llama3

# GitHub Configuration (optional, for higher rate limits)
GITHUB_TOKEN=ghp_your_github_token_here
```

### Supported LLM Providers

- **Ollama Cloud**: `https://ollama.com`
- **OpenAI**: `https://api.openai.com/v1`
- **Local Ollama**: `http://localhost:11434`
- **Custom**: Any OpenAI-compatible endpoint

## 📖 Usage

1. Enter a GitHub username
2. Select your preferred output language
3. Click "Analyze Profile"
4. Wait for AI analysis (may take 1-3 minutes for large profiles)
5. Export results in your preferred format

## 🔧 Development

### Project Structure

```
gitpage/
├── src/
│   └── main.rs          # Main application logic
├── static/
│   ├── index.html       # Frontend UI
│   └── app.js          # Frontend JavaScript
├── Dockerfile          # Container configuration
├── docker-compose.yml  # Orchestration
└── DEPLOYMENT.md       # Detailed deployment guide
```

### Running in Development

```bash
# Install dependencies
cargo build

# Run development server
cargo run

# Run tests
cargo test
```

## 🌐 Deployment Options

### Docker Compose
```bash
docker-compose up -d
```

### Cloud Platforms
- **Railway**: Auto-deploy from GitHub
- **Render**: Docker web service
- **DigitalOcean**: App Platform
- **AWS**: ECS/Fargate

See [DEPLOYMENT.md](./DEPLOYMENT.md) for detailed instructions.

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with Rust and Actix-web
- UI powered by Tailwind CSS
- AI analysis by various LLM providers
- GitHub API for repository data
