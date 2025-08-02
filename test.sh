#!/bin/bash

python3 -c 'import sys,struct; m=b"{\"x\": 1000, \"y\": 300}"; sys.stdout.buffer.write(struct.pack("<I", len(m)) + m)' | ./target/release/click_point
