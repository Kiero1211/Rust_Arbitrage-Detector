# Arbitrage Detector

A modern, scalable web service for detecting arbitrage opportunities built with Rust and Axum.

## Project Structure

This project follows Rust best practices for web service organization:

```
src/
├── main.rs          # Application entry point
├── lib.rs           # Library exports and shared types
├── error.rs         # Centralized error handling
├── config/          # Configuration management
│   └── mod.rs
├── handlers/        # HTTP request handlers
│   └── mod.rs
├── routes/          # Route definitions and organization
│   └── mod.rs
└── models/          # Data models and types
    └── mod.rs
```

## Architecture Principles

### Modular Design

- **Separation of Concerns**: Each module has a single responsibility
- **Reusable Components**: Core logic in `lib.rs` for easy testing and reuse
- **Centralized Configuration**: Environment-based config with defaults

### Error Handling

- **Consistent Error Types**: Custom `AppError` with proper HTTP responses
- **Graceful Degradation**: Proper error propagation with helpful messages
- **Logging Integration**: Structured logging with tracing

### Application State

- **Shared State Pattern**: `Arc<AppState>` for thread-safe shared data
- **Dependency Injection**: Easy to extend with databases, HTTP clients, etc.
- **Configuration Access**: Global access to configuration through state

## Running the Application

### Prerequisites

- Rust 1.70+ (uses edition 2024)
- Environment variables (see `.env.example`)

### Setup

1. Copy environment configuration:

   ```bash
   cp .env.example .env
   ```

2. Build and run:

   ```bash
   cargo run
   ```

3. Test endpoints:
   ```bash
   curl http://127.0.0.1:3000/health
   curl http://127.0.0.1:3000/hello
   curl http://127.0.0.1:3000/info
   ```

## Available Endpoints

- `GET /health` - Health check with timestamp
- `GET /hello` - Simple hello world
- `GET /info` - Application information
- `GET /api/v1/*` - Versioned API routes (for future expansion)

## Configuration

Environment variables:

- `SERVER_HOST` - Server bind address (default: 127.0.0.1)
- `SERVER_PORT` - Server port (default: 3000)
- `LOG_LEVEL` - Logging level (default: info)

## Development

### Adding New Features

1. **Models**: Add data structures to `src/models/`
2. **Handlers**: Add business logic to `src/handlers/`
3. **Routes**: Register new routes in `src/routes/`
4. **State**: Extend `AppState` for new dependencies

### Testing

```bash
cargo test
cargo clippy
cargo fmt
```

## Future Extensions

The modular structure makes it easy to add:

- Database connections (PostgreSQL, Redis)
- External API integrations
- WebSocket support for real-time data
- Authentication and authorization
- Rate limiting and caching
- Background job processing

## Dependencies

Key dependencies and their purposes:

- `axum` - Modern web framework
- `tokio` - Async runtime
- `serde` - Serialization/deserialization
- `tracing` - Structured logging
- `chrono` - Date/time handling
- `dotenvy` - Environment variable loading
