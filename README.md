# ObjectIO

[![Build Status](https://github.com/devstroop/object-io/workflows/CI/badge.svg)](https://github.com/devstroop/object-io/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**ObjectIO** is a high-performance, S3-compatible self-hosted object storage system built in Rust. It provides AWS S3 API compatibility while offering the flexibility and control of self-hosted infrastructure.

## 🚀 Features

- **S3 Compatible API**: Full compatibility with AWS S3 REST API
- **High Performance**: Built in Rust for maximum performance and safety
- **Scalable Architecture**: Modular design supporting horizontal scaling
- **Multiple Storage Backends**: Filesystem, cloud storage, and more
- **Web Management Interface**: Modern web UI built with Leptos
- **Advanced Security**: AWS SigV4 authentication, bucket policies, and ACLs
- **Monitoring & Observability**: Built-in metrics, tracing, and health checks
- **Docker Ready**: Containerized deployment with Kubernetes support

## 🏗️ Architecture

ObjectIO is designed as a modular system with the following components:

```
┌─────────────────┐    ┌─────────────────┐
│   Web Console   │    │   S3 API Server │
│    (Leptos)     │    │     (Axum)      │
│  Management UI  │    │  REST Endpoint  │
└─────────────────┘    └─────────────────┘
         │                       │
         └───────────────────────┼───────────────────────┐
                                 │                       │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Storage Engine │    │  Metadata Store │    │  Security Layer │
│  (Configurable) │    │   (SurrealDB)   │    │ (Auth & Policies│
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Core Components

- **`object-io-core`**: Shared types, utilities, and business logic
- **`object-io-api`**: S3-compatible REST API implementation
- **`object-io-storage`**: Pluggable storage backend abstraction
- **`object-io-metadata`**: Metadata management and SurrealDB integration
- **`object-io-server`**: Main server binary and configuration
- **`console`**: Unified web-based management interface

## 🛠️ Quick Start

### Prerequisites

- **Rust** 1.75+ (latest stable recommended)
- **SurrealDB** 1.1+
- **Node.js** 18+ (for frontend development)
- **Docker** (optional, for containerized deployment)

### Development Setup

1. **Clone the repository**:

   ```bash
   git clone https://github.com/devstroop/object-io.git
   cd object-io
   ```

2. **Install dependencies**:

   ```bash
   # Install Rust dependencies
   cargo build

   # Install console dependencies
   cd console && npm install && cd ..
   ```

3. **Start SurrealDB**:

   ```bash
   surreal start --log trace --user root --pass root file://objectio.db
   ```

4. **Configure environment**:

   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

5. **Run the development server**:

   ```bash
   cargo run --bin object-io-server
   ```

6. **Start the console** (in another terminal):
   ```bash
   cd console && trunk serve --open
   ```

### Docker Deployment

```bash
# Build the Docker image
docker build -t object-io:latest .

# Run with docker-compose
docker-compose up -d
```

## 📖 Documentation

- [API Documentation](docs/api.md) - Complete S3 API reference
- [Configuration Guide](docs/configuration.md) - Server and deployment configuration
- [Storage Backends](docs/storage.md) - Available storage backend options
- [Security Guide](docs/security.md) - Authentication and authorization
- [Development Guide](docs/development.md) - Contributing and development setup

## 🔧 Configuration

ObjectIO can be configured via environment variables or configuration files:

```toml
[server]
host = "0.0.0.0"
port = 3000

[database]
url = "surreal://localhost:8000/objectio"

[storage]
backend = "filesystem"
root_path = "/var/lib/objectio/data"

[auth]
default_region = "us-east-1"
signature_version = "v4"
```

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out html

# Integration tests
cargo test --test integration

# Load testing
cargo run --bin load-test
```

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Acknowledgments

- [AWS S3 API Documentation](https://docs.aws.amazon.com/s3/)
- [SurrealDB](https://surrealdb.com/) for the excellent database
- [Axum](https://github.com/tokio-rs/axum) for the web framework
- [Leptos](https://leptos.dev/) for the frontend framework

---

**ObjectIO** - Self-hosted object storage, simplified.
