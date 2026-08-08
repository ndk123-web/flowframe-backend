# FlowFrame Rust Backend

## Overview

**FlowFrame Backend** is a high-performance, asynchronous REST API built with **Rust**, **Axum**, **Tokio**, and **MongoDB**. It serves as the persistence engine for FlowFrame's interactive distributed systems architecture simulator. 

All user data (Workspaces, Diagrams, Canvas Nodes & Edges, Configurations) is securely isolated per user via **JWT (JSON Web Tokens)**.

---

## ⚡ Core Features & Implementation

1. **User Authentication & JWT Guard**
   - **Sign Up / Sign In**: Password hashing via `bcrypt`, access token generation via `jsonwebtoken`.
   - **Auth Middleware (`jwt_auth_middleware`)**: Enforces JWT verification on all protected endpoints (`/api/workspaces`, `/api/diagrams`).
   - **Firebase Token Claim Support**: Option to verify Firebase user claims directly via Google's public x509 certificates.

2. **Workspace Management API (User-Isolated)**
   - **Personal Plan Limit**: Strict limit of **5 Workspaces per user**.
   - Create, list, fetch, update (Name, Description, Environment Tag `DEV` | `STAGING` | `PROD`), and delete workspaces.

3. **Diagram & Canvas Persistence API**
   - **Workspace Limit**: Strict limit of **5 Diagrams per Workspace**.
   - **End-to-End Persistence**: Saves ReactFlow nodes, edges, DSL configurations, and metadata in MongoDB.
   - **Live Auto-Save & Manual Save (`Ctrl + S`)**: `PUT /api/workspaces/:id/diagrams/:did` endpoint.
   - **Recent Diagrams Feed**: `GET /api/diagrams/recent` fetches recently modified diagrams across all user workspaces ordered by timestamp.

4. **Production Readiness & Cloud Deployment**
   - **Dynamic CORS**: Accepts `FRONTEND_URL` from environment variables, fallback to permissive development CORS.
   - **Cloud Host Binding**: Binds dynamically to `HOST` (default `0.0.0.0`) and `PORT` (default `8000`) for containerized deployment on Railway, Render, Fly.io, or Docker.
   - **Single Configuration Source**: Clean environment configuration via `server/.env`.

---

## 🌐 API Route Reference

### 🔐 Authentication Routes (`/api/auth`)
| Method | Path | Auth Required | Description |
|--------|------|---------------|-------------|
| `POST` | `/api/auth/signup` | ❌ No | Register new user. Returns JWT & user profile |
| `POST` | `/api/auth/signin` | ❌ No | Authenticate existing user. Returns JWT & user profile |

---

### 🗂️ Workspace Routes (`/api/workspaces`)
| Method | Path | Auth Required | Description |
|--------|------|---------------|-------------|
| `GET`  | `/api/workspaces` | 🔒 Yes | List all workspaces owned by authenticated user |
| `POST` | `/api/workspaces` | 🔒 Yes | Create new workspace (Max 5 per user) |
| `GET`  | `/api/workspaces/:id` | 🔒 Yes | Fetch workspace details by ID |
| `PUT`  | `/api/workspaces/:id` | 🔒 Yes | Update workspace name, description, or env tag |
| `DELETE` | `/api/workspaces/:id` | 🔒 Yes | Delete workspace by ID |

---

### 🎨 Diagram Routes (`/api/workspaces/:id/diagrams` & `/api/diagrams`)
| Method | Path | Auth Required | Description |
|--------|------|---------------|-------------|
| `GET`  | `/api/workspaces/:id/diagrams` | 🔒 Yes | List all diagrams inside workspace |
| `POST` | `/api/workspaces/:id/diagrams` | 🔒 Yes | Create new diagram (Max 5 per workspace) |
| `GET`  | `/api/workspaces/:id/diagrams/:did` | 🔒 Yes | Fetch full diagram (nodes, edges, configs) |
| `PUT`  | `/api/workspaces/:id/diagrams/:did` | 🔒 Yes | Update diagram nodes, edges, title, description |
| `DELETE` | `/api/workspaces/:id/diagrams/:did` | 🔒 Yes | Delete diagram |
| `GET`  | `/api/diagrams/recent` | 🔒 Yes | Fetch recent diagrams across all user workspaces |

---

## 🛠️ Environment Configuration (`.env`)

Create a `.env` file in the root `server` folder:

```env
# MongoDB Atlas or Local Connection String
MONGODB_URI=mongodb+srv://<username>:<password>@cluster0.xxx.mongodb.net/
DATABASE_NAME=flowframe

# JWT Secret Key
JWT_SECRET=your_super_secret_jwt_key_here

# Optional Production Deployment Settings
HOST=0.0.0.0
PORT=8000
FRONTEND_URL=https://flowframe.vercel.app
```

---

## 🚀 Running Locally

```bash
# 1. Install dependencies & compile
cargo check

# 2. Run the Axum Server
cargo run
```

Server output:
```text
🚀 FLOWFRAME SERVER RUNNING: http://0.0.0.0:8000
```

---

## 🐳 Docker Deployment (Render / Railway)

A multi-stage `Dockerfile` for minimal container size and fast startup:

```dockerfile
FROM rust:1.80 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/server /app/server
EXPOSE 8000
CMD ["./server"]
```

---

## 📄 License
This project is licensed under the MIT License.
