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

CREATE TABLE index_data_btcusdt (
                                  id BIGSERIAL PRIMARY KEY,            -- 自增主键
                                  symbol VARCHAR(45) NOT NULL,         -- 指数名称
                                  last NUMERIC(36,18) NOT NULL,       -- 最新指数值
                                  formula VARCHAR(512) NOT NULL,       -- 计算公式
                                  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                                  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

COMMENT ON COLUMN index_data_btcusdt.id IS '自增主键';
COMMENT ON COLUMN index_data_btcusdt.symbol IS '指数名称';
COMMENT ON COLUMN index_data_btcusdt.last IS '最新指数值';
COMMENT ON COLUMN index_data_btcusdt.formula IS '计算公式';


CREATE TABLE index_data_ethusdt (
                                  id BIGSERIAL PRIMARY KEY,            -- 自增主键
                                  symbol VARCHAR(45) NOT NULL,         -- 指数名称
                                  last NUMERIC(36,18) NOT NULL,       -- 最新指数值
                                  formula VARCHAR(512) NOT NULL,       -- 计算公式
                                  created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
                                  updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

COMMENT ON COLUMN index_data_ethusdt.id IS '自增主键';
COMMENT ON COLUMN index_data_ethusdt.symbol IS '指数名称';
COMMENT ON COLUMN index_data_ethusdt.last IS '最新指数值';
COMMENT ON COLUMN index_data_ethusdt.formula IS '计算公式';

create table index_kline_data
(
    id         bigint                                      not null,
    symbol     varchar(45)                                 not null,
    open       numeric(36, 18)                             not null,
    high       numeric(36, 18)                             not null,
    low        numeric(36, 18)                             not null,
    close      numeric(36, 18)                             not null,
    ts         bigint                                      not null,
    created_at timestamp   default now(),
    updated_at timestamp   default now(),
    interval   varchar(10) default '1m'::character varying not null,
    primary key (id, symbol, interval)
);

alter table index_kline_data
    owner to postgres;

