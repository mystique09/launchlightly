-- Keep email identity consistent across signup, sign-in, and seeded users.

DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM users
        WHERE email IS NOT NULL
        GROUP BY LOWER(BTRIM(email))
        HAVING COUNT(*) > 1
    ) THEN
        RAISE EXCEPTION 'users contain duplicate canonical email addresses';
    END IF;
END
$$;

UPDATE users
SET email = LOWER(BTRIM(email))
WHERE email IS NOT NULL
  AND email <> LOWER(BTRIM(email));

ALTER TABLE users
    ADD CONSTRAINT users_email_is_canonical
    CHECK (email IS NULL OR email = LOWER(BTRIM(email)));

CREATE UNIQUE INDEX users_email_canonical_unique
    ON users (LOWER(email))
    WHERE email IS NOT NULL;
