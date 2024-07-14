mod utils;

use clipboard::{ClipboardContext, ClipboardProvider};
use rand::Rng;
use serde_json::{json, Value as json_value};
use std::{fs, thread, time::Duration};
use toml::Value as toml_value;

/* 该程序的功能，读取csv文件的数据，筛选丢包率为0%，延迟小于1000ms的，生成hiddify配置（普遍warp） */
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = "result.csv";
    let ip_with_port_vec: Vec<String> = utils::endpoint::get_endpoints_from_file(file_path, 1000);

    // 读取TOML文件内容
    let toml_str = fs::read_to_string("config.toml")?;
    // 解析TOML字符串
    let toml_value: toml_value = toml_str.parse::<toml_value>()?;

    let outbounds = r#"{"outbounds": []}"#;
    let mut outbounds_value: json_value = serde_json::from_str(outbounds)?;

    let json_str = r#"{
        "type": "wireguard",
        "tag": "",
        "local_address": [],
        "private_key": "",
        "server": "",
        "server_port": 2408,
        "peer_public_key": "bmXOC+F1FxEMF9dyiK2H5/1SUtzH0JuVo51h2wPfgyo=",
        "reserved": "",
        "mtu": 1280,
        "fake_packets": "5-10"
    }"#;
    let json_data: json_value = serde_json::from_str(json_str)?;
    if let Some(warp_accounts) = toml_value.get("warp_account").and_then(|v| v.as_array()) {
        let mut json_data_clone = json_data.clone();
        let warp_accounts_len = warp_accounts.len();
        // 这里最多获取前100个endpoint
        for endpoint in &ip_with_port_vec {
            let (ip, port) = utils::endpoint::split_ip_and_port(endpoint);
            // 初始tag名称，遇到是ipv6的地址，修改成随机名称
            let mut tag_name = format!("warp-{}", endpoint.trim());
            // 分离出来的ip，如果还有":"，说明是ipv6的地址
            if ip.contains(":") {
                let mut rng = rand::thread_rng();
                let random_string: String = (0..5)
                    .map(|_| {
                        let idx = rng.gen_range(0..62);
                        match idx {
                            0..=25 => (b'A' + idx as u8) as char,         // 大写字母
                            26..=51 => (b'a' + (idx - 26) as u8) as char, // 小写字母
                            52..=61 => (b'0' + (idx - 52) as u8) as char, // 数字
                            _ => unreachable!(),                          // 这个分支永远不会被触发
                        }
                    })
                    .collect();
                // 重新修改tag和detour名称
                tag_name = format!(
                    "warp-v6-{}-{}",
                    ip.split(":").collect::<Vec<&str>>()[2],
                    random_string
                );
            }
            let mut rng = rand::thread_rng();
            let random_index = rng.gen_range(0..=warp_accounts_len);
            if let Some(account) = warp_accounts.get(random_index) {
                json_data_clone["tag"] = json!(tag_name);
                json_data_clone["server"] = json!(ip);
                json_data_clone["server_port"] = json!(port);
                json_data_clone["local_address"] = json!(account.get("local_address"));
                json_data_clone["private_key"] = json!(account.get("private_key"));
                json_data_clone["peer_public_key"] = json!(account.get("public_key"));
                json_data_clone["reserved"] = json!(account.get("reserved"));
                json_data_clone["fake_packets"] = json!(account.get("fake_packets"));

                outbounds_value["outbounds"]
                    .as_array_mut()
                    .unwrap()
                    .push(json_data_clone.clone());
            }
        }
    }
    let formatted_json = serde_json::to_string_pretty(&outbounds_value)?;

    // 将结果复制到剪切板
    let mut ctx: ClipboardContext = ClipboardProvider::new()?;
    ctx.set_contents(formatted_json.to_owned())?;
    println!("{}", formatted_json);
    println!("{:->87}", "");
    println!("Hiddify 的配置已复制到剪切板，请粘贴到 Hiddify 客户端中使用。2秒后自动关闭窗口...");
    thread::sleep(Duration::from_secs(2));
    Ok(())
}
