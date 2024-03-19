# SoLog

Do you know `xlog`? I also wrote a `solog`, but it seems that I forgot to delete some test code...

你知道 `xlog` 吗？我也写了一个 `solog`，但是我好像忘记删掉一些测试代码了...

Test environment:

```sh
cd env/
docker build -t solog:test .
docker run -p 1337:1337 solog:test
```
**Note:**

1. Please write your solution in `framwork-solve/src/main.rs`. The interaction of this problem is to send a sequence of instructions through the socket, and the environment executes these instructions in the identity of the user's signer. Read souce code for more details.
2. If you need to start locally, you need to install the rust environment and **solana v1.18.1** environment or just use `framework`. For details, see `framework`'s `dev.sh`

**注意：**

1. 解题请写在 `framwork-solve/src/main.rs` 位置，本题题目交互方式是通过 socket 发送指令序列，由环境以 user 的 signer 身份依次执行这些指令，具体细节请看源码。
2. 如果需要本地启动，需要安装 rust 环境以及 **solana v1.18.1** 环境，或者直接使用`framework`，具体可以看 `framework` 的 `dev.sh`
