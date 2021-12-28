use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

pub struct Comp {
    input: NodeRef,
}

impl Component for Comp {
    type Message = Msg;
    type Properties = Props;

    fn create(_: &Context<Self>) -> Self {
        Comp {
            input: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FileSelected => {
                let file = self.input.get().unwrap();
                let file = file.dyn_ref::<HtmlInputElement>().unwrap();
                if let Some(files) = file.files() {
                    if let Some(file) = files.get(0) {
                        ctx.props().on_select.emit(file);
                    }
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <div class="x-github-ribbon">
                    <a href="https://github.com/SOF3/phar.rs">
                        <img loading="lazy" width="149" height="149"
                            src="https://github.blog/wp-content/uploads/2008/12/forkme_right_green_007200.png?resize=149%2C149"
                            class="attachment-full size-full" alt="Fork me on GitHub" data-recalc-dims="1" />
                    </a>
                </div>
                <section class="section">
                    <div class="container">
                        <h1 class="title">{ "Phar viewer" }</h1>
                        <div class="file is-boxed">
                            <label class="file-label">
                                <input
                                    class="file-input"
                                    type="file"
                                    ref={&self.input}
                                    onchange={ctx.link().callback(|_| Msg::FileSelected)}
                                    />

                                <span class="file-cta">
                                    <span class="file-label">
                                        { "Select a phar file" }
                                    </span>
                                </span>
                            </label>
                        </div>
                    </div>
                </section>
            </>
        }
    }
}

pub enum Msg {
    FileSelected,
}

#[derive(PartialEq, Properties)]
pub struct Props {
    pub on_select: Callback<web_sys::File>,
}
