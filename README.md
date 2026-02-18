# eToro Trading Automation App

A secure trading automation system built with **Rust (backend)** and **React + TypeScript (frontend)**.

The backend is the only component allowed to communicate with the eToro API.  
The API key is stored exclusively on the backend and must never be exposed to the frontend.

---

## 🏗 Architecture

```
React + TypeScript (UI)
        ↓ HTTP
Rust Backend (Axum API)
        ↓ HTTP
eToro API (Demo or Real)
```

---

## 📦 Tech Stack

### Backend
- Rust (stable)
- Axum (HTTP server)
- Reqwest (HTTP client)
- Tokio (async runtime)
- dotenvy (environment management)

### Frontend
- React
- TypeScript
- Vite

---

## ✅ Requirements

- Linux
- Rust (stable toolchain)
- Node.js + npm

Verify installation:

```bash
rustc --version
cargo --version
node --version
npm --version
```

---

## 🔐 Security Rules

- NEVER expose the eToro API key in the frontend.
- ALWAYS start development with a Demo key.
- Use IP whitelist when generating keys.
- Set an expiration date on API keys.
- Rotate keys regularly.
- Do not commit `.env` files to version control.

---

## 🚀 Setup

### 1. Clone the project

```bash
git clone <your-repository-url>
cd <your-project-folder>
```

---

### 2. Backend Setup

```bash
cd backend
```

Create a `.env` file:

```env
RUST_LOG=info

ETORO_BASE_URL=https://api-demo.etoro.com
ETORO_API_KEY=your_demo_key_here

BIND_ADDR=127.0.0.1:8080
CORS_ORIGIN=http://localhost:5173
```

Run the backend:

```bash
cargo run
```

Health check:

```
http://127.0.0.1:8080/health
```

Expected response:

```json
{ "ok": true }
```

---

### 3. Frontend Setup

```bash
cd frontend
npm install
npm run dev
```

Frontend runs at:

```
http://localhost:5173
```

---

## 📂 Project Structure

```
/backend
    Cargo.toml
    src/
        main.rs
        config.rs
        routes.rs
        etoro.rs
        models.rs

/frontend
    package.json
    src/
```

---

## 🔄 API Flow Example

1. Frontend sends:

```
POST /api/orders
```

2. Backend:
   - Validates request
   - Adds Authorization header
   - Sends request to eToro
   - Returns structured response

---

## 🧪 Development Workflow

1. Use Demo API key.
2. Validate:
   - Order placement
   - Position retrieval
   - Error handling
3. Deploy backend to VPS (fixed IP recommended).
4. Generate a Real API key.
5. Update `.env` with real credentials.
6. Tighten CORS and IP whitelist rules.

---

## 🏭 Production Recommendations

- Deploy backend on VPS with static IP.
- Use HTTPS (Let's Encrypt).
- Enable logging and monitoring.
- Limit CORS to your frontend domain.
- Use short-lived API keys (30 days max).
- Implement risk management logic before real trading.

---

## ⚠ Disclaimer

This software interacts with financial markets.  
Improper configuration may result in financial loss.  
Always test thoroughly in Demo mode before using real funds.

---

## 🎯 Objective

Build a secure, production-ready trading automation engine with:

- Strong backend isolation
- Safe API key management
- Clean architecture
- Scalable structure
- Full control over trading logic
