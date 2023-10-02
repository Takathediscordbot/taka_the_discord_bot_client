INSERT INTO silly_command_self_action_images 
(id_silly_command, image) 
VALUES 
($1, $2)  
RETURNING id_silly_command_self_action;