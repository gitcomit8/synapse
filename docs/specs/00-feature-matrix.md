# Synapse Product Roadmap & Feature Matrix

## 1. MVP (Seed Funding Stage)
**Objective:** Validate the "WhatsApp for Work" concept with SMBs and Indie Teams.
**Target:** 50–100 Active Teams / 1,000 Users.
**Active user definition:** Daily message sent or received.
**Constraint:** Free for teams < 50 people.

### Core Communication
- [ ] **Phone OTP Authentication:** Login via phone number (No email required).
- [ ] **Workspaces:** Org-level containers with invite-only or link-based access.
- [ ] **Text Channels:** Topic-based rooms (Slack-style) within workspaces.
- [ ] **Real-time Chat:** WebSocket messaging with Redis Pub/Sub.
- [ ] **1:1 Messaging:** Direct messaging between users.

### Productivity (The "Boards" Differentiator)
- [ ] **Boards (Announcements):** Admin-only posts with rich text formatting.
- [ ] **Acknowledgement Tracking:** Users must click "Ack" to mark as read.
- [ ] **Admin Dashboard:** View who has/hasn't acknowledged posts (Red/Green status).

### Media & Infrastructure
- [ ] **File/Image Sharing:** Upload via backend proxy to Cloudflare R2 (e2-micro bandwidth constrained).
- [ ] **Cross-Platform:** Single React Native codebase (Web, iOS, Android).
- [ ] **Push Notifications:** FCM integration for new messages and board alerts.
- [ ] **Offline Support:** Basic local caching of recent messages.

---

## 2. Post-MVP / V1 (Seed Growth)
**Objective:** Increase retention, engagement, and stickiness.
**Target:** 1,000+ Active Users.

### Advanced Communication
- [ ] **Voice Rooms:** Discord-style voice channels (WebRTC).
- [ ] **Threads:** Reply chains within text channels to reduce noise.
- [ ] **Granular Roles:** Custom roles (Admin, Moderator, Member) per channel.
- [ ] **Message Search:** Full-text search using PostgreSQL `tsvector`.

### Analytics
- [ ] **Workspace Analytics:** Admin view of message volume and activity heatmaps.
- [ ] **Board Engagement:** Analytics on which posts get the most interaction.

---

## 3. Series A (Scale & Enterprise)
**Objective:** Monetization and Enterprise readiness.
**Target:** 10,000+ Users, Paid Tiers.

### Enterprise Features
- [ ] **SSO (SAML/OIDC):** Login via Google Workspace or Microsoft Azure.
- [ ] **Custom Domains:** `company.synapse.app` white-labeling.
- [ ] **Audit Logs:** Track admin actions and message deletion for compliance.
- [ ] **Data Residency:** Option to store data in specific regions (India/EU).

### Intelligence
- [ ] **AI Summarization:** Auto-summarize long threads (LLM integration).
- [ ] **Action Items:** Extract tasks from chat messages automatically.

### Security
- [ ] **End-to-End Encryption (E2EE):** For 1:1 sensitive chats.
- [ ] **GDPR/DPDP Compliance Tools:** "Delete my data" automation.
