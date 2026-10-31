#!/usr/bin/env python3
"""Check TAPE genesis, economics and native runtime in an isolated non-mining node."""

import argparse
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request


def free_port():
    with socket.socket() as listener:
        listener.bind(("127.0.0.1", 0))
        return listener.getsockname()[1]


def check(node, timeout):
    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
    with tempfile.TemporaryDirectory(prefix="retro-boomer-release-smoke-") as directory:
        root = Path(directory)
        data = root / "data"
        rpc_port = free_port()
        endpoint = f"http://127.0.0.1:{rpc_port}"

        def rpc(method):
            body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "retro_boomer_" + method, "params": []}).encode()
            request = urllib.request.Request(endpoint, data=body, headers={"Content-Type": "application/json"})
            with opener.open(request, timeout=2) as response:
                result = json.load(response)
            if "error" in result:
                raise ValueError(f"{method}: {result['error']}")
            return result["result"]

        with (root / "node.log").open("w+", encoding="utf-8") as log:
            process = subprocess.Popen([
                str(node), "--config", str(root / "node.toml"),
                "--data-dir", str(data), "--mode", "node", "--disable-dns-seeds",
                "--p2p-listen", "127.0.0.1:0", "--rpc-listen", f"127.0.0.1:{rpc_port}",
                "--log", "warn",
            ], stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT)
            try:
                deadline = time.monotonic() + timeout
                while True:
                    if process.poll() is not None:
                        raise ValueError(f"release daemon exited with status {process.returncode}")
                    try:
                        status = rpc("getNodeStatus")
                        break
                    except (urllib.error.URLError, TimeoutError, ConnectionError):
                        if time.monotonic() >= deadline:
                            raise ValueError("release daemon RPC startup timed out")
                        time.sleep(0.25)
                protocol = rpc("getNativeProtocol")
                chain = rpc("getChainInfo")
                if status["testnet"] or status["mining"] or status["p2p_connected_peers"]:
                    raise ValueError("release smoke node must be isolated mainnet with mining off")
                if chain["height"] != 0 or protocol["tip_height"] != 0:
                    raise ValueError("release smoke node must start from a fresh chain")
                if not protocol["runtime_available"] or protocol["activation_height"] != 1 or not protocol["active_at_next_block"]:
                    raise ValueError("TAPE block-one native runtime is missing")
                genesis = "cede1246070229c7d312f341a27d801dbf5b7bf32f8c76552d21f9203b325a13"
                if chain["best_hash"] != genesis or chain["ticker"] != "TAPE" or chain["protocol"] != "V1" or chain["chain_name"] != "Retro Boomer":
                    raise ValueError("wrong chain identity")
                if chain["decimals"] != 6 or chain["target_block_seconds"] != 60:
                    raise ValueError("wrong TAPE precision or block interval")
                if chain["maximum_supply_units"] != "10000000000000000" or chain["scheduled_issuance_units"] != "10000000000000000":
                    raise ValueError("wrong maximum or scheduled issuance")
                if chain["current_subsidy_units"] != 100_000_000_000 or chain["first_zero_subsidy_height"] != 10_230_001:
                    raise ValueError("wrong initial or final subsidy")
                expected_stages = [{"first_height": 10_000 * ((1 << i) - 1) + 1,
                                    "last_height": 10_000 * ((1 << (i + 1)) - 1),
                                    "block_count": 10_000 << i, "reward_units": 100_000_000_000 >> i,
                                    "issuance_units": "1000000000000000"} for i in range(10)]
                if chain["subsidy_stage_count"] != 10 or chain["subsidy_stages"] != expected_stages or chain["circulating_supply_micronoid"] != "0":
                    raise ValueError("wrong issuance stages or genesis supply")
                return {"native_runtime": "ready", "activation_height": 1, "genesis": genesis,
                        "isolated": True, "mining": False, "issuance_units": "10000000000000000",
                        "bank_check": "Authenticated by build.rs; bank digest is not exposed by this RPC."}
            except Exception:
                log.flush()
                log.seek(0)
                print(log.read()[-8000:])
                raise
            finally:
                if process.poll() is None:
                    if os.name == "nt":
                        process.terminate()
                    else:
                        process.send_signal(signal.SIGINT)
                    try:
                        process.wait(timeout=20)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait(timeout=10)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--node", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=float, default=120)
    args = parser.parse_args()
    try:
        result = check(args.node.resolve(), args.timeout)
    except (OSError, ValueError, KeyError) as error:
        parser.exit(1, f"release node check: {error}\n")
    args.output.write_text(json.dumps(result, sort_keys=True, indent=2) + "\n", encoding="utf-8")
    print("Release node check passed: " + json.dumps(result, sort_keys=True))


if __name__ == "__main__":
    main()
