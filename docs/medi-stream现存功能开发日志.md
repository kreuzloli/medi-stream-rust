# medi-stream 现存功能开发日志


## 一、整体进展

`medi-stream-rust` 当前是后端服务主体，已从基础 Axum 服务扩展到账号认证、医院目录、直播 URL、腾讯云直播状态查询、微信服务号和 OAuth 授权能力。

`medi-stream-web` 当前是 Vite + TypeScript + Web Components 前端，已从首页展示扩展到直播播放、推流测试、微信授权播放页，并通过 `/api` 代理对接本地后端。

## 二、medi-stream-rust 功能开发日志

### 2026-07-01：后端基础工程与账号模块起步

完成 Rust 后端基础工程搭建，包括：

- Axum Web 服务入口
- 全局配置读取
- 路由注册
- 统一错误处理
- 数据库连接
- 日志初始化
- 目录接口基础实现
- JWT 和账号相关基础模块
- 初始测试用例

随后补充账号注册、登录账号绑定和解绑逻辑，并增加账号相关测试。

### 2026-07-02：认证、医院、外部 HTTP 能力补齐

完善认证体系：

- 增加登录接口
- 增加注销接口
- 增加当前用户接口
- 支持 JWT 签发
- 支持用户信息缓存
- 支持 token 缓存删除
- `loginType` 支持大小写兼容

完成医院和目录模块整合：

- 医院分页查询
- 新增医院
- 查询医院详情
- 更新医院
- 删除医院
- 科室列表查询
- 科室疾病查询
- 完整目录查询

引入外部 HTTP 调用基础能力：

- 通用 HTTP client
- 腾讯云模块基础结构
- 微信模块基础结构
- 外部 API 错误类型
- 相关配置项
- 公共常量集中管理

### 2026-07-03：腾讯云直播 URL 与直播间领域模型

新增腾讯云直播 URL 生成能力：

- `/live/urls` 可生成直播地址
- 支持 WebRTC 推流地址
- 支持 RTMP 推流地址
- 支持 WebRTC 播放地址
- 支持 RTMP 播放地址
- 支持 FLV 播放地址
- 支持 HLS 播放地址
- 支持转码模板播放地址

新增直播间相关领域模型、repository、service 和测试，覆盖：

- 直播房间
- 直播流
- 文件对象
- 直播房间详情组合数据

同时抽取通用启用/禁用状态校验到 `common::validation`，减少账号、医院、直播模块中的重复校验逻辑。

### 2026-07-06：直播 URL 生成逻辑优化

增强直播 URL 生成逻辑：

- 补充 WebRTC 推流地址生成
- 精简腾讯云直播 URL 生成代码
- 调整 live 前缀枚举命名
- 同步更新腾讯云直播相关测试

### 2026-07-07：微信服务号能力接入

开始接入微信服务号能力：

- 新增微信配置项
- 新增微信服务器回调签名校验
- 新增微信相关路由
- 支持获取微信全局 `access_token`
- 支持将微信 `access_token` 缓存到 Redis
- 多次收敛 `wechat_service` 实现，减少重复逻辑

### 2026-07-09：微信 OAuth 与账号字段调整

放宽账号资料字段约束：

- `hospital_id` 从必填调整为可空
- `dept_id` 从必填调整为可空
- `identity_type` 从必填调整为可空

这个调整使注册和用户资料场景更灵活，避免非医务人员或未完善资料用户无法创建账号。

新增微信 OAuth 授权流程：

- 后端生成微信授权地址
- 微信回调后端 callback
- 后端使用 code 换取 openId
- 授权完成后回跳前端页面

同时修复账号 repository 字段顺序问题，并补充服务端基础地址配置：

- `WEB_BASE_URL`
- `WECHAT_OAUTH_CALLBACK_BASE_URL`

这些配置用于后端生成微信授权和回跳地址。

## 三、medi-stream-rust 当前主要后端接口

认证相关：

- `POST /auth/login`
- `GET /auth/logout`
- `GET /auth/me`
- `POST /auth/register`

账号相关：

- `GET /account`
- `POST /account/bind/login`
- `DELETE /account/unbind/:login_id`

目录和医院相关：

- `GET /catalog/departments`
- `GET /catalog/departments/:dept_id/diseases`
- `GET /catalog/full`
- `GET /hospitals`
- `POST /hospitals`
- `GET /hospitals/:id`
- `PUT /hospitals/:id`
- `DELETE /hospitals/:id`

直播相关：

- `GET /live/urls`
- `POST /live/stream-state`

微信相关：

- `GET /wechat/callback`
- `GET /wechat/reload-access-token`
- `GET /wechat/oauth/authorize`
- `GET /wechat/oauth/callback`

## 四、medi-stream-web 功能开发日志

### 2026-07-01：前端基础工程和首页

完成前端基础工程初始化：

- Vite
- TypeScript
- 原生 Web Components
- 全局样式
- 静态资源整理

首页包含以下组件：

- 头部组件
- 轮播组件
- 分类组件
- 精选内容组件
- 优秀内容组件
- 直播列表组件
- 底部组件

服务层开始对接 `catalog` 数据。

新增 hash router，并拆出：

- `home-page`
- `not-found-page`

同时修复科室列表滚动抖动问题。

Vite 本地代理配置为：

- 前端请求 `/api`
- 代理到 `http://127.0.0.1:8080`
- 转发时去掉 `/api` 前缀

### 2026-07-05：直播播放测试页

新增直播播放测试页：

- 新增 `live-player` Web Component
- 新增 `live-page`
- 支持输入 FLV、HLS、WebRTC 等直播播放地址
- 支持播放
- 支持暂停
- 支持销毁播放器
- 支持展示播放状态和错误状态

同时修复路由设置，使直播测试页可通过 hash 路由访问。

### 2026-07-06：TCPlayer 与推流页面

集成腾讯云 TCPlayer 静态资源：

- 播放器 JS
- 播放器 CSS
- 字体资源
- 多语言包
- 播放器插件

新增推流页面和 `live-pusher` 组件：

- 集成 `TXLivePusher`
- 支持摄像头采集
- 支持麦克风采集
- 支持屏幕采集
- 支持开始推流
- 支持停止推流
- 支持暂停视频
- 支持暂停音频
- 支持恢复视频
- 支持恢复音频
- 支持推流状态展示

### 2026-07-09：微信直播播放页与 OAuth 公共方法

新增微信直播播放页：

- 新增 `wechat-live-page`
- 新增 `api` 服务层
- 新增 `auth` 服务层
- 新增 `wechat` 服务层
- 对接后端微信 OAuth 入口
- 支持处理授权回跳参数
- 支持在微信授权后进入播放页面

随后调整微信回调地址逻辑，并将微信 OAuth 相关公共方法抽取到 `services/oauth.ts`。

页面侧职责收敛为：

- 触发授权跳转
- 处理 token 或 openId 回跳
- 执行播放交互

## 五、medi-stream-web 当前主要页面和路由

当前前端主要页面和路由包括：

- `/`：首页
- `/login`：登录页路由声明
- `/live`：直播房间页路由声明
- `/live-push`：推流页
- `/live-play`：直播播放测试页
- `/wechat-live-play`：微信授权播放页

## 六、当前联调重点

当前最需要优先联调的是微信 H5 播放链路：

1. 前端进入 `/wechat-live-play`
2. 前端触发后端 `/wechat/oauth/authorize`
3. 后端生成微信授权地址
4. 微信回调 `/wechat/oauth/callback`
5. 后端使用 code 换取 openId
6. 后端回跳前端页面
7. 前端进入直播播放流程

