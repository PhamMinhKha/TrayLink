# TrayLink

App launcher chạy nền trên PC, lắng nghe HTTP API trên mạng LAN để mở ứng dụng, mở file, hoặc chạy lệnh đã whitelist.

**Stack:** Tauri 2 + Rust + React + shadcn/ui

![Dashboard — Apps & Commands](docs/screenshot1.png)

## Tính năng

- HTTP REST API trên LAN (mặc định port `8765`, IP LAN tự phát hiện)
- **GET** bằng link URL hoặc **POST** bằng curl/JSON — bật/tắt GET trong Settings
- Token API **tùy chọn** (mặc định tắt, phù hợp dùng trong LAN)
- System tray (menu bar / system tray): Open Dashboard, Restart Server, Exit
- Chạy nền — đóng cửa sổ chỉ ẩn app, không thoát (macOS: không hiện Dock)
- Autostart khi boot
- Allowlist apps + command whitelist
- Dashboard: trạng thái server, request log, quản lý app, settings, copy link API
- **Remote Deck**: mở link trên điện thoại/tablet cùng Wi‑Fi → grid icon app kiểu Stream Deck, chạm để mở app trên PC
- **Upload từ điện thoại**: gửi ảnh, video, tài liệu từ Remote Deck lên PC (lưu vào `Downloads/TrayLink`)
- **Phím tắt theo app**: cấu hình trong Dashboard, gọi qua Remote Deck hoặc HTTP API `/send-hotkey`
- **Monitor popup** — click icon tray xem quota **Claude Code** / **Codex** và thông số **System** (CPU, RAM, disk, mạng…) trong một popup
- **Theo dõi tùy chọn** — bật/tắt từng metric trong Settings; API riêng hoặc gộp qua `/monitor-status`

![Monitor popup — Claude, Codex, System metrics](docs/monitor-popup.png)

*Click trái icon tray → popup Monitor: quota AI (5 giờ / 7 ngày) và thông số máy. Bật từng loại trong Settings → Claude / Codex / System metrics.*

## Hướng dẫn sử dụng

### Remote Deck trên điện thoại

Mở link **HTTPS** trên điện thoại cùng Wi‑Fi → grid icon app (kiểu Stream Deck), chạm để mở app trên PC. Nút **mặt trời** góc phải giữ màn hình sáng — cần HTTPS và [vượt cảnh báo chứng chỉ](#vượt-cảnh-báo-https-chứng-chỉ-tự-ký) lần đầu.

![TrayLink Remote trên điện thoại — grid app, toàn màn hình và giữ màn hình sáng](docs/mobile%20screen%20shot.jpg)

### Vượt cảnh báo HTTPS (chứng chỉ tự ký)

TrayLink chạy HTTPS trên port **8766** (HTTP + 1) với chứng chỉ **tự ký** — trình duyệt điện thoại sẽ cảnh báo lần đầu. Đây là bước **bắt buộc** để vào Remote và bật **giữ màn hình sáng** (Wake Lock chỉ hoạt động qua `https://`, không qua `http://`).

1. Dashboard → **Overview** → copy link **Remote Deck — HTTPS** (vd `https://192.168.1.x:8766/remote`)
2. Mở link trên điện thoại (cùng Wi‑Fi với PC)

<table>
  <tr>
    <td width="50%" valign="top">
      <p><strong>Bước 1 — Cảnh báo</strong></p>
      <p>Safari/Chrome báo kết nối không riêng tư — bình thường, chọn tiếp tục.</p>
      <img src="docs/allow-https/alert%20https.jpg" alt="Cảnh báo HTTPS — kết nối không riêng tư" width="280" />
    </td>
    <td width="50%" valign="top">
      <p><strong>Bước 2 — Cho phép</strong></p>
      <p>Chọn <strong>Tiếp tục</strong>, hoặc <strong>Advanced</strong> → <strong>Proceed</strong> / <strong>Visit this website</strong>.</p>
      <img src="docs/allow-https/allow%20https.jpg" alt="Cho phép truy cập HTTPS — vượt chứng chỉ tự ký" width="280" />
    </td>
  </tr>
</table>

3. Sau khi vào được trang Remote → bật nút **mặt trời** (góc phải) để giữ màn hình sáng

### Fullscreen trên iPhone (Safari + PWA)

Safari trên iPhone **không hỗ trợ** Fullscreen API trong tab thường — thanh địa chỉ và thanh công cụ Safari vẫn hiện khi bấm nút toàn màn hình trong trang. Cách tốt nhất là **cài PWA** (Thêm vào Màn hình chính) rồi mở từ icon trên Home Screen.

**Bước 1 — Mở bằng HTTPS**

1. Copy link **Remote Deck — HTTPS** từ Dashboard (vd `https://192.168.1.x:8766/remote`)
2. Mở link trong **Safari** (không dùng Chrome/Facebook in-app browser nếu có thể)
3. [Chấp nhận chứng chỉ tự ký](#vượt-cảnh-báo-https-chứng-chỉ-tự-ký) lần đầu

**Bước 2 — Thêm vào Màn hình chính**

1. Chạm nút **Chia sẻ** (hình vuông có mũi tên lên) trên thanh Safari
2. Cuộn xuống → chọn **Thêm vào Màn hình chính** (*Add to Home Screen*)
3. (Tuỳ chọn) Đổi tên thành `TrayLink` → chạm **Thêm**

Hoặc trên trang Remote: chạm nút **Thêm shortcut** (vuông `+`, góc trái) — app sẽ nhắc thao tác tương tự.

**Bước 3 — Mở từ Home Screen (fullscreen thật)**

1. Thoát Safari, mở app **TrayLink** từ icon vừa thêm
2. App chạy **standalone** — không còn thanh Safari, giống app native
3. Chạm nút **mặt trời** để giữ màn hình sáng
4. (Tuỳ chọn) Chạm nút **toàn màn hình** (góc phải) để ẩn header, grid chiếm gần hết màn hình

**Mẹo iPhone**

| Mục tiêu | Cách làm |
|----------|----------|
| Ẩn thanh Safari | Mở từ **Home Screen**, không mở link trong tab Safari |
| Giữ màn hình sáng | Bắt buộc HTTPS + bật nút mặt trời; tắt **Tiết kiệm pin** |
| Không nhập lại token | Dùng link HTTPS có `?token=...` từ Dashboard; token được nhớ sau lần đầu |
| Treo tường làm bảng điều khiển | Cài PWA + bật giữ sáng + chế độ toàn màn hình trong app |

📖 Chi tiết thêm: [docs/allow-https/README.md — Fullscreen iPhone](docs/allow-https/README.md#fullscreen-trên-iphone-safari--pwa)

| | HTTP `:8765` | HTTPS `:8766` |
|---|-------------|---------------|
| Mở app từ điện thoại | Có | Có |
| Giữ màn hình sáng | Không | Có |
| Cần bước ảnh trên | Không | Có (lần đầu) |

📖 Chi tiết & xử lý sự cố: [docs/allow-https/README.md](docs/allow-https/README.md)

### Các bước nhanh

1. **Cài & mở TrayLink** — app chạy nền, icon nằm trên **menu bar** (macOS) hoặc **system tray** (Windows).
2. **Monitor** — click **trái** icon tray để xem quota Claude/Codex và thông số hệ thống (bật trong Settings trước).
3. **Open Dashboard** từ menu tray → tab **Apps & Commands**.
4. **Thêm app** — nhập key (vd: `chrome`), chọn app từ danh sách hoặc duyệt file `.app` / `.exe`.
5. **Test** — thử mở app trên máy này.
6. **API** — mở modal, **Copy** link GET hoặc lệnh curl POST (dùng IP LAN, vd `http://192.168.1.x:8765/open-app?app=chrome`).
7. Dán link vào **Stream Deck**, shortcut, trình duyệt trên điện thoại/tablet cùng Wi‑Fi, hoặc gọi từ script.
8. **Remote Deck** — Dashboard → **Overview** → copy link **HTTPS** Remote Deck, mở trên điện thoại để điều khiển app dạng grid icon (bật nút mặt trời để giữ màn hình sáng). Sắp xếp thứ tự và ẩn/hiện app trong tab **Remote Deck**. Xem [hướng dẫn HTTPS](docs/allow-https/README.md).

![Modal API — copy link GET / curl POST](docs/screenshot2.png)

### Ví dụ gọi API từ thiết bị khác (LAN)

```bash
# Kiểm tra server (GET)
curl http://192.168.1.x:8765/status

# Mở app bằng link (GET — cần bật "Cho phép GET" trong Settings)
open "http://192.168.1.x:8765/open-app?app=chrome"

# Hoặc POST
curl -X POST http://192.168.1.x:8765/open-app \
  -H "Content-Type: application/json" \
  -d '{"app":"chrome"}'
```

Thay `192.168.1.x` bằng **IP LAN** hiển thị trong Dashboard → **Overview** hoặc modal **API**.

### Video — Loa AI văn phòng - Đồng hồ Ai mở ứng dụng thông qua API

![Hướng dẫn sử dụng — đồng hồ AI mở ứng dụng qua TrayLink](docs/cach-su-dung.gif)

*Đồng hồ gọi API để mở app*

**Sản phẩm trong video:** [Loa AI Văn Phòng — loaai.me](https://loaai.me)

## Tải bản cài đặt

Tải từ [GitHub Releases](https://github.com/PhamMinhKha/TrayLink/releases) hoặc build local (xem bên dưới).

| Nền tảng | File |
|----------|------|
| macOS (mọi Mac) | `TrayLink-macos-universal.zip` |
| macOS Apple Silicon | `TrayLink-macos-arm64.zip` |
| macOS Intel | `TrayLink-macos-x64.zip` |
| Windows | `TrayLink_*_x64-setup.exe` |

## Yêu cầu (dev / build)

- [Node.js](https://nodejs.org/) 18+
- [Rust](https://rustup.rs/) 1.88+
- macOS / Windows / Linux

## Cài đặt & chạy dev

```bash
npm install
npm run tauri dev
```

App khởi động ẩn — click icon tray → **Open Dashboard**.

## Build production

### Build local

```bash
npm run build:macos              # macOS (kiến trúc máy hiện tại)
npm run build:macos:universal    # macOS Intel + Apple Silicon
npm run build:windows            # Windows (chạy trên máy Windows)
```

Chi tiết: [`scripts/BUILD.md`](scripts/BUILD.md)

### Release qua GitHub Actions

#### Tải nhanh (mọi lần build)

Sau khi build xong, vào **Actions → Build Release → run mới nhất → Artifacts**:
- `traylink-macos-universal` — macOS Intel + Apple Silicon (một file)
- `traylink-macos-arm64` — macOS Apple Silicon (M1/M2/M3…)
- `traylink-macos-x64` — macOS Intel
- `traylink-windows`

#### Xuất hiện trên trang Releases

**Chỉ khi push tag `v*`** — workflow sẽ tạo [GitHub Release](https://github.com/PhamMinhKha/TrayLink/releases) và đính kèm file cài đặt:

```bash
git tag v0.1.0
git push origin v0.1.0
```

File trên Releases (ví dụ `v0.1.0`):
- `TrayLink-macos-universal.zip` — chạy cả Intel + Apple Silicon
- `TrayLink-macos-arm64.zip` — chỉ Apple Silicon
- `TrayLink-macos-x64.zip` — chỉ Intel
- `TrayLink_0.1.0_x64-setup.exe` — installer Windows (nếu build NSIS thành công)
- `.dmg` — nếu bundling DMG thành công trên CI

Đổi version: tăng tag (`v0.1.1`, `v0.2.0`, ...). Tag đã tồn tại thì xóa rồi push lại, hoặc dùng tag mới.

Hoặc vào **Actions → Build Release → Run workflow** để build thủ công (chỉ có Artifacts, **không** tạo Releases page).

Mỗi lần push lên `main` cũng tự chạy build (Artifacts only, không tạo Release).

### macOS: cảnh báo “Apple could not verify…”

Bản build từ GitHub / build local **chưa code-sign & notarize** nên macOS (Gatekeeper) chặn lần mở đầu. **Không phải virus** — chỉ là app chưa được Apple xác minh danh tính developer.

**Cách mở (chọn một):**

1. **Control + click** (hoặc chuột phải) vào `TrayLink.app` → **Open** → **Open** lần nữa  
2. **System Settings → Privacy & Security** → cuộn xuống → **Open Anyway** (sau khi thử mở app lần đầu)  
3. Terminal (bỏ cờ quarantine nếu tải từ browser/GitHub):

```bash
xattr -cr /Applications/TrayLink.app
```

Thay đường dẫn nếu app nằm chỗ khác (vd. `release/macos/TrayLink.app`).

**Phân phối chính thức (tùy chọn):** cần [Apple Developer Program](https://developer.apple.com/programs/) ($99/năm), ký app bằng Developer ID, rồi notarize qua `notarytool`.

## HTTP API

Base URL: `http://<IP-LAN>:8765` (IP hiển thị trong Dashboard)

### GET /status

```bash
curl http://192.168.1.x:8765/status
```

Response:

```json
{ "online": true, "version": "0.1.24", "port": 8765, "https_port": 8766, "lan_ip": "192.168.1.x", "endpoints": ["/status", "...", "/monitor-status"] }
```

### Monitor — Claude, Codex, System metrics

Bật từng loại trong Dashboard → **Settings** (Claude usage, Codex usage, System metrics). Chỉ các mục đã bật mới có dữ liệu trong response.

| Endpoint | Mô tả |
|----------|--------|
| `GET/POST /claude-usage` | Quota Claude Code (5 giờ / 7 ngày) |
| `GET/POST /codex-usage` | Quota Codex |
| `GET/POST /system-metrics` | CPU, RAM, disk, mạng, nhiệt, quạt… |
| `GET/POST /monitor-status` | **Gộp cả ba** trong một JSON |

```bash
# Một lần gọi — trả về claude + codex + system (theo toggle Settings)
curl http://192.168.1.x:8765/monitor-status

# Hoặc POST (khi GET bị tắt)
curl -X POST http://192.168.1.x:8765/monitor-status
```

Response mẫu (`/monitor-status`):

```json
{
  "online": true,
  "version": "0.1.24",
  "updated_at": "2026-06-25T...",
  "claude": { "enabled": true, "ok": true, "session_5h": { ... }, "weekly_7d": { ... } },
  "codex": { "enabled": true, "ok": true, "session_5h": { ... }, "weekly_7d": { ... } },
  "system": { "enabled": true, "ok": true, "cpu": { ... }, "memory": { ... } }
}
```

### Remote Deck (giao diện điện thoại)

Mở trên trình duyệt điện thoại/tablet cùng Wi‑Fi (xem ảnh minh họa ở [Hướng dẫn sử dụng](#hướng-dẫn-sử-dụng)).

**HTTPS (khuyến nghị — giữ màn hình sáng):**

```
https://192.168.1.x:8766/remote
```

**HTTP (mở app, không giữ màn hình sáng):**

```
http://192.168.1.x:8765/remote
```

Hoặc `http://192.168.1.x:8765/` — hiển thị grid icon các app và lệnh đã cấu hình. Chạm icon để mở app hoặc chạy lệnh trên PC.

Trên trang Remote: nút **toàn màn hình** (góc phải) và nút **giữ màn hình sáng** (Wake Lock — chỉ hoạt động qua **HTTPS** vì trình duyệt yêu cầu secure context).

Copy link **HTTPS** và quét QR trong Dashboard → **Overview** → **Remote Deck — HTTPS**.

**Vượt cảnh báo chứng chỉ lần đầu** (xem ảnh từng bước): [Vượt cảnh báo HTTPS](#vượt-cảnh-báo-https-chứng-chỉ-tự-ký) · [Hướng dẫn đầy đủ](docs/allow-https/README.md)

Nếu bật token: thêm `?token=<token>` vào URL, hoặc nhập token khi trang yêu cầu.

Trong Dashboard → tab **Remote Deck**: kéo thả trên khung preview để sắp xếp, click icon để đổi hình tùy chỉnh.

API hỗ trợ:
- `GET /api/deck` — danh sách app/lệnh và phím tắt (JSON)
- `GET /send-hotkey` · `POST /send-hotkey` — gửi phím tắt app (xem [chi tiết](#getpost-send-hotkey--gửi-phím-tắt-app))
- `GET /api/icons/{key}` — icon app (PNG)
- `POST /api/upload` — upload file từ điện thoại (multipart, field `file`; tùy chọn `open=1` để mở file sau khi lưu)

**Upload file từ điện thoại:**

Trên trang Remote Deck, nhấn nút **upload** (góc trái) → chọn ảnh/video/tài liệu → file được lưu vào `Downloads/TrayLink` trên PC. Giới hạn **5 GB**/file. Hỗ trợ: png, jpg, mp4, mov, pdf, docx, zip, …

```bash
curl -X POST https://192.168.1.x:8766/api/upload \
  -F "file=@/path/on/phone/photo.jpg" \
  -F "open=1"
```

Khi bật token, thêm header `Authorization: Bearer <token>`.

### GET/POST /send-hotkey — Gửi phím tắt app

Gọi phím tắt đã cấu hình trong Dashboard → **Apps & Commands** → **Phím** (vd. Save, Export, Copy). TrayLink sẽ focus app rồi gửi tổ hợp phím tương ứng (hoặc chỉ mở app nếu loại hành động là **Mở app**).

| Tham số | Mô tả |
|---------|--------|
| `app` | App key (vd. `photoshop`, `chrome`) |
| `hotkey` | ID phím tắt (vd. `save`, `export`) — xem trong modal **API** của từng app hoặc JSON `/api/deck` |

**Lấy danh sách phím tắt:**

```bash
curl http://192.168.1.x:8765/api/deck
```

Mỗi app trong `items` có mảng `hotkeys` với `id`, `name`, `accelerator`, `action`.

**GET** (cần bật *Cho phép GET* trong Settings):

```bash
# Gửi phím Save trong Photoshop
http://192.168.1.x:8765/send-hotkey?app=photoshop&hotkey=save

# Có token
http://192.168.1.x:8765/send-hotkey?app=photoshop&hotkey=save&token=<token>
```

**POST:**

```bash
curl -X POST http://192.168.1.x:8765/send-hotkey \
  -H "Content-Type: application/json" \
  -d '{"app":"photoshop","hotkey":"save"}'
```

Khi bật token:

```bash
curl -X POST http://192.168.1.x:8765/send-hotkey \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"app":"photoshop","hotkey":"save"}'
```

**Response thành công:**

```json
{ "ok": true, "message": "executed hotkey 'Save' for app 'photoshop'" }
```

**Lưu ý:**

- **macOS:** cần cấp quyền **Accessibility** cho TrayLink (Dashboard → Settings) để gửi phím vào app khác.
- Phím tắt toàn cục (global shortcut) chỉ áp dụng cho hành động **Mở app** — gọi API `/send-hotkey` vẫn chạy mọi loại hành động đã cấu hình.
- Dùng được từ Stream Deck, shortcut, script Python, ESP32, đồng hồ AI… miễn cùng LAN với PC.

### GET /open-app, /open-file, /exec (khi bật trong Settings)

```bash
# Mở app
http://192.168.1.x:8765/open-app?app=chrome

# Mở app + URL (trình duyệt)
http://192.168.1.x:8765/open-app?app=chrome&url=https://google.com

# Mở file
http://192.168.1.x:8765/open-file?path=/Users/you/video.mp4

# Chạy lệnh whitelist
http://192.168.1.x:8765/exec?cmd=restart_server
```

Nếu bật token: thêm `&token=<token>` vào URL.

### POST /open-app, /open-file, /exec

```bash
curl -X POST http://192.168.1.x:8765/open-app \
  -H "Content-Type: application/json" \
  -d '{"app":"chrome"}'
```

Khi bật token, thêm header `Authorization: Bearer <token>`.

## Xác thực

Mặc định **tắt** — phù hợp LAN tin cậy. Bật trong Dashboard → **Settings → Yêu cầu token API**.

- Header: `Authorization: Bearer <token>` hoặc `X-API-Token: <token>`
- GET: query `?token=<token>`

## Cấu hình

Config lưu tại app data directory:

- **macOS:** `~/Library/Application Support/com.phamminhkha.traylink/config.json`
- **Windows:** `%APPDATA%\com.phamminhkha.traylink\config.json`
- **Linux:** `~/.config/com.phamminhkha.traylink/config.json`

Defaults: [`config/defaults.json`](config/defaults.json)

| Tuỳ chọn | Mặc định | Mô tả |
|----------|----------|--------|
| `port` | `8765` | Port API |
| `require_token` | `false` | Bắt buộc token |
| `allow_get` | `true` | Cho phép gọi API bằng link GET |
| `autostart` | `false` | Tự chạy khi boot |

## Bảo mật

- Server bind `0.0.0.0` — thiết bị trong **cùng LAN** có thể gọi API
- Chỉ mở app/command/file có trong allowlist
- `open-file` chặn path traversal và system paths
- `exec` chỉ chấp nhận command key, không chạy raw shell
- Nên bật token nếu mạng LAN không tin cậy; cân nhắc firewall cho port API

## Cross-platform

| Platform | Mở app | Mở file |
|---|---|---|
| Windows | `spawn` / `start` | `cmd /C start` |
| macOS | `open -a` / `open` | `open` |
| Linux | `spawn` / `xdg-open` | `xdg-open` |

## Cấu trúc project

```
TrayLink/
├── config/defaults.json       # Config mặc định
├── docs/                      # Screenshot & video hướng dẫn
├── src/                       # React dashboard (shadcn/ui)
└── src-tauri/src/
    ├── api/                   # Axum HTTP server
    ├── launcher/              # open_app, open_file, exec
    ├── config/                # Store & persistence
    └── tray/                  # System tray
```

## License

MIT
