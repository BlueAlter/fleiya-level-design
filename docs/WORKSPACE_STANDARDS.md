# Workspace Layout & Spacing Standards

Pedoman standardisasi tata letak (layout), skala spasi (spacing), ukuran kontrol, dan pengujian geometri (*rect testing*) untuk seluruh workspace editor di Level Craft (Enums, Tileset, Entities, Layers, Worlds).

Dokumen ini berfungsi sebagai acuan agar setiap pembuatan atau perombakan workspace baru menghasilkan tampilan yang konsisten, rapi, dan terkunci dari regresi visual.

---

## 1. Terminologi & Konsep Kunci

| Konsep | Penjelasan |
|---|---|
| **Design Tokens** | Nilai-nilai visual terpusat (angka margin, padding, tinggi kontrol) yang didefinisikan sebagai konstanta bernama (*named constants*), bukan *magic numbers* yang tersebar di kode. |
| **Spacing System** | Aturan skala jarak (padding, margin, item spacing) yang seragam di seluruh panel editor untuk menciptakan ritme visual (*visual rhythm*) yang seimbang. |
| **Layout Contract** | Kesepakatan batas geometri antarmuka (misal: panel kiri 260px, padding selalu 12px, kontrol input selalu mulai pada kolom `x` yang sama). |
| **Rect / Geometry Locking** | Pengujian headless otomatis (`Shape::Rect` dan `Shape::Text`) dengan toleransi presisi (0.5 px) untuk mencegah pergeseran layout akibat pembaruan kode atau perubahan gaya egui. |

---

## 2. Tabel Standar Metrik (Tokens)

Setiap workspace harus menggunakan nilai metrik yang konsisten:

| Token | Nilai | Penggunaan |
|---|---|---|
| `WORKSPACE_PADDING` | `12.0 px` | Padding / inner margin pada keempat sisi (kiri, kanan, atas, bawah) di panel kiri maupun panel detail. |
| `LEFT_PANEL_WIDTH` | `260.0 px` | Lebar tetap (*exact width*) panel daftar samping kiri. |
| `CONTROL_HEIGHT_COMPACT` | `28.0 px` | Tinggi elemen kompak: toolbar atas, kotak pencarian (*search box*), strip sub-tab, dan tombol footer. |
| `CONTROL_HEIGHT_FORM` | `32.0 px` | Tinggi elemen form standar: input teks (*TextEdit*), dropdown (*ComboBox*), chip tag, baris item/values, dan tombol "+ Add Value". |
| `FORM_LABEL_COL_W` | `110.0 px` | Lebar kolom label form. Memastikan kontrol input pertama pada setiap baris form mulai pada posisi horizontal (`x`) yang persis sama. |
| `ICON_BUTTON_SIZE_COMPACT` | `28.0 x 28.0 px` | Tombol ikon tier kompak (contoh: tombol kebab "⋮" di toolbar atas). |
| `ICON_BUTTON_SIZE_FORM` | `32.0 x 32.0 px` | Tombol ikon tier form (contoh: tombol export, tag "+", tag "⌄", tombol hapus / trash). |

---

## 3. Aturan & Solusi Masalah Layout Egui

### A. Jebakan Padding Tombol Ikon (*Icon Button Padding Trap*)
* **Masalah**: Tema proyek (`theme::apply`) mengatur `button_padding = vec2(10.0, 4.0)`. Jika membuat tombol ikon dengan `ui.add_sized([28.0, 28.0], egui::Button::new(icon))`, egui akan menambahkan padding horizontal `2 * 10.0 = 20.0 px` ke lebar ikon (14 px), sehingga tombol melar menjadi **34x28 px** alih-alih 28x28 px. Hal ini menyebabkan baris toolbar melebar dan mendorong lebar panel utama.
* **Solusi**: Selalu bungkus tombol ikon bujur sangkar ke dalam `ui.scope` dengan `button_padding = Vec2::ZERO`, atau alokasikan dengan `allocate_exact_size` + `paint_icon`:
  ```rust
  let icon_btn = ui
      .scope(|ui| {
          ui.spacing_mut().button_padding = egui::Vec2::ZERO;
          ui.add_sized(
              [ICON_BUTTON_SIZE_COMPACT, ICON_BUTTON_SIZE_COMPACT],
              egui::Button::new(icon.rich(ui.ctx(), 14.0, t.ink_strong))
                  .fill(t.sunken)
                  .stroke(t.hairline())
                  .corner_radius(CornerRadius::same(3)),
          )
      })
      .inner;
  ```

---

### B. Gutter Scrollbar Mengurangi Lebar Baris (*Scroll Gutter Gaps*)
* **Masalah**: `ScrollStyle::thin()` memiliki default `floating_allocated_width = 6.0`. Saat daftar panjang dimasukkan ke dalam `ScrollArea::vertical()`, kartu di dalam daftar menjadi 6 px lebih sempit daripada input search di atasnya (muncul celah kanan 18 px alih-alih 12 px).
* **Solusi**: Setel `floating_allocated_width = 0.0` pada gaya scrollbar daftar tersebut:
  ```rust
  let mut scroll = egui::style::ScrollStyle::thin();
  scroll.floating_allocated_width = 0.0;
  ui.style_mut().spacing.scroll = scroll;
  ```
  *Catatan: Pastikan ikon paling kanan pada kartu (misal tombol kebab atau drag handle) berjarak minimal 10–11 px dari tepi kartu agar tidak tertutup indikator scrollbar tipis yang mengambang.*

---

### C. Kolom Label Form (*Form Label Column*)
* **Masalah**: Jika setiap kontrol diletakkan langsung setelah `ui.label(...)`, kontrol akan mulai pada posisi `x` yang berbeda-beda karena panjang teks label ("Enum Identifier" vs "Tileset:" vs "Tags:") tidak sama.
* **Solusi**: Alokasikan lebar tetap `FORM_LABEL_COL_W` (110.0 px) sebelum kontrol pertama:
  ```rust
  fn card_label(ui: &mut Ui, text: &str, t: &Tokens) {
      let (rect, _) = ui.allocate_exact_size(
          vec2(FORM_LABEL_COL_W, CONTROL_HEIGHT_FORM),
          Sense::hover(),
      );
      let mut child = ui.new_child(egui::UiBuilder::new().max_rect(rect));
      child.with_layout(Layout::left_to_right(Align::Center), |ui| {
          ui.label(RichText::new(text).strong().color(t.ink_strong));
      });
  }
  ```

---

### D. Keselarasan Sub-Tabs dan Toolbar Atas
* **Masalah**: Tombol aksi utama di panel kiri (misal "+ New Enum") memiliki tinggi 28 px dan mulai pada `y = WORKSPACE_PADDING (12 px)`. Jika sub-tab di panel kanan menggunakan padding default yang menghasilkan tinggi 22 px pada `y = 15 px`, header kedua panel tampak tidak sejajar.
* **Solusi**: Strip sub-tab wajib memiliki tinggi `CONTROL_HEIGHT_COMPACT (28 px)` dan mulai pada `y = WORKSPACE_PADDING (12 px)` agar tepat sejajar dengan panel samping.

---

## 4. Pengujian Geometri / Rect Test (*Pixel Locking*)

Setiap workspace baru wajib dilengkapi tes integrasi headless (misalnya `tests/<workspace>_workspace_rects_test.rs`) untuk mengunci tata letak:

1. **Setup Headless**:
   - Pasang font dan tema proyek:
     ```rust
     let ctx = Context::default();
     lc_canvas::theme::install_fonts(&ctx);
     lc_canvas::theme::apply(&ctx, lc_canvas::theme::ThemeChoice::Dark);
     ```
   - Render pada resolusi target standar (1366x768) dan kondisi scrolling (misal: 12 item pada tinggi 500 px).
2. **Koleksi Shape**:
   - Ekstrak seluruh `Shape::Rect` dan `Shape::Text` dari `FullOutput.shapes`.
   - Bersihkan tekstur setelah frame selesai: `output.textures_delta.clear()`.
3. **Daftar Assert Wajib (Toleransi 0.5 px)**:
   - Panel kiri terisi tepat pada `0.0 .. LEFT_PANEL_WIDTH`.
   - Seluruh elemen kolom kiri (tombol atas, search box, kartu baris) membentang tepat pada `12.0 .. (LEFT_PANEL_WIDTH - 12.0)`.
   - Tombol-tombol ikon bujur sangkar tepat 1:1 sesuai tiernya (28x28 atau 32x32).
   - Tepi kartu detail sama dengan batas area konten (`(LEFT_PANEL_WIDTH + P) .. (SCREEN_WIDTH - P)`).
   - Seluruh baris kontrol form mulai pada posisi horizontal (`x`) yang identik.
   - Tombol footer dan tombol toolbar kanan merapat ke tepi kanan konten (`SCREEN_WIDTH - P`).
