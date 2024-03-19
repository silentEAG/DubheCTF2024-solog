# > solana --version
# solana-cli 1.18.1 (src:5d824a36; feat:756280933, client:SolanaLabs)

cd chall/ && cargo build-bpf && cd ..
cargo build -r

cp ./target/release/framework ../env/
cp ./chall/target/deploy/chall.so ../env/

# cd ../env/ && docker build -t solog:test .
# docker run -p 1337:1337 solog:test