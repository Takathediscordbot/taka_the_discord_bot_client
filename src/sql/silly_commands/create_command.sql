INSERT 
INTO silly_command_new 
(name, description, command_type, footer_text) 
VALUES 
($1, $2, $3, $4)  
RETURNING id_silly_command;