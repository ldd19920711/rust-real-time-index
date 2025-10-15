# Rust Real-Time Index

这是一个基于 Rust 的**实时指数计算系统**，可以从多个交易所获取行情数据，通过自定义公式计算指数，并提供实时行情打印功能。项目使用 `tokio` 异步运行、`sqlx` 数据库操作，以及模块化设计支持多交易所扩展。

---

## 功能概览

1. **实时行情获取**
    - 支持 Binance、Bitget、OKEx 等交易所
    - Websocket 实时订阅
    - 自动心跳和重连

2. **指数计算**
    - 支持自定义公式
    - 按配置的交易所数据计算指数
    - 异步更新指数列表

3. **任务调度**
    - `price_updater`: 获取行情更新
    - `index_calculator_task`: 根据公式计算指数
    - `market_printer`: 定时打印行情数据

4. **数据库存储**
    - Postgres 数据库存储指数配置、任务配置、Symbol 对应关系
    - 使用 `sqlx` 异步查询

## 环境配置

1. Rust & Cargo  
   建议使用最新 stable 版本。

2. 数据库环境  
   Postgres >= 12  
   `.env` 文件配置：
```env
DB_HOST=127.0.0.1
DB_PORT=5432
DB_NAME=your_db
DB_USER=your_user
DB_PASSWORD=your_password
DB_MAX_CONNECTIONS=20
