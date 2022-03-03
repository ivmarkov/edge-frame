use web_sys::{Event, HtmlInputElement};
use yew::TargetCast;

pub use callback2::*;
pub use field::*;
pub use navigation_item::*;
pub use status_item::*;

mod callback2;
mod field;
mod navigation_item;
mod status_item;

pub fn if_true<'a>(cond: bool, s: &'a str) -> &'a str {
    if cond {
        s
    } else {
        ""
    }
}

pub fn get_input_text(event: Event) -> String {
    event.target_unchecked_into::<HtmlInputElement>().value()
}

pub fn get_input_checked(event: Event) -> bool {
    event.target_unchecked_into::<HtmlInputElement>().checked()
}
