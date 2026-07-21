// lib.rs 是库入口。把各个模块公开出来后，main.rs 和 tests 都可以通过
// medi_stream_rust::xxx 的形式复用这些代码。
pub mod account;
pub mod auth;
pub mod common;
pub mod config;
pub mod error;
pub mod file;
pub mod hospital;
pub mod live;
pub mod logging;
pub mod routes;
pub mod state;
pub mod tencent_cloud;
pub mod utils;
pub mod wechat;
