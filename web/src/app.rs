use std::io;

use byte_unit::Byte;
use yew::prelude::*;

use crate::{browser, choose_file, RawReader, Reader};

pub struct Comp {
    state: State,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = ();

    fn create(_: &Context<Self>) -> Self {
        Comp {
            state: State::ChooseFile,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let link = ctx.link().clone();
        match msg {
            Msg::FileChosen(file) => {
                let file_name = file.name();
                let file_size = file.size();

                let task = gloo_file::callbacks::read_as_bytes(
                    &gloo_file::File::from(file),
                    move |result| {
                        log::debug!("Fetched result {:?}", result);
                        match result {
                            Ok(bytes) => {
                                let reader = RawReader::read(
                                    io::Cursor::new(bytes),
                                    phar::read::Options::default(),
                                );
                                match reader {
                                    Ok(reader) => {
                                        link.send_message(Msg::ReadDone(Reader::new(reader)));
                                    }
                                    Err(err) => {
                                        link.send_message(Msg::Error(format!("{:?}", err)));
                                    }
                                }
                            }
                            Err(err) => {
                                link.send_message(Msg::Error(format!("{:?}", err)));
                            }
                        }
                    },
                );

                self.state = State::Loading {
                    file_name,
                    file_size,
                    _task: task,
                };

                true
            }
            Msg::ReadDone(reader) => {
                self.state = State::Reader(reader);
                true
            }
            Msg::Error(err) => {
                self.state = State::Error {
                    error: format!("{:?}", err),
                };
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        match &self.state {
            State::ChooseFile => html! {
                <choose_file::Comp
                    on_select={ctx.link().callback(Msg::FileChosen)}
                    />
            },
            State::Loading {
                file_name,
                file_size,
                _task,
            } => html! {
                <div>
                    { format!("Loading {} ({})...", file_name, Byte::from_bytes(*file_size as u128).get_appropriate_unit(true)) }
                </div>
            },
            State::Error { error } => html! {
                <div>
                    { "Error: " }
                    { error }
                </div>
            },
            State::Reader(reader) => html! {
                <browser::Comp reader={reader.clone()} />
            },
        }
    }
}

pub enum Msg {
    FileChosen(web_sys::File),
    ReadDone(Reader),
    Error(String),
}

enum State {
    ChooseFile,
    Loading {
        file_name: String,
        file_size: f64,
        _task: gloo_file::callbacks::FileReader,
    },
    Reader(Reader),
    Error {
        error: String,
    },
}
