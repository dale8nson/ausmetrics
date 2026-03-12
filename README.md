# ausmetrics

A full-stack Australian public policy data platform — aggregating economic, environmental, demographic, and social indicators from government and independent sources, with a GraphQL API and a Leptos/WebAssembly frontend.

> **Status: Work in progress.** The GraphQL server fetches live data from the ABS SDMX REST API and RBA CSV data; the frontend renders interactive charts using ECharts via the `charming` crate. Active development is ongoing.

## Vision

`ausmetrics` aims to be a single place to explore the data behind Australia's most pressing public policy questions — cost of living, housing, climate, immigration, fiscal policy, and more. Data is drawn from both government APIs and independent research institutions, prioritising sources that are neutral, methodologically rigorous, and publicly accessible.

The longer-term roadmap includes an MCP server integration, so that the dataset can be queried directly by AI assistants.

## Data

### Currently implemented

| Metric | Source | Notes |
|--------|--------|-------|
| CPI (Consumer Price Index) | ABS SDMX REST API | Quarterly |
| Cash Rate Target | RBA (CSV) | Historical series |
| Mean Dwelling Price by State | ABS SDMX REST API | Per state: NSW, VIC, QLD, SA, WA, TAS, NT |
| Gross Domestic Product | ABS SDMX REST API | Quarterly |

### Planned

| Domain | Metrics | Sources |
|--------|---------|---------|
| **Housing** | Rental vacancy rates, social housing waitlists, homelessness estimates, affordability ratios | ABS, AIHW, AHURI, CoreLogic |
| **Environment** | Emissions by sector, renewable energy share, temperature/rainfall anomalies, land clearing | Clean Energy Regulator, BOM, CSIRO, Our World in Data |
| **Immigration** | Net overseas migration, visa grants by category, population growth by state | ABS, Dept. of Home Affairs, OECD |
| **Finance & fiscal policy** | Federal budget balance, government debt, tax revenue, welfare expenditure | Treasury, APRA, IMF, Grattan Institute |
| **Labour** | Unemployment and underemployment, wages growth, industry employment share | ABS, Melbourne Institute (HILDA Survey) |
| **Health & social** | Bulk billing rates, hospital wait times, aged care, Closing the Gap indicators | AIHW, Grattan Institute |

#### Data source notes

`ausmetrics` draws from two categories of source:

**Government & statutory bodies** — ABS, RBA, APRA, Treasury, Bureau of Meteorology, Clean Energy Regulator, AIHW (Australian Institute of Health and Welfare), and the Department of Home Affairs. These provide the primary time-series datasets via REST and SDMX APIs.

**Independent research institutions** — selected for methodological rigour and editorial independence:

- [Grattan Institute](https://grattan.edu.au) — Australia's leading independent policy think tank, covering housing, health, energy, and fiscal policy
- [AHURI](https://www.ahuri.edu.au) (Australian Housing and Urban Research Institute) — independent housing and urban research
- [Melbourne Institute / HILDA Survey](https://melbourneinstitute.unimelb.edu.au/hilda) — longitudinal household income, labour, and welfare data (University of Melbourne)
- [CSIRO](https://www.csiro.au) — national science agency; independent on climate and environmental data
- [Our World in Data](https://ourworldindata.org) — open, peer-reviewed global indicators with strong Australia coverage across most domains
- [OECD](https://data.oecd.org) — international comparative data for Australia across economics, immigration, health, and education
- [IMF](https://www.imf.org/en/Data) — macroeconomic and fiscal data for cross-checking national accounts

## Architecture

```
  ┌──────────────────────────────────────────┐
  │  ABS SDMX REST API · RBA · Data.gov.au   │
  │  Clean Energy Regulator · Home Affairs   │
  └──────────────────┬───────────────────────┘
                     │ reqwest (async HTTP)
  ┌──────────────────▼───────────────────────┐
  │            graphql-server                │
  │         (actix-web + async-graphql)      │
  └──────────┬───────────────────────────────┘
             │ GraphQL over HTTP
      ┌──────┴──────────────┐
      │                     │
  ┌───▼──────────┐   ┌──────▼──────────────┐
  │   frontend   │   │   MCP server        │
  │  (Leptos 0.8 │   │   (planned)         │
  │   + WASM)    │   │                     │
  └──────────────┘   └─────────────────────┘
```

## Workspace layout

```
ausmetrics/
├── graphql-server/       # Actix-web GraphQL server
│   └── src/
│       ├── main.rs       # Server entry point, GraphQL endpoint at /
│       ├── schema.rs     # Query resolvers — ABS/RBA data fetching
│       ├── graph.rs      # Graph utilities
│       ├── param.rs      # Query parameter helpers
│       └── error.rs      # Error types
└── frontend/             # Leptos full-stack app
    └── src/
        └── components/
            └── line_chart.rs   # CPI vs Cash Rate line chart (ECharts)
```

## Running

### GraphQL server

Create a `.env.local` file in `graphql-server/`:
```
GRAPHQL_ADDR=127.0.0.1
PORT=4000
```

```bash
cd graphql-server
cargo run
```

GraphiQL playground: `http://127.0.0.1:4000/`

### Frontend

Requires [cargo-leptos](https://github.com/leptos-rs/cargo-leptos):

```bash
cargo install cargo-leptos
cd frontend
cargo leptos watch
```

Frontend: `http://127.0.0.1:3010/`

## Example query

```graphql
{
  consumerPriceIndex(start: {year: 2025, month: 1}, end: {year: 2025, month: 12})
  cashRateTarget(start: {year: 2025, month: 1}, end: {year: 2025, month: 12})
}
```

## Built with

- [`actix-web`](https://github.com/actix/actix-web) — HTTP server
- [`async-graphql`](https://github.com/async-graphql/async-graphql) — GraphQL (with dynamic schema support)
- [`leptos`](https://github.com/leptos-rs/leptos) 0.8 — reactive full-stack WASM framework
- [`charming`](https://github.com/yuankunzhang/charming) — Apache ECharts bindings for Rust
- [`reqwest`](https://github.com/seanmonstar/reqwest) — async HTTP client for data fetching
- [`jsonpath-rust`](https://github.com/besok/jsonpath-rust) — JSONPath extraction from ABS SDMX responses
- Tailwind CSS — utility-first styling
