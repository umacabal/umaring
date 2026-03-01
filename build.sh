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

detect_domain() {
  local text="$1"
  echo "$text" | grep -qi 'umaring\.mkr\.cx' && { echo "mkr.cx"; return; }
  echo "$text" | grep -qi 'umaring\.github\.io' && { echo "github.io"; return; }
  echo "unknown"
}

# outputs: status domain (two lines)
check_member() {
  local url="$1" id="$2"
  local html
  html=$(curl -sfL --connect-timeout 2 --max-time 5 "$url" 2>/dev/null) || { printf "offline\nunknown"; return; }

  local scripts
  scripts=$(echo "$html" | grep -oiE 'src=['"'"'"]?[^'"'"'" >]+' | sed "s/^src=['\"]\\?//")

  for src in $scripts; do
    if echo "$src" | grep -qiE "umaring\\.(mkr\\.cx|github\\.io)/${id}\\.js"; then
      printf "member.js\n%s" "$(detect_domain "$src")"
      return
    fi
  done

  for src in $scripts; do
    if echo "$src" | grep -qiE 'umaring\.(mkr\.cx|github\.io)/ring\.js'; then
      printf "ring.js\n%s" "$(detect_domain "$src")"
      return
    fi
  done

  local jstmp=$(mktemp)
  for src in $scripts; do
    case "$src" in *.js|*.js\?*) ;; *) continue ;; esac
    case "$src" in
      http*) ;;
      //*) src="https:$src" ;;
      /*) src="${url%/}$src" ;;
      *) src="${url%/}/$src" ;;
    esac
    curl -sfL --connect-timeout 2 --max-time 5 "$src" -o "$jstmp" 2>/dev/null || continue
    if grep -qi "umaring" "$jstmp"; then
      printf "js\n%s" "$(detect_domain "$(cat "$jstmp")")"
      rm -f "$jstmp"
      return
    fi
  done
  rm -f "$jstmp"

  if echo "$html" | grep -qi "umaring"; then
    printf "html\n%s" "$(detect_domain "$html")"
    return
  fi
  printf "missing\nunknown"
}

index_page() {
  local data="$1"
  local date=$(date -u '+%Y-%m-%d %H:%M UTC')
  cat <<'HEADER'
<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>UMass Webring</title>
<style>
body { font-family: monospace; max-width: 640px; margin: 40px auto; padding: 0 20px; background: #111; color: #ccc; }
h1 { font-size: 1.4em; color: #fff; }
h2 { font-size: 1.1em; color: #fff; margin-top: 30px; }
p { line-height: 1.5; }
a { color: #8bf; }
table { border-collapse: collapse; width: 100%; }
td { padding: 4px 8px; }
.ok { color: #6e6; }
.warn { color: #fc6; }
.err { color: #f66; }
.tag { font-size: 0.85em; }
.note { color: #999; font-size: 0.9em; line-height: 1.6; }
.time { color: #666; font-size: 0.85em; margin-top: 20px; }
</style>
</head>
<body>
<h1>UMass Ring</h1>
<p>A webring for UMass Amherst students and alumni.
Want to join? Add yourself to
<a href="https://github.com/umaring/umaring" target="_blank">the repo</a>
and submit a PR.</p>

<h2>Members</h2>
<table>
HEADER
  echo "$data" | jq -r '.[] |
    def tag(cls; txt): " <span class=\"tag \(cls)\">\(txt)</span>";
    def status_tag:
      if .status == "ring.js" then tag("warn"; "ring.js")
      elif .status == "missing" then tag("err"; "missing")
      elif .status == "offline" then tag("err"; "offline")
      else ""
      end;
    def domain_tag:
      if .domain == "mkr.cx" then tag("warn"; "mkr.cx")
      else ""
      end;
    "<tr><td><a href=\"\(.url)\" target=\"_blank\">\(.name)</a>\(status_tag)\(domain_tag)</td></tr>"'
  cat <<'NOTES'
</table>

<h2>Migration notes</h2>
<p class="note">
<span class="warn">mkr.cx</span> — The <b>umaring.mkr.cx</b> domain is being
retired. Update your URLs to use <b>umaring.github.io</b> instead.<br><br>
<span class="warn">ring.js</span> — The shared <b>ring.js</b> script is
being replaced with per-member scripts. Switch from<br>
<code>&lt;script src=".../ring.js?id=you"&gt;&lt;/script&gt;</code> to<br>
<code>&lt;script src="https://umaring.github.io/<b>you</b>.js"&gt;&lt;/script&gt;</code><br><br>
Most members will need to do both at once.
</p>
NOTES
  cat <<FOOTER
<img src="/umass.png" alt="UMass Ring" style="image-rendering: pixelated;">
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

  local prev_json=$(echo "$prev" | jq -c '{id, name, url}')
  local next_json=$(echo "$next" | jq -c '{id, name, url}')
  sed -e "s|PREV_DATA_HERE|$prev_json|" -e "s|NEXT_DATA_HERE|$next_json|" member.js > "$OUT/$id.js"
}

# run check_member in parallel, collect results into $tmpdir/{0,1,...}
parallel_check() {
  local shuffled="$1"
  local len=$(echo "$shuffled" | jq 'length')
  local tmpdir=$(mktemp -d)
  for i in $(seq 0 $((len - 1))); do
    local url=$(echo "$shuffled" | jq -r ".[$i].url")
    local id=$(echo "$shuffled" | jq -r ".[$i].id")
    ( check_member "$url" "$id" > "$tmpdir/$i" ) &
  done
  wait
  echo "$tmpdir"
}

check() {
  local shuffled=$(shuffle)
  local len=$(echo "$shuffled" | jq 'length')
  local tmpdir=$(parallel_check "$shuffled")
  for i in $(seq 0 $((len - 1))); do
    local id=$(echo "$shuffled" | jq -r ".[$i].id")
    local url=$(echo "$shuffled" | jq -r ".[$i].url")
    local status=$(sed -n '1p' "$tmpdir/$i")
    printf "%8s: %s (%s)\n" "$status" "$id" "$url"
  done
  rm -rf "$tmpdir"
}

check_domain() {
  local shuffled=$(shuffle)
  local len=$(echo "$shuffled" | jq 'length')
  local tmpdir=$(parallel_check "$shuffled")
  for i in $(seq 0 $((len - 1))); do
    local id=$(echo "$shuffled" | jq -r ".[$i].id")
    local url=$(echo "$shuffled" | jq -r ".[$i].url")
    local domain=$(sed -n '2p' "$tmpdir/$i")
    printf "%10s: %s (%s)\n" "$domain" "$id" "$url"
  done
  rm -rf "$tmpdir"
}

build() {
  local OUT="out"
  rm -rf "$OUT"
  mkdir -p "$OUT"

  local shuffled=$(shuffle)
  local total=$(echo "$shuffled" | jq 'length')

  echo "Checking members..." >&2
  local tmpdir=$(parallel_check "$shuffled")

  local alive="[]"
  local statuses="[]"
  for i in $(seq 0 $((total - 1))); do
    local m=$(echo "$shuffled" | jq -c ".[$i]")
    local url=$(echo "$m" | jq -r '.url')
    local id=$(echo "$m" | jq -r '.id')
    local name=$(echo "$m" | jq -r '.name')
    local status=$(sed -n '1p' "$tmpdir/$i")
    local domain=$(sed -n '2p' "$tmpdir/$i")
    statuses=$(echo "$statuses" | jq -c --arg s "$status" --arg d "$domain" --arg name "$name" --arg url "$url" \
      '. + [{status: $s, domain: $d, name: $name, url: $url}]')
    case "$status" in
      member.js|ring.js|js|html)
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

  mkdir -p "$OUT/health"
  echo -n "OK" > "$OUT/health/index.html"

  mkdir -p "$OUT/all"
  echo "$alive" | jq -c '.' > "$OUT/all/index.html"

  for i in $(seq 0 $((len - 1))); do
    local member=$(echo "$alive" | jq -c ".[$i]")
    local prev=$(echo "$alive" | jq -c ".[(($i - 1 + $len) % $len)]")
    local next=$(echo "$alive" | jq -c ".[(($i + 1) % $len)]")
    local id=$(echo "$member" | jq -r '.id')
    write_member_files "$OUT" "$id" "$member" "$prev" "$next"
  done

  for i in $(seq 0 $((total - 1))); do
    local m=$(echo "$shuffled" | jq -c ".[$i]")
    local id=$(echo "$m" | jq -r '.id')
    [ -d "$OUT/$id" ] && continue
    write_member_files "$OUT" "$id" "$m" "$last" "$first"
  done

  local ring_data=$(echo "$alive" | jq -c '[.[] | {id, name, url}]')
  sed "s|RING_DATA_HERE|$ring_data|" ring.js > "$OUT/ring.js"

  index_page "$statuses" > "$OUT/index.html"
  mkdir -p "$OUT/status"
  index_page "$statuses" > "$OUT/status/index.html"

  cp umass.png "$OUT/umass.png"

  echo "Build complete: $OUT/"
}

case "${1:-build}" in
  build) build ;;
  check) check ;;
  check_domain) check_domain ;;
  *) echo "Usage: $0 [build|check|check_domain]" >&2; exit 1 ;;
esac
