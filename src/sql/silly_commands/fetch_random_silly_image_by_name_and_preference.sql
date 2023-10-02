SELECT 
image 
FROM silly_command_images 
WHERE id_silly_command = $1 AND gender_attribute = $2
ORDER BY RANDOM () LIMIT 1;