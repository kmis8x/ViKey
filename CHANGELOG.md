# Changelog

Các thay đổi đáng chú ý của dự án.

## [1.3.4] - 2026-02-03

### Thêm mới
- **Tự động cập nhật**: Nút "Cập nhật ngay" tải và cài đặt bản mới tự động
- **Giao diện cải tiến**: Thêm khung "Tuỳ chọn" gom nhóm các checkbox
- **Sắp xếp phím chuyển**: Thứ tự mới Ctrl → Shift → Alt → Win → [Key]

### Sửa lỗi
- **Tray icon**: Sửa lỗi không hiển thị icon, load từ embedded resources
- **Ẩn khi khởi động**: Làm rõ hành vi - bỏ tick = hiện Settings, tick = hiện Toast
- **Nút Lấy app**: Sửa text bị cắt trong dialog Loại trừ ứng dụng
- **Cột ListView**: Sửa cột thứ 3 trống trong dialog Gõ tắt

## [1.3.3] - 2026-02-03

### Sửa lỗi
- **Gõ tắt văn bản dài**: Sửa lỗi gõ tắt có văn bản thay thế dài (>15 ký tự) bị lỗi spacing
- Tự động sử dụng clipboard mode cho văn bản thay thế dài để đảm bảo độ tin cậy

## [1.3.2] - 2026-02-03

### Windows
- **Dark mode**: Tự động chuyển đổi giao diện sáng/tối theo hệ thống
- **Giao diện EVKey-style**: Bố cục 2 cột checkbox, các nút tiện ích ở dưới cùng
- **Chống mở nhiều cửa sổ**: Single-instance protection khi click nhanh tray icon
- **Tuỳ chọn bật/tắt gõ tắt**: Checkbox "Cho phép gõ tắt" trong cài đặt
- **Cửa sổ Gõ tắt**: Chỉ 2 cột (Tắt | Thay thế), các nút trên cùng hàng
- Xuất/nhập gõ tắt riêng (Export/Import Shortcuts)
- Hỗ trợ DPI awareness cho màn hình độ phân giải cao

### Core
- Thêm `shortcuts_enabled` setting và `ime_shortcuts_enabled()` FFI
- Cập nhật version lên 1.3.2

### macOS
- Cập nhật version lên 1.3.2
- Thêm `shortcutsEnabled` setting với RustBridge integration

### Linux
- Cập nhật version lên 1.3.2
- Thêm `shortcuts_enabled` setting với JSON serialization

## [1.3.1] - 2026-01-29

### Windows
- Cho phép phụ âm ngoại lai F, J, W, Z (tuỳ chọn trong cài đặt)
- Double-click tray icon mở Cài đặt thay vì toggle chế độ gõ
- Sửa lỗi gõ tắt thừa space/dấu gạch (sử dụng clipboard mode cho shortcuts dài)
- Cải thiện timing cho shortcut expansion
- Sửa lỗi Settings dialog không hiển thị trên taskbar

### macOS
- Thêm hỗ trợ phụ âm ngoại lai (f,j,w,z) trong Settings
- Thêm hỗ trợ gõ tắt (shortcuts) lưu trong UserDefaults
- Cập nhật Settings.swift để đồng bộ với Rust engine

### Linux
- Thêm module settings.rs lưu cài đặt vào ~/.config/vikey/config.json
- Hỗ trợ đầy đủ các tuỳ chọn: modern_tone, esc_restore, english_auto_restore, auto_capitalize, free_tone, skip_w_shortcut, bracket_shortcut, allow_foreign_consonants
- Hỗ trợ gõ tắt (shortcuts) trong config file
- Cập nhật version lên 1.3.1

## [1.3.0] - 2026-01-28

### Thêm mới
- Hỗ trợ gõ tắt với dấu gạch ngang (ví dụ: `--danger` → `--dangerously-skip-permissions`)
- Icon tray mới V.ico/E.ico với viền trắng, nổi bật trên taskbar tối
- Thêm logo vào dialog Giới thiệu
- Chế độ clipboard cho ứng dụng không hỗ trợ SendInput
- Nhớ encoding theo ứng dụng (Smart Switch)
- Chuyển mã Unicode/VNI/TCVN3
- Loại trừ ứng dụng tự động

### Sửa lỗi
- Sửa lỗi gõ tắt không nhận dấu `-`
- Cải thiện timing cho terminal (tăng delay backspace/text)
- Sửa căn chỉnh UI dialog Giới thiệu

### Thay đổi
- Tách module app_detector và encoding_converter
- Cập nhật bố cục dialog cài đặt

## [1.2.0] - 2026-01-23

### Thêm mới
- Phím tắt tuỳ chỉnh (không chỉ Ctrl+Space)
- Tuỳ chọn khởi động ẩn (silent startup)
- Logo và icon mới

### Sửa lỗi
- Sửa lỗi UTF-8 trong tooltip và menu
- Sửa lỗi hiển thị phím Space trong cài đặt
- Sửa lỗi bố cục dialog cài đặt

### Thay đổi
- Đổi tên từ VietnameseIME sang ViKey

## [1.1.0] - 2026-01-22

### Sửa lỗi
- Cải thiện độ ổn định
- Sửa lỗi nhỏ

## [1.0.0] - Phiên bản đầu tiên

### Thêm mới
- Kiểu gõ Telex (aa→â, ow→ơ, v.v.)
- Kiểu gõ VNI (a6→â, o7→ơ, v.v.)
- Tự động khôi phục từ tiếng Anh
- ESC khôi phục văn bản gốc
- Bỏ dấu kiểu mới/cũ
- Phím tắt toàn cục (Ctrl+Space)
- Bỏ dấu tự do
- Phím tắt ngoặc ([ → ơ, ] → ư)
- Bỏ qua W
- Chế độ chậm cho terminal
- System tray
- Lưu cài đặt (Registry)
- Khởi động cùng Windows
