# 🐾 Genet — A Lightweight Docker Desktop Built in Rust

Genet is a native, low-latency Docker client written in Rust that communicates directly with `dockerd` over Unix sockets and renders a live-updating GUI using egui.  
It implements real-time container state tracking and basic container control without relying on the Docker CLI or any external SDK.

This is not a wrapper around `docker ps`.  
This is a true Docker Engine client.

---

## Why Genet exists

Most Docker GUIs are built on top of the Docker CLI or high-level SDKs.  
Genet instead talks directly to the Docker Engine over its Unix socket, which allows:

- Lower latency
- Real-time streaming
- Full control over protocol behavior
- Deep insight into how Docker actually works

Genet is both a real tool and a systems-level learning project.

---

## Current Features

- Live container list  
  Containers automatically update when they start or stop, powered by Docker’s `/events` streaming API.

- Direct Docker Engine communication  
  Genet talks to `/var/run/docker.sock` using raw HTTP over Unix sockets.  
  No Docker CLI, no shelling out, no SDKs.

- Real-time container state  
  Containers can be started and stopped, and state changes propagate instantly through the event stream.

- Zero-lock UI architecture  
  No `Mutex`.  
  No shared mutable state.  
  All background threads communicate with the UI using channels.

- Native desktop UI  
  Built with `egui` and `eframe` for a fast, lightweight, cross-platform experience.

---

## How Genet talks to Docker

Genet sends raw HTTP requests over a Unix socket:

```
GET /containers/json?all=1 HTTP/1.1
GET /events HTTP/1.0
POST /containers/{id}/start HTTP/1.1
POST /containers/{id}/stop HTTP/1.1
```

Docker responds with:
- JSON for queries  
- Infinite JSON streams for events  

For streaming endpoints (`/events`), Genet uses HTTP/1.0 to disable chunked encoding, producing clean newline-delimited JSON:

```
{json}
{json}
{json}
```

This makes it safe to parse events line-by-line in real time.

---

## Event-driven UI model

Docker events are converted into strongly typed Rust enums:

```rust
enum DockerEvents {
    StartContainer { id: String },
    StopContainer { id: String },
}
```

The UI thread consumes them in `update()`:

```rust
while let Ok(event) = self.rx.try_recv() {
    self.apply_event(event);
}
```

The UI never blocks.  
It simply reacts to incoming events and renders the new state.

---

## User Interface

Genet currently provides:

- A sidebar listing all containers  
- Status grouping (Running / Exited)  
- A container detail view  
- Start and Stop controls  

All views update instantly when Docker state changes, even if those changes were triggered by another Docker client or the CLI.

---

## Why this project is interesting

Genet is not a CRUD app.

It is a:
- Streaming HTTP client  
- Over Unix sockets  
- With manual protocol handling  
- Live event ingestion  
- Concurrent UI state management  

This is the same class of architecture used in:
- Docker Desktop  
- Kubernetes dashboards  
- IDEs  
- Infrastructure monitoring tools  

---

## Technology stack

- Rust  
- egui / eframe  
- serde / serde_json  
- Unix sockets  
- mpsc channels  
- Docker Engine API  

---

## Roadmap

Planned features:

- Restart containers  
- Live container logs (`/containers/{id}/logs?follow=1`)  
- CPU and memory statistics (`/stats`)  
- Container inspect view  
- Image and volume management  
- Multi-host support  
- Windows named-pipe support  

---

## Author

Built by **Parth Sharma** as a deep systems project exploring:

- Concurrency  
- Networking  
- Streaming APIs  
- GUI state synchronization  
- Docker internals  
