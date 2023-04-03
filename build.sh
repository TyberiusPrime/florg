yarn tauri build
deb="./src-tauri/target/release/bundle/deb/florg_0.0.0_amd64.deb"
ls $deb
rm deb-unpack -rf | true
mkdir deb-unpack -p
dpkg-deb -x $deb deb-unpack


#!/bin/bash

file_to_replace="florg"
new_program="deb-unpack/usr/bin/florg"

# Try to copy the new program to the original file location
cp "$new_program" "$file_to_replace" && echo "direct replacement of ./florg" && exit 0

# If the copy fails, try to move the original file to a temporary location
tmp_file=$(mktemp "/tmp/florg_XXXXXX")
mv "$file_to_replace" "$tmp_file"

# Try to copy the new program to the original file location again
cp "$new_program" "$file_to_replace"

# If the second copy fails, move the original file back to its original location
if [ $? -ne 0 ]; then
    mv "$tmp_file" "$file_to_replace"
    echo "Failed to replace file"
    exit 1
fi

# Remove the temporary file
rm "$tmp_file"
echo "File replaced successfully"
exit 0

cp  ./florg
echo "copied to ./florg"
