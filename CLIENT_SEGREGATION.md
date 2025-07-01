# Client Segregation & Authorization (EE Feature)

This document describes the client-based segregation and authorization feature available in the Enterprise Edition (EE) of Traceloop Hub.

## Overview

The client segregation feature allows you to:
- **Multi-tenant Resource Management**: Each client can only access and modify their own providers, models, and pipelines
- **API Key Authentication**: Secure authentication using the `x-traceloop-pipeline` header
- **Complete Resource Isolation**: Clients cannot see or interact with other clients' resources
- **Backward Compatibility**: Existing EE deployments continue to work unchanged

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Client A      │    │   Client B      │    │   Client C      │
│   (ACME Corp)   │    │  (Globex Inc)   │    │ (Wayne Ent)     │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ Providers       │    │ Providers       │    │ Providers       │
│ • OpenAI        │    │ • Anthropic     │    │ • Azure OpenAI  │
│ • Bedrock       │    │ • VertexAI      │    │ • OpenAI        │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ Models          │    │ Models          │    │ Models          │
│ • acme-gpt-4    │    │ • globex-claude │    │ • wayne-gpt-4   │
│ • acme-gpt-3.5  │    │ • globex-gemini │    │ • wayne-dalle   │
├─────────────────┤    ├─────────────────┤    ├─────────────────┤
│ Pipelines       │    │ Pipelines       │    │ Pipelines       │
│ • acme-chat     │    │ • globex-chat   │    │ • wayne-chat    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## Database Schema

The client segregation adds a `clients` table and foreign key relationships:

```sql
-- Clients table
CREATE TABLE hub_llmgateway_ee_clients (
    id UUID PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    client_key VARCHAR(255) UNIQUE NOT NULL,
    enabled BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- All resource tables now include client_id (NOT NULL - every resource must belong to a client)
ALTER TABLE hub_llmgateway_ee_providers 
ADD COLUMN client_id UUID NOT NULL REFERENCES hub_llmgateway_ee_clients(id);

-- Similar for model_definitions, pipelines, and pipeline_plugin_configs
```

## API Usage

### Client Management

#### Create a Client
```bash
curl -X POST http://localhost:3000/ee/api/v1/clients \
  -H "Content-Type: application/json" \
  -d '{
    "name": "acme-corporation",
    "client_key": "cl_acme_1234567890abcdef",
    "enabled": true
  }'
```

#### List Clients
```bash
curl http://localhost:3000/ee/api/v1/clients
```

#### Update a Client
```bash
curl -X PUT http://localhost:3000/ee/api/v1/clients/{client-id} \
  -H "Content-Type: application/json" \
  -d '{
    "name": "acme-corporation-updated",
    "enabled": true
  }'
```

#### Delete a Client
```bash
curl -X DELETE http://localhost:3000/ee/api/v1/clients/{client-id}
```

### Resource Access with Client Authentication

All resource operations (providers, models, pipelines) now require the `x-traceloop-pipeline` header:

#### Create a Provider for a Client
```bash
curl -X POST http://localhost:3000/ee/api/v1/providers \
  -H "Content-Type: application/json" \
  -H "x-traceloop-pipeline: cl_acme_1234567890abcdef" \
  -d '{
    "name": "ACME OpenAI Provider",
    "provider_type": "openai",
    "config": {
      "api_key": {
        "type": "environment",
        "variable_name": "ACME_OPENAI_API_KEY"
      }
    },
    "enabled": true
  }'
```

#### List Providers for a Client
```bash
curl http://localhost:3000/ee/api/v1/providers \
  -H "x-traceloop-pipeline: cl_acme_1234567890abcdef"
```

#### Create a Model for a Client
```bash
curl -X POST http://localhost:3000/ee/api/v1/model-definitions \
  -H "Content-Type: application/json" \
  -H "x-traceloop-pipeline: cl_acme_1234567890abcdef" \
  -d '{
    "key": "acme-gpt-4",
    "model_type": "gpt-4",
    "provider_id": "{provider-id}",
    "config_details": {},
    "enabled": true
  }'
```

## Gateway Integration

The main gateway automatically respects client segregation when the EE feature is enabled:

1. **Configuration Polling**: The gateway polls the database and only loads resources for active clients
2. **Request Routing**: Each request can include the `x-traceloop-pipeline` header to specify the client context
3. **Resource Filtering**: Only models and pipelines belonging to the specified client are available

### Using Models from a Specific Client

```bash
# Chat completion using a client-specific model
curl -X POST http://localhost:3000/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "x-traceloop-pipeline: cl_acme_1234567890abcdef" \
  -d '{
    "model": "acme-gpt-4",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  }'
```

## Security Features

### Client Key Storage
- Client keys are stored directly in the database
- Client keys are used for authentication via the `x-traceloop-pipeline` header

### Resource Isolation
- Database-level filtering ensures clients can only access their own resources
- Foreign key constraints maintain data integrity
- Clients cannot see or modify other clients' configurations

### Middleware Protection
- All resource endpoints are protected by authentication middleware
- Invalid or missing API keys result in 401 Unauthorized responses
- Disabled clients are blocked with 403 Forbidden responses

## Fresh EE Installation

### Clean Architecture
The client segregation feature provides a clean, simple architecture:

1. **Every Resource Requires a Client**: All providers, models, and pipelines must belong to a client
2. **Mandatory Authentication**: The `x-traceloop-pipeline` header is required for all resource operations
3. **Simple Design**: No complex backward compatibility - every resource has a clear owner

### Setup Steps

1. **Run the Migration**:
   ```bash
   # Apply the client segregation database migration
   sqlx migrate run --source ee/migrations
   ```

2. **Create Your First Client**:
   ```bash
   curl -X POST http://localhost:3000/ee/api/v1/clients \
     -H "Content-Type: application/json" \
     -d '{
       "name": "your-organization",
       "client_key": "your-secure-client-key",
       "enabled": true
     }'
   ```

3. **Test the Setup**:
   ```bash
   # Test with a simple client creation
   curl -X POST http://localhost:3000/ee/api/v1/clients \
     -H "Content-Type: application/json" \
     -d '{
       "name": "test-client",
       "client_key": "cl_test_1234567890abcdef",
       "enabled": true
     }'
   ```

## Configuration

### Environment Variables

- `DATABASE_URL`: PostgreSQL connection string (required for EE)
- `DB_POLL_INTERVAL_SECONDS`: How often to poll for config changes (default: 30)

### Feature Flags

The client segregation is automatically enabled in EE builds. No additional configuration is needed.

## Monitoring and Logging

### Client Authentication Events
The system logs authentication events for monitoring:

```
INFO  Client 'acme-corporation' authenticated successfully
WARN  Invalid client key provided in x-traceloop-pipeline header  
WARN  Client 'disabled-client' is disabled but attempted to access resources
```

### Resource Access Patterns
Monitor client resource usage through logs:

```
INFO  Client 'acme-corporation' created provider 'ACME OpenAI Provider'
INFO  Client 'globex-inc' accessed model 'globex-gpt-4'
```

## Best Practices

### Client Key Management
- Use strong, unique client keys for each client
- Rotate client keys regularly
- Use environment variables or secret management systems
- Never expose client keys in logs or client-side code

### Resource Naming
- Use client prefixes in resource names (e.g., `acme-gpt-4`, `globex-claude`)
- Maintain consistent naming conventions
- Document client-specific configurations

### Monitoring
- Monitor client activity and resource usage
- Set up alerts for authentication failures
- Track resource creation and modification patterns

## Troubleshooting

### Common Issues

#### 401 Unauthorized
- Check that the `x-traceloop-pipeline` header is included
- Verify the client key is correct and not expired
- Ensure the client exists and is enabled

#### 404 Not Found
- Verify the resource belongs to the authenticated client
- Check that the resource ID is correct
- Ensure the client has access to the requested resource

#### 409 Conflict
- Resource names must be unique within a client's scope
- Check for existing resources with the same name
- Use different names for different clients

### Debug Commands

```bash
# List all clients
curl http://localhost:3000/ee/api/v1/clients

# Check client's resources
curl -H "x-traceloop-pipeline: your-client-key" \
     http://localhost:3000/ee/api/v1/providers

# Verify authentication
curl -v -H "x-traceloop-pipeline: your-client-key" \
     http://localhost:3000/ee/api/v1/providers
```

## Support

For questions or issues with client segregation:

1. Check the logs for authentication and authorization errors
2. Verify database connectivity and migrations
3. Test with the client creation commands shown above
4. Check the EE API endpoints listed in this documentation

The client segregation feature provides enterprise-grade multi-tenancy while maintaining the simplicity and performance of Traceloop Hub. 