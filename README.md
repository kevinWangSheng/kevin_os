## wsl下启动kernel
可以在windows安装一个图形化的vsXsr的工具按照如下操作：
首先，在 Windows 上安装 X Server（如果还没安装）：


下载 VcXsrv：https://sourceforge.net/projects/vcxsrv/
安装 VcXsrv


在 Windows 上启动 VcXsrv：

打开开始菜单
搜索并运行 XLaunch
设置过程：

选择 "Multiple windows"
Display number 设置为 0
选择 "Start no client"
在 Extra settings 页面勾选 "Disable access control"
点击 "Finish"
，然后在wsl下运行
```sh
export DISPLAY=$(grep -m 1 nameserver /etc/resolv.conf | awk '{print $2}'):0.0

# 再次检查 DISPLAY
echo $DISPLAY

# 现在尝试允许连接
xhost +local:
```
接着运行：
```sh
qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64_kevin_os/debug/bootimage-kevin_os.bin \
    -serial stdio \
    -display sdl
```
就会出现一个界面：
![alt text](image.png)


然后如果想要使用更方便的命令整合，比如cargo run运行整个部分，参照.cargo/config.toml的配置如下
```toml
[target.'cfg(target_os = "none")']
runner = "bootimage runner"

[unstable]
build-std = ["core", "compiler_builtins"]
build-std-features = ["compiler-builtins-mem"]

[build]
target = "x86_64_kevin_os.json"

[target.x86_64-kevin_os]
runner = """
bootimage runner --timeout 300
qemu-system-x86_64 \
    -drive format=raw,file=target/x86_64-kevin_os/debug/bootimage-kevin_os.bin \
    -serial stdio \
    -display sdl \
"""
```
