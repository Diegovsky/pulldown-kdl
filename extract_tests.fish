#!/usr/bin/env fish
function say
    set_color blue
    echo $argv
    set_color normal
end
say "Building..."
cargo build -p tester
set tester "$PWD/target/debug/tester"

say "Fetching archive..."
curl -sSOL 'https://github.com/kdl-org/kdl/archive/refs/heads/main.zip'

say "Unzipping..."
unzip -j main.zip '*/input/*.kdl' -d new-tests
cd new-tests

say "Filtering tests..."
for file in *.kdl
    say $file
    if ! $tester -m emit $file
        rm $file
    end
end
cd ..

say "Cleaning up..."
mv new-tests/* tests/
rm -r new-tests
rm main.zip

say "Done!"
