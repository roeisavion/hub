# API Configuration Usage Guide

## Quick Start

### 1. Set Environment Variables

```bash
# Required: Base URL of your configuration API
export API_CONFIG_BASE_URL="https://config-api.example.com/v1"

# Optional: Authentication
export API_CONFIG_AUTH_HEADER="Authorization"
export API_CONFIG_AUTH_VALUE="Bearer your-api-token"

# Optional: Polling interval (default: 30 seconds)
export API_POLL_INTERVAL_SECONDS="60"
```

### 2. Start the Application

```bash
cargo run --features api_config_feature
```

## Configuration API Endpoints

The module supports two approaches:

### Option 1: Single Endpoint (Recommended)

Provide all configuration in one endpoint:

```bash
export API_CONFIG_FULL_ENDPOINT="config"
# This will call: https://config-api.example.com/v1/config
```

### Option 2: Separate Endpoints

Use different endpoints for each configuration type:

```bash
export API_CONFIG_PROVIDERS_ENDPOINT="providers"
export API_CONFIG_MODELS_ENDPOINT="models"
export API_CONFIG_PIPELINES_ENDPOINT="pipelines"
# These will call:
# - https://config-api.example.com/v1/providers
# - https://config-api.example.com/v1/models
# - https://config-api.example.com/v1/pipelines
```

## Authentication Examples

### Bearer Token
```bash
export API_CONFIG_AUTH_HEADER="Authorization"
export API_CONFIG_AUTH_VALUE="Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### API Key
```bash
export API_CONFIG_AUTH_HEADER="X-API-Key"
export API_CONFIG_AUTH_VALUE="your-api-key-here"
```

### Custom Header
```bash
export API_CONFIG_AUTH_HEADER="X-Custom-Auth"
export API_CONFIG_AUTH_VALUE="custom-auth-value"
```

## Secret Management

### Environment Variable Secrets
```json
{
  "api_key": {
    "type": "environment",
    "variable_name": "OPENAI_API_KEY"
  }
}
```

### Literal Secrets
```json
{
  "api_key": {
    "type": "literal",
    "value": "sk-1234567890abcdef"
  }
}
```

### Kubernetes Secrets (Planned)
```json
{
  "api_key": {
    "type": "kubernetes",
    "secret_name": "openai-secret",
    "key": "api-key",
    "namespace": "default"
  }
}
```

## Complete Example

### 1. Environment Setup
```bash
#!/bin/bash
# config-env.sh

export API_CONFIG_BASE_URL="https://api.mycompany.com/v1/hub-config"
export API_CONFIG_AUTH_HEADER="Authorization"
export API_CONFIG_AUTH_VALUE="Bearer $(cat /path/to/token)"
export API_POLL_INTERVAL_SECONDS="30"

# Provider secrets
export OPENAI_API_KEY="sk-your-openai-key"
export ANTHROPIC_API_KEY="sk-ant-your-anthropic-key"

# Application settings
export PORT="3000"
export RUST_LOG="info"
```

### 2. Run the Application
```bash
source config-env.sh
cargo run --features api_config_feature
```

### 3. Expected Output
```
2024-01-01T12:00:00.000Z  INFO hub: Starting Traceloop Hub Gateway...
2024-01-01T12:00:00.001Z  INFO hub: API config feature enabled and API_CONFIG_BASE_URL is set. Loading configuration from external API.
2024-01-01T12:00:00.002Z  INFO hub: API config integration initialized.
2024-01-01T12:00:00.100Z  INFO hub: Successfully fetched initial configuration from external API.
2024-01-01T12:00:00.101Z  INFO hub: Initial API configuration validated successfully.
2024-01-01T12:00:00.102Z  INFO hub: Successfully transformed API configuration: 2 providers, 3 models, 1 pipelines
2024-01-01T12:00:00.103Z  INFO hub: Starting API configuration poller with interval: 30s.
2024-01-01T12:00:00.104Z  INFO hub: Server is running on port 3000
```

## Troubleshooting

### Common Issues

1. **Connection Failed**
   ```
   Failed to fetch configuration from https://...: Connection refused
   ```
   - Check that the API_CONFIG_BASE_URL is correct and accessible
   - Verify network connectivity

2. **Authentication Failed**
   ```
   API returned error status 401: Unauthorized
   ```
   - Verify API_CONFIG_AUTH_HEADER and API_CONFIG_AUTH_VALUE
   - Check if the token has expired

3. **Invalid Configuration**
   ```
   Initial API configuration is invalid: [...]. Halting.
   ```
   - Check the API response format against the expected schema
   - Verify all required fields are present

4. **Secret Resolution Failed**
   ```
   Failed to resolve environment variable 'OPENAI_API_KEY': environment variable not found
   ```
   - Ensure all referenced environment variables are set
   - Check secret configuration in the API response

### Debug Mode

```bash
export RUST_LOG="debug"
cargo run --features api_config_feature
```

This will show detailed logs of API requests, responses, and configuration transformations.
