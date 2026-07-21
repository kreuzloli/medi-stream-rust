CREATE TABLE `department` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    `dept_name` VARCHAR(128) NOT NULL COMMENT '科室名称',
    `dept_code` VARCHAR(64) NULL COMMENT '科室编码/拼音/自定义',
    `sort_no` INT NOT NULL DEFAULT 0 COMMENT '排序',
    `status` TINYINT NOT NULL DEFAULT 1 COMMENT '状态 1启用 0停用',
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_department_name` (`dept_name`),
    KEY `idx_department_status_sort` (`status`, `sort_no`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;

CREATE TABLE `disease` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    `dept_id` BIGINT UNSIGNED NOT NULL COMMENT '所属科室ID',
    `disease_name` VARCHAR(256) NOT NULL COMMENT '疾病名称',
    `disease_code` VARCHAR(64) NULL COMMENT '疾病编码(可选)',
    `keywords` VARCHAR(512) NULL COMMENT '检索关键词/别名(可选)',
    `sort_no` INT NOT NULL DEFAULT 0 COMMENT '排序',
    `status` TINYINT NOT NULL DEFAULT 1 COMMENT '状态 1启用 0停用',
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    KEY `idx_disease_dept` (`dept_id`),
    KEY `idx_disease_name` (`disease_name`),
    UNIQUE KEY `uk_dept_disease` (`dept_id`, `disease_name`),
    CONSTRAINT `fk_disease_dept` FOREIGN KEY (`dept_id`) REFERENCES `department` (`id`) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;

CREATE TABLE `hospital` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    `hospital_name` VARCHAR(256) NOT NULL COMMENT '医院名称',
    `hospital_code` VARCHAR(64) NULL COMMENT '医院编码/拼音/自定义',
    `province` VARCHAR(64) NULL COMMENT '省份',
    `city` VARCHAR(64) NULL COMMENT '城市',
    `address` VARCHAR(512) NULL COMMENT '医院地址',
    `sort_no` INT NOT NULL DEFAULT 0 COMMENT '排序',
    `status` TINYINT NOT NULL DEFAULT 1 COMMENT '状态 1启用 0停用',
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_hospital_name` (`hospital_name`),
    KEY `idx_hospital_status_sort` (`status`, `sort_no`),
    KEY `idx_hospital_city` (`city`)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;

CREATE TABLE `user_info` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    `user_code` VARCHAR(64) NULL COMMENT '用户编码/业务编号',
    `real_name` VARCHAR(128) NOT NULL COMMENT '姓名',
    `nickname` VARCHAR(128) NULL COMMENT '昵称',
    `hospital_id` BIGINT UNSIGNED NULL COMMENT '医院ID',
    `dept_id` BIGINT UNSIGNED NULL COMMENT '科室ID',
    `identity_type` VARCHAR(64) NULL COMMENT '身份类型 MEDICAL_WORKER医药行业相关从业人员 NON_MEDICAL_WORKER非医药行业相关从业人员',
    `doctor_cert_no` VARCHAR(128) NULL COMMENT '职业医师资格证书编号，建议加密保存',
    `id_card_no` VARCHAR(128) NULL COMMENT '身份证号，建议加密保存',
    `status` TINYINT NOT NULL DEFAULT 1 COMMENT '状态 1启用 0停用',
    `version` INT NOT NULL DEFAULT 0 COMMENT '乐观锁版本号',
    `is_deleted` TINYINT NOT NULL DEFAULT 0 COMMENT '是否删除 0否 1是',
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_user_info_user_code` (`user_code`),
    KEY `idx_user_info_hospital_dept` (`hospital_id`, `dept_id`),
    KEY `idx_user_info_identity_type` (`identity_type`),
    KEY `idx_user_info_deleted` (`is_deleted`),
    CONSTRAINT `fk_user_info_hospital` FOREIGN KEY (`hospital_id`) REFERENCES `hospital` (`id`) ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT `fk_user_info_dept` FOREIGN KEY (`dept_id`) REFERENCES `department` (`id`) ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;

CREATE TABLE `user_login_account` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    `user_id` BIGINT UNSIGNED NOT NULL COMMENT '用户ID',
    `login_type` VARCHAR(32) NOT NULL COMMENT '登录方式 EMAIL邮箱 PHONE手机 WECHAT微信 GITHUB GitHub',
    `login_identifier` VARCHAR(255) NOT NULL COMMENT '登录标识：邮箱/手机号/openid/github_id等',
    `password_hash` VARCHAR(255) NULL COMMENT '密码哈希，仅邮箱/手机号密码登录需要，第三方登录为空',
    `third_party_union_id` VARCHAR(255) NULL COMMENT '第三方统一ID，例如微信unionid，可选',
    `is_verified` TINYINT NOT NULL DEFAULT 0 COMMENT '是否已验证 0未验证 1已验证',
    `last_login_at` DATETIME NULL COMMENT '最后登录时间',
    `status` TINYINT NOT NULL DEFAULT 1 COMMENT '状态 1启用 0停用/解绑',
    `is_deleted` TINYINT NOT NULL DEFAULT 0 COMMENT '是否删除 0否 1是',
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_login_type_identifier` (
        `login_type`,
        `login_identifier`
    ),
    KEY `idx_user_login_account_user_id` (`user_id`),
    KEY `idx_user_login_account_union_id` (`third_party_union_id`),
    KEY `idx_user_login_account_deleted` (`is_deleted`),
    CONSTRAINT `fk_user_login_account_user` FOREIGN KEY (`user_id`) REFERENCES `user_info` (`id`) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci;

CREATE TABLE file_object (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    file_name VARCHAR(255) NOT NULL,
    file_url VARCHAR(1024) NOT NULL COMMENT '对象存储URL/本地路径',
    mime_type VARCHAR(128) NULL,
    file_size BIGINT UNSIGNED NULL,
    sha256 CHAR(64) NULL COMMENT '去重/校验',
    created_at DATETIME(3) DEFAULT CURRENT_TIMESTAMP(3) NOT NULL,
    KEY idx_sha256 (sha256)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci COMMENT = '文件对象';

CREATE TABLE live_room (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    owner_user_id BIGINT UNSIGNED NOT NULL COMMENT '房主用户',
    room_code VARCHAR(64) NOT NULL COMMENT '房间编码(业务唯一)',
    title VARCHAR(128) NOT NULL,
    description VARCHAR(512) NULL,
    cover_file_id BIGINT UNSIGNED NULL COMMENT '封面图',
    status TINYINT DEFAULT 1 NOT NULL COMMENT '1正常 0停用 2封禁等',
    is_deleted TINYINT DEFAULT 0 NOT NULL,
    created_at DATETIME(3) DEFAULT CURRENT_TIMESTAMP(3) NOT NULL,
    updated_at DATETIME(3) DEFAULT CURRENT_TIMESTAMP(3) NOT NULL ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_room_owner FOREIGN KEY (owner_user_id) REFERENCES user_info (id),
    CONSTRAINT fk_room_cover FOREIGN KEY (cover_file_id) REFERENCES file_object (id),
    UNIQUE KEY uk_room_code (room_code),
    KEY idx_owner (owner_user_id),
    KEY idx_status_deleted (status, is_deleted)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci COMMENT = '直播间';

ALTER TABLE live_room DROP FOREIGN KEY fk_room_cover;

CREATE TABLE live_room_stream (
    id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
    room_id BIGINT UNSIGNED NOT NULL,
    stream_code VARCHAR(64) NOT NULL COMMENT '该房间内的流编码(唯一)',
    stream_name VARCHAR(128) NOT NULL COMMENT '腾讯云streamName(用于生成URL)',
    title VARCHAR(128) NULL COMMENT '该路视频标题/来源',
    sort_no INT DEFAULT 0 NOT NULL COMMENT '排序',
    is_default TINYINT DEFAULT 0 NOT NULL COMMENT '是否默认展示',
    status TINYINT DEFAULT 1 NOT NULL COMMENT '1可用 0停用',
    is_deleted TINYINT DEFAULT 0 NOT NULL,
    created_at DATETIME(3) DEFAULT CURRENT_TIMESTAMP(3) NOT NULL,
    updated_at DATETIME(3) DEFAULT CURRENT_TIMESTAMP(3) NOT NULL ON UPDATE CURRENT_TIMESTAMP(3),
    CONSTRAINT fk_stream_room FOREIGN KEY (room_id) REFERENCES live_room (id),
    UNIQUE KEY uk_room_stream_code (room_id, stream_code),
    UNIQUE KEY uk_room_stream_name (room_id, stream_name),
    KEY idx_room (room_id),
    KEY idx_room_status (room_id, status, is_deleted)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci COMMENT = '直播间多路流/视频源';

ALTER TABLE live_room_stream DROP INDEX uk_room_stream_name;

-- 管理后台与本服务共享数据库。以下为增量结构，本项目不实现管理员后台接口。
CREATE TABLE administrator (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    username VARCHAR(128) NOT NULL COMMENT '管理员登录名',
    password_hash VARCHAR(255) NOT NULL COMMENT '密码哈希，不保存明文密码',
    real_name VARCHAR(128) NOT NULL COMMENT '管理员姓名',
    last_login_at DATETIME NULL COMMENT '最后登录时间',
    status TINYINT NOT NULL DEFAULT 1 COMMENT '状态 1启用 0停用',
    is_deleted TINYINT NOT NULL DEFAULT 0 COMMENT '是否删除 0否 1是',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY uk_administrator_username (username),
    KEY idx_administrator_status_deleted (status, is_deleted)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci COMMENT = '管理员账号';

CREATE TABLE admin_role (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    role_code VARCHAR(64) NOT NULL COMMENT '角色编码',
    role_name VARCHAR(128) NOT NULL COMMENT '角色名称',
    description VARCHAR(512) NULL COMMENT '角色说明',
    status TINYINT NOT NULL DEFAULT 1 COMMENT '状态 1启用 0停用',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY uk_admin_role_code (role_code),
    UNIQUE KEY uk_admin_role_name (role_name),
    KEY idx_admin_role_status (status)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci COMMENT = '管理员角色';

CREATE TABLE admin_permission (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    permission_code VARCHAR(128) NOT NULL COMMENT '权限编码',
    permission_name VARCHAR(128) NOT NULL COMMENT '权限名称',
    resource_type VARCHAR(32) NULL COMMENT '资源类型 MENU菜单 API接口 BUTTON按钮',
    description VARCHAR(512) NULL COMMENT '权限说明',
    status TINYINT NOT NULL DEFAULT 1 COMMENT '状态 1启用 0停用',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    UNIQUE KEY uk_admin_permission_code (permission_code),
    KEY idx_admin_permission_status (status)
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci COMMENT = '管理员权限';

CREATE TABLE administrator_role (
    administrator_id BIGINT UNSIGNED NOT NULL COMMENT '管理员ID',
    role_id BIGINT UNSIGNED NOT NULL COMMENT '角色ID',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (administrator_id, role_id),
    KEY idx_administrator_role_role (role_id),
    CONSTRAINT fk_administrator_role_administrator FOREIGN KEY (administrator_id) REFERENCES administrator (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_administrator_role_role FOREIGN KEY (role_id) REFERENCES admin_role (id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci COMMENT = '管理员角色关联';

CREATE TABLE role_permission (
    role_id BIGINT UNSIGNED NOT NULL COMMENT '角色ID',
    permission_id BIGINT UNSIGNED NOT NULL COMMENT '权限ID',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (role_id, permission_id),
    KEY idx_role_permission_permission (permission_id),
    CONSTRAINT fk_role_permission_role FOREIGN KEY (role_id) REFERENCES admin_role (id) ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_role_permission_permission FOREIGN KEY (permission_id) REFERENCES admin_permission (id) ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE = InnoDB DEFAULT CHARSET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci COMMENT = '角色权限关联';

-- 保留普通用户所有者字段，同时增加独立管理员所有者字段；已有数据无需回填。
ALTER TABLE live_room
DROP FOREIGN KEY fk_room_owner,
MODIFY COLUMN owner_user_id BIGINT UNSIGNED NULL COMMENT '房主普通用户ID',
ADD COLUMN owner_admin_id BIGINT UNSIGNED NULL COMMENT '房主管理员ID' AFTER owner_user_id,
ADD COLUMN department_id BIGINT UNSIGNED NULL COMMENT '直播间科室分类ID' AFTER cover_file_id,
ADD COLUMN disease_id BIGINT UNSIGNED NULL COMMENT '直播间疾病分类ID' AFTER department_id,
ADD COLUMN is_top TINYINT NOT NULL DEFAULT 0 COMMENT '是否置顶 0否 1是' AFTER disease_id,
ADD CONSTRAINT fk_room_owner_user FOREIGN KEY (owner_user_id) REFERENCES user_info (id),
ADD CONSTRAINT fk_room_owner_admin FOREIGN KEY (owner_admin_id) REFERENCES administrator (id),
ADD CONSTRAINT fk_room_department FOREIGN KEY (department_id) REFERENCES department (id) ON DELETE RESTRICT ON UPDATE CASCADE,
ADD CONSTRAINT fk_room_disease FOREIGN KEY (disease_id) REFERENCES disease (id) ON DELETE RESTRICT ON UPDATE CASCADE,
ADD CONSTRAINT chk_live_room_single_owner CHECK (
    (
        owner_user_id IS NOT NULL
        AND owner_admin_id IS NULL
    )
    OR (
        owner_user_id IS NULL
        AND owner_admin_id IS NOT NULL
    )
),
ADD CONSTRAINT chk_live_room_is_top CHECK (is_top IN (0, 1)),
ADD KEY idx_room_owner_admin (owner_admin_id),
ADD KEY idx_room_department_disease (department_id, disease_id),
ADD KEY idx_room_top_status_deleted (is_top, status, is_deleted);
ALTER TABLE live_room
    ADD COLUMN start_time DATETIME(3) NULL COMMENT '计划开播时间' AFTER is_top;
ALTER TABLE live_room
    ADD KEY idx_room_start_time_status_deleted (
        start_time,
        status,
        is_deleted
    );


CREATE TABLE `tencent_live_config` (
    `id` BIGINT UNSIGNED NOT NULL AUTO_INCREMENT COMMENT '配置ID',
    `name` VARCHAR(100) NOT NULL COMMENT '配置名称',

    `secret_id` VARCHAR(255) NOT NULL COMMENT '腾讯云SecretId',
    `secret_key` VARCHAR(512) NOT NULL COMMENT '腾讯云SecretKey',

    `app_name` VARCHAR(100) NOT NULL COMMENT '直播应用名称',
    `push_domain` VARCHAR(255) NOT NULL COMMENT '推流域名',
    `play_domain` VARCHAR(255) NOT NULL COMMENT '播放域名',

    `push_key` VARCHAR(255) NOT NULL COMMENT '推流防盗链Key',
    `play_key` VARCHAR(255) NOT NULL COMMENT '播放防盗链Key',
    `default_ttl_seconds` BIGINT NOT NULL DEFAULT 86400 COMMENT 'URL默认有效期，秒',

    `status` TINYINT NOT NULL DEFAULT 1 COMMENT '状态：1启用，0停用',
    `remark` VARCHAR(500) DEFAULT NULL COMMENT '备注',

    `is_deleted` TINYINT NOT NULL DEFAULT 0 COMMENT '逻辑删除：0否，1是',
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        ON UPDATE CURRENT_TIMESTAMP,

    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_tencent_live_config_name` (`name`, `is_deleted`),
    KEY `idx_tencent_live_config_status` (`status`, `is_deleted`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
COMMENT='腾讯云直播配置';


ALTER TABLE `user_info`
ADD COLUMN `mobile` VARCHAR(30) NULL COMMENT '用户联系电话',
ADD COLUMN `header_id` BIGINT UNSIGNED NULL COMMENT '用户头像文件ID';
