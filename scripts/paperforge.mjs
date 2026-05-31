#!/usr/bin/env node
import fs from "node:fs";
import path from "node:path";
import readline from "node:readline/promises";
import { stdin as input, stdout as output } from "node:process";
import crypto from "node:crypto";

const VERSION = "1.0.1";
const DEFAULT_WORKSPACE = "workspace";
const PROVIDERS = new Set(["openai-compatible", "openai", "anthropic"]);

const args = process.argv.slice(2);

function nowIso() {
  return new Date().toISOString();
}

function parseOptions(argv) {
  const values = [];
  const options = {};
  for (let index = 0; index < argv.length; index += 1) {
    const item = argv[index];
    if (item.startsWith("--")) {
      const key = item.slice(2);
      const next = argv[index + 1];
      if (next && !next.startsWith("--")) {
        options[key] = next;
        index += 1;
      } else {
        options[key] = true;
      }
    } else {
      values.push(item);
    }
  }
  return { values, options };
}

function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

function readJson(file, fallback) {
  if (!fs.existsSync(file)) return fallback;
  return JSON.parse(fs.readFileSync(file, "utf8"));
}

function writeJson(file, value) {
  ensureDir(path.dirname(file));
  fs.writeFileSync(file, `${JSON.stringify(value, null, 2)}\n`);
}

function workspaceFiles(workspaceRoot) {
  const meta = path.join(workspaceRoot, ".paperforge");
  return {
    root: workspaceRoot,
    meta,
    workspace: path.join(meta, "workspace.json"),
    models: path.join(meta, "ai-models.json"),
    settings: path.join(meta, "settings.json"),
    history: path.join(meta, "history.log"),
    papers: path.join(workspaceRoot, "papers")
  };
}

function initWorkspace(workspaceRoot = DEFAULT_WORKSPACE) {
  const root = path.resolve(workspaceRoot);
  const files = workspaceFiles(root);
  ensureDir(files.meta);
  ensureDir(files.papers);
  const existing = readJson(files.workspace, null);
  const timestamp = nowIso();
  writeJson(files.workspace, {
    version: VERSION,
    workspaceName: existing?.workspaceName || "workspace",
    createdAt: existing?.createdAt || timestamp,
    updatedAt: timestamp,
    papersDir: "papers",
    defaultLanguage: existing?.defaultLanguage || "en"
  });
  if (!fs.existsSync(files.models)) {
    writeJson(files.models, { defaultModelId: "", models: [] });
  }
  if (!fs.existsSync(files.settings)) {
    writeJson(files.settings, { language: "en", theme: "light" });
  }
  if (!fs.existsSync(files.history)) {
    fs.writeFileSync(files.history, "");
  }
  fs.appendFileSync(files.history, `${timestamp} workspace.init ${root}\n`);
  console.log(`Workspace ready: ${root}`);
  return root;
}

function findWorkspace(options) {
  if (options.workspace) return path.resolve(options.workspace);
  if (process.env.PAPERFORGE_WORKSPACE) return path.resolve(process.env.PAPERFORGE_WORKSPACE);
  const cwd = process.cwd();
  if (fs.existsSync(path.join(cwd, ".paperforge", "workspace.json"))) return cwd;
  const defaultRoot = path.join(cwd, DEFAULT_WORKSPACE);
  if (fs.existsSync(path.join(defaultRoot, ".paperforge", "workspace.json"))) return defaultRoot;
  return initWorkspace(defaultRoot);
}

async function askMissing(options, key, label, fallback = "") {
  if (options[key] !== undefined) return String(options[key]);
  const rl = readline.createInterface({ input, output });
  const suffix = fallback ? ` (${fallback})` : "";
  const answer = await rl.question(`${label}${suffix}: `);
  rl.close();
  return answer.trim() || fallback;
}

function loadModels(workspaceRoot) {
  const files = workspaceFiles(workspaceRoot);
  return readJson(files.models, { defaultModelId: "", models: [] });
}

function saveModels(workspaceRoot, config) {
  writeJson(workspaceFiles(workspaceRoot).models, config);
}

async function addModel(workspaceRoot, options) {
  const provider = await askMissing(options, "provider", "provider", "openai-compatible");
  if (!PROVIDERS.has(provider)) {
    throw new Error(`Unsupported provider: ${provider}`);
  }
  const name = await askMissing(options, "name", "name", provider);
  const baseUrl = await askMissing(options, "base-url", "baseUrl", provider === "openai" ? "https://api.openai.com/v1" : "");
  const apiKey = await askMissing(options, "api-key", "apiKey");
  const model = await askMissing(options, "model", "model");
  const temperature = Number(await askMissing(options, "temperature", "temperature", "0.2"));
  const maxTokens = Number(await askMissing(options, "max-tokens", "maxTokens", "4000"));
  const id = options.id || `${provider}_${crypto.randomUUID()}`;
  const config = loadModels(workspaceRoot);
  config.models = config.models.filter((item) => item.id !== id);
  config.models.push({ id, provider, name, baseUrl, apiKey, model, temperature, maxTokens });
  if (!config.defaultModelId) config.defaultModelId = id;
  saveModels(workspaceRoot, config);
  fs.appendFileSync(workspaceFiles(workspaceRoot).history, `${nowIso()} model.add ${id}\n`);
  console.log(`Model added: ${id}`);
}

function listModels(workspaceRoot) {
  const config = loadModels(workspaceRoot);
  if (config.models.length === 0) {
    console.log("No AI models configured.");
    return;
  }
  for (const model of config.models) {
    const marker = model.id === config.defaultModelId ? "*" : " ";
    console.log(`${marker} ${model.id} | ${model.provider} | ${model.name} | ${model.model} | apiKey=${model.apiKey ? "***" : ""}`);
  }
}

async function setDefaultModel(workspaceRoot, options, values) {
  const config = loadModels(workspaceRoot);
  const id = values[0] || options.id || await askMissing(options, "model-id", "model id");
  if (!config.models.some((item) => item.id === id || item.name === id)) {
    throw new Error(`Model not found: ${id}`);
  }
  const model = config.models.find((item) => item.id === id || item.name === id);
  config.defaultModelId = model.id;
  saveModels(workspaceRoot, config);
  fs.appendFileSync(workspaceFiles(workspaceRoot).history, `${nowIso()} model.default ${model.id}\n`);
  console.log(`Default model: ${model.id}`);
}

async function removeModel(workspaceRoot, options, values) {
  const config = loadModels(workspaceRoot);
  const id = values[0] || options.id || await askMissing(options, "model-id", "model id");
  const before = config.models.length;
  config.models = config.models.filter((item) => item.id !== id && item.name !== id);
  if (config.models.length === before) throw new Error(`Model not found: ${id}`);
  if (!config.models.some((item) => item.id === config.defaultModelId)) {
    config.defaultModelId = config.models[0]?.id || "";
  }
  saveModels(workspaceRoot, config);
  fs.appendFileSync(workspaceFiles(workspaceRoot).history, `${nowIso()} model.remove ${id}\n`);
  console.log(`Model removed: ${id}`);
}

function usage() {
  console.log(`PaperForge ${VERSION}

Commands:
  paperforge init [workspacePath]
  paperforge model add [--workspace path] [--provider openai-compatible|openai|anthropic] [--name name] [--base-url url] [--api-key key] [--model model]
  paperforge model list [--workspace path]
  paperforge model set-default <id-or-name> [--workspace path]
  paperforge model remove <id-or-name> [--workspace path]`);
}

async function main() {
  const [command, subcommand, ...rest] = args;
  const parsed = parseOptions(rest);
  if (!command || command === "--help" || command === "-h") {
    usage();
    return;
  }
  if (command === "init") {
    const { values } = parseOptions([subcommand, ...rest].filter(Boolean));
    initWorkspace(values[0] || DEFAULT_WORKSPACE);
    return;
  }
  if (command === "model") {
    const workspaceRoot = findWorkspace(parsed.options);
    if (subcommand === "add") return addModel(workspaceRoot, parsed.options);
    if (subcommand === "list") return listModels(workspaceRoot);
    if (subcommand === "set-default") return setDefaultModel(workspaceRoot, parsed.options, parsed.values);
    if (subcommand === "remove") return removeModel(workspaceRoot, parsed.options, parsed.values);
  }
  usage();
}

main().catch((error) => {
  console.error(error.message);
  process.exitCode = 1;
});
