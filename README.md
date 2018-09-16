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
```
