const {
  invoke
} = window.__TAURI__.tauri;
const {
  emit,
  listen
} = window.__TAURI__.event;
const {
  open
} = window.__TAURI__.shell;
const {
  getVersion
} = window.__TAURI__.app;

let dev;

function sleep(ms) {
  return new Promise(r => setTimeout(r, ms));
}

async function listenCommands() {
  await invoke("listen_commands", {});
}

async function startVPN() {
  let ip = await getRemoteSocksIP();
  let port = await getRemoteSocksPort();
  await invoke("start_vpn", {
    socksIp: ip,
    socksPort: port
  });
  await sleep(1000);
  reloadUI();
}

async function stopVPN() {
  await invoke("stop_vpn", {});
  await sleep(1000);
  reloadUI();
}

async function isVPNRunning() {
  return await invoke("is_vpn_running", {});
}

async function getRemoteSocksIP() {
  return await invoke("get_remote_socks_ip", {});
}

async function getRemoteSocksPort() {
  return await invoke("get_remote_socks_port", {});
}

async function getListenAddress() {
  return await invoke("get_listen_address", {});
}

async function isDebug() {
  return await invoke("is_debug", {});
}

async function listenStartVPNEvent() {
  await listen("start_vpn", (event) => {
    startVPN();
  });
}

async function listenEvents() {
  await Promise.all([listenStartVPNEvent()]);
}

function openAndroidAppLink() {
  open("https://play.google.com/store/apps/details?id=com.leaf.and.aleaf");
}

function openIosAppLink() {
  open("https://apps.apple.com/us/app/leaf-lightweight-proxy/id1534109007");
}

function openGithubLink() {
  open("https://github.com/eycorsican/leafvpn-desktop");
}

async function reloadUI() {
  if (await isVPNRunning()) {
    document.getElementById("leafvpn-status-connected").style.display = "block";
    document.getElementById("leafvpn-status-disconnected").style.display = "none";
    let msg = document.getElementById("connected-msg");
    let ip = await getRemoteSocksIP();
    let port = await getRemoteSocksPort();
    msg.innerHTML = "Connected!";
    let btn = document.getElementById("stop-btn");
    btn.onclick = async() => { await stopVPN() };
  } else {
    document.getElementById("leafvpn-status-connected").style.display = "none";
    document.getElementById("leafvpn-status-disconnected").style.display = "block";
    document.getElementById("android-link").onclick = openAndroidAppLink;
    document.getElementById("ios-link").onclick = openIosAppLink;
    document.getElementById("github-link").onclick = openGithubLink;
    getListenAddress().then((addr) => {
      let qrcode = document.getElementById("connect-qrcode");
      qrcode.innerHTML = "";
      new QRCode(qrcode, addr);
    });
  }
  getVersion().then((v) => {
    document.getElementById("foot-note").innerHTML = "v" + v;
  });
}

window.addEventListener("contextmenu", (e) => {
  if (!dev) {
    e.preventDefault();
  }
});

window.addEventListener("DOMContentLoaded", () => {
  listenEvents();
  listenCommands();
  reloadUI();
  isDebug().then((v) => {
    dev = v;
  });
});
