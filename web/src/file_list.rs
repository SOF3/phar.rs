use std::collections::BTreeMap;

use yew::prelude::*;

use crate::Reader;

pub struct Comp {
    dir: Vec<u8>,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Comp { dir: Vec::new() }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ChangeDir(path) => {
                self.dir = path;
                true
            }
            Msg::OpenFile(path) => {
                ctx.props().open_file.emit(path);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let parents = (!self.dir.is_empty()).then(|| {
            let mut parents = Vec::new();
            for (i, &byte) in self.dir.iter().enumerate() {
                if byte != b'/' || i + 1 == self.dir.len() {
                    continue;
                }

                let display = &self.dir[..i];
                let path = self.dir[..=i].to_vec();
                parents.push(html! {
                    <li onclick={ctx.link().callback(move |_| Msg::ChangeDir(path.clone()))}>
                        <a>{ String::from_utf8_lossy(display) }</a>
                    </li>
                });
            }

            html! {
                <>
                    <p class="menu-label">{ "Parents" }</p>
                    <ul class="menu-list">
                        { for parents }
                    </ul>
                </>
            }
        });

        let mut names = BTreeMap::new();

        {
            let mut reader = ctx.props().reader.borrow();

            reader.for_each_file(|name, _contents| {
                let name = match name.strip_prefix(&self.dir[..]) {
                    Some(name) => name,
                    None => return Ok(()),
                };

                if let Some(offset) = name.iter().position(|&ch| ch == b'/') {
                    let dir = &name[..=offset];
                    let path: Vec<u8> = [&self.dir[..], dir].concat();

                    names.entry(path).or_insert_with_key(|path| {
                        let path = path.clone();

                        html! {
                            <li onclick={ctx.link().callback(move |_| Msg::ChangeDir(path.clone()))}>
                                <a>{ String::from_utf8_lossy(dir) }</a>
                            </li>
                        }
                    });
                } else {
                    let path = [&self.dir[..], name].concat();

                    names.entry(path).or_insert_with_key(|path| {
                        let path = path.clone();
                        html! {
                            <li onclick={ctx.link().callback(move |_| Msg::OpenFile(path.clone()))}>
                                <a>{ String::from_utf8_lossy(name) }</a>
                            </li>
                        }
                    });
                }

                Ok(())
            }).unwrap();
        }

        html! {
            <>
                { for parents }
                <p class="menu-label">{ format!("Files under /{}", String::from_utf8_lossy(&self.dir)) }</p>
                <ul class="menu-list">
                    { for names.into_values() }
                </ul>
            </>
        }
    }
}

pub enum Msg {
    ChangeDir(Vec<u8>),
    OpenFile(Vec<u8>),
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub reader: Reader,
    pub open_file: Callback<Vec<u8>>,
}
