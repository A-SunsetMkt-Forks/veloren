/// this contains global housekeeping info during simulation
use crate::{
    Index,
    site::economy::{DAYS_PER_MONTH, DAYS_PER_YEAR, Economy, INTER_SITE_TRADE},
};
use rayon::prelude::*;
use tracing::{debug, info};

// this is an empty replacement for https://github.com/cpetig/vergleich
// which can be used to compare values acros runs
// pub mod vergleich {
//     pub struct Error {}
//     impl Error {
//         pub fn to_string(&self) -> &'static str { "" }
//     }
//     pub struct ProgramRun {}
//     impl ProgramRun {
//         pub fn new(_: &str) -> Result<Self, Error> { Ok(Self {}) }

//         pub fn set_epsilon(&mut self, _: f32) {}

//         pub fn context(&mut self, _: &str) -> Context { Context {} }

//         //pub fn value(&mut self, _: &str, val: f32) -> f32 { val }
//     }
//     pub struct Context {}
//     impl Context {
//         #[must_use]
//         pub fn context(&mut self, _: &str) -> Context { Context {} }

//         pub fn value(&mut self, _: &str, val: f32) -> f32 { val }

//         pub fn dummy() -> Self { Context {} }
//     }
// }

const TICK_PERIOD: f32 = 3.0 * DAYS_PER_MONTH; // 3 months
const HISTORY_DAYS: f32 = 500.0 * DAYS_PER_YEAR; // 500 years

/// Statistics collector (min, max, avg)
#[derive(Debug)]
struct EconStatistics {
    count: u32,
    sum: f32,
    min: f32,
    max: f32,
}

impl Default for EconStatistics {
    fn default() -> Self {
        Self {
            count: 0,
            sum: 0.0,
            min: f32::INFINITY,
            max: -f32::INFINITY,
        }
    }
}

impl std::ops::AddAssign<f32> for EconStatistics {
    fn add_assign(&mut self, rhs: f32) { self.collect(rhs); }
}

impl EconStatistics {
    fn collect(&mut self, value: f32) {
        self.count += 1;
        self.sum += value;
        if value > self.max {
            self.max = value;
        }
        if value < self.min {
            self.min = value;
        }
    }

    fn valid(&self) -> bool { self.min.is_finite() }
}

pub struct Environment {
    csv_file: Option<std::fs::File>,
    // context: vergleich::ProgramRun,
}

impl Environment {
    pub fn new() -> Result<Self, std::io::Error> {
        // let mut context = vergleich::ProgramRun::new("economy_compare.sqlite")
        //     .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other,
        // e.to_string()))?; context.set_epsilon(0.1);
        let csv_file = Economy::csv_open();
        Ok(Self {
            csv_file, /* context */
        })
    }

    fn iteration(&mut self, _: i32) {}

    fn end(mut self, index: &Index) {
        if let Some(f) = self.csv_file.as_mut() {
            use std::io::Write;
            let err = writeln!(f);
            if err.is_ok() {
                for site in index.sites.ids() {
                    let site = index.sites.get(site);
                    if Economy::csv_entry(f, site).is_err() {
                        break;
                    }
                }
            }
            self.csv_file.take();
        }

        {
            let mut towns = EconStatistics::default();
            let dungeons = EconStatistics::default();
            for site in index.sites.ids() {
                let site = &index.sites[site];
                if let Some(econ) = site.economy.as_ref() {
                    towns += econ.pop;
                }
            }
            if towns.valid() {
                info!(
                    "Towns {:.0}-{:.0} avg {:.0} inhabitants",
                    towns.min,
                    towns.max,
                    towns.sum / (towns.count as f32)
                );
            }
            if dungeons.valid() {
                info!(
                    "Dungeons {:.0}-{:.0} avg {:.0}",
                    dungeons.min,
                    dungeons.max,
                    dungeons.sum / (dungeons.count as f32)
                );
            }
        }
    }

    fn csv_tick(&mut self, index: &Index) {
        if let Some(f) = self.csv_file.as_mut() {
            if let Some(site) = index.sites.values().find(|s| s.do_economic_simulation()) {
                Economy::csv_entry(f, site).unwrap_or_else(|_| {
                    self.csv_file.take();
                });
            }
        }
    }
}

fn simulate_return(index: &mut Index) -> Result<(), std::io::Error> {
    let mut env = Environment::new()?;

    info!("economy simulation start");
    for i in 0..(HISTORY_DAYS / TICK_PERIOD) as i32 {
        if (index.time / DAYS_PER_YEAR) as i32 % 50 == 0 && (index.time % DAYS_PER_YEAR) as i32 == 0
        {
            debug!("Year {}", (index.time / DAYS_PER_YEAR) as i32);
        }
        env.iteration(i);
        tick(index, TICK_PERIOD, &mut env);
        if i % 5 == 0 {
            env.csv_tick(index);
        }
    }
    info!("economy simulation end");
    env.end(index);
    //    csv_footer(f, index);

    Ok(())
}

pub fn simulate_economy(index: &mut Index) {
    simulate_return(index)
        .unwrap_or_else(|err| info!("I/O error in simulate (economy.csv not writable?): {}", err));
}

// fn check_money(index: &Index) {
//     let mut sum_stock: f32 = 0.0;
//     for site in index.sites.values() {
//         sum_stock += site.economy.stocks[*COIN_INDEX];
//     }
//     let mut sum_del: f32 = 0.0;
//     for v in index.trade.deliveries.values() {
//         for del in v.iter() {
//             sum_del += del.amount[*COIN_INDEX];
//         }
//     }
//     info!(
//         "Coin amount {} + {} = {}",
//         sum_stock,
//         sum_del,
//         sum_stock + sum_del
//     );
// }

fn tick(index: &mut Index, dt: f32, _env: &mut Environment) {
    if INTER_SITE_TRADE {
        // move deliverables to recipient cities
        for (id, deliv) in index.trade.deliveries.drain() {
            index
                .sites
                .get_mut(id)
                .economy_mut()
                .deliveries
                .extend(deliv);
        }
    }
    index.sites.par_iter_mut().for_each(|(site_id, site)| {
        if site.do_economic_simulation() {
            site.economy_mut().tick(site_id, dt);
            // helpful for debugging but not compatible with parallel execution
            // vc.context(&site_id.id().to_string()));
        }
    });
    if INTER_SITE_TRADE {
        // distribute orders (travelling merchants)
        for (_id, site) in index.sites.iter_mut() {
            for (i, mut v) in site.economy_mut().orders.drain() {
                index.trade.orders.entry(i).or_default().append(&mut v);
            }
        }
        // trade at sites
        for (&site, orders) in index.trade.orders.iter_mut() {
            let siteinfo = index.sites.get_mut(site);
            if siteinfo.do_economic_simulation() {
                siteinfo
                    .economy_mut()
                    .trade_at_site(site, orders, &mut index.trade.deliveries);
            }
        }
    }
    //check_money(index);

    index.time += dt;
}

#[cfg(test)]
mod tests {
    use crate::{sim, util::seed_expan};
    use common::{
        store::Id,
        terrain::{BiomeKind, site::SiteKindMeta},
        trade::Good,
    };
    use hashbrown::HashMap;
    use rand::{Rng, SeedableRng};
    use rand_chacha::ChaChaRng;
    use serde::{Deserialize, Serialize};
    use std::convert::TryInto;
    use tracing::{Dispatch, Level, info};
    use tracing_subscriber::{FmtSubscriber, filter::EnvFilter};
    use vek::Vec2;

    fn execute_with_tracing(level: Level, func: fn()) {
        tracing::dispatcher::with_default(
            &Dispatch::new(
                FmtSubscriber::builder()
                    .with_max_level(level)
                    .with_env_filter(EnvFilter::from_default_env())
                    .finish(),
            ),
            func,
        );
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct ResourcesSetup {
        good: Good,
        amount: f32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct EconomySetup {
        name: String,
        position: (i32, i32),
        kind: common::terrain::site::SiteKindMeta,
        neighbors: Vec<u64>, // id
        resources: Vec<ResourcesSetup>,
    }

    fn show_economy(
        sites: &common::store::Store<crate::site::Site>,
        names: &Option<HashMap<Id<crate::site::Site>, String>>,
    ) {
        for (id, site) in sites.iter() {
            let name = names.as_ref().map_or(site.name().into(), |map| {
                map.get(&id).cloned().unwrap_or_else(|| site.name().into())
            });
            println!("Site id {:?} name {}", id.id(), name);
            if let Some(econ) = site.economy.as_ref() {
                econ.print_details();
            }
        }
    }

    /// output the economy of the currently active world
    // this expensive test is for manual inspection, not to be run automated
    // recommended command: cargo test test_economy0 -- --nocapture --ignored
    #[test]
    #[ignore]
    fn test_economy0() {
        execute_with_tracing(Level::INFO, || {
            let threadpool = rayon::ThreadPoolBuilder::new().build().unwrap();
            info!("init");
            let seed = sim::DEFAULT_WORLD_SEED;
            let opts = sim::WorldOpts {
                seed_elements: true,
                world_file: sim::FileOpts::LoadAsset(sim::DEFAULT_WORLD_MAP.into()),
                //sim::FileOpts::LoadAsset("world.map.economy_8x8".into()),
                calendar: None,
            };
            let mut index = crate::index::Index::new(seed);
            info!("Index created");
            let mut sim = sim::WorldSim::generate(seed, opts, &threadpool, &|_| {});
            info!("World loaded");
            let _civs = crate::civ::Civs::generate(seed, &mut sim, &mut index, None, &|_| {});
            info!("Civs created");
            crate::sim2::simulate(&mut index, &mut sim);
            show_economy(&index.sites, &None);
        });
    }

    /// output the economy of a small set of villages, loaded from ron
    // this cheaper test is for manual inspection, not to be run automated
    #[test]
    #[ignore]
    fn test_economy1() {
        execute_with_tracing(Level::INFO, || {
            let threadpool = rayon::ThreadPoolBuilder::new().build().unwrap();
            info!("init");
            let seed = sim::DEFAULT_WORLD_SEED;
            let opts = sim::WorldOpts {
                seed_elements: true,
                world_file: sim::FileOpts::LoadAsset(sim::DEFAULT_WORLD_MAP.into()),
                //sim::FileOpts::LoadAsset("world.map.economy_8x8".into()),
                calendar: None,
            };
            let mut index = crate::index::Index::new(seed);
            info!("Index created");
            let mut sim = sim::WorldSim::generate(seed, opts, &threadpool, &|_| {});
            info!("World loaded");
            let mut names = None;
            let regenerate_input = false;
            if regenerate_input {
                let _civs = crate::civ::Civs::generate(seed, &mut sim, &mut index, None, &|_| {});
                info!("Civs created");
                let mut outarr: Vec<EconomySetup> = Vec::new();
                for i in index.sites.values() {
                    let Some(econ) = i.economy.as_ref() else {
                        continue;
                    };
                    let resources: Vec<ResourcesSetup> = econ
                        .natural_resources
                        .chunks_per_resource
                        .iter()
                        .map(|(good, a)| ResourcesSetup {
                            good: good.into(),
                            amount: *a * econ.natural_resources.average_yield_per_chunk[good],
                        })
                        .collect();
                    let neighbors = econ.neighbors.iter().map(|j| j.id.id()).collect();
                    let val = EconomySetup {
                        name: i.name().into(),
                        position: (i.origin.x, i.origin.y),
                        resources,
                        neighbors,
                        kind: i.meta().unwrap_or_default(),
                    };
                    outarr.push(val);
                }
                let pretty = ron::ser::PrettyConfig::new();
                if let Ok(result) = ron::ser::to_string_pretty(&outarr, pretty) {
                    info!("RON {}", result);
                }
            } else {
                let mut rng = ChaChaRng::from_seed(seed_expan::rng_state(seed));
                let ron_file = std::fs::File::open("economy_testinput2.ron")
                    .expect("economy_testinput2.ron not found");
                let econ_testinput: Vec<EconomySetup> =
                    ron::de::from_reader(ron_file).expect("economy_testinput2.ron parse error");
                names = Some(HashMap::new());
                let land = crate::Land::from_sim(&sim);
                let mut meta = crate::site::SitesGenMeta::new(rng.gen());
                for i in econ_testinput.iter() {
                    let wpos = Vec2 {
                        x: i.position.0,
                        y: i.position.1,
                    };
                    // this should be a moderate compromise between regenerating the full world and
                    // loading on demand using the public API. There is no way to set
                    // the name, do we care?
                    let mut settlement = match i.kind {
                        SiteKindMeta::Castle => {
                            crate::site::Site::generate_citadel(&land, &mut rng, wpos)
                        },
                        _ => crate::site::Site::generate_city(
                            &land,
                            crate::IndexRef {
                                colors: &index.colors(),
                                features: &index.features(),
                                index: &index,
                            },
                            &mut rng,
                            wpos,
                            1.0,
                            None,
                            &mut meta,
                        ),
                    };
                    for g in i.resources.iter() {
                        //let c = sim::SimChunk::new();
                        //settlement.economy.add_chunk(ch, distance_squared)
                        // bypass the API for now
                        settlement
                            .economy_mut()
                            .natural_resources
                            .chunks_per_resource[g.good.try_into().unwrap_or_default()] = g.amount;
                        settlement
                            .economy_mut()
                            .natural_resources
                            .average_yield_per_chunk[g.good.try_into().unwrap_or_default()] = 1.0;
                    }
                    let id = index.sites.insert(settlement);
                    names.as_mut().map(|map| map.insert(id, i.name.clone()));
                }
                // we can't add these in the first loop as neighbors will refer to later sites
                // (which aren't valid in the first loop)
                for (id, econ) in econ_testinput.iter().enumerate() {
                    if let Some(id) = index.sites.recreate_id(id as u64) {
                        for nid in econ.neighbors.iter() {
                            if let Some(nid) = index.sites.recreate_id(*nid) {
                                let town = index.sites.get_mut(id).economy_mut();
                                town.add_neighbor(nid, 0);
                            }
                        }
                    }
                }
            }
            crate::sim2::simulate(&mut index, &mut sim);
            show_economy(&index.sites, &names);
        });
    }

    struct Simenv {
        index: crate::index::Index,
        sim: sim::WorldSim,
        rng: ChaChaRng,
        targets: HashMap<Id<crate::site::Site>, f32>,
        names: HashMap<Id<crate::site::Site>, String>,
    }

    #[test]
    /// test whether a site in moderate climate can survive on its own
    fn test_economy_moderate_standalone() {
        fn add_settlement(
            env: &mut Simenv,
            name: &str,
            target: f32,
            resources: &[(Good, f32)],
        ) -> Id<crate::site::Site> {
            let wpos = Vec2 { x: 42, y: 42 };
            let mut meta = crate::site::SitesGenMeta::new(env.rng.gen());
            let mut settlement = crate::site::Site::generate_city(
                &crate::Land::from_sim(&env.sim),
                crate::IndexRef {
                    colors: &env.index.colors(),
                    features: &env.index.features(),
                    index: &env.index,
                },
                &mut env.rng,
                wpos,
                1.0,
                None,
                &mut meta,
            );
            for (good, amount) in resources.iter() {
                settlement
                    .economy_mut()
                    .natural_resources
                    .chunks_per_resource[(*good).try_into().unwrap_or_default()] = *amount;
                settlement
                    .economy_mut()
                    .natural_resources
                    .average_yield_per_chunk[(*good).try_into().unwrap_or_default()] = 1.0;
            }
            let id = env.index.sites.insert(settlement);
            env.targets.insert(id, target);
            env.names.insert(id, name.into());
            id
        }

        execute_with_tracing(Level::ERROR, || {
            let threadpool = rayon::ThreadPoolBuilder::new().build().unwrap();
            info!("init");
            let seed = sim::DEFAULT_WORLD_SEED;
            let opts = sim::WorldOpts {
                seed_elements: true,
                world_file: sim::FileOpts::LoadAsset(sim::DEFAULT_WORLD_MAP.into()),
                calendar: Default::default(),
            };
            let index = crate::index::Index::new(seed);
            info!("Index created");
            let sim = sim::WorldSim::generate(seed, opts, &threadpool, &|_| {});
            info!("World loaded");
            let rng = ChaChaRng::from_seed(seed_expan::rng_state(seed));
            let mut env = Simenv {
                index,
                sim,
                rng,
                targets: HashMap::new(),
                names: HashMap::new(),
            };
            add_settlement(&mut env, "Forest", 5000.0, &[(
                Good::Terrain(BiomeKind::Forest),
                100.0_f32,
            )]);
            add_settlement(&mut env, "Grass", 700.0, &[(
                Good::Terrain(BiomeKind::Grassland),
                100.0_f32,
            )]);
            add_settlement(&mut env, "Mountain", 3.0, &[(
                Good::Terrain(BiomeKind::Mountain),
                100.0_f32,
            )]);
            // add_settlement(&mut env, "Desert", 19.0, &[(
            //     Good::Terrain(BiomeKind::Desert),
            //     100.0_f32,
            // )]);
            // add_settlement(&mut index, &mut rng, &[
            //     (Good::Terrain(BiomeKind::Jungle), 100.0_f32),
            // ]);
            // add_settlement(&mut index, &mut rng, &[
            //     (Good::Terrain(BiomeKind::Snowland), 100.0_f32),
            // ]);
            add_settlement(&mut env, "GrFoMo", 12000.0, &[
                (Good::Terrain(BiomeKind::Grassland), 100.0_f32),
                (Good::Terrain(BiomeKind::Forest), 100.0_f32),
                (Good::Terrain(BiomeKind::Mountain), 10.0_f32),
            ]);
            // add_settlement(&mut env, "Mountain", 19.0, &[
            //     (Good::Terrain(BiomeKind::Mountain), 100.0_f32),
            //     // (Good::CaveAccess, 100.0_f32),
            // ]);
            // connect to neighbors (one way)
            for i in 1..(env.index.sites.ids().count() as u64 - 1) {
                let previous = env.index.sites.recreate_id(i - 1);
                let center = env.index.sites.recreate_id(i);
                center.zip(previous).map(|(center, previous)| {
                    env.index.sites[center]
                        .economy_mut()
                        .add_neighbor(previous, i as usize);
                    env.index.sites[previous]
                        .economy_mut()
                        .add_neighbor(center, i as usize);
                });
            }
            crate::sim2::simulate(&mut env.index, &mut env.sim);
            show_economy(&env.index.sites, &Some(env.names));
            // check population (shrinks if economy gets broken)
            for (id, site) in env.index.sites.iter() {
                if let Some(econ) = site.economy.as_ref() {
                    assert!(econ.pop >= env.targets[&id]);
                }
            }
        });
    }
}
