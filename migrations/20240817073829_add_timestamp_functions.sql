CREATE OR REPLACE FUNCTION update_timestamps()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        IF NEW.created_at IS NULL THEN
            NEW.created_at := now();
        END IF;
    END IF;
    IF NEW.updated_at IS NULL OR TG_OP = 'UPDATE' THEN
        NEW.updated_at := now();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER users_timestamps
BEFORE INSERT OR UPDATE ON users
FOR EACH ROW
EXECUTE PROCEDURE update_timestamps();
