import pyperclip
import toml
import json
import re
import os

IPV4_PORT_PATTERN = re.compile(
    r"^((?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)):(\d{1,5})$"
)
IPV6_PORT_PATTERN = re.compile(r"^\[([0-9a-fA-F:]+)\]:(\d{1,5})$")


def clear_screen():
    # Windows
    if os.name == "nt":
        os.system("cls")
    # Mac 和 Linux (这里, os.name 是 'posix')
    else:
        os.system("clear")


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


if __name__ == "__main__":
    config = toml.load("config.toml")
    warp_account: list = config["warp_account"]
    while 1:
        endpoint: str = input(
            "Please Enter WARP Endpoint Address (e.g. 162.159.192.1:2408)："
        )
        (ip, port) = extract_ip_port(endpoint)
        if ip and port:
            hiddify_dict: dict = {
                "outbounds": [
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
            }
            # 将字典转换为 JSON 字符串
            # json_str = json.dumps(hiddify_dict)
            json_str = json.dumps(hiddify_dict, indent=2)
            pyperclip.copy(json_str)
            clear_screen()
            print(json_str)
            print(f"{'-' * 100}")
