#!/usr/bin/env bash
set -euo pipefail

# GTK includes its X11 frame allowance in WM_NORMAL_HINTS; nsh.json's
# 850x500 content minimum is reported to the window manager as 860x510.
expected_minimum_hint="minimum size: 860 by 510"
executable=.nsh/out/Zeedle/Zeedle

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
    if [[ "$map_state" == IsViewable ]] && grep -Fq "$expected_minimum_hint" <<<"$candidate_hints" && [[ "$width" == 900 && "$height" == 510 ]]; then
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
  echo "The app did not open a visible X11 window" >&2
  exit 1
fi

printf 'App: client %sx%s; %s\n' "$width" "$height" "$hints"

[[ "$width" == 900 && "$height" == 510 ]] || {
  echo "Startup client size should be 900x510" >&2
  exit 1
}
[[ "$hints" == *"$expected_minimum_hint"* ]] || {
  echo "X11 minimum hint should be 860x510 for its 850x500 content minimum" >&2
  exit 1
}
