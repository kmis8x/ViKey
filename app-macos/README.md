# ViKey macOS

Bộ gõ tiếng Việt cho macOS sử dụng Input Method Kit.

## Yêu cầu hệ thống

| Yêu cầu | Phiên bản |
|---------|-----------|
| macOS | 12.0+ (Monterey trở lên) |
| Chip | Apple Silicon (M1/M2/M3/M4) |
| Xcode | 14.0+ hoặc Command Line Tools |
| Rust | 1.70+ |

## Hướng dẫn Build

### Bước 1: Clone repository

```bash
git clone https://github.com/kmis8x/ViKey.git
cd ViKey
```

### Bước 2: Cài đặt Rust (nếu chưa có)

```bash
# Cài đặt Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Thêm target cho Apple Silicon
rustup target add aarch64-apple-darwin
```

### Bước 3: Cài đặt Xcode Command Line Tools

```bash
xcode-select --install
```

### Bước 4: Build

```bash
cd app-macos
chmod +x build.sh
./build.sh
```

Build script sẽ tự động:
1. Build Rust core (`libvikey_core.a`)
2. Compile Swift app
3. Tạo ViKey.app bundle
4. Code sign (development)

### Bước 5: Cài đặt Input Method

```bash
# Copy vào Input Methods
cp -r build/ViKey.app ~/Library/Input\ Methods/

# Reload SystemUIServer (để nhận diện Input Method mới)
killall -HUP SystemUIServer
```

### Bước 6: Kích hoạt ViKey

1. Mở **System Settings** (hoặc System Preferences)
2. Vào **Keyboard** → **Input Sources**
3. Click **+** (Add)
4. Tìm **Vietnamese** → Chọn **ViKey**
5. Click **Add**
6. Chọn **ViKey** từ menu input trên thanh menu

## Sử dụng

| Phím tắt | Chức năng |
|----------|-----------|
| `Ctrl+Space` | Bật/tắt gõ tiếng Việt |

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

## Cài đặt mặc định

| Tùy chọn | Mặc định |
|----------|----------|
| Kiểu gõ | Telex |
| Bỏ dấu kiểu mới | Bật (hoà thay vì hòa) |
| ESC khôi phục | Bật |
| Tự động khôi phục tiếng Anh | Bật |

## Cấu trúc dự án

```
app-macos/
├── ViKey/
│   ├── Sources/
│   │   ├── main.swift                # Entry point - IMKServer
│   │   ├── ViKeyInputController.swift # IMKit controller
│   │   ├── RustBridge.swift          # FFI wrapper cho Rust
│   │   ├── Settings.swift            # UserDefaults settings
│   │   └── ViKey-Bridging-Header.h   # C headers cho Rust FFI
│   ├── Resources/
│   │   └── Info.plist                # Bundle configuration
│   └── lib/
│       └── libvikey_core.a           # Rust static library (sau build)
├── build/
│   └── ViKey.app                     # Output (sau build)
├── build.sh                          # Build script
└── README.md
```

## Troubleshooting

### Input Method không xuất hiện trong danh sách

```bash
# Restart SystemUIServer
killall -HUP SystemUIServer

# Hoặc logout và login lại
```

### Không gõ được tiếng Việt

1. Kiểm tra đã chọn ViKey trong Input Sources (thanh menu)
2. Nhấn `Ctrl+Space` để bật/tắt
3. Kiểm tra quyền Accessibility trong System Settings → Privacy & Security

### Build lỗi "target not found"

```bash
# Cài đặt Rust target cho Apple Silicon
rustup target add aarch64-apple-darwin

# Kiểm tra đã cài đặt
rustup target list --installed | grep aarch64
```

### Build lỗi "swiftc not found"

```bash
# Cài đặt Xcode Command Line Tools
xcode-select --install

# Kiểm tra
swiftc --version
```

### Build lỗi "library not found"

```bash
# Xóa build cũ và build lại
rm -rf build/
rm -rf ViKey/lib/
./build.sh
```

## Gỡ cài đặt

```bash
# Xóa app
rm -rf ~/Library/Input\ Methods/ViKey.app

# Xóa preferences
defaults delete vn.vikey.inputmethod

# Reload
killall -HUP SystemUIServer
```

## License

BSD-3-Clause
