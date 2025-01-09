
test *args:
    cargo build -p tester
    ./testman.py {{ args }}

test-dbg *args:
    cargo build -p tester -F debug
    ./testman.py {{ args }}
