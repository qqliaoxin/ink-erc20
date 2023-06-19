## 构建节点
```
cargo install contract --git https://github.com/paritytech/substrate-contracts-node.git --force
```
## 创建ink！项目
```
cargo contract new flipper
```
## 测试您的合约
```
cargo test
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