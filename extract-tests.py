#!/usr/bin/env python3
from glob import glob
from io import BytesIO
import os
from pathlib import Path
import shutil
from requests import get
from zipfile import ZipFile

temp_test_folder = Path('new-tests')


def say(*args: str):
    print(f"\x1b[1;34m{''.join(args)}\x1b[0m")


def system(cmd: str):
    if os.system(cmd) != 0:
        raise Exception(f"Command '{cmd}' failed")


say('Building...')
_ = system('cargo build -p tester')
tester = Path('target/debug/tester').resolve()

say('Fetching archive...')
file = get('https://github.com/kdl-org/kdl/archive/refs/heads/main.zip').content
assert isinstance(file, bytes)


say('Unzipping...')
z = ZipFile(BytesIO(file))
files = [
    f
    for f in z.filelist
    if '/input/' in f.filename and f.filename.endswith('.kdl')
]
os.makedirs(temp_test_folder, exist_ok=True)
os.chdir(temp_test_folder)
for zf in files:
    name = Path(zf.filename).name
    with z.open(zf) as zf, open(name, 'wb') as f:
        _ = f.write(zf.read())

say('Filtering tests...')
for file in Path('.').iterdir():
    say(str(file))
    try:
        system(f'{tester} -m emit {file}')
    except Exception:
        os.remove(file)

os.chdir('..')

say('Cleaning up...')
for new_test in temp_test_folder.iterdir():
    _ = new_test.rename(Path('tests/') / new_test.name)

shutil.rmtree(temp_test_folder)
say('Done!')
