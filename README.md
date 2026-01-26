# ViKey - Bộ gõ tiếng Việt đa nền tảng

Bộ gõ tiếng Việt nhẹ, nhanh, chuẩn cho **Windows**, **macOS** và **Linux**.

## Tải về

| Nền tảng | Phiên bản | Tải |
|----------|-----------|-----|
| Windows 10/11 (64-bit) | v1.2.0 | [ViKey-v1.2.0-win64.zip](https://github.com/kmis8x/ViKey/releases/latest) |
| macOS (Apple Silicon) | v1.2.1 | [ViKey-macOS-arm64.zip](https://github.com/kmis8x/ViKey/releases/latest) |
| Linux x86_64 (IBus) | v1.0.0 | [ViKey-Linux-x86_64.zip](https://github.com/kmis8x/ViKey/releases/latest) |

**Không cần cài đặt!** Giải nén và chạy.

## Tính năng

- **Telex**: aa→â, ow→ơ, s→sắc, dd→đ
- **VNI**: a6→â, o7→ơ, 1→sắc, d9→đ
- **Tự động khôi phục từ tiếng Anh**: text, expect, window tự hoàn nguyên
- **ESC khôi phục**: Nhấn ESC để khôi phục văn bản gốc (úẻ → user)
- **Bỏ dấu kiểu mới/cũ**: hoà vs hòa
- **Phím tắt tuỳ chỉnh**: Mặc định Ctrl+Space
- **Hiệu suất**: <1ms độ trễ, <5MB RAM

## Hướng dẫn cài đặt

### Windows

1. Tải `ViKey-v1.2.0-win64.zip`
2. Giải nén vào thư mục bất kỳ
3. Chạy `ViKey.exe`

### macOS

1. Tải `ViKey-macOS-arm64.zip`
2. Giải nén và copy `ViKey.app` vào `~/Library/Input Methods/`
3. Vào **System Settings → Keyboard → Input Sources → Add → Vietnamese → ViKey**

Chi tiết: [app-macos/README.md](app-macos/README.md)

### Linux

1. Tải `ViKey-Linux-x86_64.zip`
2. Giải nén và chạy `sudo ./install.sh`
3. Restart IBus: `ibus restart`
4. Thêm ViKey trong **Settings → Keyboard → Input Sources**

Chi tiết: [app-linux/README.md](app-linux/README.md)

## Sử dụng

### Phím tắt

| Phím | Chức năng |
|------|-----------|
| `Ctrl+Space` | Bật/tắt gõ tiếng Việt |
| `ESC` | Khôi phục văn bản gốc |

### Bảng phím

| Telex | VNI | Kết quả |
|-------|-----|---------|
| `as` | `a1` | á |
| `af` | `a2` | à |
| `ar` | `a3` | ả |
| `ax` | `a4` | ã |
| `aj` | `a5` | ạ |
| `aa` | `a6` | â |
| `ee` | `e6` | ê |
| `oo` | `o6` | ô |
| `ow` | `o7` | ơ |
| `uw` | `u7` | ư |
| `aw` | `a8` | ă |
| `dd` | `d9` | đ |

## Cấu trúc dự án

```
ViKey/
├── core/           # Rust engine (cross-platform)
├── app-native/     # Windows GUI (C++ Win32)
├── app-macos/      # macOS (Swift + Input Method Kit)
├── app-linux/      # Linux (Rust + IBus)
└── scripts/        # Build scripts
```

## Build từ Source

### Windows

```powershell
.\scripts\build-core.ps1
.\scripts\build-native.ps1
```

### macOS

```bash
cd app-macos
./build.sh
```

### Linux

```bash
cd app-linux
./build.sh
```

## Tác giả

**Trần Công Sinh** (tcsinh89@gmail.com)

Built with [Claude Code](https://claude.ai/code)

## License

BSD-3-Clause (xem file [LICENSE](LICENSE))

### Miễn phí
- Sử dụng cá nhân, giáo dục, phi thương mại

### Thương mại
- Cần xin phép: tcsinh89@gmail.com
