# For both

- decide whether updating DB should be a task inside dnsliar or it should use the external ctl tool

Possible future improvements:
- remove hickory_dns dependency
- remove Redis dependency
- live full configuration reload without dropping requests

# dnsliar

- TTL and retention time should handle definition such as "1d","1m","1y"
- add a debug level of log? configure a default level in docker-compose
- config should allow the possibility of subscribing to all available filters
- graceful shutdown
- implement commented features
- make resolver per forwarder and run in own thread/task

# redis-ctl

- *is clean for now*
