# Example 16 - Ext3 Volume Images

Persistent named volumes backed by ext3 filesystem images instead of plain directories. The volume is created as a `.ext3` image file via `apptainer overlay create` and bound into containers using Apptainer's native `--bind image-src=/` support. Data persists across container restarts. No root required.

```mermaid
graph TD
    subgraph compose["apptainer-compose"]
        Writer["writer\n(alpine:latest)\nappends to /data/log.txt"]
        Reader["reader\n(alpine:latest)\nreads /data/log.txt"]
    end

    Reader -- "depends_on" --> Writer

    subgraph volume["Volume: appdata"]
        Img["appdata.ext3\n64MB ext3 image file\n.apptainer-compose/volumes/"]
    end

    Writer -- "--bind appdata.ext3:/data\nimage-src=/" --> Img
    Reader -- "--bind appdata.ext3:/data:ro\nimage-src=/" --> Img

    Create["apptainer overlay create\n--size 64 appdata.ext3"] -. "created on first up" .-> Img

    classDef container fill:#47854b,stroke:#2d5e30,color:#fff
    classDef volume fill:#d4a843,stroke:#a07d2e,color:#000
    classDef tool fill:#4a6fa5,stroke:#2c4a7c,color:#fff

    class Writer,Reader container
    class Img volume
    class Create tool
```

## Usage

```bash
cd examples/16-ext3-volumes
apptainer-compose up
```

On first run, apptainer-compose creates a 64MB ext3 image at `.apptainer-compose/volumes/appdata.ext3`. The writer service appends a timestamped line, and the reader prints it. On subsequent runs, data from previous runs is still present in the image.

## What it demonstrates

- Named volumes with `x-apptainer.backend: ext3` for real filesystem images
- Persistent storage that survives container restarts (unlike `--writable-tmpfs`)
- Size-limited volumes (64MB in this example)
- Read-only volume mounts from ext3 images
- Rootless volume creation via `apptainer overlay create`
