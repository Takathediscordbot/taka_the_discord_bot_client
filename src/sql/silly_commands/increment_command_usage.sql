UPDATE silly_command_usage 
SET usages = usages + 1 
WHERE id_user_1 = $1 
AND id_user_2 = $2 
AND id_silly_command = $3 
RETURNING usages;