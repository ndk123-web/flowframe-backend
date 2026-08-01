# FlowFrame Backend

## Overview

This repository implements the backend for **FlowFrame**, a collaborative workspace platform. The service is built with **Rust**, **Axum**, and **MongoDB**. Authentication is handled via **JWT** tokens. All user‑specific data (workspace files, diagrams, code snippets) will be isolated by the user's unique ID.

## Features (Current)
1. **Sign Up** – Register a new user with email and password. The password is securely hashed using bcrypt and stored in MongoDB.
2. **Sign In** – Authenticate a user and issue a signed JWT access token containing the user ID and email.
3. **User Workspace Isolation** – Future endpoints will store workspace data under a `userId` key, ensuring each user sees only their own data.
4. **User Diagrams (JSON)** – Diagrams will be saved as JSON objects isolated by `userId`.
5. **User Code Snippets** – Code snippets will also be stored per user.

## Architecture
- **Web Framework**: Axum (router, handlers, middleware)
- **Database**: MongoDB driver for async operations
- **Authentication**: JWT (jsonwebtoken crate) + bcrypt for password hashing
- **State Management**: `AppState` (Arc) holds configuration, database handle, and service instances shared across request handlers.
- **Configuration**: Loaded from `.env` (MONGODB_URI, DATABASE_NAME, JWT_SECRET).

## API Endpoints (Implemented)
| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/api/auth/signup` | Register a new user. Returns `{ access_token, user { id, email, type_of_signin } }` |
| `POST` | `/api/auth/signin` | Authenticate an existing user. Returns the same payload as signup.
| `GET`  | `/` | Simple health check returning a static string.

### Request/Response Examples
#### Sign Up
```http
POST /api/auth/signup HTTP/1.1
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "strongPassword123",
  "type_of_signin": "email"
}
```
**Response (201 Created)**
```json
{
  "access_token": "<jwt_token>",
  "user": {
    "id": "64d2a1f2b3e4f5a6b7c8d9e0",
    "email": "user@example.com",
    "type_of_signin": "email"
  }
}
```
#### Sign In
```http
POST /api/auth/signin HTTP/1.1
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "strongPassword123"
}
```
**Response (200 OK)** – Same JSON structure as signup.

## Data Model
```rust
use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub password_hash: String,
    pub type_of_signin: String,
}
```
* `id` – MongoDB ObjectId, used as the primary identifier for all future user‑scoped resources.
* `email` – Unique login identifier.
* `password_hash` – Bcrypt hash of the plaintext password.
* `type_of_signin` – Currently always "email" but can be extended for OAuth providers.

## Request Flow (Sign Up / Sign In)
1. **Request** arrives at the appropriate route.
2. **Handler** extracts JSON payload and forwards it to `AuthService`.
3. **AuthService**:
   - For **signup**: checks email uniqueness, hashes password, inserts a new `User` document, then generates a JWT.
   - For **signin**: fetches the user by email, verifies the password with bcrypt, and generates a JWT.
4. **JWT** contains `sub` (user ID) and `email` claims, with a 24‑hour expiry.
5. **Response** includes the token and a short `UserData` object for the client to store.

## Future Extensions
- **Workspace API** – Endpoints to create, read, update, and delete workspace data scoped by the `userId` extracted from the JWT.
- **Diagram API** – Store diagram definitions as JSON blobs under `/api/diagrams`.
- **Code API** – Manage code snippets per user.
- **Authorization Middleware** – A global JWT validation layer that extracts the user ID and injects it into request extensions for the protected routes.

## Running the Service
```bash
# Ensure MongoDB is running locally
cargo run
```
The server starts on `http://127.0.0.1:8000` with CORS permissively enabled for development.

## License
This project is licensed under the MIT License.
