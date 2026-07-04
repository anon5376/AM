use crate::beval::results::BevalResults;
use crate::core::axes::axis_name;
use crate::core::inspect::axis_certainty;
use crate::core::state::{AmState, ContradictionStatus};
use crate::storage::snapshot_file::load_snapshot;
use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn write_dashboard(
    snapshot: impl AsRef<Path>,
    beval_paths: &[impl AsRef<Path>],
    out: impl AsRef<Path>,
) -> Result<String> {
    let state = load_snapshot(snapshot)?;
    let mut beval = Vec::new();
    for path in beval_paths {
        let path_ref = path.as_ref();
        let text =
            fs::read_to_string(path_ref).with_context(|| format!("read {}", path_ref.display()))?;
        let results: BevalResults = serde_json::from_str(&text)
            .with_context(|| format!("parse beval results {}", path_ref.display()))?;
        beval.push(DashboardBeval::from_results(
            path_ref.display().to_string(),
            &results,
        ));
    }
    let data = DashboardData {
        schema: "AM-DASHBOARD-1",
        snapshot: DashboardSnapshot::from_state(&state),
        beval,
    };
    let html = render_dashboard(&data)?;
    if let Some(parent) = out.as_ref().parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(out.as_ref(), &html).with_context(|| format!("write {}", out.as_ref().display()))?;
    Ok(html)
}

pub fn render_dashboard(data: &DashboardData) -> Result<String> {
    let payload = serde_json::to_string_pretty(data).context("serialize dashboard payload")?;
    let payload = escape_script_json(&payload);
    Ok(format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>AM Dashboard</title>
<style>
:root {{ color-scheme: light; --ink:#182026; --muted:#5a6670; --line:#d7dde2; --panel:#f8fafb; --good:#1f7a4d; --warn:#a45d12; --bad:#a12626; }}
* {{ box-sizing: border-box; }}
body {{ margin:0; font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; color:var(--ink); background:#ffffff; }}
main {{ max-width: 1180px; margin: 0 auto; padding: 24px; }}
h1 {{ font-size: 24px; margin: 0 0 16px; letter-spacing: 0; }}
h2 {{ font-size: 16px; margin: 28px 0 10px; letter-spacing: 0; }}
.summary {{ display:grid; grid-template-columns: repeat(auto-fit, minmax(150px,1fr)); gap:8px; }}
.metric {{ border:1px solid var(--line); border-radius:6px; padding:10px 12px; background:var(--panel); }}
.metric b {{ display:block; font-size:12px; color:var(--muted); font-weight:600; }}
.metric span {{ display:block; font-size:20px; margin-top:3px; }}
table {{ width:100%; border-collapse: collapse; font-size:13px; }}
th, td {{ border-bottom:1px solid var(--line); padding:7px 8px; text-align:left; vertical-align:top; }}
th {{ color:var(--muted); font-size:12px; font-weight:700; background:#f1f4f6; }}
.axis {{ display:inline-block; margin:0 5px 4px 0; padding:2px 5px; border-radius:4px; background:#eef3f6; }}
.empty {{ color:var(--muted); font-style:italic; }}
.good {{ color:var(--good); font-weight:700; }}
.warn {{ color:var(--warn); font-weight:700; }}
.bad {{ color:var(--bad); font-weight:700; }}
</style>
</head>
<body>
<main>
<h1>AM Dashboard</h1>
<section class="summary" id="summary"></section>
<h2>Active Rows</h2>
<div id="active"></div>
<h2>Strong Links</h2>
<div id="links"></div>
<h2>Open Contradictions</h2>
<div id="contradictions"></div>
<h2>B-Eval Results</h2>
<div id="beval"></div>
</main>
<script id="am-data" type="application/json">{payload}</script>
<script>
const data = JSON.parse(document.getElementById('am-data').textContent);
const esc = value => String(value).replace(/[&<>"]/g, ch => ({{'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}}[ch]));
const metric = (name, value) => `<div class="metric"><b>${{esc(name)}}</b><span>${{esc(value)}}</span></div>`;
document.getElementById('summary').innerHTML = [
  metric('theta hash', data.snapshot.theta_hash.slice(0, 12)),
  metric('tick', data.snapshot.tick),
  metric('rows', data.snapshot.row_count),
  metric('open contradictions', data.snapshot.contradictions.length),
  metric('beval reports', data.beval.length)
].join('');
function table(headers, rows, empty) {{
  if (!rows.length) return `<p class="empty">${{esc(empty)}}</p>`;
  return `<table><thead><tr>${{headers.map(h => `<th>${{esc(h)}}</th>`).join('')}}</tr></thead><tbody>${{rows.join('')}}</tbody></table>`;
}}
document.getElementById('active').innerHTML = table(['label','a','b','cert','axes'], data.snapshot.active_rows.map(row =>
  `<tr><td>${{esc(row.label)}}</td><td>${{row.activation.toFixed(2)}}</td><td>${{row.baseline.toFixed(2)}}</td><td>${{row.certainty.toFixed(2)}}</td><td>${{row.axes.map(axis => `<span class="axis">${{esc(axis.name)}} ${{axis.value >= 0 ? '+' : ''}}${{axis.value.toFixed(2)}}</span>`).join('')}}</td></tr>`
), 'no allocated rows');
document.getElementById('links').innerHTML = table(['source','weight','target'], data.snapshot.links.map(link =>
  `<tr><td>${{esc(link.source)}}</td><td>${{link.weight.toFixed(2)}}</td><td>${{esc(link.target)}}</td></tr>`
), 'no links');
document.getElementById('contradictions').innerHTML = table(['label','axis','min-axis cert'], data.snapshot.contradictions.map(item =>
  `<tr><td>${{esc(item.label)}}</td><td>${{esc(item.axis)}}</td><td class="warn">${{item.min_axis_cert.toFixed(2)}}</td></tr>`
), 'no open contradictions');
document.getElementById('beval').innerHTML = table(['lane','evaluated','skipped','mean ctx','categories'], data.beval.map(report =>
  `<tr><td>${{esc(report.lane)}}</td><td>${{report.evaluated}}</td><td>${{report.skipped}}</td><td>${{report.mean_context_tokens.toFixed(1)}}</td><td>${{report.categories.map(cat => `<span class="${{cat.accuracy === 1 ? 'good' : cat.accuracy === 0 ? 'bad' : 'warn'}}">${{esc(cat.category)}} ${{cat.matched}}/${{cat.total}}</span>`).join(' ')}}</td></tr>`
), 'no beval results supplied');
</script>
</body>
</html>
"#
    ))
}

#[derive(Clone, Debug, Serialize)]
pub struct DashboardData {
    schema: &'static str,
    snapshot: DashboardSnapshot,
    beval: Vec<DashboardBeval>,
}

#[derive(Clone, Debug, Serialize)]
struct DashboardSnapshot {
    theta_hash: String,
    tick: i64,
    row_count: usize,
    active_rows: Vec<DashboardRow>,
    links: Vec<DashboardLink>,
    contradictions: Vec<DashboardContradiction>,
    goals: Vec<String>,
}

impl DashboardSnapshot {
    fn from_state(state: &AmState) -> Self {
        Self {
            theta_hash: state.theta.hash(),
            tick: state.tick,
            row_count: state.allocated_rows_sorted().len(),
            active_rows: active_rows(state),
            links: link_rows(state),
            contradictions: contradiction_rows(state),
            goals: state
                .goals
                .iter()
                .filter_map(|goal| state.resolve_row_ref(*goal).ok())
                .map(|goal| state.concept_label(goal.id))
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct DashboardRow {
    label: String,
    activation: f32,
    baseline: f32,
    certainty: f32,
    axes: Vec<DashboardAxis>,
}

#[derive(Clone, Debug, Serialize)]
struct DashboardAxis {
    name: String,
    value: f32,
}

#[derive(Clone, Debug, Serialize)]
struct DashboardLink {
    source: String,
    target: String,
    weight: f32,
}

#[derive(Clone, Debug, Serialize)]
struct DashboardContradiction {
    label: String,
    axis: String,
    min_axis_cert: f32,
}

#[derive(Clone, Debug, Serialize)]
struct DashboardBeval {
    path: String,
    lane: String,
    evaluated: usize,
    skipped: usize,
    mean_context_tokens: f64,
    categories: Vec<DashboardCategory>,
}

impl DashboardBeval {
    fn from_results(path: String, results: &BevalResults) -> Self {
        Self {
            path,
            lane: results.metadata.lane.as_str().to_string(),
            evaluated: results.evaluated,
            skipped: results.skipped,
            mean_context_tokens: results.token_cost.mean_context_tokens,
            categories: results
                .category_scores
                .iter()
                .map(|score| DashboardCategory {
                    category: score.category.to_string(),
                    matched: score.matched,
                    total: score.total,
                    accuracy: score.accuracy,
                })
                .collect(),
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct DashboardCategory {
    category: String,
    matched: usize,
    total: usize,
    accuracy: f64,
}

fn active_rows(state: &AmState) -> Vec<DashboardRow> {
    let mut rows = state.allocated_rows_sorted();
    rows.sort_by(|left, right| {
        state.a[*right]
            .total_cmp(&state.a[*left])
            .then_with(|| left.cmp(right))
    });
    rows.into_iter()
        .take(16)
        .map(|row| {
            let mut axes = (0..state.theta.d)
                .map(|axis| (axis, state.m_get(row, axis)))
                .filter(|(_, value)| value.abs() >= state.theta.eps_log)
                .collect::<Vec<_>>();
            axes.sort_by(|left, right| {
                right
                    .1
                    .abs()
                    .total_cmp(&left.1.abs())
                    .then_with(|| left.0.cmp(&right.0))
            });
            DashboardRow {
                label: state.concept_label(row),
                activation: state.a[row],
                baseline: state.b[row],
                certainty: state.certainty(row),
                axes: axes
                    .into_iter()
                    .take(4)
                    .map(|(axis, value)| DashboardAxis {
                        name: axis_name(axis).unwrap_or("?").to_string(),
                        value,
                    })
                    .collect(),
            }
        })
        .collect()
}

fn link_rows(state: &AmState) -> Vec<DashboardLink> {
    let mut links = BTreeMap::<(usize, usize), f32>::new();
    for source in state.allocated_rows_sorted() {
        for link in &state.links[source] {
            if !state.is_allocated(link.target) {
                continue;
            }
            let pair = if source <= link.target {
                (source, link.target)
            } else {
                (link.target, source)
            };
            links
                .entry(pair)
                .and_modify(|weight| *weight = weight.max(link.weight))
                .or_insert(link.weight);
        }
    }
    let mut entries = links.into_iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .1
            .total_cmp(&left.1)
            .then_with(|| left.0 .0.cmp(&right.0 .0))
            .then_with(|| left.0 .1.cmp(&right.0 .1))
    });
    entries
        .into_iter()
        .take(12)
        .map(|((source, target), weight)| DashboardLink {
            source: state.concept_label(source),
            target: state.concept_label(target),
            weight,
        })
        .collect()
}

fn contradiction_rows(state: &AmState) -> Vec<DashboardContradiction> {
    let mut rows = state
        .open_contradictions
        .iter()
        .filter(|contradiction| contradiction.status == ContradictionStatus::Open)
        .filter_map(|contradiction| {
            let row = state.resolve_row_ref(contradiction.concept).ok()?.id;
            Some((
                row,
                contradiction.axis,
                DashboardContradiction {
                    label: state.concept_label(row),
                    axis: axis_name(contradiction.axis).unwrap_or("?").to_string(),
                    min_axis_cert: axis_certainty(state.v_get(row, contradiction.axis)),
                },
            ))
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
    rows.into_iter().map(|(_, _, row)| row).collect()
}

fn escape_script_json(value: &str) -> String {
    value
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
}
