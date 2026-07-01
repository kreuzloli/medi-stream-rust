# Account Login Separation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split user profile data from login account bindings and wire account CRUD plus password login to the new `medi.sql` tables.

**Architecture:** `user_info` stores real-name profile fields only. `user_login_account` stores login methods such as email, phone, WeChat, and GitHub, so a login method can be unbound without deleting the user profile.

**Tech Stack:** Rust 2021, Axum, SQLx MySQL, Redis cache, JWT, Argon2 password hashing.

## Global Constraints

- Keep the existing `src/account` module shape and route style.
- Do not change database schema beyond using the three new tables already present in `db/medi.sql`.
- User profile updates must not overwrite login credentials.
- Deletion is logical deletion using `is_deleted` and `status`.
- Passwords must never be stored in plain text.

---

### Task 1: Account Models and Validation

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/account/model.rs`
- Create: `src/account/service.rs`
- Modify: `src/account/mod.rs`
- Test: `tests/account_tests.rs`

**Interfaces:**
- Produces: `CreateAccountReq`, `UpdateUserProfileReq`, `UserProfile`, `UserLoginAccount`, `AccountDetail`, `LoginType`.
- Produces: `validate_create_account_req(req: &CreateAccountReq) -> Result<(), AppError>`.
- Produces: `hash_password(password: &str) -> Result<String, AppError>` and `verify_password(password: &str, hash: &str) -> Result<bool, AppError>`.

- [ ] Write tests for required fields, login type parsing, and password hash verification.
- [ ] Add Argon2 dependency.
- [ ] Replace old `UserInfo` DTO with profile/login DTOs.
- [ ] Add validation and password helper functions.

### Task 2: Repository SQL

**Files:**
- Modify: `src/account/repository.rs`

**Interfaces:**
- Consumes: account model structs from Task 1.
- Produces: `insert_user_profile`, `insert_login_account`, `find_account_detail_by_id`, `page_user_profiles`, `update_user_profile`, `logical_delete_user`, `logical_delete_login_account`, `find_login_for_auth`, `touch_last_login`.

- [ ] Update SQL to use `real_name`, `hospital_id`, `dept_id`, `identity_type`, `doctor_cert_no`, and `id_card_no`.
- [ ] Add login account insert, lookup, unbind, and auth lookup SQL.
- [ ] Keep all dynamic filters parameter-bound through `QueryBuilder`.

### Task 3: Handlers and Auth Login

**Files:**
- Modify: `src/account/handlers.rs`
- Modify: `src/auth/handlers.rs`
- Modify: `src/routes.rs`

**Interfaces:**
- `POST /account`: create user profile plus initial login binding.
- `GET /account/:id`: profile plus login bindings.
- `GET /account`: paged profiles.
- `PUT /account/:id`: update profile only.
- `DELETE /account/:id`: logical delete profile and login bindings.
- `POST /account/:id/logins`: add a login binding.
- `DELETE /account/:id/logins/:login_id`: unbind a login method.
- `POST /auth/login`: login by `loginType`, `loginIdentifier`, and `password`.

- [ ] Wire handlers to service/repository functions.
- [ ] Add login routes.
- [ ] Replace hard-coded admin login with database-backed login.

### Task 4: Verification

**Files:**
- Test: `tests/account_tests.rs`

- [ ] Run `cargo test`.
- [ ] Run `cargo fmt`.
- [ ] Run `cargo check`.
