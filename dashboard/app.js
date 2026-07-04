const DATA = {
  build: "S06",
  currentStage: "S06",
  roadmap: "Current checked-in state: W0 mechanics complete. Next roadmap stage: M2 perception bridge.",
  stages: [
    {
      id: "S00",
      status: "complete",
      title: "Clean Rust Core",
      report: "docs/BUILD_REPORT.md",
      summary: "Initial AM001 parameter-memory attractor core.",
      tests: "base kill tests green",
      changed: [
        "Implemented M/W/a/b/V/u stores.",
        "Added deterministic snapshots, trace hashing, CLI, parser, and dormant LLM seam.",
        "Established no database memory, no raw chat memory, no LLM in runtime loop."
      ],
      files: ["src/core/*", "src/cli/*", "src/parser/*", "src/storage/*", "docs/OLLAMA_SEAM.md"]
    },
    {
      id: "S02",
      status: "complete",
      title: "Theta, Certainty, Snapshot v2",
      report: "docs/BUILD_REPORT_S02.md",
      summary: "Unified theta sweep surface and row reference hardening started.",
      tests: "sweep smoke, stale rowref, snapshot refusal, dump/axes green",
      changed: [
        "Moved kill-test criteria into reusable eval functions.",
        "Added am sweep, axes display, min-axis certainty, and bench-step.",
        "Added snapshot format_version and RowRef generation checks."
      ],
      files: ["src/eval/criteria.rs", "src/eval/sweep.rs", "src/core/state.rs", "tests/stale_rowref.rs"]
    },
    {
      id: "S03",
      status: "complete",
      title: "Allocation and W-Link Hardening",
      report: "docs/BUILD_REPORT_S03.md",
      summary: "Allocation state became explicit and stale W edges were pinned by tests.",
      tests: "16 integration tests green",
      changed: [
        "Added allocated Vec<bool> and snapshot v3.",
        "Hardened free/reuse/merge paths and W eps_log behavior.",
        "Expanded drift and diff-integrity tests."
      ],
      files: ["src/core/decay.rs", "src/core/hebb.rs", "tests/stale_w_links.rs", "tests/diff_integrity.rs"]
    },
    {
      id: "S04",
      status: "complete",
      title: "W0 World Harness",
      report: "docs/BUILD_REPORT_S04.md",
      summary: "Deterministic grid-world JSONL harness added.",
      tests: "24 integration tests green",
      changed: [
        "Added src/world modules, closed Action enum, script parser, observation JSONL.",
        "Added am world-run and world golden traces.",
        "Kept W0 at the JSONL boundary with no core wiring."
      ],
      files: ["src/world/*", "docs/W0_WORLD.md", "docs/OBSERVATION_SCHEMA.md", "tests/world_*.rs"]
    },
    {
      id: "S05",
      status: "complete",
      title: "A15 Attractor Fix",
      report: "docs/BUILD_REPORT_S05.md",
      summary: "Structural A15 cue input made the default theta pass all core criteria at t=20.",
      tests: "25 integration tests green",
      changed: [
        "Added Theta.k_i and A15c resting-field validation.",
        "Committed default theta hash dce727473be3778453afda3a214d6220ff8bca191a8ec731d151e9b301af3952.",
        "Deleted tuned completion override after default passed."
      ],
      files: ["src/core/resolve.rs", "src/core/settle.rs", "src/core/theta.rs", "sweep/grid.json", "tests/resting_field.rs"]
    },
    {
      id: "S06",
      status: "current",
      title: "W0 Mechanics Complete",
      report: "docs/BUILD_REPORT_S06.md",
      summary: "W0 now has causal mechanics for future schema learning; C1-C3 audit riders landed.",
      tests: "37 tests green; clippy clean",
      changed: [
        "Replaced rule pings with hidden behavior classes, shape-class regularity, matching tables, holding, open, removal, consumable, hazard, step cost, and termination.",
        "Added neutral held_shape_id, render-only ASCII frames, and reserved tier flag errors.",
        "Added theta validation on snapshot load and AmState::new, A16, link-decay aliveness, and recall_margin_095."
      ],
      files: [
        "src/world/classes.rs",
        "src/world/grid.rs",
        "src/world/theta.rs",
        "src/cli/render.rs",
        "tests/link_decay_alive.rs",
        "tests/world_tier_flags.rs",
        "docs/BUILD_REPORT_S06.md"
      ]
    }
  ]
};

let selectedStageId = DATA.currentStage;

const byId = (id) => document.getElementById(id);

function init() {
  byId("build-id").textContent = DATA.build;
  byId("current-stage").textContent = DATA.currentStage;
  byId("figure-stage").textContent = DATA.currentStage;
  byId("roadmap-line").textContent = DATA.roadmap;
  renderStageNav();
  renderStageTable();
  renderSelectedStage();
  bindEvents();
}

function bindEvents() {
  byId("refresh-view").addEventListener("click", () => {
    renderStageNav();
    renderStageTable();
    renderSelectedStage();
  });
  byId("export-status").addEventListener("click", exportStatus);
}

function renderStageNav() {
  const nav = byId("stage-nav");
  nav.innerHTML = "";
  for (const stage of DATA.stages) {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = `${stage.id} ${stage.title}`;
    button.setAttribute("aria-current", stage.id === selectedStageId ? "true" : "false");
    button.addEventListener("click", () => {
      selectedStageId = stage.id;
      renderStageNav();
      renderSelectedStage();
    });
    nav.appendChild(button);
  }
}

function renderStageTable() {
  const body = byId("stage-table");
  body.innerHTML = "";
  for (const stage of DATA.stages) {
    const row = document.createElement("tr");
    row.append(
      cell(stage.id),
      cell(stage.status),
      cell(stage.summary),
      cell(stage.tests),
      cell(stage.report)
    );
    body.appendChild(row);
  }
}

function renderSelectedStage() {
  const stage = DATA.stages.find((item) => item.id === selectedStageId) || DATA.stages.at(-1);
  const detail = byId("stage-detail");
  detail.innerHTML = "";
  addPair(detail, "stage", stage.id);
  addPair(detail, "status", stage.status);
  addPair(detail, "title", stage.title);
  addPair(detail, "summary", stage.summary);
  addPair(detail, "report", stage.report);

  const verification = byId("verification-table");
  verification.innerHTML = "";
  const checks = [
    ["tests", stage.tests],
    ["roadmap position", stage.id === DATA.currentStage ? "current implementation stage" : "historical stage"],
    ["runtime loop", "no LLM call path in AM core"],
    ["review source", stage.report]
  ];
  for (const [command, result] of checks) {
    const row = document.createElement("tr");
    row.append(cell(command), cell(result));
    verification.appendChild(row);
  }

  renderList("change-list", stage.changed);
  renderList("file-list", stage.files);
}

function addPair(parent, key, value) {
  const term = document.createElement("dt");
  const desc = document.createElement("dd");
  term.textContent = key;
  desc.textContent = value;
  parent.append(term, desc);
}

function cell(text) {
  const td = document.createElement("td");
  td.textContent = text;
  return td;
}

function renderList(id, items) {
  const list = byId(id);
  list.innerHTML = "";
  for (const item of items) {
    const li = document.createElement("li");
    li.textContent = item;
    list.appendChild(li);
  }
}

function exportStatus() {
  const blob = new Blob([JSON.stringify(DATA, null, 2)], { type: "application/json" });
  const link = document.createElement("a");
  link.href = URL.createObjectURL(blob);
  link.download = "am001-development-status.json";
  link.click();
  URL.revokeObjectURL(link.href);
}

init();
