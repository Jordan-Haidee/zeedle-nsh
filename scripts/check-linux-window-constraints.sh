#!/usr/bin/env bash
set -euo pipefail

ui=${1:?usage: check-linux-window-constraints.sh tsxui|rustui}
case "$ui" in
  tsxui) executable=.nsh/out/tsxui/Zeedle/Zeedle ;;
  rustui) executable=.nsh/out/rustui/Zeedle/Zeedle ;;
  *) echo "unknown UI variant: $ui" >&2; exit 2 ;;
esac

if [[ ! -x "$executable" ]]; then
  echo "packaged executable not found: $executable" >&2
  exit 1
fi

"$executable" >"target/${ui}-window-constraints.log" 2>&1 &
app_pid=$!
cleanup() {
  kill "$app_pid" 2>/dev/null || true
  wait "$app_pid" 2>/dev/null || true
}
trap cleanup EXIT

window_id=
for _ in $(seq 1 150); do
  window_id=$(xwininfo -root -tree 2>/dev/null | awk '/"Zeedle"/ { print $1; exit }')
  [[ -n "$window_id" ]] && break
  sleep 0.1
done

if [[ -z "$window_id" ]]; then
  cat "target/${ui}-window-constraints.log" >&2
  echo "$ui did not open a visible X11 window" >&2
  exit 1
fi

# Let the app's post-titlebar sizing request reach the GTK main loop.
sleep 0.5
geometry=$(xwininfo -id "$window_id")
width=$(awk '/^[[:space:]]*Width:/ { print $2; exit }' <<<"$geometry")
height=$(awk '/^[[:space:]]*Height:/ { print $2; exit }' <<<"$geometry")
hints=$(xprop -id "$window_id" WM_NORMAL_HINTS)
printf '%s: client %sx%s; %s\n' "$ui" "$width" "$height" "$hints"

[[ "$width" == 900 && "$height" == 510 ]] || {
  echo "$ui startup client size should be 900x510" >&2
  exit 1
}
[[ "$hints" == *"minimum size: 850 by 500"* ]] || {
  echo "$ui native minimum should be 850x500" >&2
  exit 1
}
