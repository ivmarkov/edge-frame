use web_sys::{Event, HtmlInputElement};
use yew::TargetCast;

pub fn get_input_text(event: Event) -> String {
    event.target_unchecked_into::<HtmlInputElement>().value()
}

pub fn get_input_checked(event: Event) -> bool {
    event.target_unchecked_into::<HtmlInputElement>().checked()
}
