#!/bin/env bash

state=$1
http_port=$2
socks_port=$3

# gsettings set org.gnome.system.proxy.http host 127.0.0.1
function set_GNOME_proxy() {
    local type=$1
    local port=$2

    gsettings set org.gnome.system.proxy.$type host 127.0.0.1
    gsettings set org.gnome.system.proxy.$type port $port
}

function enable_GNOME_PROXY() {
    gsettings set org.gnome.system.proxy.http enabled true
    gsettings set rg.gnome.system.proxy mode manual
}

function disable_GNOME_proxy() {
    gsettings set org.gnome.system.proxy mode none
}

# kwriteconfig5 --file kioslaverc --group "Proxy Settings" --key httpProxy "http://127.0.0.1:1080"
function set_KDE_proxy() {
    local type=$1
    local port=$2

    local url="${type}://127.0.0.1:$port"
    kwriteconfig5 --file kioslaverc --group "Proxy Settings" --key ${type}Proxy "$url"
}

function enable_KED_proxy() {
    kwriteconfig5 --file kioslaverc --group "Proxy Settings" --key ProxyType 1
    kwriteconfig5 --file kioslaverc --group "Proxy Settings" --key Authmode 0
    dbus-send --type=signal /KIO/Scheduler org.kde.KIO.Scheduler.reparseSlaveConfiguration string:''
}

function disable_KDE_proxy() {
    kwriteconfig5 --file kioslaverc --group "Proxy Settings" --key "ProxyType" 0
    dbus-send --type=signal /KIO/Scheduler org.kde.KIO.Scheduler.reparseSlaveConfiguration string:''
}

[[ -n $(type -p kwriteconfig5) ]] && desktop=KDE # KDE
[[ -n $(type -p gsettings) ]] && desktop=GNOME   # GNOME

[[ -z "$desktop" ]] && echo "Unsupported desktop" && exit 1

if [ "$state" == "on" ]; then
    if [ $http_port -gt 0 ]; then
        set_${desktop}_proxy http $http_port
    fi
    if [ $socks_port -gt 0 ]; then
        set_${desktop}_proxy socks $socks_port
    fi

    disable_${desktop}_proxy
fi

if [ "$state" == "off" ]; then
    disable_${desktop}_proxy
fi
