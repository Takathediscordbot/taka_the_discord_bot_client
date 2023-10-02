SELECT 
id_silly_command, name, description,
command_type, texts, images, self_texts, self_images,
gender_attributes, footer_text, preferences
FROM public.silly_commands_data_new   
WHERE name = $1;