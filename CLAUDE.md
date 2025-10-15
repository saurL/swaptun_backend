# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build, Test, and Run Commands

```bash
# Build the project
cargo build

# Build for production (optimized)
cargo build --release

# Run the application
cargo run

# Run with auto-reload during development (requires cargo-watch)
cargo watch -x run

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Database migrations (auto-run on startup)
# Migrations are applied automatically when the server starts via Migrator::up()
```

## High-Level Architecture

### Workspace Structure

This is a Cargo workspace with 6 crates forming a clean layered architecture:

```
swaptun-backend/
├── api/              # HTTP layer - Actix-web routes and handlers
├── services/         # Business logic - Service layer for all domains
├── repositories/     # Data access - Database operations via SeaORM
├── models/           # Database entities - SeaORM entity definitions
├── migrations/       # Database migrations - SeaORM migration files
└── src/              # Main entry point - Delegates to api crate
```

### Dependency Flow

```
main.rs → api → services → repositories → models
                     ↓
                migrations (for schema)
```

### Layered Architecture Pattern

**API Layer** (`api/src/api/`)

- HTTP route handlers and request/response models
- Routes configured in `api/src/api/mod.rs::configure_routes()`
- Endpoints organized by domain: `auth.rs`, `users.rs`, `spotify.rs`, `apple.rs`, `youtube.rs`, `playlist.rs`, `notification.rs`
- All protected routes wrapped with `JwtMiddleware` and role-based `RoleGuard`

**Services Layer** (`services/src/`)

- Business logic for each domain
- Platform-specific music services: `spotify/`, `apple/`, `yt_music/`, `deezer/`
- Core services: `user/`, `playlist/`, `music/`, `notification/`, `mail/`
- Authentication: `auth/` (JWT, password hashing, role guards)
- Each service takes `Arc<DatabaseConnection>` and instantiates repositories

**Repositories Layer** (`repositories/src/`)

- Data access abstraction using SeaORM
- Pattern: Each repository wraps `Arc<DatabaseConnection>`
- Methods return `Result<T, DbErr>`
- Examples: `UserRepository`, `PlaylistRepository`, `SpotifyTokenRepository`, `AppleTokenRepository`

**Models Layer** (`models/src/`)

- SeaORM entity definitions representing database tables
- Exports `ActiveModel`, `Column`, `Entity`, and `Model` for each table
- Key models: `user`, `playlist`, `music`, `spotify_token`, `apple_token`, `youtube_token`, `deezer_token`, `friendship`

**Migrations Layer** (`migrations/src/`)

- SeaORM migrations in `migrations/src/`
- All migrations registered in `migrations/src/lib.rs::Migrator`
- Migration naming: descriptive with dates (e.g., `m20250910_000000_create_apple_token_table.rs`)

## Key Patterns and Conventions

### Authentication & Authorization

**JWT Tokens**

- Tokens generated with `services/src/auth/jwt.rs::generate_token()`
- No expiration by default (`exp: None`), unless using `generate_token_expiration()` with a duration
- Claims structure includes: `user_id`, `username`, `role`
- Secret from env var `JWT_SECRET`

**Middleware Chain**

```rust
web::scope("")
    .wrap(JwtMiddleware)              // Validates JWT token
    .wrap(RoleGuard::user())           // Enforces minimum role
```

**Role Hierarchy**

- Roles: `Admin`, `User`, `Guest` (defined in `services/src/auth/roles.rs`)
- Admin has all User permissions
- Role validation: `UserRole::is_valid_role()`

**Password Security**

- Argon2 hashing via `services/src/auth/password.rs`
- Functions: `hash_password()`, `verify_password()`

### Multi-Platform Music Integration

**Supported Platforms**

- Spotify (`services/src/spotify/`)
- Apple Music (`services/src/apple/`)
- YouTube Music (`services/src/yt_music/`)
- Deezer (`services/src/deezer/`)

**Platform Service Pattern**
Each platform service:

1. Stores OAuth tokens in platform-specific token tables (`spotify_token`, `apple_token`, `youtube_token`, `deezer_token`)
2. Uses `PlaylistService` and `MusicService` for cross-platform data
3. Implements token management: `add_token()`, `delete_token()`, `update_token()`
4. Fetches playlists from platform and saves to local database
5. Sends playlists back to platform

**OAuth Token Storage**

- Each platform has a token repository (e.g., `SpotifyTokenRepository`)
- Tokens linked to users via `user_id` foreign key
- Spotify uses both `spotify_code` and `spotify_token` tables (OAuth code flow)
- YouTube uses PKCE flow with verifier storage in-memory

**Playlist Origin Tracking**

- `PlaylistOrigin` enum in `models/src/playlist.rs`
- Values: `Spotify`, `AppleMusic`, `YoutubeMusic`, `Deezer`, `Local`
- Used to track playlist source and enable cross-platform sync

### Service Instantiation Pattern

All services follow this pattern:

```rust
pub struct SomeService {
    repository: SomeRepository,
    db: Arc<DatabaseConnection>,
}

impl SomeService {
    pub fn new(db: Arc<DatabaseConnection>) -> Self {
        Self {
            repository: SomeRepository::new(db.clone()),
            db,
        }
    }
}
```

Services can be composed:

```rust
pub struct PlaylistService {
    playlist_repository: PlaylistRepository,
    music_playlist_repository: MusicPlaylistRepository,
}
```

### Error Handling

**AppError enum** (`services/src/error/app_error.rs`)

- Variants: `Database(DbErr)`, `Validation(String)`, `NotFound(String)`, `Unauthorized(String)`, `InternalServerError`
- Implements `ResponseError` for Actix-web (auto HTTP response conversion)
- Database errors logged but returned as generic "internal error" to client

**Error Propagation**

- Services return `Result<T, AppError>`
- API handlers use `?` operator, `AppError::error_response()` handles HTTP conversion

### Validation

**Input Validation**

- Uses `validator` crate with derive macros
- Centralized validators in `services/src/validators/`
- Helper function: `process_json_validation()` in `user_validators.rs`
- Example: Phone regex, password complexity, username format

### Database Migrations Workflow

1. Create migration file in `migrations/src/` with descriptive name
2. Implement `Migration` struct with `up()` and `down()` methods
3. Register in `migrations/src/lib.rs::Migrator::migrations()`
4. Migrations run automatically on server startup via `Migrator::up(&db, None)`
5. Use SeaORM migration helpers: `Table::create()`, `Table::alter()`, etc.

### Configuration Management

**Environment Variables** (`.env` file)

```
DATABASE_URL=postgres://...          # PostgreSQL connection string
SERVER_HOST=0.0.0.0                  # Server bind address
SERVER_PORT=8000                     # Server port
JWT_SECRET=...                       # JWT signing secret
RUST_LOG=info                        # Logging level

# Platform-specific credentials
RSPOTIFY_CLIENT_ID=...
RSPOTIFY_CLIENT_SECRET=...
RSPOTIFY_REDIRECT_URI=...

YOUTUI_OAUTH_CLIENT_ID=...
YOUTUI_OAUTH_CLIENT_SECRET=...

APPLE_KEY_ID=...
APPLE_TEAM_ID=...

# SMTP for email service
SMTP_HOST=...
SMTP_PORT=...
SMTP_USERNAME=...
SMTP_PASSWORD=...
```

**AppConfig** (`api/src/config/app_config.rs`)

- Loaded from environment via `AppConfig::from_env()`
- Structured config: `ServerConfig`, `DatabaseConfig`

### Feature Flags

The workspace uses Cargo features for conditional compilation:

**Root workspace** (`Cargo.toml`)

- `default = ["full"]`
- `full = ["swaptun-api/full"]`

**API crate** (`api/Cargo.toml`)

- `full` feature activates all dependencies (actix-web, sea-orm, etc.)
- Most code gated with `#[cfg(feature = "full")]`

**Services crate** (`services/Cargo.toml`)

- `default = ["full"]`
- `full` feature includes tokio, argon2, jsonwebtoken, platform SDKs, etc.
- Allows library usage without heavy dependencies

### Notification Service

**Firebase Cloud Messaging**

- Service: `services/src/notification/notification.rs`
- Stores FCM tokens in `fcm_token` table
- Uses `google-fcm1` crate for push notifications
- Tested on startup in production mode

### Mail Service

**SMTP Integration**

- Service: `services/src/mail/mail_service.rs`
- Uses `lettre` crate with tokio runtime
- Configuration via SMTP\_\* environment variables
- Connection tested on startup in production mode
- DTO models: `MailRequest`, `MailResponse`

### Friendship & Sharing

**Social Features**

- Friendships tracked in `friendship` table (created in `m20250816_000000_create_friendships_table`)
- Shared playlists in `shared_playlist` table
- Endpoints: Share playlists with friends, send playlists to platforms

### Testing

**Test Infrastructure**

- Test utilities in `services/src/test/` (feature-gated)
- `test_database.rs` provides test DB setup with testcontainers
- Uses PostgreSQL testcontainer for integration tests
- Migrations run automatically in test environment

## Important Notes

### Startup Sequence

1. Load environment variables from `.env`
2. Initialize logger with `env_logger`
3. Load `AppConfig` from environment
4. Connect to database via SeaORM
5. Run migrations with `Migrator::up()`
6. Test mail service connection (production only)
7. Test notification service connection (production only)
8. Start Actix-web HTTP server
9. Configure routes with JWT middleware

### Database Connection

- PostgreSQL via SeaORM with SQLx driver
- Connection string from `DATABASE_URL` env var
- Shared via `Arc<DatabaseConnection>` (wrapped in `web::Data` for Actix)
- All queries use SeaORM's query builder (no raw SQL)

### Logging

- Uses `log` crate with `env_logger`
- Log level from `RUST_LOG` env var (default: info)
- Actix-web request logging via `Logger` middleware
- Important operations logged: token generation, DB errors, service startup

### Route Organization

Protected routes nested under JWT middleware:

```rust
web::scope("/api")
    .service(web::scope("/auth")...)        // Public auth endpoints
    .service(web::scope("/register")...)    // Public registration
    .service(
        web::scope("")
            .wrap(JwtMiddleware)            // All routes below require auth
            .service(web::scope("/users").wrap(RoleGuard::user())...)
            .service(web::scope("/spotify")...)
            .service(web::scope("/apple")...)
            .service(web::scope("/youtube")...)
            .service(web::scope("/playlists")...)
            .service(web::scope("/notifications")...)
    )
```

### Fuzzy Search

- PostgreSQL `pg_trgm` extension enabled in `m20250814_000000_enable_fuzzy_search`
- Used for fuzzy text matching on user searches

### Music Metadata

- MusicBrainz integration in `services/src/musicbrainz/`
- Function: `get_track_metadata()` for enriching track data
- Test endpoint at `/test/musicbrainz`

## Consigne

- crée quand c'est possible des test unitaires
- si des changement sont fait au niveau de l'API je veux que tu rédige des consignes sur comment les implémenter au niveau du frontend ( endpoint api , cas d'usages etc ) , ces consigne s'adresse a un autre agent Claude
- Quand tu rédige ces consignes il faut être le plus synthétique possible
