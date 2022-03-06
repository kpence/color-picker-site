use yew::prelude::*;

type Color = (u8, u8, u8);
enum Msg {
    Guess(Color),
}

struct Model {
    score: u64,
    color: Color,
    colors: [Color; 4],
}

impl Model {
    fn new() -> Self {
        let colors = rand::random::<[Color; 4]>();
        Self {
            score: 0,
            color: colors[rand::random::<usize>() % 4],
            colors: colors,
        }
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self::new()
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Guess(guess) => {
                if self.color == guess {
                    self.score += 1;
                } else {
                    self.score = 0;
                }
                let Self {color, colors, ..} = Self::new();
                self.color = color;
                self.colors = colors;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        html! {
            <>
                {
                    self.colors.into_iter().map(|color| {
                        let color = color.clone();
                        html!{
                            <button style={format!("width:128px;height:128px;background-color: rgb{:?};", color)} onclick={link.callback(move |_| Msg::Guess(color))}>
                            </button>
                        }
                    }).collect::<Html>()
                }
                <p>{ format!("Match the RGB value: {:?}", self.color) }</p>
                <p>{ format!("Score: {:?}", self.score) }</p>
            </>
        }
    }
}

fn main() {
    yew::start_app::<Model>();
}
