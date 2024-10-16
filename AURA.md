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

You can start the `ingester` service using one of the methods:

**Option 1:** Run the following Docker Compose command:

```bash
docker compose -f docker-compose.yaml up -d ingester
```

**Option 2:** Use the Makefile to start the service:

Download the `Makefile` from [GitHub](https://github.com/metaplex-foundation/aura/blob/main/Makefile) and run:

```bash
make start
```

### Verify `ingester` Startup

To verify that the `ingester` service has started correctly, you have these options:

**Option 1:** Manually verify the metrics:

- Check the value of `ingester_buffers{name="buffer_transactions"}` is 0 using:

  ```bash
  curl -s localhost:9091/metrics | grep 'ingester_buffers{name="buffer_transactions"}' | awk '{print $2}'
  ```

- Check that the value of `ingester_processed_total{name="accounts_dynamic_merge_with_batch",status="SUCCESS"}` is increasing over time using:

  ```bash
  curl -s localhost:9091/metrics | grep 'ingester_processed_total{name="accounts_dynamic_merge_with_batch",status="SUCCESS"}' | awk '{print $2}'
  ```

**Option 2:** Use the Makefile to automate verification:

```bash
make check-ingester
```

The process will complete with the message "Container ingester has successfully started and is operating correctly."

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

You can start the synchronizer using one of the methods:

**Option 1:** Run the following Docker Compose command:

```bash
docker compose -f docker-compose.yaml up -d synchronizer
```

**Option 2:** Use the Makefile to start the service:

```bash
make start-synchronizer
```

### Synchronization Check

**Option 1:** Manually check the synchronization status:

Use the following command to continuously check the synchronization status until the slot difference is below the threshold:

```bash
INGESTER_RPC_HOST=$(docker exec ingester env | grep INGESTER_RPC_HOST | cut -d "=" -f2)
docker exec -it synchronizer sh -c '
while true; do
  solana_slot=$(curl -s -X POST -H "Content-Type: application/json" -d '''{"jsonrpc": "2.0","id": 1,"method": "getSlot","params": [{"commitment": "processed"}]}''' $INGESTER_RPC_HOST | grep -oP "(?<=\"result\":)[0-9]+");
  synchronizer_slot=$(curl -s localhost:$SYNCHRONIZER_METRICS_PORT/metrics | grep "synchronizer_last_synchronized_slot{name=\"last_synchronized_slot\"}" | awk '''{print $2}''');
  difference=$((solana_slot - synchronizer_slot));
  echo "Solana slot: $solana_slot";
  echo "Synchronizer last synchronized slot: $synchronizer_slot";
  echo "Difference: $difference slots";
  if [ "$difference" -lt 50 ]; then
    echo "Slot difference is below threshold";
    break;
  fi;
  sleep 5;
done'
```

**Option 2:** Use the Makefile to check the synchronization status. This should complete with the message "Slot difference is below threshold":

```bash
make check-synchronizer
```

## 8. Start the API

You can start the `das-api` service using one of the methods:

**Option 1:** Run the following Docker Compose command:

```bash
docker compose -f docker-compose.yaml up -d das-api
```

**Option 2:** Use the Makefile to start the service:

```bash
make start-api
```

### API Health Check

Use this command to verify the status of the `das-api` service:

```bash
curl -s -X POST -H "Content-Type: application/json" -d '{"jsonrpc": "2.0","id": 1,"method": "health"}' http://api_address:api_server_port
```

Make sure the service returns a healthy status before proceeding.

