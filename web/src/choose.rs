use yew::prelude::*;

pub struct Comp {
    link: ComponentLink<Self>,
    props: Properties,
}

impl Component for Comp {
    type Message = Message;
    type Properties = Properties;

    fn create(props: Properties, link: ComponentLink<Self>) -> Self {
        Self { link, props }
    }

    fn update(&mut self, msg: Message) -> ShouldRender {
        match msg {
            Message::Chosen(files) => {
                if let Some(file) = files.get(0) {
                    self.props.on_choose.emit(file);
                }
                false
            }
            Message::Etc => false,
        }
    }

    fn change(&mut self, props: Properties) -> ShouldRender {
        self.props = props;
        false
    }

    fn view(&self) -> Html {
        html! {
            <input
                type="file"
                onchange=self.link.callback(|data| match data {
                    ChangeData::Files(files) => Message::Chosen(files),
                    _ => Message::Etc,
                })
                />
        }
    }
}

pub enum Message {
    Chosen(web_sys::FileList),
    Etc,
}

#[derive(Properties, PartialEq, Clone)]
pub struct Properties {
    pub on_choose: Callback<web_sys::File>,
}
