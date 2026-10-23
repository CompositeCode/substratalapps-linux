#!/usr/bin/env bash
# Renders the app headlessly (Xvfb) and saves PNG screenshots. Runs inside
# the dev container: scripts/dev.sh scripts/screenshot.sh [OUT_DIR] [ROUTE…]
#
# Uses the developer preview (SUBSTRATAL_PREVIEW), so no account is needed:
# layout only, nothing is loaded from the API. Set PERSONA, SCHEME (light|dark),
# SIZE (WxH) to vary it.
set -euo pipefail
out=${1:-_screens}
shift || true
routes=("$@")
[ ${#routes[@]} -eq 0 ] && routes=(/ /admin/users /settings)
size=${SIZE:-1280x800}
mkdir -p "$out"
cargo build -q -p substratal-hub
export DISPLAY=:99 GDK_BACKEND=x11 GSK_RENDERER=cairo NO_AT_BRIDGE=1 \
       XDG_DATA_DIRS="$PWD/data:/usr/local/share:/usr/share" \
       XDG_CONFIG_HOME="$(mktemp -d)" SUBSTRATAL_PREVIEW="${PERSONA-superadmin}"
Xvfb :99 -screen 0 "${size}x24" -nolisten tcp >/dev/null 2>&1 &
xvfb=$!
trap 'kill $xvfb 2>/dev/null || true' EXIT
sleep 1
if [ -n "${SCHEME:-}" ]; then
  mkdir -p "$XDG_CONFIG_HOME/com.substratalapps.Hub"
  w=${size%x*}; h=${size#*x}
  printf '{"color_scheme":"%s","width":%s,"height":%s}\n' "$SCHEME" "$w" "$h" > "$XDG_CONFIG_HOME/com.substratalapps.Hub/preferences.json"
else
  w=${size%x*}; h=${size#*x}
  mkdir -p "$XDG_CONFIG_HOME/com.substratalapps.Hub"
  printf '{"width":%s,"height":%s}\n' "$w" "$h" > "$XDG_CONFIG_HOME/com.substratalapps.Hub/preferences.json"
fi
for route in "${routes[@]}"; do
  name=$(echo "${route#/}" | tr '/' '-'); name=${name:-home}
  SUBSTRATAL_ROUTE="$route" dbus-run-session -- target/debug/substratal-hub >/tmp/hub.log 2>&1 &
  app=$!
  sleep "${WAIT:-3}"
  import -window root "$out/${SCHEME:-light}-$name.png"
  kill $app 2>/dev/null || true
  wait $app 2>/dev/null || true
  echo "screenshot: $out/${SCHEME:-light}-$name.png"
done
grep -iE 'critical|error' /tmp/hub.log | head -5 || true
