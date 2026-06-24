#!/usr/bin/env bash
# Tạo TrayLink Dev.app — macOS ghi nhận Accessibility ổn định hơn .app bundle.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEBUG_DIR="$ROOT/src-tauri/target/debug"
BINARY="$DEBUG_DIR/traylink"
APP="$DEBUG_DIR/TrayLink Dev.app"
IDENTIFIER="com.phamminhkha.traylink"

if [[ ! -f "$BINARY" ]]; then
  echo "Chưa có $BINARY — chạy cargo build hoặc tauri dev trước."
  exit 1
fi

mkdir -p "$APP/Contents/MacOS"

cat > "$APP/Contents/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleExecutable</key>
  <string>traylink</string>
  <key>CFBundleIdentifier</key>
  <string>${IDENTIFIER}</string>
  <key>CFBundleName</key>
  <string>TrayLink Dev</string>
  <key>CFBundleDisplayName</key>
  <string>TrayLink Dev</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.19</string>
  <key>CFBundleVersion</key>
  <string>0.1.19</string>
  <key>LSMinimumSystemVersion</key>
  <string>10.13</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>NSAppleEventsUsageDescription</key>
  <string>TrayLink cần Apple Events để focus app và gửi phím tắt.</string>
</dict>
</plist>
EOF

cp -f "$BINARY" "$APP/Contents/MacOS/traylink"
chmod +x "$APP/Contents/MacOS/traylink"
xattr -cr "$APP" 2>/dev/null || true

codesign --force --sign - --identifier "$IDENTIFIER" --deep "$APP"

echo "Đã tạo: $APP"
echo "→ Thêm *TrayLink Dev.app* (không phải file traylink lẻ) vào Accessibility."
