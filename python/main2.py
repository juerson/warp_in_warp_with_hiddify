import pyperclip
import toml
import json
import re
import os

IPV4_PORT_PATTERN = re.compile(
    r"^((?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)):(\d{1,5})$"
)
IPV6_PORT_PATTERN = re.compile(r"^\[([0-9a-fA-F:]+)\]:(\d{1,5})$")


def extract_ip_port(endpoint: str) -> tuple:
    """
    提取IP和PORT
    :param endpoint:
    :return:
    """
    # 尝试匹配 IPv4 地址和端口
    ipv4_match = re.match(IPV4_PORT_PATTERN, endpoint)
    if ipv4_match:
        (ip, port) = ipv4_match.groups()
        return ip, port

    # 尝试匹配 IPv6 地址和端口
    ipv6_match = re.match(IPV6_PORT_PATTERN, endpoint)
    if ipv6_match:
        (ip, port) = ipv6_match.groups()
        return ip, port

    # 如果没有匹配，则返回 None
    return None, None


def read_file_to_list(file_path: str) -> list:
    lines = []
    try:
        with open(file_path, "r") as file:
            lines = file.readlines()
            lines = [line.strip() for line in lines if line.strip() != ""]
    except Exception as e:
        print(f"Error reading file {file_path}: {e}")
    return lines


if __name__ == "__main__":
    file_path = "ip.txt"
    endpoints: list = read_file_to_list(file_path)
    if len(endpoints) == 0:
        exit()
    config = toml.load("config.toml")
    warp_account: list = config["warp_account"]
    hiddify_dict = {"outbounds": []}
    for endpoint in endpoints[:50]:
        (ip, port) = extract_ip_port(endpoint)
        if ip and port:
            hiddify_list = [
                {
                    "type": "wireguard",
                    "tag": f"WARP-{endpoint}",
                    "local_address": warp_account[0]["local_address"],
                    "private_key": warp_account[0]["private_key"],
                    "server": ip,
                    "server_port": int(port),
                    "peer_public_key": warp_account[0]["public_key"],
                    "reserved": warp_account[0]["reserved"],
                    "mtu": 1280,
                    "fake_packets": warp_account[0]["fake_packets"],
                },
                {
                    "type": "wireguard",
                    "tag": f"WARP-{endpoint}-Detour",
                    "detour": f"WARP-{endpoint}",
                    "local_address": warp_account[1]["local_address"],
                    "private_key": warp_account[1]["private_key"],
                    "server": ip,
                    "server_port": int(port),
                    "peer_public_key": warp_account[1]["public_key"],
                    "reserved": warp_account[1]["reserved"],
                    "mtu": 1280,
                    "fake_packets": warp_account[1]["fake_packets"],
                },
            ]
            hiddify_dict["outbounds"].extend(hiddify_list)
    json_str = json.dumps(hiddify_dict, indent=2)
    print(json_str)
    pyperclip.copy(json_str)
    os.system("pause")
