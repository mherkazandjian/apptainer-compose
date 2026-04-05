# Example 10 - Profiles

Conditionally start services based on named profiles. Core services (app and db) run every time, while optional services like a debug shell or monitoring stack only start when their profile is explicitly activated. This avoids running heavyweight tools during normal development while keeping them one flag away.

```mermaid
graph TD
    subgraph always["Core Services (always started)"]
        App["app\npython http.server\n:8000"]
        DB["db\npostgres:16-alpine"]
    end

    subgraph debug["Profile: debug"]
        DebugShell["debug-shell\nalpine\nsleep infinity"]
    end

    subgraph monitoring["Profile: monitoring"]
        Prometheus["prometheus\n:9090"]
        Grafana["grafana\n:3000"]
    end

    Flag1["--profile debug"] -. "activates" .-> debug
    Flag2["--profile monitoring"] -. "activates" .-> monitoring

    Prometheus -- "scrapes metrics" --> App
    Grafana -- "queries" --> Prometheus

    classDef core fill:#47854b,stroke:#2d5e30,color:#fff
    classDef debugSvc fill:#e67e22,stroke:#c0651a,color:#fff
    classDef monSvc fill:#3498db,stroke:#2176ad,color:#fff
    classDef flag fill:#9b59b6,stroke:#7d3c98,color:#fff

    class App,DB core
    class DebugShell debugSvc
    class Prometheus,Grafana monSvc
    class Flag1,Flag2 flag
```

## Usage

```bash
cd examples/10-profiles

# Start only core services (app + db)
apptainer-compose up

# Start core services + debug shell
apptainer-compose --profile debug up

# Start core services + monitoring stack
apptainer-compose --profile monitoring up

# Start everything
apptainer-compose --profile debug --profile monitoring up
```

## What it demonstrates

- The `profiles` field for conditional service startup
- Core services with no profile that always run
- Activating optional services via `--profile <name>`
- Combining multiple profiles in a single command
