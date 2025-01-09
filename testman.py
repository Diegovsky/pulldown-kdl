#!/usr/bin/env python3
from io import BytesIO
import os
from pathlib import Path
import shutil
from subprocess import PIPE, STDOUT, Popen
from glob import glob
from argparse import ArgumentParser
from zipfile import ZipFile
from typing import cast
from requests import get

tester = Path('target/debug/tester').resolve()
print(f'Tester: {tester!r}')
if not tester.exists():
    print('`tester` binary not found!')
    exit(-1)

tests_folder = Path('tests/')


type Run = tuple[Path, str]


def test_all_files(
    where: Path, pattern: str, mode: str
) -> tuple[list[Run], list[Run]]:
    procs: list[tuple[Path, Popen[bytes]]] = []
    for f in glob(pattern, root_dir=where):
        f = where / f
        proc = Popen(
            [tester, '-m', mode, f],
            stdout=PIPE,
            stderr=STDOUT,
        )
        procs.append((f, proc))

    failed: list[Run] = []
    passed: list[Run] = []
    for f, p in procs:
        has_failed = p.wait() != 0
        stdout = p.stdout
        assert stdout is not None
        run = (f, stdout.read().decode())
        if has_failed:
            failed.append(run)
            print(f'\x1b[1;34m{f} \x1b[1;31mFAIL\x1b[0m')
        else:
            passed.append(run)
            print(f'\x1b[1;34m{f} \x1b[1;32mOK\x1b[0m')
    return failed, passed


def say(*args: str):
    print(f"\x1b[1;34m{''.join(args)}\x1b[0m")


def fetch_extract_tests():
    temp_test_folder = Path('new-tests')
    shutil.rmtree(tests_folder)

    say('Fetching archive...')
    file = get(
        'https://github.com/kdl-org/kdl/archive/refs/heads/main.zip'
    ).content
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
    os.chdir('..')

    say('Filtering tests...')
    failed, _ = test_all_files(temp_test_folder, '*.kdl', 'emit')
    for failed, _ in failed:
        os.remove(failed)

    say('Copying tests...')
    os.makedirs(tests_folder, exist_ok=True)
    for f in temp_test_folder.iterdir():
        _ = f.rename(tests_folder / f.name)

    say('Cleaning up...')
    shutil.rmtree(temp_test_folder)
    say('Done!')


def main():
    parser = ArgumentParser('testman')
    _ = parser.add_argument('pattern', nargs='?', default='')
    _ = parser.add_argument(
        '-m',
        dest='mode',
        default='compare',
        choices=['compare', 'check', 'emit', 'extract'],
    )
    args = parser.parse_args()
    mode = cast(str, args.mode)
    if mode == 'extract':
        fetch_extract_tests()
        return 0

    pattern = cast(str, args.pattern) + '*.kdl'
    failed, passed = test_all_files(tests_folder, pattern, mode)
    total = len(passed) + len(failed)
    if failed:
        print('fails:')
        for fail, output in failed:
            print(f'\x1b[1;31m{fail}:\x1b[0m')
            print(output)
            print()

        print(f'{len(failed)}/{total} tests failed.')
        return 1

    else:
        print(f'All {total} tests passed!')
        return 0


if __name__ == '__main__':
    exit(main())
