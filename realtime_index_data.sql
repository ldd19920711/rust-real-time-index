INSERT INTO index_config (name, formula) VALUES
                                             ('BTCUSDT', '(Binance.BTCUSDT + Bitget.BTCUSDT)/2'),
                                             ('ETHUSDT', '(Binance.ETHUSDT + Bitget.ETHUSDT)/2');


INSERT INTO symbol (symbol_name, exchange_name, third_symbol_name) VALUES
                                                                       ('BTCUSDT', 'Binance', 'BTCUSDT'),
                                                                       ('ETHUSDT', 'Binance', 'ETHUSDT'),
                                                                       ('BTCUSDT', 'Bitget', 'BTCUSDT'),
                                                                       ('ETHUSDT', 'Bitget', 'ETHUSDT');
INSERT INTO task (exchange_name, symbol_ids, is_enabled) VALUES
                                                             ('Binance', '1,2', TRUE),
                                                             ('Bitget', '3,4', TRUE);