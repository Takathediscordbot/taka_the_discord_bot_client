


use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::application_command::CommandData,
    gateway::payload::incoming::InteractionCreate, http::attachment::Attachment,
};

use crate::{
    context::Context, utils::box_commands::RunnableCommand, 
    services::silly_command::SillyCommandPDO,
};

#[derive(CreateCommand, CommandModel)]
#[command(name = "export_silly_commands", desc = "Get all silly commands (author only)")]
pub struct ExportSillyCommands {

}

#[async_trait::async_trait]
impl RunnableCommand for ExportSillyCommands {
    async fn run(
        _shard: u64,
        interaction: &InteractionCreate,
        _data: Box<CommandData>,
        context: &Context,
    ) -> anyhow::Result<anyhow::Result<()>> {
        context.defer_response(interaction).await?;
        let interaction_client = context.http_client.interaction(context.application.id);
      
        let Some(author) = interaction.author_id() else {
            return Ok(Err(anyhow::anyhow!("❌ You're probably not the author of this bot!")))
        };

        if author.get() != context.author_id {
            return Ok(Err(anyhow::anyhow!("❌ You're definitely not the author of this bot!")));
        }


        let commands = SillyCommandPDO::fetch_silly_commands(&context).await;

        let json = serde_json::to_string(&commands)?;
        let content = json.into_bytes();
        // let content = content.to_vec();

        // let mut buf = [0; 31_457_280];
        // {
        //     let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf[..]));
        //     let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
        

        //     for command in commands {
        //         zip.add_directory(&command.name, options)?;
        //         for image in command.images {
        //             let image_path = PathBuf::from_str(image.as_str())?;
                    
        //             let filename = image_path.file_name().ok_or(anyhow::anyhow!("Couldn't find image {}", image))?;
                    
        //             let image_name = format!("{}/{}", &command.name, filename.to_str().ok_or(anyhow::anyhow!("Couldn't create file {}", image))?);
                    
        //             zip.start_file(&image_name, options)?;

        //             let image_bytes = { std::fs::read(image)? };

        //             zip.write_all(&image_bytes)?;
        //         }

        //         zip.add_directory(&format!("{}/self_images", command.name), options)?;

        //         for image in command.self_images {
        //             let image_path = PathBuf::from_str(image.as_str())?;
                    
        //             let filename = image_path.file_name().ok_or(anyhow::anyhow!("Couldn't find image {}", image))?;
                    
        //             let image_name = format!("{}/self_images/{}", &command.name, filename.to_str().ok_or(anyhow::anyhow!("Couldn't create file {}", image))?);
                    
        //             zip.start_file(&image_name, options)?;

        //             let image_bytes = { std::fs::read(image)? };

        //             zip.write_all(&image_bytes)?;
        //         }

        //         zip.start_file(&format!("{}/texts.txt", command.name), options)?;
                
        //         for text in command.texts {
        //             zip.write_all(text.as_bytes())?;
        //         }

        //         zip.start_file(&format!("{}/self_texts.txt", command.name), options)?;
        //         for text in command.self_texts {
        //             zip.write_all(text.as_bytes())?;
        //         }
        //     }
            
        //     zip.finish()?;
        // }



        // let buf = buf.to_vec();
        interaction_client
            .update_response(&interaction.token)
            .attachments(&[
                Attachment::from_bytes("data.json".to_string(), content, 0),
                // Attachment::from_bytes("images.zip".to_string(), buf, 1),
            ])?
            .await?;
        

        Ok(Ok(()))
    }
}
