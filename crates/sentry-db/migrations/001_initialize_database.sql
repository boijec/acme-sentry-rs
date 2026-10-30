CREATE TABLE IF NOT EXISTS acme_users(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT(36) NOT NULL,
    key_type TEX(15) NOT NULL,
    key_path TEXT(256) NOT NULL,
    user_dump_path TEXT(256) NOT NULL
);

CREATE TABLE IF NOT EXISTS acme_users_directory(
    directory_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    directory_base_url TEXT(512) NOT NULL,
    new_nonce TEXT(512) NOT NULL,
    new_account TEXT(512),
    new_order TEXT(512) NOT NULL,
    new_authz TEXT(512) NOT NULL,
    revoke_cert TEXT(512) NOT NULL,
    key_change TEXT(512) NOT NULL,
    FOREIGN KEY (user_id) REFERENCES acme_users(id) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS acme_account_links(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    account_url TEXT(512) NOT NULL,
    FOREIGN KEY (user_id) REFERENCES acme_users(id) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS acme_orders(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    domain TEXT(256) NOT NULL,
    order_url TEXT(512),
    authorization_url TEXT(512),
    finalize_url TEXT(512),
    certificate_url TEXT(512),
    status TEXT(32) NOT NULL DEFAULT 'pending',
    created_at TEXT(32) NOT NULL,
    FOREIGN KEY (user_id) REFERENCES acme_users(id) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS acme_dns_challenges(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id INTEGER NOT NULL,
    token TEXT(512) NOT NULL,
    key_authorization TEXT(1024) NOT NULL,
    dns_record_name TEXT(512) NOT NULL,
    status TEXT(32) NOT NULL DEFAULT 'pending',
    FOREIGN KEY (order_id) REFERENCES acme_orders(id) ON DELETE RESTRICT
);

CREATE TABLE IF NOT EXISTS acme_certificates(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    order_id INTEGER NOT NULL,
    certificate_path TEXT(256) NOT NULL,
    downloaded_at TEXT(32) NOT NULL,
    FOREIGN KEY (order_id) REFERENCES acme_orders(id) ON DELETE RESTRICT
);