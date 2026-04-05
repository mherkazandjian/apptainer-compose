# Example 07 - GPU (NVIDIA)

Two services demonstrating GPU passthrough for compute workloads. The first uses standard Docker Compose `deploy` syntax to request NVIDIA GPUs. The second uses the `x-apptainer` extension to enable Apptainer's native `--nv` flag and extra bind mounts for a PyTorch training environment.

```mermaid
graph TD
    subgraph host["Host Machine"]
        GPU["NVIDIA GPU(s)"]
        Driver["NVIDIA Driver"]
        Scratch[("/scratch\nfast storage")]
    end

    subgraph compose["apptainer-compose"]
        subgraph svc1["gpu-test service"]
            CUDA["nvidia/cuda:12.3.1\n-base-ubuntu22.04"]
            Deploy["deploy:\n  driver: nvidia\n  count: all\n  capabilities: gpu"]
            CMD1["command: nvidia-smi"]
        end

        subgraph svc2["gpu-training service"]
            PyTorch["pytorch/pytorch:2.2.0\n-cuda12.1-cudnn8-runtime"]
            XApptainer["x-apptainer:\n  nv: true\n  bind_extra:\n    - /scratch:/scratch"]
        end
    end

    GPU -- "all GPUs" --> CUDA
    Driver -- "deploy.resources" --> CUDA
    GPU -- "--nv flag" --> PyTorch
    Driver -- "x-apptainer.nv" --> PyTorch
    Scratch -- "bind_extra" --> PyTorch

    CUDA --> Out1["nvidia-smi output"]
    PyTorch --> Out2["Training with\nGPU acceleration"]

    classDef gpu fill:#76b900,stroke:#5a8c00,color:#000
    classDef driver fill:#333,stroke:#555,color:#76b900
    classDef container fill:#3498db,stroke:#2176ad,color:#fff
    classDef apptainer fill:#9b59b6,stroke:#7d3c98,color:#fff
    classDef storage fill:#e67e22,stroke:#c0651a,color:#fff
    classDef output fill:#d4a843,stroke:#a07d2e,color:#000

    class GPU gpu
    class Driver driver
    class CUDA,CMD1,Deploy container
    class PyTorch,XApptainer apptainer
    class Scratch storage
    class Out1,Out2 output
```

## Usage

```bash
cd examples/07-gpu-nvidia

# Run the nvidia-smi diagnostic
apptainer-compose up gpu-test

# Run a GPU training workload
apptainer-compose up gpu-training
```

## What it demonstrates

- GPU passthrough using Docker Compose `deploy.resources.reservations.devices` syntax
- Apptainer-native GPU support via the `x-apptainer.nv` extension flag
- Extra bind mounts with `x-apptainer.bind_extra` for scratch storage
- Compatibility between Docker Compose GPU syntax and Apptainer's `--nv` flag
- Running CUDA and PyTorch workloads in containers
