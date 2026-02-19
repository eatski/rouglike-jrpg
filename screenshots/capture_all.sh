#!/bin/bash
set -e

echo "=== Screenshot capture all ==="

echo "[1/12] battle_command..."
cargo run -p screenshot-battle -- command

echo "[2/12] battle_spell..."
cargo run -p screenshot-battle -- spell

echo "[3/12] battle_target..."
cargo run -p screenshot-battle -- target

echo "[4/12] battle_message..."
cargo run -p screenshot-battle -- message

echo "[5/12] battle_victory..."
cargo run -p screenshot-battle -- victory

echo "[6/12] town_menu..."
cargo run -p screenshot-town -- menu

echo "[7/12] town_shop..."
cargo run -p screenshot-town -- shop

echo "[8/12] town_shop_char..."
cargo run -p screenshot-town -- shop_char

echo "[9/12] town_inn..."
cargo run -p screenshot-town -- inn

echo "[10/12] field..."
cargo run -p screenshot-field

echo "[11/12] cave..."
cargo run -p screenshot-cave

echo "[12/12] world..."
cargo run -p screenshot-world

echo ""
echo "=== Done! ==="
ls -la screenshots/output/
