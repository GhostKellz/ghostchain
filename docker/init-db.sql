-- GhostChain PostgreSQL initialization script

-- Create database and user (already done by env vars, but keeping for reference)
-- CREATE DATABASE ghostchain;
-- CREATE USER ghostchain WITH PASSWORD 'ghostchain_secure_password';
-- GRANT ALL PRIVILEGES ON DATABASE ghostchain TO ghostchain;

-- Connect to ghostchain database
\c ghostchain;

-- Create tables for blockchain indexing and analytics

-- Blocks table
CREATE TABLE IF NOT EXISTS blocks (
    height BIGINT PRIMARY KEY,
    hash VARCHAR(66) UNIQUE NOT NULL,
    previous_hash VARCHAR(66) NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    validator VARCHAR(42) NOT NULL,
    state_root VARCHAR(66) NOT NULL,
    transaction_count INTEGER NOT NULL DEFAULT 0,
    gas_used BIGINT NOT NULL DEFAULT 0,
    gas_limit BIGINT NOT NULL DEFAULT 0,
    size_bytes INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Transactions table
CREATE TABLE IF NOT EXISTS transactions (
    id UUID PRIMARY KEY,
    block_height BIGINT REFERENCES blocks(height),
    tx_index INTEGER NOT NULL,
    hash VARCHAR(66) UNIQUE,
    tx_type VARCHAR(50) NOT NULL,
    from_address VARCHAR(42),
    to_address VARCHAR(42),
    amount NUMERIC(78, 0), -- Support for 256-bit integers
    token_type VARCHAR(20),
    gas_price BIGINT NOT NULL,
    gas_used BIGINT NOT NULL,
    signature BYTEA,
    status VARCHAR(20) DEFAULT 'pending',
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Accounts table
CREATE TABLE IF NOT EXISTS accounts (
    address VARCHAR(42) PRIMARY KEY,
    spirit_balance NUMERIC(78, 0) DEFAULT 0,
    mana_balance NUMERIC(78, 0) DEFAULT 0,
    rlusd_balance NUMERIC(78, 0) DEFAULT 0,
    nonce BIGINT DEFAULT 0,
    soul_id UUID,
    staked_amount NUMERIC(78, 0) DEFAULT 0,
    mana_earned NUMERIC(78, 0) DEFAULT 0,
    first_seen TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_active TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Contracts table
CREATE TABLE IF NOT EXISTS contracts (
    contract_id VARCHAR(66) PRIMARY KEY,
    deployer VARCHAR(42) NOT NULL,
    contract_type VARCHAR(20) NOT NULL,
    name VARCHAR(100),
    symbol VARCHAR(10),
    code_hash VARCHAR(66),
    creation_block BIGINT REFERENCES blocks(height),
    gas_used BIGINT NOT NULL,
    status VARCHAR(20) DEFAULT 'active',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Domains table (for ZNS/ENS/UD integration)
CREATE TABLE IF NOT EXISTS domains (
    domain VARCHAR(255) PRIMARY KEY,
    domain_type VARCHAR(20) NOT NULL, -- 'ens', 'unstoppable', 'web5', 'ghost'
    owner VARCHAR(42),
    resolver VARCHAR(42),
    expiry_date TIMESTAMP WITH TIME ZONE,
    records JSONB,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Validators table
CREATE TABLE IF NOT EXISTS validators (
    address VARCHAR(42) PRIMARY KEY,
    staked_amount NUMERIC(78, 0) NOT NULL,
    is_active BOOLEAN DEFAULT true,
    commission_rate DECIMAL(5, 4),
    blocks_produced INTEGER DEFAULT 0,
    last_block_time TIMESTAMP WITH TIME ZONE,
    uptime_percentage DECIMAL(5, 2),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Events table (for contract events and system events)
CREATE TABLE IF NOT EXISTS events (
    id BIGSERIAL PRIMARY KEY,
    block_height BIGINT REFERENCES blocks(height),
    transaction_id UUID REFERENCES transactions(id),
    contract_id VARCHAR(66) REFERENCES contracts(contract_id),
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB,
    topics TEXT[],
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Performance metrics table
CREATE TABLE IF NOT EXISTS metrics (
    id BIGSERIAL PRIMARY KEY,
    metric_name VARCHAR(100) NOT NULL,
    metric_value DECIMAL,
    labels JSONB,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_blocks_validator ON blocks(validator);

CREATE INDEX IF NOT EXISTS idx_transactions_block_height ON transactions(block_height);
CREATE INDEX IF NOT EXISTS idx_transactions_from_address ON transactions(from_address);
CREATE INDEX IF NOT EXISTS idx_transactions_to_address ON transactions(to_address);
CREATE INDEX IF NOT EXISTS idx_transactions_timestamp ON transactions(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions(tx_type);

CREATE INDEX IF NOT EXISTS idx_accounts_balances ON accounts(spirit_balance DESC);
CREATE INDEX IF NOT EXISTS idx_accounts_last_active ON accounts(last_active DESC);

CREATE INDEX IF NOT EXISTS idx_contracts_deployer ON contracts(deployer);
CREATE INDEX IF NOT EXISTS idx_contracts_type ON contracts(contract_type);
CREATE INDEX IF NOT EXISTS idx_contracts_creation_block ON contracts(creation_block);

CREATE INDEX IF NOT EXISTS idx_domains_owner ON domains(owner);
CREATE INDEX IF NOT EXISTS idx_domains_type ON domains(domain_type);
CREATE INDEX IF NOT EXISTS idx_domains_expiry ON domains(expiry_date);

CREATE INDEX IF NOT EXISTS idx_validators_active ON validators(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_validators_stake ON validators(staked_amount DESC);

CREATE INDEX IF NOT EXISTS idx_events_block_height ON events(block_height);
CREATE INDEX IF NOT EXISTS idx_events_contract ON events(contract_id);
CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type);
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_metrics_name_timestamp ON metrics(metric_name, timestamp DESC);

-- Create views for common queries

-- Account summary view
CREATE OR REPLACE VIEW account_summary AS
SELECT 
    a.address,
    a.spirit_balance,
    a.mana_balance,
    a.rlusd_balance,
    a.staked_amount,
    a.nonce,
    COUNT(t.id) as transaction_count,
    MAX(t.timestamp) as last_transaction
FROM accounts a
LEFT JOIN transactions t ON (a.address = t.from_address OR a.address = t.to_address)
GROUP BY a.address, a.spirit_balance, a.mana_balance, a.rlusd_balance, a.staked_amount, a.nonce;

-- Block summary view
CREATE OR REPLACE VIEW block_summary AS
SELECT 
    b.height,
    b.hash,
    b.timestamp,
    b.validator,
    b.transaction_count,
    b.gas_used,
    AVG(t.gas_price) as avg_gas_price
FROM blocks b
LEFT JOIN transactions t ON b.height = t.block_height
GROUP BY b.height, b.hash, b.timestamp, b.validator, b.transaction_count, b.gas_used
ORDER BY b.height DESC;

-- Token transfer events view
CREATE OR REPLACE VIEW token_transfers AS
SELECT 
    t.id,
    t.hash,
    t.block_height,
    t.from_address,
    t.to_address,
    t.amount,
    t.token_type,
    t.timestamp,
    b.validator
FROM transactions t
JOIN blocks b ON t.block_height = b.height
WHERE t.tx_type = 'Transfer'
ORDER BY t.timestamp DESC;

-- Daily statistics view
CREATE OR REPLACE VIEW daily_stats AS
SELECT 
    DATE(b.timestamp) as date,
    COUNT(b.height) as blocks_created,
    COUNT(t.id) as transactions_count,
    COUNT(DISTINCT t.from_address) as unique_senders,
    COUNT(DISTINCT t.to_address) as unique_receivers,
    SUM(CASE WHEN t.token_type = 'Spirit' THEN t.amount ELSE 0 END) as spirit_volume,
    AVG(t.gas_price) as avg_gas_price
FROM blocks b
LEFT JOIN transactions t ON b.height = t.block_height
GROUP BY DATE(b.timestamp)
ORDER BY date DESC;

-- Grant permissions
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO ghostchain;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO ghostchain;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA public TO ghostchain;

-- Insert some initial data (optional)
INSERT INTO accounts (address, spirit_balance, mana_balance, rlusd_balance) VALUES
('0x0000000000000000000000000000000000000001', '1000000000000000000000000', 0, '10000000000000000000000'),
('0x0000000000000000000000000000000000000002', '100000000000000000000000', 0, '1000000000000000000000'),
('0x0000000000000000000000000000000000000003', '10000000000000000000000', 0, '100000000000000000000')
ON CONFLICT (address) DO NOTHING;

COMMIT;