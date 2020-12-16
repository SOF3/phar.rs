use std::io::Cursor;

use wasm_bindgen::prelude::*;
use yew::prelude::*;
use yew::services::reader::ReaderTask;

mod choose;
mod load;

#[wasm_bindgen(start)]
pub fn run_app() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    App::<Main>::new().mount_to_body();
}

struct Main {
    link: ComponentLink<Self>,
    state: State,
}

impl Component for Main {
    type Message = Message;
    type Properties = Properties;

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            state: State::Choose,
        }
    }

    fn update(&mut self, msg: Message) -> ShouldRender {
        match msg {
            Message::Choose(file) => {
                let file_size = file.size() as u128;
                let handle = load::start_read_file(file, &self.link).expect("Failed to read file");
                self.state = State::Loading {
                    file_size,
                    _handle: handle,
                };
                true
            }
            Message::PharLoad(phar) => {
                self.state = State::Browse(phar);
                true
            }
            Message::Err(err) => {
                self.state = State::Error(err);
                true
            }
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        match &self.state {
            State::Choose => html! {
                <choose::Comp
                    on_choose=self.link.callback(|file| Message::Choose(file))/>
            },
            State::Loading { file_size, .. } => html! {
                <p>{ format!("Reading {} of data...", byte_unit::Byte::from_bytes(*file_size).get_appropriate_unit(true)) }</p>
            },
            State::Browse(_) => html! {},
            State::Error(err) => html! {
                <div>
                    <h1>{ "Error" }</h1>
                    <p>{ err }</p>
                </div>
            },
        }
    }
}

enum State {
    Choose,
    Loading {
        file_size: u128,
        _handle: ReaderTask,
    },
    Browse(Box<Phar>),
    Error(anyhow::Error),
}

enum Message {
    Choose(web_sys::File),
    PharLoad(Box<Phar>),
    Err(anyhow::Error),
}

type Properties = ();

type Phar = phar::Reader<Cursor<Vec<u8>>>;
