# wlog
Small application that logs messages with a date into a sqlite db. Useful for logging your daily activities at work.

## Usage

```
# Dump all logs of today
wlog

# Dump all logs of another day
wlog -d "2018-08-01"

# Dump logs and log a new entry for today
wlog -m "Worked on ticket 123"

# Dump logs and log a new entry for another day
wlog -m "Worked on ticket 123" -d "2018-08-01"

# Display logs as desktop notification (only on Linux and MacOS)
wlog -n
```

## Hosting a remote server

This is useful to keep your worklogs in sync across multiple devices.
You can either run the server manually or use the provided docker-image `sawmurai/wlog-server`.

```bash
docker run --init --rm -itp 8001:8001 -e SECRET=<your-api-key> -v /data/:/data/ sawmurai/wlog-server
```

Since the data is sent *without any encryption* I strongly *recommend* to host the server only behind a reverse proxy that
takes care of SSL encryption.

Now, do sync with your remote server, simply run:

```bash
API_KEY=<your-api-key> wlog --remote https://your-server.com
```

This command will send all local logs and fetch all remote logs, ending up with two identical databases.