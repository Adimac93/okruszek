CREATE TABLE users (
    id UUID DEFAULT gen_random_uuid(),
    username TEXT NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE credentials (
    user_id UUID,
    email TEXT NOT NULL,
    password TEXT NOT NULL,
    PRIMARY KEY (user_id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE sessions (
    id UUID DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TABLE products (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    image_id UUID,
    name TEXT NOT NULL,
    price FLOAT NOT NULL,
    PRIMARY KEY (id)
);

CREATE TABLE product_ratings (
    product_id UUID NOT NULL,
    user_id UUID NOT NULL,
    rating INT NOT NULL,
    PRIMARY KEY (product_id, user_id),
    FOREIGN KEY (product_id) REFERENCES products(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- https://www.aleksandra.codes/comments-db-model
CREATE TABLE product_comments (
    id UUID DEFAULT gen_random_uuid(),
    product_id UUID NOT NULL,
    user_id UUID NOT NULL,
    content TEXT NOT NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (product_id) REFERENCES products(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE TYPE reactions as ENUM ('like','love','sad','angry');

CREATE TABLE comment_reactions (
    product_comment_id UUID,
    user_id UUID,
    reaction reactions NOT NULL,
    PRIMARY KEY (product_comment_id, user_id)
);
