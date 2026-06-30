# Synapse Architecture Diagrams

## 1. High-Level System Architecture
```mermaid
graph TD
    subgraph "Clients (React Native)"
        Web[React Native Web]
        iOS[React Native iOS]
        Android[React Native Android]
    end

    subgraph "Network / Edge"
        CF[Cloudflare CDN]
    end

    subgraph "Backend (GCP e2-micro)"
        Nginx[Nginx Reverse Proxy]
        Axum[Axum Server (Rust)]
    end

    subgraph "Data & Services"
        Redis[(Upstash Redis)]
        DB[(Supabase Postgres)]
        R2[Cloudflare R2 Storage]
        Firebase[Firebase Auth/FCM]
    end

    Web --> CF
    iOS --> CF
    Android --> CF
    CF --> Nginx
    Nginx --> Axum

    Axum <--> Redis
    Axum <--> DB
    Axum <--> R2
    Axum <--> Firebase
2. WebSocket Message Flow (Redis Pub/Sub)sequenceDiagram
    participant C1 as Client A
    participant S as Axum Server
    participant R as Redis Pub/Sub
    participant C2 as Client B
    participant DB as PostgreSQL

    C1->>S: Connect WS + JWT
    S->>S: Validate JWT
    S->>R: Subscribe to "channel_123"
    
    Note over S,R: Connection Established

    C1->>S: Send Message (Text)
    S->>DB: Save Message (Async)
    S->>R: Publish Message Payload
    
    R->>S: Broadcast Payload
    S->>C2: Forward Message via WS
3. Media Upload Flow (Proxy via Backend)sequenceDiagram
    participant Client as React Native App
    participant Server as Axum Server
    participant R2 as Cloudflare R2

    Client->>Server: POST /api/media/upload (multipart)
    Server->>Server: Validate file (type, size)
    Server->>R2: Upload with service credentials
    R2-->>Server: 200 OK (Object Key)
    Server-->>Client: Return { object_key, url }

    Client->>Server: POST /api/messages (with url)
    Server->>DB: Save Message + Object Key
4. Database Schema (ERD)erDiagram
    USERS ||--o{ WORKSPACES : creates
    USERS ||--o{ WORKSPACE_MEMBERS : joins
    USERS ||--o{ MESSAGES : sends
    USERS ||--o{ BOARD_POSTS : authors
    
    WORKSPACES ||--o{ WORKSPACE_MEMBERS : contains
    WORKSPACES ||--o{ CHANNELS : contains

    CHANNELS ||--o{ MESSAGES : contains
    CHANNELS ||--o{ BOARD_POSTS : contains

    MESSAGES ||--o{ MESSAGES : replies_to

    USERS ||--o{ BOARD_ACK : acknowledges

    USERS {
        uuid id PK
        text phone UK
        text display_name
        text avatar_url
    }

    WORKSPACES {
        uuid id PK
        text name
        text slug
    }

    CHANNELS {
        uuid id PK
        uuid workspace_id FK
        text name
        text type
    }

    MESSAGES {
        uuid id PK
        uuid channel_id FK
        uuid sender_id FK
        text content
        text media_url
    }

    BOARD_POSTS {
        uuid id PK
        uuid channel_id FK
        uuid author_id FK
        text title
        text body
    }

    BOARD_ACK {
        uuid post_id FK
        uuid user_id FK
        timestamp acked_at
    }
