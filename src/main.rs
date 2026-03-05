use dotenv::from_filename;
use log::{debug, LevelFilter};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};
pub mod components;

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use ausmetrics::app::*;
    use leptos::config::get_configuration;
    use leptos::prelude::*;
    use leptos_actix::{generate_route_list, LeptosRoutes};
    use leptos_meta::MetaTags;

    from_filename(".env.local").ok();

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;

    env_logger::Builder::new()
        .format_source_path(true)
        .format_module_path(false)
        .format_target(false)
        .format_timestamp(None)
        // .filter(Some("leptos"), LevelFilter::Trace)
        .filter_module("ausmetrics::components", LevelFilter::Debug)
        // .filter(Some("reactive_graph-0.2.12"), LevelFilter::Off)
        // .filter_module("leptos_reactive", LevelFilter::Error)
        // .filter_level(LevelFilter::Debug)
        .init();

    // let filter = EnvFilter::try_from_default_env()
    //     .unwrap_or_else(|_| EnvFilter::new("info")) // Fallback to info
    //     .add_directive("reactive_graph=error".parse().unwrap())
    //     .add_directive("leptos_router=error".parse().unwrap())
    //     .add_directive("leptos_chartistry=error".parse().unwrap());

    // tracing_subscriber::registry()
    //     .with(fmt::layer())
    //     .with(filter)
    //     .init();

    // debug!("{:?}", std::env::vars());

    HttpServer::new(move || {
        // Generate the list of routes in your Leptos App
        let routes = generate_route_list(App);
        let leptos_options = &conf.leptos_options;
        let site_root = leptos_options.site_root.clone().to_string();

        println!("listening on http://{}", &addr);

        App::new()
            // serve JS/WASM/CSS from `pkg`
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            // serve other assets from the `assets` directory
            .service(Files::new("/assets", &site_root))
            // serve the favicon from /favicon.ico
            .service(favicon)
            .leptos_routes(routes, {
                let leptos_options = leptos_options.clone();
                move || {
                    view! {
                        <!DOCTYPE html>
                        <html lang="en">
                            <head>
                                <meta charset="utf-8"/>
                                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                                <AutoReload options=leptos_options.clone() />
                                <HydrationScripts options=leptos_options.clone()/>
                                <MetaTags/>
                            </head>
                            <body class="flex flex-col w-screen h-screen bg-neutral-200">
                                <App/>
                            </body>
                        </html>
                    }
                }
            })
            .app_data(web::Data::new(leptos_options.to_owned()))
        //.wrap(middleware::Compress::default())
    })
    .bind(&addr)?
    .run()
    .await
}

#[cfg(feature = "ssr")]
#[actix_web::get("favicon.ico")]
async fn favicon(
    leptos_options: actix_web::web::Data<leptos::config::LeptosOptions>,
) -> actix_web::Result<actix_files::NamedFile> {
    let leptos_options = leptos_options.into_inner();
    let site_root = &leptos_options.site_root;
    Ok(actix_files::NamedFile::open(format!(
        "{site_root}/favicon.ico"
    ))?)
}

#[cfg(not(any(feature = "ssr", feature = "csr")))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
    // see optional feature `csr` instead
}

#[cfg(all(not(feature = "ssr"), feature = "csr"))]
pub fn main() {
    // a client-side main function is required for using `trunk serve`
    // prefer using `cargo leptos serve` instead
    // to run: `trunk serve --open --features csr`
    use ausmetrics::app::*;

    console_error_panic_hook::set_once();

    leptos::mount_to_body(App);
}
