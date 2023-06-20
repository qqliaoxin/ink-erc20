## 构建节点
```
cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git --force


substrate-contracts-node --dev
```
## 安装合约编译器
```
cargo install cargo-contract
```
## 创建ink！项目
```
cargo contract new flipper
```
## 测试您的合约
```
cargo test

cargo test --features  e2e-tests
```
## 编译您的合约
```
cargo contract build
```
## 调试
```
ink::env::debug_println!("magic number: {}", value);
```
## 发布合约
```
cargo contract build --release
```