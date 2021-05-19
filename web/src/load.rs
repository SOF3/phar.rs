use std::io::Cursor;

use yew::services::reader::{FileData, ReaderService, ReaderTask};
use yew::ComponentLink;

use crate::Phar;

pub(crate) fn start_read_file(
    file: web_sys::File,
    link: &ComponentLink<super::Main>,
) -> anyhow::Result<ReaderTask> {
    let handle = ReaderService::read_file(
        file,
        link.callback(|data: FileData| {
            match Phar::read(
                Cursor::new(data.content),
                phar::read::Options::builder().build(),
            ) {
                Ok(phar) => super::Message::PharLoad(Box::new(phar)),
                Err(err) => super::Message::Err(err.into()),
            }
        }),
    )?;

    Ok(handle)
}
