// lib.rs 是库入口。把各个模块公开出来后，main.rs 和 tests 都可以通过
// medi_stream_rust::xxx 的形式复用这些代码。
pub mod account;
pub mod auth;
pub mod common;
pub mod config;
pub mod error;
pub mod hospital;
pub mod logging;
pub mod routes;
pub mod state;
pub mod wechat;
pub mod tencent_cloud;
