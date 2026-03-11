from kafka import KafkaProducer
import json
import time

TOPIC = "1-browser"
BOOTSTRAP = "127.0.0.1:9092"

producer = KafkaProducer(
    bootstrap_servers=BOOTSTRAP,
    value_serializer=lambda v: json.dumps(v).encode("utf-8")
)

print(f"🚀 Sending messages to topic: {TOPIC}")

while True:
    data = {
        "url": "google.com",
        "timestamp": int(time.time() * 1000)
    }

    producer.send(TOPIC, value=data)
    producer.flush()

    print("✅ Message sent:", data)
    time.sleep(1)
