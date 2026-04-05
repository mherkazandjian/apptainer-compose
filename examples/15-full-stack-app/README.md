# Example 15 - Full-Stack App

A production-grade application demonstrating most Docker Compose features working together under apptainer-compose. Seven services form a complete stack: PostgreSQL and Redis as the data layer, a Node.js API and background worker as the application layer, nginx as the reverse proxy, and pgAdmin plus Redis Commander available on-demand via the debug profile. The configuration uses `.env` variable substitution, init scripts, health checks, resource limits, restart policies, capability controls, and named volumes.

```mermaid
graph TD
    Client["Client\n(browser / curl)"] -- "HTTP :80 / :443" --> Nginx

    subgraph compose["apptainer-compose"]

        subgraph proxy_layer["Reverse Proxy"]
            Nginx["nginx\nnginx:alpine\n:80 :443\nnginx.conf mounted"]
        end

        subgraph app_layer["Application Layer"]
            API["api\nnode:20-alpine\n:3000\ncap_drop: ALL\ncap_add: NET_BIND_SERVICE"]
            Worker["worker\nnode:20-alpine\nbackground jobs"]
        end

        subgraph data_layer["Data Layer"]
            DB["db\npostgres:16-alpine\n:5432\nhealthcheck: pg_isready"]
            Redis["redis\nredis:7-alpine\n:6379\nhealthcheck: redis-cli ping"]
        end

        subgraph debug_profile["Profile: debug"]
            PGAdmin["pgadmin\npgadmin4\n:5050"]
            RedisCmd["redis-commander\n:8081"]
        end
    end

    Nginx -- "depends_on" --> API
    API -- "depends_on\nservice_healthy" --> DB
    API -- "depends_on\nservice_healthy" --> Redis
    Worker -- "depends_on\nservice_healthy" --> DB
    Worker -- "depends_on\nservice_healthy" --> Redis
    PGAdmin -. "depends_on" .-> DB
    RedisCmd -. "depends_on" .-> Redis

    DB --- PGVol[("pgdata\nvolume")]
    Redis --- RedisVol[("redis-data\nvolume")]

    EnvFile[".env file\nDB_USER, DB_PASS\nNODE_ENV, JWT_SECRET"] -. "env_file" .-> API
    InitSQL["init.sql\nschema setup"] -. "bind mount" .-> DB

    subgraph resources["Resource Limits"]
        R_DB["db: 1.0 CPU / 512M"]
        R_Redis["redis: 0.5 CPU / 256M"]
        R_API["api: 1.0 CPU / 512M"]
        R_Worker["worker: 0.5 CPU / 256M"]
        R_Nginx["nginx: 0.5 CPU / 128M"]
    end

    R_DB -. "limits" .-> DB
    R_Redis -. "limits" .-> Redis
    R_API -. "limits" .-> API
    R_Worker -. "limits" .-> Worker
    R_Nginx -. "limits" .-> Nginx

    classDef proxy fill:#9b59b6,stroke:#7d3c98,color:#fff
    classDef app fill:#3498db,stroke:#2176ad,color:#fff
    classDef data fill:#47854b,stroke:#2d5e30,color:#fff
    classDef debug fill:#e67e22,stroke:#c0651a,color:#fff
    classDef volume fill:#d4a843,stroke:#a07d2e,color:#000
    classDef config fill:#6b7280,stroke:#4b5563,color:#fff
    classDef external fill:#2c3e50,stroke:#1a252f,color:#fff
    classDef resource fill:#e74c3c,stroke:#c0392b,color:#fff

    class Nginx proxy
    class API,Worker app
    class DB,Redis data
    class PGAdmin,RedisCmd debug
    class PGVol,RedisVol volume
    class EnvFile,InitSQL config
    class Client external
    class R_DB,R_Redis,R_API,R_Worker,R_Nginx resource
```

## Usage

```bash
cd examples/15-full-stack-app

# Start core services (db, redis, api, worker, nginx)
apptainer-compose up -d

# Start everything including debug tools (pgAdmin, redis-commander)
apptainer-compose --profile debug up -d

# Access the application
curl http://localhost

# Access debug tools (when profile is active)
# pgAdmin:          http://localhost:5050
# Redis Commander:  http://localhost:8081
```

## What it demonstrates

- Full dependency chain with health-check gating (`condition: service_healthy`)
- Environment variable substitution from a `.env` file with fallback defaults
- Named volumes for persistent database and cache storage
- Resource limits (`deploy.resources.limits`) on every service
- Restart policies (`restart: unless-stopped`) for resilience
- Linux capability controls (`cap_drop: ALL`, `cap_add: NET_BIND_SERVICE`)
- Debug-only services gated behind the `debug` profile
- Bind-mounted configuration files (`nginx.conf`, `init.sql`)
- A realistic multi-layer architecture suitable as a project template
