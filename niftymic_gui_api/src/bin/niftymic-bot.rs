use std::path::Path;

use teloxide::net::Download;
use teloxide::prelude::*;

use niftymic_gui::config::Config;
use niftymic_gui::*;
use teloxide::types::InputFile;
use tokio::fs;

async fn start_reconstruction(archive_path: &str) -> niftymic::Result<String> {
    let config = Config::new(None)?;
    let niftymic = niftymic::NiftyMic::new(&archive_path, &config)?;

    niftymic.convert_dicom_to_nifti()?;
    niftymic.generate_masks_from_nifti()?;
    niftymic.reconstruct(niftymic::Options::default())?;
    niftymic.convert_nifti_to_dicom()
}

async fn start_bot() -> niftymic::Result<()> {
    log::info!("Starting NiftyMIC_bot ...");
    let config = Config::new(None)?;

    if let Some(telegram) = config.telegram {
        let bot = Bot::new(telegram.teloxide_token);
        teloxide::repl(bot, |bot: Bot, msg: Message| async move {
            if let Some(document) = msg.document() {
                bot.send_message(msg.chat.id, "Downloading document ...".to_string())
                    .await?;
                log::debug!("Input document {:?}", document);
                let file = bot.get_file(&document.file.id).await?;
                let file_name = document.file_name.clone().unwrap();
                let archive_path = format!("/tmp/{}", file_name);
                let mut dst = fs::File::create(&archive_path).await?;
                bot.download_file(&file.path, &mut dst).await?;
                bot.send_message(msg.chat.id, "Archive downloaded".to_string())
                    .await?;
                bot.send_message(msg.chat.id, "Starting reconstruction ...".to_string())
                    .await?;
                let result = start_reconstruction(&archive_path.clone()).await.unwrap();
                let file_name = Path::new(&result)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                let file = fs::File::open(&result).await?;
                let input_file = InputFile::read(file);
                let input_file = input_file.file_name(file_name.clone());
                bot.send_document(msg.chat.id, input_file).await?;
            } else {
                bot.send_message(
                    msg.chat.id,
                    "Bot is running, send a .zip archive to start a reconstruction".to_string(),
                )
                .await?;
            }
            Ok(())
        })
        .await;
        Ok(())
    } else {
        Err(niftymic::Error::FailedToStartBot)
    }
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    match start_bot().await {
        Ok(_) => log::info!("Terminated with no errors"),
        Err(err) => log::error!("{}", err.to_string()),
    }
}
