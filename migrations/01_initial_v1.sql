CREATE TABLE IF NOT EXISTS items(
    id          INTEGER     PRIMARY KEY NOT NULL,
    name        TEXT                    NOT NULL,
    description TEXT                    NOT NULL
);

CREATE TABLE IF NOT EXISTS orders(
    id              INTEGER     PRIMARY KEY NOT NULL,
    table_id        INTEGER                 NOT NULL,
    item_id         INTEGER                 NOT NULL,
    time_to_prepare INTEGER                 NOT NULL,
    created_at      INTEGER                 NOT NULL,
    paid_at         INTEGER,
    FOREIGN KEY(item_id) REFERENCES items(id)
);

INSERT INTO items(name, description) VALUES 
    ("から揚げ小鉢付", "四川風～麻婆茄子豆腐定食"), 
    ("四川風", "辛旨・麻婆茄子豆腐定食"), 
    ("から揚げ小鉢付", "～四川風～辛旨・麻婆茄子豆腐定食"),
    ("ミックスとじ定食", "ロースカツ、エビフライ、牛肉。３つの美味しさをとろ～り卵でふんわりやさしくとじ込めました。"),
    ("大分とり天定食", "ジューシーな鶏もも肉のとり天、大分ならではの食べ方、ぽん酢・ねりからしでさっぱりとお召し上がりください。"),
    ("味噌かつ煮定食", "豆板醤の辛みを効かせた特製味噌ダレの濃厚な味に、半熟卵が絡んだ絶妙な美味しさです。"),
    ("サバの味噌煮定食", "味噌のほんのりとした甘さをじっくりしみ込ませました。");