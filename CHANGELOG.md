# Changelog

Các thay đổi đáng chú ý của dự án.

## [1.3.1] - 2026-01-28

### Thêm mới
- Cho phép phụ âm ngoại lai F, J, W, Z (tuỳ chọn trong cài đặt)
- Double-click tray icon mở Cài đặt thay vì toggle chế độ gõ

### Sửa lỗi
- Sửa lỗi gõ tắt thừa space/dấu gạch (sử dụng clipboard mode cho shortcuts dài)
- Cải thiện timing cho shortcut expansion

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
