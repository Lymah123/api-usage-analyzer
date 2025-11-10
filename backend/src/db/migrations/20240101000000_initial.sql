-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    organization VARCHAR(255),
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_email ON users(email);

-- Create api_keys table
CREATE TABLE api_keys (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    provider VARCHAR(100) NOT NULL,
    encrypted_key TEXT NOT NULL,
    cost_per_1k_input DECIMAL(10,6) NOT NULL,
    cost_per_1k_output DECIMAL(10,6) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);

-- Create api_usage table
CREATE TABLE api_usage (
    id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    input_tokens INTEGER NOT NULL DEFAULT 0,
    output_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    requests INTEGER NOT NULL DEFAULT 1,
    errors INTEGER NOT NULL DEFAULT 0,
    cost DECIMAL(10,4) NOT NULL,
    model_name VARCHAR(100),
    endpoint VARCHAR(255),
    status_code INTEGER,
    response_time_ms INTEGER,
    metadata JSONB
);

CREATE INDEX idx_api_usage_user_id ON api_usage(user_id);
CREATE INDEX idx_api_usage_api_key_id ON api_usage(api_key_id);
CREATE INDEX idx_api_usage_timestamp ON api_usage(timestamp DESC);
CREATE INDEX idx_api_usage_user_timestamp ON api_usage(user_id, timestamp DESC);

-- Create predictions table
CREATE TABLE predictions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    api_key_id UUID REFERENCES api_keys(id) ON DELETE SET NULL,
    prediction_date TIMESTAMPTZ NOT NULL,
    predicted_daily_cost DECIMAL(10,4) NOT NULL,
    predicted_weekly_cost DECIMAL(10,4) NOT NULL,
    predicted_monthly_cost DECIMAL(10,4) NOT NULL,
    confidence_score DECIMAL(3,2) NOT NULL,
    model_used VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_predictions_user_id ON predictions(user_id);
CREATE INDEX idx_predictions_created_at ON predictions(created_at DESC);

-- Create alerts table
CREATE TABLE alerts (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    api_key_id UUID REFERENCES api_keys(id) ON DELETE SET NULL,
    alert_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL DEFAULT 'info',
    threshold_value DECIMAL(10,4),
    current_value DECIMAL(10,4),
    message TEXT NOT NULL,
    is_read BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_alerts_user_id ON alerts(user_id);
CREATE INDEX idx_alerts_is_read ON alerts(is_read);

-- Create budgets table
CREATE TABLE budgets (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    api_key_id UUID REFERENCES api_keys(id) ON DELETE SET NULL,
    limit_type VARCHAR(20) NOT NULL, -- 'daily', 'weekly', 'monthly'
    limit_amount DECIMAL(10,4) NOT NULL,
    alert_threshold DECIMAL(3,2) DEFAULT 0.80,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_budgets_user_id ON budgets(user_id);