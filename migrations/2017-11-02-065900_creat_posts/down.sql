-- This file should undo anything in `up.sql`
Drop table posts;

-- drop function and trigger

Drop Function update_timestamp_column();

Drop trigger update_posts_modify_time on posts;
