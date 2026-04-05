# Example 14 - DNS and Networking

Configure hostnames, DNS servers, extra host entries, network aliases, and multi-network membership. In Apptainer, all services share the host network by default and reach each other via an injected `/etc/hosts` file. This example shows how to layer additional networking configuration on top of that foundation.

```mermaid
graph TD
    subgraph backend_net["Network: backend"]
        API["api\nhostname: api-server\nalias: api-internal\n:5000"]
        Web["web\nhostname: web-frontend\n:80"]
        Monitor["monitor\npolls api every 30s"]
    end

    subgraph frontend_net["Network: frontend"]
        WebFront["web\n(also on frontend)"]
    end

    Web -. "same service\ntwo networks" .-> WebFront

    DNS["DNS Servers\n8.8.8.8\n8.8.4.4"] -- "resolv.conf" --> API
    ExtraHost["extra_hosts\nexternal-service\n10.0.0.50"] -- "/etc/hosts" --> API

    Monitor -- "wget http://api:5000" --> API
    Web -- "depends_on" --> API
    Monitor -- "depends_on" --> API

    Client["Client"] -- "HTTP :80" --> Web

    classDef api fill:#3498db,stroke:#2176ad,color:#fff
    classDef web fill:#47854b,stroke:#2d5e30,color:#fff
    classDef monitor fill:#e67e22,stroke:#c0651a,color:#fff
    classDef network fill:#9b59b6,stroke:#7d3c98,color:#fff
    classDef dns fill:#4a6fa5,stroke:#2c4a7c,color:#fff
    classDef external fill:#6b7280,stroke:#4b5563,color:#fff

    class API api
    class Web,WebFront web
    class Monitor monitor
    class DNS,ExtraHost dns
    class Client external
```

## Usage

```bash
cd examples/14-dns-and-networking
apptainer-compose up -d
curl http://localhost
```

## What it demonstrates

- Custom `hostname` for services
- Custom `dns` server configuration
- `extra_hosts` for injecting entries into `/etc/hosts`
- Network aliases (`aliases`) for alternative service names
- Attaching a service to multiple networks (`backend` + `frontend`)
- Service discovery via injected `/etc/hosts` in Apptainer's host-networking model
