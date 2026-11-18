#!/usr/bin/env python3
"""Copy the WebAssembly module of a built single-file game (the release asset) into the
published game page, so the live game is exactly the released engine.

  python3 -I scripts/embed-wasm.py <built turbocursa.html> <docs/turbocursa.html>
"""
import re, sys
RX = re.compile(r"AGFzbQ[A-Za-z0-9+/=]{1000,}")
src, dst = sys.argv[1], sys.argv[2]
new = RX.search(open(src, encoding="utf-8").read())
page = open(dst, encoding="utf-8").read()
old = RX.search(page)
if not new or not old:
    sys.exit("embed-wasm: no WebAssembly module found")
if new.group(0) == old.group(0):
    print("wasm already identical")
else:
    open(dst, "w", encoding="utf-8").write(page[:old.start()] + new.group(0) + page[old.end():])
    print("wasm replaced")
