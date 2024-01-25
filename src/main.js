const { invoke } = window.__TAURI__.tauri;
const { emit, listen } = window.__TAURI__.event;
const { open } = window.__TAURI__.shell;
const { getVersion } = window.__TAURI__.app;

let qrcode;

async function listen_commands() {
	await invoke("listen_commands", {});
}

async function run_leaf() {
	let ip = await get_remote_socks_ip();
	let port = await get_remote_socks_port();
	await invoke("run_leaf", { socksIp: ip, socksPort: port });
}

async function stop_leaf() {
	await invoke("stop_leaf", {});
}

async function is_leaf_running() {
	return await invoke("is_leaf_running", {});
}

async function get_remote_socks_ip() {
	return await invoke("get_remote_socks_ip", {});
}

async function get_remote_socks_port() {
	return await invoke("get_remote_socks_port", {});
}

async function get_listen_address() {
	return await invoke("get_listen_address", {});
}

async function listen_events() {
	await listen("start_vpn", (event) => {
		setTimeout(() => {
			reload_ui();
		}, "1000");
		run_leaf();
	});
}

function on_click_stop() {
	setTimeout(() => {
		reload_ui();
	}, "1000");
	stop_leaf();
}

function open_dl_android() {
	open("https://play.google.com/store/apps/details?id=com.leaf.and.aleaf");
}

async function reload_ui() {
	let is_running = await is_leaf_running();
	if (is_running) {
  		document.getElementById("leafvpn-status-connected").style.display = "block";
  		document.getElementById("leafvpn-status-disconnected").style.display = "none";
  		let msg = document.getElementById("connected-msg");
		let ip = await get_remote_socks_ip();
		let port = await get_remote_socks_port();
		msg.innerHTML = "Connected!";

  		let btn = document.getElementById("stop-btn");
		btn.onclick = on_click_stop;
	} else {
  		document.getElementById("leafvpn-status-connected").style.display = "none";
  		document.getElementById("leafvpn-status-disconnected").style.display = "block";
  		document.getElementById("dl-for-android").onclick = open_dl_android;
		let qrcode = document.getElementById("connect-qrcode");
		qrcode.innerHTML = "";
		get_listen_address().then((addr) => {
  			qrcode = new QRCode(qrcode, addr);
		});
	}
	getVersion().then((v) => {
		document.getElementById("foot-note").innerHTML =  "v" + v;
	});
}

window.addEventListener("DOMContentLoaded", () => {
	listen_events();
	listen_commands();
	reload_ui();
});
