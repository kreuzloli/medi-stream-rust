CREATE TABLE `department`
(
    `id`         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    `dept_name`  VARCHAR(128) NOT NULL COMMENT '科室名称',
    `dept_code`  VARCHAR(64) NULL COMMENT '科室编码/拼音/自定义',
    `sort_no`    INT          NOT NULL DEFAULT 0 COMMENT '排序',
    `status`     TINYINT      NOT NULL DEFAULT 1 COMMENT '状态 1启用 0停用',
    `created_at` DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_department_name` (`dept_name`),
    KEY          `idx_department_status_sort` (`status`, `sort_no`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `disease`
(
    `id`           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    `dept_id`      BIGINT UNSIGNED NOT NULL COMMENT '所属科室ID',
    `disease_name` VARCHAR(256) NOT NULL COMMENT '疾病名称',
    `disease_code` VARCHAR(64) NULL COMMENT '疾病编码(可选)',
    `keywords`     VARCHAR(512) NULL COMMENT '检索关键词/别名(可选)',
    `sort_no`      INT          NOT NULL DEFAULT 0 COMMENT '排序',
    `status`       TINYINT      NOT NULL DEFAULT 1 COMMENT '状态 1启用 0停用',
    `created_at`   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at`   DATETIME     NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    KEY            `idx_disease_dept` (`dept_id`),
    KEY            `idx_disease_name` (`disease_name`),
    UNIQUE KEY `uk_dept_disease` (`dept_id`, `disease_name`),
    CONSTRAINT `fk_disease_dept`
        FOREIGN KEY (`dept_id`) REFERENCES `department` (`id`)
            ON DELETE RESTRICT
            ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;

CREATE TABLE `user_info`
(
    `id`         BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    `user_code`  VARCHAR(64) NULL,
    `nickname`   VARCHAR(128) NULL,
    `email`      VARCHAR(255) NULL,
    `phone`      VARCHAR(64) NULL,
    `status`     INT NULL DEFAULT 1,
    `version`    INT NULL DEFAULT 0,
    `is_deleted` INT NULL DEFAULT 0,
    `created_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    KEY `idx_user_info_user_code` (`user_code`),
    KEY `idx_user_info_deleted` (`is_deleted`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci;
