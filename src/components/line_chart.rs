use std::{
    fmt::Pointer,
    fs::{read, read_to_string, File},
    io::BufReader,
    path::PathBuf,
    str::FromStr,
};

use chrono::{DateTime, FixedOffset, Local, NaiveDate, NaiveDateTime, TimeZone, Utc};
use jsonpath_rust::query::{js_path, js_path_process, js_path_vals};
use leptos::{
    attr::height,
    html::{Caption, Div},
    prelude::*,
    reactive::signal,
};
use leptos::{logging::debug_log, reactive::diagnostics::SpecialNonReactiveZone};
// use leptos_chartistry::{
//     AspectRatio, AxisMarker, Chart, Colour, ColourScheme, IntoInner, Legend, Line, Period,
//     RotatedLabel, Series, TickLabels, Timestamps, Tooltip, XGridLine, XGuideLine, YGridLine,
//     YGuideLine, LINEAR_GRADIENT,
// };
//
#[cfg(target_arch = "wasm32")]
use charming::{
    component::{Axis, Legend, Title},
    datatype::{CompositeValue, NumericValue},
    element::{AxisType, LabelAlign, TextAlign},
    WasmRenderer,
};

use log::{debug, error};
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, Map, Number, Value};

use wasm_bindgen::UnwrapThrowExt;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Data {
    x: f64,
    y: f64,
    ir: f64,
    date: String,
}

#[server]
async fn load_json_file(path: PathBuf) -> Result<Value, ServerFnError> {
    if let Ok(file) = File::open(path) {
        // debug!("{:#?}", file.metadata().unwrap());
        let reader = BufReader::new(file);
        if let Ok(value) = from_reader(reader) {
            // debug!("value: {value:#?}");
            return Ok(value);
        } else {
            return Err(ServerFnError::ServerError(
                "Unable to read JSON file".into(),
            ));
        }
    } else {
        return Err(ServerFnError::ServerError(
            "Unable to read JSON file".into(),
        ));
    }
}

pub fn extract_observations(binding: &Value) -> Vec<Data> {
    let mut arr =
        js_path_vals("$.data.dataSets[0].series..observations", binding).unwrap_or_default();

    let ir = vec![
        4.10, 4.01, 3.85, 3.85, 3.70, 3.60, 3.60, 3.60, 3.60, 3.60, 3.83,
    ];

    // debug!("arr: {arr:#?}");
    //
    if arr.is_empty() {
        return Vec::<Data>::new();
    }

    let Value::Object(mut map) = arr[0].clone() else {
        return Vec::new();
    };

    let data_points = map
        .iter()
        .zip(ir.into_iter().enumerate())
        .filter_map(|((k, v), (idx, r))| {
            let x = f64::from_str(k).ok()?;
            let Value::Array(a) = v else {
                return None;
            };
            let Value::Number(num) = &a[0] else {
                return None;
            };
            let y = num.as_f64()?;
            Some((x, y, r))
        })
        .collect::<Vec<(f64, f64, f64)>>();

    let mut periods = js_path_vals(
        "$.data.structures[0].dimensions.observation[0].values[*].start",
        binding,
    )
    .unwrap();
    let val = Value::String("2026-02-01T00:00:00".to_string());
    periods.push(&val);

    debug!("periods: {periods:#?}");

    let mut dates = Vec::<String>::new();
    periods.sort_by(|a, b| {
        let s1 = if let Value::String(s1) = a {
            s1
        } else {
            &String::new()
        };
        let s2 = if let Value::String(s2) = b {
            s2
        } else {
            &String::new()
        };
        s1.cmp(s2)
    });
    for period in periods {
        let period = if let Value::String(period) = period.clone() {
            period
        } else {
            "".to_string()
        };
        debug!("period: {period:?}");
        let date = chrono::NaiveDateTime::parse_from_str(period.as_str(), "%Y-%m-%dT%H:%M:%S")
            .unwrap()
            .and_local_timezone(Local::now().timezone())
            .unwrap();
        // let date_string = date.to_string();
        // debug!("date: {date_string}");
        dates.push(format!("{}", date.format("%b %Y").to_string()));
    }

    dates.sort();

    data_points
        .into_iter()
        .zip(dates)
        .map(|((x, y, r), date)| Data { x, y, ir: r, date })
        .collect::<Vec<Data>>()
}

#[component]
pub fn LineChart() -> impl IntoView {
    debug!("LineChart");
    let chart_ref = NodeRef::<Div>::new();
    let data = Resource::new(
        || (),
        |_| async move {
            load_json_file(PathBuf::from_str("graphql-server/static/CPI_simple.json").unwrap())
                .await
        },
    );

    let observations: Signal<Vec<Data>> = Signal::derive(move || {
        let data = data.get();
        let Some(result) = data else {
            return Vec::new();
        };

        let res = match result {
            Ok(ref value) => extract_observations(value),
            Err(err) => {
                debug!("Error loading JSON: {err:?}");
                Vec::<Data>::new()
            }
        };
        res
    });
    debug!("observations: {:#?}", observations.get());
    // let x_ticks = TickLabels::from_generator(Timestamps::from_period(Period::Month))
    //     .with_format(|d, _| format!("{}", d.format("%b %Y")));
    // debug!("{x_ticks:#?}");
    // let _guard =
    // SpecialNonReactiveZone::enter();
    // debug!("_guard: {_guard:?}");
    //
    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        use std::char::CharTryFromError;

        println!("Effect");

        type EChart = charming::Chart;

        let data = observations.get();
        let chart = EChart::new()
            .title(
                Title::new()
                    .text("Consumer Price Index (CPI) / Cash Rate Target (CRT)")
                    .text_align(TextAlign::Auto),
            )
            .series(charming::series::Series::Line(
                charming::series::Line::new().data(data.iter().map(|ob| ob.y).collect()),
            ))
            .series(charming::series::Series::Line(
                charming::series::Line::new().data(data.iter().map(|ob| ob.ir).collect()),
            ))
            .x_axis(Axis::new().data(data.into_iter().map(|d| d.date).collect::<Vec<String>>()))
            .y_axis(Axis::new().type_(AxisType::Value))
            .legend(
                Legend::new()
                    .data(vec!["CPI", "CRT"])
                    .align(LabelAlign::Center)
                    .show(true),
            );

        let el = chart_ref.get().expect("div should be mounted").value();
        debug!("{el:#?}");
        let width = el.client_width();
        let height = el.client_height();

        eprintln!("width: {width} height: {height}");
        let renderer = WasmRenderer::new(width as u32, height as u32);
        renderer.render("chart", &chart).unwrap();
    });

    view! {

        <div class="w-full flex-col h-fit m-auto border-2 border-solid border-black bg-white font-serif">
          <div node_ref=chart_ref class="w-2/3 h-full" id="chart" ></div>
        </div>
        <p class="w-full text-right">"Sources: ABS/RBA"</p>
    }
}

// The most useful combinations of economic measures generally blend
// leading indicators (predicting future activity), coincident indicators (measuring current state), and lagging indicators (confirming trends) to provide a comprehensive view of economic health.
// Here are 10 of the most useful combinations of economic measures, combining both data sets and policy levers:

//     GDP Growth Rate + Unemployment Rate (The Core Health Check): Real GDP provides the overall economic output, while the unemployment rate shows the health of the labor market. Together, they identify whether the economy is expanding or contracting, and if it's impacting jobs.
//     Consumer Price Index (CPI) + Interest Rates (The Inflation/Policy Mix): CPI measures inflation (cost of living), and central bank interest rates are the primary tool to control it. This pairing is essential for assessing monetary policy impact and purchasing power.
//     Retail Sales + Consumer Confidence (The Sentiment-Action Pair): Consumer spending drives a large portion of the economy, and retail sales reflect actual spending. Combined with consumer confidence surveys, this tells you if people are spending and if they feel secure enough to continue doing so.
//     Housing Starts + Mortgage Rates (The Leading Indicator Combo): Housing starts are a primary leading indicator, as they signal future economic activity in construction and related industries. Tracking them against mortgage rates reveals how housing demand is affected by financing costs.
//     Stock Market Performance + Business Confidence Index (The Sentiment-Action Pair): Stock markets (like the S&P 500) act as a leading indicator of future business performance and investor confidence. Pairing this with business confidence surveys gives a picture of corporate investment outlook.
//     Industrial Production + Capacity Utilization (The Manufacturing Duo): These measure the output of manufacturing, mining, and utilities, showing how efficiently factories are running. A high, sustainable rate suggests expansion, while low capacity indicates a sluggish economy.
//     Trade Balance + Currency Strength (The External Sector Pair): The trade balance (exports vs. imports) indicates a country's competitiveness. This must be paired with currency strength, as a strong currency can hinder exports, while a weak one can increase import costs.
//     Real GDP per Capita + Gini Coefficient (The Development & Equity Mix): While GDP measures total size, per capita shows average income, and the Gini coefficient measures income inequality. This combination shows whether growth is benefiting the average citizen or only a few.
//     Job Openings ("Help Wanted" Ads) + Quit Rates (The Labor Dynamics Duo): Rather than just looking at the unemployment rate, this combination tracks "labor market tightness"—how easily people can find work and how confident they feel switching jobs.
//     Government Debt-to-GDP Ratio + Fiscal Deficit/Surplus (The Sustainability Check): This pair indicates long-term fiscal sustainability, measuring a government’s debt load against its capacity to repay (GDP).

// These combinations are used by economists, businesses, and investors to gauge the business cycle, identify turning points, and make informed decisions about future economic conditions.

// For analyzing economic health, these formulas provide the mathematical foundation to interpret raw data and compare it across different time periods or regions.
// 1. Productivity and Living Standards

// * GDP (Expenditure Method): $C + I + G + (X - M)$
// * $C$ = Consumer Spending, $I$ = Investment, $G$ = Government Spending, $X$ = Exports, $M$ = Imports.
// * GDP per Capita: $\frac{\text{Total GDP}}{\text{Total Population}}$
// * Essential for comparing the standard of living between countries with different population sizes.
// * Real GDP: $\frac{\text{Nominal GDP}}{\text{GDP Deflator}}$
// * Adjusts for inflation to show the actual volume of production. [1, 2, 3, 4, 5]

// 2. Inflation and Purchasing Power

// * Inflation Rate: $\left( \frac{\text{CPI}_{\text{Current}} - \text{CPI}_{\text{Prior}}}{\text{CPI}_{\text{Prior}}} \right) \times 100$
// * Measures the percentage change in price levels over a specific period.
// * Real Interest Rate: $\text{Nominal Interest Rate} - \text{Inflation Rate}$
// * Shows the actual cost of borrowing or the true return on savings. [2, 6, 7]

// 3. Labour Market Health

// * Unemployment Rate: $\left( \frac{\text{Number of Unemployed}}{\text{Total Labour Force}} \right) \times 100$
// * Note: Labour Force = Employed + Unemployed persons.
// * Labour Force Participation Rate: $\left( \frac{\text{Labour Force}}{\text{Working-Age Population}} \right) \times 100$
// * Measures the percentage of the population that is either working or actively seeking work. [2, 8, 9, 10, 11, 12]

// 4. Business and Market Sentiment

// * Purchasing Managers' Index (PMI): $P_1 + (0.5 \times P_2)$
// * $P_1$ = % reporting improvement, $P_2$ = % reporting no change.
//    * A reading above 50 indicates expansion; below 50 indicates contraction.
// * Price-to-Earnings (P/E) Ratio: $\frac{\text{Share Price}}{\text{Earnings Per Share (EPS)}}$
// * Used by investors to determine if a market or company is overvalued. [13, 14, 15, 16, 17, 18]

// 5. Corporate Economic Value

// * Economic Value Added (EVA): $\text{NOPAT} - (\text{WACC} \times \text{Invested Capital})$
// * $\text{NOPAT}$ = Net Operating Profit After Tax, $\text{WACC}$ = Weighted Average Cost of Capital.
//    * Measures true profit after accounting for the full cost of capital. [19, 20]

// Would you like a deeper breakdown of sector-specific formulas, such as those used in the banking or real estate sectors?

// 1. Housing Affordability and Stress
// The most widely used metric in Australia for housing policy is the 30/40 rule.

//     Housing Stress Formula:
//     .
//         Context: This is typically only applied to the bottom 40% of the income distribution.
//     Rental Affordability Index (RAI):
//     .
//         An index score of 100 represents the threshold where households pay exactly 30% of income on rent.
//     Mortgage Affordability Indicator:
//     .
//         Repayments exceeding 30% of income are often flagged as "mortgage stress".

// 2. Cost of Living
// Analysis of cost of living in Australia often uses specific indexes beyond the standard Consumer Price Index (CPI).

//     Selected Living Cost Indexes (SLCIs): Unlike the CPI, these indexes include mortgage interest charges, making them more reflective of the actual out-of-pocket expenses for different household types (e.g., employee households vs. age pensioners).
//     Real Wage Growth:
//         If the result is negative, purchasing power is declining despite nominal wage increases.

// 3. Environmental Economic Analysis
// Australia uses the System of Environmental-Economic Accounting (SEEA) to link environmental data with economic performance.

//     Resource Intensity Ratio:
//     .
//         This measures how many resources (e.g., water, energy) are required to produce one unit of economic value.
//     Carbon Storage Valuation: Australia's first National Ecosystem Accounts value ecosystem services, such as carbon storage (e.g., 34.5 million kilotonnes of carbon stored valued at ~$43.2 billion).

// Recommended Data Tools for Your Analysis
// Topic 	Recommended Source/Tool
// Housing	AIHW Housing Data Dashboard
// Living Costs	ABS Selected Living Cost Indexes
// Environment	Environmental-Economic Accounts (EEA) Dashboard
// Social Policy	ANU PolicyMod Microsimulation (for future projections)

// <Chart
// aspect_ratio=AspectRatio::from_env_width(500.)
// series=Series::new(|data: &Data|  data.date )
// .line(Line::new(|data: &Data| data.y).with_name("CPI").with_colour(Colour::from_rgb(0, 0x88, 0)))
// .line(Line::new(|data: &Data| data.ir).with_name("CRT").with_gradient(LINEAR_GRADIENT))
// data=observations
// top=Legend::start()
// // <!-- left=TickLabels::aligned_floats() -->
// left=TickLabels::aligned_floats()

// // <!-- bottom=x_ticks.clone()
// // -->
// bottom=x_ticks.clone()
//   // .with_strftime("%MMM %y".to_string())
// tooltip=Tooltip::left_cursor()
// inner=[
//   // AxisMarker::left_edge().into_inner(),
//   // AxisMarker::bottom_edge().into_inner(),
//   XGridLine::from_ticks(x_ticks).with_colour(Colour::from_rgb(0xA0,0xA0,0xA0)).into_inner(),
//   YGridLine::default().with_colour(Colour::from_rgb(0xA0,0xA0,0xA0)).into_inner(),
//   YGuideLine::over_mouse().into_inner(),
//   XGuideLine::over_data().into_inner(),
// ]
// font_height=Signal::stored(22.)
// />
