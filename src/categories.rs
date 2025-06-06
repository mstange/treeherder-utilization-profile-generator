use fxprof_processed_profile::CategoryColor::*;

use super::category_matcher::CategoryOrSubcategory;

use CategoryOrSubcategory::Category as C;
use CategoryOrSubcategory::Subcategory as S;

pub const CATEGORIES: &[(&[&str], CategoryOrSubcategory)] = &[
    (
        &["tp6m", "essential", "fenix"],
        S("Tp6", Yellow, "Fenix (essential)"),
    ),
    (
        &["tp6m", "fenix"],
        S("Tp6", Yellow, "Fenix (non-essential)"),
    ),
    (
        &["tp6m", "webextensions", "fenix"],
        S("Tp6", Yellow, "Fenix (WebExtensions)"),
    ),
    (
        &["tp6m", "essential", "chrome"],
        S("Tp6", Yellow, "Chrome (essential)"),
    ),
    (
        &["tp6m", "chrome"],
        S("Tp6", Yellow, "Chrome (non-essential)"),
    ),
    (&["tp6m"], C("Tp6", Yellow)),
    (&["tp6"], C("Tp6", Yellow)),
    (&["startup"], C("Startup", Blue)),
    (&["resource"], C("Resource", Purple)),
    (&["power"], C("Resource", Purple)),
    (&["speedometer"], S("Benchmark", Red, "Speedometer 2")),
    (&["speedometer3"], S("Benchmark", Red, "Speedometer 3")),
    (&["jetstream2"], S("Benchmark", Red, "Jetstream 2")),
    (&["jetstream3"], S("Benchmark", Red, "Jetstream 3")),
    (&["benchmark"], C("Benchmark", Red)),
    // ("chrome", C("Chrome", Yellow)),
    // ("fenix", C("Fenix", Green)),
    // ("geckoview", C("GeckoView example", Blue)),
    // ("refbrow", C("Reference browser", Blue)),
];
