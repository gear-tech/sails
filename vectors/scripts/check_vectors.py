#!/usr/bin/env python3
"""Verify canonical vectors using Python.

Requires:
    pip install blake3 canonicaljson
"""

from __future__ import annotations

import json
from pathlib import Path

from blake3 import blake3
from canonicaljson import encode_canonical_json

DOMAIN = b"SAILS-IDL/v1/interface-id"
CANONICAL_DIR = Path(__file__).resolve().parent.parent / "canonical"


def load_manifest() -> dict[str, dict[str, str]]:
    with (CANONICAL_DIR / "manifest.json").open("r", encoding="utf-8") as f:
        return json.load(f)


def canonical_bytes(document: dict) -> bytes:
    return encode_canonical_json(document)


def compute_interface_id(payload: bytes) -> int:
    hasher = blake3()
    hasher.update(DOMAIN)
    hasher.update(payload)
    digest = hasher.digest()
    return int.from_bytes(digest[:8], "little")


def main() -> int:
    errors = 0
    manifest = load_manifest()
    for filename, services in manifest.items():
        with (CANONICAL_DIR / filename).open("r", encoding="utf-8") as f:
            doc = json.load(f)
        for service_name, expected_hex in services.items():
            service = doc.get("services", {}).get(service_name)
            if service is None:
                print(f"Service {service_name} missing in {filename}")
                errors += 1
                continue
            single_doc = {
                "canon_schema": doc.get("canon_schema", "sails-idl-jcs"),
                "canon_version": doc.get("canon_version", doc.get("version", "1")),
                "hash": doc.get(
                    "hash",
                    {"algo": "blake3", "domain": DOMAIN.decode("utf-8")},
                ),
                "services": {service_name: service},
                "types": doc.get("types", {}),
            }
            payload = canonical_bytes(single_doc)
            actual = compute_interface_id(payload)
            expected = int(expected_hex, 16)
            if actual != expected:
                print(
                    f"Mismatch for {service_name} in {filename}: expected {expected_hex}, got 0x{actual:016x}",
                )
                errors += 1
    if errors:
        return 1
    print("Canonical vectors verified successfully.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
