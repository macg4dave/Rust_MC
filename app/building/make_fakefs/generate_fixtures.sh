#!/usr/bin/env bash
set -euo pipefail

# POSIX fixtures generator: always creates 500 fixture files and a manifest



TOTAL=500

echo "Generating $TOTAL fixtures under $fixtures_dir"

# Helper to write an entry relative to fixtures_dir
emit() {
  local fpath="$1"
  relpath="${fpath#$fixtures_dir/}"
  printf "%s\n" "$relpath" >> "$manifest"
}

# Clear existing manifest (we will write fresh entries)
rm -f "$manifest"

# Create a few deterministic special files the tests expect
mkdir -p "$fixtures_dir/deep/level1/level2"
printf "emoji content" > "$fixtures_dir/emoji-😊"
emit "$fixtures_dir/emoji-😊"

printf "complex log" > "$fixtures_dir/COMPLEX.name.with.many.dots.log"
emit "$fixtures_dir/COMPLEX.name.with.many.dots.log"

printf "contains spaces and\ttabs" > "$fixtures_dir/spaces and tabs.txt"
emit "$fixtures_dir/spaces and tabs.txt"

printf "nested content" > "$fixtures_dir/deep/level1/level2/nested_file.txt"
emit "$fixtures_dir/deep/level1/level2/nested_file.txt"

count_created=4

# keep a list of created files (used for symlink targets)
FILES=()
# record the deterministic specials
FILES+=("$fixtures_dir/emoji-😊")
FILES+=("$fixtures_dir/COMPLEX.name.with.many.dots.log")
FILES+=("$fixtures_dir/spaces and tabs.txt")
FILES+=("$fixtures_dir/deep/level1/level2/nested_file.txt")

# helper to create a file with a given size (bytes)
create_file_of_size() {
  local path="$1"
  local size="$2"
  local block="0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-+_"
  mkdir -p "$(dirname "$path")"
  if [[ "$size" -le 0 ]]; then
    : > "$path"
  else
    local chunk="$block"
    local written=0
    : > "$path"
    while [[ $written -lt $size ]]; do
      echo -n "$chunk" >> "$path" || true
      written=$((written + ${#chunk}))
    done
    # truncate to exact size if available
    truncate -s "$size" "$path" 2>/dev/null || true
  fi
}

# Generate many files with tricky names and varied depth/sizes
i=0
while [[ $count_created -lt $TOTAL ]]; do
  depth=$((RANDOM % 6))
  dir=""
  for ((d=0; d<depth; d++)); do
    dir+="dir_$((RANDOM % 100))/"
  done

  case $((i % 12)) in
    0) name="file.with.many.dots.$(printf "%03d" $i).txt" ;;
    1) name="name#special&chars@$((1000+i)).log" ;;
    2) name="unicode-漢字-$i.bin" ;;
    3) name="emoji-🙂-$(printf "%03d" $i)" ;;
    4) name="complex;name;semi;$i.txt" ;;
    5) name="space name $i.txt" ;;
    6) name=".leading.dot.$i" ;;
    7) name="trailing-space-$i " ;;
    8) name="very-long-name-$(printf '%0.10d' $i)-$(head -c 24 /dev/urandom | base64 | tr -dc 'a-zA-Z0-9' )" ;;
    9) name="reserved%20name$i" ;;
    10) name="combining-á-$i" ;;
    *) name="file_$(printf "%04d" $i).txt" ;;
  esac

  # sanitize name: avoid problematic trailing or leading whitespace which can
  # cause permission/handling issues on some shells/filesystems. Replace
  # trailing whitespace with underscore and trim leading whitespace.
  safe_name=$(printf '%s' "$name" | sed -e 's/[[:space:]]\+$/_/' -e 's/^[[:space:]]\+//')

  fullpath="$fixtures_dir/${dir}${safe_name}"

  r=$((RANDOM % 10))
  if [[ $r -le 1 ]]; then
    size=0
  elif [[ $r -le 5 ]]; then
    size=$((10 + RANDOM % 200))
  elif [[ $r -le 8 ]]; then
    size=$((500 + RANDOM % 2000))
  else
    size=$((10000 + RANDOM % 50000))
  fi

  create_file_of_size "$fullpath" "$size"
  emit "$fullpath"

  # track this file for later possible symlink targets
  FILES+=("$fullpath")

  # --- advanced attributes (best-effort) ---
  # extended attributes (user.*) if available
  if command -v setfattr >/dev/null 2>&1 && [[ $((RANDOM % 100)) -lt 30 ]]; then
    xname="user.random$((RANDOM % 100))"
    xval="xattr-$RANDOM"
    setfattr -n "$xname" -v "$xval" "$fullpath" 2>/dev/null || true
  fi

  # random permission tweaks
  if [[ $((RANDOM % 100)) -lt 40 ]]; then
    case $((RANDOM % 7)) in
      0) chmod 644 "$fullpath" 2>/dev/null || true ;;
      1) chmod 600 "$fullpath" 2>/dev/null || true ;;
      2) chmod 666 "$fullpath" 2>/dev/null || true ;;
      3) chmod 755 "$fullpath" 2>/dev/null || true ;;
      4) chmod 700 "$fullpath" 2>/dev/null || true ;;
      5) chmod 444 "$fullpath" 2>/dev/null || true ;;
      *) chmod 664 "$fullpath" 2>/dev/null || true ;;
    esac
  fi

  # random timestamp change
  if [[ $((RANDOM % 100)) -lt 50 ]]; then
    days=$((RANDOM % 365))
    secs=$((RANDOM % 86400))
    touch -d "-$days days -$secs seconds" "$fullpath" 2>/dev/null || true
  fi

  # ACLs using setfacl if available
  if command -v setfacl >/dev/null 2>&1 && [[ $((RANDOM % 100)) -lt 10 ]]; then
    setfacl -m u:$(id -un):r-- "$fullpath" 2>/dev/null || true
  fi

  # occasionally create a symlink pointing to an existing file
  if [[ ${#FILES[@]} -gt 1 ]] && [[ $((RANDOM % 100)) -lt 8 ]]; then
    # pick a random existing target (not the newly-created file)
    pick=$((RANDOM % (${#FILES[@]} - 1)))
    tgt="${FILES[$pick]}"
    if [[ "$tgt" != "$fullpath" ]]; then
      ln -sf "$tgt" "${fullpath}.link" 2>/dev/null || true
      emit "${fullpath}.link"
    fi
  fi

  # create FIFO occasionally in this file's directory
  if [[ $((RANDOM % 1000)) -lt 8 ]]; then
    dir_for_fifo=$(dirname "$fullpath")
    mkfifo "$dir_for_fifo/fifo_$(date +%s%N)_$RANDOM" 2>/dev/null || true
  fi

  count_created=$((count_created + 1))
  i=$((i + 1))
done

echo "Wrote $count_created entries to $manifest"
exit 0
