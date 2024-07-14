mod utils;

use clipboard::{ClipboardContext, ClipboardProvider};
use rand::Rng;
use serde_json::{json, Value as json_value};
use std::{collections::HashSet, fs};
use toml::Value as toml_value;

/* 该程序：可以连续捕捉用户输入的endpoint，并支持将之前输入的所有endpoint汇总起来(去重)，生成在一个 Hiddify 配置（warp in warp） */
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 读取TOML文件内容
    let toml_str = fs::read_to_string("config.toml")?;
    // 解析TOML字符串
    let toml_value: toml_value = toml_str.parse::<toml_value>()?;

    let outbounds = r#"{"outbounds": []}"#;
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

    let mut outbounds_value: json_value = serde_json::from_str(outbounds)?;
    let json_data: json_value = serde_json::from_str(json_str)?;
    let mut set: HashSet<String> = HashSet::new();
    if let Some(warp_accounts) = toml_value.get("warp_account").and_then(|v| v.as_array()) {
        let warp_accounts_len = warp_accounts.len();
        loop {
            let mut json_data_clone = json_data.clone();

            let (endpoint, ip, port) = utils::endpoint::get_endpoint_from_user();
            // 确保输入的endpoint不重复，可以连续输入多个endpoint，生成一个由多个endpoint生成hiddify配置
            if set.contains(endpoint.trim()) {
                continue;
            } else {
                set.insert(endpoint.trim().to_string());
            }

            // 初始tag和detour名称，遇到是ipv6的地址，修改成随机名称
            let mut tag_name = format!("warp-{}", endpoint.trim());
            let mut detour_name = format!("warp-{}-detour", endpoint.trim());

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
                detour_name = format!(
                    "warp-v6-{}-{}-detour",
                    ip.split(":").collect::<Vec<&str>>()[2],
                    random_string
                );
            }

            if warp_accounts_len < 2 {
                println!("请在 config.toml 文件中配置 warp_account 字段，至少需要两个账号。");
                std::process::exit(1);
            }

            /*
             *选择其中的两个账号
             */
            let mut rng = rand::thread_rng();
            // 随机生成第一个索引
            let index1 = rng.gen_range(0..warp_accounts_len);
            // 随机生成一个不等于第一个索引的索引
            let mut index2 = rng.gen_range(0..warp_accounts_len);
            while index2 == index1 {
                index2 = rng.gen_range(0..warp_accounts_len);
            }

            // 遍历warp_accounts
            for index in [index1, index2] {
                if let Some(account) = warp_accounts.get(index) {
                    if index == index2 {
                        json_data_clone["tag"] = json!(detour_name);
                        json_data_clone["detour"] = json!(tag_name);
                    } else {
                        json_data_clone["tag"] = json!(tag_name);
                        // 删除 json_data_clone 中的字段detour
                        if let Some(obj) = json_data_clone.as_object_mut() {
                            obj.remove("detour");
                        }
                    }

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

            // 格式化（缩进）打印修改后的 JSON 对象
            let formatted_json = serde_json::to_string_pretty(&outbounds_value)?;
            // 将结果复制到剪切板
            let mut ctx: ClipboardContext = ClipboardProvider::new()?;
            ctx.set_contents(formatted_json.to_owned())?;
            println!("{}", formatted_json);
            println!("{:->87}", "");
            println!("Hiddify 的配置已复制到剪切板，请粘贴到 Hiddify 客户端中使用。");
            println!("{:->87}", "");
        }
    }

    Ok(())
}
