# Synapse Development Roadmap

**See also:** [02-development-plan.md](./02-development-plan.md) — granular task breakdown with decisions log.

## Phase 1: Foundation & Infrastructure (Weeks 1–2)
**Goal:** Establish the environment, CI/CD, and database schema.

- [ ] **DevOps Setup:**
    - Provision GCP e2-micro instance (Ubuntu 22.04, 2 core, 1GB RAM).
    - Configure Nginx + SSL (Certbot/Cloudflare).
    - Setup Cloudflare R2 bucket with CORS policies.
- [ ] **Database & Cache:**
    - Initialize Supabase project (PostgreSQL).
    - Create Upstash Redis instance.
    - Run initial SQLx migrations (`users`, `workspaces`, `channels`).
- [ ] **Backend Initialization:**
    - Setup Axum project structure (`main.rs`, `routes/`, `ws/`, `db/`).
    - Configure `Cargo.toml` dependencies (Axum, SQLx, Redis, Tokio).
- [ ] **Frontend Initialization:**
    - Setup React Native project (Web, iOS, Android targets).
    - Configure `pubspec.yaml` (`dio`, `web_socket_channel`, `riverpod`, `go_router`).
- [ ] **CI/CD:**
    - GitHub Actions: Rust lint/test on PR.
    - GitHub Actions: React Native Web build -> Cloudflare Pages deploy.

---

## Phase 2: Auth & The "Pipes" (Weeks 3–4)
**Goal:** Secure login and the real-time message bus.

- [ ] **Backend (Auth):**
    - Implement Phone OTP verification (via Firebase/MSG91).
    - Implement JWT generation (30-day expiry, no refresh tokens).
    - Create JWT Middleware for protected routes.
- [ ] **Backend (Real-time Engine):**
    - **Critical:** Implement Redis Pub/Sub logic.
    - Create `WebSocketManager` struct:
        - Client connects -> Validates JWT -> Subscribes to Redis Channel.
        - `tokio::spawn` task to listen to Redis and forward to WebSocket.
- [ ] **Frontend (Auth):**
    - Onboarding Flow: Phone Input -> OTP -> Display Name.
    - Store JWT in `react-native-keychain`.
- [ ] **Frontend (Real-time):**
    - WebSocket Client: Connect, Auth, and Handle Reconnection (5s retry logic).
    - Basic "Connected/Disconnected" status indicator.

---

## Phase 3: Workspaces & Channels (Weeks 5–7)
**Goal:** The "Slack" structure and basic chat.

- [ ] **Backend (Workspaces):**
    - API: Create Workspace, Join via Link, Invite User.
    - API: Create Channel (Text type).
    - DB: Enforce Workspace Membership checks.
- [ ] **Backend (Messaging):**
    - API: Send Message -> Save to Postgres (Async) -> Publish to Redis.
    - API: Get History (Pagination support).
- [ ] **Frontend (UI/UX):**
    - Home Screen: List of Workspaces.
    - Sidebar: List of Channels within a Workspace.
    - Chat View:
        - Message List (Bubble UI).
        - Input Bar (Text + Attachment button).
        - Real-time message insertion.

---

## Phase 4: Boards & Media (Weeks 8–9)
**Goal:** The "Classroom" differentiator and file sharing.

- [ ] **Backend (Boards):**
    - API: Create Board Post (Admin only).
    - API: Acknowledge Post (Upsert `board_acknowledgements`).
    - API: Get Ack Status (Who read vs. Who didn't).
- [ ] **Backend (Media):**
    - API: `/api/media/presign` -> Returns R2 URL.
    - Logic: Client uploads directly to R2, then sends URL to `/api/messages`.
- [ ] **Frontend (Boards):**
    - Board Channel View: Card layout for announcements.
    - Ack Button: Visual state change (Pending -> Acknowledged).
    - Admin View: List of users with status.
- [ ] **Frontend (Media):**
    - Image Picker integration.
    - R2 Upload flow (Show progress bar).

---

## Phase 5: Polish & Beta Launch (Weeks 10–12)
**Goal:** Stability, Notifications, and First Users.

- [ ] **Notifications:**
    - Backend: Trigger FCM token push on new message/board.
    - Frontend: Handle notification payload -> Deep link to channel.
- [ ] **Optimization:**
    - Resize images client-side before upload (save bandwidth).
    - Implement "Loading" skeletons for chat history.
    - Error handling (Network loss, Server 500).
- [ ] **Launch Prep:**
    - Generate App Icons & Splash Screens.
    - Draft Privacy Policy & Terms of Service.
    - Deploy iOS (TestFlight) & Android (Internal Track).
    - **Milestone:** Onboard first 5 SMB teams.
