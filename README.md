# ViKey - Nhẹ, nhanh, chuẩn Việt

Bộ gõ tiếng Việt cho Windows 10/11.

## Tính năng

- **Telex**: aa→â, ow→ơ, s→sắc, dd→đ
- **VNI**: a6→â, o7→ơ, 1→sắc, d9→đ
- **Tự động khôi phục từ tiếng Anh**: text, expect, window tự hoàn nguyên
- **ESC khôi phục**: Nhấn ESC để khôi phục văn bản gốc (úẻ → user)
- **Bỏ dấu kiểu mới/cũ**: hoà vs hòa
- **Phím tắt tuỳ chỉnh**: Mặc định Ctrl+Space
- **Bỏ dấu tự do**: Bỏ qua kiểm tra vị trí dấu
- **Phím tắt ngoặc**: [ → ơ, ] → ư
- **Bỏ qua W**: w giữ nguyên ở đầu từ
- **Chế độ chậm**: Dành cho terminal/Claude Code
- **System tray**: Giao diện tối giản, chạy nền
- **Lưu cài đặt**: Registry-based, giữ nguyên sau khởi động lại
- **Khởi động cùng Windows**: Tuỳ chọn tự động chạy
- **Khởi động ẩn**: Tuỳ chọn không hiện thông báo
- **Hiệu suất**: <1ms độ trễ, <5MB RAM

## Cài đặt

### Tải về (Khuyến nghị)

1. Tải [`ViKey-v1.2.0-win64.zip`](https://github.com/kmis8x/ViKey/releases/latest) từ Releases
2. Giải nén vào thư mục bất kỳ
3. Chạy `ViKey.exe`

**Không cần cài .NET!** Tổng dung lượng: ~1MB

### Build từ Source

Yêu cầu:
- Windows 10/11 (64-bit)
- Visual Studio 2022 Build Tools
- Rust toolchain (MSVC target)

```powershell
# Clone
git clone https://github.com/kmis8x/ViKey.git
cd ViKey

# Build Rust core
.\scripts\build-core.ps1

# Build native app
.\scripts\build-native.ps1

# Chạy
.\app-native\bin\Release\ViKey.exe
```

## Sử dụng

### Phím tắt

| Phím | Chức năng |
|------|-----------|
| `Ctrl+Space` | Bật/tắt gõ tiếng Việt (tuỳ chỉnh được) |
| `ESC` | Khôi phục văn bản gốc (khi bật) |

### System Tray

- **Double-click**: Bật/tắt gõ tiếng Việt
- **Click phải**: Mở menu
  - Bật/Tắt
  - Chuyển Telex/VNI
  - Cài đặt
  - Giới thiệu
  - Thoát

### Bảng phím tắt

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

### Hoàn nguyên bằng phím đôi

Gõ phím modifier hai lần để hoàn nguyên:
- `aa` → â (chuyển đổi)
- `aaa` → aa (hoàn nguyên)

## Cấu trúc dự án

```
ViKey/
├── core/               # Rust engine xử lý tiếng Việt
│   └── src/            # Source code engine
├── app-native/         # C++ Win32 GUI (~1MB)
│   └── src/            # Source code native
└── scripts/            # Scripts build và đóng gói
    ├── build-core.ps1  # Build Rust core
    ├── build-native.ps1# Build native app
    └── package-native.ps1
```

## Hạn chế

- **UWP/Store apps**: Có thể không hoạt động trong ứng dụng sandbox
- **Ứng dụng Admin**: Cần chạy ViKey với quyền admin
- **Antivirus**: Có thể cảnh báo do keyboard hook (false positive)

## Tác giả

**Trần Công Sinh** (tcsinh89@gmail.com)

Built with [Claude Code](https://claude.ai/code) + [ClaudeKit](https://claudekit.cc/)

## Chính sách sử dụng

### Miễn phí
- Sử dụng cá nhân
- Mục đích giáo dục
- Dự án phi thương mại
- Dự án mã nguồn mở (với attribution)

### Thương mại
Sử dụng thương mại cần xin phép trước từ tác giả:
- Bán phần mềm hoặc sản phẩm phái sinh
- Tích hợp vào sản phẩm/dịch vụ thương mại
- Sử dụng trong ứng dụng tạo doanh thu
- Phân phối như một phần của gói thương mại

**Liên hệ:** tcsinh89@gmail.com

### Yêu cầu
Mọi bản phân phối phải giữ nguyên:
- File LICENSE
- Thông tin bản quyền trong mã nguồn
- Attribution trong tài liệu/About dialog

## License

BSD-3-Clause (xem file [LICENSE](LICENSE) để biết chi tiết)
