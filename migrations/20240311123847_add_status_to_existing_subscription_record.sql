-- UPDATE all the columns with null status to confirmed.
UPDATE SUBSCRIPTIONS
    SET status='confirmed'
WHERE status IS NULL
