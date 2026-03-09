use crate::components::theme_toggle::ThemeToggle;
use leptos::prelude::*;

#[component]
pub fn Header() -> impl IntoView {
    leptos::logging::log!("Header");

    let dark_theme = RwSignal::new(false);

    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        let window = window();
        let ls = window.local_storage();
        leptos::logging::log!("ls: {ls:#?}");
    });
    leptos::logging::log!("Header view!");
    view! {
      <header class="flex w-full h-[58px] bg-[#001b69] border-b-[#e50027]" >
        <ThemeToggle dark_theme ={ dark_theme } />
      </header>
    }
}
