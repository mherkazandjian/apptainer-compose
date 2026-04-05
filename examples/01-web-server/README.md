# Example 01 - Web Server

A single nginx web server exposed on port 8080 with a restart policy. This example shows how to run a long-lived network service and access it from the host.

```mermaid
graph LR
    Client["Client\n(browser / curl)"] -- "HTTP request\nport 8080" --> PortMap["Host :8080"]
    PortMap -- "forwards to :80" --> Nginx["nginx service\n(nginx:alpine)"]
    Nginx -- "serves" --> Content["Default nginx\nwelcome page"]

    subgraph compose["apptainer-compose"]
        PortMap
        Nginx
    end

    Restart["restart: unless-stopped"] -. "policy" .-> Nginx

    classDef external fill:#6b7280,stroke:#4b5563,color:#fff
    classDef port fill:#4a6fa5,stroke:#2c4a7c,color:#fff
    classDef container fill:#47854b,stroke:#2d5e30,color:#fff
    classDef policy fill:#9b59b6,stroke:#7d3c98,color:#fff
    classDef content fill:#d4a843,stroke:#a07d2e,color:#000

    class Client external
    class PortMap port
    class Nginx container
    class Restart policy
    class Content content
```

## Usage

```bash
cd examples/01-web-server
apptainer-compose up -d
curl http://localhost:8080
```

## What it demonstrates

- Running a long-lived service in the background
- Port mapping from host to container (`8080:80`)
- Restart policy (`unless-stopped`) for automatic recovery
