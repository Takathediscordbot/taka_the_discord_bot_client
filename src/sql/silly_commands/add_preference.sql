UPDATE
silly_commands
SET
gender_attributes = array_append(gender_attributes, $1) 
WHERE name = $2;