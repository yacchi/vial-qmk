const { invoke } = window.__TAURI__.core;

// State
let connected = false;
let deviceStatus = null;

// DOM elements
const connectionStatus = document.getElementById("connection-status");
const layerCount = document.getElementById("layer-count");
const defaultLayer = document.getElementById("default-layer");
const activeLayers = document.getElementById("active-layers");
const osOverride = document.getElementById("os-override");
const layerButtons = document.getElementById("layer-buttons");
const currentApp = document.getElementById("current-app");

// Update connection status indicator
function updateConnectionUI(isConnected) {
  connected = isConnected;
  connectionStatus.textContent = isConnected ? "Connected" : "Disconnected";
  connectionStatus.className = `status ${isConnected ? "connected" : "disconnected"}`;
}

// Update device status display
function updateStatusUI(status) {
  deviceStatus = status;
  layerCount.textContent = status.layer_count;
  defaultLayer.textContent = status.default_layer;

  const activeList = [];
  for (let i = 0; i < 32; i++) {
    if (status.layer_state & (1 << i)) activeList.push(i);
  }
  activeLayers.textContent = activeList.join(", ") || "none";

  const overrideNames = ["Disabled", "US on JP OS", "JP on US OS"];
  osOverride.textContent = overrideNames[status.os_override] || "Unknown";

  // Update override radio buttons
  document.querySelectorAll('input[name="os-override"]').forEach((radio) => {
    radio.checked = parseInt(radio.value) === status.os_override;
  });

  // Generate layer buttons
  layerButtons.innerHTML = "";
  for (let i = 0; i < status.layer_count; i++) {
    const btn = document.createElement("button");
    btn.textContent = `Layer ${i}`;
    btn.className = i === status.default_layer ? "active" : "";
    btn.addEventListener("click", () => setLayer(i));
    layerButtons.appendChild(btn);
  }
}

// Commands
async function connectDevice() {
  try {
    const result = await invoke("connect_device");
    updateConnectionUI(true);
    await refreshStatus();
  } catch (e) {
    updateConnectionUI(false);
    console.error("Connection failed:", e);
  }
}

async function refreshStatus() {
  try {
    const status = await invoke("get_device_status");
    updateStatusUI(status);
  } catch (e) {
    console.error("Failed to get status:", e);
  }
}

async function setLayer(layer) {
  try {
    await invoke("set_layer", { layer });
    await refreshStatus();
  } catch (e) {
    console.error("Failed to set layer:", e);
  }
}

async function setOsOverride(overrideType) {
  try {
    await invoke("set_os_override", { overrideType });
    await refreshStatus();
  } catch (e) {
    console.error("Failed to set OS override:", e);
  }
}

async function openVial() {
  try {
    await invoke("open_vial");
  } catch (e) {
    console.error("Failed to open Vial:", e);
  }
}

async function pollActiveApp() {
  try {
    const appName = await invoke("get_active_app");
    currentApp.textContent = appName || "-";
  } catch (_) {}
}

// Event listeners
document.getElementById("btn-connect").addEventListener("click", connectDevice);
document.getElementById("btn-refresh").addEventListener("click", refreshStatus);
document.getElementById("btn-open-vial").addEventListener("click", openVial);

document.querySelectorAll('input[name="os-override"]').forEach((radio) => {
  radio.addEventListener("change", (e) => {
    setOsOverride(parseInt(e.target.value));
  });
});

// Auto-switch polling
const autoSwitchCheckbox = document.getElementById("auto-switch-enabled");
let pollInterval = null;

autoSwitchCheckbox.addEventListener("change", (e) => {
  if (e.target.checked) {
    pollInterval = setInterval(pollActiveApp, 1000);
  } else {
    clearInterval(pollInterval);
    pollInterval = null;
  }
});

// Initial connection attempt
connectDevice();
