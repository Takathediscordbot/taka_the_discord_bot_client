SELECT usages
FROM silly_command_usage 
WHERE id_user_1 = $1
AND id_user_2 = $2 
AND id_silly_command = $3
