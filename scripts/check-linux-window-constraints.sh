#!/usr/bin/env bash
set -euo pipefail

minimum_width=$(jq -er '.window.minSize.width' nsh.json)
minimum_height=$(jq -er '.window.minSize.height' nsh.json)
executable=.nsh/out/Zeedle/Zeedle

minimum_hint_dimensions() {
  sed -nE 's/.*minimum size: ([0-9]+) by ([0-9]+).*/\1 \2/p' <<<"$1"
}

minimum_hint_satisfies_app_constraints() {
  local dimensions hint_width hint_height
  dimensions=$(minimum_hint_dimensions "$1")
  read -r hint_width hint_height <<<"$dimensions"
  [[ "${hint_width:-}" =~ ^[0-9]+$ && "${hint_height:-}" =~ ^[0-9]+$ ]] || return 1
  ((hint_width >= minimum_width && hint_height >= minimum_height))
}

if [[ ! -x "$executable" ]]; then
  echo "packaged executable not found: $executable" >&2
  exit 1
fi

GDK_BACKEND=x11 GTK_A11Y=none GSK_RENDERER=cairo \
  "$executable" >"target/window-constraints.log" 2>&1 &
app_pid=$!
cleanup() {
  kill "$app_pid" 2>/dev/null || true
  wait "$app_pid" 2>/dev/null || true
}
trap cleanup EXIT

window_id=
geometry=
hints=
last_window_diagnostics=
for _ in $(seq 1 150); do
  for candidate in $(xwininfo -root -tree 2>/dev/null | awk '/"Zeedle"/ { print $1 }'); do
    candidate_geometry=$(xwininfo -id "$candidate" 2>/dev/null) || continue
    candidate_hints=$(xprop -id "$candidate" WM_NORMAL_HINTS 2>/dev/null) || continue
    map_state=$(awk '/^[[:space:]]*Map State:/ { print $3; exit }' <<<"$candidate_geometry")
    width=$(awk '/^[[:space:]]*Width:/ { print $2; exit }' <<<"$candidate_geometry")
    height=$(awk '/^[[:space:]]*Height:/ { print $2; exit }' <<<"$candidate_geometry")
    last_window_diagnostics="window=$candidate map_state=$map_state client=${width}x${height}; $candidate_hints"
    if [[ "$map_state" == IsViewable ]] && minimum_hint_satisfies_app_constraints "$candidate_hints" && [[ "$width" == 900 && "$height" == 510 ]]; then
      window_id=$candidate
      geometry=$candidate_geometry
      hints=$candidate_hints
      break
    fi
  done
  [[ -n "$window_id" ]] && break
  if ! kill -0 "$app_pid" 2>/dev/null; then
    cat "target/window-constraints.log" >&2
    echo "The app exited before its native size hints were applied" >&2
    exit 1
  fi
  sleep 0.1
done

if [[ -z "$window_id" ]]; then
  echo "X11 windows after startup:" >&2
  xwininfo -root -tree >&2 || true
  echo "$last_window_diagnostics" >&2
  ps -o pid,stat,cmd -p "$app_pid" >&2 || true
  cat "target/window-constraints.log" >&2
  echo "No visible 900x510 window reported minimum size hints of at least ${minimum_width}x${minimum_height}" >&2
  exit 1
fi

printf 'App: client %sx%s; %s\n' "$width" "$height" "$hints"

[[ "$width" == 900 && "$height" == 510 ]] || {
  echo "Startup client size should be 900x510" >&2
  exit 1
}
minimum_hint=$(minimum_hint_dimensions "$hints")
read -r hinted_minimum_width hinted_minimum_height <<<"$minimum_hint"
[[ "${hinted_minimum_width:-}" =~ ^[0-9]+$ && "${hinted_minimum_height:-}" =~ ^[0-9]+$ ]] || {
  echo "X11 did not report a parseable minimum size hint" >&2
  exit 1
}
((hinted_minimum_width >= minimum_width && hinted_minimum_height >= minimum_height)) || {
  echo "X11 minimum hint ${hinted_minimum_width}x${hinted_minimum_height} is below the app minimum ${minimum_width}x${minimum_height}" >&2
  exit 1
}
