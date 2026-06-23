# RustKan - High-Performance Kanban Board

RustKan is a clean, secure, and lightning-fast self-hosted Kanban board application built in Rust using Yew (WebAssembly frontend) and Axum (API backend).

---

## 🚀 Time-To-First-Run

### Option 1: Docker Compose (Recommended)
1. Ensure a `docker-compose.yml` file is configured in your project root:
```yaml
services:
  rustkan:
    image: ubermetroid/rustkan:latest
    container_name: rustkan
    restart: unless-stopped
    ports:
      - 4405:4405
    environment:
      - PORT=4405
      - SITE_TITLE=RustKan
      - ALLOWED_ORIGINS=*
      - RUSTKAN_PIN=1234
      - APPRISE_URL=
      - APPRISE_MESSAGE=Kanban Board updated: {action}
    volumes:
      - ./data:/usr/src/app/data
```
2. Spin up the container:
```bash
docker compose up -d
```
3. Open your browser and navigate to `http://localhost:4405`.

---

## 🛠️ Local Development

### 1. Prerequisites
Ensure you have the Rust toolchain installed. Add the WebAssembly target and install the **Trunk** WASM bundler:
```bash
# Add WebAssembly target
rustup target add wasm32-unknown-unknown

# Install Trunk
wget -qO- "https://github.com/trunk-rs/trunk/releases/download/v0.21.14/trunk-x86_64-unknown-linux-gnu.tar.gz" | tar -xzf- -C /usr/local/bin
```

### 2. Run the Application
1. **Frontend**: Start the Yew development server (runs with hot-reloading at `http://localhost:8080`):
   ```bash
   cd frontend
   trunk serve
   ```
2. **Backend**: Start the Axum API server (listens on `http://localhost:4405`):
   ```bash
   cd backend
   cargo run
   ```

---

## 📋 Environment Configuration

| Variable | Description | Default | Required |
| :--- | :--- | :--- | :--- |
| `PORT` | Local host port mapping for the Axum backend | `4405` | Optional |
| `SITE_TITLE` | Custom title rendered in the navigation header | `RustKan` | Optional |
| `ALLOWED_ORIGINS` | Comma-separated HTTP request origins (CORS filter) | `*` | Optional |
| `RUSTKAN_PIN` | Optional 4-10 digit PIN to lock access to the boards | None | Optional |
| `APPRISE_URL` | Apprise API webhook URL (e.g. Discord, Telegram) | None | Optional |
| `APPRISE_MESSAGE` | Custom webhook alert message template | `Kanban Board updated: {action}` | Optional |

---

## 📂 Repository File Tree

```
RustKan/
├── backend/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       └── static_files.rs
├── data/
│   └── tasks.json
├── frontend/
│   ├── Assets/
│   │   ├── login.css
│   │   ├── service-worker.js
│   │   └── styles.css
│   ├── Cargo.toml
│   ├── index.html
│   └── src/
│       ├── header.rs
│       ├── i18n.rs
│       ├── main.rs
│       ├── storage.rs
│       └── types.rs
├── .github/
│   └── workflows/
│       └── ci.yml
├── .env
├── .env.example
├── Cargo.toml
├── docker-compose.yml
├── Dockerfile
├── logo.png
└── README.md
```
