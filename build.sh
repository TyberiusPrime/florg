yarn tauri build
deb="./src-tauri/target/release/bundle/deb/florg_0.0.0_amd64.deb"
ls $deb
rm deb-unpack -rf | true
mkdir deb-unpack -p
dpkg-deb -x $deb deb-unpack
cp deb-unpack/usr/bin/florg ./florg
echo "copied to ./florg"
