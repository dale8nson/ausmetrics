use crate::components::theme_toggle::ThemeToggle;
use leptos::prelude::*;

#[component]
pub fn Header() -> impl IntoView {
    leptos::logging::log!("Header");

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let window = window();
        let ls = window.local_storage();
        leptos::logging::log!("ls: {ls:#?}");
    });
    leptos::logging::log!("Header view!");
    view! {
      <header class="flex w-full h-[58px] bg-[#001b69] border-b-[#e50027] items-center p-8" >
        <ThemeToggle />
      </header>
    }
}
