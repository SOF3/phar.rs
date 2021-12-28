use yew::prelude::*;

use crate::file_list;
use crate::Reader;

pub struct Comp {
    file_selected: Option<FileSelected>,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Comp {
            file_selected: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::OpenStub | Msg::OpenMetadata => {
                let contents = {
                    let mut reader = ctx.props().reader.borrow();

                    match msg {
                        Msg::OpenStub => reader.stub_bytes().unwrap().as_ref().to_vec(),
                        Msg::OpenMetadata => reader.metadata_bytes().unwrap().as_ref().to_vec(),
                        _ => unreachable!(),
                    }
                };

                self.file_selected = Some(FileSelected {
                    name: match msg {
                        Msg::OpenStub => &b"Stub"[..],
                        Msg::OpenMetadata => b"Metadata",
                        _ => unreachable!(),
                    }
                    .to_vec(),
                    contents,
                });

                true
            }
            Msg::OpenFile(path) => {
                let mut contents = None;

                {
                    let mut reader = ctx.props().reader.borrow();

                    let mut found = false;
                    reader
                        .for_each_file(|name, read| {
                            if !found && name == path {
                                let mut buf = Vec::new();
                                read.read_to_end(&mut buf).unwrap();
                                contents = Some(buf);
                                found = true;
                            }
                            Ok(())
                        })
                        .unwrap();
                }

                self.file_selected = Some(FileSelected {
                    name: path,
                    contents: contents.expect("Invalid OpenFile message"),
                });

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let main_col = match &self.file_selected {
            Some(path) => {
                html! {
                    <div class="column">
                        <h2 class="subtitle">{ String::from_utf8_lossy(&path.name) }</h2>
                        <pre class="x-raw-code-view">
                            <code>
                                { String::from_utf8_lossy(&path.contents) }
                            </code>
                        </pre>
                    </div>
                }
            }
            None => html! {
                <div class="column section">
                    <div class="container">
                        <p>
                            { "Select a file from the sidebar" }
                        </p>
                    </div>
                </div>
            },
        };
        html! {
            <div class="columns">
                <div class="column is-narrow">
                    <aside class="menu x-file-list">
                        <file_list::Comp
                            reader={ctx.props().reader.clone()}
                            open_stub={ctx.link().callback(|()| Msg::OpenStub)}
                            open_metadata={ctx.link().callback(|()| Msg::OpenMetadata)}
                            open_file={ctx.link().callback(Msg::OpenFile)}
                            />
                    </aside>
                </div>
                { main_col }
            </div>
        }
    }
}

pub enum Msg {
    OpenStub,
    OpenMetadata,
    OpenFile(Vec<u8>),
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub reader: Reader,
}

struct FileSelected {
    name: Vec<u8>,
    contents: Vec<u8>,
}
