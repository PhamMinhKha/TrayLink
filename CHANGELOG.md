# Changelog

Tất cả thay đổi đáng chú ý của dự án TrayLink được ghi tại đây.

Format dựa trên [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
và dự án tuân theo [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.23] - 2026-06-25

### Added

- **Dashboard** — hiển thị version app cạnh link GitHub (từ `getServerStatus()`)
- **API `/status`** — thêm field `endpoints` liệt kê toàn bộ route HTTP (kể cả `/monitor-status`)

### Changed

- **Monitor popup** — quota Claude/Codex dùng **progress bar** thay vòng tròn; tăng cỡ chữ và padding cho dễ đọc

### Documentation

- README: screenshot popup Monitor, hướng dẫn click tray + bảng API Claude/Codex/System/`/monitor-status`

## [0.1.22] - 2026-06-25

### Added

- **API `/monitor-status`** — một endpoint trả về trạng thái Claude, Codex và System metrics (theo toggle Settings); Tauri command `getMonitorStatus()`; popup tray gọi gộp thay vì 3 request riêng

## [0.1.21] - 2026-06-25

### Added

- **System metrics** — bật/tắt từng metric (CPU, RAM, ổ đĩa, mạng, nhiệt CPU/pin, quạt) trong Settings; hiển thị section **System** trong popup tray; API **`GET/POST /system-metrics`**
- Thu thập cross-platform qua `sysinfo`; nhiệt/quạt best-effort trên Windows (WMI) và macOS (`ioreg`), trả `unsupported` khi không đọc được
- **Claude quota trên Windows** — đọc `%USERPROFILE%\.claude\.credentials.json` và `%USERPROFILE%\.claude.json` (kể cả `claudeAiOauth`), fallback Windows Credential Manager (kể cả target có hash theo `CLAUDE_CONFIG_DIR`), hỗ trợ `CLAUDE_CODE_OAUTH_TOKEN`; diagnostics + `where claude` cho Windows

## [0.1.20] - 2026-06-25

### Added

- **Popup tray AI Quota** — click trái icon tray xem vòng % quota **Claude Code** và **Codex** (5 giờ / 7 ngày); click phải vẫn mở menu (thêm mục **Usage**)
- Cửa sổ popup riêng (`usage-popup`) — gauge tròn, làm mới, nút **Mở Settings**
- **Light / Dark theme** — toggle ở header dashboard, popup tray và mục **Giao diện** trong Settings; lưu `localStorage`, đồng bộ realtime giữa các cửa sổ qua event Tauri
- Theme **`traylink-theme`**: dark navy gradient; light dùng palette shadcn mặc định (dịu, không chói)
- Command `show_dashboard(tab?)` — popup mở thẳng tab Settings

### Changed

- Dashboard dùng theme đồng bộ với popup thay vì chỉ light mặc định
- Thông báo lỗi quota Claude/Codex gọn, tiếng Việt (không dump JSON API)

### Fixed

- **Server không khởi động** sau v0.1.19 — xung đột rustls `ring` / `aws-lc-rs` khi thêm `reqwest`; dùng `rustls-tls-webpki-roots-no-provider` + `install_default()`
- Popup tự co giãn chiều cao theo nội dung, không scroll; nền opaque tránh góc trắng trên macOS

## [0.1.15] - 2026-05-28

### Added

- **Phím tắt theo từng app** — nhiều binding/app (Save, Export, Copy…); dialog quản lý trong Allowlist
- **Sửa phím tắt** đã thêm; **icon** Font Awesome (kho có sẵn) hoặc upload hình tùy chỉnh
- **Global shortcut** plugin (`tauri-plugin-global-shortcut`) — chỉ đăng ký phím loại **Mở / focus app**; phím **Gửi phím** chỉ qua Remote/API (không chặn phím gốc của app)
- API **`GET/POST /send-hotkey`** — Remote gọi phím tắt theo `app` + `hotkey` id
- Modal **API** từng app: copy link/curl **`/send-hotkey`** cho từng phím đã cấu hình
- README: tài liệu gọi **`/send-hotkey`** (GET/POST, ví dụ curl, lấy danh sách qua `/api/deck`)
- **Profile trình duyệt** — chọn profile Chrome, Edge, Firefox… khi thêm/sửa app; quét profile từ hệ thống
- Remote Deck: chạm app có phím → **mở app trên PC** và vào màn phím tắt; lưới tile giống launcher
- Remote: tile **Trở về** lớn trong màn phím tắt (dễ bấm trên điện thoại)
- Remote: hiển thị icon phím tắt (Font Awesome / hình custom); route `/api/icons/hotkey/{app}/{id}`
- **Accessibility (macOS)** — panel Settings, thử gửi phím, ký dev binary, **TrayLink Dev.app**, reset TCC
- Script `npm run sign:dev`, `npm run dev:app`; `scripts/sign-dev-macos.sh`, `scripts/sync-dev-app.sh`
- macOS: gửi phím qua **CGEvent** trên **main thread**; focus app bằng AppleScript
- macOS: quét app **đệ quy** trong `/Applications` (Photoshop trong subfolder)
- macOS: `Info.plist` Apple Events; entitlement automation

### Changed

- Remote: app **không có phím tắt** chỉ mở app, không vào màn phím (tránh màn trống)
- Remote: **ẩn toast** khi mở app / gửi phím thành công (giữ phản hồi tile + rung)
- Remote màn phím tắt: bỏ header logo/tên lớn; dùng lưới `.grid` gọn như launcher
- Dashboard: **ẩn cột Path** trong bảng Apps & Commands
- Accessibility: không auto-prompt mỗi lần khởi động; hướng dẫn dev dùng **TrayLink Dev.app** thay vì binary `traylink` lẻ

### Fixed

- macOS: không crash khi phím tắt không hợp lệ (normalize accelerator, skip lỗi khi đăng ký global)
- macOS: không chặn phím tắt gốc (⌘S, ⌘C…) khi TrayLink đăng ký global cho Photoshop
- macOS: gửi phím từ Remote/API hoạt động (main thread + CGEvent thay osascript/enigo)
- macOS dev: chữ ký ổn định `com.phamminhkha.traylink` + app bundle cho Accessibility
- macOS: mở trình duyệt với **profile** được **focus** đúng (launch binary + AppleScript activate)

## [0.1.14] - 2026-05-28

### Added

- **PWA Remote Deck** — `manifest.webmanifest`, service worker, icon 180/192/512; cài shortcut trên iPhone/Android qua HTTPS
- Nút **Thêm shortcut** (góc trái, mobile) — hướng dẫn cài PWA / Thêm vào Màn hình chính
- Token Remote lưu `localStorage` + đồng bộ `?token=` trên URL; nhớ ưu tiên HTTPS sau lần mở đầu
- Chứng chỉ TLS **lưu trên disk** — tái sử dụng qua các lần restart (chỉ tạo mới khi IP LAN đổi)
- Hướng dẫn **fullscreen trên iPhone (Safari + PWA)** — [docs/allow-https/README.md](docs/allow-https/README.md#fullscreen-trên-iphone-safari--pwa)

### Changed

- Header Remote căn giữa màn hình, cùng hàng với toolbar
- Toolbar trái: nút **Upload** đứng trước **Thêm shortcut**; icon shortcut dạng vuông `+`
- Ẩn nút **Thêm shortcut** khi bật toàn màn hình (immersive / native fullscreen)

### Fixed

- F5 / refresh không mất token khi chuyển HTTP → HTTPS
- Không cần chấp nhận lại chứng chỉ HTTPS sau mỗi lần restart TrayLink (cùng IP LAN)

## [0.1.13] - 2026-05-28

### Added

- **Upload file từ điện thoại lên PC** qua Remote Deck — nút upload (góc trái), chọn ảnh/video/tài liệu, thanh tiến trình
- API `POST /api/upload` (multipart, field `file`; tùy chọn `open=1` để mở file sau khi lưu)
- File lưu vào `Downloads/TrayLink` trên máy tính; giới hạn **5 GB**/file
- Hỗ trợ ảnh, video, audio, PDF, Office, zip và nhiều định dạng phổ biến khác
- `GET /api/deck` trả thêm `upload_enabled`, `max_upload_mb` cho giao diện mobile

### Changed

- README: hướng dẫn upload từ Remote Deck và ví dụ `curl`

## [0.1.12] - 2026-05-26

### Fixed

- Windows: mở **Chọn app** không còn PowerShell loop (bỏ resolve/icon hàng loạt; chỉ resolve `.lnk` → `.exe` khi chọn một app)
- Windows: vẫn mở app đúng sau khi chọn (path `.exe` lưu vào config)

### Changed

- CI: workflow **Build Release** chỉ chạy khi push tag `v*` (không build trùng mỗi lần push `main`)

## [0.1.11] - 2026-05-26

### Fixed

- CI Windows: export API `windows_lnk` qua `apps` module — sửa lỗi build `module windows_lnk is private` (E0603)

## [0.1.10] - 2026-05-26

### Fixed

- Windows: **Chọn app** từ Start Menu resolve shortcut `.lnk` → `.exe` — mở app không còn lỗi *not a valid Win32 application* (193)
- Windows: hiện lại **icon** app trong dropdown Chọn app (lấy từ file `.exe` thay vì `.lnk`)
- Windows: migration tự sửa app đã lưu trong config còn dùng path `.lnk`
- Windows: launch đúng cho `.exe`, `.lnk` (fallback `cmd start`) và app mở kèm URL

### Added

- Command `resolve_launch_path` — resolve path khi browse hoặc chọn shortcut trên Windows

## [0.1.9] - 2026-05-26

### Fixed

- Windows: bấm **Chọn app** không còn mở PowerShell liên tục (bỏ tải icon từng shortcut trong dropdown)
- Remote Deck / API qua HTTPS: sửa lỗi `Missing request extension: ConnectInfo` khi chạm mở app trên PC
- PowerShell lấy icon ẩn cửa sổ (`-WindowStyle Hidden`); bỏ qua icon cho file `.lnk`

## [0.1.8] - 2026-05-26

### Added

- Server **HTTPS** trên port `HTTP + 1` (mặc định `8766`) — chứng chỉ tự ký cho IP LAN, bật **Wake Lock** giữ màn hình sáng trên Remote Deck
- Checkbox **Mở bằng URL** (`url_enabled`) khi thêm/sửa app — hỗ trợ trình duyệt không tự nhận diện (Arc, Zen, …)
- Fallback video `nosleep.mp4` trên Remote khi không dùng được Wake Lock
- Dashboard: link **Remote Deck HTTPS** riêng, copy/QR HTTPS
- Tài liệu [docs/allow-https/](docs/allow-https/) — hướng dẫn vượt cảnh báo chứng chỉ (ảnh từng bước)

### Changed

- Remote Deck: Wake Lock API chuẩn, tự xin lại lock khi bị thu hồi, chuyển HTTP → HTTPS khi bật giữ màn hình sáng
- README: ưu tiên ảnh mobile, hướng dẫn HTTPS song song (bước 1 & 2), video demo xuống cuối

### Fixed

- Giữ màn hình sáng không hoạt động qua `http://` LAN — cần mở `https://` (secure context)

## [0.1.7] - 2026-05-26

### Added

- Trường **Tên hiển thị** (`name`) cho app — Remote Deck hiện tên thay vì key
- Nút **Sửa** + dialog chỉnh tên, path, URL mặc định (key giữ nguyên)
- Cache icon app/deck trong RAM — chuyển tab Remote Deck nhanh hơn

### Changed

- Tab Remote Deck giữ mounted (`forceMount`), refresh nền khi quay lại
- Icon app trong bảng **Apps đã đăng ký** lớn hơn (48px), cột icon cố định
- Ẩn cột URL khỏi bảng — xem/sửa trong dialog **Sửa**

### Fixed

- Sửa Chrome: dialog **Sửa** không load URL cũ do effect xóa URL nhầm lúc mở

## [0.1.6] - 2026-05-26

### Added

- **Remote Deck** — mở link trên điện thoại/tablet cùng Wi‑Fi để điều khiển PC dạng grid icon kiểu Stream Deck
- Trang mobile `remote.html`: grid app/command, chạm để mở app trên PC
- API remote: `GET /`, `GET /remote`, `GET /api/deck`, `GET /api/icons/{kind}/{key}`
- Tab **Remote Deck** trên Dashboard: xem trước, sắp xếp thứ tự, ẩn/hiện app và command
- Kéo-thả sắp xếp bằng `@dnd-kit` (drag handle ≡), tự lưu khi thả; nút ↑/↓ làm fallback
- Icon tùy chỉnh cho từng ô deck (chọn file / khôi phục mặc định)
- Modal QR code để mở Remote Deck nhanh trên điện thoại
- Toolbar mobile: toàn màn hình (Fullscreen API + chế độ immersive cho iOS Safari), giữ màn hình sáng (Wake Lock)
- Icon SVG mặc định khi app chưa có PNG (tránh lỗi 404 trên mobile)
- Cấu hình `remote_deck`: `display_order`, `app_order`, `command_order`, `hidden_apps`, `hidden_commands`, `custom_icons`
- Overview hiển thị link Remote Deck và nút copy/QR

### Changed

- README bổ sung hướng dẫn Remote Deck
- Allowlist editor đồng bộ layout remote deck khi thêm/xóa app hoặc command

## [0.1.0] - 2026-05-26

### Added

- Ứng dụng Tauri 2 + Rust + React + shadcn/ui chạy nền với system tray
- HTTP REST API trên `127.0.0.1:8765` (port cấu hình được):
  - `GET /status` — health check
  - `POST /open-app` — mở app theo allowlist (hỗ trợ `url` cho trình duyệt)
  - `POST /open-file` — mở file bằng app mặc định OS
  - `POST /exec` — chạy command key đã whitelist
- Xác thực Bearer token / `X-API-Token` cho các endpoint POST
- Allowlist apps và command whitelist lưu bằng `tauri-plugin-store`
- Launcher cross-platform (Windows / macOS / Linux)
- System tray: Open Dashboard, Restart Server, Exit
- Ẩn window khi đóng — app chạy nền trên tray
- Autostart khi boot (bật/tắt từ Dashboard)
- Single instance — mở lại app sẽ focus Dashboard
- Dashboard 4 tab: Overview, Request Log, Apps & Commands, Settings
- Chọn app từ danh sách app đã cài hoặc duyệt file (dialog)
- Hỗ trợ URL mặc định cho trình duyệt (Chrome, Safari, Firefox, Edge, …)
- Request log ring buffer (~200 entries)
- README hướng dẫn cài đặt và sử dụng API

### Security

- API chỉ bind `127.0.0.1` (localhost)
- Chỉ mở app/command có trong allowlist
- `open-file` chặn path traversal và system paths
- `exec` chỉ chấp nhận command key, không chạy raw shell

[0.1.23]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.23
[0.1.22]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.22
[0.1.21]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.21
[0.1.20]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.20
[0.1.15]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.15
[0.1.14]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.14
[0.1.13]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.13
[0.1.12]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.12
[0.1.11]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.11
[0.1.10]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.10
[0.1.9]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.9
[0.1.8]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.8
[0.1.7]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.7
[0.1.6]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.6
[0.1.0]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.0
