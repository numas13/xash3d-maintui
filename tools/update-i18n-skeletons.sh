#!/usr/bin/env sh

script=$(readlink -f "$0")
scriptroot=$(dirname "$script")

cd "$scriptroot"
cargo run --bin i18n-skeleton -- > ../data/maintui_LANG_all.txt || exit 1
cargo run --bin i18n-skeleton -- strip > ../data/maintui_LANG_stripped.txt || exit 1
