# For both

- write tests
- decide whether updating DB should be a task inside dnsliar or it should use the external cli tool

Possible future improvements:
- remove hickory_dns dependency
- remove Redis dependency
- make it so bytes over DNS is not possible on this forwarder
- live full configuration reload without dropping requests

# dnsliar

- config should allow the possibility of subscribing to all available filters
- implement commented features
- make resolver per forwarder and run in own thread/task

# redis-ctl

- *is clean for now*
