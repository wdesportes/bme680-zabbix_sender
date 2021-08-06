## bme680 data zabbix sender

Send BME 680 data to Zabbix using Zabbix sender

### How to run

(will run each 10s)

```sh
WAIT_TIME_MS=10000 RUST_LOG=info ZABBIX_HOST=localhost ZABBIX_SEND_AS_HOST=my_host cargo run --release
```

### ENV variables

- RUST_LOG (recommended: info), see: [env_logger crate](https://github.com/env-logger-rs/env_logger/)
- ZABBIX_HOST (the hostname or IP of the Zabbix host)
- ZABBIX_SEND_AS_HOST (the host you have configured on Zabbix, hint: the name not the visible name)
- ZABBIX_PORT (defaults to 10051)
- WAIT_TIME_MS (defaults to 5000, hint: 60000ms = 60s = 1m)

### "Zabbix trapper"s to configure

- Numeric (float), key: bme680.temperature, unit: °C
- Numeric (float), key: bme680.pressure, unit: hPa
- Numeric (float), key: bme680.humidity, unit: %
- Numeric (unsigned), key: bme680.gas-resistence, unit: Ω
