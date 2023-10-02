INSERT INTO silly_command_usage 
(id_silly_command, id_user_1, id_user_2, usages) 
VALUES ($1, $2, $3, 0) 
RETURNING id_silly_command_usage;