#!/usr/bin/env python3
"""
HTTP smoke test for a running wonky-ord-pepecoin server.

Covers every route registered in src/subcommand/server.rs. Uses live chain
data where needed (from a known inscription id).

Usage:
  ORD_BASE_URL=http://127.0.0.1:8888 python3 scripts/smoke_local_ord.py

  # Speed test (ms per endpoint) + optional address for balance routes:
  ORD_BENCH=1 ORD_TEST_ADDRESS=PepcoinAddr... python3 scripts/smoke_local_ord.py

Exit code 0 if all checks pass; non-zero on failure.
"""

from __future__ import annotations

import os
import sys
import time
import urllib.parse
from dataclasses import dataclass

import requests

DEFAULT_INSCRIPTION = (
    "e5653762a8b801ac09056a0019663bc589a2c3612aa456d81ea7222ffe530508i0"
)


@dataclass
class Case:
    name: str
    method: str
    path: str
    ok_status: frozenset[int]
    note: str = ""


def main() -> int:
    base = os.environ.get("ORD_BASE_URL", "http://127.0.0.1:8888").rstrip("/")
    ins = os.environ.get("ORD_TEST_INSCRIPTION", DEFAULT_INSCRIPTION)
    bench = os.environ.get("ORD_BENCH", "").lower() in ("1", "true", "yes")
    override_addr = os.environ.get("ORD_TEST_ADDRESS", "").strip()
    session = requests.Session()
    session.headers.setdefault("User-Agent", "smoke_local_ord/1.0")

    # Warm up + sample data
    r = session.get(f"{base}/status", timeout=30)
    if r.status_code != 200 or r.text.strip() != "OK":
        print(f"FAIL: /status expected 200 OK body, got {r.status_code} {r.text!r}")
        return 1

    r = session.get(f"{base}/inscription/{ins}?json=true", timeout=60)
    if r.status_code != 200:
        print(f"FAIL: inscription json {r.status_code}")
        return 1
    meta = r.json()
    addr = override_addr or meta.get("address")
    if not addr:
        print("FAIL: set ORD_TEST_ADDRESS or use an inscription with an address")
        return 1
    sp = str(meta.get("satpoint", ""))
    parts = sp.split(":")
    if len(parts) < 2:
        print(f"FAIL: bad satpoint {sp!r}")
        return 1
    outpoint = f"{parts[0]}:{parts[1]}"

    r = session.get(f"{base}/block-count", timeout=30)
    if r.status_code != 200:
        print(f"FAIL: block-count {r.status_code}")
        return 1
    tip = int(r.text.strip())
    h = max(0, tip - 1)

    txid = parts[0]
    enc_addr = urllib.parse.quote(addr, safe="")

    cases: list[Case] = []

    # Pepemap: list is JSON; then hit a real claim number when the index has data.
    r = session.get(f"{base}/pepemap?limit=10&offset=0", timeout=60)
    if r.status_code != 200:
        print(f"FAIL: pepemap index {r.status_code}")
        return 1
    try:
        pepe_list = r.json()
    except Exception as e:
        print(f"FAIL: pepemap index not JSON: {e}")
        return 1
    if not isinstance(pepe_list, list):
        print("FAIL: pepemap index expected JSON array")
        return 1
    pepe_num = None
    if pepe_list:
        first = pepe_list[0]
        if isinstance(first, dict):
            pepe_num = first.get("number")

    cases.extend(
        [
        Case("home", "GET", "/", frozenset({200})),
        Case("status", "GET", "/status", frozenset({200})),
        Case("block_count", "GET", "/block-count", frozenset({200})),
        Case("block_height", "GET", f"/block/{h}", frozenset({200})),
        Case("blocks_range", "GET", f"/blocks/{h}/{h}", frozenset({200})),
        Case("bounties_redirect", "GET", "/bounties", frozenset({302, 303, 307, 308})),
        Case("faq_redirect", "GET", "/faq", frozenset({302, 303, 307, 308})),
        Case("install_redirect", "GET", "/install.sh", frozenset({302, 303, 307, 308})),
        Case("favicon", "GET", "/favicon.ico", frozenset({200})),
        Case("feed", "GET", "/feed.xml", frozenset({200})),
        Case("rare_txt", "GET", "/rare.txt", frozenset({200})),
        Case("inscriptions", "GET", "/inscriptions", frozenset({200})),
        Case("inscriptions_from", "GET", "/inscriptions/0", frozenset({200})),
        Case("inscription_html", "GET", f"/inscription/{ins}", frozenset({200})),
        Case("inscription_json", "GET", f"/inscription/{ins}?json=true", frozenset({200})),
        Case("content", "GET", f"/content/{ins}", frozenset({200})),
        Case("preview", "GET", f"/preview/{ins}", frozenset({200})),
        Case("output", "GET", f"/output/{outpoint}", frozenset({200})),
        Case(
            "outputs_multi",
            "GET",
            f"/outputs/{urllib.parse.quote(outpoint + ',' + outpoint, safe=':,')}",
            frozenset({200}),
        ),
        Case("tx", "GET", f"/tx/{txid}", frozenset({200})),
        Case("tx_json", "GET", f"/tx/{txid}?json=true", frozenset({200})),
        Case("address", "GET", f"/address/{enc_addr}", frozenset({200})),
        Case("input_genesis", "GET", "/input/0/0/0", frozenset({200})),
        Case(
            "ordinal",
            "GET",
            "/ordinal/0",
            frozenset({200, 404, 302, 303, 307, 308}),
        ),
        Case("sat", "GET", "/sat/0", frozenset({200, 404})),
        Case("range", "GET", "/range/0/100", frozenset({200})),
        Case("dunes", "GET", "/dunes", frozenset({200})),
        Case("dunes_balances", "GET", "/dunes/balances", frozenset({200})),
        Case("dune_A", "GET", "/dune/A", frozenset({200, 404})),
        Case("dunes_balance_addr", "GET", f"/dunes/balance/{enc_addr}", frozenset({200})),
        Case("dunes_balance_addr_page", "GET", f"/dunes/balance/{enc_addr}/0", frozenset({200})),
        Case("utxos_balance_addr", "GET", f"/utxos/balance/{enc_addr}", frozenset({200})),
        Case("utxos_balance_addr_page", "GET", f"/utxos/balance/{enc_addr}/0", frozenset({200})),
        Case(
            "inscriptions_balance_addr",
            "GET",
            f"/inscriptions/balance/{enc_addr}",
            frozenset({200}),
        ),
        Case(
            "inscriptions_balance_addr_page",
            "GET",
            f"/inscriptions/balance/{enc_addr}/0",
            frozenset({200}),
        ),
        Case("prc20_tick_list", "GET", "/prc20/tick", frozenset({200})),
        Case("prc20_tick", "GET", "/prc20/tick/PEPE", frozenset({200, 404})),
        Case(
            "prc20_tick_addr_balance",
            "GET",
            f"/prc20/tick/PEPE/address/{enc_addr}/balance",
            frozenset({200, 404}),
        ),
        Case("prc20_addr_balance", "GET", f"/prc20/address/{enc_addr}/balance", frozenset({200})),
        Case("pepemap_list", "GET", "/pepemap?limit=5&offset=0", frozenset({200})),
        Case("pepemap_number_probe", "GET", "/pepemap/0", frozenset({200, 404})),
        Case("pepemap_address", "GET", f"/pepemap/address/{enc_addr}", frozenset({200})),
        Case("dunes_on_outputs", "GET", f"/dunes_on_outputs?outputs={outpoint}", frozenset({200})),
        Case(
            "inscriptions_on_outputs",
            "GET",
            f"/inscriptions_on_outputs?outputs={outpoint}",
            frozenset({200}),
        ),
        Case(
            "inscriptions_by_outputs",
            "GET",
            f"/inscriptions_by_outputs?outputs={outpoint}",
            frozenset({200}),
        ),
        Case("search_query", "GET", f"/search?query={ins}", frozenset({302, 303, 307, 308})),
        Case("search_path", "GET", f"/search/{ins}", frozenset({302, 303, 307, 308})),
        Case("static_asset", "GET", "/static/modern-normalize.css", frozenset({200})),
        ]
    )

    if pepe_num is not None:
        cases.append(
            Case(
                "pepemap_by_number_live",
                "GET",
                f"/pepemap/{int(pepe_num)}",
                frozenset({200}),
            )
        )

    failures: list[str] = []
    timings: list[tuple[str, float]] = []
    print(f"API smoke against {base} ({len(cases)} requests)" + (" [bench]" if bench else "") + "\n")
    for c in cases:
        url = f"{base}{c.path}"
        try:
            t0 = time.perf_counter()
            resp = session.request(c.method, url, timeout=120, allow_redirects=False)
            elapsed_ms = (time.perf_counter() - t0) * 1000.0
        except requests.RequestException as e:
            failures.append(f"{c.name}: request error {e}")
            print(f"  FAIL {c.name:32} error {e}")
            continue
        timings.append((c.name, elapsed_ms))
        ok = resp.status_code in c.ok_status
        ct = (resp.headers.get("Content-Type") or "").split(";")[0].strip()
        mark = "ok" if ok else "FAIL"
        extra = f"  {elapsed_ms:8.1f} ms" if bench else ""
        print(
            f"  [{mark}] {c.name:32} {resp.status_code:3}  {ct or '-'}{extra}"
        )
        if not ok:
            snippet = (resp.text or "")[:120].replace("\n", " ")
            failures.append(
                f"{c.name}: {url} -> {resp.status_code} (allowed {sorted(c.ok_status)}) {snippet}"
            )

    if failures:
        print(f"\n{len(failures)} failure(s)")
        for line in failures:
            print(line)
        return 1

    print(f"\nAll OK — {len(cases)} endpoints")
    if bench and timings:
        timings.sort(key=lambda x: -x[1])
        total = sum(t for _, t in timings)
        slow = [t for t in timings if t[1] >= 500.0]
        print(f"Total wall time (sequential): {total:.1f} ms")
        print("Slowest endpoints (>= 500 ms):")
        for name, ms in slow:
            print(f"  {ms:8.1f} ms  {name}")
        print("Top 10 slowest:")
        for name, ms in timings[:10]:
            print(f"  {ms:8.1f} ms  {name}")
    print(f"Pepemap: list returned {len(pepe_list)} row(s)" + (f", live number={pepe_num}" if pepe_num is not None else ""))
    print(f"Balance / pepemap address: {addr}")
    print(f"Sample inscription: {ins}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
