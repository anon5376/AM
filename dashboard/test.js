const STORAGE = {
  provider: "am001.dashboard.modelProvider",
  host: "am001.dashboard.modelHost",
  key: "am001.dashboard.modelApiKey",
  model: "am001.dashboard.modelName"
};

const PROVIDERS = {
  "ollama-cloud": {
    endpoint: "https://ollama.com/api"
  }
};

const byId = (id) => document.getElementById(id);

function init() {
  loadModelConfig();
  bindEvents();
  loadModels();
}

function bindEvents() {
  byId("model-provider").addEventListener("change", onProviderChange);
  byId("save-model-config").addEventListener("click", saveModelConfig);
  byId("load-models").addEventListener("click", loadModels);
  byId("test-model").addEventListener("click", testConnection);
  byId("run-model").addEventListener("click", runModelEventTest);
  byId("clear-output").addEventListener("click", () => {
    byId("model-output").textContent = "";
    byId("event-check").textContent = "";
    setState("not tested", "");
  });
  byId("available-models").addEventListener("change", (event) => {
    if (event.target.value) {
      byId("selected-model").value = event.target.value;
      saveModelConfig();
    }
  });
}

function loadModelConfig() {
  const storedProvider = localStorage.getItem(STORAGE.provider) || "ollama-cloud";
  const provider = PROVIDERS[storedProvider] ? storedProvider : "ollama-cloud";
  const defaultHost = providerDefaults(provider).endpoint;
  const storedHost = localStorage.getItem(STORAGE.host) || "";
  byId("model-provider").value = provider;
  byId("model-host").value = isOllamaHost(storedHost)
    ? normalizeOllamaApiBase(storedHost)
    : defaultHost;
  byId("model-key").value = localStorage.getItem(STORAGE.key) || "";
  byId("selected-model").value = localStorage.getItem(STORAGE.model) || "";
  resetModelSelect("load available models");
}

function onProviderChange() {
  const provider = byId("model-provider").value;
  byId("model-host").value = providerDefaults(provider).endpoint;
  resetModelSelect("load available models");
  saveModelConfig();
}

function saveModelConfig() {
  const config = getModelConfig();
  localStorage.setItem(STORAGE.provider, config.provider);
  localStorage.setItem(STORAGE.host, config.host);
  localStorage.setItem(STORAGE.key, config.key);
  localStorage.setItem(STORAGE.model, config.model);
  setState("config saved", "good");
}

function getModelConfig() {
  return {
    provider: byId("model-provider").value,
    host: normalizeOllamaApiBase(byId("model-host").value.trim()),
    key: byId("model-key").value.trim(),
    model: byId("selected-model").value.trim()
  };
}

function providerDefaults(provider) {
  return PROVIDERS[provider] || PROVIDERS["ollama-cloud"];
}

function trimSlash(value) {
  return value.endsWith("/") ? value.slice(0, -1) : value;
}

function normalizeOllamaApiBase(value) {
  const trimmed = trimSlash(value || "https://ollama.com/api");
  return trimmed.endsWith("/api") ? trimmed : `${trimmed}/api`;
}

function isOllamaHost(value) {
  return normalizeOllamaApiBase(value) === "https://ollama.com/api";
}

function authHeaders(key) {
  const headers = { "Content-Type": "application/json" };
  if (key) {
    headers.Authorization = `Bearer ${key}`;
  }
  return headers;
}

function resetModelSelect(label) {
  const select = byId("available-models");
  select.innerHTML = "";
  select.appendChild(new Option(label, ""));
}

async function loadModels() {
  const config = getModelConfig();
  setState("loading models", "");
  try {
    const models = await loadOllamaModels(config);
    fillModels(models, config.model);
    setState(`models available: ${models.length}`, models.length ? "good" : "");
    byId("event-check").textContent = models.length
      ? `available models\n${models.join("\n")}`
      : "connected, but no models were returned";
  } catch (err) {
    resetModelSelect("model load failed");
    setState("model load failed", "bad");
    byId("event-check").textContent = formatError(err, config);
  }
}

async function loadOllamaModels(config) {
  const response = await fetch("/dashboard-api/ollama/tags", {
    method: "GET",
    headers: proxyHeaders(config)
  });
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  const data = await response.json();
  return Array.isArray(data.models)
    ? data.models.map((model) => model.model || model.name).filter(Boolean).sort()
    : [];
}

function fillModels(models, saved) {
  const select = byId("available-models");
  select.innerHTML = "";
  select.appendChild(new Option(models.length ? "select from available models" : "no models returned", ""));
  for (const model of models) {
    select.appendChild(new Option(model, model));
  }
  if (saved && models.includes(saved)) {
    select.value = saved;
  } else if (models[0]) {
    select.value = models[0];
    byId("selected-model").value = models[0];
    saveModelConfig();
  }
}

async function testConnection() {
  const config = getModelConfig();
  setState("testing", "");
  try {
    const models = await loadOllamaModels(config);
    setState("connected", "good");
    byId("event-check").textContent =
      `connected\nprovider=${config.provider}\nmodels=${models.length}\nendpoint=${config.host}`;
  } catch (err) {
    setState("connection failed", "bad");
    byId("event-check").textContent = formatError(err, config);
  }
}

async function runModelEventTest() {
  const config = getModelConfig();
  saveModelConfig();
  if (!config.model) {
    setState("model required", "bad");
    byId("event-check").textContent = "Select an available Ollama Cloud model or type one in Selected model.";
    return;
  }

  const input = byId("am-input").value.trim();
  setState("running", "");
  byId("model-output").textContent = "";
  byId("event-check").textContent = "";

  try {
    const content = await runOllamaChat(config, input);
    byId("model-output").textContent = content;
    byId("event-check").textContent = checkEventJson(content);
    setState("response received", "good");
  } catch (err) {
    setState("run failed", "bad");
    byId("event-check").textContent = formatError(err, config);
  }
}

function parserMessages(input) {
  return [
    {
      role: "system",
      content: [
        "Return only JSON. No markdown. No prose.",
        "JSON shape:",
        "{\"id\":1,\"cues\":[],\"asserts\":[],\"links\":[],\"goal_ops\":[]}.",
        "For an assertion, use {\"concept\":{\"Label\":\"rust\"},\"targets\":{\"truth_assert\":1},\"weight\":1}.",
        "This is an edge-format test only. Do not say it entered AM core."
      ].join(" ")
    },
    { role: "user", content: input }
  ];
}

async function runOllamaChat(config, input) {
  const response = await fetch("/dashboard-api/ollama/chat", {
    method: "POST",
    headers: proxyHeaders(config),
    body: JSON.stringify({
      model: config.model,
      messages: parserMessages(input),
      stream: false,
      options: {
        temperature: 0
      }
    })
  });
  if (!response.ok) {
    throw new Error(`HTTP ${response.status}`);
  }
  const data = await response.json();
  return data.message && typeof data.message.content === "string"
    ? data.message.content
    : JSON.stringify(data, null, 2);
}

function proxyHeaders(config) {
  const headers = authHeaders(config.key);
  headers["X-Ollama-Base"] = config.host;
  return headers;
}

function checkEventJson(text) {
  const jsonText = extractJson(text);
  if (!jsonText) {
    return "invalid: no JSON object found";
  }
  try {
    const event = JSON.parse(jsonText);
    const issues = [];
    if (!Number.isInteger(event.id)) {
      issues.push("id must be an integer");
    }
    for (const field of ["cues", "asserts", "links", "goal_ops"]) {
      if (!Array.isArray(event[field])) {
        issues.push(`${field} must be an array`);
      }
    }
    if (Array.isArray(event.asserts)) {
      for (const item of event.asserts) {
        if (!item.concept) {
          issues.push("assert missing concept");
        }
        if (!item.targets || typeof item.targets !== "object" || Array.isArray(item.targets)) {
          issues.push("assert targets must be an object");
        }
      }
    }
    if (issues.length) {
      return `shape check: failed\n${issues.join("\n")}\n\nparsed:\n${JSON.stringify(event, null, 2)}`;
    }
    return `shape check: passed\nnot submitted to AM core\n\nparsed:\n${JSON.stringify(event, null, 2)}`;
  } catch (err) {
    return `invalid JSON\n${formatError(err, getModelConfig())}`;
  }
}

function extractJson(text) {
  const start = text.indexOf("{");
  const end = text.lastIndexOf("}");
  if (start === -1 || end === -1 || end <= start) {
    return "";
  }
  return text.slice(start, end + 1);
}

function setState(text, tone) {
  const node = byId("model-state");
  node.textContent = text;
  node.className = `state ${tone || ""}`.trim();
}

function formatError(err, config) {
  const lines = [
    String(err && err.message ? err.message : err),
    "",
    `provider=${config.provider}`,
    `endpoint=${config.host}`
  ];
  lines.push("", "Ollama native API calls use:");
  lines.push("browser -> GET /dashboard-api/ollama/tags -> Ollama GET /api/tags");
  lines.push("browser -> POST /dashboard-api/ollama/chat -> Ollama POST /api/chat");
  lines.push("", "For Ollama Cloud, use https://ollama.com/api and an Authorization Bearer API key.");
  lines.push("The dashboard proxy is same-origin so browser CORS does not block model loading.");
  return lines.join("\n");
}

init();
