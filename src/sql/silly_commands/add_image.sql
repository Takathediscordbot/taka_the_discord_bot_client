INSERT INTO
silly_command_images 
(id_silly_command, image, gender_attribute) 
VALUES
($1, $2, $3)  
RETURNING id_silly_command_images;