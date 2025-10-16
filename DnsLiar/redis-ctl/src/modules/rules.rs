use dnsliar::config::time_abrv_to_secs;

use redis::{Commands, Connection, RedisResult, pipe};
use serde::Deserialize;
use std::{
    fs, io::{BufRead, BufReader, Cursor},
    net::IpAddr, path::PathBuf, process::ExitCode
};

use super::{get_date, is_valid_domain, is_public_ip, has_redis_wildcard};

#[derive(Deserialize)]
struct SourcesLists {
    name: String,
    lists: Vec<List>,
}
#[derive(Deserialize)]
struct List {
    filter: String,
    urls: Vec<String>,
}

/// Add a new rule
pub fn add(
    con: &mut Connection,
    filter: &str,
    item: &str,
    src: Option<&str>,
    ttl: Option<&str>
) -> RedisResult<ExitCode> {
    let key = if is_valid_domain(item) {
        format!("DBL;D;{filter};{item}")
    } else if item.parse::<IpAddr>().is_ok() {
        format!("DBL;I;{filter};{item}")
    } else {
        println!("ERR: Provided item is not a valid domain or IP\n");
        return Ok(ExitCode::from(65));
    };

    let items = [("enabled", "1"), ("date", &get_date()), ("src", src.unwrap_or("custom"))];
    let () = con.hset_multiple(&key, &items)?;

    if let Some(ttl) = ttl {
        let secs_to_expiry = match time_abrv_to_secs(ttl) {
            Err(e) => {
                println!("ERR: {e}\n");
                return Ok(ExitCode::from(65))
            },
            Ok(secs) => secs
        };
        let () = con.expire(&key, secs_to_expiry as i64)?;
    }

    println!("Rule added: {key}\n");
    Ok(ExitCode::SUCCESS)
}

/// Delete a rule
pub fn remove(
    con: &mut Connection,
    filter: &str,
    item: &str,
) -> RedisResult<ExitCode> {
    let key = if is_valid_domain(item) {
        format!("DBL;D;{filter};{item}")
    } else if item.parse::<IpAddr>().is_ok() {
        format!("DBL;I;{filter};{item}")
    } else {
        println!("ERR: Provided item is not a valid domain or IP\n");
        return Ok(ExitCode::from(65));
    };

    let () = con.del(&key)?;
    println!("Rule deleted: {key}\n");
    Ok(ExitCode::SUCCESS)
}

/// Search for rules using a pattern
pub fn search(
    con: &mut Connection,
    pattern: &str,
    filter: Option<&str>
) -> RedisResult<ExitCode> {
    let filter = filter.unwrap_or("*");
    if has_redis_wildcard(filter) || has_redis_wildcard(pattern) {
        let search_q = format!("DBL;[DI];{filter};{pattern}");
        let keys: Vec<String> = con.scan_match(&search_q)?.collect();
        if keys.is_empty() {
            println!("No match for: {search_q}\n");
        } else {
            for key in keys {
                let values: Vec<String> = con.hgetall(&key)?;
                println!("{key}\n{values:?}\n");
            }
        }
        return Ok(ExitCode::SUCCESS);
    }

    let key = if is_valid_domain(pattern) {
        format!("DBL;D;{filter};{pattern}")
    } else if pattern.parse::<IpAddr>().is_ok() {
        format!("DBL;I;{filter};{pattern}")
    } else {
        println!("ERR: Provided item is not a valid domain or IP\n");
        return Ok(ExitCode::from(65));
    };
    let values: Vec<String> = con.hgetall(&key)?;
    if values.is_empty() {
        println!("Key not found\n");
    } else {
        println!("{key}\n{values:?}\n");
    }
    Ok(ExitCode::SUCCESS)
}

/// Disable/enable rules using a pattern
pub fn enabled(
    con: &mut Connection,
    pattern: &str,
    filter: Option<&str>,
    enabled: bool
) -> RedisResult<ExitCode> {
    let filter = filter.unwrap_or("*");
    if has_redis_wildcard(filter) || has_redis_wildcard(pattern) {
        let pattern = format!("DBL;[DI];{filter};{pattern}");
        let keys: Vec<String> = con.scan_match(&pattern)?.collect();
        if keys.is_empty() {
            println!("No match for: {pattern}\n");
        } else {
            let mut modified_acc: u64 = 0;
            for key in keys {
                let () = con.hset(&key, "enabled", enabled)?;
                modified_acc += 1;
            }
            println!("{modified_acc} rule(s) were {}", if enabled { "enabled" } else { "disabled" });
        }
        return Ok(ExitCode::SUCCESS);
    }

    let key = if is_valid_domain(pattern) {
        format!("DBL;D;{filter};{pattern}")
    } else if pattern.parse::<IpAddr>().is_ok() {
        format!("DBL;I;{filter};{pattern}")
    } else {
        println!("ERR: Provided item is not a valid domain or IP\n");
        return Ok(ExitCode::from(65));
    };
    let () = con.hset(&key, "enabled", enabled)?;
    println!("Key {}: {key}", if enabled { "enabled" } else { "disabled" });
    Ok(ExitCode::SUCCESS)
}

/// This functions expects to read files formatted as /etc/hosts but its lenient
fn feed_from_reader<R: BufRead>(
    con: &mut Connection,
    reader: R,
    filter: &str,
    src: &str
) -> RedisResult<ExitCode> {
    let date = get_date();
    let fields = [("enabled", "1"), ("date", &date), ("src", src)];

    let mut lines_acc: u64 = 0;
    let mut q_sent_acc: u64 = 0;

    let mut pipe = pipe();
    'lines: for bytes in reader.split(b'\n') {
        lines_acc += 1;

        let Ok(bytes) = bytes else {
            println!("ERR: Could not buffer file content: Bad EOF/IO");
            break;
        };

        if !bytes.is_ascii() {
            continue;
        }
        let Ok(line) = core::str::from_utf8(&bytes) else {
            continue; // not fastest conversion but safe and has ascii optimizations -- forbid unsafe
        };

        // need to handle comments and adblock filter list format
        let comments = ['#', '!', '%'];
        let trimmed_line = line.trim_matches(|c| c == ' ' || c == '^' || c == '|');
        if trimmed_line.starts_with(comments) {
            continue;
        }

        for item in trimmed_line.split_whitespace() {
            if item.starts_with(comments) {
                continue 'lines;
            }

            if let Ok(ip) = item.parse::<IpAddr>() && is_public_ip(&ip) {
                let key = format!("DBL;I;{filter};{item}");
                pipe.hset_multiple(&key, &fields);
            } else if is_valid_domain(item) {
                let key = format!("DBL;D;{filter};{item}");
                pipe.hset_multiple(&key, &fields);
            }

            let pipe_len = pipe.len();
            #[allow(clippy::collapsible_if)]
            if pipe_len >= 10000 {
                if let Err(e) = pipe.exec(con) {
                    println!("WARN: {e}\nWARN: {pipe_len} queries left in pipe queue");
                } else {
                    q_sent_acc += pipe_len as u64;
                    pipe.clear();
                }
            }
        }
    }
    #[allow(clippy::collapsible_if)]
    if !pipe.is_empty() {
        if let Err(e) = pipe.exec(con) {
            println!("ERR: {e}\nERR: {} queries lost", pipe.len());
        } else {
            q_sent_acc += pipe.len() as u64;
        }
    }
    println!("{q_sent_acc} queries to DB | {lines_acc} lines parsed");
    Ok(ExitCode::SUCCESS)
}

/// This functions expects to read files formatted as /etc/hosts but its lenient
fn feed_from_reader_ttl<R: BufRead>(
    con: &mut Connection,
    reader: R,
    filter: &str,
    src: &str,
    ttl: &str
) -> RedisResult<ExitCode> {
    let date = get_date();
    let secs_to_expiry = match time_abrv_to_secs(ttl) {
        Err(e) => {
            println!("ERR: {e}\n");
            return Ok(ExitCode::from(65))
        },
        Ok(secs) => secs
    };
    let fields = [("enabled", "1"), ("date", &date), ("src", src)];

    let mut lines_acc: u64 = 0;
    let mut q_sent_acc: u64 = 0;

    let mut pipe = pipe();
    'lines: for bytes in reader.split(b'\n') {
        lines_acc += 1;

        let Ok(bytes) = bytes else {
            println!("ERR: Could not buffer file content: Bad EOF/IO");
            break;
        };

        if !bytes.is_ascii() {
            continue;
        }
        let Ok(line) = core::str::from_utf8(&bytes) else {
            continue; // not fastest conversion but safe and has ascii optimizations -- forbid unsafe
        };

        // need to handle comments and adblock filter list format
        let comments = ['#', '!', '%'];
        let trimmed_line = line.trim_matches(|c| c == ' ' || c == '^' || c == '|');
        if trimmed_line.starts_with(comments) {
            continue;
        }

        for item in trimmed_line.split_whitespace() {
            if item.starts_with(comments) {
                continue 'lines;
            }

            if let Ok(ip) = item.parse::<IpAddr>() && is_public_ip(&ip) {
                let key = format!("DBL;I;{filter};{item}");
                pipe.hset_multiple(&key, &fields)
                    .expire(&key, secs_to_expiry as i64);
            } else if is_valid_domain(item) {
                let key = format!("DBL;D;{filter};{item}");
                pipe.hset_multiple(&key, &fields)
                    .expire(&key, secs_to_expiry as i64);
            }

            let pipe_len = pipe.len();
            #[allow(clippy::collapsible_if)]
            if pipe_len >= 10000 {
                if let Err(e) = pipe.exec(con) {
                    println!("WARN: {e}\nWARN: {pipe_len} queries left in pipe queue");
                } else {
                    q_sent_acc += pipe_len as u64;
                    pipe.clear();
                }
            }
        }
    }
    #[allow(clippy::collapsible_if)]
    if !pipe.is_empty() {
        if let Err(e) = pipe.exec(con) {
            println!("ERR: {e}\nERR: {} queries lost", pipe.len());
        } else {
            q_sent_acc += pipe.len() as u64;
        }
    }
    println!("{q_sent_acc} queries to DB | {lines_acc} lines parsed");
    Ok(ExitCode::SUCCESS)
}

/// Feed the blacklist using a list of blacklist sources such as in the `blacklist_sources.json` file
pub fn feed_from_downloads(
    con: &mut Connection,
    path_to_file: &PathBuf,
    ttl: Option<&str>,
) -> RedisResult<ExitCode> {
    let data = match fs::read_to_string(path_to_file) {
        Err(e) => {
            println!("Error reading \"{path_to_file:?}\": {e}");
            return Ok(ExitCode::from(66)); // EX_NOINPUT
        }
        Ok(data) => data,
    };

    let srcs_list: Vec<SourcesLists> = match serde_json::from_str(&data) {
        Err(e) => {
            println!("Error deserializing \"{path_to_file:?}\" data: {e}");
            return Ok(ExitCode::from(65)); // EX_DATAERR
        }
        Ok(srcs_list) => srcs_list,
    };

    let http_client = reqwest::blocking::Client::new();
    for src in srcs_list {
        for list in src.lists {
            for url in list.urls {
                println!("Fetching: {url}");
                let resp = match http_client.get(&url).send() {
                    Err(e) => {
                        println!("Error retrieving data: {e}\nSkipping...");
                        continue;
                    }
                    Ok(resp) => resp,
                };

                if !resp.status().is_success() {
                    println!(
                        "Error {}: Request was not successful\nSkipping...",
                        resp.status()
                    );
                    continue;
                }

                let cl = resp.content_length().map(|len| len.to_string())
                    .unwrap_or_else(|| "---Missing header---".to_string());
                println!("Retrieved content length: {cl} | Parsing and feeding DB...");
                let Ok(bytes) = resp.bytes() else {
                    continue;
                };

                let reader = BufReader::new(Cursor::new(bytes));
                if let Some(ttl) = ttl {
                    feed_from_reader_ttl(con, reader, &list.filter, &src.name, ttl)?;
                } else {
                    feed_from_reader(con, reader, &list.filter, &src.name)?;
                }
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// Feed a list to a filter
pub fn feed_filter(
    con: &mut Connection,
    path_to_file: &PathBuf,
    filter: &str,
    src: Option<&str>,
    ttl: Option<&str>
) -> RedisResult<ExitCode> {
    let file = fs::File::open(path_to_file)?;
    let reader = BufReader::new(file);
    if let Some(ttl) = ttl {
        feed_from_reader_ttl(con, reader, filter, src.unwrap_or("custom"), ttl)?;
    } else {
        feed_from_reader(con, reader, filter, src.unwrap_or("custom"))?;
    }
    Ok(ExitCode::SUCCESS)
}

// /// Add a new domain rule
// pub fn add_domain(
//     con: &mut Connection,
//     filter: &str,
//     src: &str,
//     domain: &str,
//     ttl: &str,
//     ip1: Option<String>,
//     ip2: Option<String>,
// ) -> RedisResult<ExitCode> {
//     if !is_valid_domain(domain) {
//         println!("ERR: Given domain is invalid");
//         return Ok(ExitCode::from(65));
//     }
//     let Some(secs_to_expiry) = time_abrv_to_secs(ttl) else {
//         println!("ERR: Given TTL is not properly formatted or is too big");
//         return Ok(ExitCode::from(65));
//     };
// 
//     let mut pipe = pipe();
//     let key = format!("DBL;D;{filter};{domain}");
//     let fields = [("enabled", "1"), ("date", &get_date()), ("src", src)];
//     match (ip1, ip2) {
//         (None, None) => {
//             println!("No IP provided, adding domain rule for both v4 and v6");
//             pipe.hset_multiple(
//                 &key,
//                 &[fields[0], fields[1], fields[2], ("A", "1"), ("AAAA", "1")],
//             );
//         }
//         (Some(ip1), Some(ip2)) => match (ip1.as_str(), ip2.as_str()) {
//             ("A", "AAAA") | ("AAAA", "A") => {
//                 pipe.hset_multiple(
//                     &key,
//                     &[fields[0], fields[1], fields[2], ("A", "1"), ("AAAA", "1")],
//                 );
//             }
//             ("A", ip) | (ip, "A") => {
//                 if ip.parse::<Ipv6Addr>().is_err() {
//                     println!("ERR: IP parsed was not IPv6");
//                     return Ok(ExitCode::from(65));
//                 }
//                 pipe.hset_multiple(
//                     &key,
//                     &[fields[0], fields[1], fields[2], ("A", "1"), ("AAAA", ip)],
//                 );
//             }
//             ("AAAA", ip) | (ip, "AAAA") => {
//                 if ip.parse::<Ipv4Addr>().is_err() {
//                     println!("ERR: IP parsed was not IPv4");
//                     return Ok(ExitCode::from(65));
//                 }
//                 pipe.hset_multiple(
//                     &key,
//                     &[fields[0], fields[1], fields[2], ("A", ip), ("AAAA", "1")],
//                 );
//             }
//             _ => {
//                 if let (Ok(ip1), Ok(ip2)) = (ip1.parse::<IpAddr>(), ip2.parse::<IpAddr>()) {
//                     match (ip1, ip2) {
//                         (IpAddr::V4(ipv4), IpAddr::V6(ipv6))
//                         | (IpAddr::V6(ipv6), IpAddr::V4(ipv4)) => {
//                             pipe.hset_multiple(
//                                 &key,
//                                 &[
//                                     fields[0],
//                                     fields[1],
//                                     fields[2],
//                                     ("A", &ipv4.to_string()),
//                                     ("AAAA", &ipv6.to_string()),
//                                 ],
//                             );
//                         }
//                         _ => {
//                             println!("ERR: Provided IPs cannot both be v4 or v6");
//                             return Ok(ExitCode::from(65));
//                         }
//                     }
//                 } else {
//                     println!("ERR: Could not parse provided IPs");
//                     return Ok(ExitCode::from(65));
//                 }
//             }
//         },
//         (Some(ip), None) => {
//             if matches!(ip.as_str(), "A" | "AAAA") {
//                 pipe.hset_multiple(&key, &[fields[0], fields[1], fields[2], (&ip, "1")]);
//             } else if let Ok(ip) = ip.parse::<IpAddr>() {
//                 let ip_field: (&str, &str) = match ip {
//                     IpAddr::V4(ipv4) => ("A", &ipv4.to_string()),
//                     IpAddr::V6(ipv6) => ("AAAA", &ipv6.to_string()),
//                 };
//                 pipe.hset_multiple(&key, &[fields[0], fields[1], fields[2], ip_field]);
//             } else {
//                 println!("ERR: Could not parse provided IP");
//                 return Ok(ExitCode::from(65));
//             }
//         }
//         _ => unreachable!(),
//     }
// 
//     pipe.expire(key, secs_to_expiry).exec(con)?;
//     println!("Domain rule added");
//     Ok(ExitCode::SUCCESS)
// }
// 
// /// Delete a domain rule or only one IP version
// pub fn remove_domain(
//     con: &mut Connection,
//     filter: &str,
//     domain: &str,
//     ip_ver: Option<u8>,
// ) -> RedisResult<ExitCode> {
//     if !is_valid_domain(domain) {
//         println!("ERR: Given domain is invalid");
//         return Ok(ExitCode::from(65));
//     }
// 
//     let key = format!("DBL;D;{filter};{domain}");
//     let del_cnt: u64 = match ip_ver {
//         None => {
//             println!("No IP version provided, deleting domain rule");
//             con.del(key)?
//         }
//         Some(ip_ver) => {
//             let q_type = match ip_ver {
//                 4 => "A",
//                 6 => "AAAA",
//                 _ => {
//                     println!("ERR: Given IP version is invalid");
//                     return Ok(ExitCode::from(65));
//                 }
//             };
//             con.hdel(key, q_type)?
//         }
//     };
//     match del_cnt {
//         1 => println!("Rule deleted"),
//         _ => println!("Nothing deleted, are you sure this rule exists?"),
//     }
//     Ok(ExitCode::SUCCESS)
// }
