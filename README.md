# 🚦 **DnsLiar** 🚦

DNS forwarder with blacklist using [Hickory DNS](https://github.com/hickory-dns/hickory-dns) and [Redis-rs](https://github.com/redis-rs/redis-rs) (currently using Valkey as server).

This DNS forwarder **filters queries** using a **blacklist**. It **blocks** DNS requests querying for **unwanted domains** to **protect** its **users** from them.

# 🚀 **Quick start** 🚀

Prerequisites: **git**, **docker**

**Clone** this repository and **make** it:

```sh
git clone https://github.com/Tibso/DnsLiar.git
cd DnsLiar/
make
```

# ⭐ **Guidelines** ⭐

+ **Fast, secure and reliable**
+ **Simple to setup and operate**
+ **Safe Rust only**

# 🎯 **Milestones** 🎯

+ [x] It works
+ [ ] Satisfactory initial blacklist (ficsit~ by suggesting URLs)
+ [ ] Sufficiently stable for a v1.0 release
+ [ ] No Hickory DNS dependency
+ [ ] No Redis dependency
+ [ ] Each thread is fully independent, not sharing a connection object

# ⚙️ **What does it do?** ⚙️

Filters incoming DNS queries based on the server’s bound socket address and the associated blacklists.

If a blacklisted domain is requested or a blacklisted IP is resolved, the request is blocked and an NXDOMAIN response is sent back.
If nothing found, the legitimate response is sent back.

## 🧹 **Filtering** 🧹

The following **query types** are filtered:

+ **A**
+ **AAAA**

**Other** query types are simply **forwarded** to other DNS servers.
The **returned IPs** are still filtered against the **IP blacklist**.

## 📜 **Blacklist rules** 📜

A rule **defines** a **domain** or **IP** that must be blocked.

+ [HASH] DBL;D;porn;tig-biddies.com
+ [HASH] DBL;I;malware;198.51.100.42

  + **enabled** : *1* - ***1** or **0**, indicates an **enabled/disabled** rule.*
  + **date** : *2025-09-05-17:18* - *The **date** when the **rule** was **added** to the blacklist.*
  + **src** : *firebog* - *The **source** where the rule **originates** from.*

# 🛠️ **Redis-ctl** 🛠️

***Note**: This tool still contains parts of a temporarily removed feature, the handling of v4 and v6 separately for domains.*

```
This is a command-line tool used to manipulate the Redis blacklist

Usage: redis-ctl <PATH_TO_CONFILE> <COMMAND>

Commands:
  add-domain           Add a new domain rule
  remove-domain        Delete a domain rule or either of its v4 or v6 IPs
  search-rules         Search rules by pattern
  disable-rules        Disable rules by pattern
  enable-rules         Enable rules by pattern
  add-ips              Add new IP rules
  remove-ips           Remove IP rules
  feed-filter          Feed rules to a filter from a file
  feed-from-downloads  Feed rules from downloads
  help                 Print this message or the help of the given subcommand(s)

Arguments:
  <PATH_TO_CONFILE>  Path to dnsliar.conf is required

Options:
  -h, --help  Print help
```
