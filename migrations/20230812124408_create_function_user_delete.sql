CREATE OR REPLACE FUNCTION user_delete(
    IN UUID
)
RETURNS TABLE (LIKE"User")
STRICT
ROWS 1
LANGUAGE SQL
AS $body$
    DELETE
    FROM "User"
    WHERE id = $1
    RETURNING *;
$body$;