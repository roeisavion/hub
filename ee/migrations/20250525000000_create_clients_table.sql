-- Create clients table and add foreign key constraints

-- Create clients table
CREATE TABLE hub_llmgateway_ee_clients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    client_key VARCHAR(255) NOT NULL UNIQUE, -- Store x-traceloop-pipeline header value
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add foreign key constraints to existing client_id columns
-- (The client_id columns are already created in the original table migrations)
ALTER TABLE hub_llmgateway_ee_providers 
ADD CONSTRAINT fk_providers_client_id 
FOREIGN KEY (client_id) REFERENCES hub_llmgateway_ee_clients(id) ON DELETE CASCADE;

ALTER TABLE hub_llmgateway_ee_model_definitions 
ADD CONSTRAINT fk_model_definitions_client_id 
FOREIGN KEY (client_id) REFERENCES hub_llmgateway_ee_clients(id) ON DELETE CASCADE;

ALTER TABLE hub_llmgateway_ee_pipelines 
ADD CONSTRAINT fk_pipelines_client_id 
FOREIGN KEY (client_id) REFERENCES hub_llmgateway_ee_clients(id) ON DELETE CASCADE;

ALTER TABLE hub_llmgateway_ee_pipeline_plugin_configs 
ADD CONSTRAINT fk_pipeline_plugin_configs_client_id 
FOREIGN KEY (client_id) REFERENCES hub_llmgateway_ee_clients(id) ON DELETE CASCADE;

-- Create index for clients table
CREATE INDEX idx_clients_client_key ON hub_llmgateway_ee_clients(client_key);
-- Note: Indexes for client_id columns on other tables are created in their respective table creation migrations

-- Add updated_at trigger for clients table
CREATE TRIGGER update_clients_updated_at
BEFORE UPDATE ON hub_llmgateway_ee_clients
FOR EACH ROW
EXECUTE FUNCTION update_modified_column();

-- Optional: For existing data, you might want to create a default client
-- This is commented out since it depends on your migration strategy
-- INSERT INTO hub_llmgateway_ee_clients (name, client_key) 
-- VALUES ('default', 'default_client_key_placeholder'); 