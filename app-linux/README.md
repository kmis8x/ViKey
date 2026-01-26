# ViKey Linux

Bộ gõ tiếng Việt cho Linux sử dụng IBus framework.

## Yêu cầu hệ thống

| Yêu cầu | Phiên bản |
|---------|-----------|
| Linux | Ubuntu 20.04+, Fedora 35+, Arch Linux |
| IBus | 1.5.0+ |
| Rust | 1.70+ |

## Cài đặt dependencies

### Ubuntu/Debian

```bash
sudo apt update
sudo apt install build-essential pkg-config
sudo apt install libibus-1.0-dev libglib2.0-dev
```

### Fedora

```bash
sudo dnf install gcc pkg-config
sudo dnf install ibus-devel glib2-devel
```

### Arch Linux

```bash
sudo pacman -S base-devel pkgconf
sudo pacman -S ibus glib2
```

## Hướng dẫn Build

### Bước 1: Clone repository

```bash
git clone https://github.com/kmis8x/ViKey.git
cd ViKey
```

### Bước 2: Cài đặt Rust (nếu chưa có)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Bước 3: Build

```bash
cd app-linux
chmod +x build.sh
./build.sh
```

### Bước 4: Cài đặt

#### Cài đặt system-wide (khuyến nghị)

```bash
sudo cp install/usr/lib/ibus/ibus-engine-vikey /usr/lib/ibus/
sudo cp install/usr/share/ibus/component/vikey.xml /usr/share/ibus/component/
ibus restart
```

#### Cài đặt cho user hiện tại

```bash
mkdir -p ~/.local/lib/ibus
mkdir -p ~/.local/share/ibus/component

cp install/usr/lib/ibus/ibus-engine-vikey ~/.local/lib/ibus/

# Sửa đường dẫn trong XML
sed 's|/usr/lib/ibus|'"$HOME"'/.local/lib/ibus|g' data/vikey.xml > ~/.local/share/ibus/component/vikey.xml

ibus restart
```

### Bước 5: Kích hoạt ViKey

#### GNOME (Ubuntu, Fedora)

1. Mở **Settings** → **Keyboard**
2. Click **Input Sources** → **+**
3. Chọn **Vietnamese** → **ViKey**
4. Click **Add**

#### KDE Plasma

1. Mở **System Settings** → **Input Devices** → **Virtual Keyboard**
2. Chọn **IBus**
3. Vào **IBus Preferences** → **Input Method**
4. Click **Add** → Tìm **ViKey**

#### Hoặc dùng command line

```bash
# Mở IBus Preferences
ibus-setup

# Hoặc restart IBus
ibus restart
```

## Sử dụng

| Phím tắt | Chức năng |
|----------|-----------|
| `Super+Space` | Chuyển đổi input method (GNOME) |
| `Ctrl+Space` | Bật/tắt ViKey |

### Telex

| Gõ | Kết quả |
|----|---------|
| `aa` | â |
| `aw` | ă |
| `ee` | ê |
| `oo` | ô |
| `ow` | ơ |
| `uw` | ư |
| `dd` | đ |
| `s` | sắc (á) |
| `f` | huyền (à) |
| `r` | hỏi (ả) |
| `x` | ngã (ã) |
| `j` | nặng (ạ) |

### VNI

| Gõ | Kết quả |
|----|---------|
| `a6` | â |
| `a8` | ă |
| `e6` | ê |
| `o6` | ô |
| `o7` | ơ |
| `u7` | ư |
| `d9` | đ |
| `1` | sắc |
| `2` | huyền |
| `3` | hỏi |
| `4` | ngã |
| `5` | nặng |

## Cấu trúc dự án

```
app-linux/
├── src/
│   ├── main.rs           # IBus engine entry point
│   └── keymap.rs         # Linux → macOS keycode mapping
├── data/
│   └── vikey.xml         # IBus component descriptor
├── lib/
│   └── libvikey_core.a   # Rust static library (sau build)
├── install/              # Installation files (sau build)
├── build.sh              # Build script
├── Cargo.toml
└── README.md
```

## Troubleshooting

### ViKey không xuất hiện trong danh sách Input Method

```bash
# Restart IBus daemon
ibus restart

# Hoặc kill và khởi động lại
ibus exit
ibus-daemon -drx
```

### Không gõ được tiếng Việt

1. Kiểm tra đã chọn ViKey trong Input Sources
2. Nhấn `Super+Space` hoặc `Ctrl+Space` để chuyển đổi
3. Kiểm tra IBus daemon đang chạy: `pgrep -x ibus-daemon`

### Build lỗi "ibus-1.0 not found"

```bash
# Ubuntu/Debian
sudo apt install libibus-1.0-dev

# Fedora
sudo dnf install ibus-devel

# Arch
sudo pacman -S ibus
```

### Build lỗi "glib-2.0 not found"

```bash
# Ubuntu/Debian
sudo apt install libglib2.0-dev

# Fedora
sudo dnf install glib2-devel
```

## Gỡ cài đặt

### System-wide

```bash
sudo rm /usr/lib/ibus/ibus-engine-vikey
sudo rm /usr/share/ibus/component/vikey.xml
ibus restart
```

### User installation

```bash
rm ~/.local/lib/ibus/ibus-engine-vikey
rm ~/.local/share/ibus/component/vikey.xml
ibus restart
```

## Alternative: Fcitx5

Nếu bạn dùng Fcitx5 thay vì IBus, xem hướng dẫn tại:
https://github.com/kmis8x/ViKey/wiki/Fcitx5

## License

BSD-3-Clause
