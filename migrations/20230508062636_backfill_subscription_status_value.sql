-- sqlx does not wrap migration scripts on a transation by default.
BEGIN;
  -- Backfill all nullable values from status
  UPDATE subscriptions
    SET status = 'confirmed'
    WHERE status IS NULL;
  -- Mark the status column as required
  ALTER TABLE subscriptions ALTER COLUMN status SET NOT NULL;  
COMMIT;
