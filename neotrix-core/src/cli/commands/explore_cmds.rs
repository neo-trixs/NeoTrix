use crate::cli::commands::types::{CliCommand, CommandOutput};
use crate::neotrix::l2_world_impl::nt_world_exploration_engine::{ExplorationEngine, ExplorationConfig};
use crate::neotrix::l8_autonomic_impl::nt_mind_knowledge_pipeline::KnowledgeAbsorptionPipeline;

pub struct ExploreCmd;

impl CliCommand for ExploreCmd {
    fn name(&self) -> &str { "/explore" }
    fn aliases(&self) -> Vec<&str> { vec!["/discover", "/ingest"] }
    fn description(&self) -> &str {
        "外部知识探索: /explore [cycle|status|absorb <url>|github <url>|search <q>]"
    }

    fn execute(&self, args: &[String], _brain: Option<&std::sync::Arc<tokio::sync::RwLock<crate::neotrix::nt_mind::SelfIteratingBrain>>>) -> CommandOutput {
        let mode = args.first().map(|s| s.as_str()).unwrap_or("status");

        // Try to open KB connection for pipeline ops
        fn try_open_kb() -> Option<crate::neotrix::nt_memory_kb::KnowledgeBase> {
            crate::neotrix::nt_memory_kb::KnowledgeBase::open(None).ok()
        }
        fn try_prepare_pipeline() -> KnowledgeAbsorptionPipeline {
            let mut p = KnowledgeAbsorptionPipeline::new();
            if let Some(kb) = try_open_kb() {
                p.attach_kb(std::sync::Arc::new(kb));
            }
            p
        }

        match mode {
            "cycle" | "run" => {
                let config = ExplorationConfig::default();
                let mut engine = ExplorationEngine::new(config);
                if let Some(kb) = try_open_kb() {
                    engine.attach_kb(kb.into());
                }
                let report = engine.run_cycle();
                CommandOutput::ok(&format!(
                    "🔍 探索完成: 发现 {} | 摄入 {} | 跳过 {} | 失败 {} | KB 总计 {}",
                    report.discovered, report.ingested, report.skipped, report.failed, report.total_in_kb
                ))
            }

            "status" | "stat" => {
                let engine = ExplorationEngine::new(ExplorationConfig::default());
                let s = engine.stats();
                let p = try_prepare_pipeline();
                let pstats = p.stats();
                CommandOutput::ok(&format!(
                    "📊 探索状态:\n  探索引擎: {} 轮, {} 已发现, {} 已摄入\n  知识管道: {} 源吸收",
                    s.total_cycles, s.discovered, s.ingested_urls, pstats.total_sources
                ))
            }

            "absorb" | "ingest" => {
                let url = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if url.is_empty() {
                    return CommandOutput::err("用法: /explore absorb <url>");
                }
                let mut pipeline = try_prepare_pipeline();
                match pipeline.absorb_url(url) {
                    Ok(r) => CommandOutput::ok(&format!(
                        "📥 {}: {} 节点, {} 边 ({})",
                        r.action, r.nodes_created, r.edges_created, r.url
                    )),
                    Err(e) => CommandOutput::err(&format!("吸收失败: {}", e)),
                }
            }

            "github" => {
                let url = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if url.is_empty() {
                    return CommandOutput::err("用法: /explore github <repo_url>");
                }
                let mut pipeline = try_prepare_pipeline();
                match pipeline.absorb_github(url) {
                    Ok(r) => {
                        let mut msg = format!("📦 GitHub {}: {} 节点, {} 边", r.action, r.nodes_created, r.edges_created);
                        if let Some(ref summary) = r.distil_summary {
                            msg.push_str(&format!("\n  蒸馏: {}", summary.chars().take(120).collect::<String>()));
                        }
                        CommandOutput::ok(&msg)
                    }
                    Err(e) => CommandOutput::err(&format!("GitHub 吸收失败: {}", e)),
                }
            }

            "alphaxiv" | "ax" => {
                let pages = args.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(5);
                let page_size = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(20);
                let category = args.get(3).map(|s| s.as_str()).unwrap_or("");
                match try_open_kb() {
                    Some(kb) => match kb.ingest_alphaxiv_feed(pages, page_size, category) {
                        Ok(n) => {
                            let cat_suffix = if category.is_empty() {
                                String::new()
                            } else {
                                format!(", 分类 {}", category)
                            };
                            let msg = format!(
                                "📄 alphaXiv feed: {} 篇论文入库 ({} 页 × {} 篇/页{})",
                                n, pages, page_size, cat_suffix
                            );
                            CommandOutput::ok(&msg)
                        }
                        Err(e) => CommandOutput::err(&format!("alphaXiv 吸收失败: {}", e)),
                    },
                    None => CommandOutput::err("无法打开 KB"),
                }
            }

            "hf" | "hfdataset" | "hfd" => {
                let what = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if what == "queue" {
                    let max = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(50);
                    match try_open_kb() {
                        Some(kb) => match kb.run_hf_queue_batch(max) {
                            Ok((ok, fail)) => CommandOutput::ok(&format!("🤗 HF 队列消费: {} 成功, {} 失败 (上限 {})", ok, fail, max)),
                            Err(e) => CommandOutput::err(&format!("HF 队列消费失败: {}", e)),
                        },
                        None => CommandOutput::err("无法打开 KB"),
                    }
                } else if what.is_empty() {
                    CommandOutput::err("用法: /explore hf <owner/name | url>  |  /explore hf queue [max]")
                } else {
                    match try_open_kb() {
                        Some(kb) => match kb.ingest_hf_dataset(what) {
                            Ok(n) => CommandOutput::ok(&format!("🤗 HF dataset: {} 节点入库 ({})", n, what)),
                            Err(e) => CommandOutput::err(&format!("HF dataset 吸收失败: {}", e)),
                        },
                        None => CommandOutput::err("无法打开 KB"),
                    }
                }
            }

            "wikipedia" | "wiki" => {
                let topic = args.get(1).map(|s| s.as_str()).unwrap_or("");
                if topic.is_empty() {
                    return CommandOutput::err("用法: /explore wikipedia <topic>");
                }
                match try_open_kb() {
                    Some(kb) => match kb.ingest_wikipedia(topic) {
                        Ok(n) => CommandOutput::ok(&format!("📚 Wikipedia: {} 节点入库 (主题 {})", n, topic)),
                        Err(e) => CommandOutput::err(&format!("Wikipedia 吸收失败: {}", e)),
                    },
                    None => CommandOutput::err("无法打开 KB"),
                }
            }

            "arxiv" => {
                let ids: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();
                if ids.is_empty() {
                    return CommandOutput::err("用法: /explore arxiv <id> [id...]");
                }
                match try_open_kb() {
                    Some(kb) => {
                        let mut ok = 0;
                        let mut errs: Vec<String> = Vec::new();
                        for id in ids {
                            match kb.ingest_arxiv(id) {
                                Ok(n) => ok += n,
                                Err(e) => errs.push(format!("{}: {}", id, e)),
                            }
                        }
                        if errs.is_empty() {
                            CommandOutput::ok(&format!("📄 arXiv: {} 篇论文入库", ok))
                        } else {
                            CommandOutput::ok(&format!(
                                "📄 arXiv: {} 篇入库, {} 失败: {}",
                                ok,
                                errs.len(),
                                errs.join("; ")
                            ))
                        }
                    }
                    None => CommandOutput::err("无法打开 KB"),
                }
            }

            "panorama" | "map" => {
                let mut pipeline = try_prepare_pipeline();
                match pipeline.update_panorama() {
                    Ok(p) => {
                        let types: Vec<String> = p.by_type.iter()
                            .map(|(k, v)| format!("{}: {}", k, v)).collect();
                        CommandOutput::ok(&format!(
                            "🗺️ 知识全景: {} 源 | {}\n  更新于 {}",
                            p.total_sources, types.join(" | "),
                            chrono::DateTime::from_timestamp(p.updated_at, 0)
                                .map(|d| d.format("%Y-%m-%d %H:%M:%S").to_string())
                                .unwrap_or_else(|| "unknown".into())
                        ))
                    }
                    Err(e) => CommandOutput::err(&format!("全景更新失败: {}", e)),
                }
            }

            "geo" | "geosync" => {
                let sub = args.get(1).map(|s| s.as_str()).unwrap_or("sync");
                match sub {
                    "cities" | "city" => {
                        let country = args.get(2).map(|s| s.to_string());
                        let limit = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(5000);
                        let url = "https://cdn.jsdelivr.net/npm/cities.json@1.1.2/cities.json";
                        match try_open_kb() {
                            Some(kb) => {
                                let mut conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::ingest_geo_cities(
                                    &mut conn, url, country.as_deref(), limit,
                                ) {
                                    Ok((nodes, geo)) => CommandOutput::ok(&format!(
                                        "🌆 城市坐标摄取: {} 节点 + {} geo_index 条 (country={}, limit={})",
                                        nodes, geo, country.unwrap_or_else(|| "*".into()), limit
                                    )),
                                    Err(e) => CommandOutput::err(&format!("城市坐标摄取失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "countries" | "country" => {
                        let url = "https://cdn.jsdelivr.net/npm/world-atlas@2/countries-110m.json";
                        match try_open_kb() {
                            Some(kb) => {
                                let conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::ingest_country_boundaries(&conn, url) {
                                    Ok(n) => CommandOutput::ok(&format!(
                                        "🗺️ 国家边界登记: {} 个国家 (world-atlas)",
                                        n
                                    )),
                                    Err(e) => CommandOutput::err(&format!("国家边界登记失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "peaks" | "peak" | "mountains" => {
                        // GeoNames allCountries dump (TSV) — 本地文件优先, 支持 file:// 前缀
                        let default_path = format!(
                            "{}/.neotrix/geo/allCountries.txt",
                            std::env::var("HOME").unwrap_or_default()
                        );
                        let limit = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(50000);
                        let path = args.get(3).cloned().unwrap_or_else(|| {
                            format!("file://{}", default_path)
                        });
                        match try_open_kb() {
                            Some(kb) => {
                                let mut conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::ingest_geo_peaks(
                                    &mut conn, &path, limit,
                                ) {
                                    Ok(n) => CommandOutput::ok(&format!(
                                        "⛰️ 峰点摄取: {} 条 (GeoNames T 类, limit={})",
                                        n, limit
                                    )),
                                    Err(e) => CommandOutput::err(&format!("峰点摄取失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "airports" | "airport" => {
                        // OurAirports CSV (Public Domain) — 本地文件优先, 支持 file:// 前缀
                        let default_path = format!(
                            "{}/.neotrix/geo/airports.csv",
                            std::env::var("HOME").unwrap_or_default()
                        );
                        let limit = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(40000);
                        let path = args.get(3).cloned().unwrap_or_else(|| {
                            format!("file://{}", default_path)
                        });
                        match try_open_kb() {
                            Some(kb) => {
                                let mut conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::ingest_geo_airports(
                                    &mut conn, &path, limit,
                                ) {
                                    Ok(n) => CommandOutput::ok(&format!(
                                        "✈️ 机场摄取: {} 条 (OurAirports, limit={})",
                                        n, limit
                                    )),
                                    Err(e) => CommandOutput::err(&format!("机场摄取失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "admin0" | "boundaries" => {
                        // NE Admin 0 国家边界 (Public Domain)
                        let default_path = format!(
                            "{}/.neotrix/geo/ne_10m_admin_0_countries.geojson",
                            std::env::var("HOME").unwrap_or_default()
                        );
                        let limit = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(300);
                        let path = args.get(3).cloned().unwrap_or_else(|| {
                            format!("file://{}", default_path)
                        });
                        match try_open_kb() {
                            Some(kb) => {
                                let mut conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::ingest_geo_boundaries(
                                    &mut conn, &path, "admin0", limit,
                                ) {
                                    Ok(n) => CommandOutput::ok(&format!(
                                        "🗺️ 国家边界摄取: {} 条 (NE Admin 0)",
                                        n
                                    )),
                                    Err(e) => CommandOutput::err(&format!("国家边界摄取失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "admin1" | "provinces" | "states" => {
                        // NE Admin 1 省级边界 (Public Domain)
                        let default_path = format!(
                            "{}/.neotrix/geo/ne_10m_admin_1_states_provinces.geojson",
                            std::env::var("HOME").unwrap_or_default()
                        );
                        let limit = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(5000);
                        let path = args.get(3).cloned().unwrap_or_else(|| {
                            format!("file://{}", default_path)
                        });
                        match try_open_kb() {
                            Some(kb) => {
                                let mut conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::ingest_geo_boundaries(
                                    &mut conn, &path, "admin1", limit,
                                ) {
                                    Ok(n) => CommandOutput::ok(&format!(
                                        "🏛️ 省级边界摄取: {} 条 (NE Admin 1)",
                                        n
                                    )),
                                    Err(e) => CommandOutput::err(&format!("省级边界摄取失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "volcanoes" | "volcano" => {
                        // Smithsonian GVP Holocene 火山 (WFS JSON, 官方数据库)
                        let default_path = format!(
                            "{}/.neotrix/geo/gvp_holocene_volcanoes.json",
                            std::env::var("HOME").unwrap_or_default()
                        );
                        let limit = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(2000);
                        let path = args.get(3).cloned().unwrap_or_else(|| {
                            format!("file://{}", default_path)
                        });
                        match try_open_kb() {
                            Some(kb) => {
                                let mut conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::ingest_geo_volcanoes(
                                    &mut conn, &path, limit,
                                ) {
                                    Ok(n) => CommandOutput::ok(&format!(
                                        "🌋 火山摄取: {} 条 (Smithsonian GVP Holocene)",
                                        n
                                    )),
                                    Err(e) => CommandOutput::err(&format!("火山摄取失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "rivers" | "river" => {
                        let url = "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_10m_rivers_lake_centerlines_scale_rank.geojson";
                        match try_open_kb() {
                            Some(kb) => {
                                let conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::ingest_geo_vectors(&conn, url, "river") {
                                    Ok(n) => CommandOutput::ok(&format!(
                                        "🏞️ 河流摄取: {} 条 (Natural Earth 10m)",
                                        n
                                    )),
                                    Err(e) => CommandOutput::err(&format!("河流摄取失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "lakes" | "lake" => {
                        let url = "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_10m_lakes.geojson";
                        match try_open_kb() {
                            Some(kb) => {
                                let conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::ingest_geo_vectors(&conn, url, "lake") {
                                    Ok(n) => CommandOutput::ok(&format!(
                                        "🌊 湖泊摄取: {} 条 (Natural Earth 10m)",
                                        n
                                    )),
                                    Err(e) => CommandOutput::err(&format!("湖泊摄取失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "coastline" | "coast" => {
                        let url = "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_10m_coastline.geojson";
                        match try_open_kb() {
                            Some(kb) => {
                                let conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::ingest_geo_vectors(&conn, url, "coastline") {
                                    Ok(n) => CommandOutput::ok(&format!(
                                        "🗺️ 海岸线摄取: {} 条 (Natural Earth 10m)",
                                        n
                                    )),
                                    Err(e) => CommandOutput::err(&format!("海岸线摄取失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "glaciers" | "ice" | "glaciated" => {
                        let url = "https://raw.githubusercontent.com/nvkelso/natural-earth-vector/master/geojson/ne_10m_glaciated_areas.geojson";
                        match try_open_kb() {
                            Some(kb) => {
                                let conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_crawl::ingest_geo_vectors(&conn, url, "glacier") {
                                    Ok(n) => CommandOutput::ok(&format!(
                                        "🏔️ 冰川/冰原摄取: {} 条 (Natural Earth 10m)",
                                        n
                                    )),
                                    Err(e) => CommandOutput::err(&format!("冰川摄取失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "tag" | "taggeo" => {
                        let limit = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(50000);
                        match try_open_kb() {
                            Some(kb) => {
                                let conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::geo_tag_nodes(&conn, limit) {
                                    Ok(n) => CommandOutput::ok(&format!(
                                        "🏷️ 地理标签挂载: {} 个节点 (country keyword match, limit={})",
                                        n, limit
                                    )),
                                    Err(e) => CommandOutput::err(&format!("地理标签挂载失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "tagcity" | "citytag" | "tagcities" => {
                        let limit = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(50000);
                        match try_open_kb() {
                            Some(kb) => {
                                let conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::geo_tag_cities(&conn, limit) {
                                    Ok(n) => CommandOutput::ok(&format!(
                                        "🏙️ 城市级地理标签挂载: {} 个节点 (city match, limit={})",
                                        n, limit
                                    )),
                                    Err(e) => CommandOutput::err(&format!("城市级地理标签挂载失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "links" | "nodes" | "linked" => {
                        let place = args.get(2).map(|s| s.as_str()).unwrap_or("");
                        let limit = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(30);
                        if place.is_empty() {
                            return CommandOutput::err("用法: /explore geo links <国家或城市> [limit]");
                        }
                        match try_open_kb() {
                            Some(kb) => {
                                let conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::geo_linked_nodes(&conn, place, limit) {
                                    Ok(list) => {
                                        if list.is_empty() {
                                            return CommandOutput::ok(&format!(
                                                "🔗 '{}' 暂无关联知识节点 (试试国家名: 中国/美国/日本…)",
                                                place
                                            ));
                                        }
                                        let mut lines = vec![format!("🔗 '{}' 关联知识节点 ({}):", place, list.len())];
                                        for (id, title, ntype, lat, lng) in &list {
                                            lines.push(format!(
                                                "  [{}{}] {} @ ({:.2}, {:.2})",
                                                ntype, id, title, lat, lng
                                            ));
                                        }
                                        CommandOutput::ok(&lines.join("\n"))
                                    }
                                    Err(e) => CommandOutput::err(&format!("关联查询失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "coverage" | "missing" | "gap" => {
                        let min = args.get(2).and_then(|s| s.parse::<i64>().ok()).unwrap_or(5);
                        match try_open_kb() {
                            Some(kb) => {
                                let conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::geo_coverage_report(&conn, min) {
                                    Ok(report) => {
                                        let mut lines = vec![format!(
                                            "🗺️ 区域覆盖度报告 (国家知识节点密度, 阈值={}):", min
                                        )];
                                        for (country, cnt) in &report {
                                            let flag = if *cnt < min { "⚠️ 缺失" } else { "✅" };
                                            lines.push(format!("  {} {}: {} 节点", flag, country, cnt));
                                        }
                                        if report.is_empty() {
                                            lines.push("  (无已挂载国家数据)".to_string());
                                        }
                                        CommandOutput::ok(&lines.join("\n"))
                                    }
                                    Err(e) => CommandOutput::err(&format!("覆盖度报告失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "elevation" | "elev" | "altitude" => {
                        let limit = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1000);
                        match try_open_kb() {
                            Some(kb) => {
                                let conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::fetch_elevations(&conn, limit) {
                                    Ok(n) => CommandOutput::ok(&format!(
                                        "⛰️ 海拔摄取: {} 条 (Open-Meteo, limit={})",
                                        n, limit
                                    )),
                                    Err(e) => CommandOutput::err(&format!("海拔摄取失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "weather" | "wx" => {
                        let limit = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1000);
                        match try_open_kb() {
                            Some(kb) => {
                                let conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::fetch_weather_snapshot(&conn, limit) {
                                    Ok(n) => CommandOutput::ok(&format!(
                                        "🌦️ 气象快照摄取: {} 条 (Open-Meteo, limit={})",
                                        n, limit
                                    )),
                                    Err(e) => CommandOutput::err(&format!("气象快照摄取失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "export" | "pack" => {
                        // NT-Pack 高密度导出 (R-P79 接线: nt_memory_pack 生产消费者)
                        // 用法: /explore geo export [source] [limit] [path]
                        let source = args.get(2).map(|s| s.to_string());
                        let limit = args.get(3).and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
                        let default_path = format!(
                            "{}/.neotrix/geo/geo_index.ntpack",
                            std::env::var("HOME").unwrap_or_else(|_| ".".into())
                        );
                        let path = args.get(4).cloned().unwrap_or(default_path);
                        match try_open_kb() {
                            Some(kb) => {
                                let conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::export_geo_ntpack(
                                    &conn, source.as_deref(), limit, &path,
                                ) {
                                    Ok((n, bytes)) => CommandOutput::ok(&format!(
                                        "📦 NT-Pack 导出: {} 条 -> {} ({} bytes, {:.1} B/记录)",
                                        n, path, bytes, bytes as f64 / n as f64
                                    )),
                                    Err(e) => CommandOutput::err(&format!("NT-Pack 导出失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "import" | "restore" => {
                        // NT-Pack 导入回 KB (备份恢复/跨机器传输)
                        // 用法: /explore geo import <path>
                        let Some(path) = args.get(2).cloned() else {
                            return CommandOutput::err("用法: /explore geo import <path>");
                        };
                        match try_open_kb() {
                            Some(kb) => {
                                let conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::import_geo_ntpack_to_kb(
                                    &conn, &path,
                                ) {
                                    Ok(n) => CommandOutput::ok(&format!(
                                        "📥 NT-Pack 导入: {} 条 -> geo_index (幂等 upsert)",
                                        n
                                    )),
                                    Err(e) => CommandOutput::err(&format!("NT-Pack 导入失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "archive" => {
                        // B1 冷存储: 将指定 source 归档为 NT-Pack 冷层并从热表删除
                        // 用法: /explore geo archive <source> [path]
                        let Some(source) = args.get(2).cloned() else {
                            return CommandOutput::err("用法: /explore geo archive <source> [path]");
                        };
                        let default_path = format!(
                            "{}/.neotrix/geo/geo_{}.ntpack",
                            std::env::var("HOME").unwrap_or_else(|_| ".".into()),
                            source
                        );
                        let path = args.get(3).cloned().unwrap_or(default_path);
                        match try_open_kb() {
                            Some(kb) => {
                                let conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                                match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::archive_geo_cold(
                                    &conn, &source, &path,
                                ) {
                                    Ok((n, bytes)) => CommandOutput::ok(&format!(
                                        "❄️ 冷归档: {} 条 -> {} ({} bytes, {:.1} B/记录), 已从热表删除",
                                        n, path, bytes, bytes as f64 / n as f64
                                    )),
                                    Err(e) => CommandOutput::err(&format!("冷归档失败: {}", e)),
                                }
                            }
                            None => CommandOutput::err("无法打开 KB"),
                        }
                    }
                    "cold" => {
                        // B1 冷存储: 枚举冷层归档文件
                        // 用法: /explore geo cold
                        let dir = format!(
                            "{}/.neotrix/geo",
                            std::env::var("HOME").unwrap_or_else(|_| ".".into())
                        );
                        match crate::neotrix::l3_memory_impl::nt_memory_kb::nt_memory_geo::geo_cold_layers(&dir) {
                            Ok(layers) => {
                                if layers.is_empty() {
                                    CommandOutput::ok("冷层为空 (无 geo_*.ntpack 归档)")
                                } else {
                                    let lines: Vec<String> = layers
                                        .iter()
                                        .map(|(s, p, b)| format!("  {}: {} bytes @ {}", s, b, p))
                                        .collect();
                                    CommandOutput::ok(&format!("❄️ 冷层 ({}):\n{}", layers.len(), lines.join("\n")))
                                }
                            }
                            Err(e) => CommandOutput::err(&format!("枚举冷层失败: {}", e)),
                        }
                    }
                    _ => match try_open_kb() {
                        Some(kb) => {
                            let conn = match kb.conn.lock() {
                                    Ok(c) => c,
                                    Err(e) => return CommandOutput::err(&format!("KB 锁失败: {}", e)),
                                };
                            match crate::neotrix::nt_shanhai_geo::geo_sync::sync_shanhai_to_geo(&conn) {
                                Ok(n) => CommandOutput::ok(&format!(
                                    "🌍 地理索引同步: {} 条坐标入库 (shanhai 山峰+全球映射)",
                                    n
                                )),
                                Err(e) => CommandOutput::err(&format!("地理索引同步失败: {}", e)),
                            }
                        }
                        None => CommandOutput::err("无法打开 KB"),
                    },
                }
            }

            "recent" | "last" => {
                let pipeline = try_prepare_pipeline();
                let recent = pipeline.recent_sources(10);
                if recent.is_empty() {
                    return CommandOutput::ok("📭 暂无吸收记录");
                }
                let mut lines = vec!["📋 最近吸收:".to_string()];
                for e in recent {
                    lines.push(format!("  {} | {} | {}", e.source_type, e.title, e.url));
                }
                CommandOutput::ok(&lines.join("\n"))
            }

            _ => {
                CommandOutput::ok(
                    "🔍 外部探索命令:\n\
                    /explore status         探索状态\n\
                    /explore cycle          执行一次探索循环\n\
                    /explore absorb <url>   吸收 URL 到知识库\n\
                    /explore github <url>   吸收 GitHub 仓库 + 蒸馏\n\
                    /explore alphaxiv [pages] [page_size] [category]  吸收 alphaXiv 论文 feed (category: ai-ml/q-bio/q-fin)\n\
                    /explore panorama       查看知识全景\n\
                    /explore recent         最近吸收记录\n\
                    /explore geo            同步 shanhai 坐标到地理索引\n\
                    /explore geo cities [country] [limit]  摄取全球城市坐标\n\
                    /explore geo countries  登记国家边界 (world-atlas)\n\
                    /explore geo peaks [limit] [path]  摄取 GeoNames 峰点 (T 类, 默认 ~/.neotrix/geo/allCountries.txt)\n\
                    /explore geo airports [limit] [path]  摄取全球机场 (OurAirports, Public Domain)\n\
                    /explore geo admin0 [limit] [path]    摄取国家边界 (NE Admin 0 GeoJSON)\n\
                    /explore geo admin1 [limit] [path]    摄取省级边界 (NE Admin 1 GeoJSON)\n\
                    /explore geo volcanoes [limit] [path] 摄取火山 (Smithsonian GVP, ~1200座)\n\
                    /explore geo rivers     河流摄取 (10m, 4000+条)\n\
                    /explore geo lakes      湖泊摄取 (10m, 1300+条)\n\
                    /explore geo coastline  海岸线摄取 (10m, 4100+条)\n\
                    /explore geo glaciers   冰川/冰原摄取 (10m, 1900+条)\n\
                    /explore geo tag [limit]       国家关键词挂地理标签
                    /explore geo tagcity [limit]   城市词典挂地理标签 (精确坐标)
                    /explore geo coverage [阈值]    区域覆盖度报告 (识别缺失)
                    /explore geo links <国家/城市> [limit]  反向关联知识节点
                    /explore geo elevation [limit] 摄取海拔 (Open-Meteo)
                    /explore geo weather [limit]   摄取气象快照 (Open-Meteo)\n                    /explore geo export [source] [limit] [path]  导出 NT-Pack 高密度格式 (默认 ~/.neotrix/geo/geo_index.ntpack)\n\
                    /explore geo import <path>  从 NT-Pack 文件导入回 geo_index (备份恢复)\n\
                    /explore geo archive <source> [path]  冷归档 source 为 NT-Pack 并从热表删除\n\
                    /explore geo cold  枚举冷层 geo_*.ntpack 归档"
                )
            }
        }
    }
}
