# replit-xray

[![CI](https://github.com/xhuz/replit-xray/actions/workflows/ci.yml/badge.svg)](https://github.com/xhuz/replit-xray/actions/workflows/ci.yml)

部署xray的trojan ws到replit, 毎分钟自动call，防止程序被kill


# Feature
1. 通过rust的build脚本来下载xray的二进制文件，每次build都会更新到最新xray
2. 把xray打包到二进制文件中，防止代码探测，被和谐
3. 通过stdin方式设置xray的配置，不输出带有敏感信息的配置文件
4. 运行后自动生成一个文件x，就是xray的二进制执行文件

### Usage
1. replit新建空白repls
2. 下载release，修改二进制程序名字不要出现xray trojan之类的东西，拖到空白repls中
3. 新建文件main.sh
4. 修改.replit文件的```run = ["bash", "main.sh"]``` 
5. 脚本启动后会输出```password```到控制台，复制自用，ws的path为 ```/{password}```

### main.sh
假如上传上来的二进制文件名为 server
```bash
#/usr/bin/env bash

chmod +x server

./server

```

### 本项目仅供学习参考