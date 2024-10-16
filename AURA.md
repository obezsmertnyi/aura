# Step-by-Step Guide to Launching Metaplex Aura

## 1. Launch the Solana Node

Set up and launch your Solana node according to the official Solana documentation.

## 2. Download RocksDB Backup from the S3 Bucket

Download the latest RocksDB backup from the S3 bucket.

## 3. Update the `.env` File

Download the `.env` example file from [GitHub](https://github.com/metaplex-foundation/aura/blob/main/.env.example) and update it with the correct configuration settings for each microservice as needed.

## 4. Download Docker Compose File

Use the `docker-compose.yaml` file directly from [GitHub](https://raw.githubusercontent.com/metaplex-foundation/aura/main/docker-compose.yaml).

Update the file with your PostgreSQL credentials to ensure that both the username and password are set properly for secure access.

## 5. Start the `ingester`

Run the following Docker Compose command to start the `ingester` service using the downloaded `docker-compose.yaml` file:

```bash
docker compose -f docker-compose.yaml up -d ingester
```

## 6. Run ETL

### 6.1 Backup Solana Snapshot to the Server with Aura

Copy the most recent files from the Solana node snapshot directory:

- incremental-snapshot-\*.tar.zs
- snapshot-\*.tar.zst

### 6.2 Start ETL Process

Run the ETL service to process the snapshots and RocksDB backups:

```bash
docker run -it --rm -d --name solana-snapshot-etl -p 5000:5000 -v /path/to/snapshots:/snapshots -e TCP_PORT=5000 ghcr.io/metaplex-foundation/digital-asset-validator-plugin/solana-snapshot-etl:latest
```

Wait for the ETL process to complete. Success is indicated by the message:

```console
All snapshot files processed successfully.
```

## 7. Start the Synchronizer

Run the following command to start the synchronizer using the downloaded `docker-compose.yaml` file:

```bash
docker compose -f docker-compose.yaml up -d synchronizer
```

### Synchronization Check

Run the following command to continuously check the synchronization status:

```bash
docker exec -it synchronizer sh -c '
while true; do
  solana_slot=$(curl -s -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0","id": 1,"method": "getSlot","params": [{"commitment": "processed"}]}' $INGESTER_RPC_HOST | grep -oP "(?<=\"result\":)[0-9]+");
  synchronizer_slot=$(curl -s localhost:$SYNCHRONIZER_METRICS_PORT/metrics | grep "synchronizer_last_synchronized_slot{name=\"last_synchronized_slot\"}" | awk '{print $2}');
  difference=$((solana_slot - synchronizer_slot));
  echo "Solana slot: $solana_slot";
  echo "Synchronizer last synchronized slot: $synchronizer_slot";
  echo "Difference: $difference slots";
  if [ "$difference" -lt 50 ]; then break; fi;
  sleep 5;
done'
```

We can also add this logic to `make check-synchronizer` for convenience.

## 8. Start the API

Run the following Docker Compose command to start the `das-api` using the downloaded `docker-compose.yaml` file:

```bash
docker compose -f docker-compose.yaml up -d das-api
```

### API Health Check

Use this command to verify the status of the `das-api` service:

```bash
curl -s -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0","id": 1,"method": "health"}' http://api_address:api_server_port
```

Make sure the service returns a healthy status before proceeding.

