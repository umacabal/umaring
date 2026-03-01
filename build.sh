#!/usr/bin/env bash
set -euo pipefail

MEMBERS=$(cat members.json)
WEEK=$(( $(date +%s) / (7 * 86400) ))

shuffle() {
  echo "$MEMBERS" | jq -c '.[]' | while read -r m; do
    id=$(echo "$m" | jq -r '.id')
    key=$(echo -n "$WEEK-$id" | sha256sum | awk '{print $1}')
    echo "$key $m"
  done | sort | cut -d' ' -f2- | jq -s '.'
}

redirect_html() {
  cat <<REOF
<!DOCTYPE html>
<html>
<head><meta http-equiv="refresh" content="0; url=$1"></head>
<body>Redirecting to <a href="$1">$2</a></body>
</html>
REOF
}

check_member() {
  local url="$1"
  local html
  html=$(curl -sfL --connect-timeout 2 --max-time 5 "$url" 2>/dev/null) || { echo "offline"; return; }

  local scripts
  scripts=$(echo "$html" | grep -oiE 'src=['"'"'"]?[^'"'"'" >]+' | sed "s/^src=['\"]\\?//")

  # check for our ring.js embed
  for src in $scripts; do
    echo "$src" | grep -qiE 'umaring\.(mkr\.cx|github\.io)/ring\.js' && { echo "ring.js"; return; }
  done

  # check linked JS files for umaring references
  for src in $scripts; do
    case "$src" in
      http*) ;;
      //*) src="https:$src" ;;
      /*) src="${url%/}$src" ;;
      *) src="${url%/}/$src" ;;
    esac
    curl -sfL --connect-timeout 2 --max-time 5 "$src" 2>/dev/null | grep -qi "umaring" && { echo "js"; return; }
  done

  echo "$html" | grep -qi "umaring" && { echo "html"; return; }
  echo "missing"
}


check() {
  local shuffled=$(shuffle)
  local len=$(echo "$shuffled" | jq 'length')
  local tmpdir=$(mktemp -d)
  for i in $(seq 0 $((len - 1))); do
    local url=$(echo "$shuffled" | jq -r ".[$i].url")
    ( check_member "$url" > "$tmpdir/$i" ) &
  done
  wait
  for i in $(seq 0 $((len - 1))); do
    local m=$(echo "$shuffled" | jq -c ".[$i]")
    local url=$(echo "$m" | jq -r '.url')
    local id=$(echo "$m" | jq -r '.id')
    printf "%8s: %s (%s)\n" "$(cat "$tmpdir/$i")" "$id" "$url"
  done
  rm -rf "$tmpdir"
}

status_page() {
  local statuses="$1"
  local date=$(date -u '+%Y-%m-%d %H:%M UTC')
  cat <<'HEADER'
<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>UMass Ring Status</title>
<style>
body { font-family: monospace; max-width: 640px; margin: 40px auto; padding: 0 20px; background: #111; color: #ccc; }
h1 { font-size: 1.2em; color: #fff; }
table { border-collapse: collapse; width: 100%; }
td { padding: 4px 8px; }
a { color: #8bf; }
.ring\.js, .js, .html { color: #6e6; }
.missing { color: #fc6; }
.offline { color: #f66; }
.time { color: #666; font-size: 0.85em; margin-top: 20px; }
</style>
</head>
<body>
<h1>UMass Ring Status</h1>
<table>
HEADER
  echo "$statuses" | jq -r '.[] | "<tr><td class=\"\(.status)\"><b>\(.status)</b></td><td><a href=\"\(.url)\">\(.name)</a></td></tr>"'
  cat <<FOOTER
</table>
<p class="time">Last checked: $date</p>
</body>
</html>
FOOTER
}

write_member_files() {
  local OUT="$1" id="$2" member="$3" prev="$4" next="$5"

  local prev_url=$(echo "$prev" | jq -r '.url')
  local prev_name=$(echo "$prev" | jq -r '.name')
  local next_url=$(echo "$next" | jq -r '.url')
  local next_name=$(echo "$next" | jq -r '.name')

  local data=$(jq -nc --argjson prev "$prev" --argjson member "$member" --argjson next "$next" \
    '{prev: $prev, member: $member, next: $next}')

  mkdir -p "$OUT/$id/prev" "$OUT/$id/next"
  echo "$data" > "$OUT/$id/index.html"
  echo "$data" > "$OUT/$id.json"

  redirect_html "$prev_url" "$prev_name" > "$OUT/$id/prev/index.html"
  redirect_html "$next_url" "$next_name" > "$OUT/$id/next/index.html"
}

build() {
  local OUT="out"
  rm -rf "$OUT"
  mkdir -p "$OUT"

  local shuffled=$(shuffle)
  local total=$(echo "$shuffled" | jq 'length')

  # check liveness in parallel
  echo "Checking liveness..." >&2
  local tmpdir=$(mktemp -d)
  for i in $(seq 0 $((total - 1))); do
    local url=$(echo "$shuffled" | jq -r ".[$i].url")
    ( check_member "$url" > "$tmpdir/$i" ) &
  done
  wait

  local alive="[]"
  local statuses="[]"
  for i in $(seq 0 $((total - 1))); do
    local m=$(echo "$shuffled" | jq -c ".[$i]")
    local url=$(echo "$m" | jq -r '.url')
    local id=$(echo "$m" | jq -r '.id')
    local name=$(echo "$m" | jq -r '.name')
    local status=$(cat "$tmpdir/$i")
    statuses=$(echo "$statuses" | jq -c --arg s "$status" --arg name "$name" --arg url "$url" \
      '. + [{status: $s, name: $name, url: $url}]')
    case "$status" in
      ring.js|js|html)
        alive=$(echo "$alive" | jq -c --argjson m "$m" '. + [$m]')
        ;;
      *)
        echo "  $status: $id ($url)" >&2
        ;;
    esac
  done
  rm -rf "$tmpdir"

  local len=$(echo "$alive" | jq 'length')
  echo "$len/$total members alive" >&2
  local first=$(echo "$alive" | jq -c '.[0]')
  local last=$(echo "$alive" | jq -c '.[-1]')

  redirect_html "https://github.com/umaring/umaring" "UMass Ring" > "$OUT/index.html"

  mkdir -p "$OUT/health"
  echo -n "OK" > "$OUT/health/index.html"

  mkdir -p "$OUT/all"
  echo "$alive" | jq -c '.' > "$OUT/all/index.html"

  # alive members get normal ring navigation
  for i in $(seq 0 $((len - 1))); do
    local member=$(echo "$alive" | jq -c ".[$i]")
    local prev=$(echo "$alive" | jq -c ".[(($i - 1 + $len) % $len)]")
    local next=$(echo "$alive" | jq -c ".[(($i + 1) % $len)]")
    local id=$(echo "$member" | jq -r '.id')
    write_member_files "$OUT" "$id" "$member" "$prev" "$next"
  done

  # dead members link to first/last of the live ring
  for i in $(seq 0 $((total - 1))); do
    local m=$(echo "$shuffled" | jq -c ".[$i]")
    local id=$(echo "$m" | jq -r '.id')
    [ -d "$OUT/$id" ] && continue
    write_member_files "$OUT" "$id" "$m" "$last" "$first"
  done

  local ring_data=$(echo "$alive" | jq -c '[.[] | {id, name, url}]')
  sed "s|RING_DATA_HERE|$ring_data|" ring.js > "$OUT/ring.js"

  mkdir -p "$OUT/status"
  status_page "$statuses" > "$OUT/status/index.html"

  cp umass.png "$OUT/umass.png"

  echo "Build complete: $OUT/"
}

case "${1:-build}" in
  build) build ;;
  check) check ;;
  *) echo "Usage: $0 [build|check]" >&2; exit 1 ;;
esac
