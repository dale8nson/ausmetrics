use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment, WildcardSegment,
};

use crate::components::header::Header;

use crate::components::line_chart::LineChart;

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
      <div class="flex-col w-full h-full">
        <Header />
        <div class="flex w-full h-full">
          <div class="flex-col w-1/6 h-full justify-start items-start"> </div>
      <div class={"flex-col py-8 space-y-4 justify-start items-start w-5/6 m-auto h-full [&_*]:font-sans [&_*]:text-[#001B69] [&_text]:fill-[#001B69] [&_*]:text-lg"}>
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
        <LineChart />
        </Transition>
      </div>
        </div>
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
