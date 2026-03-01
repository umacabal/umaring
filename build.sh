#!/usr/bin/env bash
set -euo pipefail

OUT="out"
rm -rf "$OUT"
mkdir -p "$OUT"

MEMBERS=$(cat members.json)
WEEK=$(( $(date +%s) / (7 * 86400) ))

# Deterministic weekly shuffle
SHUFFLED=$(echo "$MEMBERS" | jq -c '.[]' | while read -r m; do
  id=$(echo "$m" | jq -r '.id')
  key=$(echo -n "$WEEK-$id" | sha256sum | awk '{print $1}')
  echo "$key $m"
done | sort | cut -d' ' -f2- | jq -s '.')

LEN=$(echo "$SHUFFLED" | jq 'length')

cat > "$OUT/index.html" <<'EOF'
<!DOCTYPE html>
<html>
<head><meta http-equiv="refresh" content="0; url=https://github.com/umaring/umaring"></head>
<body>Redirecting to <a href="https://github.com/umaring/umaring">UMass Ring</a></body>
</html>
EOF

mkdir -p "$OUT/health"
echo -n "OK" > "$OUT/health/index.html"

mkdir -p "$OUT/all"
echo "$SHUFFLED" | jq -c '.' > "$OUT/all/index.html"

for i in $(seq 0 $((LEN - 1))); do
  MEMBER=$(echo "$SHUFFLED" | jq -c ".[$i]")
  PREV=$(echo "$SHUFFLED" | jq -c ".[(($i - 1 + $LEN) % $LEN)]")
  NEXT=$(echo "$SHUFFLED" | jq -c ".[(($i + 1) % $LEN)]")

  ID=$(echo "$MEMBER" | jq -r '.id')
  PREV_URL=$(echo "$PREV" | jq -r '.url')
  PREV_NAME=$(echo "$PREV" | jq -r '.name')
  NEXT_URL=$(echo "$NEXT" | jq -r '.url')
  NEXT_NAME=$(echo "$NEXT" | jq -r '.name')

  DATA=$(jq -nc --argjson prev "$PREV" --argjson member "$MEMBER" --argjson next "$NEXT" \
    '{prev: $prev, member: $member, next: $next}')

  mkdir -p "$OUT/$ID/prev" "$OUT/$ID/next"
  echo "$DATA" > "$OUT/$ID/index.html"
  echo "$DATA" > "$OUT/$ID.json"

  cat > "$OUT/$ID/prev/index.html" <<REOF
<!DOCTYPE html>
<html>
<head><meta http-equiv="refresh" content="0; url=$PREV_URL"></head>
<body>Redirecting to <a href="$PREV_URL">$PREV_NAME</a></body>
</html>
REOF

  cat > "$OUT/$ID/next/index.html" <<REOF
<!DOCTYPE html>
<html>
<head><meta http-equiv="refresh" content="0; url=$NEXT_URL"></head>
<body>Redirecting to <a href="$NEXT_URL">$NEXT_NAME</a></body>
</html>
REOF
done

RING_DATA=$(echo "$SHUFFLED" | jq -c '[.[] | {id, name, url}]')
sed "s|RING_DATA_HERE|$RING_DATA|" ring.js > "$OUT/ring.js"

cp umass.png "$OUT/umass.png"

echo "Build complete: $OUT/"
