-- drop function and trigger

Drop trigger update_posts_modify_time on articles;

Drop Function update_modified_column();

-- This file should undo anything in `up.sql`

Drop table articles;
