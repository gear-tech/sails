#!/usr/bin/env python3
"""Static/probabilistic analysis for VFT static storage layouts.

This script does not execute a Gear program. It mirrors the hash/index shape
used by sails-storage WAT-shaped VFT maps, simulates linear probing for loaded
tables, estimates lazy-page touches for transfer-like operations, and can count
Wasm instructions from a WAT file or a Wasm binary via `wasm2wat`.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
from collections import Counter
from dataclasses import dataclass
from typing import Iterable


MASK64 = (1 << 64) - 1
HASH_GOLDEN_RATIO = 0x9E37_79B9
HASH_PAIR_SPENDER = 0x85EB_CA6B
LCG_MUL = 6364136223846793005
LCG_ADD = 1442695040888963407
OWNER_DOMAIN = 0xA11C_E001_D15C_A11C
SPENDER_DOMAIN = 0x5EED_5EED_A110_CAFE


def actor_bytes(actor: int) -> bytes:
    data = bytearray(32)
    data[12:20] = max(actor, 1).to_bytes(8, "little")
    return bytes(data)


def random_u64(seed: int, domain: int) -> int:
    state = ((seed + domain) & MASK64)
    state = (state * LCG_MUL + LCG_ADD) & MASK64
    return (state ^ (state >> 32)) & MASK64


def actor_for_seed(seed: int) -> bytes:
    return actor_bytes(random_u64(seed, OWNER_DOMAIN))


def spender_for_seed(seed: int) -> bytes:
    return actor_bytes(random_u64(seed, SPENDER_DOMAIN))


def fold_words(data: bytes) -> int:
    out = 0
    for offset in range(0, 32, 4):
        out ^= int.from_bytes(data[offset : offset + 4], "little")
    return out & 0xFFFF_FFFF


def hash_actor_current(key: bytes) -> int:
    return (fold_words(key) * HASH_GOLDEN_RATIO) & 0xFFFF_FFFF


def fmix32(hash_value: int) -> int:
    hash_value ^= hash_value >> 16
    hash_value = (hash_value * 0x85EB_CA6B) & 0xFFFF_FFFF
    hash_value ^= hash_value >> 13
    hash_value = (hash_value * 0xC2B2_AE35) & 0xFFFF_FFFF
    return (hash_value ^ (hash_value >> 16)) & 0xFFFF_FFFF


def hash_actor_mixed(key: bytes) -> int:
    return fmix32(fold_words(key))


def hash_actor(key: bytes, variant: str) -> int:
    if variant == "current":
        return hash_actor_current(key)
    if variant == "mixed":
        return hash_actor_mixed(key)
    raise ValueError(f"unknown actor hash variant: {variant}")


def hash_allowance(owner: bytes, spender: bytes) -> int:
    return (
        fold_words(owner) * HASH_GOLDEN_RATIO
        + fold_words(spender) * HASH_PAIR_SPENDER
    ) & 0xFFFF_FFFF


def index_for_hash(hash_value: int, log2_slots: int) -> int:
    if log2_slots == 0:
        return 0
    return hash_value >> (32 - log2_slots)


def percentile(sorted_values: list[int], numerator: int, denominator: int) -> int:
    if not sorted_values:
        return 0
    index = (len(sorted_values) - 1) * numerator // denominator
    return sorted_values[index]


def summarize(values: Iterable[int]) -> dict[str, float | int]:
    data = sorted(values)
    if not data:
        return {"avg": 0.0, "p50": 0, "p95": 0, "p99": 0, "max": 0}
    return {
        "avg": sum(data) / len(data),
        "p50": percentile(data, 1, 2),
        "p95": percentile(data, 95, 100),
        "p99": percentile(data, 99, 100),
        "max": data[-1],
    }


@dataclass
class LookupTrace:
    found: bool
    probes: int
    pages: set[int]


class LinearTable:
    def __init__(self, log2_slots: int, slot_size: int, page_size: int):
        self.log2_slots = log2_slots
        self.slots = 1 << log2_slots
        self.mask = self.slots - 1
        self.slot_size = slot_size
        self.page_size = page_size
        self.data: dict[int, bytes] = {}

    def _page(self, slot: int) -> int:
        return (slot * self.slot_size) // self.page_size

    def lookup(self, key: bytes, hash_value: int) -> LookupTrace:
        index = index_for_hash(hash_value, self.log2_slots)
        probes = 0
        pages: set[int] = set()
        while probes < self.slots:
            pages.add(self._page(index))
            stored = self.data.get(index)
            probes += 1
            if stored is None:
                return LookupTrace(False, probes, pages)
            if stored == key:
                return LookupTrace(True, probes, pages)
            index = (index + 1) & self.mask
        return LookupTrace(False, probes, pages)

    def insert(self, key: bytes, hash_value: int) -> LookupTrace:
        index = index_for_hash(hash_value, self.log2_slots)
        probes = 0
        pages: set[int] = set()
        while probes < self.slots:
            pages.add(self._page(index))
            stored = self.data.get(index)
            probes += 1
            if stored is None or stored == key:
                self.data[index] = key
                return LookupTrace(stored == key, probes, pages)
            index = (index + 1) & self.mask
        raise RuntimeError("table is full")


def load_tables(args: argparse.Namespace) -> tuple[LinearTable, LinearTable, dict[str, object]]:
    balances = LinearTable(args.balance_log2, 64, args.lazy_page_size)
    allowances = LinearTable(args.allowance_log2, 96, args.lazy_page_size)
    balance_probes: list[int] = []
    allowance_probes: list[int] = []

    for seed in range(1, args.load + 1):
        owner = actor_for_seed(seed)
        spender = spender_for_seed(seed)
        balance_trace = balances.insert(owner, hash_actor(owner, args.actor_hash))
        allowance_key = owner + spender
        allowance_trace = allowances.insert(
            allowance_key, hash_allowance(owner, spender)
        )
        balance_probes.append(balance_trace.probes)
        allowance_probes.append(allowance_trace.probes)

    return balances, allowances, {
        "balance_prepare_probes": summarize(balance_probes),
        "allowance_prepare_probes": summarize(allowance_probes),
    }


def operation_stats(
    balances: LinearTable, allowances: LinearTable, args: argparse.Namespace
) -> dict[str, object]:
    transfer_probes: list[int] = []
    transfer_pages: list[int] = []
    fresh_probes: list[int] = []
    fresh_pages: list[int] = []
    transfer_from_probes: list[int] = []
    transfer_from_pages: list[int] = []
    approve_probes: list[int] = []
    approve_pages: list[int] = []

    for offset in range(args.ops):
        seed = args.sample_seed + offset
        owner = actor_for_seed(seed)
        recipient = actor_for_seed(seed + 1 if seed < args.load else 1)
        fresh = actor_for_seed(args.load + seed + 1)
        spender = spender_for_seed(seed)
        fresh_spender = spender_for_seed(args.load + args.sample_seed + offset + 60_000)

        owner_trace = balances.lookup(owner, hash_actor(owner, args.actor_hash))
        recipient_trace = balances.lookup(recipient, hash_actor(recipient, args.actor_hash))
        transfer_probes.append(owner_trace.probes + recipient_trace.probes)
        transfer_pages.append(len(owner_trace.pages | recipient_trace.pages))

        fresh_trace = balances.lookup(fresh, hash_actor(fresh, args.actor_hash))
        fresh_probes.append(owner_trace.probes + fresh_trace.probes)
        fresh_pages.append(len(owner_trace.pages | fresh_trace.pages))

        allowance_key = owner + spender
        allowance_trace = allowances.lookup(
            allowance_key, hash_allowance(owner, spender)
        )
        transfer_from_probes.append(
            owner_trace.probes + recipient_trace.probes + allowance_trace.probes
        )
        transfer_from_pages.append(
            len(owner_trace.pages | recipient_trace.pages | allowance_trace.pages)
        )

        approve_key = owner + fresh_spender
        approve_trace = allowances.lookup(
            approve_key, hash_allowance(owner, fresh_spender)
        )
        approve_probes.append(approve_trace.probes)
        approve_pages.append(len(approve_trace.pages))

    return {
        "transfer_existing": {
            "probes": summarize(transfer_probes),
            "lazy_pages": summarize(transfer_pages),
        },
        "transfer_fresh": {
            "probes": summarize(fresh_probes),
            "lazy_pages": summarize(fresh_pages),
        },
        "transfer_from": {
            "probes": summarize(transfer_from_probes),
            "lazy_pages": summarize(transfer_from_pages),
        },
        "approve_fresh": {
            "probes": summarize(approve_probes),
            "lazy_pages": summarize(approve_pages),
        },
    }


def wasm_audit(path: str) -> dict[str, object]:
    if not path:
        return {}
    if path.endswith(".wasm"):
        text = subprocess.check_output(["wasm2wat", path], text=True)
    else:
        with open(path, "r", encoding="utf-8") as file:
            text = file.read()

    op_names = [
        "i32.load",
        "i64.load",
        "i32.store",
        "i64.store",
        "i32.add",
        "i64.add",
        "i64.sub",
        "call",
        "br_if",
        "if",
        "loop",
    ]
    counts = {
        op: len(re.findall(r"(?<![\w.$-])" + re.escape(op) + r"(?![\w.$-])", text))
        for op in op_names
    }
    calls = Counter(re.findall(r"\bcall\s+\$([\w.$-]+)", text))
    return {
        "path": os.path.abspath(path),
        "functions": len(re.findall(r"\(func(?:\s+\$[\w.$-]+)?", text)),
        "instructions": counts,
        "top_calls": calls.most_common(20),
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--load", type=int, default=1_000_000)
    parser.add_argument("--balance-log2", type=int, default=21)
    parser.add_argument("--allowance-log2", type=int, default=21)
    parser.add_argument("--actor-hash", choices=["current", "mixed"], default="current")
    parser.add_argument("--ops", type=int, default=4096)
    parser.add_argument("--sample-seed", type=int, default=20_000)
    parser.add_argument("--lazy-page-size", type=int, default=16 * 1024)
    parser.add_argument("--wat", help="Optional .wat or .wasm path for static instruction counts")
    parser.add_argument("--out", help="Optional JSON output path")
    args = parser.parse_args()

    balances, allowances, prepare = load_tables(args)
    result = {
        "config": {
            "load": args.load,
            "balance_log2": args.balance_log2,
            "allowance_log2": args.allowance_log2,
            "actor_hash": args.actor_hash,
            "balance_load_factor": args.load / balances.slots,
            "allowance_load_factor": args.load / allowances.slots,
            "ops": args.ops,
            "lazy_page_size": args.lazy_page_size,
        },
        "prepare": prepare,
        "operations": operation_stats(balances, allowances, args),
        "wasm_audit": wasm_audit(args.wat) if args.wat else {},
    }

    encoded = json.dumps(result, indent=2, sort_keys=True)
    if args.out:
        os.makedirs(os.path.dirname(args.out), exist_ok=True)
        with open(args.out, "w", encoding="utf-8") as file:
            file.write(encoded)
            file.write("\n")
    print(encoded)


if __name__ == "__main__":
    main()
