#!/usr/bin/env python3
import os
from pathlib import Path
import sys
from subprocess import STDOUT, Popen
from glob import glob

tester = Path('target/debug/tester').resolve()
os.chdir('tests')
procs: list[tuple[str, Popen[bytes]]] = []
for f in glob('*.kdl'):
    proc = Popen([tester, '-m', 'compare', f], stdout=sys.stdout, stderr=STDOUT)
    procs.append((f, proc))

failed: list[str] = []
passed = 0
for f, p in procs:
    has_failed = p.wait()
    if has_failed:
        failed.append(f)
    else:
        passed += 1
        print(f'\x1b[1;34m{f} \x1b[1;32mOK\x1b[0m')

if failed:
    print('fails:')
    for fail in failed:
        print(f'{fail}')

    print(f'Some {len(failed)}/{len(failed)+passed} tests failed.')
    exit(1)

else:
    print(f'All {passed} tests passed!')
