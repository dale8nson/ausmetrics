use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment, WildcardSegment,
};

// use crate::components::line_chart::LineChart;
use crate::line_chart::LineChart;

/// red #E50027 blue #001B69

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/ausmetrics.css"/>

        // sets the document title
        <Title text="AUSMETRICS"/>

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=move || "Not found.">
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=WildcardSegment("any") view=NotFound/>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
      <header class="p-5 w-full h-32 flex flex-col justify-center  items-start"><h1 class="subpixel-antialiased tracking-[-3.5]  text-7xl font-thin text-[#001B69] font-serif"><span class="font-black">"A"</span><span class="text-6xl font-semibold text-[#E50027]">"US"</span><span class="font-black">"M"</span><span class="text-6xl font-semibold text-[#E50027]">"ETRICS"</span></h1>
        <hr class="border-[#001B69] border-double border-2 w-full"/>
      </header>
      <div class="flex-col space-y-4 justify-start items-start w-2/3 m-auto h-full [&_*]:font-sans [&_*]:text-[#001B69] [&_text]:fill-[#001B69] [&_*]:text-lg">
        <Transition>
        <LineChart/>
        </Transition>
      </div>
    }
}

/// 404 - Not Found
#[component]
fn NotFound() -> impl IntoView {
    // set an HTTP status code 404
    // this is feature gated because it can only be done during
    // initial server-side rendering
    // if you navigate to the 404 page subsequently, the status
    // code will not be set because there is not a new HTTP request
    // to the server
    #[cfg(feature = "ssr")]
    {
        // this can be done inline because it's synchronous
        // if it were async, we'd use a server function
        let resp = expect_context::<leptos_actix::ResponseOptions>();
        resp.set_status(actix_web::http::StatusCode::NOT_FOUND);
    }

    view! {
        <h1>"Not Found"</h1>
    }
}
