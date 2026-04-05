"""
Demo app that generates logs, HTTP traffic, and APM traces.
Provides a few endpoints and a background traffic generator so
Kibana has real data to visualize immediately.
"""

import json
import logging
import os
import random
import threading
import time
from datetime import datetime, timezone
from pathlib import Path

from flask import Flask, jsonify, request

# ---------- APM setup ----------
try:
    from elasticapm.contrib.flask import ElasticAPM
    apm_available = True
except ImportError:
    apm_available = False

app = Flask(__name__)

if apm_available:
    apm = ElasticAPM(app)

# ---------- File logging ----------
log_dir = os.environ.get("APP_LOG_DIR", "/var/log/demo-app")
Path(log_dir).mkdir(parents=True, exist_ok=True)

file_handler = logging.FileHandler(os.path.join(log_dir, "app.log"))
file_handler.setFormatter(
    logging.Formatter("%(asctime)s %(levelname)s %(name)s %(message)s")
)
app.logger.addHandler(file_handler)
app.logger.setLevel(logging.INFO)

# ---------- In-memory store ----------
items = {}
request_count = 0
start_time = time.time()

# ---------- Endpoints ----------

@app.route("/")
def index():
    return jsonify({
        "service": "demo-app",
        "version": "1.0.0",
        "endpoints": ["/", "/health", "/api/items", "/api/items/<id>", "/api/stats", "/api/slow", "/api/error"],
    })


@app.route("/health")
def health():
    return jsonify({"status": "healthy", "uptime_seconds": round(time.time() - start_time, 1)})


@app.route("/api/items", methods=["GET"])
def list_items():
    global request_count
    request_count += 1
    app.logger.info("Listed %d items", len(items))
    return jsonify(list(items.values()))


@app.route("/api/items", methods=["POST"])
def create_item():
    global request_count
    request_count += 1
    data = request.get_json(force=True, silent=True) or {}
    item_id = str(len(items) + 1)
    item = {
        "id": item_id,
        "name": data.get("name", f"item-{item_id}"),
        "created_at": datetime.now(timezone.utc).isoformat(),
    }
    items[item_id] = item
    app.logger.info("Created item %s: %s", item_id, item["name"])
    return jsonify(item), 201


@app.route("/api/items/<item_id>", methods=["GET"])
def get_item(item_id):
    global request_count
    request_count += 1
    item = items.get(item_id)
    if item is None:
        app.logger.warning("Item %s not found", item_id)
        return jsonify({"error": "not found"}), 404
    return jsonify(item)


@app.route("/api/stats")
def stats():
    return jsonify({
        "request_count": request_count,
        "item_count": len(items),
        "uptime_seconds": round(time.time() - start_time, 1),
    })


@app.route("/api/slow")
def slow():
    """Simulates a slow endpoint — visible in APM as a long transaction."""
    delay = random.uniform(0.5, 2.0)
    time.sleep(delay)
    app.logger.info("Slow request completed in %.2fs", delay)
    return jsonify({"delay_seconds": round(delay, 2)})


@app.route("/api/error")
def error():
    """Simulates errors — visible in APM error tracking."""
    if random.random() < 0.5:
        app.logger.error("Simulated internal error")
        raise RuntimeError("Simulated internal error")
    app.logger.warning("Near-miss error simulation")
    return jsonify({"status": "survived"})


@app.errorhandler(Exception)
def handle_error(e):
    app.logger.error("Unhandled exception: %s", str(e))
    return jsonify({"error": str(e)}), 500


# ---------- Background traffic generator ----------

def generate_traffic():
    """Sends requests to own endpoints to produce realistic telemetry."""
    import urllib.request

    base = "http://127.0.0.1:8080"
    time.sleep(10)  # wait for server to be ready
    app.logger.info("Traffic generator started")

    names = [
        "widget", "gadget", "doohickey", "gizmo", "thingamajig",
        "contraption", "apparatus", "mechanism", "device", "instrument",
    ]

    while True:
        try:
            # Mix of different request types
            r = random.random()
            if r < 0.3:
                # Create an item
                name = random.choice(names) + "-" + str(random.randint(1, 9999))
                data = json.dumps({"name": name}).encode()
                req = urllib.request.Request(
                    f"{base}/api/items", data=data,
                    headers={"Content-Type": "application/json"},
                    method="POST",
                )
                urllib.request.urlopen(req, timeout=5)
            elif r < 0.55:
                # List items
                urllib.request.urlopen(f"{base}/api/items", timeout=5)
            elif r < 0.7:
                # Get specific item (may 404)
                item_id = str(random.randint(1, max(len(items) + 5, 10)))
                urllib.request.urlopen(f"{base}/api/items/{item_id}", timeout=5)
            elif r < 0.8:
                # Health check
                urllib.request.urlopen(f"{base}/health", timeout=5)
            elif r < 0.9:
                # Slow endpoint
                urllib.request.urlopen(f"{base}/api/slow", timeout=10)
            else:
                # Error endpoint
                try:
                    urllib.request.urlopen(f"{base}/api/error", timeout=5)
                except urllib.error.HTTPError:
                    pass
        except Exception:
            pass

        time.sleep(random.uniform(0.5, 3.0))


# ---------- Main ----------

if __name__ == "__main__":
    traffic_thread = threading.Thread(target=generate_traffic, daemon=True)
    traffic_thread.start()
    app.run(host="0.0.0.0", port=8080)
