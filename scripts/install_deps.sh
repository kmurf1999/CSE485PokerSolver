sudo apt-get update
sudo apt-get upgrade -y

# install gcc and other build tools
sudo apt-get install build-essential -y
sudo apt-get install cmake -y
sudo apt-get install libclang-dev -y

# download rust install file
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > install_rust.sh
chmod +x ./install_rust.sh
# install rust
./install_rust.sh -y
# remove file
rm install_rust.sh
# add to path
export PATH="$HOME/.cargo/bin:$PATH"
# switch to nightly version
rustup default nightly
