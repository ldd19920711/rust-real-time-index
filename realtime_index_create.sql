CREATE TABLE index_config (
                              id SERIAL PRIMARY KEY,
                              name VARCHAR(64) NOT NULL UNIQUE,         -- 指数名称，例如 BTCUSDT
                              formula TEXT NOT NULL,                    -- 计算公式
                              is_active BOOLEAN DEFAULT TRUE,           -- 是否启用
                              created_at TIMESTAMPTZ DEFAULT now(),     -- 创建时间
                              updated_at TIMESTAMPTZ DEFAULT now()      -- 更新时间
);

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_index_config_updated_at
    BEFORE UPDATE ON index_config
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TABLE task (
                      id BIGSERIAL PRIMARY KEY,
                      exchange_name VARCHAR(50) NOT NULL,
                      symbol_ids TEXT NOT NULL,
                      is_enabled BOOLEAN DEFAULT TRUE,
                      created_at TIMESTAMP DEFAULT NOW(),
                      updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE symbol (
                        id BIGSERIAL PRIMARY KEY,
                        symbol_name VARCHAR(20) NOT NULL,
                        exchange_name VARCHAR(50) NOT NULL,
                        third_symbol_name VARCHAR(50) NOT NULL,
                        created_at TIMESTAMP DEFAULT NOW(),
                        updated_at TIMESTAMP DEFAULT NOW()
);
