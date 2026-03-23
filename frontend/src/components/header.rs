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
      <header class="flex w-full h-[88px] bg-[#001b69] border-b-2 border-solid border-b-[#e50027] items-end justify-between p-4 dark:bg-[#091c2a] " >
        <div class="flex gap-2 items-end w-auto h-full">
          <div class="flex w-auto h-full justify-center items-end bg-[#e50027] rounded-xs aspect-square">
            <div class="logo-icon pb-[6px] aspect-square flex gap-[calc(3/34*100%)] justify-center items-end w-auto h-full">
              <span class="bg-white block w-[calc(5/34*100%)] h-[calc(11/34*100%)]" />
              <span class="bg-white block w-[calc(5/34*100%)] h-[calc(16/34*100%)]" />
              <span class="bg-white block w-[calc(5/34*100%)] h-[calc(22/34*100%)]" />
            </div>
          </div>
          <div class="flex-col items-start justify-end w-full">
            <p class="block h-full font-stretch-[148%] text-[48px] text-white font-serif m-0">"A"<span class="text-[0.8em]">"US"</span>M<span class="text-[0.8em]">"ETRICS"</span></p>
            <p class="text-neutral-500 text-sm">"AUSTRALIAN PUBLIC DATA · ANALYSIS"</p>
          </div>
        </div>
        <ThemeToggle />
      </header>
    }
}
