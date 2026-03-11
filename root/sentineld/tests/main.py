from kafka import KafkaConsumer
import json

TOPIC = "1-browser"
BOOTSTRAP = "127.0.0.1:9092"

print("Hello")

consumer = KafkaConsumer(
    TOPIC,
    bootstrap_servers=BOOTSTRAP,
enable_auto_commit=True,
value_deserializer=lambda m: json.loads(m.decode("utf-8"))
)

print(f"📡 Streaming messages from topic: {TOPIC}\n")

for message in consumer:
    data = message.value
    print("------ Browser Event ------")
    print("URL:", data.get("url"))
    print("Timestamp:", data.get("timestamp"))
    print()
