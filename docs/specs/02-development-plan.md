# Synapse Development Plan

Granular roadmap from planning → first line of code → production.

---

## Decisions Log

Resolutions from grilling session (2026-06-30):

| # | Decision | Choice | Rationale |
|---|----------|--------|-----------|
| 1 | Auth provider | Firebase Phone OTP | Cost-sensitive MVP, migration tomorrow's problem |
| 2 | WebSocket bus | Redis Pub/Sub | Horizontal scaling ready, latency negligible |
| 3 | Refresh tokens | Rotate on each use | Prevents leaked tokens from 30-day usability |
| 4 | Database | Supabase Postgres | Managed DB, vendor lock-in tomorrow's problem |
| 5 | Media upload | Backend proxy | e2-micro has no bandwidth to spare, client direct would bottleneck |
| 6 | Message deletion | Soft delete (`deleted_at`) | Compliance requirement + QoL for accidental deletes |
| 7 | Board ack | Reset on any edit | Ensures "acknowledged" means "read current version" |
| 8 | Pagination cursor | UUIDv7 message ID | Time-ordered, simple comparison, already in schema |
| 9 | 1:1 DMs | Separate solution (not text channels) | Cleaner schema, dedicated UI handling |
| 10 | Notifications | User-configurable per channel | Balances engagement with noise prevention |
| 11 | Backend deploy | GitHub Actions → SSH → pull → recreate | Automated, matches existing React Native Web CI/CD pattern |
| 12 | RLS policies | Implement at ship time | Supabase warns, but safety net non-negotiable before launch |
| 13 | OTP lockout | 15 min after 5 failed attempts | Simple brute-force prevention, no email dependency |
| 14 | Offline caching | Last 100 messages + queue + server wins | Covers most use cases, conflict resolution straightforward |
| 15 | Active user metric | Daily: sent/received ≥1 message | Measures real engagement, not just login |

---

## Phase 0: Planning & Design (Week 0)

**Goal:** Finalize specs, set up tooling, create shared understanding.

### Tasks

- [ ] **Finalize API spec**
  - [ ] Define all REST endpoints with request/response schemas
  - [ ] Define WebSocket message formats (connect, message, presence, etc.)
  - [ ] Define error response format (RFC 9457 Problem Details)
  - [ ] Version strategy: `/api/v1/` prefix
  - [ ] Output: OpenAPI 3.0 spec or equivalent markdown doc

- [ ] **Database schema design**
  - [ ] ERD finalization (see `arch-diag.md`)
  - [ ] Index strategy for common queries
  - [ ] Migration naming convention: `YYYYMMDD_hhmmss_descriptive_name`
  - [ ] Seed data requirements

- [ ] **UI/UX wireframes**
  - [ ] Login flow (phone → OTP → display name)
  - [ ] Workspace list → channel list → chat view
  - [ ] Board view (admin + user)
  - [ ] Mobile-responsive layouts

- [ ] **Dev environment setup**
  - [ ] GCP e2-micro instance provisioned (Ubuntu 22.04, 2 core, 1GB RAM)
  - [ ] SSH key configured, sudo access
  - [ ] Local React Native dev environment (Web, iOS, Android emulators)
  - [ ] Rust toolchain installed (stable, tokio-console)
  - [ ] Docker installed for local DB/Redis

- [ ] **Account creation**
  - [ ] Supabase project created
  - [ ] Upstash Redis instance created
  - [ ] Cloudflare account with R2 bucket
  - [ ] Firebase project (Phone Auth + FCM)
  - [ ] Cloudflare Pages account (React Native Web deploy)
  - [ ] GitHub repo initialized with README

---

## Phase 1: Foundation & Infrastructure (Weeks 1–2)

**Goal:** CI/CD pipeline, database schema, backend/frontend scaffolding.

### Week 1: Backend Foundation

**Day 1–2: Project Scaffolding**
- [ ] Initialize Axum project: `cargo init --name synapse-backend`
- [ ] Configure `Cargo.toml` with dependencies:
  ```toml
  [dependencies]
  axum = "0.7"
  tokio = { version = "1", features = ["full"] }
  sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "migrate"] }
  redis = { version = "0.25", features = ["tokio-comp", "connection-manager"] }
  serde = { version = "1", features = ["derive"] }
  serde_json = "1"
  uuid = { version = "1", features = ["v7", "serde"] }
  chrono = { version = "0.4", features = ["serde"] }
  tracing = "0.1"
  tracing-subscriber = { version = "0.3", features = ["json"] }
  tokio-tungstenite = "0.21"
  serde_urlencoded = "0.7"
  tower-http = { version = "0.5", features = ["cors"] }
  anyhow = "1"
  ```
- [ ] Create directory structure:
  ```
  src/
  ├── main.rs
  ├── config.rs
  ├── db/
  │   ├── mod.rs
  │   └── migrations.rs
  ├── routes/
  │   ├── mod.rs
  │   ├── auth.rs
  │   ├── workspace.rs
  │   ├── channel.rs
  │   ├── message.rs
  │   └── board.rs
  ├── ws/
  │   ├── mod.rs
  │   ├── handler.rs
  │   └── manager.rs
  └── middleware/
      ├── mod.rs
      └── auth.rs
  ```
- [ ] Configure `config.rs` for env vars:
  - `DATABASE_URL`
  - `REDIS_URL`
  - `JWT_SECRET`
  - `FIREBASE_CREDENTIALS_PATH`
  - `R2_BUCKET_NAME`
  - `R2_ACCESS_KEY_ID`
  - `R2_SECRET_ACCESS_KEY`
  - `PORT` (default 8080)

**Day 3–4: Database & Migrations**
- [ ] Install SQLx CLI: `cargo install sqlx-cli`
- [ ] Create `migrations/` directory
- [ ] Write first migration: `0001_create_users.sql`
  ```sql
  CREATE TABLE users (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      phone TEXT UNIQUE NOT NULL,
      display_name TEXT NOT NULL,
      avatar_url TEXT,
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  ```
- [ ] Write migration: `0002_create_workspaces.sql`
  ```sql
  CREATE TABLE workspaces (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      name TEXT NOT NULL,
      slug TEXT UNIQUE NOT NULL,
      owner_id UUID NOT NULL REFERENCES users(id),
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  ```
- [ ] Write migration: `0003_create_channels.sql`
  ```sql
  CREATE TABLE channels (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
      name TEXT NOT NULL,
      channel_type TEXT NOT NULL CHECK (channel_type IN ('text', 'board')),
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  ```
- [ ] Write migration: `0004_create_workspace_members.sql`
  ```sql
  CREATE TABLE workspace_members (
      workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
      user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
      role TEXT NOT NULL DEFAULT 'member' CHECK (role IN ('owner', 'admin', 'member')),
      joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      PRIMARY KEY (workspace_id, user_id)
  );
  ```
- [ ] Write migration: `0005_create_messages.sql`
  ```sql
  CREATE TABLE messages (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
      sender_id UUID NOT NULL REFERENCES users(id),
      content TEXT,
      media_url TEXT,
      reply_to_id UUID REFERENCES messages(id),
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      deleted_at TIMESTAMPTZ
  );
  ```
- [ ] Write migration: `0006_create_board_posts.sql`
  ```sql
  CREATE TABLE board_posts (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
      author_id UUID NOT NULL REFERENCES users(id),
      title TEXT NOT NULL,
      body TEXT NOT NULL,
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  ```
- [ ] Write migration: `0007_create_board_ack.sql`
  ```sql
  CREATE TABLE board_acknowledgements (
      post_id UUID NOT NULL REFERENCES board_posts(id) ON DELETE CASCADE,
      user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
      acknowledged_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
      PRIMARY KEY (post_id, user_id)
  );
  ```
- [ ] Run `sqlx migrate run` against local Postgres
- [ ] Verify schema with `psql \dt` and `\d users`

**Day 5–7: Axum Server Basics**
- [ ] Implement `main.rs`:
  - Load config from env
  - Initialize tracing (JSON output)
  - Create database pool (SQLx)
  - Create Redis connection pool
  - Start HTTP server on `$PORT`
  - Graceful shutdown on SIGINT/SIGTERM
- [ ] Implement `config.rs`:
  - Load env vars with `dotenvy`
  - Type-safe config struct
  - Validation on startup
- [ ] Add health check endpoint: `GET /health` → `{"status": "ok"}`
- [ ] Add CORS middleware (allow all origins for dev, restrict in prod)
- [ ] Test server starts and responds to `/health`

### Week 2: CI/CD & Frontend Scaffolding

**CI/CD Setup (Day 1–2)**
- [ ] Create `.github/workflows/rust.yml`:
  - Trigger: PR and push to `main`
  - Steps: `cargo check`, `cargo clippy -- -D warnings`, `cargo test`, `docker build`
  - On push to `main`: SSH to GCP e2-micro, `docker pull`, `docker rm` old, `docker run` new
- [ ] Create `.github/workflows/yarn-react-native-web.yml`:
  - Trigger: PR and push to `main`
  - Steps: setup React Native, `npm install`, `npm test`, `npx react-native build-web`
  - Deploy to Cloudflare Pages if push to `main`
- [ ] Create `rust-toolchain.toml`:
  ```toml
  [toolchain]
  channel = "stable"
  components = ["rustfmt", "clippy"]
  ```
- [ ] Create `.cargo/config.toml` for SQLx offline mode (optional)

**Frontend Scaffolding (Day 3–5)**
- [ ] Initialize React Native project: `npx react-native init synapse --platforms web,ios,android`
- [ ] Configure `pubspec.yaml`:
  ```yaml
  dependencies:
/* RN: skip */
/* RN: skip */
    zustand: ^2.5.1
    go_router: ^14.2.0
    dio: ^5.4.3
    web_socket_channel: ^3.0.0
    react-native-keychain: ^9.2.2
    image_picker: ^1.1.2
    cached_network_image: ^3.4.0
    intl: ^0.19.0
  
  dev_dependencies:
    jest:
      /* RN: no SDK dependency */ 
    mockito: ^5.4.4
    build_runner: ^2.4.9
  ```
- [ ] Create directory structure:
  ```
  lib/
  ├── main.dart
  ├── app.dart
  ├── config/
  │   └── env.dart
  ├── routes/
  │   ├── app_router.dart
  │   ├── auth_routes.dart
  │   ├── workspace_routes.dart
  │   └── board_routes.dart
  ├── services/
  │   ├── auth_service.dart
  │   ├── api_service.dart
  │   ├── websocket_service.dart
  │   └── storage_service.dart
  ├── models/
  │   ├── user.dart
  │   ├── workspace.dart
  │   ├── channel.dart
  │   ├── message.dart
  │   └── board_post.dart
  ├── providers/
  │   ├── auth_provider.dart
  │   ├── workspace_provider.dart
  │   ├── channel_provider.dart
  │   └── message_provider.dart
  └── ui/
      ├── auth/
      │   ├── phone_input_screen.dart
      │   ├── otp_input_screen.dart
      │   └── display_name_screen.dart
      ├── home/
      │   ├── workspace_list_screen.dart
      │   ├── channel_sidebar.dart
      │   └── chat_view.dart
      └── boards/
          ├── board_list_screen.dart
          └── board_post_card.dart
  ```
- [ ] Set up GoRouter with auth state listener
- [ ] Configure Dio with JWT interceptor
- [ ] Create base API service with error handling (RFC 9457)

**Cloudflare Setup (Day 5–7)**
- [ ] Create R2 bucket `synapse-media`
- [ ] Configure CORS: allow `*.cloudflarepages.com` and `localhost:*`
- [ ] Create R2 worker or use presigned URL logic (backend)
- [ ] Configure Cloudflare Pages for React Native Web:
  - Build command: `npx react-native build-web --release`
  - Output directory: `build/web`
  - Branch: `main`

**Supabase & Upstash (Day 7)**
- [ ] Configure Supabase project:
  - Enable email/password (optional, for admin)
  - Configure Row Level Security (RLS) policies later
  - Get anon key and service role key
- [ ] Configure Upstash Redis:
  - Get Redis URL
  - Test connection from backend

---

## Phase 2: Auth & The "Pipes" (Weeks 3–4)

**Goal:** Phone OTP login, JWT, WebSocket engine.

### Week 3: Authentication

**Backend (Day 1–3): Firebase Phone OTP**
- [ ] Add Firebase Admin SDK dependency: `firebase-admin = "0.17"`
- [ ] Implement `/api/v1/auth/send-otp`:
  ```rust
  POST /api/v1/auth/send-otp
  Body: { "phone": "+1234567890" }
  Response: { "message": "OTP sent" }
  ```
  - Validate phone format (E.164)
  - Call Firebase `verifyPhoneNumber`
  - Rate limit: 1 OTP per phone per 60s
- [ ] Implement `/api/v1/auth/verify-otp`:
  ```rust
  POST /api/v1/auth/verify-otp
  Body: { "phone": "+1234567890", "otp": "123456" }
  Response: { "access_token": "...", "refresh_token": "..." }
  ```
  - Verify OTP with Firebase
  - Check if user exists, create if new
  - Generate JWT (24h access, 30d refresh)
  - Store refresh token hash in DB (for rotation)
  - Return tokens

- [ ] Implement `/api/v1/auth/refresh`:
  ```rust
  POST /api/v1/auth/refresh
  Body: { "refresh_token": "..." }
  Response: { "access_token": "...", "refresh_token": "..." }
  ```
  - Verify refresh token hash exists in DB
  - Invalidate old refresh token (delete from DB)
  - Generate new access + refresh tokens
  - Store new refresh token hash
  - Return new tokens

- [ ] Implement JWT middleware:
  - Extract `Authorization: Bearer <token>`
  - Validate signature and expiry
  - Attach `user_id` to request state
  - Return 401 on invalid/expired

- [ ] Add `/api/v1/auth/me` endpoint:
  ```rust
  GET /api/v1/auth/me
  Response: { "id": "...", "phone": "...", "display_name": "..." }
  ```

**Backend (Day 4–5): WebSocket Manager**
- [ ] Implement `ws/manager.rs`:
  - `WebSocketManager` struct with `HashMap<uuid, Sender<WebSocketMessage>>`
  - `subscribe(channel: &str) -> Sender`
  - `unsubscribe(channel: &str)`
  - `broadcast(channel: &str, message: Message)`
- [ ] Configure Redis Pub/Sub:
  - Channel name: `ws:{channel_id}`
  - On connect: `SUBSCRIBE ws:{channel_id}`
  - On disconnect: `UNSUBSCRIBE`
  - Use `tokio::spawn` to forward Redis messages to WebSocket
- [ ] Implement `ws/handler.rs`:
  - Accept WebSocket upgrade
  - Validate JWT from query param `?token=...`
  - Extract `channel_id` from request
  - Subscribe to Redis channel
  - Loop: receive from client → handle → send to Redis; receive from Redis → send to client
  - Handle close gracefully

**Frontend (Day 5–7): Auth Flow**
- [ ] Implement phone input screen:
  - Country code selector
  - Phone number input
  - "Send OTP" button
- [ ] Implement OTP input screen:
  - 6-digit code input
  - Resend OTP button (disable for 60s)
  - "Verify" button
- [ ] Implement display name screen (first-time only)
- [ ] Store JWT in `react-native-keychain`
- [ ] Create auth provider (Riverpod):
  - `authStateProvider` → `AsyncValue<User?>`
  - `login(phone, otp)` → `Future<User>`
  - `logout()` → clear storage
- [ ] Wrap app with auth listener → redirect to login if no token

### Week 4: WebSocket Engine

**Backend (Day 1–3): Message Flow**
- [ ] Implement `/api/v1/channels/{channel_id}/messages`:
  ```rust
  POST /api/v1/channels/{channel_id}/messages
  Body: { "content": "Hello", "media_url": null, "reply_to_id": null }
  Response: { "id": "...", "created_at": "..." }
  ```
  - Validate channel membership
  - Save message to DB (async, don't block)
  - Publish to Redis: `PUBLISH ws:{channel_id} <JSON>`
  - Return 201 with message ID

- [ ] Implement history endpoint:
  ```rust
  GET /api/v1/channels/{channel_id}/messages?cursor=...&limit=50
  Response: { "messages": [...], "next_cursor": "..." }
  ```
  - Cursor-based pagination (based on `created_at` or ID)
  - Limit max 100

- [ ] Implement WebSocket message handling:
  - On receive: validate format, broadcast to Redis
  - On receive media message: validate `media_url` exists in R2

**Frontend (Day 3–7): WebSocket Client**
- [ ] Implement `websocket_service.dart`:
  - Connect to `wss://api.synapse.app/ws?token=...&channel_id=...`
  - Handle reconnection with 5s backoff
  - Emit events: `connected`, `disconnected`, `message`, `error`
  - Send heartbeat every 30s
- [ ] Create message provider (Riverpod):
  - `messagesProvider(channelId)` → `AsyncValue<List<Message>>`
  - `sendMessage(channelId, content)` → `Future<void>`
  - Listen to WebSocket → update state
- [ ] Update chat view to show real-time messages
- [ ] Add "Connected/Disconnected" indicator

---

## Phase 3: Workspaces & Channels (Weeks 5–7)

**Goal:** Slack-style structure, basic chat.

### Week 5: Backend Workspaces & Channels

**Day 1–2: Workspace APIs**
- [ ] Implement `/api/v1/workspaces`:
  ```rust
  POST /api/v1/workspaces
  Body: { "name": "My Team", "slug": "my-team" }
  Response: { "id": "...", "name": "...", "slug": "..." }
  ```
  - Create workspace, set creator as owner
  - Add creator to `workspace_members`

- [ ] Implement `/api/v1/workspaces/{slug}/join`:
  ```rust
  POST /api/v1/workspaces/{slug}/join
  Body: { "invite_code": "..." }
  Response: { "workspace": {...} }
  ```
  - Validate invite code (UUID)
  - Add user to `workspace_members`

- [ ] Implement `/api/v1/workspaces/{slug}/members`:
  ```rust
  GET /api/v1/workspaces/{slug}/members
  Response: { "members": [{ "user": {...}, "role": "..." }] }
  ```

**Day 3–5: Channel APIs**
- [ ] Implement `/api/v1/workspaces/{slug}/channels`:
  ```rust
  GET /api/v1/workspaces/{slug}/channels
  Response: { "channels": [{ "id": "...", "name": "...", "type": "..." }] }
  ```

- [ ] Implement `/api/v1/workspaces/{slug}/channels`:
  ```rust
  POST /api/v1/workspaces/{slug}/channels
  Body: { "name": "general", "type": "text" }
  Response: { "id": "...", "name": "...", "type": "..." }
  ```
  - Only admin/owner can create

**Day 5–7: Frontend Workspaces**
- [ ] Implement workspace list screen:
  - Fetch from `/api/v1/workspaces`
  - Show workspace name, member count
  - Pull-to-refresh
- [ ] Implement channel sidebar:
  - Fetch from `/api/v1/workspaces/{slug}/channels`
  - Show channel name, unread count
- [ ] Implement "Create Workspace" flow:
  - Name, slug input
  - Generate invite code
- [ ] Implement "Join Workspace" flow:
  - Invite code input
  - API call to join

### Week 6: Messaging UI

**Day 1–3: Chat View**
- [ ] Implement message list:
  - Bubble UI (sender avatar, name, time, content)
  - Differentiate sent vs received
  - Scroll to bottom on new message
- [ ] Implement message input bar:
  - Text input
  - Attachment button (camera/gallery)
  - Send button
- [ ] Integrate WebSocket for real-time updates
- [ ] Handle loading state for history

**Day 4–7: 1:1 Messaging**
- [ ] Backend: auto-create DM channel
  - When user A messages user B, create channel with type `dm`
  - Channel name: "User A & User B"
  - Add both users as members
- [ ] Frontend: "New Message" screen
  - Search users
  - Select recipient → create DM channel → navigate to chat

### Week 7: Polish & Testing

**Day 1–3: Backend Polish**
- [ ] Add input validation (phone format, channel name length)
- [ ] Add rate limiting (100 req/min per IP)
- [ ] Add request logging (tracing)
- [ ] Write unit tests for service layer
- [ ] Write integration tests for API endpoints

**Day 4–7: Frontend Polish**
- [ ] Add loading skeletons for chat history
- [ ] Add error handling (network loss, 500 errors)
- [ ] Add pull-to-refresh for message history
- [ ] Add "typing..." indicator (optional)
- [ ] Test on iOS and Android emulators

---

## Phase 4: Boards & Media (Weeks 8–9)

**Goal:** Admin announcements with ack tracking, file uploads.

### Week 8: Boards

**Backend (Day 1–3): Board APIs**
- [ ] Implement `/api/v1/channels/{channel_id}/posts`:
  ```rust
  POST /api/v1/channels/{channel_id}/posts
  Body: { "title": "New Policy", "body": "<p>Details...</p>" }
  Response: { "id": "...", "created_at": "..." }
  ```
  - Only admin/owner can create
  - Channel must be type `board`

- [ ] Implement `/api/v1/posts/{post_id}/ack`:
  ```rust
  POST /api/v1/posts/{post_id}/ack
  Response: { "acknowledged": true }
  ```
  - Upsert into `board_acknowledgements`

- [ ] Implement `/api/v1/posts/{post_id}/acks`:
  ```rust
  GET /api/v1/posts/{post_id}/acks
  Response: { "acknowledged": [{ "user": {...}, "acknowledged_at": "..." }], "pending": [{ "user": {...} }] }
  ```

**Frontend (Day 3–7): Board UI**
- [ ] Implement board channel view:
  - Card layout for each post (title, excerpt, date)
  - Tap to expand full post
  - Rich text rendering (HTML → React Native)
- [ ] Implement "Ack" button:
  - Visual state change (pending → acknowledged)
  - Disable after ack
- [ ] Implement admin ack dashboard:
  - List all members
  - Red/green status per post
  - Pull-to-refresh

### Week 9: Media Uploads

**Backend (Day 1–2): Proxy Upload**
- [ ] Implement `/api/v1/media/upload`:
  ```rust
  POST /api/v1/media/upload
  Body: multipart/form-data with file
  Response: { "object_key": "uploads/123/image.jpg", "url": "..." }
  ```
  - Validate content type (image/*, video/*)
  - Validate file size (max 10MB)
  - Generate unique object key (`uploads/{timestamp}/{uuid}_{filename}`)
  - Upload to R2 using service role credentials
  - Return object key and public URL

**Frontend (Day 2–5): Upload Flow**
- [ ] Implement image picker:
  - Use `image_picker` package
  - Select from gallery or camera
- [ ] Implement upload flow:
  - Send file to `/api/v1/media/upload` as multipart
  - Show progress bar (Dio supports progress callbacks)
  - Receive `object_key` and `url`
  - Pass `url` to `/api/v1/messages` when sending

**Backend (Day 5–7): Message with Media**
- [ ] Update `/api/v1/channels/{channel_id}/messages`:
  - Accept `media_url` parameter
  - Validate `media_url` matches R2 bucket
  - Save to DB

**Frontend (Day 5–7): Media Display**
- [ ] Render images in chat (cached)
- [ ] Lightbox for full-size image
- [ ] Video player for video messages (optional)

---

## Phase 5: Polish & Beta Launch (Weeks 10–12)

**Goal:** Notifications, optimization, deploy to app stores.

### Week 10: Push Notifications

**Backend (Day 1–3): FCM Integration**
- [ ] Add FCM dependency: `firebase-admin = "0.17"` (already added)
- [ ] Implement `/api/v1/users/{id}/fcm-token`:
  ```rust
  POST /api/v1/users/{id}/fcm-token
  Body: { "token": "..." }
  Response: { "success": true }
  ```
  - Store FCM token in users table
- [ ] Implement notification trigger:
  - On new message: send FCM to channel members
  - On board post: send FCM to all workspace members
  - Use Firebase Admin SDK `send()`

**Frontend (Day 3–7): Handle Notifications**
- [ ] Add `firebase_messaging` package
- [ ] Request permissions on first launch
- [ ] Register FCM token → send to backend
- [ ] Handle foreground notifications:
  - Show in-app banner
  - Navigate to channel on tap
- [ ] Handle background notifications:
  - Show system notification
  - Deep link to channel

### Week 11: Optimization & Security

**Performance (Day 1–3)**
- [ ] Client-side image resizing before upload (use `react-native-image-resizer`)
- [ ] Implement message list virtualization (only render visible items)
- [ ] Add Redis cache for workspace/channel lists (TTL: 5 min)
- [ ] Optimize DB queries (add indexes for `messages.channel_id`, `messages.created_at`)

**Security (Day 3–5)**
- [ ] Enable RLS on Supabase tables (ship-time requirement)
- [ ] Validate JWT on every WebSocket connection
- [ ] Sanitize HTML in board posts (prevent XSS)
- [ ] Add rate limiting on auth endpoints (5 attempts/min, 15 min lockout)
- [ ] Rotate JWT secret
- [ ] Verify refresh token rotation works (hash match, invalidate old)

**Testing (Day 5–7)**
- [ ] Write E2E tests for critical flows (login, send message, join workspace)
- [ ] Load test with `k6` or `locust` (100 concurrent WS connections)
- [ ] Security audit: SQLi, XSS, CSRF

### Week 12: Launch Prep

**App Store (Day 1–3)**
- [ ] Generate app icons (iOS + Android)
- [ ] Create splash screens
- [ ] Write app store descriptions
- [ ] Prepare privacy policy and terms of service
- [ ] Submit to TestFlight (iOS) and Internal Track (Android)

**Infrastructure (Day 3–5)**
- [ ] Configure Nginx reverse proxy on GCP e2-micro:
  ```nginx
  server {
      listen 443 ssl;
      server_name api.synapse.app;
      
      ssl_certificate /etc/letsencrypt/live/api.synapse.app/fullchain.pem;
      ssl_certificate_key /etc/letsencrypt/live/api.synapse.app/privkey.pem;
      
      location / {
          proxy_pass http://127.0.0.1:8080;
          proxy_http_version 1.1;
          proxy_set_header Upgrade $http_upgrade;
          proxy_set_header Connection "upgrade";
          proxy_set_header Host $host;
          proxy_set_header X-Real-IP $remote_addr;
      }
  }
  ```
- [ ] Set up Certbot for Let's Encrypt
- [ ] Configure firewall (UFW): allow 22, 80, 443
- [ ] Set up log rotation for Axum
- [ ] Configure monitoring: `htop` for CPU/RAM, alert if >80% for 5 min

**Launch (Day 5–7)**
- [ ] Deploy backend: `docker build -t synapse-backend . && docker run -d synapse-backend`
- [ ] Deploy React Native Web to Cloudflare Pages
- [ ] Test end-to-end on real devices
- [ ] Onboard first 5 SMB teams
- [ ] Monitor logs (tracing → Loki → Grafana)

---

## Post-Launch (Weeks 13+)

### V1 Features (Weeks 13–16)
- [ ] Voice rooms (WebRTC)
- [ ] Threads for messages
- [ ] Granular roles (moderator)
- [ ] Full-text search (Postgres `tsvector`)

### V2 Features (Weeks 17–20)
- [ ] Workspace analytics dashboard
- [ ] Board engagement metrics
- [ ] SSO (SAML/OIDC)
- [ ] Custom domains

---

## Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Firebase OTP rate limits | Medium | High | Implement queuing, use MSG91 as fallback |
| WebSocket connection drops | High | Medium | Reconnection with exponential backoff |
| Media upload failure | Medium | Medium | Retry on network error, validate file size/type server-side |
| Supabase RLS misconfiguration | Low | High | Test policies thoroughly before launch |
| GCP e2-micro resource exhaustion | Medium | High | Monitor CPU/RAM, upgrade to e2-small if needed |

---

## Success Metrics

- **Week 4:** Backend responds to `/health`, frontend builds for Web/iOS/Android
- **Week 7:** Users can send/receive messages in real-time, offline queue works
- **Week 9:** Admins can create boards, users can acknowledge, media uploads work
- **Week 12:** 5 teams onboarded, 50+ daily active users (sent/received message)
- **Week 16:** Push notifications working, voice rooms in beta
