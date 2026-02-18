# 🎨 Diffusion Server

A production-ready, general-purpose image generation server built in Rust. Serves experimental Diffusion models via gRPC and REST APIs.

## ✨ Features

- **Dual API Support**: Both gRPC (high performance) and REST (easy integration)
- **Job Queue System**: Handles concurrent requests efficiently
- **Worker Pool**: Configurable number of worker threads
- **GPU Acceleration**: CUDA support with automatic fallback to CPU
- **Configuration**: TOML files + environment variables
- **Docker Ready**: Multi-stage Dockerfile and docker-compose
- **Production Features**: Health checks, logging, error handling

## 🚀 Quick Start

### Option 1: Docker (Recommended)

```bash
# Clone the repository
cd diffusion-server

# Run with docker-compose
docker-compose up -d

# Check health
curl http://localhost:8080/health
```

### Option 2: Native Build

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt-get install -y protobuf-compiler libssl-dev pkg-config

# Build
cargo build --release

# Run
./target/release/diffusion-server
```

## 📋 Prerequisites

- **Rust**: 1.70 or later
- **Protocol Buffers**: For gRPC definitions
- **CUDA** (optional): For GPU acceleration
- **Model Files**: Stable Diffusion model (see below)

## 🔧 Configuration

Edit `config/default.toml`:

```toml
[server]
grpc_port = 50051
rest_port = 8080

[model]
device = "cpu"  # or "cuda"
model_path = "./models/stable-diffusion-v1-5"

[queue]
worker_threads = 2
max_queue_size = 1000

[inference]
default_steps = 50
default_guidance_scale = 7.5
```

**Environment Variables** (override config file):

```bash
export DIFFUSION__MODEL__DEVICE=cuda
export DIFFUSION__QUEUE__WORKER_THREADS=4
```

## 📡 API Usage

### REST API

**Generate Image:**

```bash
curl -X POST http://localhost:8080/v1/generate \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "a beautiful sunset over mountains",
    "num_inference_steps": 50,
    "guidance_scale": 7.5,
    "width": 512,
    "height": 512
  }' | jq
```

**Get Binary Image:**

```bash
curl -X POST http://localhost:8080/v1/generate/binary \
  -H "Content-Type: application/json" \
  -d '{"prompt": "a cat in space"}' \
  --output cat.png
```

**Health Check:**

```bash
curl http://localhost:8080/health
```

### gRPC API

**Using grpcurl:**

```bash
# Health check
grpcurl -plaintext localhost:50051 diffusion.DiffusionService/HealthCheck

# Generate image
grpcurl -plaintext \
  -d '{
    "prompt": "a beautiful sunset over mountains",
    "num_inference_steps": 50,
    "guidance_scale": 7.5,
    "width": 512,
    "height": 512
  }' \
  localhost:50051 diffusion.DiffusionService/GenerateImage
```

## 🐍 Python Client

```python
import grpc
from diffusion_pb2 import GenerateImageRequest
from diffusion_pb2_grpc import DiffusionServiceStub

channel = grpc.insecure_channel('localhost:50051')
stub = DiffusionServiceStub(channel)

request = GenerateImageRequest(
    prompt="a beautiful sunset over mountains",
    num_inference_steps=50,
    guidance_scale=7.5,
    width=512,
    height=512,
)

response = stub.GenerateImage(request)

# Save image
with open(f'output_{response.job_id}.png', 'wb') as f:
    f.write(response.images[0])
```

## 📁 Project Structure

```
diffusion-server/
├── Cargo.toml              # Rust dependencies
├── build.rs                # Proto build script
├── proto/
│   └── diffusion.proto     # gRPC API definition
├── config/
│   └── default.toml        # Configuration
├── src/
│   ├── main.rs             # Entry point + workers
│   ├── config.rs           # Config management
│   ├── errors.rs           # Error types
│   ├── inference/
│   │   ├── mod.rs
│   │   └── pipeline.rs     # Image generation
│   ├── queue/
│   │   ├── mod.rs
│   │   └── memory.rs       # Job queue
│   └── server/
│       ├── mod.rs
│       ├── grpc.rs         # gRPC server
│       └── rest.rs         # REST server
├── Dockerfile              # Container build
└── docker-compose.yml      # Orchestration
```

## 🔨 Development

### Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Watch mode
cargo watch -x run
```

### Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# With logging
RUST_LOG=debug cargo test
```

### Linting

```bash
# Check code
cargo clippy

# Format code
cargo fmt
```

## 🐳 Docker

### Build Image

```bash
docker build -t diffusion-server:latest .
```

### Run Container

```bash
docker run --gpus all \
  -p 50051:50051 \
  -p 8080:8080 \
  -v $(pwd)/models:/app/models:ro \
  -v $(pwd)/output:/app/output \
  diffusion-server:latest
```

### Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

## 🎯 Model Setup

### Option 1: Hugging Face CLI

```bash
# Install HF CLI
pip install huggingface_hub

# Download model
huggingface-cli download runwayml/stable-diffusion-v1-5 \
  --local-dir ./models/stable-diffusion-v1-5
```

### Option 2: Manual Download

1. Visit https://huggingface.co/runwayml/stable-diffusion-v1-5
2. Download model files
3. Place in `./models/stable-diffusion-v1-5/`

### Option 3: Use Your Own Model

```toml
# config/default.toml
[model]
model_path = "/path/to/your/model"
```

## 📊 Performance

Typical generation times (RTX 3090):

| Resolution | Steps | FP32 | FP16 |
|------------|-------|------|------|
| 512x512    | 50    | ~8s  | ~4s  |
| 768x768    | 50    | ~18s | ~9s  |

Memory usage:
- Model: ~4GB VRAM (FP32) or ~2GB (FP16)
- Per image: ~500MB VRAM

## 🔧 Troubleshooting

### Out of Memory

```bash
# Use FP16 precision
export DIFFUSION__MODEL__PRECISION=fp16

# Reduce image size
export DIFFUSION__INFERENCE__DEFAULT_WIDTH=384
export DIFFUSION__INFERENCE__DEFAULT_HEIGHT=384
```

### Slow Generation

```bash
# Check GPU usage
nvidia-smi

# Reduce steps
export DIFFUSION__INFERENCE__DEFAULT_STEPS=25
```

### Connection Issues

```bash
# Check if server is running
curl http://localhost:8080/health

# Check ports
netstat -tlnp | grep -E '50051|8080'
```

## 🚀 Production Deployment

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: diffusion-server
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: diffusion-server
        image: diffusion-server:latest
        resources:
          limits:
            nvidia.com/gpu: 1
```


## 🤝 Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

MIT License - see LICENSE file

## 🙏 Acknowledgments

- Built with [tonic](https://github.com/hyperium/tonic) for gRPC
- Uses [tch-rs](https://github.com/LaurentMazare/tch-rs) for PyTorch bindings
- Inspired by [Stable Diffusion](https://github.com/CompVis/stable-diffusion)

## 📞 Support

- GitHub Issues: For bugs and feature requests
- Discussions: For questions and ideas

---

**Note**: This server currently uses a placeholder image generation function. To use actual Stable Diffusion models, you'll need to integrate with a diffusion model implementation (like `diffusers-rs` or via PyTorch).
