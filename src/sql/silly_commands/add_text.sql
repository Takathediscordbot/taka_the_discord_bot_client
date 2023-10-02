INSERT INTO 
silly_command_texts 
(id_silly_command, text) 
VALUES($1, $2)  
RETURNING id_silly_command_text;