# API Configuration Module

This module provides the ability to load configuration from an external API service instead of static YAML files or database.

## Overview

The API Configuration module allows the Traceloop Hub Gateway to:
- Fetch configuration from external REST APIs
- Support various authentication methods (Bearer tokens, API keys, custom headers)
- Poll for configuration updates at configurable intervals
- Resolve secrets from multiple sources (environment variables, literals, Kubernetes)
- Transform external API responses into the internal configuration format

## Environment Variables

### Required
- `API_CONFIG_BASE_URL`: Base URL of the configuration API (e.g., "https://config-api.example.com/v1")

### Optional
- `API_CONFIG_TIMEOUT_SECONDS`: HTTP request timeout in seconds (default: 30)
- `API_CONFIG_AUTH_HEADER`: Authentication header name (e.g., "Authorization", "X-API-Key")
- `API_CONFIG_AUTH_VALUE`: Authentication header value (e.g., "Bearer token", "api-key-value")
- `API_POLL_INTERVAL_SECONDS`: Configuration polling interval in seconds (default: 30)

### Endpoint Customization
- `API_CONFIG_FULL_ENDPOINT`: Single endpoint for complete configuration (default: "config")
- `API_CONFIG_PROVIDERS_ENDPOINT`: Providers endpoint (default: "providers")
- `API_CONFIG_MODELS_ENDPOINT`: Models endpoint (default: "models")
- `API_CONFIG_PIPELINES_ENDPOINT`: Pipelines endpoint (default: "pipelines")

## Usage

1. Enable the api_config feature in Cargo.toml
2. Set the required environment variables
3. Start the application - it will automatically detect and use API configuration

```bash
export API_CONFIG_BASE_URL="https://config-api.example.com/v1"
export API_CONFIG_AUTH_HEADER="Authorization"
export API_CONFIG_AUTH_VALUE="Bearer your-api-token"
cargo run --features api_config_feature
```

## Configuration Priority

1. **API Configuration** (highest priority) - If `API_CONFIG_BASE_URL` is set
2. **Database Configuration** (EE feature) - If `DATABASE_URL` is set
3. **YAML Configuration** (fallback) - Uses config.yaml file

## API Response Format

The external API should return JSON responses matching the expected schema. See `example-api-config.json` for a complete example.

## Secret Management

The module supports three types of secrets:

1. **Literal secrets**: Plain text values (with optional encryption support)
2. **Environment variables**: References to environment variables
3. **Kubernetes secrets**: References to Kubernetes secret objects (planned)

## Architecture

- `lib.rs`: Main integration function
- `dto.rs`: Data transfer objects for API responses
- `secret_resolver.rs`: Secret resolution from various sources
- `config_provider_service.rs`: HTTP client and configuration transformation logic

## Testing

Run tests with:
```bash
cd api_config
cargo test
```

The tests use wiremock to simulate API responses and validate the configuration transformation logic.
