# 17 — Elastic Stack

Full Elastic (ELK) stack with observability: Elasticsearch, Kibana, Logstash,
Filebeat, Metricbeat, APM Server, and a demo application that generates
live traffic, logs, and APM traces.

```mermaid
graph TD
    Client["Client\n(browser / curl)"] -- "HTTP :8080" --> Demo

    subgraph compose["apptainer-compose"]

        subgraph app_layer["Application"]
            Demo["demo-app\npython:3.12-alpine\n:8080\nFlask + traffic generator"]
        end

        subgraph traces_pipe["Traces Pipeline"]
            APM["apm-server\n:8200"]
        end

        subgraph logs_pipe["Logs Pipeline"]
            AppLogs[("app-logs")]
            Filebeat["filebeat"]
            Logstash["logstash\n:5044"]
        end

        subgraph metrics_pipe["Metrics Pipeline"]
            Metricbeat["metricbeat"]
        end

        subgraph storage_layer["Storage & Visualization"]
            ES["elasticsearch\n:9200"]
            ESData[("esdata")]
            Kibana["kibana\n:5601"]
        end

    end

    Demo -- "APM traces" --> APM
    Demo -- "writes logs" --> AppLogs
    AppLogs --> Filebeat
    Filebeat -- ":5044" --> Logstash

    APM --> ES
    Logstash --> ES
    Metricbeat -- "system & ES metrics" --> ES

    Kibana -- "queries" --> ES
    ES --- ESData

    APM -. "registers" .-> Kibana
    Metricbeat -. "loads dashboards" .-> Kibana

    classDef app fill:#47854b,stroke:#2d5e30,color:#fff
    classDef collector fill:#e67e22,stroke:#c0651a,color:#fff
    classDef storage fill:#3498db,stroke:#2176ad,color:#fff
    classDef ui fill:#9b59b6,stroke:#7d3c98,color:#fff
    classDef volume fill:#6b7280,stroke:#4b5563,color:#fff
    classDef external fill:#95a5a6,stroke:#7f8c8d,color:#fff

    class Demo app
    class APM,Filebeat,Logstash,Metricbeat collector
    class ES storage
    class Kibana ui
    class AppLogs,ESData volume
    class Client external
```

## Services

| Service         | Port  | Description                                  |
|-----------------|-------|----------------------------------------------|
| elasticsearch   | 9200  | Search & analytics engine                    |
| kibana          | 5601  | Visualization & dashboards                   |
| logstash        | 5044  | Data processing pipeline (Beats input)       |
| apm-server      | 8200  | Application performance monitoring collector |
| filebeat        | —     | Ships demo-app logs → Logstash → ES          |
| metricbeat      | —     | Ships system & ES metrics → ES + dashboards  |
| demo-app        | 8080  | Flask app with APM tracing + traffic gen     |

## Quick start

```bash
apptainer-compose up -d
```

Wait ~60 seconds for everything to initialize, then open:

- **Kibana**: http://localhost:5601
- **Demo app**: http://localhost:8080
- **Elasticsearch**: http://localhost:9200

## What to explore in Kibana

1. **Discover** — search `demo-logs-*` index for application logs
2. **APM** → Services → `demo-app` — request traces, latency, errors
3. **Dashboards** — Metricbeat auto-loads system dashboards (CPU, memory, disk, network)
4. **Observability** — unified view of logs, metrics, and traces

## Tear down

```bash
apptainer-compose down
```
