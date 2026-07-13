<p align="center">
  <a href="https://github.com/etecoons">
    <img src="assets/header.jpg" alt="etecoons banner" width="100%">
  </a>
</p>

# <img src="assets/icon.png" width="32" height="32" valign="middle"> Grid

[![CI](https://github.com/etecoons/grid/actions/workflows/ci.yml/badge.svg)](https://github.com/etecoons/grid/actions/workflows/ci.yml)

Clean, secure, and lightning-fast self-hosted Kanban board in Rust.

## Quick Start

### Self-Hosting (Docker)
Pull and run the official Docker container:
```bash
docker run -d -p 4405:4405 -v /path/to/appdata:/app/data ghcr.io/etecoons/grid:latest
```
