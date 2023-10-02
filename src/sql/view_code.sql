 SELECT silly_commands.id_silly_command,
    silly_commands.name,
    silly_commands.command_type,
    silly_commands.description,
    silly_command_images.id_images,
    silly_command_images.images,
    silly_command_images.preferences,
    silly_command_texts.id_texts,
    silly_command_texts.texts,
    silly_command_self_action_images.id_self_images,
    silly_command_self_action_images.self_images,
    silly_command_self_action_texts.id_self_texts,
    silly_command_self_action_texts.self_texts
   FROM ((((silly_commands
     LEFT JOIN ( SELECT silly_command_images_1.id_silly_command,
            array_agg(silly_command_images_1.id_silly_command_images) AS id_images,
            array_agg(silly_command_images_1.image) AS images,
            array_agg(silly_command_images.gender_attribute) as preferences
           FROM silly_command_images silly_command_images_1
          GROUP BY silly_command_images_1.id_silly_command) silly_command_images USING (id_silly_command))
     LEFT JOIN ( SELECT silly_command_texts_1.id_silly_command,
            array_agg(silly_command_texts_1.id_silly_command_text) AS id_texts,
            array_agg(silly_command_texts_1.text) AS texts
           FROM silly_command_texts silly_command_texts_1
          GROUP BY silly_command_texts_1.id_silly_command) silly_command_texts USING (id_silly_command))
     LEFT JOIN ( SELECT silly_command_self_action_images_1.id_silly_command,
            array_agg(silly_command_self_action_images_1.id_silly_command_self_action) AS id_self_images,
            array_agg(silly_command_self_action_images_1.image) AS self_images
           FROM silly_command_self_action_images silly_command_self_action_images_1
          GROUP BY silly_command_self_action_images_1.id_silly_command) silly_command_self_action_images USING (id_silly_command))
     LEFT JOIN ( SELECT silly_command_self_action_texts_1.id_silly_command,
            array_agg(silly_command_self_action_texts_1.id_silly_command_self_action_text) AS id_self_texts,
            array_agg(silly_command_self_action_texts_1.text) AS self_texts
           FROM silly_command_self_action_texts silly_command_self_action_texts_1
          GROUP BY silly_command_self_action_texts_1.id_silly_command) silly_command_self_action_texts USING (id_silly_command));