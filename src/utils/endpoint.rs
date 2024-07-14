use csv::ReaderBuilder;
use regex::Regex;
use serde::Deserialize;
use std::{
    fs::File,
    io::{self, Write},
};

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(rename = "IP:PORT")]
    endpoint: String,
    #[serde(rename = "LOSS")]
    loss: String,
    #[serde(rename = "DELAY")]
    delay: String,
}

#[allow(dead_code)]
pub fn get_endpoints_from_file(file_path: &str, delay_number: u16) -> Vec<String> {
    let file = File::open(file_path);
    if file.is_err() {
        return Vec::new();
    }
    let file = file.unwrap();
    // trim(csv::Trim::All)：去除字段名和数据项两端的所有空格
    let mut rdr = ReaderBuilder::new().trim(csv::Trim::All).from_reader(file);
    let mut endpoints = Vec::new();
    for result in rdr.deserialize::<Record>() {
        if let Ok(record) = result {
            // 丢包率等于0%且延迟小于delay_number的endpoint添加到endpoints中
            if record.loss == "0.00%" {
                if let Some(ms_value) = record.delay.strip_suffix(" ms") {
                    if let Ok(ms) = ms_value.parse::<u16>() {
                        // 延迟delay_number（延迟，单位：ms）
                        if ms < delay_number.into() {
                            endpoints.push(record.endpoint);
                        }
                    }
                }
            }
        }
    }

    endpoints
}

#[allow(dead_code)]
pub fn get_endpoint_from_user() -> (String, String, u16) {
    let re = Regex::new(
        r"(?x)
        (?P<ipv4>(?:\d{1,3}\.){3}\d{1,3}):(?P<port>\d+) |  # IPv4:port
        \[(?P<ipv6>[a-fA-F0-9:]+)\]:(?P<port_v6>\d+)       # [IPv6]:port
    ",
    )
    .unwrap();

    let mut endpoint: String = String::new();
    let mut ip: String;
    let port: u16;

    loop {
        // 捕捉用户输入
        print!("请输入一个IPv4:PORT或[IPv6]:PORT: ");
        io::stdout().flush().unwrap(); // 确保提示立即显示
        endpoint.clear(); // 清空之前的输入内容
        io::stdin().read_line(&mut endpoint).expect("无法读取输入");
        let endpoint_trim = endpoint.trim();

        // 使用正则表达式提取IP和端口
        if let Some(captures) = re.captures(endpoint_trim) {
            if let Some(ipv4) = captures.name("ipv4") {
                ip = ipv4.as_str().to_string();
                if let Ok(p) = captures.name("port").unwrap().as_str().parse::<u16>() {
                    port = p.clone();
                    break;
                }
            } else if let Some(ipv6) = captures.name("ipv6") {
                ip = ipv6.as_str().to_string();
                if let Ok(p) = captures.name("port_v6").unwrap().as_str().parse::<u16>() {
                    port = p;
                    break;
                }
            }
        }
    }
    (endpoint, ip, port)
}

// 分割出IP和PORT,支持合法的IPv6:PORT分割出IP和PORT，并返回IP和PORT
#[allow(dead_code)]
pub fn split_ip_and_port(endpoint: &str) -> (&str, i32) {
    let mut parts = endpoint.rsplitn(2, ':');
    let port = parts
        .next()
        .unwrap_or("2408")
        .parse()
        .unwrap_or_else(|_| 2408);
    let ip = parts
        .next()
        .unwrap_or("")
        .trim_start_matches('[')
        .trim_end_matches(']');

    (ip, port)
}
