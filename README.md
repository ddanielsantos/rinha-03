## Rodar localmente

No diretório `/rinha`, execute:

```bash
docker compose up -d
```

Na raiz do projeto, execute:

```bash
docker compose up redis
```

No PowerShell, execute:

```bash
$env:REDIS_URL="redis://localhost:6379"; $env:PAYMENT_PROCESSOR_URL_DEFAULT="http://localhost:8001";
cargo run
```

(opcional) Para resetar as variáveis de ambiente, execute:

```bash
Remove-Item Env:REDIS_URL; Remove-Item Env:PAYMENT_PROCESSOR_URL_DEFAULT;
```